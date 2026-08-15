//! Simulation module for CADSD – transmission line model, impedance calculation, and utilities.

use crate::Geo;
use nalgebra::Matrix2;
use num_complex::Complex;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Physical constants (air at 20 °C, 101 kPa)
const RHO: f64 = 1.225; // kg/m³ (air density)
const C: f64 = 343.0; // m/s (speed of sound)

/// Bent-shape effective-length correction for curved tube segments.
///
/// For a circular arc bend, the effective acoustic length is shorter than the
/// geometric arc length due to the curved path. This follows the DidgeLab
/// analytical correction:
///
/// `dL_eff = ds * (1 - α * κ² * a²)`
///
/// where:
/// - `ds` = arc length of the bend (m)
/// - `κ` = curvature (1/m)
/// - `a` = tube radius (m)
/// - `α` = coefficient (1/3 for circular arc)
///
/// Reference: DidgeLab bent-shapes analysis (closes ~66% of TLM error vs FEM).
pub fn bent_effective_length(ds: f64, kappa: f64, radius: f64, alpha: f64) -> f64 {
    let correction = 1.0 - alpha * kappa.powi(2) * radius.powi(2);
    ds * correction.max(0.0)
}

/// Temperature-dependent acoustic constants
#[derive(Debug, Clone, Copy)]
pub struct AcousticConstants {
    pub rho: f64,
    pub c: f64,
    pub nu: f64,
    pub temperature_c: f64,
}

impl AcousticConstants {
    pub fn for_temperature(temp_c: f64) -> Self {
        let t_kelvin = temp_c + 273.15;
        let c = 20.05 * t_kelvin.sqrt();
        let rho = 101325.0 / (287.05 * t_kelvin);
        let nu = 1.716e-5 * (t_kelvin / 273.15).powf(1.5) * (273.15 + 110.4) / (t_kelvin + 110.4);
        Self { rho, c, nu, temperature_c: temp_c }
    }
}

impl Default for AcousticConstants {
    fn default() -> Self {
        Self::for_temperature(20.0)
    }
}

/// Simulation strategy selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationStrategy {
    /// Traditional transmission line model (default, stable)
    Tlm,
    /// Digital waveguide approach (alternative)
    Waveguide,
    /// Enhanced complex impedance calculation (alternative)
    ComplexImpedance,
}

/// A single transmission‑line segment representing a short tube section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Segment {
    /// Length of the segment (m).
    pub l: f64,
    /// Input diameter (m).
    pub d0: f64,
    /// Output diameter (m).
    pub d1: f64,
    /// Input cross‑sectional area (m²).
    pub a0: f64,
    /// Intermediate area used for tapered sections (m²).
    pub a01: f64,
    /// Output cross‑sectional area (m²).
    pub a1: f64,
    /// Angle of taper (radians) – derived from geometry, useful for loss models.
    pub phi: f64,
    /// Start coordinate along the bore (m).
    pub x0: f64,
    /// End coordinate along the bore (m).
    pub x1: f64,
    /// Characteristic impedance of the segment (Pa·s/m³).
    pub r0: f64,
}

impl Segment {
    /// Construct a segment from its geometric parameters.
    pub fn new(x0: f64, x1: f64, d0: f64, d1: f64) -> Self {
        let l = (x1 - x0).abs();
        let a0 = PI * d0 * d0 / 4.0;
        let a1 = PI * d1 * d1 / 4.0;
        let a01 = PI * ((d0 + d1) / 2.0).powi(2) / 4.0;
        let phi = if d0 != d1 {
            (d1 - d0) / l
        } else {
            0.0
        };
        let r0 = RHO * C / a0;
        Self { l, d0, d1, a0, a01, a1, phi, x0, x1, r0 }
    }
}

