//! Evolutionary optimization module
//!
//! This module provides evolutionary algorithms for didgeridoo design optimization,
//! matching the Python DidgeLab evo module functionality.

use crate::geo::Geo;
use crate::conv::note_to_freq;
use std::fmt;

/// Represents target sound description (like DidgeLab web interface)
#[derive(Clone, Debug)]
pub struct TargetSound {
    /// Target fundamental frequency (Hz)
    pub fundamental_freq: f64,
    /// Target toot frequencies (Hz) - optional resonance peaks
    pub toots: Vec<f64>,
    /// Target overtone series (harmonic numbers)
    pub overtones: Vec<usize>,
    /// Preferred bore shape
    pub bore_shape: BoreShapePreference,
    /// Length constraints (min, max in mm)
    pub length_range: (f64, f64),
    /// Bell diameter constraints (min, max in mm)
    pub bell_range: (f64, f64),
}

impl TargetSound {
    pub fn new(fundamental_freq: f64) -> Self {
        Self {
            fundamental_freq,
            toots: vec![],
            overtones: vec![2, 3, 4],
            bore_shape: BoreShapePreference::Any,
            length_range: (500.0, 2500.0),
            bell_range: (30.0, 120.0),
        }
    }
    
    /// Create from musical note (e.g., "D1", "A4")
    pub fn from_note(note_name: &str) -> Result<Self, String> {
        let note_value = parse_note_name(note_name)?;
        let freq = note_to_freq(note_value);
        Ok(Self::new(freq))
    }
    
    /// Add a toot at specific frequency
    pub fn with_toot(mut self, freq: f64) -> Self {
        self.toots.push(freq);
        self
    }
    
    /// Add a toot at specific musical note
    pub fn with_toot_note(mut self, note_name: &str) -> Result<Self, String> {
        let note_value = parse_note_name(note_name)?;
        let freq = note_to_freq(note_value);
        self.toots.push(freq);
        Ok(self)
    }
    
    /// Set target overtones (harmonic numbers)
    pub fn with_overtones(mut self, overtones: Vec<usize>) -> Self {
        self.overtones = overtones;
        self
    }
    
    /// Set bore shape preference
    pub fn with_bore_shape(mut self, shape: BoreShapePreference) -> Self {
        self.bore_shape = shape;
        self
    }
    
    /// Set length constraints
    pub fn with_length_range(mut self, min: f64, max: f64) -> Self {
        self.length_range = (min, max);
        self
    }
    
    /// Set bell diameter constraints
    pub fn with_bell_range(mut self, min: f64, max: f64) -> Self {
        self.bell_range = (min, max);
        self
    }
}

/// Bore shape preference for optimization
#[derive(Clone, Debug)]
pub enum BoreShapePreference {
    /// Any shape is acceptable
    Any,
    /// Prefer cylindrical drone
    Cylindrical,
    /// Prefer conical bore
    Conical,
    /// Prefer flared horn
    Flared,
}

impl fmt::Display for BoreShapePreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoreShapePreference::Any => write!(f, "Any"),
            BoreShapePreference::Cylindrical => write!(f, "Cylindrical"),
            BoreShapePreference::Conical => write!(f, "Conical"),
            BoreShapePreference::Flared => write!(f, "Flared"),
        }
    }
}

/// Parse musical note name to MIDI note number
fn parse_note_name(note: &str) -> Result<i32, String> {
    let note = note.trim().to_uppercase();
    let chars: Vec<char> = note.chars().collect();
    
    if chars.len() < 2 {
        return Err(format!("Invalid note name: {}", note));
    }
    
    // Parse note name
    let base_note = match chars[0] {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return Err(format!("Invalid note: {}", chars[0])),
    };
    
    // Parse accidental
    let mut accidental = 0;
    let mut octave_start = 1;
    if chars.len() > 2 {
        match chars[1] {
            '#' => accidental = 1,
            'B' => accidental = -1,
            _ => octave_start = 1,
        }
        if chars[1] == '#' || chars[1] == 'B' {
            octave_start = 2;
        }
    }
    
    // Parse octave
    let octave: i32 = note[octave_start..].parse()
        .map_err(|_| format!("Invalid octave in: {}", note))?;
    
    // Convert to MIDI note number
    let midi_note = 12 * (octave + 1) + base_note + accidental;
    Ok(midi_note - 12) // Adjust to match Python DidgeLab convention
}

