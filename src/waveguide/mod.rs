//! Digital waveguide simulation module
//!
//! This module implements digital waveguide methods for didgeridoo acoustic simulation.
//! Waveguides model sound propagation as bidirectional delay lines, which is computationally
//! efficient for real-time applications and provides physically interpretable results.
//!
//! # Architecture
//!
//! - `WaveguideCell`: A single delay line with scattering coefficients
//! - `WaveguideEngine`: A cascade of cells representing the bore geometry
//! - `WaveguideSimulator`: High-level interface for impedance calculation

use crate::sim::AcousticConstants;
use crate::tonehole::Tonehole;
use crate::Geo;
use num_complex::Complex64;
use std::f64::consts::PI;

/// Physical constants (air at 20°C, 101 kPa)
const RHO: f64 = 1.225;
const C: f64 = 343.0;

/// A single waveguide cell representing a segment of bore
#[derive(Debug, Clone)]
pub struct WaveguideCell {
    /// Length of the segment in meters
    pub length: f64,
    /// Input diameter in meters
    pub d0: f64,
    /// Output diameter in meters
    pub d1: f64,
    /// Characteristic impedance at input
    pub zc0: f64,
    /// Characteristic impedance at output
    pub zc1: f64,
    /// Scattering coefficient for reflection at input
    pub reflection_coeff: Complex64,
    /// Scattering coefficient for transmission
    pub transmission_coeff: Complex64,
}

impl WaveguideCell {
    /// Create a new waveguide cell from geometric parameters
    pub fn new(x0: f64, x1: f64, d0_mm: f64, d1_mm: f64) -> Self {
        let length = (x1 - x0).abs() / 1000.0; // Convert mm to m
        let d0 = d0_mm / 1000.0;
        let d1 = d1_mm / 1000.0;

        // Cross-sectional areas
        let a0 = PI * d0 * d0 / 4.0;
        let a1 = PI * d1 * d1 / 4.0;

        // Characteristic impedances
        let zc0 = RHO * C / a0;
        let zc1 = RHO * C / a1;

        // Scattering coefficients for area change
        // R = (Z2 - Z1) / (Z2 + Z1) for reflection at junction
        let reflection_coeff = Complex64::new((zc1 - zc0) / (zc1 + zc0), 0.0);
        let transmission_coeff = Complex64::new(2.0 * zc1 / (zc1 + zc0), 0.0);

        Self {
            length,
            d0,
            d1,
            zc0,
            zc1,
            reflection_coeff,
            transmission_coeff,
        }
    }

    /// Compute the delay in samples for a given sampling rate
    pub fn delay_samples(&self, sample_rate: f64) -> usize {
        let wavelength = C / (sample_rate / 1024.0); // Approximate wavelength
        (self.length / wavelength).ceil() as usize
    }
}

/// Waveguide engine - cascade of waveguide cells representing bore geometry
#[derive(Debug, Clone)]
pub struct WaveguideEngine {
    /// Vector of waveguide cells
    pub cells: Vec<WaveguideCell>,
    /// Total bore length in meters
    pub total_length: f64,
    /// Number of segments
    pub n_segments: usize,
    /// Acoustic constants for tonehole calculations
    pub acoustic_constants: AcousticConstants,
    /// Toneholes positioned along the bore (x in mm)
    pub toneholes: Vec<Tonehole>,
}

    /// Internal element type for waveguide cascade
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    enum Element {
        Cell(WaveguideCell),
        ToneholePosition(f64),
    }

impl WaveguideEngine {
    /// Create a waveguide engine from a bore geometry
    pub fn from_geo(geo: &Geo) -> Self {
        Self::from_geo_with_toneholes(geo, &[], AcousticConstants::default())
    }

