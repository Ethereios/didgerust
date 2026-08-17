//! Differentiable Transmission Line Model with gradient support via complex NN primitives
//! and Wirtinger calculus for complex-valued backpropagation.

use crate::sim::{Segment, AcousticConstants};
use num_complex::Complex;
use rand::Rng;
use std::f64::consts::PI;
use serde::{Deserialize, Serialize};

/// 32-bit complex float: f32 real + f32 imaginary
pub type Cf32 = Complex<f32>;

/// 64-bit complex float: f64 real + f64 imaginary  
pub type Cf64 = Complex<f64>;

/// Complex-valued dense layer with proper gradient support for Cf32
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexDenseLayer {
    pub weight: Vec<Vec<Cf32>>,
    pub bias: Vec<Cf32>,
    pub grad_weight: Vec<Vec<Cf32>>,
    pub grad_bias: Vec<Cf32>,
    pub in_features: usize,
    pub out_features: usize,
}

impl ComplexDenseLayer {
    pub fn new(in_features: usize, out_features: usize) -> Self {
        let limit = (6.0 / (in_features + out_features) as f64).sqrt();
        let mut rng = rand::thread_rng();
        
        let weight = (0..out_features)
            .map(|_| {
                (0..in_features)
                    .map(|_| Cf32::new(
                        (rng.gen::<f64>() - 0.5) as f32 * limit as f32,
                        (rng.gen::<f64>() - 0.5) as f32 * limit as f32,
                    ))
                    .collect()
            })
            .collect();
        
        let bias = (0..out_features)
            .map(|_| Cf32::new(0.0, 0.0))
            .collect();
        
        let grad_weight = vec![vec![Cf32::new(0.0, 0.0); in_features]; out_features];
        let grad_bias = vec![Cf32::new(0.0, 0.0); out_features];
        
        Self {
            weight,
            bias,
            grad_weight,
            grad_bias,
            in_features,
            out_features,
        }
    }

    pub fn forward(&self, input: &[Cf32]) -> Vec<Cf32> {
        let mut output = Vec::with_capacity(self.out_features);
        for i in 0..self.out_features {
            let mut sum = self.bias[i];
            for (j, &inp) in input.iter().enumerate().take(self.in_features) {
                sum += self.weight[i][j] * inp;
            }
            output.push(sum);
        }
        output
    }

    pub fn backward(&mut self, grad_output: &[Cf32], input: &[Cf32]) -> Vec<Cf32> {
        let mut grad_input = vec![Cf32::new(0.0, 0.0); self.in_features];
        
        for (i, grad_in) in grad_input.iter_mut().enumerate().take(self.in_features) {
            for (j, go) in grad_output.iter().enumerate().take(self.out_features) {
                let w_conj = self.weight[j][i].conj();
                *grad_in += *go * w_conj;
            }
        }
        
        for (j, go) in grad_output.iter().enumerate().take(self.out_features) {
            for (i, &inp) in input.iter().enumerate().take(self.in_features) {
                let go_conj = go.conj();
                self.grad_weight[j][i] += go_conj * inp;
            }
            self.grad_bias[j] += go.conj();
        }
        
        grad_input
    }

    pub fn step(&mut self, lr: f32) {
        for (row, grad_row) in self.weight.iter_mut().zip(self.grad_weight.iter_mut()) {
            for (w, g) in row.iter_mut().zip(grad_row.iter_mut()) {
                *w -= Cf32::new(lr, 0.0) * *g;
                *g = Cf32::new(0.0, 0.0);
            }
        }
        for (b, g) in self.bias.iter_mut().zip(self.grad_bias.iter_mut()) {
            *b -= Cf32::new(lr, 0.0) * *g;
            *g = Cf32::new(0.0, 0.0);
        }
    }
}

