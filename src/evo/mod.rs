//! Evolutionary optimization for didgeridoo shape design
//!
//! This module implements genetic algorithms for optimizing didgeridoo geometries
//! to achieve target acoustic properties using the CADSD framework.

use crate::geo::Geo;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::f64::consts::PI;
use rand_distr::Distribution;

/// Crossover strategy for evolutionary optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossoverStrategy {
    /// Single-point crossover
    SinglePoint,
    /// Average of both parents
    Average,
    /// Swap a contiguous segment between parents
    PartSwap,
    /// Average a contiguous segment between parents
    PartAverage,
}

/// Mutation strategy for evolutionary optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationStrategy {
    /// Standard Gaussian mutation
    Gaussian,
    /// Prime-number indexed mutation for better space-filling exploration
    PrimeSequence,
    /// Mutate exactly one gene per individual
    SingleMutation,
}

/// Trait for evolvable genomes
pub trait Genome: Send + Sync {
    /// Get the genome vector (genes in [0,1] range)
    fn genome(&self) -> &[f64];
    
    /// Set the genome vector
    fn set_genome(&mut self, genome: Vec<f64>);
    
    /// Get mutable reference to genome
    fn genome_mut(&mut self) -> &mut [f64];
    
    /// Create a random genome of specified size
    fn random(n_genes: usize) -> Self
    where
        Self: Sized;
    
    /// Clone the genome with a new ID (invalidates cached loss)
    fn clone_with_new_id(&self) -> Box<dyn Genome>;
    
    /// Clone the genome preserving its loss (for elite selection)
    fn clone_with_loss(&self) -> Box<dyn Genome>;
    
    /// Convert genome to geometry
    fn genome2geo(&self) -> Geo;
    
    /// Get unique identifier
    fn id(&self) -> u64;
    
    /// Get/set loss value
    fn loss(&self) -> Option<f64>;
    fn set_loss(&mut self, loss: Option<f64>);
    
    /// Get representation for logging
    fn representation(&self) -> serde_json::Value;
}

/// Base genome implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseGenome {
    genome: Vec<f64>,
    id: u64,
    loss: Option<f64>,
}

/// Prime number generator for prime-indexed mutation strategies
pub struct PrimeGenerator {
    primes: Vec<u32>,
    current_index: usize,
}

impl PrimeGenerator {
    /// Initialize with a list of primes up to a specified limit
    pub fn new(limit: u32) -> Self {
        let mut sieve = vec![true; (limit + 1) as usize];
        sieve[0] = false;
        sieve[1] = false;

        for i in 2..=((limit as f64).sqrt() as usize) {
            if sieve[i] {
                for j in (i * i..=limit as usize).step_by(i) {
                    sieve[j] = false;
                }
            }
        }

        let primes: Vec<_> = sieve.into_iter().enumerate().filter(|(_, is_prime)| *is_prime)
            .map(|(p, _)| p as u32)
            .collect();

        Self {
            primes,
            current_index: 0,
        }
    }

    /// Get the next prime number in the sequence
    pub fn next_prime(&mut self) -> u32 {
        let prime = self.primes[self.current_index % self.primes.len()];
        self.current_index += 1;
        prime
    }

    /// Get a prime at a specific index
    pub fn nth(&self, index: usize) -> u32 {
        self.primes[index % self.primes.len()]
    }
}

impl Default for PrimeGenerator {
    fn default() -> Self {
        Self::new(1000)
    }
}

static GENOME_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

impl BaseGenome {
    fn generate_id() -> u64 {
        GENOME_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
    }
}

impl Genome for BaseGenome {
    fn genome(&self) -> &[f64] {
        &self.genome
    }
    
    fn set_genome(&mut self, genome: Vec<f64>) {
        self.genome = genome;
        self.loss = None;
    }
    
    fn genome_mut(&mut self) -> &mut [f64] {
        &mut self.genome
    }
    