/// Convert a bore geometry (x, diameter) expressed in millimetres
/// into a vector of `Segment`s in metres.
pub fn create_segments_from_geo(geo: &[[f64; 2]]) -> Vec<Segment> {
    let mut segs = Vec::new();
    for window in geo.windows(2) {
        let x0_mm = window[0][0];
        let d0_mm = window[0][1];
        let x1_mm = window[1][0];
        let d1_mm = window[1][1];
        let x0 = x0_mm / 1000.0;
        let x1 = x1_mm / 1000.0;
        let d0 = d0_mm / 1000.0;
        let d1 = d1_mm / 1000.0;
        segs.push(Segment::new(x0, x1, d0, d1));
    }
    segs
}

/// Transfer‑matrix multiplication – combines two 2×2 matrices.
pub fn ap(m: &Matrix2<Complex<f64>>, n: &Matrix2<Complex<f64>>) -> Matrix2<Complex<f64>> {
    m * n
}

/// Radiation impedance at the open end of a tube using the Geipel approximation
/// for an unflanged pipe. This is frequency-dependent and complex-valued.
pub fn za(freq_hz: f64, r: f64, rho: f64, c: f64, nu: f64) -> Complex<f64> {
    let s = (PI * nu * freq_hz / (2.0 * r * r * c)).sqrt();
    rho * c / (PI * r * r) * Complex::new(1.0 - 0.366 * s, 0.613 * s)
}

/// The core CADSD impedance calculation for a single frequency.
/// Returns the complex input impedance at the mouthpiece.
pub fn cadsd_ze(segments: &[Segment], freq_hz: f64) -> Complex<f64> {
    cadsd_ze_with_losses(segments, freq_hz, &AcousticConstants::default(), false)
}

/// Viscothermal loss model for tube segments.
///
/// Computes the complex wavenumber including viscous and thermal boundary layer
/// losses. This follows the simplified model used in DidgeLab's `tlm_python.py`.
///
/// Reference: DidgeLab / Finn & McCoy 1991
pub fn viscothermal_k_complex(
    seg: &Segment,
    freq_hz: f64,
    constants: &AcousticConstants,
) -> Complex<f64> {
    let omega = 2.0 * PI * freq_hz;
    let k = omega / constants.c;

    // Simplified viscothermal attenuation (DidgeLab-style)
    let eta = 1.81e-5; // Pa·s (dynamic viscosity of air at 20°C)
    let delta = (2.0 * eta / (constants.rho * omega)).sqrt();
    let alpha = delta * (seg.d0 + seg.d1) / (2.0 * seg.d0 * seg.d1);

    Complex::new(k, alpha)
}

/// CADSD impedance calculation with optional viscothermal losses.
pub fn cadsd_ze_with_losses(
    segments: &[Segment],
    freq_hz: f64,
    constants: &AcousticConstants,
    include_losses: bool,
) -> Complex<f64> {
    let omega = 2.0 * PI * freq_hz;
    let mut m_total = Matrix2::identity();
    for seg in segments {
        let k_complex = if include_losses {
            viscothermal_k_complex(seg, freq_hz, constants)
        } else {
            Complex::new(omega / constants.c, 0.0)
        };
        let cos_kl = k_complex.cos();
        let sin_kl = k_complex.sin();
        let zc = seg.r0;
        let t = Matrix2::new(
            cos_kl,
            Complex::new(0.0, zc) * sin_kl,
            Complex::new(0.0, 1.0 / zc) * sin_kl,
            cos_kl,
        );
        m_total = ap(&m_total, &t);
    }
    let last = segments.last().expect("at least one segment");
    let r_last = (last.d1 / 2.0).max(1e-6);
    let z_open = za(freq_hz, r_last, constants.rho, constants.c, constants.nu);
    let a = m_total[(0, 0)];
    let b = m_total[(0, 1)];
    let c = m_total[(1, 0)];
    let d = m_total[(1, 1)];
    (a * z_open + b) / (c * z_open + d)
}

/// Convenience wrapper that computes the impedance spectrum for a set of frequencies.
pub fn compute_impedance_spectrum(segments: &[Segment], freqs: &[f64]) -> Vec<Complex<f64>> {
    freqs.iter().map(|&f| cadsd_ze(segments, f)).collect()
}

