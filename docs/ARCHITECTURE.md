# Didgerust — Architecture, UI & Development Plan

> Practical implementation reference. Complements `RESEARCH.md` (theory/physics) and `docs/losses.md` (loss-function catalog).

---

## 1. System Architecture

### 1.1 Tech Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Language | Rust (2021 edition) | Safety, performance, single-binary distribution |
| Math/core | `nalgebra`, `num-complex` | Matrix ops, complex arithmetic |
| Parallelism | `rayon` | Data-parallel loss evaluation |
| GUI | `bevy` + `bevy_egui` + `egui_plot` | Immediate-mode UI inside ECS; `rfd` for file dialogs |
| Serialization | `serde`, `serde_json` | Settings, checkpoints, geometry I/O |
| Optional ML | `dfdx`, `tch-rs`, `burn` (behind feature flags) | See §6 |

### 1.2 Module Layout

```
src/
  lib.rs              – crate root; declares public modules
  main.rs             – binary entrypoint
  app.rs              – CadsdState, Bevy systems, panel renderers
  bin/
    gui.rs            – Bevy app entrypoint
    cli.rs            – CLI for non-Rust developers (simulate/optimize/validate/ml/waveguide/tonehole/primes)
  geo/                – (in rust-cadsd-accurate) Geo geometry representation + ops
  sim/mod.rs          – TLM cascade, impedance strategies, peak detection, physics models
  waveguide/mod.rs    – Digital waveguide prototype (frequency-domain)
  evo/mod.rs          – Genome trait, BaseGenome, KigaliGenome, optimizer, PrimeGenerator
  loss/mod.rs         – LossComponent trait + 10+ concrete losses, PeakDetectionMode
  tonehole/mod.rs     – Tonehole models (open/closed impedance)
  nn/mod.rs           – Neural integration placeholders (behind nn-integration flag)
  persistence/mod.rs  – AppSettings, ProjectState, OptimizerCheckpoint
  prime_conv/mod.rs   – Complex-valued prime-kernel CNN (ComplexPrimeMLP, PrimeConvBlock)
  fdtd/               – 3-D acoustic FDTD validator
  visualization/      – (future) plotters/egui_plot helpers
  integration/        – (future) external tool bridges
  export/             – (future) WAV, CSV, geometry serialization
  dwm/                – Digital waveguide mesh prototypes
```

### 1.3 Data Flow

```
Geometry (Geo: Vec<[f64;2]> mm)
    │
    ▼
create_segments_from_geo() → Vec<Segment> (m)
    │
    ▼
DidgeridooSimulator::impedance(freqs)
    │   ├─ Tlm        → cadsd_ze_with_losses() cascade (transfer matrices + viscothermal losses)
    │   ├─ Waveguide  → WaveguideEngine::transfer_function()
    │   └─ ComplexImpedance → frequency-domain with boundary-layer attenuation
    ▼
Vec<Complex<f64>> spectrum
    │
    ▼
find_peaks() / find_peaks_with_prominence() / find_peaks_phase_based() → Vec<(idx, freq, mag)>
    │
    ▼
LossFunction::calculate(genome) → f64 (or HashMap)
    │
    ▼
EvolutionaryOptimizer::evolve() → best genome
```

### 1.4 State Ownership

- **Bevy resource**: `CadsdState` holds all UI state, geometry params, simulation results, optimizer progress.
- **Derived geometry**: `current_geo(state)` reconstructs `Geo` from sliders on every frame — no persistent Geo in state until operations complete.
- **History**: `geo_history: Vec<GeoHistoryEntry>` + `geo_history_index` for undo/redo.
- **Persistence**: `AppSettings` saved to disk on demand; `OptimizerCheckpoint` saved per-generation.

---

## 2. Current Codebase Reality

### 2.1 Implementation Status

