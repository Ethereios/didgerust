# Rust CADSD - Computer-Aided Didgeridoo Sound Design

A comprehensive Rust implementation of CADSD (Computer-Aided Didgeridoo Sound Design) for didgeridoo and wind instrument modeling, based on the existing DidgeLab Python project.

## Testing Status

✅ Initial Testing Passed - Core functionality validated:
- 30+ unit tests passing across geometry, simulation, evolutionary optimization, and persistence modules
- Numerical stability verified for acoustic impedance calculations
- Validation logic confirmed for geometry points and constraints
- Basic simulation and optimization workflows operational

*Note: This represents initial validation. Comprehensive testing will be conducted before major feature development.*

## Overview

This project provides a Rust implementation of the CADSD methodology for:
- **Acoustical simulation** - Compute resonant frequencies and impedance spectra for didgeridoo geometries
- **Evolutionary optimization** - Search for bore shapes that meet target sonic properties
- **Geometry representation** - Parametric shape generation and manipulation
- **Loss functions** - Modular framework for acoustic evaluation

## Features

### Core Modules

1. **Acoustical Simulation** (`src/sim/`)
   - Transmission Line Model (TLM) implementation
   - Webster Horn Equation solver
   - Viscothermal loss modeling
   - Radiation impedance calculations
   - Frequency domain analysis

2. **Geometry Representation** (`src/geo/`)
   - Segment-based bore geometry
   - Parametric shape generators (Kigali, Mbeya)
   - Geometry operations (scaling, stretching, bubbles)
   - RBF constraint systems

3. **Evolutionary Optimization** (`src/evo/`)
   - Genome representation framework
   - Mutation and crossover operators
   - Selection mechanisms
   - Parallel evaluation with Rayon

4. **Loss Functions** (`src/loss/`)
   - Modular loss component system
   - Frequency tuning, Q-factor, modal density
   - Harmonic relationships and scale tuning
   - Composite loss orchestration

5. **Visualization** (`src/visualization/`)
   - Impedance spectrum plotting
   - Geometry visualization
   - Evolution progress tracking

## Installation

### Prerequisites

- Rust 1.56+ (for full version) or any recent version (for simple demo)
- Basic understanding of acoustics and didgeridoo design

### Building

```bash
# Clone the repository
git clone <repository-url>
cd rust-cadsd

# Build the full library (requires newer Rust)
cargo build --release

# Or run the simple demo (works with older Rust)
rustc src/simple_demo.rs -o simple_demo
./simple_demo
```

## Usage Examples

### Basic Simulation

```rust
use cadsd::geo::Geo;
use cadsd::sim::{DidgeridooSimulator, SimulationStrategy};

// Create a conical didgeridoo
let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);

// TLM (default - stable)
let simulator_tlm = DidgeridooSimulator::from_geo(&geo.geo);
let spectrum = simulator_tlm.impedance(&freqs);

// Digital Waveguide (new - real-time friendly)
let simulator_wg = DidgeridooSimulator::with_strategy(&geo.geo, SimulationStrategy::Waveguide);
let spectrum = simulator_wg.impedance(&freqs);

// Complex Impedance (new - phase-aware)
let simulator_ci = DidgeridooSimulator::with_strategy(&geo.geo, SimulationStrategy::ComplexImpedance);
let spectrum = simulator_ci.impedance(&freqs);

let peaks = simulator.find_resonance_peaks();
```

### Evolutionary Optimization

```rust
use cadsd::evo::{EvolutionaryOptimizer, KigaliGenome};
use cadsd::loss::CompositeTairuaLoss;

// Create genome template
let genome = KigaliGenome::new(
    20, 32.0, 50.0, 80.0, 1800.0, 1500.0, 2, 0.3, 0.2, 300.0
);

// Set up loss function
let mut loss = CompositeTairuaLoss::new(5.0);
// Add loss components...

// Run optimization with Prime-Indexed mutation for enhanced exploration
let mut optimizer = EvolutionaryOptimizer::with_random_population(
    Box::new(loss), &genome, 30,
    EvolutionParameters {
        population_size: 50,
        generation_size: 20,
        num_generations: 100,
        mutation_rate: 0.1,
        crossover_rate: 0.7,
        elite_size: 5,
        mutation_strategy: MutationStrategy::PrimeSequence, // Try MutationStrategy::Gaussian for standard behavior
    },
);
let best_genome = optimizer.evolve()?;
```

### Simple Demo

For systems with older Rust versions, run the simplified demo:

```bash
rustc src/simple_demo.rs -o simple_demo
./simple_demo
```

This demonstrates basic acoustic simulation and simple optimization.

## Architecture

The implementation follows the same architectural principles as the Python DidgeLab project:

1. **Segment-based geometry** - Bore represented as chain of conical/cylindrical segments
2. **Multi-strategy simulation** - Supports TLM, Digital Waveguide, and Complex Impedance methods
3. **Modular loss system** - Composable loss functions for multi-objective optimization
4. **Evolutionary framework** - Genetic algorithms with Gaussian and Prime-Indexed mutation strategies
5. **Parametric shapes** - Kigali-style genome-to-geometry mapping

### Simulation Strategies

- **TLM (Transmission Line Model)** - Default stable method based on transfer matrix cascade
- **Digital Waveguide** - Bidirectional delay-line model for real-time applications
- **Complex Impedance** - Enhanced complex-number calculations for phase-aware analysis

### Mutation Strategies

- **Gaussian** - Standard random perturbations with normal distribution
- **PrimeSequence** - Prime-number indexed mutations for improved space exploration

## Scientific Foundation

The implementation is based on established acoustical principles:

- **Webster Horn Equation** - 1D wave equation for varying cross-section tubes
- **Transmission Line Theory** - Mapes-Riordan approach for acoustic modeling
- **Digital Waveguides** - Bidirectional delay-line modeling for efficient simulation
- **Viscothermal Losses** - Boundary layer effects in wave propagation
- **Radiation Models** - Levine-Schwinger impedance at open ends

## Performance Considerations

- **Parallel evaluation** - Rayon-based parallel processing for evolution
- **Memory efficiency** - Optimized data structures for large populations
- **Digital Waveguides** - Lightweight time-domain modeling suitable for real-time applications
- **Prime-Indexed Mutation** - Enhanced exploration with minimal overhead (uses cached prime sieve)

## Testing

```bash
# Run unit tests
cargo test

# Run specific test modules
cargo test sim::tests
```