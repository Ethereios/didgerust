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

use crate::geo::Geo;
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
        let wavelength = C / (sample_rate as f64 / 1024.0); // Approximate wavelength
        (self.length / wavelength).ceil() as usize
    }
}

/// Waveguide engine - cascade of waveguide cells representing bore geometry
pub struct WaveguideEngine {
    /// Vector of waveguide cells
    pub cells: Vec<WaveguideCell>,
    /// Total bore length in meters
    pub total_length: f64,
    /// Number of segments
    pub n_segments: usize,
}

impl WaveguideEngine {
    /// Create a waveguide engine from a bore geometry
    pub fn from_geo(geo: &Geo) -> Self {
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
        }
    }

    /// Compute the transfer function at a given frequency
    pub fn transfer_function(&self, freq_hz: f64) -> Complex64 {
        let omega = 2.0 * PI * freq_hz;
        let k = omega / C; // Wave number

        // Start with identity matrix
        let mut total_matrix = [[Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
                                 [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)]];

        for cell in &self.cells {
            // Propagation matrix for the segment
            let kl = k * cell.length;
            let cos_kl = Complex64::new(kl.cos(), 0.0);
            let sin_kl = Complex64::new(0.0, kl.sin());

            // Characteristic impedance (use geometric mean for segment)
            let zc = (cell.zc0 * cell.zc1).sqrt();

            // Transfer matrix for uniform tube section
            let segment_matrix = [
                [cos_kl, sin_kl * zc],
                [sin_kl / zc, cos_kl],
            ];

            // Matrix multiplication: total = segment * total
            let new_matrix = [
                [
                    total_matrix[0][0] * segment_matrix[0][0] + total_matrix[0][1] * segment_matrix[1][0],
                    total_matrix[0][0] * segment_matrix[0][1] + total_matrix[0][1] * segment_matrix[1][1],
                ],
                [
                    total_matrix[1][0] * segment_matrix[0][0] + total_matrix[1][1] * segment_matrix[1][0],
                    total_matrix[1][0] * segment_matrix[0][1] + total_matrix[1][1] * segment_matrix[1][1],
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

    /// Compute impedance spectrum at multiple frequencies
    pub fn impedance_spectrum(&self, freqs: &[f64]) -> Vec<Complex64> {
        freqs.iter().map(|&f| self.transfer_function(f)).collect()
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
    use crate::geo::Geo;

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
        assert_eq!(gen.next(), 2);
        assert_eq!(gen.next(), 3);
        assert_eq!(gen.next(), 5);
        assert_eq!(gen.next(), 7);
    }
}