/// Represents a genome for evolutionary optimization (same as Python)
#[derive(Clone)]
pub struct GeoGenome {
    pub genes: Vec<f64>,
    pub fitness: Option<f64>,
    pub geo: Option<Geo>,
}

impl GeoGenome {
    pub fn new(genes: Vec<f64>) -> Self {
        Self {
            genes,
            fitness: None,
            geo: None,
        }
    }
    
    /// Convert genome to geometry (same interface as Python)
    pub fn to_geo(&self) -> Geo {
        // For now, use a simple interpretation of genes as cone parameters
        // This matches the Python interface where genes define geometry
        if self.genes.len() >= 4 {
            let length = self.genes[0].max(500.0).min(2000.0);  // 500-2000mm
            let top_diam = self.genes[1].max(10.0).min(50.0);   // 10-50mm
            let bot_diam = self.genes[2].max(30.0).min(100.0);  // 30-100mm
            let segments = (self.genes[3] * 10.0).max(5.0).min(50.0) as usize; // 5-50 segments
            
            Geo::make_cone(length, top_diam, bot_diam, segments)
        } else {
            // Default cone if not enough genes
            Geo::make_cone(1500.0, 32.0, 65.0, 20)
        }
    }
    
    /// Evaluate fitness using a loss function (same as Python)
    pub fn evaluate_fitness(&mut self, loss_fn: &LossFunctionType) -> Result<f64, String> {
        let geo = self.to_geo();
        let loss_value = loss_fn.compute_loss(&geo)?;
        self.fitness = Some(loss_value);
        self.geo = Some(geo);
        Ok(loss_value)
    }
}

/// Loss function trait (same as Python)
pub trait LossFunction {
    fn compute_loss(&self, geo: &Geo) -> Result<f64, String>;
}

/// Type alias for loss functions (matches Python interface)
/// Type alias for loss functions (matches Python interface)
pub enum LossFunctionType {
    TairuaLoss(crate::loss::TairuaLoss),
    DidgeLabLoss(crate::loss::DidgeLabLoss),
    Custom(Box<dyn Fn(&Geo) -> Result<f64, String>>),
}

impl LossFunctionType {
    pub fn compute_loss(&self, geo: &Geo) -> Result<f64, String> {
        match self {
            LossFunctionType::TairuaLoss(loss) => loss.compute_loss(geo),
            LossFunctionType::DidgeLabLoss(loss) => loss.compute_loss(geo),
            LossFunctionType::Custom(func) => func(geo),
        }
    }
}

/// Mutation operator enum (matches Python interface)
pub enum MutationOperator {
    Gaussian { rate: f64, scale: f64 },
    Uniform { rate: f64, range: (f64, f64) },
    RandomResetting { rate: f64 },
}

impl MutationOperator {
    pub fn mutate(&self, genome: &mut GeoGenome) {
        match self {
            MutationOperator::Gaussian { rate, scale } => {
                for gene in genome.genes.iter_mut() {
                    if rand::random::<f64>() < *rate {
                        *gene += rand::random::<f64>() * *scale * 2.0 - *scale;
                    }
                }
            },
            MutationOperator::Uniform { rate, range } => {
                for gene in genome.genes.iter_mut() {
                    if rand::random::<f64>() < *rate {
                        *gene = rand::random::<f64>() * (range.1 - range.0) + range.0;
                    }
                }
            },
            MutationOperator::RandomResetting { rate } => {
                for gene in genome.genes.iter_mut() {
                    if rand::random::<f64>() < *rate {
                        *gene = rand::random::<f64>();
                    }
                }
            },
        }
    }
}