    fn random(n_genes: usize) -> Self {
        let mut genome = vec![0.0; n_genes];
        for gene in &mut genome {
            *gene = rand::random::<f64>();
        }
        
        Self {
            genome,
            id: Self::generate_id(),
            loss: None,
        }
    }
    
    fn clone_with_new_id(&self) -> Box<dyn Genome> {
        Box::new(Self {
            genome: self.genome.clone(),
            id: Self::generate_id(),
            loss: None,
        })
    }
    
    fn clone_with_loss(&self) -> Box<dyn Genome> {
        Box::new(Self {
            genome: self.genome.clone(),
            id: Self::generate_id(),
            loss: self.loss,
        })
    }
    
    fn genome2geo(&self) -> Geo {
        // Default implementation - should be overridden by specific genome types
        Geo::make_cone(1500.0, 32.0, 60.0, 20)
    }
    
    fn id(&self) -> u64 {
        self.id
    }
    
    fn loss(&self) -> Option<f64> {
        self.loss
    }
    
    fn set_loss(&mut self, loss: Option<f64>) {
        self.loss = loss;
    }
    
    fn representation(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "genome": self.genome,
            "loss": self.loss
        })
    }
}

/// Kigali-style parametric shape genome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KigaliGenome {
    base: BaseGenome,
    n_segments: usize,
    d0: f64,           // mouth diameter (mm)
    d_bell_min: f64,   // minimum bell diameter (mm)
    d_bell_max: f64,   // maximum bell diameter (mm)
    max_length: f64,   // maximum length (mm)
    min_length: f64,   // minimum length (mm)
    n_bubbles: usize,   // number of bubbles
    smoothness: f64,   // smoothness parameter
    bell_accent: f64,  // bell accent factor
    bell_start: f64,   // bell start position (mm)
}

impl KigaliGenome {
    pub fn new(
        n_segments: usize,
        d0: f64,
        d_bell_min: f64,
        d_bell_max: f64,
        max_length: f64,
        min_length: f64,
        n_bubbles: usize,
        smoothness: f64,
        bell_accent: f64,
        bell_start: f64,
    ) -> Self {
        let n_genes = 3 + 2 * (n_segments - 1) + n_bubbles * 3;
        
        Self {
            base: BaseGenome::random(n_genes),
            n_segments,
            d0,
            d_bell_min,
            d_bell_max,
            max_length,
            min_length,
            n_bubbles,
            smoothness,
            bell_accent,
            bell_start,
        }
    }
    
    /// Decode genome parameters
    fn decode_parameters(&self) -> (f64, f64, f64, Vec<f64>, Vec<f64>, Vec<(f64, f64, f64)>) {
        let genome = self.base.genome();
        
        // Length and bell size
        let length = genome[0] * (self.max_length - self.min_length) + self.min_length;
        let bell_size = genome[1] * (self.d_bell_max - self.d_bell_min) + self.d_bell_min;
        let power = genome[2] * 4.0;
        
        // Bubbles
        let mut bubbles = Vec::new();
        let bubble_width = 300.0;
        let bubble_height = 40.0;
        
        for i in 0..self.n_bubbles {
            let idx = 3 + i * 3;
            if idx + 2 < genome.len() {
                let pos = bubble_width + genome[idx] * (length - 2.0 * bubble_width);
                let width = bubble_width * (0.2 + genome[idx + 1]) / 1.2;
                let height = (0.2 + genome[idx + 2]) * bubble_height / 1.2;
                bubbles.push((pos, width, height));
            }
        }
        
        // Segment offsets
        let geo_offset = 3 + self.n_bubbles * 3;
        let mut x_genome = Vec::new();
        let mut y_genome = Vec::new();
        
        let mut i = geo_offset;
        while i + 1 < genome.len() {
            x_genome.push(genome[i]);
            y_genome.push(genome[i + 1]);
            i += 2;
        }
        
        (length, bell_size, power, x_genome, y_genome, bubbles)
    }
    
