# Rust CADSD - Computer-Aided Didgeridoo Sound Design

A comprehensive Rust implementation of CADSD (Computer-Aided Didgeridoo Sound Design) for didgeridoo and wind instrument modeling, based on the existing DidgeLab Python project.

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
use cadsd::sim::DidgeridooSimulator;

// Create a conical didgeridoo
let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);

// Simulate acoustics
let simulator = DidgeridooSimulator::new(geo.geo);
let spectrum = simulator.compute_impedance_spectrum();
let peaks = simulator.find_resonance_peaks();

println!("Found {} resonance peaks", peaks.len());
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

// Run optimization
let mut optimizer = EvolutionaryOptimizer::with_random_population(
    Box::new(loss), &genome, 30, Default::default()
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
2. **Transmission line modeling** - CADSD algorithm for acoustic impedance calculation
3. **Modular loss system** - Composable loss functions for multi-objective optimization
4. **Evolutionary framework** - Genetic algorithms with parallel evaluation
5. **Parametric shapes** - Kigali-style genome-to-geometry mapping

## Scientific Foundation

The implementation is based on established acoustical principles:

- **Webster Horn Equation** - 1D wave equation for varying cross-section tubes
- **Transmission Line Theory** - Mapes-Riordan approach for acoustic modeling
- **Viscothermal Losses** - Boundary layer effects in wave propagation
- **Radiation Models** - Levine-Schwinger impedance at open ends

## Performance Considerations

- **SIMD optimization** - Vectorized calculations for acoustic simulations
- **Parallel evaluation** - Rayon-based parallel processing for evolution
- **Memory efficiency** - Optimized data structures for large populations
- **Caching** - Results caching for unchanged geometries

## Testing

```bash
# Run unit tests
cargo test

# Run specific test modules
cargo test sim::tests
cargo test geo::tests
```

## Future Enhancements

- GPU acceleration for intensive computations
- Support for other wind instruments
- Advanced optimization algorithms (NSGA-II, CMA-ES)
- Interactive GUI for design exploration
- Real-time audio synthesis integration

## References

- Frank Geipel's CADSD methodology
- Dan Mapes-Riordan's transmission line modeling
- Didgmo/DidjiImp reference implementations
- DidgeLab Python project (reference implementation)

## License

This project is licensed under Creative Commons BY-NC-SA 4.0, consistent with the original DidgeLab project.