| Component | Location | Status | Notes |
|-----------|----------|--------|-------|
| TLM cascade | `src/sim/mod.rs::cadsd_ze_with_losses` | ✅ Complete | Full transfer-matrix cascade with viscothermal losses |
| Strategy dispatch | `SimulationStrategy` + `DidgeridooSimulator::impedance` | ✅ Complete | Tlm/Waveguide/ComplexImpedance dispatch |
| Waveguide freq-domain | `src/waveguide/mod.rs::WaveguideEngine` | ⚠️ Partial | Returns complex spectrum; no time-domain synthesis |
| Complex impedance | `SimulationStrategy::ComplexImpedance` | ⚠️ Partial | Basic implementation; needs validation against TLM |
| Geometry ops | `rust-cadsd-accurate/src/geo/mod.rs` | ✅ Complete | cone, cylinder, bubble, stretch, scale, volume, Kigali, Mbeya |
| Evolutionary optimizer | `src/evo/mod.rs` | ✅ Complete | Multiple mutation/crossover strategies, tournament selection, elite preservation, async execution |
| Loss functions | `src/loss/mod.rs` | ✅ Complete | 10+ components, CompositeTairuaLoss with PeakDetectionMode, from_toggles() |
| Peak detection | `src/sim/mod.rs::find_peaks*` | ✅ Complete | Three modes: local maxima, prominence, phase-based |
| Persistence | `src/persistence/mod.rs` | ✅ Complete | JSON save/load for settings, checkpoints, project state |
| GUI | `src/app.rs`, `src/bin/gui.rs` | ⚠️ Partial | Bevy + egui app launches; real async optimizer; some UI features missing |
| Strategy comparison | `run_comparison_simulation` | ✅ Complete | Overlay plot with 3-line legend |
| Radiation impedance | `src/sim/mod.rs::za` | ⚠️ Partial | Levine-Schwinger IIR; doc comments mislabel as "Geipel" |
| Viscothermal losses | `src/sim/mod.rs::viscothermal_k_complex` | ⚠️ Partial | Full Tw/Zcw system implemented; needs validation against published data |
| AcousticConstants | `src/sim/mod.rs::AcousticConstants` | ⚠️ Partial | Temperature, pressure, humidity via `for_conditions` |
| Bent-shape correction | `src/sim/mod.rs::bent_effective_length` | ✅ Complete | Analytical formula, wired into optimizer loss and GUI sliders |
| Tonehole models | `src/tonehole/mod.rs` | ⚠️ Partial | Open/closed impedance; no three-port scattering junction |
| Differentiable TLM | `src/diff_tlm.rs` | ⚠️ Partial | Analytical gradients + Adam; no real backprop through cascade |
| FDTD validator | `src/fdtd/mod.rs`, `src/fdtd/validator.rs` | 🔄 Partial | 3-D Yee grid with PML; small grid, no validation study |
| Prime-conv ML | `src/prime_conv/mod.rs` | 🔄 Partial | Forward pass demo; no training pipeline or dataset |
| DWM prototypes | `src/dwm/mod.rs` | 🔄 Partial | 2-D/3-D mesh; not integrated with main simulator |
| Neural fitness predictor | `src/nn/mod.rs` | ❌ Missing | Placeholder only; no MLP, no training loop |
| Time-domain synthesis | `src/waveguide/mod.rs` | ❌ Missing | Frequency-domain only; no sample-by-sample loop |
| GUI tonehole editor | `src/app.rs` | ⚠️ Partial | Sliders work; no drag-and-drop on bore preview |
| 3-D bore preview | `src/app.rs::draw_bore_gizmos` | ⚠️ Partial | Wireframe centerline exists; no camera controls or rotated solid |

### 2.2 What Is Broken or Incomplete

| Component | Issue | Location | Fix Needed |
|-----------|-------|----------|------------|
| Frequency grid | Linear by default; log grid not cents-based everywhere | `src/app.rs::compute_spectrum` | Standardise on cents-based log grid |
| GUI tonehole editor | No drag-and-drop on bore preview | `src/app.rs` | Add gizmo interaction for tonehole markers |
| Cents-based grid | Not used universally in GUI | `src/sim/mod.rs::grid` | Replace linear grid with `log_grid` in simulation panel |
| Segment editor | No table view of individual [x,d] points | `src/app.rs` | Add editor panel for segment-level geometry editing |
| Compute thread count | No user-configurable thread count | `src/app.rs` | Add slider in Settings; wire to rayon thread pool |
| 3-D bore preview | Only 3-line wireframe; no camera controls or rotated solid | `src/app.rs::draw_bore_gizmos` | Implement proper 3D conical frustum + orbit controls |
| Loss caching name | Doc claims `cached_loss` field; actual field is `loss` | `src/evo/mod.rs::Genome` | Rename or update docs |

### 2.3 What Does Not Exist Yet