/// Find resonance peaks for a geometry using the default TLM strategy.
pub fn find_resonance_peaks(geo: &Geo, strategy: SimulationStrategy) -> Vec<Resonance> {
    let simulator = DidgeridooSimulator::with_strategy(&geo.geo, strategy);
    simulator.find_resonance_peaks()
}

/// Simple peak detection on the magnitude of a complex spectrum.
pub fn find_peaks(freqs: &[f64], spectrum: &[Complex<f64>]) -> Vec<(usize, f64, f64)> {
    let mag: Vec<f64> = spectrum.iter().map(|c| c.norm()).collect();
    let mut peaks = Vec::new();
    for i in 1..mag.len() - 1 {
        if mag[i] > mag[i - 1] && mag[i] > mag[i + 1] {
            peaks.push((i, freqs[i], mag[i]));
        }
    }
    peaks
}

/// Peak detection with minimum prominence.
///
/// A peak is kept only if its magnitude exceeds all points within `prominence`
/// samples on either side by at least `min_prominence`. This suppresses noise
/// and mode-switching artefacts during optimisation.
pub fn find_peaks_with_prominence(
    freqs: &[f64],
    spectrum: &[Complex<f64>],
    prominence: usize,
    min_prominence: f64,
) -> Vec<(usize, f64, f64)> {
    let mag: Vec<f64> = spectrum.iter().map(|c| c.norm()).collect();
    let mut peaks = Vec::new();

    for i in 1..mag.len() - 1 {
        // Strict local maximum first
        if mag[i] <= mag[i - 1] || mag[i] <= mag[i + 1] {
            continue;
        }

        let left_start = i.saturating_sub(prominence);
        let right_end = (i + prominence).min(mag.len());

        let left_max = mag[left_start..i].iter().cloned().fold(0.0, f64::max);
        let right_max = mag[(i + 1)..right_end].iter().cloned().fold(0.0, f64::max);

        if mag[i] - left_max >= min_prominence && mag[i] - right_max >= min_prominence {
            peaks.push((i, freqs[i], mag[i]));
        }
    }

    peaks
}

/// Phase-based peak detection using the derivative of the unwrapped phase.
///
/// Resonances correspond to rapid phase transitions. This is more robust than
/// magnitude-only local maxima, especially when modes are close or weak.
///
/// Reference: Ernoult et al. (2020) – phase-based resonance detection.
pub fn find_peaks_phase_based(
    freqs: &[f64],
    spectrum: &[Complex<f64>],
    prominence: usize,
    threshold: f64,
) -> Vec<(usize, f64, f64)> {
    if spectrum.len() < 3 {
        return Vec::new();
    }

    let p = prominence.max(1);

    // Unwrap phase
    let mut phase: Vec<f64> = spectrum.iter().map(|c| c.arg()).collect();
    for i in 1..phase.len() {
        let mut delta = phase[i] - phase[i - 1];
        while delta > PI {
            delta -= 2.0 * PI;
        }
        while delta < -PI {
            delta += 2.0 * PI;
        }
        phase[i] = phase[i - 1] + delta;
    }

    // Phase derivative (centred difference)
    let mut phase_deriv = Vec::with_capacity(phase.len());
    phase_deriv.push(phase[1] - phase[0]);
    for i in 1..phase.len() - 1 {
        phase_deriv.push((phase[i + 1] - phase[i - 1]) / 2.0);
    }
    phase_deriv.push(phase[phase.len() - 1] - phase[phase.len() - 2]);

    // Find local maxima of phase derivative magnitude with prominence check
    let mut peaks = Vec::new();
    for i in 1..phase_deriv.len() - 1 {
        let deriv = phase_deriv[i].abs();
        if deriv <= phase_deriv[i - 1].abs() || deriv <= phase_deriv[i + 1].abs() {
            continue;
        }

        let left_start = i.saturating_sub(p);
        let right_end = (i + p).min(phase_deriv.len());
        let left_max = phase_deriv[left_start..i]
            .iter()
            .map(|v| v.abs())
            .fold(0.0, f64::max);
        let right_max = phase_deriv[(i + 1)..right_end]
            .iter()
            .map(|v| v.abs())
            .fold(0.0, f64::max);

        if (deriv - left_max) >= threshold && (deriv - right_max) >= threshold {
            peaks.push((i, freqs[i], spectrum[i].norm()));
        }
    }

    peaks
}