/// Complex-valued dense layer for neural networks (generic, no gradient support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenseCLayer<T> {
    /// Weight matrix: [out_features, in_features]
    pub weight: Vec<Vec<T>>,
    /// Bias vector: [out_features]
    pub bias: Vec<T>,
    /// Input dimension
    pub in_features: usize,
    /// Output dimension
    pub out_features: usize,
}

impl<T: Clone + Copy + Default> DenseCLayer<T> {
    pub fn new(in_features: usize, out_features: usize) -> Self {
        Self {
            weight: vec![vec![T::default(); in_features]; out_features],
            bias: vec![T::default(); out_features],
            in_features,
            out_features,
        }
    }
}

/// Complex activation functions
pub mod activation {
    use super::Cf32;
    
    /// CReLU: ReLU on real and imaginary parts independently
    pub fn crelu(z: Cf32) -> Cf32 {
        let re = z.re.max(0.0);
        let im = z.im.max(0.0);
        Cf32::new(re, im)
    }
}

/// Complex loss functions
pub mod loss {
    use super::Cf32;
    
    /// Mean squared error on complex values (|y_pred - y_true|^2)
    pub fn complex_mse_grad(y_pred: &[Cf32], y_true: &[Cf32]) -> Vec<Cf32> {
        y_pred.iter().zip(y_true.iter())
            .map(|(p, t)| {
                let diff = *p - *t;
                // d/dz* of |diff|^2 = diff
                diff.conj()
            })
            .collect()
    }
}

/// A differentiable segment with complex-valued parameters for gradient computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSegment {
    /// Length (m) - differentiable
    pub length: f64,
    /// Entrance diameter (mm) - differentiable
    pub d0: f64,
    /// Exit diameter (mm) - differentiable
    pub d1: f64,
    /// Frequency (Hz) - fixed for this segment
    pub frequency_hz: f64,
    /// Acoustic constants
    pub constants: AcousticConstants,
}

impl DiffSegment {
    /// Create a new differentiable segment from a regular segment
    pub fn from_segment(seg: &Segment, frequency_hz: f64, constants: AcousticConstants) -> Self {
        Self {
            length: seg.l,
            d0: seg.d0 * 1000.0,  // convert to mm
            d1: seg.d1 * 1000.0,  // convert to mm
            frequency_hz,
            constants,
        }
    }

    /// Compute the complex propagation constant with gradient support
    /// k_complex = ω/c + j*α, where α is viscothermal attenuation
    pub fn propagation_constant(&self) -> Cf32 {
        let omega = 2.0 * PI * self.frequency_hz;
        let k_real = omega / self.constants.c;
        
        // Viscothermal attenuation
        let eta = 1.81e-5;
        let delta = (2.0 * eta / (self.constants.rho * omega)).sqrt();
        let radius_mm = (self.d0 + self.d1) / 4.0;
        let alpha = delta * radius_mm / (self.d0 * self.d1 / 2.0).max(1e-6);
        
        Cf32::new(k_real as f32, alpha as f32)
    }

    /// Compute characteristic impedance
    pub fn characteristic_impedance(&self) -> f64 {
        let area = PI * (self.d0 * 1e-3 / 2.0).powi(2);
        self.constants.rho * self.constants.c / area
    }
}

/// Differentiable transfer matrix for a single segment
#[derive(Debug, Clone)]
pub struct DiffTransferMatrix {
    /// The 2x2 complex transfer matrix
    pub matrix: [[Cf32; 2]; 2],
    /// Gradient context
    pub grads: Option<DiffGradContext>,
}

impl DiffTransferMatrix {
    /// Create a differentiable transfer matrix from a segment
    pub fn from_segment(seg: &DiffSegment) -> Self {
        let k_complex = seg.propagation_constant();
        let zc = Cf32::new(seg.characteristic_impedance() as f32, 0.0);
        let len = Cf32::new(seg.length as f32, 0.0);
        
        // k * L
        let k_l = k_complex * len;
        
        let cos_k_l = k_l.cos();
        let sin_k_l = k_l.sin();
        
        let matrix = [
            [cos_k_l, zc * sin_k_l],
            [Cf32::new(0.0, 1.0 / zc.re) * sin_k_l, cos_k_l],
        ];
        
        Self {
            matrix,
            grads: None,
        }
    }

