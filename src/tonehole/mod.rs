//! Tonehole module for CADSD
//!
//! This module implements tonehole models for wind instrument simulation.
//! Toneholes are modeled as side branches in the transmission line.

use crate::sim::{Segment, za, AcousticConstants};
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
}

impl Tonehole {
    /// Create a new tonehole
    pub fn new(x: f64, diameter: f64, depth: f64, is_open: bool) -> Self {
        Self { x, diameter, depth, is_open }
    }

    /// Convert tonehole dimensions to a short side-branch segment.
    ///
    /// The side branch is modeled as a small tube section with its own
    /// radiation impedance at the open end.
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
    /// An open tonehole acts as a side branch with low impedance.
    /// The impedance is dominated by the radiation impedance of the hole.
    pub fn open_impedance(&self, freq_hz: f64, constants: &AcousticConstants) -> Complex<f64> {
        let r = (self.diameter / 1000.0) / 2.0;
        let z_rad = za(freq_hz, r.max(1e-6), constants.rho, constants.c, constants.nu);
        let l = self.depth / 1000.0;
        let omega = 2.0 * PI * freq_hz;
        let k = omega / constants.c;
        let cos_kl = (k * l).cos();
        let sin_kl = (k * l).sin();
        let zc = constants.rho * constants.c / (PI * r * r);
        let a = cos_kl;
        let b = Complex::new(0.0, zc) * sin_kl;
        let c = Complex::new(0.0, 1.0 / zc) * sin_kl;
        let d = cos_kl;
        
        // Debug
        println!("open_impedance: r={}, z_rad={:?}, l={}, k={}, cos_kl={}, sin_kl={}, zc={}, a={}, b={:?}, c={:?}, d={}", 
            r, z_rad, l, k, cos_kl, sin_kl, zc, a, b, c, d);
        
        let numerator = a * z_rad + b;
        let denominator = c * z_rad + d;
        println!("numerator={:?}, denominator={:?}", numerator, denominator);
        
        (numerator) / (denominator)
    }

    /// Compute the impedance of a closed tonehole at a given frequency.
    ///
    /// A closed tonehole acts as a lumped compliance (inverse of stiffness).
    pub fn closed_impedance(&self, freq_hz: f64, constants: &AcousticConstants) -> Complex<f64> {
        let r = (self.diameter / 1000.0) / 2.0;
        let volume = PI * r * r * (self.depth / 1000.0);
        let omega = 2.0 * PI * freq_hz;
        let bulk_modulus = constants.rho * constants.c * constants.c;
        let compliance = volume / bulk_modulus;
        Complex::new(0.0, -1.0 / (omega * compliance))
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
    }

#[test]
    fn test_open_impedance() {
        let hole = Tonehole::new(500.0, 12.0, 5.0, true);
        let constants = AcousticConstants {
            rho: 1.225,  // kg/m³
            c: 343.0,    // m/s
            nu: 1.51e-5, // mPa·s
            temperature_c: 20.0
        };
        let z = hole.open_impedance(440.0, &constants);
        println!("test_open_impedance: z = {:?}", z);
        assert!(z.re > 0.0 || z.im > 0.0);
    }

    #[test]
    fn test_closed_impedance() {
        let hole = Tonehole::new(500.0, 12.0, 5.0, false);
        let constants = AcousticConstants {
            rho: 1.225,
            c: 343.0,
            nu: 1.51e-5,
            temperature_c: 20.0
        };
        let z = hole.closed_impedance(440.0, &constants);
        println!("test_closed_impedance: z = {:?}", z);
        assert!(z.im < 0.0);
    }

    #[test]
    fn test_tonehole_set() {
        let mut set = ToneholeSet::new();
        assert!(set.is_empty());
        set.add(Tonehole::new(200.0, 10.0, 4.0, true));
        assert_eq!(set.len(), 1);
    }
}
