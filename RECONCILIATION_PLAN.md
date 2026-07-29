# Reconciliation Plan

This document proposes how to reconcile the `cadsd` (wrapper) and `cadsd-accurate` crates given their current divergence.

## Current state summary

- `cadsd-accurate` holds the **reference implementation** with full viscothermal physics, exact Python DidgeLab parity, GUI, CLI, audio, export, persistence, and inverse-design modules.
- `cadsd` (wrapper) holds a **simplified Rust-first API** with lossless physics, a different evo system (`Genome` trait + `BaseGenome`/`KigaliGenome`), modular loss components, and stub visualization.
- Both crates share the same conceptual domain (didgeridoo geometry, impedance simulation, evolutionary optimization) but have **diverged in type names, physics fidelity, and module coverage**.
- `cadsd` already depends on `cadsd-accurate` as a path dependency.

## Target architecture: Adapter layer within `cadsd`

**Decision**: Keep both crates, but convert `cadsd` into a **Rust-idiomatic facade** over `cadsd-accurate`.

Rationale:
1. `cadsd-accurate` is the correctness anchor (Python parity). Changing it risks breaking the reference implementation.
2. `cadsd` already exists as the consumer-facing crate and has nicer Rust APIs (`DidgeridooSimulator`, `Genome` trait, `CompositeTairuaLoss`).
3. An adapter layer avoids duplicating physics or peak-picking logic while still letting `cadsd` users use the simplified API.

### High-level structure

```
cadsd-accurate  (unchanged reference implementation)
      ↑
      │ path dependency + feature flags
      │
cadsd  (facade + adapters)
  ├── geo  →  adapts cadsd_accurate::Geo
  ├── sim  →  wraps acoustical_simulation with DidgeridooSimulator
  ├── evo  →  keeps Genome trait, delegates loss to accurate
  ├── loss →  maps CompositeTairuaLoss → TairuaLoss components
  ├── visualization →  delegates to accurate analysis + real plotters
  └── lib.rs →  re-exports accurate conv, integration, etc.
```

## Step-by-step alignment plan

### Step 1: Unify geometry (`geo`)

**Goal**: `cadsd::Geo` should be a thin newtype or alias over `cadsd_accurate::Geo`.

Tasks:
- Add `pub struct Geo(pub cadsd_accurate::Geo);` or re-export.
- Provide conversion constructors:
  - `Geo::new(points: Vec<[f64; 2]>)` → calls `cadsd_accurate::Geo::new()`
  - `Geo::make_cone(...)`, `Geo::make_kigali(...)`, `Geo::make_mbeya(...)` → delegate
- Preserve wrapper convenience methods:
  - `length()`, `bellsize()`, `diameter_at_x()`, `stretch()`, `add_bubble()`, `compute_volume()`
  - `get_max_diameter()` → rename to `get_max_d()` or add alias
- **Deprecate** wrapper-only `scale_diameter(factor)` in favor of accurate `scale_diameter(max_d)` or keep both with distinct names.
- Align `compute_volume()` formula to accurate (`/12.0`) after validation, or add `compute_volume_trapezoid()` alias.

### Step 2: Align simulation (`sim`)

**Goal**: `DidgeridooSimulator` should delegate to `cadsd_accurate::sim::acoustical_simulation`.

Tasks:
- Keep `DidgeridooSimulator` as a convenience wrapper:
  - `from_geo(geo)` stores adapted `Geo`
  - `impedance(freqs)` calls `acoustical_simulation(&self.geo, freqs, "tlm_cython")` and returns `Vec<Complex<f64>>` (wrapper can complexify accurate magnitudes)
  - `peaks(freqs)` uses wrapper `find_peaks()` on complex spectrum, or delegates to accurate `get_notes()` after complexifying
  - `find_resonance_peaks()` uses `get_log_simulation_frequencies()` and accurate peak extraction, then wraps in `Resonance`
- **Deprecate** the wrapper-only `cadsd_ze` and `Segment` struct in favor of accurate internals, or keep them private.
- Remove the placeholder `za()` and lossless `k = ω/c` propagation from wrapper `sim`.

### Step 3: Reuse accurate peak / analysis semantics

**Goal**: Peak extraction and resonance detection use identical definitions in both crates.

Tasks:
- Replace wrapper `find_peaks()` implementation with a call to accurate `analysis::get_notes()` when operating on accurate magnitudes, or keep wrapper for complex inputs.
- Ensure both use strict local maxima: `imp[i] > imp[i-1] && imp[i] > imp[i+1]`.
- Ensure `find_resonance_peaks()` and `get_notes()` return frequencies in ascending order.

### Step 4: Align loss functions

**Goal**: Wrapper `CompositeTairuaLoss` should reuse accurate `TairuaLoss` where possible.

Tasks:
- `CompositeTairuaLoss::calculate(genome)` can:
  1. Convert genome → `Geo` (already done)
  2. Call `cadsd_accurate::loss::TairuaLoss::compute_loss(&geo)` for the base acoustic loss
  3. Add wrapper-specific components (frequency tuning, Q-factor, modal density, etc.) on top