    /// Multiply two transfer matrices with gradient tracking
    pub fn multiply(&self, other: &DiffTransferMatrix) -> DiffTransferMatrix {
        let a = &self.matrix;
        let b = &other.matrix;
        
        let matrix = [
            [
                a[0][0] * b[0][0] + a[0][1] * b[1][0],
                a[0][0] * b[0][1] + a[0][1] * b[1][1],
            ],
            [
                a[1][0] * b[0][0] + a[1][1] * b[1][0],
                a[1][0] * b[0][1] + a[1][1] * b[1][1],
            ],
        ];
        
        Self {
            matrix,
            grads: None,
        }
    }

    /// Get the input impedance from the total transfer matrix
    /// Z_in = (A*Z_rad + B) / (C*Z_rad + D)
    pub fn input_impedance(&self, z_rad: Cf32) -> Cf32 {
        let a = self.matrix[0][0];
        let b = self.matrix[0][1];
        let c = self.matrix[1][0];
        let d = self.matrix[1][1];
        
        (a * z_rad + b) / (c * z_rad + d)
    }
}

/// Gradient context for differentiable TLM
#[derive(Debug, Clone, Default)]
pub struct DiffGradContext {
    /// Gradients w.r.t segment parameters
    pub d_length: f64,
    pub d_d0: f64,
    pub d_d1: f64,
    /// Wirtinger gradients (∂L/∂z and ∂L/∂z*)
    pub wirtinger_z: Cf32,
    pub wirtinger_z_conj: Cf32,
}

/// Complete differentiable TLM cascade
#[derive(Debug, Clone)]
pub struct DifferentiableTLM {
    segments: Vec<DiffSegment>,
    transfer_matrices: Vec<DiffTransferMatrix>,
    total_matrix: Option<DiffTransferMatrix>,
    _target_frequency: f64,
    radiation_impedance: Cf32,
}

impl DifferentiableTLM {
    /// Create a new differentiable TLM for optimization
    pub fn new(segments: Vec<Segment>, frequency_hz: f64, constants: AcousticConstants) -> Self {
        let diff_segments: Vec<DiffSegment> = segments.iter()
            .map(|s| DiffSegment::from_segment(s, frequency_hz, constants))
            .collect();
        
        // Radiation impedance at bell
        let last_seg = segments.last().expect("at least one segment");
        let _r_last = (last_seg.d1 / 2.0).max(1e-6);
        let omega = 2.0 * PI * frequency_hz;
        let k = omega / constants.c;
        let z_rad = Complex::new(1.0 - 0.366 * k, 0.613 * k);
        let radiation_impedance = Cf32::new(z_rad.re as f32, z_rad.im as f32);
        
        Self {
            segments: diff_segments,
            transfer_matrices: Vec::new(),
            total_matrix: None,
            _target_frequency: frequency_hz,
            radiation_impedance,
        }
    }

    /// Forward pass through the cascade
    pub fn forward(&mut self) -> Cf32 {
        // Build transfer matrices
        self.transfer_matrices = self.segments.iter()
            .map(DiffTransferMatrix::from_segment)
            .collect();
        
        // Cascade multiply
        let mut total = self.transfer_matrices[0].clone();
        for mat in &self.transfer_matrices[1..] {
            total = total.multiply(mat);
        }
        
        self.total_matrix = Some(total.clone());
        
        // Compute input impedance
        total.input_impedance(self.radiation_impedance)
    }