    /// Apply bell accent to geometry
    fn apply_bell_accent(&self, x: &mut [f64], y: &mut [f64], length: f64, bell_size: f64) {
        if self.bell_accent <= 0.0 {
            return;
        }
        
        let bell_start_pos = length - self.bell_start;
        if bell_start_pos < 1.0 {
            return;
        }
        
        // Simple bell accent implementation
        for i in 0..x.len() {
            if x[i] > bell_start_pos {
                let t = (x[i] - bell_start_pos) / (length - bell_start_pos);
                y[i] += self.bell_accent * (bell_size - y[i]) * t;
            }
        }
    }
    
    /// Add bubble to geometry
    fn make_bubble(x: &mut Vec<f64>, y: &mut Vec<f64>, pos: f64, width: f64, height: f64) {
        let bubble_start = pos - width / 2.0;
        let bubble_end = pos + width / 2.0;
        
        // Find indices for insertion
        let start_idx = x.iter().position(|&xi| xi >= bubble_start).unwrap_or(x.len());
        let end_idx = x.iter().position(|&xi| xi >= bubble_end).unwrap_or(x.len());
        
        // Create bubble points
        let n_bubble_points = 10;
        let mut bubble_x = Vec::new();
        let mut bubble_y = Vec::new();
        
        for i in 0..n_bubble_points {
            let t = i as f64 / (n_bubble_points - 1) as f64;
            let bubble_pos = bubble_start + t * width;
            
            // Interpolate original diameter
            let original_diameter = if !x.is_empty() {
                let geo = Geo::new(x.iter().zip(y.iter()).map(|(&xi, &yi)| [xi, yi]).collect());
                geo.diameter_at_x(bubble_pos)
            } else {
                32.0
            };
            
            // Add sinusoidal bulge
            let bulge = height * (PI * t).sin();
            bubble_x.push(bubble_pos);
            bubble_y.push(original_diameter + bulge);
        }
        
        // Insert bubble points
        x.splice(start_idx..end_idx, bubble_x);
        y.splice(start_idx..end_idx, bubble_y);
    }
}

impl Genome for KigaliGenome {
    fn genome(&self) -> &[f64] {
        self.base.genome()
    }
    
    fn set_genome(&mut self, genome: Vec<f64>) {
        self.base.set_genome(genome);
    }
    
    fn genome_mut(&mut self) -> &mut [f64] {
        self.base.genome_mut()
    }
    
    fn random(n_genes: usize) -> Self {
        Self {
            base: BaseGenome::random(n_genes),
            n_segments: 24,
            d0: 32.0,
            d_bell_min: 50.0,
            d_bell_max: 80.0,
            max_length: 1900.0,
            min_length: 1500.0,
            n_bubbles: 0,
            smoothness: 0.3,
            bell_accent: 0.0,
            bell_start: 300.0,
        }
    }
    
    fn clone_with_new_id(&self) -> Box<dyn Genome> {
        Box::new(Self {
            base: BaseGenome {
                genome: self.base.genome.clone(),
                id: BaseGenome::generate_id(),
                loss: None,
            },
            ..*self
        })
    }
    
    fn clone_with_loss(&self) -> Box<dyn Genome> {
        Box::new(Self {
            base: BaseGenome {
                genome: self.base.genome.clone(),
                id: BaseGenome::generate_id(),
                loss: self.base.loss,
            },
            ..*self
        })
    }
    
