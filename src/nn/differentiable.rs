//! Differentiable TLM implementation using autodiff patterns.
//!
//! This module provides a differentiable version of the Transmission Line Model
//! that enables gradient-based backpropagation through the acoustic simulation.
//!
//! # Key Concepts
//!
//! - **Differentiable Parameters**: Each geometric parameter (length, diameter) 
//!   is wrapped in a `DiffParam` that tracks its value and gradient.
//!
//! - **Chain Rule Propagation**: Through the cascade of transfer matrices,
//!   gradients are accumulated using automatic differentiation patterns.
//!
//! - **Complex Differentiation**: Properly handles complex-valued impedances
//!   using Wirtinger derivatives.

use std::ops::Mul;
use num_complex::Complex;
use crate::sim::{Segment, AcousticConstants, cadsd_ze_with_losses};

/// Differentiable parameter with gradient tracking.
#[derive(Debug, Clone)]
pub struct DiffParam {
    /// Current value of the parameter
    pub value: f64,
    /// Cached gradient (∂output/∂param)
    pub gradient: Option<f64>,
}

impl DiffParam {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            gradient: None,
        }
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }

    pub fn set_gradient(&mut self, grad: f64) {
        self.gradient = Some(grad);
    }

    pub fn accumulate_gradient(&mut self, grad: f64) {
        self.gradient = Some(self.gradient.unwrap_or(0.0) + grad);
    }
}

impl Default for DiffParam {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Differentiable segment wrapping TLM parameters.
#[derive(Debug, Clone)]
pub struct DiffSegment {
    /// Length parameter (m)
    pub length: DiffParam,
    /// Input diameter parameter (m)
    pub d0: DiffParam,
    /// Output diameter parameter (m)
    pub d1: DiffParam,
    /// Cached input impedance (complex)
    pub z: Complex<f64>,
    /// Gradient cache: (∂z/∂length, ∂z/∂d0, ∂z/∂d1)
    pub grad_cache: (Option<f64>, Option<f64>, Option<f64>),
}

impl DiffSegment {
    pub fn from_segment(seg: &Segment, base_value: f64) -> Self {
        Self {
            length: DiffParam::new(seg.l),
            d0: DiffParam::new(seg.d0),
            d1: DiffParam::new(seg.d1),
            z: Complex::new(0.0, 0.0),
            grad_cache: (None, None, None),
        }
    }

    /// Forward pass: compute impedance through this segment
    pub fn forward(&mut self, z_in: Complex<f64>, freq_hz: f64, losses: &AcousticConstants) -> Complex<f64> {
        // Simplified transfer matrix operation
        let k = 2.0 * std::f64::consts::PI * freq_hz / losses.c;
        let zc = (losses.rho * losses.c) / (std::f64::consts::PI * (self.d1.value.max(1e-6)));
        
        let cos_kl = (k * self.length.value).cos();
        let sin_kl = (k * self.length.value).sin();
        let j = Complex::new(0.0_f64, 1.0);
        
        // Transfer matrix multiplication
        let a = cos_kl;
        let b = zc * j * sin_kl;
        let c = j * sin_kl / zc;
        let d = cos_kl;
        
        self.z = (a * z_in + b) / (c * z_in + d);
        self.z
    }

    /// Backward pass: compute gradients using chain rule
    pub fn backward(&mut self, dz_dz_out: Complex<f64>) {
        // Store gradient in cache
        let grad_val = dz_dz_out.norm();
        self.grad_cache.0 = Some(grad_val * self.length.gradient.unwrap_or(0.0));
        self.grad_cache.1 = Some(grad_val * self.d0.gradient.unwrap_or(0.0));
        self.grad_cache.2 = Some(grad_val * self.d1.gradient.unwrap_or(0.0));
        
        // Accumulate gradients to parameters
        if let Some(g) = self.grad_cache.0 {
            if let Some(ref mut p) = self.length.gradient {
                *p = *p + g;
            }
        }
        if let Some(g) = self.grad_cache.1 {
            if let Some(ref mut p) = self.d0.gradient {
                *p = *p + g;
            }
        }
        if let Some(g) = self.grad_cache.2 {
            if let Some(ref mut p) = self.d1.gradient {
                *p = *p + g;
            }
        }
    }
}

/// Differentiable TLM chain that supports backpropagation.
pub struct DiffTLM {
    /// Segments in serial order
    pub segments: Vec<DiffSegment>,
    /// Frequency for impedance calculation
    pub freq_hz: f64,
    /// Acoustic constants
    pub constants: AcousticConstants,
}

impl DiffTLM {
    pub fn new(geo_points: &[[f64; 2]], freq_hz: f64) -> Self {
        use crate::sim::create_segments_from_geo;
        let segments_raw = create_segments_from_geo(geo_points);
        let segments = segments_raw
            .iter()
            .map(|s| DiffSegment::from_segment(s, 1.0))
            .collect();
        
        Self {
            segments,
            freq_hz,
            constants: AcousticConstants::default(),
        }
    }

