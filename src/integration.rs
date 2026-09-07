use crate::{Geo, sim::DidgeridooSimulator, evo::{EvolutionaryOptimizer, EvolutionParameters, MutationStrategy, CrossoverStrategy}, loss::{CompositeTairuaLoss, FrequencyTuningLoss}};

#[derive(Debug)]
pub struct DefaultSimulator;

impl DefaultSimulator {
    pub fn simulate(&self, geometry: &Geo, frequencies: &[f64]) -> Vec<f64> {
        let simulator = DidgeridooSimulator::from_geo(&geometry.geo);
        let spectrum = simulator.impedance(frequencies);
        spectrum.iter().map(|c| c.norm()).collect()
    }

    pub fn get_fundamental(&self, geometry: &Geo) -> Option<f64> {
        let simulator = DidgeridooSimulator::from_geo(&geometry.geo);
        let peaks = simulator.find_resonance_peaks();
        peaks.first().map(|p| p.frequency)
    }
}

#[derive(Debug)]
pub struct DefaultOptimizer;

impl DefaultOptimizer {
    pub fn optimize(&self, target_frequency: f64, population_size: usize, generations: usize) -> Geo {
        let genome = crate::evo::KigaliGenome::new(
            20, 32.0, 50.0, 80.0, 1800.0, 1500.0, 0, 0.3, 0.2, target_frequency, 0
        );
        let mut loss = CompositeTairuaLoss::new(5.0);
        loss.add_component(
            "frequency_tuning".to_string(),
            Box::new(FrequencyTuningLoss::new(
                vec![target_frequency],
                vec![100.0],
                vec![10.0],
            )),
        );
        
        let params = EvolutionParameters {
            population_size: population_size.min(20),
            generation_size: 10,
            num_generations: generations.min(10),
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            elite_size: 2,
            mutation_strategy: MutationStrategy::Gaussian,
            crossover_strategy: CrossoverStrategy::SinglePoint,
            convergence_patience: 5,
            convergence_threshold: 0.01,
        };
        
        let mut optimizer = EvolutionaryOptimizer::with_random_population(
            Box::new(loss), &genome, population_size.min(20), params
        );
        
        if let Ok(best) = optimizer.evolve() {
            let (geo, _) = best.geo_and_toneholes();
            return geo;
        }
        
        Geo::make_cone(1500.0, 32.0, 65.0, 30)
    }
}