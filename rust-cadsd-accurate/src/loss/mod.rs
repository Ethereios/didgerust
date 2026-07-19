//! Loss functions for didgeridoo optimization
//!
//! This module provides various loss functions for evaluating didgeridoo designs,
//! with a focus on the Tairua composite loss function that matches the Python DidgeLab implementation.

use crate::geo::Geo;
use crate::sim::{acoustical_simulation, get_log_simulation_frequencies, get_fundamental};
use crate::evo::TargetSound;

/// Main Tairua loss function that combines multiple acoustic criteria
pub struct TairuaLoss {
    target_frequency: Option<f64>,
    target_notes: Vec<String>,
    weight_fundamental: f64,
    weight_harmonics: f64,
    weight_peaks: f64,
    frequency_tolerance: f64,
}

impl TairuaLoss {
    /// Create a new Tairua loss function with default parameters
    pub fn new() -> Self {
        Self {
            target_frequency: None,
            target_notes: Vec::new(),
            weight_fundamental: 1.0,
            weight_harmonics: 1.0,
            weight_peaks: 1.0,
            frequency_tolerance: 5.0, // 5 Hz tolerance
        }
    }
    
    /// Set target fundamental frequency
    pub fn with_target_frequency(mut self, freq: f64) -> Self {
        self.target_frequency = Some(freq);
        self
    }
    
    /// Set target notes (will convert to frequencies internally)
    pub fn with_target_notes(mut self, notes: Vec<&str>) -> Self {
        self.target_notes = notes.iter().map(|&s| s.to_string()).collect();
        self
    }
    
    /// Set weights for different loss components
    pub fn with_weights(mut self, fundamental: f64, harmonics: f64, peaks: f64) -> Self {
        self.weight_fundamental = fundamental;
        self.weight_harmonics = harmonics;
        self.weight_peaks = peaks;
        self
    }
    
    /// Compute the total loss for a given geometry
    pub fn compute_loss(&self, geo: &Geo) -> Result<f64, String> {
        let frequencies = get_log_simulation_frequencies();
        let _impedances = match acoustical_simulation(geo, &frequencies, "tlm_python") {
            Ok(imp) => imp,
            Err(e) => return Err(format!("Simulation failed: {}", e)),
        };
        
        let mut total_loss = 0.0;
        
        // Fundamental frequency loss
        if let Some(target_freq) = self.target_frequency {
            if let Ok((fundamental, _)) = get_fundamental(geo, "tlm_python", 20.0) {
                let freq_diff = (fundamental - target_freq).abs();
                let fundamental_loss = (freq_diff / self.frequency_tolerance).powi(2);
                total_loss += self.weight_fundamental * fundamental_loss;
            }
        }
        
        // Peak detection and harmonic alignment loss
        // Note: We can't use impedances anymore since it's been renamed to _impedances
        // So we recompute the simulation to get the impedances
        let frequencies = get_log_simulation_frequencies();
        let impedances = match acoustical_simulation(geo, &frequencies, "tlm_python") {
            Ok(imp) => imp,
            Err(e) => return Err(format!("Simulation failed: {}", e)),
        };
        
        let peak_indices = self.find_peaks(&impedances);
        if !peak_indices.is_empty() {
            let peak_frequencies: Vec<f64> = peak_indices.iter()
                .map(|&i| frequencies[i])
                .collect();
            
            // Calculate harmonic alignment loss
            if let Some(base_freq) = peak_frequencies.first() {
                let mut harmonic_loss = 0.0;
                for (i, &freq) in peak_frequencies.iter().enumerate().skip(1) {
                    let expected_harmonic = base_freq * (i + 1) as f64;
                    let harmonic_deviation = (freq - expected_harmonic).abs() / expected_harmonic;
                    harmonic_loss += harmonic_deviation;
                }
                total_loss += self.weight_harmonics * harmonic_loss / peak_frequencies.len().max(1) as f64;
            }
        }
        
        Ok(total_loss)
    }
    
    /// Find peaks in the impedance spectrum
    fn find_peaks(&self, impedances: &[f64]) -> Vec<usize> {
        let mut peaks = Vec::new();
        
        for i in 1..impedances.len() - 1 {
            if impedances[i] > impedances[i-1] && impedances[i] > impedances[i+1] {
                peaks.push(i);
            }
        }
        
        peaks
    }
}

impl Default for TairuaLoss {
    fn default() -> Self {
        Self::new()
    }
}

/// Alternative loss functions for specific optimization goals