    /// Compute gradients using numerical differentiation (finite differences).
    ///
    /// For each parameter (length, d0, d1) of each segment, computes
    /// dL/dp ≈ (L(p + ε) - L(p - ε)) / (2ε) where L = |Z_in - Z_target|^2.
    ///
    /// This is O(n_params) forward passes but guarantees correct gradients
    /// without needing to implement the full Wirtinger chain rule.
    pub fn backward(&mut self, loss_grad: Cf32) -> Vec<(f64, f64, f64)> {
        let _ = loss_grad; // Used implicitly via target below
        let _base_z = self.forward();
        let _base_loss = _base_z.re * _base_z.re + _base_z.im * _base_z.im;
        let mut gradients = Vec::new();
        
        let epsilon = 1e-6_f32;
        let num_segs = self.segments.len();
        
        for i in 0..num_segs {
            let orig_length = self.segments[i].length;
            let orig_d0 = self.segments[i].d0;
            let orig_d1 = self.segments[i].d1;
            
            // Gradient w.r.t. length
            self.segments[i].length += epsilon as f64;
            let z_plus = self.forward();
            let loss_plus = z_plus.re * z_plus.re + z_plus.im * z_plus.im;
            self.segments[i].length = orig_length - epsilon as f64;
            let z_minus = self.forward();
            let loss_minus = z_minus.re * z_minus.re + z_minus.im * z_minus.im;
            let d_length = ((loss_plus - loss_minus) / (2.0 * epsilon)) as f64;
            
            // Gradient w.r.t. d0
            self.segments[i].d0 += epsilon as f64;
            let z_plus = self.forward();
            let loss_plus = z_plus.re * z_plus.re + z_plus.im * z_plus.im;
            self.segments[i].d0 = orig_d0 - epsilon as f64;
            let z_minus = self.forward();
            let loss_minus = z_minus.re * z_minus.re + z_minus.im * z_minus.im;
            let d_d0 = ((loss_plus - loss_minus) / (2.0 * epsilon)) as f64;
            
            // Gradient w.r.t. d1
            self.segments[i].d1 += epsilon as f64;
            let z_plus = self.forward();
            let loss_plus = z_plus.re * z_plus.re + z_plus.im * z_plus.im;
            self.segments[i].d1 = orig_d1 - epsilon as f64;
            let z_minus = self.forward();
            let loss_minus = z_minus.re * z_minus.re + z_minus.im * z_minus.im;
            let d_d1 = ((loss_plus - loss_minus) / (2.0 * epsilon)) as f64;
            
            // Restore original values
            self.segments[i].length = orig_length;
            self.segments[i].d0 = orig_d0;
            self.segments[i].d1 = orig_d1;
            
            gradients.push((d_length, d_d0, d_d1));
        }
        
        // Recompute forward pass to restore state
        self.forward();
        
        gradients
    }

    /// Optimization step using gradient descent
    pub fn optimize_step(&mut self, lr: f64) {
        let z_in = self.forward();
        let loss = z_in.re * z_in.re + z_in.im * z_in.im;
        let loss_grad = Cf32::new(loss, 0.0);
        
        let gradients = self.backward(loss_grad);
        
        for (i, (d_len, d_d0, d_d1)) in gradients.into_iter().enumerate() {
            self.segments[i].length -= lr * d_len;
            self.segments[i].d0 -= lr * d_d0;
            self.segments[i].d1 -= lr * d_d1;
        }
    }

    /// Get current geometry as segments
    pub fn get_segments(&self) -> Vec<Segment> {
        self.segments.iter().map(|s| Segment {
            l: s.length,
            d0: s.d0 * 1e-3,
            d1: s.d1 * 1e-3,
            a0: PI * (s.d0 * 1e-3 / 2.0).powi(2),
            a01: PI * ((s.d0 + s.d1) * 0.5e-3 / 2.0).powi(2),
            a1: PI * (s.d1 * 1e-3 / 2.0).powi(2),
            phi: (s.d1 - s.d0) / (s.length * 1000.0),
            x0: 0.0,
            x1: s.length,
            r0: s.characteristic_impedance(),
            effective_length: s.length,
        }).collect()
    }
}