- This preserves the rich wrapper component library while offloading physics to accurate.
- `FrequencyTuningLoss`, `QFactorLoss`, etc. should accept accurate peak data (from `get_notes()`).

### Step 5: Evolve evo systems

**Goal**: Choose one evo system as canonical and adapt the other.

**Option A (recommended)**: Keep wrapper `Genome` trait / `EvolutionaryOptimizer` and have it call accurate `InverseDesigner` under the hood for optimization, while keeping wrapper `KigaliGenome` for parametric shapes.

Tasks:
- `EvolutionaryOptimizer::evolve()` delegates to `cadsd_accurate::integration::DefaultOptimizer::optimize()` wrapped in adapter types.
- `TargetSound` in wrapper maps to accurate `TargetSound`.
- `LossFunction` trait in wrapper maps to `LossFunctionType` in accurate.

**Option B**: Keep accurate `Nuevolution` / `GeoGenome` and add wrapper aliases.

Either way, **do not maintain two independent optimizer implementations**.

### Step 6: Re-export accurate modules from wrapper

**Goal**: `cadsd` becomes the single entry point for users.

Tasks:
- In `cadsd/src/lib.rs`:
  - Re-export `cadsd_accurate::conv::*`
  - Re-export `cadsd_accurate::integration::{AcousticSimulator, DefaultSimulator, EvolutionaryOptimizer, DefaultOptimizer, AudioSynthesizer, GeometryExporter, DefaultSynthesizer, DefaultExporter}`
  - Re-export `cadsd_accurate::inverse_design::{InverseDesigner, DesignResult}`
  - Re-export `cadsd_accurate::audio::DefaultSynthesizer`
  - Re-export `cadsd_accurate::export::DefaultExporter`
  - Re-export `cadsd_accurate::persistence::{AppSettings, ProjectState}`
- Enable `cadsd-accurate` features from wrapper `Cargo.toml` as needed.

### Step 7: Enable real visualization

**Goal**: Replace stubs in `visualization` with real plotting using `plotters` (already dependency of accurate crate).

Tasks:
- `plot_bore_geometry()` → use `plotters` to draw bore profile from `Geo`
- `plot_impedance_spectrum()` → use `plotters` to draw magnitude spectrum
- `plot_evolution_progress()` → use `plotters` to draw loss curves
- `create_analysis_report()` writes real PNG + report.txt

### Step 8: CLI and GUI wiring

**Goal**: Keep accurate crate as the owner of CLI and Bevy GUI.

Tasks:
- Add top-level `cargo run --bin cadsd` alias or re-export `cadsd-accurate` bin via wrapper `Cargo.toml`.
- Document that `cadsd-gui` lives in `cadsd-accurate`.

### Step 9: Deprecation path

| Wrapper item | Action |
|-------------|--------|
| `Geo { points }` field | Add adapter; eventually add `geo` field or keep `points` as getter |
| `Geo::scale_diameter(factor)` | Deprecate; add `Geo::scale_diameter_to(max_d)` |
| `Geo::compute_volume()` | Validate `/3.0` vs `/12.0` and align |
| `DidgeridooSimulator::impedance()` | Keep; delegate to accurate simulation |
| `sim::Segment` (pub) | Make private; expose only via `DidgeridooSimulator` |
| `sim::cadsd_ze()` | Make private |
| `evo::EvolutionaryOptimizer` | Keep; delegate to accurate `DefaultOptimizer` |
| `evo::Genome` trait | Keep; map to accurate `LossFunctionType` |
| `loss::CompositeTairuaLoss` | Keep; compose with accurate `TairuaLoss` |
| `visualization` stubs | Implement with `plotters` |

## Risk areas

1. **Physics behavior change**: If wrapper `DidgeridooSimulator` switches from lossless to full viscothermal model, resonance peak counts and frequencies will shift. This is expected and desired for accuracy, but existing tests and examples may need updates.
2. **Volume formula discrepancy**: Wrapper `compute_volume()` uses per-segment frustum (`/3.0`); accurate uses trapezoidal (`/12.0`). Need to validate which matches the Python reference.
3. **Diameter interpolation discrepancy**: Wrapper uses linear interpolation; accurate uses `atan` formula. Need to validate against Python.
4. **Geo field rename**: Changing `geo.points` to `geo.geo` or vice versa is a breaking change. Prefer adding an accessor / newtype over renaming.

## Milestones

| Milestone | Description | Validation |
|-----------|-------------|------------|
| M1 | `cadsd::Geo` delegates to `cadsd_accurate::Geo` | `cargo test` passes for both crates |
| M2 | `DidgeridooSimulator` delegates to accurate simulation | Impedance magnitudes match accurate within floating-point tolerance |
| M3 | Peak / analysis semantics unified | Wrapper and accurate return identical peaks for same input geometry |
| M4 | Loss functions composed | `CompositeTairuaLoss` produces same values as accurate for same geometry (when component weights match) |
| M5 | Accurate modules re-exported from wrapper | `cadsd::conv::note_to_freq` etc. compile and pass tests |
| M6 | Real visualization replaces stubs | PNG files generated with correct content |
| M7 | CLI / GUI wired | `cargo run --bin cadsd` and `cargo run --bin cadsd-gui` functional |