/// Simple loss based on deviation from target fundamental frequency
pub struct FundamentalFrequencyLoss {
    target_frequency: f64,
    tolerance: f64,
}

impl FundamentalFrequencyLoss {
    pub fn new(target_frequency: f64) -> Self {
        Self {
            target_frequency,
            tolerance: 5.0,
        }
    }
    
    pub fn compute_loss(&self, geo: &Geo) -> Result<f64, String> {
        let frequencies = get_log_simulation_frequencies();
        let _impedances = match acoustical_simulation(geo, &frequencies, "tlm_python") {
            Ok(imp) => imp,
            Err(e) => return Err(format!("Simulation failed: {}", e)),
        };
        
        if let Ok((fundamental, _)) = get_fundamental(geo, "tlm_python", 20.0) {
            let diff = (fundamental - self.target_frequency).abs();
            Ok((diff / self.tolerance).powi(2))
        } else {
            Ok(f64::MAX / 2.0) // Very high loss if no fundamental detected
        }
    }
}

/// Loss based on geometric properties (length, taper, etc.)
pub struct GeometricLoss {
    target_length: Option<f64>,
    target_taper: Option<f64>,
}

impl GeometricLoss {
    pub fn new() -> Self {
        Self {
            target_length: None,
            target_taper: None,
        }
    }
    
    pub fn with_target_length(mut self, length: f64) -> Self {
        self.target_length = Some(length);
        self
    }
    
    pub fn with_target_taper(mut self, taper: f64) -> Self {
        self.target_taper = Some(taper);
        self
    }
    
    pub fn compute_loss(&self, geo: &Geo) -> Result<f64, String> {
        let mut loss = 0.0;
        
        if let Some(target_length) = self.target_length {
            let length_diff = (geo.length() - target_length).abs() / target_length;
            loss += length_diff;
        }
        
        // Use taper ratio from the Geo struct
        if let Some(target_taper) = self.target_taper {
            let current_taper = geo.taper_ratio();
            let taper_diff = (current_taper - target_taper).abs() / target_taper.max(0.001);
            loss += taper_diff;
        }
        
        Ok(loss)
    }
}

/// Multi-objective loss combining acoustic and geometric factors
pub struct MultiObjectiveLoss {
    acoustic_loss: TairuaLoss,
    geometric_loss: GeometricLoss,
    acoustic_weight: f64,
    geometric_weight: f64,
}

impl MultiObjectiveLoss {
    pub fn new(acoustic_weight: f64, geometric_weight: f64) -> Self {
        Self {
            acoustic_loss: TairuaLoss::new(),
            geometric_loss: GeometricLoss::new(),
            acoustic_weight,
            geometric_weight,
        }
    }
    
    pub fn with_acoustic_target_frequency(mut self, freq: f64) -> Self {
        self.acoustic_loss = self.acoustic_loss.with_target_frequency(freq);
        self
    }
    
    pub fn with_geometric_target_length(mut self, length: f64) -> Self {
        self.geometric_loss = self.geometric_loss.with_target_length(length);
        self
    }
    
    pub fn compute_loss(&self, geo: &Geo) -> Result<f64, String> {
        let acoustic_result = self.acoustic_loss.compute_loss(geo);
        let geometric_result = self.geometric_loss.compute_loss(geo);
        
        match (acoustic_result, geometric_result) {
            (Ok(ac_loss), Ok(geom_loss)) => {
                Ok(self.acoustic_weight * ac_loss + self.geometric_weight * geom_loss)
            }
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    }
}

/// DidgeLab-style comprehensive loss function for inverse design
/// Matches the web app's capability: "Describe the sound you want... and find matching geometry"
pub struct DidgeLabLoss {
    target_sound: TargetSound,
    weight_fundamental: f64,
    weight_toots: f64,
    weight_overtones: f64,
    weight_bore_shape: f64,
    weight_volume: f64,
    frequency_tolerance: f64,
}

impl DidgeLabLoss {
    pub fn new(target_sound: TargetSound) -> Self {
        Self {
            target_sound,
            weight_fundamental: 2.0,  // Fundamental is most important
            weight_toots: 1.5,        // Toots are important
            weight_overtones: 1.0,    // Overtone alignment
            weight_bore_shape: 0.5,   // Shape preference
            weight_volume: 0.3,       // Volume constraint
            frequency_tolerance: 3.0, // 3 Hz tolerance
        }
    }
    