    /// Create a waveguide engine from a bore geometry with toneholes
    pub fn from_geo_with_toneholes(geo: &Geo, toneholes: &[Tonehole], acoustic_constants: AcousticConstants) -> Self {
        let mut cells = Vec::new();
        let mut total_length = 0.0;

        for window in geo.geo.windows(2) {
            let x0 = window[0][0];
            let x1 = window[1][0];
            let d0 = window[0][1];
            let d1 = window[1][1];

            cells.push(WaveguideCell::new(x0, x1, d0, d1));
            total_length += (x1 - x0).abs() / 1000.0; // Convert to meters
        }

        let n_segments = cells.len();
        Self {
            cells,
            total_length,
            n_segments,
            acoustic_constants,
            toneholes: toneholes.to_vec(),
        }
    }

    /// Compute the transfer function at a given frequency
    pub fn transfer_function(&self, freq_hz: f64) -> Complex64 {
        let omega = 2.0 * PI * freq_hz;
        let k = omega / C; // Wave number

        // Sort toneholes by position
        let mut sorted_toneholes: Vec<&Tonehole> = self.toneholes.iter().collect();
        sorted_toneholes.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

        // Build element list: interleave cells and tonehole shunts
        let mut elements: Vec<Element> = Vec::new();
        let mut cum_x_mm = 0.0;

        for cell in &self.cells {
            let cell_end_mm = cum_x_mm + cell.length * 1000.0;

            // Check if any tonehole falls within this cell
            while let Some(th) = sorted_toneholes.first() {
                if th.x >= cum_x_mm && th.x <= cell_end_mm {
                    elements.push(Element::ToneholePosition(cum_x_mm));
                    sorted_toneholes.remove(0);
                } else {
                    break;
                }
            }

            elements.push(Element::Cell(cell.clone()));
            cum_x_mm = cell_end_mm;
        }

        // Add any remaining toneholes at the end
        for _ in sorted_toneholes.iter() {
            elements.push(Element::ToneholePosition(cum_x_mm));
        }

        // Cascade all elements
        let mut total_matrix = [[Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
                                 [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)]];

        for elem in &elements {
            let elem_matrix = match elem {
                Element::Cell(cell) => {
                    let kl = k * cell.length;
                    let cos_kl = Complex64::new(kl.cos(), 0.0);
                    let sin_kl = Complex64::new(0.0, kl.sin());
                    let zc = (cell.zc0 * cell.zc1).sqrt();
                    [
                        [cos_kl, sin_kl * zc],
                        [sin_kl / zc, cos_kl],
                    ]
                }
                Element::ToneholePosition(_) => {
                    let y = self.tonehole_admittance(freq_hz);
                    [
                        [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
                        [y, Complex64::new(1.0, 0.0)],
                    ]
                }
            };

            let new_matrix = [
                [
                    total_matrix[0][0] * elem_matrix[0][0] + total_matrix[0][1] * elem_matrix[1][0],
                    total_matrix[0][0] * elem_matrix[0][1] + total_matrix[0][1] * elem_matrix[1][1],
                ],
                [
                    total_matrix[1][0] * elem_matrix[0][0] + total_matrix[1][1] * elem_matrix[1][0],
                    total_matrix[1][0] * elem_matrix[0][1] + total_matrix[1][1] * elem_matrix[1][1],
                ],
            ];
            total_matrix = new_matrix;
        }

        // Radiation impedance at open end
        let r_last = (self.cells.last().map(|c| c.d1).unwrap_or(0.01) / 2.0).max(1e-6);
        let z_rad = Complex64::new(RHO * C / (2.0 * PI * r_last), 0.0);

        // Input impedance: Z_in = (A * Z_rad + B) / (C * Z_rad + D)
        let a = total_matrix[0][0];
        let b = total_matrix[0][1];
        let c = total_matrix[1][0];
        let d = total_matrix[1][1];

        (a * z_rad + b) / (c * z_rad + d)
    }

    /// Compute the total tonehole admittance at a given frequency
    fn tonehole_admittance(&self, freq_hz: f64) -> Complex64 {
        let mut y_total = Complex64::new(0.0, 0.0);
        for th in &self.toneholes {
            let z = if th.is_open {
                th.open_impedance(freq_hz, &self.acoustic_constants)
            } else {
                th.closed_impedance(freq_hz, &self.acoustic_constants)
            };
            if z.norm() > 1e-15 {
                y_total += Complex64::new(1.0, 0.0) / z;
            } else {
                y_total += Complex64::new(1e15, 0.0);
            }
        }
        if self.toneholes.is_empty() {
            Complex64::new(0.0, 0.0)
        } else {
            y_total
        }
    }

    /// Compute impedance spectrum at multiple frequencies
    pub fn impedance_spectrum(&self, freqs: &[f64]) -> Vec<Complex64> {
        freqs.iter().map(|&f| self.transfer_function(f)).collect()
    }
    
    /// Compute impedance with simplified model for real-time performance
    /// Uses precomputed values and reduced complexity calculations
    pub fn impedance_spectrum_fast(&self, freqs: &[f64]) -> Vec<Complex64> {
        // Precompute frequency-independent geometric factors
        let total_length = self.total_length.max(1e-6);
        let r_last = (self.cells.last().map(|c| c.d1).unwrap_or(0.01) / 2.0).max(1e-6);
        
        freqs.iter().map(|&f| {
            let omega = 2.0 * PI * f;
            let k = omega / C;
            
            // Simplified: assume uniform tube for fast calculation (reduces matrix chain)
            let zc_avg = self.cells.iter()
                .map(|c| (c.zc0 + c.zc1) / 2.0)
                .sum::<f64>() / self.n_segments as f64;
            
            let kl = k * total_length;
            let cos_kl = kl.cos();
            let sin_kl = kl.sin();
            
            // Simplify to single segment approximation
            let z_rad = Complex64::new(RHO * C / (2.0 * PI * r_last), 0.0);
            
            let a = cos_kl;
            let b = Complex64::new(0.0, sin_kl * zc_avg);
            let c = Complex64::new(0.0, sin_kl / zc_avg);
            let d = cos_kl;
            
            (a * z_rad + b) / (c * z_rad + d)
        }).collect()
    }
}

/// High-level waveguide simulator interface
pub struct WaveguideSimulator {
    pub engine: WaveguideEngine,
}

impl WaveguideSimulator {
    /// Create a new waveguide simulator from geometry
    pub fn new(geo: &Geo) -> Self {
        Self {
            engine: WaveguideEngine::from_geo(geo),
        }
    }