| Feature | Notes |
|---------|-------|
| Neural fitness predictor training pipeline | No MLP, no dataset generation, no training loop |
| Time-domain synthesis for audio | No sample-by-sample waveguide loop; no cpal audio output from simulation |
| Moist-air AcousticConstants | Humidity/pressure dependence exists via `for_conditions`; needs experimental validation |
| 3-D FDTD validation study | No comparison against measured data or FEM reference |
| Prime-conv training pipeline | No dataset, no training loop, no serialization |
| DWM integration | Not wired into `SimulationStrategy` dispatch |
| Segment editor | No table view of individual `[x, d]` points |
| Compute thread count | No user-configurable thread count |
| True 3-D bore preview | No camera controls, no rotated solid (only centerline wireframe) |
| Tonehole drag-and-drop | No interactive tonehole placement on bore preview |

---

## 3. UI Inventory & Gap Analysis

### 3.1 Existing Panels

| Panel | File | Elements | Status |
|-------|------|----------|--------|
| Simulation | `show_simulation_panel` | Freq grid sliders, compute button, peak button, export CSV/JSON, spectrum plot with tooltip, phase toggle | ⚠️ Code ready, untested |
| Optimizer | `show_optimizer_panel` | Pop/gen sliders, loss toggles with weights, mutation display, progress bar, conservation dashboard, run/pause/resume, save/load checkpoint, export genome | ⚠️ Code ready, untested |
| Geometry | `show_geometry_panel` | Length/diameter/segment sliders, bore style presets, undo/redo, bubble/stretch dialogs, import/export JSON, curvature/taper sliders, bore profile plot | ⚠️ Code ready, untested |
| Settings | `show_settings_panel` | Theme, log verbosity, default strategy/mutation, budget, γ/η weights, prime sieve size, waveguide delay resolution, phase unwrap, save/load config | ⚠️ Code ready, untested |
| Sidebar | `ui_system` | Strategy radios, mutation radio, budget slider, export geometry, compare strategies | ⚠️ Code ready, untested |

### 3.2 Dialogs (Modal Windows)

| Dialog | Trigger | Status |
|--------|---------|--------|
| Export Geometry | Sidebar button | ✅ JSON/CSV radio, logs to stdout |
| Compare Strategies | Sidebar button | ✅ Overlay plot with 3-line legend |
| Add Bubble | Geometry panel | ✅ Position/width/height sliders |
| Stretch Geometry | Geometry panel | ✅ Factor slider |
| Import Geometry JSON | Geometry panel | ✅ Text paste + parse |
| Save Configuration | Settings panel | ✅ Path text field |
| Load Configuration | Settings panel | ✅ Path text field |
| Resume from Checkpoint | Optimizer panel | ✅ Path text field |
| Export Best Genome | Optimizer panel | ✅ Path text field |

### 3.3 Missing UI Features (Priority Order)

**P0 — Blocking real use:**
1. **Segment editor** — table view of individual `[x, d]` points with add/remove/reorder

**P1 — Needed for usability:**
2. **True 3-D bore preview** — Bevy gizmos showing rotated conical frustum with camera controls
3. **Tonehole drag-and-drop** — add/remove toneholes on bore preview
4. **Phase-aware spectrum toggle** — overlay unwrapped phase on spectrum plot (partially exists)
5. **Peak markers on plot** — draw vertical lines/dots at detected resonances
6. **Strategy legend** — show which curve is which when comparing

**P2 — Nice to have:**
7. **Cents-based log grid everywhere** — standardise frequency grid to cents-based spacing
8. **Compute thread count** — slider for parallel loss evaluation thread count
9. **Auto-save interval** — for checkpoints
10. **Population diversity metric** — average pairwise genome distance
11. **Convergence stop condition** — early stopping if best loss plateaus

---

## 4. Development Plan

### 4.1 Phase A — Core Stability (Complete)
- [x] TLM cascade with complex arithmetic
- [x] Strategy dispatch (TLM / Waveguide / ComplexImpedance)
- [x] Geometry module (cone, bubble, stretch, volume)
- [x] Evolutionary optimizer (Gaussian + PrimeSequence)
- [x] Loss functions (10+ components)
- [x] Persistence (settings, checkpoints)
- [x] GUI shell code (Bevy + egui, 4 panels) — ⚠️ untested

### 4.2 Phase B — UI Completion (Partial)
**Owner:** Frontend / GUI  
**Blocked by:** None

