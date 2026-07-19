//! Loss functions for evolutionary optimization
//!
//! This module implements various loss functions for evaluating didgeridoo geometries
//! based on their acoustic properties, similar to the Python implementation.

use crate::evo::Genome;
use crate::sim::DidgeridooSimulator;
use ndarray::{Array1, s};
use serde::{Deserialize, Serialize};

/// Base trait for loss components
pub trait LossComponent: Send + Sync {
    /// Calculate loss value for given spectral data
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        peak_impedances: &Array1<f64>,
        all_freqs: &Array1<f64>,
        all_impedances: &Array1<f64>,
        peak_indices: &[usize],
    ) -> f64;
}

/// Test loss function for basic testing
#[derive(Debug, Clone)]
pub struct TestLossFunction {
    target_value: f64,
}

impl TestLossFunction {
    pub fn new() -> Self {
        Self { target_value: 100.0 }
    }
    
    pub fn with_target(target: f64) -> Self {
        Self { target_value: target }
    }
}

impl crate::evo::LossFunction for TestLossFunction {
    fn calculate(&self, genome: &dyn Genome) -> f64 {
        // Simple test: loss based on genome sum
        let sum: f64 = genome.genome().iter().sum();
        (sum - self.target_value).abs()
    }
}

/// Frequency tuning loss - align peaks to specific frequencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyTuningLoss {
    target_freqs_log: Array1<f64>,  // Target frequencies in log2 scale
    target_impedances: Array1<f64>, // Target normalized impedances [0,1]
    weights: Array1<f64>,          // Per-peak weights
}

impl FrequencyTuningLoss {
    pub fn new(
        target_freqs_log: Vec<f64>,
        target_impedances: Vec<f64>,
        weights: Vec<f64>,
    ) -> Self {
        Self {
            target_freqs_log: Array1::from(target_freqs_log),
            target_impedances: Array1::from(target_impedances),
            weights: Array1::from(weights),
        }
    }
}

impl LossComponent for FrequencyTuningLoss {
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        let mut total_loss = 0.0;
        
        for (i, &target_f_log) in self.target_freqs_log.iter().enumerate() {
            // Find closest actual peak to target frequency
            let closest_idx = peak_freqs_log.iter()
                .enumerate()
                .min_by(|(_, &a), (_, &b)| {
                    (a - target_f_log).abs().partial_cmp(&(b - target_f_log).abs()).unwrap()
                })
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            
            let actual_f_log = peak_freqs_log[closest_idx];
            let actual_amp = peak_impedances[closest_idx];
            
            // Frequency loss (normalized by 600 cents)
            let freq_error_cents = 1200.0 * (target_f_log - actual_f_log).abs();
            let freq_loss = freq_error_cents / 600.0;
            
            // Impedance loss (if target != -1)
            let amp_loss = if self.target_impedances.get(i).map_or(false, |&imp| imp != -1.0) {
                (self.target_impedances[i] - actual_amp).abs()
            } else {
                0.0
            };
            
            // Combine with weight
            if let Some(&weight) = self.weights.get(i) {
                total_loss += (freq_loss + amp_loss) * weight;
            }
        }
        
        total_loss
    }
}

/// Q-factor loss - control resonance sharpness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QFactorLoss {
    target_q: f64,
    weight: f64,
}

impl QFactorLoss {
    pub fn new(target_q: f64, weight: f64) -> Self {
        Self { target_q, weight }
    }
}