    /// Set weights for different loss components
    pub fn with_weights(
        mut self,
        fundamental: f64,
        toots: f64,
        overtones: f64,
        bore_shape: f64,
        volume: f64,
    ) -> Self {
        self.weight_fundamental = fundamental;
        self.weight_toots = toots;
        self.weight_overtones = overtones;
        self.weight_bore_shape = bore_shape;
        self.weight_volume = volume;
        self
    }
    
    /// Compute comprehensive loss for target sound
    pub fn compute_loss(&self, geo: &Geo) -> Result<f64, String> {
        // Run acoustic simulation
        let frequencies = get_log_simulation_frequencies();
        let impedances = acoustical_simulation(geo, &frequencies, "tlm_python")
            .map_err(|e| format!("Simulation failed: {}", e))?;
        
        let mut total_loss = 0.0;
        
        // 1. Fundamental frequency loss
        if let Ok((fundamental, fund_imp)) = get_fundamental(geo, "tlm_python", 20.0) {
            let freq_diff = (fundamental - self.target_sound.fundamental_freq).abs();
            let fundamental_loss = (freq_diff / self.frequency_tolerance).powi(2);
            
            // Weight by impedance strength (stronger resonance = better)
            let impedance_weight = 1.0 / (1.0 + fund_imp / 1e8);
            total_loss += self.weight_fundamental * fundamental_loss * impedance_weight;
        } else {
            total_loss += self.weight_fundamental * 10.0; // High penalty if no fundamental
        }
        
        // 2. Toot frequency loss (target resonance peaks)
        if !self.target_sound.toots.is_empty() {
            let peak_indices = self.find_peaks(&impedances, 5); // Top 5 peaks
            let peak_frequencies: Vec<f64> = peak_indices.iter()
                .map(|&i| frequencies[i])
                .collect();
            
            let mut toot_loss = 0.0;
            for &target_toot in &self.target_sound.toots {
                // Find closest peak to this toot
                let min_diff = peak_frequencies.iter()
                    .map(|&peak_freq| (peak_freq - target_toot).abs())
                    .fold(f64::INFINITY, f64::min);
                
                if min_diff == f64::INFINITY {
                    toot_loss += 5.0; // High penalty if no peaks found
                } else {
                    toot_loss += (min_diff / self.frequency_tolerance).powi(2);
                }
            }
            total_loss += self.weight_toots * toot_loss / self.target_sound.toots.len() as f64;
        }
        
        // 3. Overtone alignment loss
        if !self.target_sound.overtones.is_empty() {
            let fund = self.target_sound.fundamental_freq;
            let peak_indices = self.find_peaks(&impedances, 10);
            let peak_frequencies: Vec<f64> = peak_indices.iter()
                .map(|&i| frequencies[i])
                .collect();
            
            let mut overtone_loss = 0.0;
            for &harmonic_num in &self.target_sound.overtones {
                let expected_freq = fund * harmonic_num as f64;
                
                // Find closest peak
                let min_diff = peak_frequencies.iter()
                    .map(|&peak_freq| (peak_freq - expected_freq).abs() / expected_freq)
                    .fold(f64::INFINITY, f64::min);
                
                if min_diff < f64::INFINITY {
                    overtone_loss += min_diff;
                } else {
                    overtone_loss += 0.1; // Small penalty for missing overtone
                }
            }
            total_loss += self.weight_overtones * overtone_loss / self.target_sound.overtones.len() as f64;
        }
        
        // 4. Bore shape preference loss
        total_loss += self.compute_bore_shape_loss(geo);
        
        // 5. Volume constraint loss
        total_loss += self.compute_volume_loss(geo);
        
        // 6. Length constraint penalty
        let length = geo.length();
        if length < self.target_sound.length_range.0 || length > self.target_sound.length_range.1 {
            let violation = if length < self.target_sound.length_range.0 {
                self.target_sound.length_range.0 - length
            } else {
                length - self.target_sound.length_range.1
            };
            total_loss += (violation / 100.0).powi(2);
        }
        
        // 7. Bell diameter constraint penalty
        let bell = geo.bellsize();
        if bell < self.target_sound.bell_range.0 || bell > self.target_sound.bell_range.1 {
            let violation = if bell < self.target_sound.bell_range.0 {
                self.target_sound.bell_range.0 - bell
            } else {
                bell - self.target_sound.bell_range.1
            };
            total_loss += (violation / 10.0).powi(2);
        }
        
        Ok(total_loss)
    }
    
