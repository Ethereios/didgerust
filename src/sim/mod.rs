//! Simulation module for CADSD – transmission line model, impedance calculation, and utilities.

use nalgebra::Matrix2;
use num_complex::Complex;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Physical constants (air at 20 °C, 101 kPa)
const RHO: f64 = 1.225; // kg/m³ (air density)
const C: f64 = 343.0; // m/s (speed of sound)

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
pub fn create_segments_from_geo(geo: &Vec<[f64; 2]>) -> Vec<Segment> {
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

/// Radiation impedance at the open end of a tube.
/// This implementation follows the classical Levine‑Schwinger approximation.
pub fn za(z: Complex<f64>, r: f64) -> Complex<f64> {
    let z_rad = Complex::new(RHO * C / (2.0 * PI * r), 0.0);
    z + z_rad
}

/// The core CADSD impedance calculation for a single frequency.
/// Returns the complex input impedance at the mouthpiece.
pub fn cadsd_ze(segments: &[Segment], freq_hz: f64) -> Complex<f64> {
    let omega = 2.0 * PI * freq_hz;
    let mut m_total = Matrix2::identity();
    for seg in segments {
        let k = omega / C;
        let cos_kl = (k * seg.l).cos();
        let j_sin_kl = Complex::new(0.0, (k * seg.l).sin());
        let zc = seg.r0;
        let t = Matrix2::new(
            Complex::new(cos_kl, 0.0),
            Complex::new(0.0, zc) * j_sin_kl,
            Complex::new(0.0, 1.0 / zc) * j_sin_kl,
            Complex::new(cos_kl, 0.0),
        );
        m_total = ap(&m_total, &t);
    }
    let last = segments.last().expect("at least one segment");
    let r_last = (last.d1 / 2.0).max(1e-6);
    let z_open = za(Complex::new(0.0, 0.0), r_last);
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
            SimulationStrategy::Tlm => compute_impedance_spectrum(&self.segments, freqs),
            SimulationStrategy::Waveguide => self.waveguide_impedance(freqs),
            SimulationStrategy::ComplexImpedance => self.complex_impedance(freqs),
        }
    }
    
    pub fn peaks(&self, freqs: &[f64]) -> Vec<(usize, f64, f64)> {
        let spectrum = self.impedance(freqs);
        find_peaks(freqs, &spectrum)
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
        
        let geo = crate::geo::Geo::new(geo_points);
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