impl LossComponent for QFactorLoss {
    fn calculate(
        &self,
        _peak_freqs_log: &Array1<f64>,
        _peak_impedances: &Array1<f64>,
        all_freqs: &Array1<f64>,
        all_impedances: &Array1<f64>,
        peak_indices: &[usize],
    ) -> f64 {
        let mut qs = Vec::new();
        
        for &p_idx in peak_indices {
            let f_center = all_freqs[p_idx];
            let target_amp = all_impedances[p_idx] / 2.0f64.sqrt();
            
            // Find -3dB points
            let left_side = &all_impedances.slice(s![..p_idx]);
            let right_side = &all_impedances.slice(s![p_idx..]);
            
            let f_low_idx = left_side.iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let diff_a = (**a - target_amp).abs();
                    let diff_b = (**b - target_amp).abs();
                    diff_a.partial_cmp(&diff_b).unwrap()
                })
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            
            let f_high_idx = right_side.iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let diff_a = (**a - target_amp).abs();
                    let diff_b = (**b - target_amp).abs();
                    diff_a.partial_cmp(&diff_b).unwrap()
                })
                .map(|(idx, _)| p_idx + idx)
                .unwrap_or(all_freqs.len() - 1);
            
            let f_low = all_freqs[f_low_idx];
            let f_high = all_freqs[f_high_idx];
            
            let q = f_center / (f_high - f_low + 1e-9);
            qs.push(q);
        }
        
        let avg_q = if !qs.is_empty() {
            qs.iter().sum::<f64>() / qs.len() as f64
        } else {
            0.0
        };
        
        (avg_q - self.target_q).abs() * self.weight
    }
}

/// Modal density loss - reward peak proximity for shimmering effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalDensityLoss {
    cluster_range_cents: f64,
    weight: f64,
}

impl ModalDensityLoss {
    pub fn new(cluster_range_cents: f64, weight: f64) -> Self {
        Self { cluster_range_cents, weight }
    }
}

impl LossComponent for ModalDensityLoss {
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        _peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        if peak_freqs_log.len() < 2 {
            return self.weight;
        }
        
        // Calculate differences between adjacent peaks
        let diffs: Vec<f64> = peak_freqs_log.windows(2)
            .into_iter()
            .map(|window| (window[1] - window[0]) * 1200.0) // Convert to cents
            .collect();
        
        // Calculate shimmer score
        let shimmer_score: f64 = diffs.iter()
            .map(|&diff| {
                let val: f64 = diff - self.cluster_range_cents;
                (-val.powi(2) / 100.0).exp()
            })
            .sum();
        
        self.weight / (1.0 + shimmer_score)
    }
}

/// High inharmonic loss - maximize dissonance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighInharmonicLoss {
    weight: f64,
}

impl HighInharmonicLoss {
    pub fn new(weight: f64) -> Self {
        Self { weight }
    }
}

impl LossComponent for HighInharmonicLoss {
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        _peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        if peak_freqs_log.is_empty() {
            return self.weight;
        }
        
        let f0 = 2.0f64.powf(peak_freqs_log[0]);
        let ratios: Vec<f64> = peak_freqs_log.iter()
            .map(|&f_log| 2.0f64.powf(f_log) / f0)
            .collect();
        
        let dist_to_int: f64 = ratios.iter()
            .map(|&ratio| (ratio - ratio.round()).abs())
            .sum::<f64>() / ratios.len() as f64;
        
        self.weight * (0.5 - dist_to_int)
    }
}

/// Composite loss function that combines multiple components
pub struct CompositeTairuaLoss {
    components: Vec<(String, Box<dyn LossComponent>)>,
    max_error: f64,
    target_freqs: Option<Vec<f64>>,
}

impl std::fmt::Debug for CompositeTairuaLoss {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeTairuaLoss")
            .field("components", &self.components.iter().map(|(name, _)| name).collect::<Vec<_>>())
            .field("max_error", &self.max_error)
            .field("target_freqs", &self.target_freqs)
            .finish()
    }
}

impl CompositeTairuaLoss {
    /// Create a new CompositeTairuaLoss with a maximum error tolerance (in cents).
    pub fn new(max_error: f64) -> Self {
        Self {
            components: Vec::new(),
            max_error,
            target_freqs: None,
        }
    }

    /// Add a named loss component to the composite loss.
    pub fn add_component(&mut self, name: String, component: Box<dyn LossComponent>) {
        self.components.push((name, component));
    }

    /// Set explicit target frequencies for the simulation (optional).
    pub fn set_target_freqs(&mut self, target_freqs: Vec<f64>) {
        self.target_freqs = Some(target_freqs);
    }