/// Crossover operator enum (matches Python interface)
pub enum CrossoverOperator {
    Uniform { rate: f64 },
    SinglePoint,
    TwoPoint,
}

impl CrossoverOperator {
    pub fn crossover(&self, parent1: &GeoGenome, parent2: &GeoGenome) -> (GeoGenome, GeoGenome) {
        let mut child1_genes = parent1.genes.clone();
        let mut child2_genes = parent2.genes.clone();
        
        match self {
            CrossoverOperator::Uniform { rate } => {
                for i in 0..child1_genes.len().min(child2_genes.len()) {
                    if rand::random::<f64>() < *rate {
                        std::mem::swap(&mut child1_genes[i], &mut child2_genes[i]);
                    }
                }
            },
            CrossoverOperator::SinglePoint => {
                let point = rand::random::<usize>() % (child1_genes.len().max(1));
                for i in point..child1_genes.len() {
                    if i < child2_genes.len() {
                        std::mem::swap(&mut child1_genes[i], &mut child2_genes[i]);
                    }
                }
            },
            CrossoverOperator::TwoPoint => {
                let len = child1_genes.len().max(2);
                let point1 = rand::random::<usize>() % (len - 1);
                let point2 = point1 + 1 + rand::random::<usize>() % (len - point1 - 1);
                
                for i in point1..point2 {
                    if i < child2_genes.len() {
                        std::mem::swap(&mut child1_genes[i], &mut child2_genes[i]);
                    }
                }
            },
        }
        
        (GeoGenome::new(child1_genes), GeoGenome::new(child2_genes))
    }
}

/// Main evolutionary algorithm (matches Python Nuevolution)
pub struct Nuevolution {
    population_size: usize,
    generations: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    elite_size: usize,
    verbose: bool,
}

impl Nuevolution {
    pub fn new(population_size: usize, generations: usize) -> Self {
        Self {
            population_size,
            generations,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            elite_size: 1,
            verbose: true,
        }
    }
    
    pub fn set_mutation_rate(mut self, rate: f64) -> Self {
        self.mutation_rate = rate;
        self
    }
    
    pub fn set_crossover_rate(mut self, rate: f64) -> Self {
        self.crossover_rate = rate;
        self
    }
    
    pub fn set_elite_size(mut self, size: usize) -> Self {
        self.elite_size = size;
        self
    }
    