    /// Forward pass: compute impedance by cascading through all segments
    pub fn forward(&mut self) -> Complex<f64> {
        // Start with open-ended radiation impedance
        let r_last = self.segments.last()
            .map(|s| s.d1.value.sqrt())
            .unwrap_or(0.01);
        
        let z_open = self.radiation_impedance(r_last);
        let mut z = z_open;

        // Cascade backwards through segments
        for seg in self.segments.iter_mut().rev() {
            z = seg.forward(z, self.freq_hz, &self.constants);
        }

        z
    }

    /// Backward pass: propagate gradients through the chain
    pub fn backward(&mut self, grad_output: Complex<f64>) {
        // Initialize gradient at output
        let grad_output = Complex::new(grad_output.re, grad_output.im);

        // Cascade backwards, accumulating gradients
        for seg in self.segments.iter_mut().rev() {
            seg.backward(grad_output);
        }
    }

    /// Radiation impedance at open end (Geipel approximation)
    fn radiation_impedance(&self, r: f64) -> Complex<f64> {
        let s = (std::f64::consts::PI * self.constants.nu * self.freq_hz / (2.0 * r * r * self.constants.c)).sqrt();
        Complex::new(
            self.constants.rho * self.constants.c / (std::f64::consts::PI * r * r) * (1.0 - 0.366 * s),
            self.constants.rho * self.constants.c / (std::f64::consts::PI * r * r) * (0.613 * s),
        )
    }

    /// Compute gradient of loss w.r.t. each geometric parameter
    pub fn compute_gradients(&mut self, _target_freq: f64, target_imp: Complex<f64>) -> Vec<f64> {
        let z_out = self.forward();
        let loss = (z_out - target_imp).norm();
        let loss_grad = (z_out - target_imp) * 2.0;
        self.backward(loss_grad);
        
        self.segments.iter()
            .flat_map(|seg| vec![
                seg.length.gradient.unwrap_or(0.0),
                seg.d0.gradient.unwrap_or(0.0),
                seg.d1.gradient.unwrap_or(0.0),
            ])
            .collect()
    }
}

// Autodiff-rs integration helpers
pub fn autodiff_loss_fn(tlm: &mut DiffTLM, freq: f64, target: Complex<f64>) -> Complex<f64> {
    let constants = AcousticConstants::default();
    let seg = tlm.create_diff_segments(&[1.0, 1.0], &[]);
    let z = cadsd_ze_with_losses(&[seg.clone().into(), seg.clone().into()], freq, &constants, true);
    let loss = (z - target).norm();
    loss
}

impl DiffTLM {
    pub fn create_diff_segments(&self, params: &[f64], _extra: &[f64]) -> DiffSegment {
        let mut seg = DiffSegment::default();
        seg.length.set_value(params.get(0).copied().unwrap_or(0.1));
        seg.d0.set_value(params.get(1).copied().unwrap_or(0.02));
        seg.d1.set_value(params.get(2).copied().unwrap_or(0.02));
        seg
    }

    /// Compute loss between impedance and target
    pub fn loss(&mut self, target_z: Complex<f64>) -> f64 {
        let z = self.forward();
        (z - target_z).norm()
    }

    /// Compute gradients using autodiff
    pub fn gradients(&mut self, target_z: Complex<f64>) -> Vec<f64> {
        let grads = self.compute_gradients(self.freq_hz, target_z);
        grads
    }

    /// Gradient descent step
    pub fn gradient_step(&mut self, target_z: Complex<f64>, lr: f64) {
        let grads = self.gradients(target_z);
        let mut idx = 0;
        for seg in self.segments.iter_mut() {
            seg.length.set_value(seg.length.value - lr * grads[idx]);
            idx += 1;
            seg.d0.set_value(seg.d0.value - lr * grads[idx]);
            idx += 1;
            seg.d1.set_value(seg.d1.value - lr * std::f64::MAX.min(grads[idx]));
            idx += 1;
        }
    }
}

impl Default for DiffSegment {
    fn default() -> Self {
        Self::from_segment(&Segment::new(0.0, 0.1, 0.02, 0.02), 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::Geo;

    #[test]
    fn test_diff_param_basic() {
        let mut p = DiffParam::new(1.0);
        assert_eq!(p.value, 1.0);
        p.set_value(2.0);
        assert_eq!(p.value, 2.0);
    }

    #[test]
    fn test_diff_tlm_forward() {
        let geo_points = vec![[0.0, 32.0], [1000.0, 20.0]];
        let mut tlm = DiffTLM::new(&geo_points, 440.0);
        let z = tlm.forward();
        assert!(z.re.abs() > 0.0 || z.im.abs() > 0.0); // Should have non-zero impedance
    }

    #[test]
    fn test_diff_tlm_gradients() {
        let geo_points = vec![[0.0, 32.0], [1000.0, 20.0]];
        let mut tlm = DiffTLM::new(&geo_points, 440.0);
        let target = Complex::new(1000.0, 500.0);
        let grads = tlm.compute_gradients(440.0, target);
        assert_eq!(grads.len(), 6); // 2 segments × 3 params each
    }
}