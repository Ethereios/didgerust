# CADSD - Computer-Aided Didgeridoo Sound Design

CADSD is a comprehensive Rust implementation of the Computer-Aided Didgeridoo Sound Design methodology, based on Frank Geipel's research and the Python DidgeLab toolkit. This project provides tools for didgeridoo and wind instrument modeling, design, and analysis.

## Features

- **Acoustic Simulation**: Transmission line modeling for predicting acoustic properties
- **Geometry Generation**: Parametric shape generation (conical, bubble, stretched, scaled)
- **Evolutionary Optimization**: Genetic algorithms for inverse design
- **Analysis Tools**: Frequency spectrum analysis, resonance detection, note conversion
- **Loss Functions**: Tairua composite loss function for multi-objective optimization
- **Visual Interface**: Real-time 3D visualization with interactive controls (GUI mode)

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd cadssd

# Build with GUI support
cargo build --release --features gui

# Run the console demo
cargo run

# Run the GUI application
cargo run --features gui -- --gui
```

## Usage

### Console Mode (Default)
```bash
cargo run
```
This runs the demonstration suite showing:
- Forward design: Geometry → Acoustic properties
- Parametric shape generation
- Analysis tools and utilities

### GUI Mode
```bash
cargo run --features gui -- --gui
```
This launches the visual CADSD interface with:
- Interactive geometry controls (length, diameters, segments)
- Real-time acoustic simulation
- Frequency spectrum visualization
- Parameter optimization tools

## Architecture

### Core Modules

- **geo**: Geometry representation and manipulation
- **sim**: Acoustic simulation using transmission line modeling
- **conv**: Note/frequency conversion utilities
- **analysis**: Spectrum analysis and visualization tools
- **evo**: Evolutionary algorithms for optimization
- **loss**: Loss functions for design objectives
- **app**: Visual application using Bevy engine

### Key Algorithms

- **Transmission Line Model**: Physics-based acoustic simulation
- **Webster Horn Equation**: For varying cross-section tubes
- **Viscothermal Loss**: Boundary layer effects modeling
- **Radiation Impedance**: End correction calculations
- **Genetic Algorithm**: Multi-objective optimization
- **Tairua Loss**: Composite acoustic quality metric

## Dependencies

- **nalgebra**: Linear algebra operations
- **ndarray**: N-dimensional arrays
- **num-complex**: Complex number operations
- **bevy**: Game engine for 3D visualization (optional)
- **bevy_egui**: Immediate mode GUI
- **rand**: Random number generation
- **plotters**: 2D plotting (optional)

## Python DidgeLab Compatibility

This Rust implementation maintains API compatibility with the Python DidgeLab toolkit:
- Same function signatures and parameter names
- Equivalent mathematical models and algorithms
- Compatible data structures and units
- Similar analysis and visualization capabilities

## License

This project is licensed under Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0).

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Research References

This project is based on the research by Frank Geipel and colleagues on didgeridoo acoustics and computer-aided design methodologies. For more information on the underlying physics and mathematics, see the academic publications referenced in the original DidgeLab project.