    /// Construct a CompositeTairuaLoss with a sensible set of default components.
    ///
    /// The defaults include:
    /// - IntegerHarmonicLoss (weight 5.0)
    /// - NearIntegerLoss (tolerance 0.05, weight 5.0)
    /// - StretchedOddLoss (stretch_factor 1.0, weight 5.0)
    /// - HarmonicSplittingLoss (weight 5.0)
    /// - PeakQuantityLoss (target_quantity 5, weight 2.0)
    /// - PeakAmplitudeLoss (weight 2.0)
    /// - ScaleTuningLoss (weight 5.0)
    pub fn with_default_components(max_error: f64) -> Self {
        let mut loss = Self::new(max_error);
        loss.add_component("integer_harmonic".to_string(), Box::new(IntegerHarmonicLoss::new(5.0)));
        loss.add_component("near_integer".to_string(), Box::new(NearIntegerLoss::new(0.05, 5.0)));
        loss.add_component("stretched_odd".to_string(), Box::new(StretchedOddLoss::new(1.0, 5.0)));
        loss.add_component("harmonic_splitting".to_string(), Box::new(HarmonicSplittingLoss::new(5.0)));
        loss.add_component("peak_quantity".to_string(), Box::new(PeakQuantityLoss::new(5, 2.0)));
        loss.add_component("peak_amplitude".to_string(), Box::new(PeakAmplitudeLoss::new(2.0)));
        loss.add_component("scale_tuning".to_string(), Box::new(ScaleTuningLoss::new(5.0)));
        loss
    }

    /// Get frequency grid for simulation (internal helper).
    fn _get_frequency_grid(&self) -> Vec<f64> {
        let mut freqs = Vec::new();
        let mut f = 20.0; // Start frequency
        let step_ratio = 2.0f64.powf(self.max_error / 1200.0);

        while f <= 2000.0 {
            freqs.push(f);
            f *= step_ratio;
        }

        freqs
    }
}

impl crate::evo::LossFunction for CompositeTairuaLoss {
    fn calculate(&self, genome: &dyn Genome) -> f64 {
        // Convert genome to geometry
        let geo = genome.genome2geo();

        // Create simulator from geometry points
        let simulator = DidgeridooSimulator::from_geo(&geo.points);
        // Frequency grid for simulation
        let freqs = self._get_frequency_grid();

        // Compute impedance spectrum (complex values) and convert to magnitudes
        let spectrum = simulator.impedance(&freqs);
        if spectrum.is_empty() {
            return 1e6; // Large penalty for invalid geometries
        }
        // All frequencies and impedances as arrays
        let all_freqs = Array1::from(freqs.clone());
        let all_impedances = Array1::from(spectrum.iter().map(|c| c.norm()).collect::<Vec<f64>>());

        // Find peaks using the same frequency grid
        let peaks = simulator.peaks(&freqs);
        if peaks.is_empty() {
            return 1e6; // Large penalty for no peaks
        }

        // Convert peak data to arrays for loss components
        let peak_freqs_log = Array1::from(peaks.iter().map(|p| f64::log2(p.1)).collect::<Vec<f64>>());
        let max_imp = all_impedances.iter().cloned().fold(0.0_f64, f64::max);
        let peak_impedances = Array1::from(peaks.iter().map(|p| p.2 / max_imp).collect::<Vec<f64>>());
        let peak_indices: Vec<usize> = peaks.iter().map(|p| p.0).collect();

        // Calculate total loss from all components
        let mut total_loss = 0.0;
        for (_name, component) in &self.components {
            let component_loss = component.calculate(
                &peak_freqs_log,
                &peak_impedances,
                &all_freqs,
                &all_impedances,
                &peak_indices,
            );
            total_loss += component_loss;
        }
        total_loss
    }
}


/// Integer harmonic loss - encourage integer harmonic relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegerHarmonicLoss {
    weight: f64,
}

impl IntegerHarmonicLoss {
    pub fn new(weight: f64) -> Self {
        Self { weight }
    }
}