    /// Compute bore shape preference loss
    fn compute_bore_shape_loss(&self, geo: &Geo) -> f64 {
        use crate::evo::BoreShapePreference;
        
        match self.target_sound.bore_shape {
            BoreShapePreference::Any => 0.0,
            BoreShapePreference::Cylindrical => {
                // Penalize taper (difference between start and end diameter)
                let start_diam = geo.diameter_at_x(0.0);
                let end_diam = geo.bellsize();
                let taper = (end_diam - start_diam) / start_diam;
                (taper / 0.2).powi(2) // Prefer taper < 20%
            },
            BoreShapePreference::Conical => {
                // Prefer smooth, linear taper
                let start_diam = geo.diameter_at_x(0.0);
                let end_diam = geo.bellsize();
                let taper = (end_diam - start_diam) / start_diam;
                
                // Ideal taper for conical is 50-150%
                if taper < 0.5 {
                    ((0.5 - taper) / 0.5).powi(2)
                } else if taper > 1.5 {
                    ((taper - 1.5) / 0.5).powi(2)
                } else {
                    0.0 // Good conical taper
                }
            },
            BoreShapePreference::Flared => {
                // Prefer strong flare at end
                let start_diam = geo.diameter_at_x(0.0);
                let mid_diam = geo.diameter_at_x(geo.length() * 0.7);
                let end_diam = geo.bellsize();
                
                let first_half_taper = (mid_diam - start_diam) / start_diam;
                let second_half_taper = (end_diam - mid_diam) / mid_diam;
                
                // Flared means second half should taper more than first half
                if second_half_taper > first_half_taper {
                    0.0 // Good flare
                } else {
                    ((first_half_taper - second_half_taper) / first_half_taper.max(0.01)).powi(2)
                }
            },
        }
    }
    
    /// Compute volume constraint loss
    fn compute_volume_loss(&self, geo: &Geo) -> f64 {
        let volume = geo.compute_volume();
        
        // Estimate expected volume from length and bell size
        let length = geo.length();
        let bell = geo.bellsize();
        let expected_volume = std::f64::consts::PI * length * (bell / 2.0).powi(2) / 3.0;
        
        // Penalize if volume is too far from expected
        let ratio = volume / expected_volume;
        if ratio < 0.5 || ratio > 2.0 {
            ((ratio - 1.0).abs() / 0.5).powi(2)
        } else {
            0.0
        }
    }
    
    /// Find top N peaks in impedance spectrum
    fn find_peaks(&self, impedances: &[f64], n: usize) -> Vec<usize> {
        let mut peaks = Vec::new();
        
        for i in 1..impedances.len() - 1 {
            if impedances[i] > impedances[i-1] && impedances[i] > impedances[i+1] {
                peaks.push((i, impedances[i]));
            }
        }
        
        // Sort by impedance magnitude (descending)
        peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Return top N peak indices
        peaks.into_iter().take(n).map(|(i, _)| i).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::Geo;
    
    #[test]
    fn test_tairua_loss_creation() {
        let loss = TairuaLoss::new()
            .with_target_frequency(65.41) // C2 note
            .with_weights(1.0, 0.5, 0.5);
        
        let geo = Geo::make_cone(1500.0, 32.0, 65.0, 20);
        let result = loss.compute_loss(&geo);
        
        assert!(result.is_ok());
        let loss_value = result.unwrap();
        assert!(loss_value >= 0.0);
    }
    
    #[test]
    fn test_fundamental_frequency_loss() {
        let loss = FundamentalFrequencyLoss::new(65.41); // C2 note
        
        let geo = Geo::make_cone(1500.0, 32.0, 65.0, 20);
        let result = loss.compute_loss(&geo);
        
        assert!(result.is_ok());
        let loss_value = result.unwrap();
        assert!(loss_value >= 0.0);
    }
    
    #[test]
    fn test_geometric_loss() {
        let loss = GeometricLoss::new()
            .with_target_length(1500.0);
        
        let geo = Geo::make_cone(1500.0, 32.0, 65.0, 20);
        let result = loss.compute_loss(&geo);
        
        assert!(result.is_ok());
        let loss_value = result.unwrap();
        assert!(loss_value >= 0.0);
    }
    
    #[test]
    fn test_multi_objective_loss() {
        let loss = MultiObjectiveLoss::new(1.0, 0.5)
            .with_acoustic_target_frequency(65.41)
            .with_geometric_target_length(1500.0);
        
        let geo = Geo::make_cone(1500.0, 32.0, 65.0, 20);
        let result = loss.compute_loss(&geo);
        
        assert!(result.is_ok());
        let loss_value = result.unwrap();
        assert!(loss_value >= 0.0);
    }
}