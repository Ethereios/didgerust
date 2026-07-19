# CADSD DidgeLab - Computer-Aided Didgeridoo Sound Design

A modern desktop application for designing didgeridoos using acoustic simulation and evolutionary optimization, inspired by [DidgeLab.com](https://didgelab.com).

## 🎵 Features

### Inverse Design (Like DidgeLab.com)
- **Describe the sound you want**: Specify key, toots, and overtones
- **Bore shape preferences**: Choose cylindrical, conical, or flared designs
- **Evolutionary optimization**: AI explores thousands of geometries to find matches
- **Real-time visualization**: See impedance spectra and bore profiles
- **Export designs**: Save geometries in TXT or JSON format

### Acoustic Simulation
- Transmission Line Modeling (TLM) for accurate acoustic prediction
- Impedance spectrum computation
- Resonance peak detection
- Fundamental frequency analysis

### Parametric Shape Generation
- Conical bores
- Cylindrical drones  
- Kigali-style power-law tapers
- Mbeya-style multi-section designs
- Custom bubble/bulge insertion

### Evolutionary Optimization
- Multi-objective loss functions
- Target sound descriptions
- Population-based genetic algorithms
- Configurable mutation and crossover operators

## 🚀 Quick Start

### Run the DidgeLab Demo (Command Line)

```bash
cd rust-cadsd-accurate
cargo run --bin didgelab-demo
```

This demonstrates 4 examples of inverse design:
1. Simple cylindrical drone in D
2. Design with specific toots (D and A)
3. Flared horn design
4. Complex multi-objective optimization

### Run the DidgeLab GUI (Desktop App)

```bash
cd rust-cadsd-accurate
cargo run --features gui-didgelab -- --didgelab
```

The GUI provides:
- Interactive sound description interface
- Real-time optimization progress
- Impedance spectrum visualization
- Bore profile preview
- Export to file formats

### Run Traditional CADSD Demo

```bash
cargo run --bin cadsd
```

## 📖 Usage Examples

### Example 1: Design a Didgeridoo in D1

```rust
use cadsd_accurate::{InverseDesigner, TargetSound, BoreShapePreference};

// Describe the sound you want
let target = TargetSound::new(73.4) // D1 ≈ 73.4 Hz
    .with_bore_shape(BoreShapePreference::Cylindrical)
    .with_length_range(1200.0, 1800.0);

// Run optimization
let designer = InverseDesigner::new()
    .with_population_size(50)
    .with_generations(100);

let result = designer.design(target)?;

// Use the resulting geometry
println!("Length: {:.1} mm", result.geometry.length());
println!("Bell: {:.1} mm", result.geometry.bellsize());
```

### Example 2: Design with Toots

```rust
let target = TargetSound::new(73.4) // D1
    .with_toot(293.7)  // D4 toot
    .with_toot(440.0)  // A4 toot
    .with_overtones(vec![2, 3, 4, 5, 6])
    .with_bore_shape(BoreShapePreference::Conical);

let result = InverseDesigner::new().design(target)?;
```

### Example 3: From Musical Note

```rust
let target = TargetSound::from_note("D1")?
    .with_toot_note("D4")?
    .with_toot_note("A4")?
    .with_bore_shape(BoreShapePreference::Flared);
```

## 🏗️ Architecture

### Core Modules

- **`geo`**: Geometry representation and parametric shapes
- **`sim`**: Acoustic simulation (Transmission Line Modeling)
- **`evo`**: Evolutionary optimization algorithms
- **`loss`**: Multi-objective loss functions
- **`conv`**: Note/frequency conversion utilities
- **`analysis`**: Spectrum analysis and visualization
- **`inverse_design`**: DidgeLab-style inverse design engine
- **`didgelab_app`**: Modern egui-based GUI application

### Data Flow

```
Target Sound Description
    ↓
DidgeLabLoss (Multi-objective loss function)
    ↓
InverseDesigner (Evolutionary optimizer)
    ↓
GeoGenome Population (Candidate geometries)
    ↓
Best Geometry + Acoustic Analysis
    ↓
Visualization + Export
```

## 📊 Loss Function Components

The DidgeLab loss function optimizes for:

1. **Fundamental frequency** (weight: 2.0) - Match target key
2. **Toot frequencies** (weight: 1.5) - Match target resonance peaks
3. **Overtone alignment** (weight: 1.0) - Harmonic series alignment
4. **Bore shape** (weight: 0.5) - Match shape preference
5. **Volume** (weight: 0.3) - Reasonable volume constraints
6. **Length constraints** - Stay within specified range
7. **Bell diameter constraints** - Stay within specified range

## 🔧 Configuration

### Optimization Parameters

- **Population size**: 20-100 (default: 50)
- **Generations**: 20-200 (default: 100)
- **Mutation rate**: 0.15
- **Crossover rate**: 0.8
- **Elite size**: 3

### Geometry Constraints

- **Length**: 500-3000 mm
- **Top diameter**: 10-100 mm
- **Bell diameter**: 20-150 mm
- **Segments**: 10-50

## 📁 File Formats

### Export Formats

**TXT Format** (X-Y coordinates):
```
0.0 32.0
100.0 35.5
200.0 40.2
...
```

**JSON Format**:
```json
[
  [0.0, 32.0],
  [100.0, 35.5],
  [200.0, 40.2]
]
```

## 🧪 Testing

Run all tests:

```bash
cargo test
```

Run specific module tests:

```bash
cargo test inverse_design
cargo test loss
cargo test evo
```

## 📚 References

- Based on [DidgeLab.com](https://didgelab.com) functionality
- Frank Geipel's DidgeLab Python toolkit
- Transmission Line Modeling for acoustic simulation
- Evolutionary algorithms for inverse design

## 🎯 Roadmap

- [x] Inverse design engine
- [x] Multi-objective loss functions
- [x] Target sound descriptions
- [x] Desktop GUI application
- [x] Impedance spectrum visualization
- [x] Bore profile visualization
- [x] Export to file formats
- [ ] Real-time audio synthesis
- [ ] WebAssembly web app
- [ ] 3D mesh export (STL, OBJ)
- [ ] Advanced bore profiles (NURBS)
- [ ] Multi-material simulation

## 📄 License

CC-BY-NC-SA-4.0 (Creative Commons Attribution-NonCommercial-ShareAlike 4.0)

## 🤝 Contributing

Contributions welcome! Please submit issues and pull requests.
