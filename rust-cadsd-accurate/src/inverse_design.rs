//! Inverse design module - matches DidgeLab web app functionality
//!
//! This module implements the core DidgeLab feature:
//! "Describe the sound you want... and find matching geometry"
//!
//! Users specify target acoustic properties, and the system uses
//! evolutionary optimization to find geometries that match.

use crate::geo::Geo;
use crate::evo::{Nuevolution, GeoGenome, TargetSound, LossFunctionType};
use crate::loss::DidgeLabLoss;
use rand;

/// Result of inverse design optimization
#[derive(Clone, Debug)]
pub struct DesignResult {
    /// Best geometry found
    pub geometry: Geo,
    /// Fundamental frequency (Hz)
    pub fundamental_freq: f64,
    /// Resonance peaks (frequency, impedance)
    pub resonances: Vec<(f64, f64)>,
    /// Final loss value
    pub loss: f64,
    /// Number of generations evolved
    pub generations: usize,
    /// All candidate geometries (top N)
    pub candidates: Vec<Geo>,
}

impl DesignResult {
    /// Print summary of design result
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("=== Design Result ===\n"));
        s.push_str(&format!("Fundamental: {:.2} Hz\n", self.fundamental_freq));
        s.push_str(&format!("Loss: {:.6}\n", self.loss));
        s.push_str(&format!("Generations: {}\n", self.generations));
        s.push_str(&format!("Geometry length: {:.1} mm\n", self.geometry.length()));
        s.push_str(&format!("Bell diameter: {:.1} mm\n", self.geometry.bellsize()));
        s.push_str(&format!("Resonances: {}\n", self.resonances.len()));
        
        for (i, (freq, imp)) in self.resonances.iter().take(5).enumerate() {
            s.push_str(&format!("  Peak {}: {:.1} Hz (imp: {:.2e})\n", i + 1, freq, imp));
        }
        
        s
    }
}

/// Inverse design engine - main interface like DidgeLab web app
pub struct InverseDesigner {
    population_size: usize,
    generations: usize,
    verbose: bool,
    return_top_n: usize,
}

impl InverseDesigner {
    pub fn new() -> Self {
        Self {
            population_size: 50,
            generations: 100,
            verbose: true,
            return_top_n: 5,
        }
    }
    
    /// Set population size for evolution
    pub fn with_population_size(mut self, size: usize) -> Self {
        self.population_size = size;
        self
    }
    
    /// Set number of generations
    pub fn with_generations(mut self, gens: usize) -> Self {
        self.generations = gens;
        self
    }
    
    /// Set verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// Set number of top candidates to return
    pub fn with_top_n(mut self, n: usize) -> Self {
        self.return_top_n = n;
        self
    }
    
    /// Design didgeridoo from target sound description
    /// This is the main DidgeLab feature: "Describe the sound, get geometry"
    pub fn design(&self, target: TargetSound) -> Result<DesignResult, String> {
        self.design_with_progress(target, None)
    }

    /// Design didgeridoo with an optional progress callback
    pub fn design_with_progress(
        &self,
        target: TargetSound,
        progress_cb: Option<std::sync::Arc<dyn Fn(usize, f64) + Send + Sync>>,
    ) -> Result<DesignResult, String> {
        if self.verbose {
            println!("🎯 Target sound:");
            println!("   Fundamental: {:.2} Hz", target.fundamental_freq);
            println!("   Toots: {:?}", target.toots);
            println!("   Overtones: {:?}", target.overtones);
            println!("   Bore shape: {}", target.bore_shape);
            println!();
            println!("🧬 Starting evolutionary optimization...");
            println!("   Population: {}", self.population_size);
            println!("   Generations: {}", self.generations);
        }
        
        // Create loss function for this target
        let loss_fn = DidgeLabLoss::new(target.clone());
        let loss_fn_type = LossFunctionType::DidgeLabLoss(loss_fn);
        
        // Generate initial population
        let initial_population = self.generate_initial_population(&target)?;
        
        // Create evolution engine
        let evolver = Nuevolution::new(self.population_size, self.generations)
            .set_mutation_rate(0.15)
            .set_crossover_rate(0.8)
            .set_elite_size(3)
            .set_verbose(self.verbose);
        
        // Run evolution
        let final_population = evolver.evolve(
            initial_population,
            &loss_fn_type,
            progress_cb.as_ref().map(|arc| &**arc),
        )?;
        
        // Extract best result
        let best_genome = &final_population[0];
        let best_geo = best_genome.to_geo();
        
        // Run final analysis on best geometry
        let (fund_freq, _fund_imp) = crate::sim::get_fundamental(&best_geo, "tlm_python", 20.0)
            .unwrap_or((target.fundamental_freq, 0.0));
        
        // Find resonances
        let resonances = self.find_resonances(&best_geo)?;
        
        // Collect top N candidates
        let candidates: Vec<Geo> = final_population.iter()
            .take(self.return_top_n)
            .map(|g| g.to_geo())
            .collect();
        
        let result = DesignResult {
            geometry: best_geo,
            fundamental_freq: fund_freq,
            resonances,
            loss: best_genome.fitness.unwrap_or(f64::NAN),
            generations: self.generations,
            candidates,
        };
        
        if self.verbose {
            println!("\n✅ Optimization complete!");
            println!("{}", result.summary());
        }
        
        Ok(result)
    }
    