| Task | File(s) | Effort | Notes |
|------|---------|--------|-------|
| Wire optimizer loop | `src/app.rs`, `src/evo/mod.rs` | Medium | ✅ Background thread + mpsc progress callbacks implemented |
| Add `rfd` file dialogs | `src/app.rs` | Small | ✅ Done |
| Fix undo/redo off-by-one | `src/app.rs:901-911` | Tiny | ✅ Done |
| Add `prominence` to `find_peaks` | `src/sim/mod.rs`, `src/app.rs` | Small | ✅ Done |
| Cents-based log grid | `src/sim/mod.rs::grid`, `src/app.rs::compute_spectrum` | Small | ⚠️ Log grid exists; linear still default in GUI |
| Loss caching on genome | `src/loss/mod.rs`, `src/evo/mod.rs` | Small | ✅ Done (`loss` field on Genome) |
| Export CSV with magnitude+phase | `src/app.rs::export_spectrum_csv` | Tiny | ✅ Done |
| Test UI manually/automated | `src/app.rs`, `src/bin/gui.rs` | Medium | ❌ No UI testing done; all panels untested |
| Bent-shape correction in GUI | `src/app.rs`, `src/sim/mod.rs`, `src/loss/mod.rs` | Medium | ✅ Curvature/taper sliders + BentEffectiveLengthLoss wired |
| Loss component toggles functional | `src/app.rs`, `src/loss/mod.rs` | Small | ✅ CompositeTairuaLoss::from_toggles() builds from GUI state |
| Conservation dashboard | `src/app.rs` | Small | ✅ γ/η visualization in Optimizer panel |
| Prime-based population init | `src/evo/mod.rs` | Small | ✅ with_prime_population() uses prime-seeded RNG |
| Export JSON | `src/app.rs` | Tiny | ✅ Export JSON button + export_spectrum_json() |
| Advanced config panel | `src/app.rs` | Small | ✅ γ/η weights, prime sieve size, delay resolution, phase unwrap |
| Export CSV with magnitude+phase | `src/app.rs::export_spectrum_csv` | Tiny | ✅ Done |
| Test UI manually/automated | `src/app.rs`, `src/bin/gui.rs` | Medium | ❌ No UI testing done; all panels untested |

### 4.3 Phase C — Physics Accuracy (Partial)
**Owner:** Simulation / Physics  
**Dependencies:** Phase B complete

| Task | File(s) | Effort | Notes |
|------|---------|--------|-------|
| Radiation impedance | `src/sim/mod.rs::za` | Medium | ⚠️ Geipel approximation; upgrade to Levine-Schwinger IIR |
| Viscothermal losses | `src/sim/mod.rs::cadsd_ze_with_losses` | Medium | ⚠️ Full Tw/Zcw system implemented; needs validation against published data |
| AcousticConstants | `src/sim/mod.rs::AcousticConstants` | Small | ⚠️ Temperature-dependent only; add humidity/pressure |
| Bent-shape correction | `src/sim/mod.rs::bent_effective_length` | Large | 🔄 Analytical formula exists; wire into optimizer and GUI |

### 4.4 Phase D — Evolution Engine Enhancements (Complete)
**Owner:** Optimization  
**Dependencies:** Phase B

| Task | File(s) | Effort | Notes |
|------|---------|--------|-------|
| Add mutation operators | `src/evo/mod.rs` | Small | ✅ Done |
| Phase-based resonance finder | `src/sim/mod.rs` | Medium | ✅ Done |
| Loss caching | `src/evo/mod.rs`, `src/loss/mod.rs` | Small | ✅ Done |
| Prominence-based peak detection | `src/sim/mod.rs` | Small | ✅ Done |
| Tonehole support | `src/tonehole/mod.rs` | Medium | ⚠️ Open/closed impedance done; three-port junction missing |

### 4.5 Phase E — Machine Learning Integration (Partial)
**Owner:** ML / Research  
**Dependencies:** Phase C (accurate simulator) for training data

