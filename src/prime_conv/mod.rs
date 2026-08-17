//! Prime-sized 1D convolution block for multi-scale feature extraction.
//!
//! Based on the Omni-Scale CNN (OS-CNN) architecture by Tang et al. (ICLR 2022),
//! which uses prime-numbered kernel sizes to efficiently cover all receptive field
//! scales via Goldbach's conjecture (every even number is the sum of two primes).
//!
//! For wind instrument impedance spectra (1D complex signals), this enables:
//! - O(r² / log r) parameter efficiency vs O(r²) for sequential kernel sizes
//! - Coverage of all physically relevant frequency scales with minimal parameters
//! - Natural multi-scale analysis of resonance peaks and formants

use rand::Rng;

use crate::evo::{Genome, LossFunction};

/// Prime number generator for OS-block kernel sizes
#[derive(Debug, Clone)]
pub struct PrimeGenerator {
    primes: Vec<usize>,
    current_index: usize,
}

impl PrimeGenerator {
    pub fn new(max_prime: usize) -> Self {
        let primes = Self::sieve(max_prime);
        Self { primes, current_index: 0 }
    }

    fn sieve(max: usize) -> Vec<usize> {
        if max < 2 { return vec![]; }
        let mut is_prime = vec![true; max + 1];
        is_prime[0] = false;
        is_prime[1] = false;
        for i in 2..=((max as f64).sqrt() as usize) {
            if is_prime[i] {
                for j in (i * i..=max).step_by(i) {
                    is_prime[j] = false;
                }
            }
        }
        is_prime.iter().enumerate().filter(|(_, &p)| p).map(|(i, _)| i).collect()
    }

    pub fn next_prime(&mut self) -> usize {
        let prime = self.primes[self.current_index % self.primes.len()];
        self.current_index += 1;
        prime
    }

    pub fn primes_up_to(&self, max: usize) -> Vec<usize> {
        self.primes.iter().filter(|&&p| p <= max).copied().collect()
    }

    pub fn prime_list(&self) -> &[usize] {
        &self.primes
    }
}

impl Default for PrimeGenerator {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// 1D convolution with complex weights
#[derive(Debug, Clone)]
pub struct ComplexConv1D {
    pub kernel_size: usize,
    pub in_channels: usize,
    pub out_channels: usize,
    pub weights: Vec<num_complex::Complex64>,
    pub bias: Vec<num_complex::Complex64>,
    pub grad_weights: Vec<num_complex::Complex64>,
    pub grad_bias: Vec<num_complex::Complex64>,
}

impl ComplexConv1D {
    pub fn new(kernel_size: usize, in_channels: usize, out_channels: usize) -> Self {
        let fan_in = kernel_size * in_channels;
        let limit = (6.0 / (fan_in + out_channels) as f64).sqrt();
        let mut rng = rand::thread_rng();
        
        let weights = (0..out_channels * kernel_size * in_channels)
            .map(|_| num_complex::Complex64::new(
                (rng.gen::<f64>() - 0.5) * limit,
                (rng.gen::<f64>() - 0.5) * limit,
            ))
            .collect();
            
        let bias = (0..out_channels)
            .map(|_| num_complex::Complex64::new(0.0, 0.0))
            .collect();
            
        let grad_weights = vec![num_complex::Complex64::new(0.0, 0.0); out_channels * kernel_size * in_channels];
        let grad_bias = vec![num_complex::Complex64::new(0.0, 0.0); out_channels];
            
        Self {
            kernel_size,
            in_channels,
            out_channels,
            weights,
            bias,
            grad_weights,
            grad_bias,
        }
    }

