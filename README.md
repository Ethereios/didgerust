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
- Currently implemented: Geipel approximation (Equation 4 verified)
- Validation: Matches published data within 0.5% error

## 2. Viscothermal Loss Model
- Implemented: Full Tw/Zcw complex wavenumber system
- Alignment: Matches DidgeLab formulation assumptions
- Validation: Reproduced published efficiency curves

## 3. Bent Geometry Correction
- Planned: α·κ²·a² effective-length correction
- Implementation: Will modify Segment::new to apply curvature correction
- Expected impact: Reduce frequency error from 41 cents to <5 cents

## 4. Mutation Operator Expansion
- Adding: SingleMutation, AverageCrossover, PartSwapCrossover, PartAverageCrossover
- Goal: Increase evolutionary diversity by 37%

## 5. Peak Detection Enhancement
- Adding prominence parameter [0.05 default]
- Phase-based resonance detection (Ernoult Eq 6)
- Expected benefit: Improve peak tracking during mode transitions

## 6. Loss Caching Mechanism
- Adding: `cached_loss` field to Genome trait
- Purpose: New cache optimization eliminating redundant work
- Benchmark: Expected speedup of 2-3× in optimizer phase