| Task | Crate | Effort | Notes |
|------|-------|--------|-------|
| Differentiable TLM | `src/diff_tlm.rs` | Large | ⚠️ Analytical gradients + Adam exist; no real backprop through cascade |
| Complex-valued NN primitives | Custom `src/nn/mod.rs` | Large | 🔄 Placeholder exists; no training pipeline |
| Neural fitness predictor | `tch-rs` or `dfdx` | Large | ❌ No MLP, no dataset, no training loop |
| FDTD validator | `src/fdtd/mod.rs` | Large | 🔄 3-D Yee grid implemented; needs validation study |
| Prime-conv ML | `src/prime_conv/mod.rs` | Large | 🔄 Forward pass demo; no training, no dataset |
| DWM prototypes | `src/dwm/mod.rs` | Large | 🔄 2-D/3-D mesh; not integrated with main simulator |

**Feature flags:**
```toml
[features]
default = []
gui-bevy = ["bevy", "bevy_egui", "bevy_gizmos", "egui_plot", "rfd"]
nn-integration = []        # placeholder module exists; future: "tch-rs"
diff-tlm = ["autodiff-rs"] # differentiable TLM prototype
fdtd-validator = []        # 3-D acoustic FDTD (no external dep)
cpal-integration = ["cpal"] # audio output
```

### 4.6 Phase F — Polish & Performance (Ongoing)

- [x] Remove duplicate `PrimeGenerator` ✅ Done
- [x] Standardize frequency grid to cents-based log spacing ✅ Done in `sim::grid`
- [x] Add `cargo bench` benchmarks ✅ `benches/waveguide_vs_tlm.rs`, `benches/loss_benchmark.rs`
- [ ] Expand unit tests (currently 102 library + 30 integration; target 150+)
- [x] Add integration tests for full Geo → impedance → peaks → loss → optimizer pipeline ✅ Done
- [x] Wire real optimizer loop with progress callbacks ✅ Done
- [ ] Add 3-D bore preview camera controls
- [ ] Add tonehole drag-and-drop on bore preview
- [ ] Add segment editor (table view of [x,d] points)
- [ ] Add compute thread count configuration

---

## 5. UI Requirements (Detailed)

### 5.1 Simulation Panel

**Current:**
- Frequency grid config (min/max/points, log toggle)
- Compute spectrum button
- Find peaks button
- Export CSV button (with rfd file picker)
- Spectrum plot with hover tooltip
- Fundamental frequency display
- Strategy comparison overlay

**Missing:**
- **Phase overlay toggle** — show unwrapped phase (degrees) on secondary y-axis
- **Peak markers** — draw vertical lines / dots at detected resonances on plot
- **Strategy legend** — show which curve is which when comparing
- **Export impedance CSV/JSON with phase** — include `frequency_hz, magnitude, phase_deg`
- **Grid type selector** — cents-step log vs. point-count log vs. linear (currently only point-count log)

### 5.2 Optimizer Panel

**Current:**
- Population / generation / rate sliders
- Loss component toggles with inline weight sliders
- Generation progress bar
- Run / Pause / Resume buttons (log only)
- Save / Load checkpoint (with rfd file picker)
- Export best genome (with rfd file picker)
- Mutation strategy radio (Gaussian, PrimeSequence, SingleMutation)
- Crossover strategy radio (SinglePoint, Average, PartSwap, PartAverage)

**Missing:**
- **Real async execution** — run optimizer in background; update `best_loss`, `current_generation`, `generation_progress` live
- **Loss curve plot** — best loss per generation (time series)
- **Convergence stop condition** — early stopping if best loss plateaus
- **Population diversity metric** — e.g., average pairwise genome distance

### 5.3 Geometry Panel

**Current:**
- Length / top-diameter / bottom-diameter / segment sliders
- Undo / redo buttons (fixed)
- Add bubble dialog
- Stretch dialog
- Import / export JSON (with rfd file picker)
- Bore profile preview (2D cross-section)

**Missing:**
- **3-D wireframe preview** — Bevy gizmos showing bore as rotated cylinder/conical frustum
- **Segment editor** — table view of individual `[x, d]` points with add/remove/reorder
- **Parametric shape presets** — Kigali, Mbeya, cone, cylinder one-click generators
- **Curvature editor** — for bent-shape correction (Phase C)
- **Volume / surface-area readout** — already have volume; add surface area
- **Tonehole editor** — add/remove toneholes with position/diameter/depth

### 5.4 Settings Panel

**Current:**
- Theme (light/dark)
- Log verbosity (0-4)
- Default strategy / mutation
- Budget ops slider
- Export format radio
- Save / load config (with rfd file picker)
- Reset to defaults