    fn genome2geo(&self) -> Geo {
        let (length, bell_size, power, x_genome, y_genome, bubbles) = self.decode_parameters();
        
        // Generate base geometry (power-law taper)
        let mut x: Vec<f64> = (0..=self.n_segments)
            .map(|i| length * i as f64 / self.n_segments as f64)
            .collect();
        
        let mut y: Vec<f64> = (0..=self.n_segments)
            .map(|i| {
                let t = i as f64 / self.n_segments as f64;
                t.powf(power) * (bell_size - self.d0) + self.d0
            })
            .collect();
        
        // Apply genome jitter/offsets
        let shift_x = length / self.n_segments as f64;
        for (i, &offset) in x_genome.iter().enumerate() {
            if i < x.len() && i > 0 && i < x.len() - 1 {
                x[i] += (offset - 0.5) * shift_x;
            }
        }
        
        let shift_y = (1.0 - self.smoothness) * bell_size;
        for (i, &offset) in y_genome.iter().enumerate() {
            if i < y.len() && i > 0 && i < y.len() - 1 {
                y[i] += 0.3 * (offset - 0.5) * shift_y;
            }
        }
        
        // Apply bell accent
        self.apply_bell_accent(&mut x, &mut y, length, bell_size);
        
        // Add bubbles
        for (pos, width, height) in bubbles {
            Self::make_bubble(&mut x, &mut y, pos, width, height);
        }
        
        // Clamp diameters to reasonable range
        for diameter in &mut y {
            *diameter = diameter.max(0.9 * self.d0).min(1.3 * bell_size);
        }
        
        Geo::new(x.into_iter().zip(y.into_iter()).map(|(xi, yi)| [xi, yi]).collect())
    }
    
    fn id(&self) -> u64 {
        self.base.id()
    }
    
    fn loss(&self) -> Option<f64> {
        self.base.loss()
    }
    
    fn set_loss(&mut self, loss: Option<f64>) {
        self.base.set_loss(loss);
    }
    
    fn representation(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id(),
            "genome": self.genome(),
            "loss": self.loss(),
            "parameters": {
                "n_segments": self.n_segments,
                "d0": self.d0,
                "d_bell_min": self.d_bell_min,
                "d_bell_max": self.d_bell_max,
                "max_length": self.max_length,
                "min_length": self.min_length,
                "n_bubbles": self.n_bubbles,
                "smoothness": self.smoothness,
                "bell_accent": self.bell_accent,
                "bell_start": self.bell_start,
            }
        })
    }
}

/// Loss function trait
pub trait LossFunction: Send + Sync {
    fn calculate(&self, genome: &dyn Genome) -> f64;
}

/// Evolutionary optimizer
pub struct EvolutionaryOptimizer {
    pub loss_function: Box<dyn LossFunction>,
    pub population: Vec<Box<dyn Genome>>,
    pub parameters: EvolutionParameters,
    pub prime_generator: PrimeGenerator,
}

/// Evolution parameters
#[derive(Debug, Clone)]
pub struct EvolutionParameters {
    pub population_size: usize,
    pub generation_size: usize,
    pub num_generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elite_size: usize,
    pub mutation_strategy: MutationStrategy,
    pub crossover_strategy: CrossoverStrategy,
}

impl Default for EvolutionParameters {
    fn default() -> Self {
        Self {
            population_size: 50,
            generation_size: 20,
            num_generations: 100,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            elite_size: 5,
            mutation_strategy: MutationStrategy::Gaussian,
            crossover_strategy: CrossoverStrategy::SinglePoint,
        }
    }
}

impl EvolutionaryOptimizer {
    pub fn new(
        loss_function: Box<dyn LossFunction>,
        initial_population: Vec<Box<dyn Genome>>,
        parameters: EvolutionParameters,
    ) -> Self {
        Self {
            loss_function,
            population: initial_population,
            parameters,
            prime_generator: PrimeGenerator::default(),
        }
    }
    
