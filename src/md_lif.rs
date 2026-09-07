//! Leaky Integrate-and-Fire (LIF) spiking neuron module.
//!
//! This module provides a simple spiking neural network implementation
//! using the LIF neuron model, which is commonly used in neuromorphic
//! computing and brain-inspired architectures.

/// LIF neuron parameters
#[derive(Debug, Clone, Copy)]
pub struct LifParams {
    /// Membrane time constant (ms)
    pub tau_m: f64,
    /// Resting potential (mV)
    pub v_rest: f64,
    /// Reset potential (mV)
    pub v_reset: f64,
    /// Threshold potential (mV)
    pub v_threshold: f64,
    /// Refractory period (ms)
    pub refractory_period: f64,
}

impl Default for LifParams {
    fn default() -> Self {
        Self {
            tau_m: 20.0,
            v_rest: -70.0,
            v_reset: -70.0,
            v_threshold: -50.0,
            refractory_period: 2.0,
        }
    }
}

/// A single LIF neuron
#[derive(Debug, Clone)]
pub struct LifNeuron {
    pub params: LifParams,
    pub membrane_potential: f64,
    pub last_spike_time: f64,
}

impl LifNeuron {
    pub fn new(params: LifParams) -> Self {
        Self {
            params,
            membrane_potential: params.v_rest,
            last_spike_time: -1.0,
        }
    }

    pub fn step(&mut self, input_current: f64, dt: f64, current_time: f64) -> bool {
        let refractory = current_time - self.last_spike_time < self.params.refractory_period;
        
        if !refractory {
            let alpha = dt / self.params.tau_m;
            self.membrane_potential = self.membrane_potential * (1.0 - alpha) + input_current * alpha;
        } else {
            self.membrane_potential = self.params.v_reset;
        }

        if self.membrane_potential >= self.params.v_threshold {
            self.membrane_potential = self.params.v_reset;
            self.last_spike_time = current_time;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.membrane_potential = self.params.v_rest;
        self.last_spike_time = -1.0;
    }
}

/// A layer of LIF neurons
#[derive(Debug, Clone)]
pub struct LifLayer {
    pub neurons: Vec<LifNeuron>,
    pub weights: Vec<Vec<f64>>,
}

impl LifLayer {
    pub fn new(size: usize, params: LifParams) -> Self {
        let neurons = vec![LifNeuron::new(params); size];
        Self {
            neurons,
            weights: Vec::new(),
        }
    }

    pub fn step(&mut self, inputs: &[f64], dt: f64, time: f64) -> Vec<bool> {
        self.neurons
            .iter_mut()
            .enumerate()
            .map(|(i, neuron)| {
                let input = inputs.get(i).copied().unwrap_or(0.0);
                neuron.step(input, dt, time)
            })
            .collect()
    }

    pub fn reset(&mut self) {
        for neuron in &mut self.neurons {
            neuron.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lif_neuron_fires() {
        let params = LifParams {
            tau_m: 20.0,
            v_rest: -70.0,
            v_reset: -70.0,
            v_threshold: -50.0,
            refractory_period: 0.0,
        };
        let mut neuron = LifNeuron::new(params);
        for _ in 0..100 {
            neuron.step(100.0, 0.1, 0.0);
        }
        assert!(neuron.membrane_potential >= params.v_threshold || neuron.last_spike_time >= 0.0);
    }

    #[test]
    fn test_lif_layer() {
        let mut layer = LifLayer::new(3, LifParams::default());
        let inputs = vec![10.0, 20.0, 30.0];
        let spikes = layer.step(&inputs, 0.1, 0.0);
        assert_eq!(spikes.len(), 3);
    }
}