**Missing:**
- **Default mutation operator** — extend beyond Gaussian / PrimeSequence / SingleMutation
- **Compute thread count** — for parallel loss evaluation
- **Auto-save interval** — for checkpoints
- **Acoustic constants editor** — temperature, humidity, pressure (Phase C)

---

## 6. Machine Learning Integration Roadmap

### 6.1 Crate Selection

| Task | Primary | Fallback | Rationale |
|------|---------|----------|-----------|
| Differentiable TLM | `dfdx` | `autodiff-rs` scalar | Compile-time graph opt; custom complex backward passes |
| PINN / surrogate training | `tch-rs` | `burn` | GPU support, `Tensor::of_complex()`, mature ecosystem |
| Neural fitness predictor | `tch-rs` | `dfdx` | Easy serialization, TorchScript export |
| Complex-valued ops | Custom `src/nn/mod.rs` | — | `renplex` archived; extract primitives |

### 6.2 Integration Sequence

1. **Prototype differentiable TLM** (1-2 weeks)
   - Implement `Value` type supporting `Complex64` data
   - Wrap `Segment` parameters as `Value`
   - Cascade transfer matrices using `Value` arithmetic
   - Numerical gradient check against analytical derivatives

2. **Replace evolutionary loop with Adam** (1 week)
   - Benchmark convergence: gradient-based vs. genetic algorithm
   - Success criterion: sub-cent tuning in <500 steps vs. >10,000 evaluations

3. **Train neural fitness predictor** (2-3 weeks)
   - Generate dataset: 10,000 genomes → TLM impedance peaks
   - MLP: 2 hidden layers, 64 units, ReLU
   - Use as surrogate during evolution; true TLM on elite 5%
   - Success: 5–10× wall-clock speedup, <2% final loss degradation

4. **Complex-valued PINN for bent geometries** (4-6 weeks)
   - Training data: FDTD or corrected TLM on 1,000 random bent shapes
   - Architecture: MLP with complex weights, CReLU activation
   - Loss: MSE on complex impedance + PDE residual (1-D wave equation)
   - Inference: <1 ms forward pass

---

## 7. Reference Repositories

All three repos are cloned locally under the project root and gitignored:

| Repo | Location | Purpose |
|------|----------|---------|
| `autodiff-rs` | `./autodiff-rs/` | Scalar autodiff reference for differentiable TLM |
| `renplex` | `./renplex/` | Complex-valued NN primitives reference |
| `fdtd-waveguide` | `./fdtd-waveguide/` | 3-D FDTD algorithmic skeleton |

Do **not** add these as Cargo dependencies. Study them for patterns and extract only what is needed into `didgerust` modules.

---

## 8. Key Decisions & Constraints

1. **Shell principle** — new simulation strategies, loss components, and mutation operators are additive. Never break existing TLM path.
2. **12V constraint** — all new computation paths must respect `budget_ops` limit. This is an internal design constraint, NOT an external SuperInstance dependency.
3. **Conservation budget** — `γ + η = C` where γ = simulation ops, η = evolution overhead. Implemented as `CadsdState::budget_ops` slider.
4. **Feature flags** — ML integrations behind `nn-integration`, `diff-tlm`, `fdtd-validator`. Default build stays lightweight.
5. **Units** — geometry in mm internally; convert to m at `Segment::new`. Frequency in Hz. Impedance in Pa·s/m³ (characteristic).
6. **Complex arithmetic** — use `num_complex::Complex64` everywhere; do not introduce `Cf32` unless extracting from `renplex`.

---

## 9. Immediate Next Actions

1. **Add segment editor** — table view of `[x, d]` points in geometry panel
2. **Add true 3-D bore preview** — Bevy gizmos conical frustum with orbit controls
3. **Add tonehole drag-and-drop** — interactive placement on bore preview
4. **Add compute thread count** — slider in Settings panel wired to rayon
5. **Fix radiation impedance docs** — rename "Geipel" comments to "Levine-Schwinger IIR"
6. **Standardise frequency grid** — cents-based log spacing as default everywhere
7. **Add UI tests** — manual + automated tests for panel interactions
8. **Expand unit tests** — target 150+ library tests from current 102
9. **Wire conservation-law crate** — use `SymplecticIntegrator` in long-time simulation loops
10. **Wire lau-signal-processing crate** — use FFT/IirFilter for bore-wall absorption modeling
