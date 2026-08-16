//! Tonehole module for CADSD
//!
//! This module implements tonehole models for wind instrument simulation.
//! Toneholes are modeled as side branches in the transmission line with
//! viscothermal losses and radiation impedance.

use crate::sim::{Segment, za, viscothermal_loss_params, AcousticConstants};
use nalgebra::Matrix2;
use num_complex::Complex;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// A single tonehole in the bore.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tonehole {
    /// Position along the bore (mm)
    pub x: f64,
    /// Hole diameter (mm)
    pub diameter: f64,
    /// Hole depth / wall thickness (mm)
    pub depth: f64,
    /// Whether the hole is open (true) or closed (false)
    pub is_open: bool,
    /// Key coverage fraction (0.0 = fully open, 1.0 = fully closed)
    pub coverage: f64,
}

impl Tonehole {
    /// Create a new tonehole
    pub fn new(x: f64, diameter: f64, depth: f64, is_open: bool) -> Self {
        Self { x, diameter, depth, is_open, coverage: 0.0 }
    }

    /// Create a tonehole with partial key coverage
    pub fn with_coverage(x: f64, diameter: f64, depth: f64, coverage: f64) -> Self {
        Self { x, diameter, depth, is_open: coverage < 0.5, coverage: coverage.clamp(0.0, 1.0) }
    }

    /// Effective open area fraction (0.0 = fully closed, 1.0 = fully open)
    pub fn effective_area_fraction(&self) -> f64 {
        if self.is_open {
            1.0 - self.coverage * 0.8
        } else {
            0.0
        }
    }

    /// Convert tonehole dimensions to a short side-branch segment.
    ///
    /// The side branch is modeled as a small tube section with its own
    /// radiation impedance at the open end. Viscothermal losses are included.
    pub fn to_segment(&self, _constants: &AcousticConstants) -> Segment {
        let d_mm = self.diameter;
        let l_mm = self.depth;
        let x0 = 0.0;
        let x1 = l_mm;
        let d0 = d_mm / 1000.0;
        let d1 = d_mm / 1000.0;
        Segment::new(x0 / 1000.0, x1 / 1000.0, d0, d1)
    }

    /// Compute the shunt impedance of an open tonehole at a given frequency.
    ///
    /// Uses a side-branch transmission line model with viscothermal losses
    /// and radiation impedance at the open end. The effective area is reduced
    /// by key coverage.
    pub fn open_impedance(&self, freq_hz: f64, constants: &AcousticConstants) -> Complex<f64> {
        let r = (self.diameter / 1000.0) / 2.0;
        let l = self.depth / 1000.0;
        let area_fraction = self.effective_area_fraction();

        if area_fraction < 1e-6 {
            return Complex::new(1e15, 0.0);
        }

        let z_rad = za(freq_hz, r.max(1e-6), constants.rho, constants.c, constants.nu);
        let omega = 2.0 * PI * freq_hz;
        let k = omega / constants.c;
        let zc = constants.rho * constants.c / (PI * r * r);

        let (k_complex, zc_lossy) = if freq_hz > 10.0 {
            let seg = self.to_segment(constants);
            let (tw, zcw) = viscothermal_loss_params(&seg, freq_hz, constants);
            (tw, Complex::new(zcw.re, zcw.im))
        } else {
            (Complex::new(k, 0.0), Complex::new(zc, 0.0))
        };

        let cos_kl = (k_complex * l).cos();
        let sin_kl = (k_complex * l).sin();

        let a = cos_kl;
        let b = Complex::new(0.0, zc_lossy.re) * sin_kl;
        let c = Complex::new(0.0, 1.0 / zc_lossy.re.max(1e-15)) * sin_kl;
        let d = cos_kl;

        let numerator = a * z_rad + b;
        let denominator = c * z_rad + d;

        if denominator.norm() < 1e-15 {
            Complex::new(1e15, 0.0)
        } else {
            (numerator / denominator) / area_fraction
        }
    }

    /// Compute the impedance of a closed tonehole at a given frequency.
    ///
    /// A closed tonehole acts as a lumped compliance (inverse of stiffness)
    /// with viscothermal losses in the trapped air volume.
    pub fn closed_impedance(&self, freq_hz: f64, constants: &AcousticConstants) -> Complex<f64> {
        let r = (self.diameter / 1000.0) / 2.0;
        let volume = PI * r * r * (self.depth / 1000.0);
        let omega = 2.0 * PI * freq_hz;
        let bulk_modulus = constants.rho * constants.c * constants.c;
        let compliance = volume / bulk_modulus;

        let z_compliance = Complex::new(0.0, -1.0 / (omega * compliance));

        if freq_hz > 10.0 && self.depth > 0.1 {
            let seg = self.to_segment(constants);
            let (_, zcw) = viscothermal_loss_params(&seg, freq_hz, constants);
            let zc = Complex::new(zcw.re, zcw.im);
            let k_complex = Complex::new(omega / constants.c, zcw.im / (2.0 * constants.rho * constants.c));
            let l = self.depth / 1000.0;
            let cos_kl = (k_complex * l).cos();
            let sin_kl = (k_complex * l).sin();
            let shunt = Matrix2::new(
                cos_kl,
                Complex::new(0.0, zc.re) * sin_kl,
                Complex::new(0.0, 1.0 / zc.re.max(1e-15)) * sin_kl,
                cos_kl,
            ) * z_compliance;
            let result = shunt[(0, 0)] - shunt[(0, 1)] * shunt[(1, 0)] / shunt[(1, 1)];
            if result.norm() > 1e-15 { result } else { z_compliance }
        } else {
            z_compliance
        }
    }
}

/// Collection of toneholes in a bore.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToneholeSet {
    pub holes: Vec<Tonehole>,
}

impl ToneholeSet {
    pub fn new() -> Self {
        Self { holes: Vec::new() }
    }

    pub fn add(&mut self, hole: Tonehole) {
        self.holes.push(hole);
    }

    pub fn is_empty(&self) -> bool {
        self.holes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.holes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::AcousticConstants;

    #[test]
    fn test_tonehole_creation() {
        let hole = Tonehole::new(500.0, 12.0, 5.0, true);
        assert_eq!(hole.x, 500.0);
        assert_eq!(hole.diameter, 12.0);
        assert_eq!(hole.depth, 5.0);
        assert!(hole.is_open);
        assert_eq!(hole.coverage, 0.0);
    }

    #[test]
    fn test_tonehole_coverage() {
        let hole = Tonehole::with_coverage(500.0, 12.0, 5.0, 0.5);
        assert!(!hole.is_open);
        assert_eq!(hole.coverage, 0.5);
        assert!((hole.effective_area_fraction() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_open_impedance() {
        let hole = Tonehole::new(500.0, 12.0, 5.0, true);
        let constants = AcousticConstants::for_temperature(20.0);
        let z = hole.open_impedance(440.0, &constants);
        assert!(z.norm() > 0.0, "Open tonehole impedance should be positive, got {:?}", z);
    }

    #[test]
    fn test_closed_impedance() {
        let hole = Tonehole::new(500.0, 12.0, 5.0, false);
        let constants = AcousticConstants::for_temperature(20.0);
        let z = hole.closed_impedance(440.0, &constants);
        assert!(z.im < 0.0, "Closed tonehole impedance should be capacitive, got {:?}", z);
    }

    #[test]
    fn test_tonehole_set() {
        let mut set = ToneholeSet::new();
        assert!(set.is_empty());
        set.add(Tonehole::new(200.0, 10.0, 4.0, true));
        assert_eq!(set.len(), 1);
    }
}
