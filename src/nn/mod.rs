//! Neural network integration module (behind `nn-integration` feature flag).
//!
//! This module provides placeholders and trait definitions for future
//! machine learning integration:
//! - Complex-valued neural network primitives (inspired by `renplex`)
//! - Differentiable TLM interface (inspired by `autodiff-rs`)
//! - Neural fitness predictor (MLP surrogate for resonance peaks)
//! - FDTD waveguide validation tools
//!
//! All code here is gated behind the `nn-integration` feature flag.

#![cfg(feature = "nn-integration")]

use crate::evo::Genome;

pub mod complex_activations {
    use num_complex::Complex;

    pub fn sigmoid(z: Complex<f64>) -> Complex<f64> {
        let exp_z = z.exp();
        exp_z / (Complex::new(1.0, 0.0) + exp_z)
    }

    pub fn relu(z: Complex<f64>) -> Complex<f64> {
        if z.re > 0.0 { z } else { Complex::new(0.0, 0.0) }
    }

    pub fn tanh(z: Complex<f64>) -> Complex<f64> {
        z.tanh()
    }
}

pub mod differentiable {
    use std::f64::consts::PI;
    use num_complex::Complex;
    use crate::sim::{Segment, AcousticConstants};

    #[derive(Debug, Clone)]
    pub struct DiffParam {
        pub value: f64,
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

    #[derive(Debug, Clone)]
    pub struct DiffSegment {
        pub length: DiffParam,
        pub d0: DiffParam,
        pub d1: DiffParam,
        pub z: Complex<f64>,
        pub grad_cache: (Option<f64>, Option<f64>, Option<f64>),
    }

    impl DiffSegment {
        pub fn from_segment(seg: &Segment, _base_value: f64) -> Self {
            Self {
                length: DiffParam::new(seg.l),
                d0: DiffParam::new(seg.d0),
                d1: DiffParam::new(seg.d1),
                z: Complex::new(0.0, 0.0),
                grad_cache: (None, None, None),
            }
        }

        pub fn forward(&mut self, z_in: Complex<f64>, freq_hz: f64, losses: &AcousticConstants) -> Complex<f64> {
            let k = 2.0 * PI * freq_hz / losses.c;
            let zc = (losses.rho * losses.c) / (PI * (self.d1.value.max(1e-6)));
            
            let cos_kl = (k * self.length.value).cos();
            let sin_kl = (k * self.length.value).sin();
            let j = Complex::new(0.0_f64, 1.0);
            
            let a = cos_kl;
            let b = zc * j * sin_kl;
            let c = j * sin_kl / zc;
            let d = cos_kl;
            
            self.z = (a * z_in + b) / (c * z_in + d);
            self.z
        }

        pub fn backward(&mut self, dz_dz_out: Complex<f64>) {
            let grad_val = dz_dz_out.norm();
            self.grad_cache.0 = Some(grad_val * self.length.gradient.unwrap_or(0.0));
            self.grad_cache.1 = Some(grad_val * self.d0.gradient.unwrap_or(0.0));
            self.grad_cache.2 = Some(grad_val * self.d1.gradient.unwrap_or(0.0));
            
            if let Some(g) = self.grad_cache.0 {
                if let Some(ref mut p) = self.length.gradient {
                    *p += g;
                }
            }
            if let Some(g) = self.grad_cache.1 {
                if let Some(ref mut p) = self.d0.gradient {
                    *p += g;
                }
            }
            if let Some(g) = self.grad_cache.2 {
                if let Some(ref mut p) = self.d1.gradient {
                    *p += g;
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct DiffTLM {
        pub segments: Vec<DiffSegment>,
        pub freq_hz: f64,
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

        pub fn forward(&mut self) -> Complex<f64> {
            let r_last = self.segments.last()
                .map(|s| s.d1.value.sqrt())
                .unwrap_or(0.01);
            
            let z_open = self.radiation_impedance(r_last);
            let mut z = z_open;

            for seg in self.segments.iter_mut().rev() {
                z = seg.forward(z, self.freq_hz, &self.constants);
            }

            z
        }

        pub fn backward(&mut self, grad_output: Complex<f64>) {
            for seg in self.segments.iter_mut().rev() {
                seg.backward(grad_output);
            }
        }

        fn radiation_impedance(&self, r: f64) -> Complex<f64> {
            let s = (PI * self.constants.nu * self.freq_hz / (2.0 * r * r * self.constants.c)).sqrt();
            Complex::new(
                self.constants.rho * self.constants.c / (PI * r * r) * (1.0 - 0.366 * s),
                self.constants.rho * self.constants.c / (PI * r * r) * (0.613 * s),
            )
        }

        pub fn compute_gradients(&mut self, _target_freq: f64, target_imp: Complex<f64>) -> Vec<f64> {
            let z_out = self.forward();
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

    impl Default for DiffSegment {
        fn default() -> Self {
            Self::from_segment(&Segment::new(0.0, 0.1, 0.02, 0.02), 1.0)
        }
    }
}

pub fn complex_xavier_init(fan_in: usize, fan_out: usize) -> (f64, f64) {
    let limit = (6.0 / (fan_in + fan_out) as f64).sqrt();
    (limit, limit)
}

pub struct NeuralFitnessPredictor {
    pub input_dim: usize,
    pub hidden_dim: usize,
    pub output_dim: usize,
}

impl NeuralFitnessPredictor {
    pub fn new(input_dim: usize, hidden_dim: usize, output_dim: usize) -> Self {
        Self { input_dim, hidden_dim, output_dim }
    }

    pub fn predict(&self, input: &[f64]) -> Vec<f64> {
        if input.is_empty() {
            return vec![0.0; self.output_dim];
        }
        let mean: f64 = input.iter().sum::<f64>() / input.len() as f64;
        vec![mean; self.output_dim]
    }

    pub fn estimate_fitness_from_genome<G: Genome>(&self, genome: &G) -> f64 {
        genome.loss().unwrap_or(f64::INFINITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_complex::Complex;

    #[test]
    fn test_complex_sigmoid() {
        let z = Complex::new(0.0, 0.0);
        let out = complex_activations::sigmoid(z);
        assert!((out.re - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_complex_xavier() {
        let (re, im) = complex_xavier_init(10, 5);
        assert!(re > 0.0);
        assert!(im > 0.0);
    }
}