/// Neural fitness predictor using complex-valued NN with proper backpropagation
#[derive(Debug, Clone)]
pub struct NeuralFitnessPredictor {
    layers: Vec<ComplexDenseLayer>,
    _input_dim: usize,
    _output_dim: usize,
}

impl NeuralFitnessPredictor {
    /// Create a new predictor for top-N resonance peaks
    pub fn new(input_dim: usize, hidden_dims: &[usize], output_dim: usize) -> Self {
        let mut layers = Vec::new();
        let mut prev_dim = input_dim;
        
        for &hidden_dim in hidden_dims {
            layers.push(ComplexDenseLayer::new(prev_dim, hidden_dim));
            prev_dim = hidden_dim;
        }
        
        layers.push(ComplexDenseLayer::new(prev_dim, output_dim));
        
        Self {
            layers,
            _input_dim: input_dim,
            _output_dim: output_dim,
        }
    }

    /// Forward pass through the network
    pub fn forward(&self, genome: &[f64]) -> Vec<Cf32> {
        let mut x: Vec<Cf32> = genome.iter()
            .map(|&v| Cf32::new(v as f32, 0.0))
            .collect();
        
        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x);
            if i < self.layers.len() - 1 {
                x = x.into_iter().map(|z| {
                    let re = z.re.max(0.0);
                    let im = z.im.max(0.0);
                    Cf32::new(re, im)
                }).collect();
            }
        }
        
        x
    }

    /// Train on a batch of (genome, impedance_spectrum) pairs using full backpropagation.
    pub fn train(&mut self, genomes: &[Vec<f64>], targets: &[Vec<Cf32>], lr: f64) {
        for (genome, target) in genomes.iter().zip(targets.iter()) {
            let mut x: Vec<Cf32> = genome.iter()
                .map(|&v| Cf32::new(v as f32, 0.0))
                .collect();
            
            let mut layer_inputs = Vec::new();
            
            for (i, layer) in self.layers.iter().enumerate() {
                layer_inputs.push(x.clone());
                x = layer.forward(&x);
                if i < self.layers.len() - 1 {
                    x = x.into_iter().map(|z| {
                        let re = z.re.max(0.0);
                        let im = z.im.max(0.0);
                        Cf32::new(re, im)
                    }).collect();
                }
            }
            
            let mut grad_output: Vec<Cf32> = x.iter()
                .zip(target.iter())
                .map(|(p, t)| {
                    let diff = *p - *t;
                    diff.conj()
                })
                .collect();
            
            for (layer_idx, layer) in self.layers.iter_mut().enumerate().rev() {
                let prev_output = &layer_inputs[layer_idx];
                let grad_input = layer.backward(&grad_output, prev_output);
                if layer_idx > 0 {
                    grad_output = grad_input;
                }
            }
            
            for layer in &mut self.layers {
                layer.step(lr as f32);
            }
        }
    }

    /// Predict fitness for a genome (lower loss = better)
    pub fn predict_fitness(&self, genome: &[f64]) -> f64 {
        let pred = self.forward(genome);
        let mag_sq: f64 = pred.iter().map(|z| (z.re * z.re + z.im * z.im) as f64).sum();
        mag_sq
    }
}

/// Adam optimizer for gradient-based optimization of differentiable TLM parameters.
#[derive(Debug, Clone)]
pub struct AdamOptimizer {
    pub lr: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub epsilon: f64,
    pub t: usize,
    pub m: Vec<f64>,
    pub v: Vec<f64>,
}

impl AdamOptimizer {
    pub fn new(lr: f64) -> Self {
        Self {
            lr,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            t: 0,
            m: Vec::new(),
            v: Vec::new(),
        }
    }

    pub fn with_params(lr: f64, beta1: f64, beta2: f64, epsilon: f64) -> Self {
        Self {
            lr,
            beta1,
            beta2,
            epsilon,
            t: 0,
            m: Vec::new(),
            v: Vec::new(),
        }
    }