    /// Compute impedance at a list of frequencies
    pub fn compute_impedance(&self, freqs: &[f64]) -> Vec<Complex64> {
        self.engine.impedance_spectrum(freqs)
    }

    /// Get the number of segments
    pub fn n_segments(&self) -> usize {
        self.engine.n_segments
    }

    /// Get total bore length in meters
    pub fn total_length(&self) -> f64 {
        self.engine.total_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evo::PrimeGenerator;
use crate::Geo;

    #[test]
    fn test_waveguide_cell_creation() {
        let cell = WaveguideCell::new(0.0, 100.0, 32.0, 40.0);
        assert!((cell.length - 0.1).abs() < 1e-10);
        assert!((cell.d0 - 0.032).abs() < 1e-10);
        assert!((cell.d1 - 0.040).abs() < 1e-10);
    }

    #[test]
    fn test_waveguide_engine_from_geo() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        let engine = WaveguideEngine::from_geo(&geo);
        assert_eq!(engine.n_segments, 20);
        assert!((engine.total_length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_transfer_function() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        let engine = WaveguideEngine::from_geo(&geo);
        let z = engine.transfer_function(440.0);
        assert!(z.re > 0.0);
    }

    #[test]
    fn test_prime_generator() {
        let mut gen = PrimeGenerator::new(1000);
assert_eq!(gen.next_prime(), 2);
    assert_eq!(gen.next_prime(), 3);
    assert_eq!(gen.next_prime(), 5);
    assert_eq!(gen.next_prime(), 7);
    }
}