    /// Generate initial population based on target sound
    fn generate_initial_population(&self, target: &TargetSound) -> Result<Vec<GeoGenome>, String> {
        let mut population = Vec::with_capacity(self.population_size);
        
        // Estimate rough geometry from fundamental frequency
        // Quarter-wave resonator: L ≈ c / (4 * f)
        let speed_of_sound = 343000.0; // mm/s
        let estimated_length = speed_of_sound / (4.0 * target.fundamental_freq);
        
        // Clamp to reasonable range
        let estimated_length = estimated_length.max(target.length_range.0)
            .min(target.length_range.1);
        
        // Generate diverse initial population
        for i in 0..self.population_size {
            let genes = if i < 5 {
                // First 5 are educated guesses near estimated values
                let length = estimated_length * (0.9 + 0.2 * rand::random::<f64>());
                let top_diam = 25.0 + 15.0 * rand::random::<f64>();
                let bot_diam = top_diam * (1.2 + 1.0 * rand::random::<f64>());
                let segments = 20.0 + 10.0 * rand::random::<f64>();
                vec![length, top_diam, bot_diam, segments]
            } else {
                // Rest are diverse random samples
                let length = target.length_range.0 
                    + (target.length_range.1 - target.length_range.0) * rand::random::<f64>();
                let top_diam = 15.0 + 35.0 * rand::random::<f64>();
                let bot_diam = target.bell_range.0 
                    + (target.bell_range.1 - target.bell_range.0) * rand::random::<f64>();
                let segments = 15.0 + 20.0 * rand::random::<f64>();
                vec![length, top_diam, bot_diam, segments]
            };
            
            population.push(GeoGenome::new(genes));
        }
        
        if self.verbose {
            println!("   Generated {} initial candidates", population.len());
        }
        
        Ok(population)
    }
    
    /// Find resonance peaks for a geometry
    fn find_resonances(&self, geo: &Geo) -> Result<Vec<(f64, f64)>, String> {
        let frequencies = crate::sim::get_log_simulation_frequencies();
        let impedances = crate::sim::acoustical_simulation(geo, &frequencies, "tlm_python")
            .map_err(|e| format!("Simulation failed: {}", e))?;
        
        let peaks = crate::analysis::get_notes(&frequencies, &impedances);
        Ok(peaks)
    }
}

impl Default for InverseDesigner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inverse_design_basic() {
        // Design a didgeridoo in D1 (≈73.4 Hz)
        let target = TargetSound::new(73.4);
        
        let designer = InverseDesigner::new()
            .with_population_size(20)
            .with_generations(10)
            .with_verbose(false);
        
        let result = designer.design(target);
        assert!(result.is_ok());
        
        let design = result.unwrap();
        assert!(design.fundamental_freq > 50.0);
        assert!(design.fundamental_freq < 100.0);
        assert!(!design.resonances.is_empty());
    }
    
    #[test]
    fn test_design_with_toots() {
        // Design with specific toots
        let target = TargetSound::new(65.4) // C2
            .with_toot(196.0) // G3
            .with_toot(261.6); // C4
        
        let designer = InverseDesigner::new()
            .with_population_size(15)
            .with_generations(8)
            .with_verbose(false);
        
        let result = designer.design(target);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_design_with_bore_preference() {
        use crate::evo::BoreShapePreference;
        
        let target = TargetSound::from_note("D1")
            .unwrap()
            .with_bore_shape(BoreShapePreference::Conical);
        
        let designer = InverseDesigner::new()
            .with_population_size(15)
            .with_generations(8)
            .with_verbose(false);
        
        let result = designer.design(target);
        assert!(result.is_ok());
        
        let design = result.unwrap();
        assert!(design.geometry.length() > 1000.0);
    }
}