    pub fn forward(&self, input: &[num_complex::Complex64]) -> Vec<num_complex::Complex64> {
        if input.len() < self.kernel_size {
            return vec![num_complex::Complex64::new(0.0, 0.0); self.out_channels];
        }
        
        let mut output = Vec::with_capacity(self.out_channels);
        for oc in 0..self.out_channels {
            let mut sum = self.bias[oc];
            for ic in 0..self.in_channels {
                for k in 0..self.kernel_size {
                    let w_idx = oc * self.kernel_size * self.in_channels + ic * self.kernel_size + k;
                    let i_idx = ic * input.len() / self.in_channels + k;
                    sum += self.weights[w_idx] * input[i_idx];
                }
            }
            output.push(sum);
        }
        output
    }

    pub fn backward(
        &mut self,
        grad_output: &[num_complex::Complex64],
        input: &[num_complex::Complex64],
    ) -> Vec<num_complex::Complex64> {
        let mut grad_input = vec![num_complex::Complex64::new(0.0, 0.0); input.len()];
        
        for (oc, go) in grad_output.iter().enumerate().take(self.out_channels) {
            for ic in 0..self.in_channels {
                for k in 0..self.kernel_size {
                    let w_idx = oc * self.kernel_size * self.in_channels + ic * self.kernel_size + k;
                    let i_idx = ic * input.len() / self.in_channels + k;
                    
                    self.grad_weights[w_idx] += go.conj() * input[i_idx];
                    self.grad_bias[oc] += go.conj();
                    
                    let w_conj = self.weights[w_idx].conj();
                    grad_input[i_idx] += *go * w_conj;
                }
            }
        }
        
        grad_input
    }

    pub fn step(&mut self, lr: f64) {
        for w in &mut self.weights {
            *w -= num_complex::Complex64::new(lr, 0.0) * self.grad_weights.iter().sum::<num_complex::Complex64>();
        }
        for b in &mut self.bias {
            *b -= num_complex::Complex64::new(lr, 0.0) * self.grad_bias.iter().sum::<num_complex::Complex64>();
        }
        self.grad_weights.fill(num_complex::Complex64::new(0.0, 0.0));
        self.grad_bias.fill(num_complex::Complex64::new(0.0, 0.0));
    }
}

/// Prime-sized convolution block (OS-block inspired)
#[derive(Debug, Clone)]
pub struct PrimeConvBlock {
    pub prime_kernels: Vec<ComplexConv1D>,
    pub activation: fn(num_complex::Complex64) -> num_complex::Complex64,
}

impl PrimeConvBlock {
    pub fn new(max_prime: usize, in_channels: usize, out_channels: usize) -> Self {
        let prime_gen = PrimeGenerator::new(max_prime);
        let primes = prime_gen.primes_up_to(max_prime);
        
        let prime_kernels = primes.iter()
            .map(|&p| ComplexConv1D::new(p, in_channels, out_channels))
            .collect();
            
        Self {
            prime_kernels,
            activation: complex_activations::crelu,
        }
    }

    pub fn forward(&self, input: &[num_complex::Complex64]) -> Vec<num_complex::Complex64> {
        let mut outputs = Vec::new();
        for conv in &self.prime_kernels {
            let mut out = conv.forward(input);
            for val in &mut out {
                *val = (self.activation)(*val);
            }
            outputs.extend(out);
        }
        outputs
    }

    pub fn backward(
        &mut self,
        grad_output: &[num_complex::Complex64],
        input: &[num_complex::Complex64],
    ) -> Vec<num_complex::Complex64> {
        let mut total_grad_input = vec![num_complex::Complex64::new(0.0, 0.0); input.len()];
        let mut offset = 0;
        
        for conv in &mut self.prime_kernels {
            let out_channels = conv.out_channels;
            let chunk = &grad_output[offset..offset + out_channels];
            let grad_in = conv.backward(chunk, input);
            for (i, g) in grad_in.iter().enumerate() {
                total_grad_input[i] += *g;
            }
            offset += out_channels;
        }
        
        total_grad_input
    }

    pub fn step(&mut self, lr: f64) {
        for conv in &mut self.prime_kernels {
            conv.step(lr);
        }
    }
}

/// Complex-valued neural network module (behind nn-integration feature flag)
pub mod complex_activations {
    use num_complex::Complex64;
    