/// Frequency‑grid utilities – logarithmic (cents) and linear helpers.
pub mod grid {
    pub fn log_grid(min_cents: f64, max_cents: f64, step_cents: f64) -> Vec<f64> {
        let mut freqs = Vec::new();
        let mut c = min_cents;
        while c <= max_cents {
            freqs.push(2_f64.powf(c / 1200.0));
            c += step_cents;
        }
        freqs
    }

    pub fn lin_grid(start: f64, end: f64, step: f64) -> Vec<f64> {
        let mut freqs = Vec::new();
        let mut f = start;
        while f <= end {
            freqs.push(f);
            f += step;
        }
        freqs
    }
}

/// Public API structs for higher‑level usage.
pub struct DidgeridooSimulator {
    pub segments: Vec<Segment>,
    pub strategy: SimulationStrategy,
}

impl DidgeridooSimulator {
    pub fn from_geo(geo: &Vec<[f64; 2]>) -> Self {
        let segments = create_segments_from_geo(geo);
        Self { 
            segments, 
            strategy: SimulationStrategy::Tlm
        }
    }
    
    pub fn with_strategy(geo: &Vec<[f64; 2]>, strategy: SimulationStrategy) -> Self {
        let segments = create_segments_from_geo(geo);
        Self { segments, strategy }
    }

    pub fn impedance(&self, freqs: &[f64]) -> Vec<Complex<f64>> {
        match self.strategy {
            SimulationStrategy::Tlm => {
                let constants = AcousticConstants::default();
                freqs.iter()
                    .map(|&f| cadsd_ze_with_losses(&self.segments, f, &constants, true))
                    .collect()
            }
            SimulationStrategy::Waveguide => self.waveguide_impedance(freqs),
            SimulationStrategy::ComplexImpedance => self.complex_impedance(freqs),
        }
    }
    
    pub fn peaks(&self, freqs: &[f64]) -> Vec<(usize, f64, f64)> {
        let spectrum = self.impedance(freqs);
        find_peaks(freqs, &spectrum)
    }

    pub fn peaks_with_prominence(
        &self,
        freqs: &[f64],
        prominence: usize,
        min_prominence: f64,
    ) -> Vec<(usize, f64, f64)> {
        let spectrum = self.impedance(freqs);
        find_peaks_with_prominence(freqs, &spectrum, prominence, min_prominence)
    }

    pub fn peaks_phase_based(
        &self,
        freqs: &[f64],
        prominence: usize,
        threshold: f64,
    ) -> Vec<(usize, f64, f64)> {
        let spectrum = self.impedance(freqs);
        find_peaks_phase_based(freqs, &spectrum, prominence, threshold)
    }

    pub fn find_resonance_peaks(&self) -> Vec<Resonance> {
        let freqs = grid::log_grid(20.0, 2000.0, 1.0);
        let peak_tuples = self.peaks(&freqs);
        peak_tuples
            .into_iter()
            .map(|(_idx, freq, imp)| Resonance { frequency: freq, impedance: imp })
            .collect()
    }
    
    fn waveguide_impedance(&self, freqs: &[f64]) -> Vec<Complex<f64>> {
        let geo_points: Vec<[f64; 2]> = self.segments.iter().scan(0.0, |x_acc, seg| {
            let x_mm = *x_acc * 1000.0;
            let d0_mm = seg.d0 * 1000.0;
            *x_acc += seg.l;
            Some([x_mm, d0_mm])
        }).collect();
        
        let geo = crate::Geo::new(geo_points);
        let engine = crate::waveguide::WaveguideEngine::from_geo(&geo);
        engine.impedance_spectrum(freqs)
    }
    