impl LossComponent for IntegerHarmonicLoss {
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        _peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        if peak_freqs_log.is_empty() {
            return self.weight;
        }
        
        let f0 = 2.0f64.powf(peak_freqs_log[0]);
        let inharmonicity: f64 = peak_freqs_log.iter()
            .map(|&f_log| {
                let f = 2.0f64.powf(f_log);
                let ratio = f / f0;
                (ratio - ratio.round()).abs()
            })
            .sum::<f64>() / peak_freqs_log.len() as f64;
            
        inharmonicity * self.weight
    }
}

/// Near integer loss - allow near-integer harmonic relationships within tolerance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearIntegerLoss {
    tolerance: f64,
    weight: f64,
}

impl NearIntegerLoss {
    pub fn new(tolerance: f64, weight: f64) -> Self {
        Self { tolerance, weight }
    }
}

impl LossComponent for NearIntegerLoss {
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        _peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        if peak_freqs_log.is_empty() {
            return self.weight;
        }
        
        let f0 = 2.0f64.powf(peak_freqs_log[0]);
        let error: f64 = peak_freqs_log.iter()
            .map(|&f_log| {
                let f = 2.0f64.powf(f_log);
                let ratio = f / f0;
                let dist = (ratio - ratio.round()).abs();
                if dist < self.tolerance {
                    0.0
                } else {
                    dist - self.tolerance
                }
            })
            .sum::<f64>() / peak_freqs_log.len() as f64;
            
        error * self.weight
    }
}

/// Stretched odd loss - target stretched odd harmonics (e.g. 1, 3, 5... with stretch factor)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StretchedOddLoss {
    stretch_factor: f64,
    weight: f64,
}

impl StretchedOddLoss {
    pub fn new(stretch_factor: f64, weight: f64) -> Self {
        Self { stretch_factor, weight }
    }
}

impl LossComponent for StretchedOddLoss {
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        _peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        if peak_freqs_log.is_empty() {
            return self.weight;
        }
        
        let f0 = 2.0f64.powf(peak_freqs_log[0]);
        let error: f64 = peak_freqs_log.iter().enumerate()
            .map(|(i, &f_log)| {
                let f = 2.0f64.powf(f_log);
                let ratio = f / f0;
                let target_ratio = ((2 * i + 1) as f64) * self.stretch_factor;
                (ratio - target_ratio).abs()
            })
            .sum::<f64>() / peak_freqs_log.len() as f64;
            
        error * self.weight
    }
}

/// Harmonic splitting loss - drive adjacent peak ratios away from integer values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicSplittingLoss {
    weight: f64,
}

impl HarmonicSplittingLoss {
    pub fn new(weight: f64) -> Self {
        Self { weight }
    }
}

impl LossComponent for HarmonicSplittingLoss {
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        _peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        if peak_freqs_log.is_empty() {
            return self.weight;
        }
        
        let f0 = 2.0f64.powf(peak_freqs_log[0]);
        let split_loss: f64 = peak_freqs_log.iter()
            .map(|&f_log| {
                let f = 2.0f64.powf(f_log);
                let ratio = f / f0;
                let dist = (ratio - ratio.round()).abs();
                (0.5 - dist).max(0.0)
            })
            .sum::<f64>() / peak_freqs_log.len() as f64;
            
        split_loss * self.weight
    }
}

/// Peak quantity loss - encourage a larger number of resonance peaks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakQuantityLoss {
    target_quantity: usize,
    weight: f64,
}

impl PeakQuantityLoss {
    pub fn new(target_quantity: usize, weight: f64) -> Self {
        Self { target_quantity, weight }
    }
}

impl LossComponent for PeakQuantityLoss {
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        _peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        let n = peak_freqs_log.len();
        if n >= self.target_quantity {
            0.0
        } else {
            ((self.target_quantity - n) as f64) * self.weight
        }
    }
}

/// Peak amplitude loss - encourage stronger resonance peaks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakAmplitudeLoss {
    weight: f64,
}

