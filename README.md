# DidgeRust - CADSD Didgeridoo Simulator

A Rust-based Computer-Aided Didgeridoo Sound Design (CADSD) toolkit featuring TLM simulation, evolutionary optimization, and a Bevy/egui GUI.

## CLI Tools (for non-Rust developers)

The CLI provides access to all core scientific features without needing Rust knowledge.

### Quick Start

```bash
# Simulate a cone and print results
cargo run --bin cli -- simulate cone --length 1500 --top 32 --bottom 65

# Optimize a cone for 20 generations
cargo run --bin cli -- optimize cone --length 1500 --top 32 --bottom 65 --generations 20

# Validate TLM vs analytical solution
cargo run --bin cli -- validate cone --length 1500 --top 32 --bottom 65
```

### Advanced Features (Experimental)

```bash
# Run ML prime-conv demo (complex neural network with prime kernels)
cargo run --bin cli -- ml primes --max-prime 17 --input 10

# List prime numbers up to 100
cargo run --bin cli -- primes list --max 100

# Run 3D waveguide simulation
cargo run --bin cli -- waveguide cone --length 1500 --top 32 --bottom 65

# Compute tonehole impedance spectrum
cargo run --bin cli -- tonehole --diameter 10 --depth 5 --open
```

### JSON Output

Add `--json` to any command for machine-readable output:

```bash
cargo run --bin cli -- simulate cone --length 1500 --top 32 --bottom 65 --json
```

### Example Config Files

See the `examples/` directory for sample JSON configurations:
- `cone.json` - Basic cone geometry
- `kigali.json` - Kigali-style geometry
- `optimize.json` - Optimization parameters

## Planned Future Work

## 1. Radiation Impedance Model
- Currently implemented: Levine-Schwinger IIR approximation (unflanged pipe)
- Validation: Matches Geipel approximation within expected tolerance
- Future: Add Silva et al. rational approximation for flanged pipes

## 2. Viscothermal Loss Model
- Implemented: Full Tw/Zcw complex wavenumber system (DidgeLab formulation)
- Validation: Reproduces published efficiency curves
- Future: Add frequency-dependent boundary-layer thickness validation

## 3. Bent Geometry Correction
- Status: IMPLEMENTED — α·κ²·a² effective-length correction wired into simulator, optimizer, and GUI
- `src/sim/mod.rs::bent_effective_length` + `Segment::new_with_curvature` + GUI sliders
- Future: Extend to non-uniform curvature along bore length

## 4. Mutation Operator Expansion
- Status: IMPLEMENTED — SingleMutation, AverageCrossover, PartSwapCrossover, PartAverageCrossover
- All four operators are unit-tested and available in optimizer

## 5. Peak Detection Enhancement
- Status: IMPLEMENTED — prominence parameter + phase-based resonance detection (Ernoult et al. 2020)
- All three modes available: local maxima, prominence, phase-based

## 6. Loss Caching Mechanism
- Status: IMPLEMENTED — `loss: Option<f64>` on Genome trait; `evaluate_genomes` skips cached evaluations
- Note: Field is named `loss`, not `cached_loss` as originally planned