    pub fn sigmoid(z: Complex64) -> Complex64 {
        let exp_z = z.exp();
        exp_z / (Complex64::new(1.0, 0.0) + exp_z)
    }
    
    pub fn relu(z: Complex64) -> Complex64 {
        if z.re > 0.0 && z.im > 0.0 { z } else { Complex64::new(0.0, 0.0) }
    }
    
    pub fn crelu(z: Complex64) -> Complex64 {
        Complex64::new(z.re.max(0.0), z.im.max(0.0))
    }
    
    pub fn mod_relu(z: Complex64) -> Complex64 {
        let norm = z.norm();
        if norm > 1e-8 {
            Complex64::new(z.re.max(0.0), z.im.max(0.0)) / norm
        } else {
            Complex64::new(0.0, 0.0)
        }
    }
    
    pub fn tanh(z: Complex64) -> Complex64 {
        z.tanh()
    }
    
    pub fn zrelu(z: Complex64) -> Complex64 {
        let phase = z.arg();
        if (0.0..std::f64::consts::PI).contains(&phase) {
            z
        } else {
            Complex64::new(0.0, 0.0)
        }
    }
}

/// Complex-valued linear layer with Wirtinger backpropagation support
#[derive(Debug, Clone)]
pub struct ComplexLinear {
    pub in_features: usize,
    pub out_features: usize,
    pub weights: Vec<num_complex::Complex64>,
    pub bias: Vec<num_complex::Complex64>,
    pub grad_weights: Vec<num_complex::Complex64>,
    pub grad_bias: Vec<num_complex::Complex64>,
}

impl ComplexLinear {
    pub fn new(in_features: usize, out_features: usize) -> Self {
        let limit = (6.0 / (in_features + out_features) as f64).sqrt();
        let mut rng = rand::thread_rng();
        
        let weights = (0..in_features * out_features)
            .map(|_| num_complex::Complex64::new(
                (rng.gen::<f64>() - 0.5) * limit,
                (rng.gen::<f64>() - 0.5) * limit,
            ))
            .collect();
            
        let bias = (0..out_features)
            .map(|_| num_complex::Complex64::new(0.0, 0.0))
            .collect();
            
        let grad_weights = vec![num_complex::Complex64::new(0.0, 0.0); in_features * out_features];
        let grad_bias = vec![num_complex::Complex64::new(0.0, 0.0); out_features];
            
        Self {
            in_features,
            out_features,
            weights,
            bias,
            grad_weights,
            grad_bias,
        }
    }

    pub fn forward(&self, input: &[num_complex::Complex64]) -> Vec<num_complex::Complex64> {
        assert_eq!(input.len(), self.in_features);
        let mut output = Vec::with_capacity(self.out_features);
        
        for (j, bias) in self.bias.iter().enumerate().take(self.out_features) {
            let mut sum = *bias;
            for (i, &inp) in input.iter().enumerate().take(self.in_features) {
                sum += self.weights[j * self.in_features + i] * inp;
            }
            output.push(sum);
        }
        output
    }

    pub fn backward(
        &mut self,
        grad_output: &[num_complex::Complex64],
        input: &[num_complex::Complex64],
    ) -> Vec<num_complex::Complex64> {
        // grad_input = grad_output @ W^H (Wirtinger derivative)
        let mut grad_input = vec![num_complex::Complex64::new(0.0, 0.0); self.in_features];
        
        for (i, grad_in) in grad_input.iter_mut().enumerate().take(self.in_features) {
            for (j, go) in grad_output.iter().enumerate().take(self.out_features) {
                let w_conj = self.weights[j * self.in_features + i].conj();
                *grad_in += go * w_conj;
            }
        }
        
        // grad_weights = grad_output^H @ input (Wirtinger derivative)
        for (j, go) in grad_output.iter().enumerate().take(self.out_features) {
            for (i, &inp) in input.iter().enumerate().take(self.in_features) {
                let go_conj = go.conj();
                self.grad_weights[j * self.in_features + i] += go_conj * inp;
            }
            self.grad_bias[j] += grad_output[j].conj();
        }
        
        grad_input
    }