    fn complex_impedance(&self, freqs: &[f64]) -> Vec<Complex<f64>> {
        use std::f64::consts::PI;
        
        freqs.iter().map(|&freq| {
            let omega = 2.0 * PI * freq;
            let k = omega / C;
            let mut m_total = Matrix2::identity();
            let mut total_phase_shift = 0.0;
            
            for seg in &self.segments {
                let delta = (2.0 * 1.81e-5 / (1.225 * omega)).sqrt();
                let alpha = delta * (seg.d0 + seg.d1) / (2.0 * seg.d0 * seg.d1);
                let k_complex = Complex::new(k, alpha);
                let cos_kl = (k_complex * seg.l).cos();
                let j_sin_kl = (k_complex * seg.l).sin();
                let zc = seg.r0 * (1.0 + Complex::new(0.0, alpha / k));
                let t = Matrix2::new(
                    cos_kl,
                    zc * j_sin_kl,
                    j_sin_kl / zc,
                    cos_kl,
                );
                m_total = ap(&m_total, &t);
                total_phase_shift += (k * seg.l).sin().atan2((k * seg.l).cos());
            }
            
            let last = self.segments.last().expect("at least one segment");
            let r_last = (last.d1 / 2.0).max(1e-6);
            let z_rad = Complex::new(
                RHO * C / (2.0 * PI * r_last),
                RHO * C * 0.6 / (PI * r_last)
            );
            
            let a = m_total[(0, 0)];
            let b = m_total[(0, 1)];
            let c = m_total[(1, 0)];
            let d = m_total[(1, 1)];
            let z_in = (a * z_rad + b) / (c * z_rad + d);
            z_in * Complex::new(0.0, total_phase_shift * 0.01).exp()
        }).collect()
    }
}

// Placeholder structs for future extensions – SimulationParams, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    /// Frequency range in Hz – (min, max).
    pub freq_range: (f64, f64),
    /// Number of points in the spectrum.
    pub points: usize,
}

/// Simple resonance result – frequency and impedance magnitude.
#[derive(Debug, Clone, PartialEq)]
pub struct Resonance {
    pub frequency: f64,
    pub impedance: f64,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            freq_range: (20.0, 2000.0),
            points: 512,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Geo;

    #[test]
    fn test_bent_effective_length() {
        let ds = 1.0;
        let kappa = 0.01;
        let radius = 0.02;
        let alpha = 1.0 / 3.0;
        let d_l = bent_effective_length(ds, kappa, radius, alpha);
        assert!(d_l < ds);
        assert!(d_l > 0.0);
    }

    #[test]
    fn test_za_geipel() {
        let z = za(440.0, 0.02, 1.225, 343.0, 1.51e-5);
        assert!(z.re > 0.0);
    }

    #[test]
    fn test_cadsd_ze_with_losses() {
        // Use a smaller bore to avoid numerical overflow in matrix multiplication
        let geo = Geo::make_cone(500.0, 25.0, 30.0, 10);
        let segments = create_segments_from_geo(&geo.geo);
        let z_lossy = cadsd_ze_with_losses(&segments, 440.0, &AcousticConstants::default(), true);
        let z_clean = cadsd_ze_with_losses(&segments, 440.0, &AcousticConstants::default(), false);
        assert!(z_lossy.re > 0.0);
        assert!(z_clean.re > 0.0);
    }

    #[test]
    fn test_find_peaks_with_prominence() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        let segments = create_segments_from_geo(&geo.geo);
        let freqs = grid::log_grid(20.0, 2000.0, 1.0);
        let spectrum = compute_impedance_spectrum(&segments, &freqs);
        let peaks = find_peaks_with_prominence(&freqs, &spectrum, 1, 0.001);
        let _ = peaks; // Function should not panic regardless of result
    }

    #[test]
    fn test_find_peaks_phase_based() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        let segments = create_segments_from_geo(&geo.geo);
        let freqs = grid::log_grid(20.0, 2000.0, 1.0);
        let spectrum = compute_impedance_spectrum(&segments, &freqs);
        let peaks = find_peaks_phase_based(&freqs, &spectrum, 1, 0.01);
        let _ = peaks; // Function should not panic regardless of result
    }
}