    pub fn set_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// Evolve a population toward optimal geometry (same as Python)
    pub fn evolve(
        &self,
        initial_population: Vec<GeoGenome>,
        loss_fn: &LossFunctionType,
        progress_cb: Option<&(dyn Fn(usize, f64) + Send + Sync)>,
    ) -> Result<Vec<GeoGenome>, String> {
        let mut population = initial_population;
        
        for gen in 0..self.generations {
            // Evaluate fitness for all individuals
            for individual in population.iter_mut() {
                individual.evaluate_fitness(loss_fn)?;
            }
            
            // Sort by fitness (ascending - lower loss is better)
            population.sort_by(|a, b| {
                a.fitness.unwrap_or(f64::INFINITY).partial_cmp(&b.fitness.unwrap_or(f64::INFINITY)).unwrap()
            });
            
            // Call progress callback
            if let Some(cb) = progress_cb {
                let best_fitness = population.first().and_then(|ind| ind.fitness).unwrap_or(f64::INFINITY);
                cb(gen, best_fitness);
            }
            
            if self.verbose && gen % 10 == 0 {
                let best_fitness = population.first().unwrap().fitness.unwrap_or(f64::NAN);
                println!("Generation {}: Best fitness = {:.6}", gen, best_fitness);
            }
            
            // Create next generation
            let mut next_gen = Vec::new();
            
            // Preserve elites
            for i in 0..self.elite_size.min(population.len()) {
                next_gen.push(population[i].clone());
            }
            
            // Generate offspring
            while next_gen.len() < self.population_size {
                // Selection: tournament selection
                let parent1 = self.tournament_select(&population);
                let parent2 = self.tournament_select(&population);
                
                let (mut child1, mut child2) = if rand::random::<f64>() < self.crossover_rate {
                    CrossoverOperator::Uniform { rate: 0.5 }.crossover(&population[parent1], &population[parent2])
                } else {
                    (population[parent1].clone(), population[parent2].clone())
                };
                
                // Apply mutation
                MutationOperator::Gaussian { rate: self.mutation_rate, scale: 0.1 }.mutate(&mut child1);
                MutationOperator::Gaussian { rate: self.mutation_rate, scale: 0.1 }.mutate(&mut child2);
                
                next_gen.push(child1);
                if next_gen.len() < self.population_size {
                    next_gen.push(child2);
                }
            }
            
            population = next_gen;
        }
        
        // Final evaluation
        for individual in population.iter_mut() {
            individual.evaluate_fitness(loss_fn)?;
        }
        
        // Sort final population
        population.sort_by(|a, b| {
            a.fitness.unwrap_or(f64::INFINITY).partial_cmp(&b.fitness.unwrap_or(f64::INFINITY)).unwrap()
        });
        
        Ok(population)
    }
    
    fn tournament_select(&self, population: &[GeoGenome]) -> usize {
        let tournament_size = 3.min(population.len());
        let mut best_idx = rand::random::<usize>() % population.len();
        let mut best_fitness = population[best_idx].fitness.unwrap_or(f64::INFINITY);
        
        for _ in 1..tournament_size {
            let idx = rand::random::<usize>() % population.len();
            let fitness = population[idx].fitness.unwrap_or(f64::INFINITY);
            if fitness < best_fitness {
                best_idx = idx;
                best_fitness = fitness;
            }
        }
        
        best_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_geogenome_creation() {
        let genes = vec![1000.0, 20.0, 50.0, 15.0];
        let genome = GeoGenome::new(genes);
        
        assert_eq!(genome.genes.len(), 4);
        assert!(genome.fitness.is_none());
        assert!(genome.geo.is_none());
    }
    
    #[test]
    fn test_geogenome_to_geo() {
        let genes = vec![1200.0, 25.0, 60.0, 25.0];
        let genome = GeoGenome::new(genes);
        let geo = genome.to_geo();
        
        assert!(geo.length() > 0.0);
        // Gene 25.0 * 10 = 250, clamped to max 50 segments; make_cone produces segments+1 points
        assert_eq!(geo.geo.len(), 51);
    }
    
    #[test]
    fn test_evolution_basic() {
        // Create a simple loss function that favors longer didges
        let loss_fn = LossFunctionType::Custom(Box::new(|geo: &Geo| {
            Ok(-geo.length()) // Negative length so longer = lower loss
        }));
        
        // Create initial population
        let mut population = Vec::new();
        for _ in 0..10 {
            let genes = vec![
                500.0 + rand::random::<f64>() * 1000.0,  // length: 500-1500mm
                20.0 + rand::random::<f64>() * 30.0,      // top diam: 20-50mm
                40.0 + rand::random::<f64>() * 40.0,      // bottom diam: 40-80mm
                5.0 + rand::random::<f64>() * 20.0,       // segments: 5-25
            ];
            population.push(GeoGenome::new(genes));
        }
        
        let evolver = Nuevolution::new(10, 5).set_verbose(false);
        let result = evolver.evolve(population, &loss_fn, None);
        
        assert!(result.is_ok());
        let final_pop = result.unwrap();
        assert_eq!(final_pop.len(), 10);
    }
}