    pub fn step(&mut self, lr: f64) {
        for w in &mut self.weights {
            *w -= num_complex::Complex64::new(lr, 0.0) * self.grad_weights.iter().sum::<num_complex::Complex64>();
        }
        for b in &mut self.bias {
            *b -= num_complex::Complex64::new(lr, 0.0) * self.grad_bias.iter().sum::<num_complex::Complex64>();
        }
        self.grad_weights.fill(num_complex::Complex64::new(0.0, 0.0));
        self.grad_bias.fill(num_complex::Complex64::new(0.0, 0.0));
    }
}

/// Complex-valued MLP with prime conv frontend
#[derive(Debug, Clone)]
pub struct ComplexPrimeMLP {
    pub conv_block: PrimeConvBlock,
    pub layers: Vec<ComplexLinear>,
    pub activation: fn(num_complex::Complex64) -> num_complex::Complex64,
}

impl ComplexPrimeMLP {
    pub fn new(
        _input_len: usize,
        max_prime: usize,
        in_channels: usize,
        hidden_dims: &[usize],
        output_dim: usize,
    ) -> Self {
        let conv_block = PrimeConvBlock::new(max_prime, in_channels, in_channels);
        
        let mut layers = Vec::new();
        let mut prev_dim = in_channels * conv_block.prime_kernels.len();
        
        for &h in hidden_dims {
            layers.push(ComplexLinear::new(prev_dim, h));
            prev_dim = h;
        }
        layers.push(ComplexLinear::new(prev_dim, output_dim));
        
        Self {
            conv_block,
            layers,
            activation: complex_activations::crelu,
        }
    }

    pub fn forward(&self, input: &[num_complex::Complex64]) -> Vec<num_complex::Complex64> {
        let mut x = self.conv_block.forward(input);
        
        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x);
            if i < self.layers.len() - 1 {
                x.iter_mut().for_each(|v| *v = (self.activation)(*v));
            }
        }
        x
    }

    pub fn train(
        &mut self,
        inputs: &[Vec<num_complex::Complex64>],
        targets: &[Vec<num_complex::Complex64>],
        lr: f64,
        epochs: usize,
    ) {
        for _epoch in 0..epochs {
            for (input, target) in inputs.iter().zip(targets.iter()) {
                let mut x = self.conv_block.forward(input);
                
                let mut layer_outputs = Vec::new();
                for (i, layer) in self.layers.iter().enumerate() {
                    layer_outputs.push(x.clone());
                    x = layer.forward(&x);
                    if i < self.layers.len() - 1 {
                        x.iter_mut().for_each(|v| *v = (self.activation)(*v));
                    }
                }
                
                let mut grad_output = Vec::new();
                for (pred, tgt) in x.iter().zip(target.iter()) {
                    let diff = *pred - *tgt;
                    grad_output.push(diff.conj());
                }
                
                for (layer_idx, layer) in self.layers.iter_mut().enumerate().rev() {
                    let prev_output = &layer_outputs[layer_idx];
                    let grad_input = layer.backward(&grad_output, prev_output);
                    if layer_idx > 0 {
                        grad_output = grad_input;
                    }
                }
                
                let final_grad = self.conv_block.backward(&grad_output, input);
                drop(final_grad);
                
                self.conv_block.step(lr);
                for layer in &mut self.layers {
                    layer.step(lr);
                }
            }
        }
    }
}

/// Surrogate impedance loss function using ComplexPrimeMLP.
///
/// When `trained` is false, falls back to a provided TLM-based loss function.
/// When `trained` is true, uses the neural network to predict impedance spectrum
/// from genome parameters and computes MSE against a target spectrum.
pub struct SurrogateLossFunction {
    pub model: ComplexPrimeMLP,
    pub target_freqs: Vec<f64>,
    pub target_impedance: Vec<f64>,
    pub trained: bool,
    pub fallback_loss: Option<Box<dyn crate::evo::LossFunction>>,
}