    /// Create optimizer with random initial population
    pub fn with_random_population<G: Genome + 'static>(
        loss_function: Box<dyn LossFunction>,
        genome_template: &G,
        population_size: usize,
        parameters: EvolutionParameters,
    ) -> Self {
        let population: Vec<Box<dyn Genome>> = (0..population_size)
            .map(|_| genome_template.clone_with_new_id())
            .collect();
        
        Self::new(loss_function, population, parameters)
    }
    
    /// Evolve the population for specified number of generations
    pub fn evolve(&mut self) -> Result<Box<dyn Genome>, Box<dyn std::error::Error>> {
        self.evolve_with_progress(|_, _| {})
    }
    
    /// Evolve with progress callback (generation, best_loss)
    pub fn evolve_with_progress<F>(&mut self, mut progress_cb: F) -> Result<Box<dyn Genome>, Box<dyn std::error::Error>>
    where
        F: FnMut(usize, f64),
    {
        // Evaluate initial population
        self.evaluate_population()?;
        
        // Evolution loop
        for generation in 0..self.parameters.num_generations {
            log::info!("Generation {}/{}", generation + 1, self.parameters.num_generations);
            
            // Create offspring
            let mut offspring = self.create_offspring()?;
            
            // Evaluate offspring
            Self::evaluate_genomes(&*self.loss_function, &mut offspring)?;
            
            // Select new population
            self.select_population(offspring)?;
            
            // Report progress
            if let Some(best) = self.get_best_individual() {
                let best_loss = best.loss().unwrap_or(f64::INFINITY);
                progress_cb(generation, best_loss);
            }
        }
        
        // Return best individual
        self.get_best_individual()
            .ok_or_else(|| "No individuals found".into())
    }
    
    /// Evaluate population fitness
    fn evaluate_population(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Self::evaluate_genomes(&*self.loss_function, &mut self.population)?;
        Ok(())
    }
    
    /// Evaluate genomes in parallel, skipping those with cached loss.
    fn evaluate_genomes(loss_function: &dyn LossFunction, genomes: &mut [Box<dyn Genome>]) -> Result<(), Box<dyn std::error::Error>> {
        genomes.par_iter_mut().for_each(|genome| {
            if genome.loss().is_none() {
                let loss = loss_function.calculate(genome.as_ref());
                genome.set_loss(Some(loss));
            }
        });
        
        Ok(())
    }
    
    /// Create offspring through mutation and crossover
    fn create_offspring(&self) -> Result<Vec<Box<dyn Genome>>, Box<dyn std::error::Error>> {
        let mut offspring = Vec::new();
        
        // Add elite individuals
        let mut sorted_population: Vec<Box<dyn Genome>> = self.population.iter()
            .map(|g| g.clone_with_new_id())
            .collect();
        sorted_population.sort_by(|a: &Box<dyn Genome>, b: &Box<dyn Genome>| {
            a.loss().unwrap_or(f64::INFINITY)
                .partial_cmp(&b.loss().unwrap_or(f64::INFINITY))
                .unwrap()
        });
        
        for i in 0..self.parameters.elite_size.min(sorted_population.len()) {
            offspring.push(sorted_population[i].clone_with_loss());
        }
        
        // Create remaining offspring
        while offspring.len() < self.parameters.generation_size {
            if rand::random::<f64>() < self.parameters.crossover_rate {
                // Crossover
                let parent1 = self.tournament_selection()?;
                let parent2 = self.tournament_selection()?;
                let child = self.crossover(&*parent1, &*parent2)?;
                offspring.push(child);
            } else {
                // Mutation
                let parent = self.tournament_selection()?;
                let mutant = self.mutate(&*parent)?;
                offspring.push(mutant);
            }
        }
        
        Ok(offspring)
    }
    
    /// Tournament selection
    fn tournament_selection(&self) -> Result<Box<dyn Genome>, Box<dyn std::error::Error>> {
        let tournament_size = 3;
        let mut best: Option<Box<dyn Genome>> = None;
        
        for _ in 0..tournament_size {
            let idx = rand::random::<usize>() % self.population.len();
            let candidate = &self.population[idx];
            
            if let Some(current_best) = &best {
                if candidate.loss().unwrap_or(f64::INFINITY) < current_best.loss().unwrap_or(f64::INFINITY) {
                    best = Some(candidate.clone_with_new_id());
                }
            } else {
                best = Some(candidate.clone_with_new_id());
            }
        }
        
        best.ok_or_else(|| "Tournament selection failed".into())
    }
    
    /// Crossover operation supporting multiple strategies.
    fn crossover(&self, parent1: &dyn Genome, parent2: &dyn Genome) -> Result<Box<dyn Genome>, Box<dyn std::error::Error>> {
        let genome1 = parent1.genome();
        let genome2 = parent2.genome();

        if genome1.len() != genome2.len() {
            return Err("Genomes must have same length for crossover".into());
        }

        let mut child_genome = Vec::with_capacity(genome1.len());

        match self.parameters.crossover_strategy {
            CrossoverStrategy::SinglePoint => {
                let crossover_point = rand::random::<usize>() % genome1.len();
                for i in 0..genome1.len() {
                    let gene = if i < crossover_point {
                        genome1[i]
                    } else {
                        genome2[i]
                    };
                    child_genome.push(gene.clamp(0.0, 1.0));
                }
            }
            CrossoverStrategy::Average => {
                for i in 0..genome1.len() {
                    child_genome.push(((genome1[i] + genome2[i]) / 2.0).clamp(0.0, 1.0));
                }
            }
            CrossoverStrategy::PartSwap => {
                child_genome = genome1.to_vec();
                let start = rand::random::<usize>() % genome1.len();
                let end = (start + rand::random::<usize>() % (genome1.len() - start)).min(genome1.len());
                for i in start..end {
                    child_genome[i] = genome2[i].clamp(0.0, 1.0);
                }
            }
            CrossoverStrategy::PartAverage => {
                child_genome = genome1.to_vec();
                let start = rand::random::<usize>() % genome1.len();
                let end = (start + rand::random::<usize>() % (genome1.len() - start)).min(genome1.len());
                for i in start..end {
                    child_genome[i] = ((genome1[i] + genome2[i]) / 2.0).clamp(0.0, 1.0);
                }
            }
        }

        let mut child = parent1.clone_with_new_id();
        child.set_genome(child_genome);
        Ok(child)
    }

    /// Mutation operation supporting multiple strategies.
    fn mutate(&self, genome: &dyn Genome) -> Result<Box<dyn Genome>, Box<dyn std::error::Error>> {
        let mut mutated = genome.clone_with_new_id();
        let mutated_genome = mutated.genome_mut();

        match self.parameters.mutation_strategy {
            MutationStrategy::Gaussian => {
                for gene in mutated_genome {
                    if rand::random::<f64>() < self.parameters.mutation_rate {
                        let noise = rand_distr::Normal::new(0.0, 0.1)
                            .map_err(|e| format!("Mutation failed: {}", e))?
                            .sample(&mut rand::thread_rng());
                        *gene = (*gene + noise).clamp(0.0, 1.0);
                    }
                }
            }
            MutationStrategy::PrimeSequence => {
                let generator = PrimeGenerator::new(1000);
                let mut index = 0;
                for gene in mutated_genome {
                    if rand::random::<f64>() < self.parameters.mutation_rate {
                        let prime = generator.nth(index);
                        let noise = (prime as f64 / 100.0) * rand::random::<f64>();
                        *gene = (*gene + noise).clamp(0.0, 1.0);
                        index += 1;
                    }
                }
            }
            MutationStrategy::SingleMutation => {
                if !mutated_genome.is_empty() {
                    let idx = rand::random::<usize>() % mutated_genome.len();
                    let noise = rand_distr::Normal::new(0.0, 0.1)
                        .map_err(|e| format!("Mutation failed: {}", e))?
                        .sample(&mut rand::thread_rng());
                    mutated_genome[idx] = (mutated_genome[idx] + noise).clamp(0.0, 1.0);
                }
            }
        }

        Ok(mutated)
    }
    
    /// Select new population from combined parent and offspring
    fn select_population(&mut self, offspring: Vec<Box<dyn Genome>>) -> Result<(), Box<dyn std::error::Error>> {
        let mut combined: Vec<Box<dyn Genome>> = self.population.iter()
            .map(|g| g.clone_with_new_id())
            .collect();
        combined.extend(offspring);
        
        // Sort by fitness
        combined.sort_by(|a: &Box<dyn Genome>, b: &Box<dyn Genome>| {
            a.loss().unwrap_or(f64::INFINITY)
                .partial_cmp(&b.loss().unwrap_or(f64::INFINITY))
                .unwrap()
        });
        
        // Keep best individuals
        self.population = combined.into_iter().take(self.parameters.population_size).collect();
        Ok(())
    }
    
    /// Get best individual from current population
    fn get_best_individual(&self) -> Option<Box<dyn Genome>> {
        self.population.iter()
            .min_by(|a, b| {
                a.loss().unwrap_or(f64::INFINITY)
                    .partial_cmp(&b.loss().unwrap_or(f64::INFINITY))
                    .unwrap()
            })
            .map(|genome| genome.clone_with_new_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loss::TestLossFunction;
    
    #[test]
    fn test_base_genome() {
        let genome = BaseGenome::random(10);
        assert_eq!(genome.genome().len(), 10);
        assert!(genome.id() > 0);
    }
    
    #[test]
    fn test_kigali_genome_creation() {
        let genome = KigaliGenome::new(
            24,     // n_segments
            32.0,   // d0
            50.0,   // d_bell_min
            80.0,   // d_bell_max
            1900.0, // max_length
            1500.0, // min_length
            2,      // n_bubbles
            0.3,    // smoothness
            0.2,    // bell_accent
            300.0,  // bell_start
        );
        
        let geo = genome.genome2geo();
        assert!(!geo.geo.is_empty());
        assert!(geo.length() >= 1500.0);
        assert!(geo.length() <= 1900.0);
    }
    
    #[test]
    fn test_evolution_optimizer() {
        let loss_function = Box::new(TestLossFunction::new());
        let genome_template = KigaliGenome::new(10, 32.0, 50.0, 80.0, 1800.0, 1500.0, 0, 0.3, 0.0, 300.0);
        
        let mut optimizer = EvolutionaryOptimizer::with_random_population(
            loss_function,
            &genome_template,
            10, // population size
            EvolutionParameters {
                population_size: 10,
                generation_size: 5,
                num_generations: 3,
                mutation_rate: 0.1,
                crossover_rate: 0.7,
                elite_size: 2,
                mutation_strategy: MutationStrategy::Gaussian,
                crossover_strategy: CrossoverStrategy::SinglePoint,
            },
        );
        
        // This is a basic test - in practice, you'd want to test with actual loss functions
        let result = optimizer.evolve();
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_mutation_strategies() {
        let genome = BaseGenome::random(10);
        let original = genome.genome().to_vec();

        let mut params = EvolutionParameters::default();
        params.mutation_strategy = MutationStrategy::SingleMutation;
        let optimizer = EvolutionaryOptimizer::new(
            Box::new(TestLossFunction::new()),
            vec![genome.clone_with_new_id()],
            params,
        );

        let mutant = optimizer.mutate(&genome).unwrap();
        let mutated = mutant.genome();
        let diff_count = original.iter().zip(mutated.iter()).filter(|(a, b)| (**a - **b).abs() > 1e-12).count();
        assert_eq!(diff_count, 1);
    }

    #[test]
    fn test_crossover_strategies() {
        let parent1 = BaseGenome::random(8);
        let parent2 = BaseGenome::random(8);

        for strategy in [
            CrossoverStrategy::SinglePoint,
            CrossoverStrategy::Average,
            CrossoverStrategy::PartSwap,
            CrossoverStrategy::PartAverage,
        ] {
            let params = EvolutionParameters {
                crossover_strategy: strategy,
                ..Default::default()
            };
            let optimizer = EvolutionaryOptimizer::new(
                Box::new(TestLossFunction::new()),
                vec![parent1.clone_with_new_id()],
                params,
            );
            let child = optimizer.crossover(&parent1, &parent2).unwrap();
            assert_eq!(child.genome().len(), 8);
        }
    }
}