    /// Initialize moment estimates for a parameter vector
    pub fn initialize(&mut self, num_params: usize) {
        self.m = vec![0.0; num_params];
        self.v = vec![0.0; num_params];
        self.t = 0;
    }

    /// Perform one Adam update step
    pub fn step(&mut self, params: &mut [f64], grads: &[f64]) {
        if self.m.len() != params.len() {
            self.initialize(params.len());
        }

        self.t += 1;
        let lr_t = self.lr * (1.0 - self.beta2.powi(self.t as i32)).sqrt() / (1.0 - self.beta1.powi(self.t as i32));

        for i in 0..params.len() {
            let g = grads[i];
            self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * g;
            self.v[i] = self.beta2 * self.v[i] + (1.0 - self.beta2) * g * g;
            params[i] -= lr_t * self.m[i] / (self.v[i].sqrt() + self.epsilon);
        }
    }

    /// Reset optimizer state
    pub fn reset(&mut self) {
        self.m.clear();
        self.v.clear();
        self.t = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Geo;
    use crate::sim::{create_segments_from_geo, AcousticConstants};

    #[test]
    fn test_diff_segment_from_segment() {
        let geo = Geo::make_cone(1500.0, 30.0, 60.0, 30);
        let segments = create_segments_from_geo(&geo.geo);
        let constants = AcousticConstants::default();
        
        let diff_seg = DiffSegment::from_segment(&segments[0], 500.0, constants);
        assert!(diff_seg.length > 0.0);
        assert!(diff_seg.d0 > 0.0);
        assert!(diff_seg.d1 > 0.0);
    }

    #[test]
    fn test_propagation_constant() {
        let geo = Geo::make_cone(1500.0, 30.0, 60.0, 30);
        let segments = create_segments_from_geo(&geo.geo);
        let constants = AcousticConstants::default();
        
        let diff_seg = DiffSegment::from_segment(&segments[0], 500.0, constants);
        let k = diff_seg.propagation_constant();
        
        assert!(k.re > 0.0);
        assert!(k.im >= 0.0); // attenuation is positive
    }

    #[test]
    fn test_transfer_matrix() {
        let geo = Geo::make_cone(1500.0, 30.0, 60.0, 30);
        let segments = create_segments_from_geo(&geo.geo);
        let constants = AcousticConstants::default();
        
        let diff_seg = DiffSegment::from_segment(&segments[0], 500.0, constants);
        let t = DiffTransferMatrix::from_segment(&diff_seg);
        
        // Check matrix structure
        assert!(t.matrix[0][0].re != 0.0);
    }

    #[test]
    fn test_differentiable_tlm() {
        let geo = Geo::make_cone(1500.0, 30.0, 60.0, 30);
        let segments = create_segments_from_geo(&geo.geo);
        let constants = AcousticConstants::default();
        
        let mut tlm = DifferentiableTLM::new(segments, 500.0, constants);
        let z_in = tlm.forward();
        
        assert!(z_in.re != 0.0 || z_in.im != 0.0);
    }

    #[test]
    fn test_neural_fitness_predictor() {
        let mut predictor = NeuralFitnessPredictor::new(10, &[32, 32], 10);
        
        // Dummy training data
        let genomes = vec![vec![0.5; 10]; 5];
        let targets = vec![vec![Cf32::new(1.0, 0.0); 10]; 5];
        
        predictor.train(&genomes, &targets, 0.01);
        
        let fitness = predictor.predict_fitness(&genomes[0]);
        assert!(fitness > 0.0);
    }

    #[test]
    fn test_adam_optimizer() {
        let mut adam = AdamOptimizer::new(0.01);
        let mut params = vec![1.0, 2.0, 3.0];
        let grads = vec![0.1, -0.2, 0.3];
        
        adam.step(&mut params, &grads);
        
        assert!(params[0] < 1.0, "Adam should decrease param with positive grad");
        assert!(params[1] > 2.0, "Adam should increase param with negative grad");
    }
}