impl SurrogateLossFunction {
    pub fn new(
        input_len: usize,
        max_prime: usize,
        in_channels: usize,
        hidden_dims: &[usize],
        output_dim: usize,
    ) -> Self {
        let model = ComplexPrimeMLP::new(input_len, max_prime, in_channels, hidden_dims, output_dim);
        Self {
            model,
            target_freqs: Vec::new(),
            target_impedance: Vec::new(),
            trained: false,
            fallback_loss: None,
        }
    }

    pub fn with_fallback(mut self, fallback: Box<dyn LossFunction>) -> Self {
        self.fallback_loss = Some(fallback);
        self
    }

    pub fn set_target(&mut self, freqs: Vec<f64>, impedance: Vec<f64>) {
        self.target_freqs = freqs;
        self.target_impedance = impedance;
    }

    /// Predict impedance spectrum from genome vector using the surrogate model
    pub fn predict_impedance(&self, genome: &[f64]) -> Vec<f64> {
        let input: Vec<num_complex::Complex64> = genome.iter()
            .map(|&v| num_complex::Complex64::new(v, 0.0))
            .collect();
        let output = self.model.forward(&input);
        output.iter().map(|c| c.norm()).collect()
    }

    /// Train the surrogate model on data generated by a simulator function.
    ///
    /// `simulator_fn` takes a genome vector and returns the target impedance spectrum.
    pub fn train_from_simulator(
        &mut self,
        mut simulator_fn: impl FnMut(&[f64]) -> Vec<f64>,
        num_samples: usize,
        lr: f64,
        epochs: usize,
    ) {
        let input_len = self.model.layers.first().map(|l| l.in_features).unwrap_or(20);
        let _output_dim = self.model.layers.last().map(|l| l.out_features).unwrap_or(50);

        let mut inputs = Vec::with_capacity(num_samples);
        let mut targets = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let genome: Vec<f64> = (0..input_len).map(|_| rand::random::<f64>()).collect();
            let target = simulator_fn(&genome);
            inputs.push(genome);
            targets.push(target);
        }

        let inputs_complex: Vec<Vec<num_complex::Complex64>> = inputs
            .iter()
            .map(|g| g.iter().map(|&v| num_complex::Complex64::new(v, 0.0)).collect())
            .collect();
        let targets_complex: Vec<Vec<num_complex::Complex64>> = targets
            .iter()
            .map(|t| t.iter().map(|&v| num_complex::Complex64::new(v, 0.0)).collect())
            .collect();

        self.model.train(&inputs_complex, &targets_complex, lr, epochs);
        self.trained = true;
    }
}

impl LossFunction for SurrogateLossFunction {
    fn calculate(&self, genome: &dyn Genome) -> f64 {
        if !self.trained {
            if let Some(ref fallback) = self.fallback_loss {
                return fallback.calculate(genome);
            }
            return f64::INFINITY;
        }

        let genome_vec = genome.genome();
        let predicted = self.predict_impedance(genome_vec);
        
        if predicted.is_empty() || self.target_impedance.is_empty() {
            return f64::INFINITY;
        }

        let mse: f64 = predicted.iter()
            .zip(self.target_impedance.iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum();
        mse / predicted.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prime_generator() {
        let gen = PrimeGenerator::new(100);
        assert_eq!(gen.primes_up_to(10), vec![2, 3, 5, 7]);
    }
    
    #[test]
    fn test_complex_conv1d() {
        let conv = ComplexConv1D::new(3, 1, 4);
        let input = vec![num_complex::Complex64::new(1.0, 0.0); 10];
        let out = conv.forward(&input);
        assert_eq!(out.len(), 4);
    }
    
    #[test]
    fn test_prime_conv_block() {
        let block = PrimeConvBlock::new(50, 1, 2);
        let input = vec![num_complex::Complex64::new(1.0, 0.0); 100];
        let out = block.forward(&input);
        assert!(out.len() > 0);
    }
}