impl PeakAmplitudeLoss {
    pub fn new(weight: f64) -> Self {
        Self { weight }
    }
}

impl LossComponent for PeakAmplitudeLoss {
    fn calculate(
        &self,
        _peak_freqs_log: &Array1<f64>,
        peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        if peak_impedances.is_empty() {
            return self.weight;
        }
        
        let avg_amp = peak_impedances.iter().sum::<f64>() / peak_impedances.len() as f64;
        (1.0 - avg_amp).max(0.0) * self.weight
    }
}

/// Scale tuning loss - pull resonance peaks toward standard chromatic musical notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleTuningLoss {
    weight: f64,
}

impl ScaleTuningLoss {
    pub fn new(weight: f64) -> Self {
        Self { weight }
    }
}

impl LossComponent for ScaleTuningLoss {
    fn calculate(
        &self,
        peak_freqs_log: &Array1<f64>,
        _peak_impedances: &Array1<f64>,
        _all_freqs: &Array1<f64>,
        _all_impedances: &Array1<f64>,
        _peak_indices: &[usize],
    ) -> f64 {
        if peak_freqs_log.is_empty() {
            return self.weight;
        }
        
        let error: f64 = peak_freqs_log.iter()
            .map(|&f_log| {
                let f = 2.0f64.powf(f_log);
                let midi_note = 69.0 + 12.0 * (f / 440.0).log2();
                let dist = (midi_note - midi_note.round()).abs();
                dist / 0.5
            })
            .sum::<f64>() / peak_freqs_log.len() as f64;
            
        error * self.weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evo::{BaseGenome, KigaliGenome, LossFunction};
    use approx::assert_abs_diff_eq;
    use ndarray::Array1;
    
    #[test]
    fn test_test_loss_function() {
        let loss_fn = TestLossFunction::with_target(5.0);
        let genome = BaseGenome::random(5);
        let loss = loss_fn.calculate(&genome);
        assert!(loss >= 0.0);
    }
    
    #[test]
    fn test_frequency_tuning_loss() {
        let loss_fn = FrequencyTuningLoss::new(
            vec![2.0, 3.0],  // target frequencies (log2)
            vec![0.5, 0.3],  // target impedances
            vec![1.0, 1.0],  // weights
        );
        
        let peak_freqs = Array1::from(vec![2.1, 2.9]);  // close to targets
        let peak_impedances = Array1::from(vec![0.4, 0.4]);  // close to targets
        
        let loss = loss_fn.calculate(
            &peak_freqs,
            &peak_impedances,
            &Array1::zeros(10),
            &Array1::zeros(10),
            &[0, 1],
        );
        
        assert!(loss >= 0.0);
        assert!(loss.is_finite());
    }
    
    #[test]
    fn test_modal_density_loss() {
        let loss_fn = ModalDensityLoss::new(50.0, 1.0); // 50 cents cluster range
        
        let peak_freqs = Array1::from(vec![2.0, 2.1, 2.2]); // closely spaced peaks
        
        let loss = loss_fn.calculate(
            &peak_freqs,
            &Array1::zeros(3),
            &Array1::zeros(10),
            &Array1::zeros(10),
            &[0, 1, 2],
        );
        
        assert!(loss >= 0.0);
        assert!(loss <= 1.0); // Should be bounded
    }
    
    #[test]
    fn test_high_inharmonic_loss() {
        let loss_fn = HighInharmonicLoss::new(1.0);
        
        // Perfect integer harmonics
        let perfect_harmonics = Array1::from(vec![0.0, 1.0, 1.585]); // log2(1), log2(2), log2(3)
        let perfect_loss = loss_fn.calculate(
            &perfect_harmonics,
            &Array1::zeros(3),
            &Array1::zeros(10),
            &Array1::zeros(10),
            &[0, 1, 2],
        );
        
        // Inharmonic frequencies
        let inharmonic_freqs = Array1::from(vec![0.0, 1.1, 1.7]); // slightly inharmonic
        let inharmonic_loss = loss_fn.calculate(
            &inharmonic_freqs,
            &Array1::zeros(3),
            &Array1::zeros(10),
            &Array1::zeros(10),
            &[0, 1, 2],
        );
        
        // Inharmonic should have lower loss (better fitness)
        assert!(inharmonic_loss <= perfect_loss);
    }
    
    #[test]
    fn test_composite_loss() {
        let mut composite_loss = CompositeTairuaLoss::new(5.0);
        
        composite_loss.add_component(
            "freq_tuning".to_string(),
            Box::new(FrequencyTuningLoss::new(
                vec![2.0],  // target D1 (~73.4 Hz)
                vec![0.5],
                vec![1.0],
            ))
        );
        
        composite_loss.add_component(
            "modal_density".to_string(),
            Box::new(ModalDensityLoss::new(50.0, 0.5))
        );
        
        let genome = KigaliGenome::new(10, 32.0, 50.0, 80.0, 1800.0, 1500.0, 0, 0.3, 0.0, 300.0);
        let loss = composite_loss.calculate(&genome);
        
        assert!(loss >= 0.0);
        assert!(loss.is_finite());
    }

    #[test]
    fn test_new_loss_components() {
        let f_log = Array1::from(vec![f64::log2(100.0), f64::log2(200.0), f64::log2(300.0)]);
        let amps = Array1::from(vec![1.0, 0.8, 0.6]);
        let all_f = Array1::zeros(10);
        let all_z = Array1::zeros(10);
        let idx = vec![0, 1, 2];

        // IntegerHarmonicLoss: 100, 200, 300 are integer ratios 1, 2, 3, so loss should be 0.0
        let loss_int = IntegerHarmonicLoss::new(10.0);
        let val_int = loss_int.calculate(&f_log, &amps, &all_f, &all_z, &idx);
        assert_abs_diff_eq!(val_int, 0.0, epsilon = 1e-6);

        // NearIntegerLoss: since they are integer ratios, they are within tolerance, so loss is 0.0
        let loss_near = NearIntegerLoss::new(0.05, 10.0);
        let val_near = loss_near.calculate(&f_log, &amps, &all_f, &all_z, &idx);
        assert_abs_diff_eq!(val_near, 0.0, epsilon = 1e-6);

        // StretchedOddLoss: target is odd harmonics (1, 3, 5) with stretch=1.0. Ratios (1, 2, 3) will have error.
        let loss_odd = StretchedOddLoss::new(1.0, 10.0);
        let val_odd = loss_odd.calculate(&f_log, &amps, &all_f, &all_z, &idx);
        assert!(val_odd > 0.0);

        // HarmonicSplittingLoss: Ratios are 1, 2, 3 (integers), which are heavily penalized.
        let loss_split = HarmonicSplittingLoss::new(10.0);
        let val_split = loss_split.calculate(&f_log, &amps, &all_f, &all_z, &idx);
        assert!(val_split > 0.0);

        // PeakQuantityLoss: We have 3 peaks. Target is 5. Loss should be (5 - 3) * 10 = 20.
        let loss_qty = PeakQuantityLoss::new(5, 10.0);
        let val_qty = loss_qty.calculate(&f_log, &amps, &all_f, &all_z, &idx);
        assert_abs_diff_eq!(val_qty, 20.0, epsilon = 1e-6);

        // PeakAmplitudeLoss: Average amplitude is (1.0 + 0.8 + 0.6)/3 = 0.8. Loss is (1.0 - 0.8) * 10 = 2.0.
        let loss_amp = PeakAmplitudeLoss::new(10.0);
        let val_amp = loss_amp.calculate(&f_log, &amps, &all_f, &all_z, &idx);
        assert_abs_diff_eq!(val_amp, 2.0, epsilon = 1e-6);

        // ScaleTuningLoss: Calculates musical note offsets.
        let loss_scale = ScaleTuningLoss::new(10.0);
        let val_scale = loss_scale.calculate(&f_log, &amps, &all_f, &all_z, &idx);
        assert!(val_scale >= 0.0);
    }
}