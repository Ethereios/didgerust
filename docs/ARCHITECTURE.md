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
  geo/mod.rs          – Geo geometry representation + ops
  sim/mod.rs          – TLM cascade, impedance strategies, peak detection, physics models
  waveguide/mod.rs    – Digital waveguide prototype (frequency-domain)
  evo/mod.rs          – Genome trait, BaseGenome, KigaliGenome, optimizer, PrimeGenerator
  loss/mod.rs         – LossComponent trait + 10+ concrete losses, PeakDetectionMode
  tonehole/mod.rs     – Tonehole models (open/closed impedance)
  nn/mod.rs           – Neural integration placeholders (behind nn-integration flag)
  persistence/mod.rs  – AppSettings, ProjectState, OptimizerCheckpoint
  app.rs              – CadsdState, Bevy systems, panel renderers
  bin/gui.rs          – Bevy app entrypoint
  visualization/      – (future) plotters/egui_plot helpers
  integration/        – (future) external tool bridges
  export/             – (future) WAV, CSV, geometry serialization
  fdtd/               – (future) 3-D acoustic FDTD validator
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

### 2.1 What is Wired and Working

| Component | Location | Status |
|-----------|----------|--------|
| TLM cascade | `src/sim/mod.rs::cadsd_ze_with_losses` | ✅ Working, viscothermal losses enabled |
| Strategy dispatch | `SimulationStrategy` enum + `DidgeridooSimulator::impedance` | ✅ UI wired |
| Waveguide freq-domain | `src/waveguide/mod.rs::WaveguideEngine` | ✅ Returns complex spectrum |
| Complex impedance approx | `SimulationStrategy::ComplexImpedance` | ✅ Returns complex spectrum |
| Geometry ops | `src/geo/mod.rs` — cone, cylinder, bubble, stretch, scale, volume | ✅ All working |
| Evolutionary optimizer | `src/evo/mod.rs` — Gaussian, PrimeSequence, SingleMutation + Average/PartSwap/PartAverage crossover, tournament selection, elite preservation | ✅ Working |
| Loss functions | `src/loss/mod.rs` — 10+ components, `CompositeTairuaLoss` with `PeakDetectionMode` | ✅ Modular |
| Peak detection | `src/sim/mod.rs::find_peaks`, `find_peaks_with_prominence`, `find_peaks_phase_based` | ✅ Three modes available |
| Persistence | `src/persistence/mod.rs` — settings, checkpoints | ✅ JSON save/load |
| GUI shell | `src/app.rs`, `src/bin/gui.rs` — Bevy + egui | ✅ Launchable |
| Strategy comparison | `run_comparison_simulation` + overlay plot | ✅ UI wired |
| Conservation budget | `CadsdState::budget_ops` slider | ✅ UI present |
| Radiation impedance | `src/sim/mod.rs::za` — Geipel unflanged-pipe approximation | ✅ Frequency-dependent, complex |
| Viscothermal losses | `src/sim/mod.rs::viscothermal_k_complex`, `cadsd_ze_with_losses` | ✅ Integrated in TLM path |
| AcousticConstants | `src/sim/mod.rs::AcousticConstants` | ✅ Temperature-dependent air properties |
| Bent-shape correction | `src/sim/mod.rs::bent_effective_length` | ✅ Analytical formula available |
| Tonehole models | `src/tonehole/mod.rs` — open/closed impedance | ✅ Implemented |
| File dialogs | `rfd` native pickers throughout GUI | ✅ Wired |
| Undo/redo | `geo_history` + `geo_history_index` | ✅ Fixed off-by-one |
| Loss caching | `Genome::clone_with_loss`, cached loss preserved in elite selection | ✅ Working |
| Feature flags | `nn-integration`, `fdtd-validator` in Cargo.toml | ✅ Defined |

### 2.2 What Exists But Is Broken / Incomplete

| Component | Issue | Location |
|-----------|-------|----------|
| Frequency grid | Linear by default; log grid uses step_ratio but not cents-based | `src/app.rs::compute_spectrum` |
| Optimizer wiring | Buttons log messages; no real parallel evaluation or progress callbacks | `src/app.rs` optimizer panel |
| 3-D bore preview | Not implemented | `src/app.rs` geometry panel |
| Time-domain synthesis | `WaveguideEngine` is frequency-domain only | `src/waveguide/mod.rs` |
| Differentiable TLM | No autodiff integration yet | `src/nn/mod.rs` placeholder only |
| Neural fitness predictor | Placeholder struct only; no training pipeline | `src/nn/mod.rs` |
| 3-D FDTD validator | Not started | `docs/RESEARCH.md` references only |

### 2.3 What Does Not Exist Yet

| Feature | Notes |
|---------|-------|
| Moist-air `AcousticConstants` with humidity/pressure | Current implementation covers temperature only |
| Differentiable TLM | No autodiff integration; `Value`-based gradient computation not prototyped |
| Neural fitness predictor training pipeline | Placeholder struct exists; no training loop |
| 3-D FDTD module | No acoustic FDTD; `fdtd-waveguide` is EM-only reference |
| Time-domain synthesis | `WaveguideEngine` is frequency-domain only |
| 3-D bore preview | Bevy gizmos wireframe mentioned in TODO but not implemented |
| Cents-based frequency grid everywhere | Log grid exists in `sim::grid` but not used universally |

---

## 3. UI Inventory & Gap Analysis

### 3.1 Existing Panels

| Panel | File | Elements | Wired? |
|-------|------|----------|--------|
| Simulation | `show_simulation_panel` | Freq grid sliders, compute button, peak button, export CSV, spectrum plot with tooltip | ✅ Mostly wired |
| Optimizer | `show_optimizer_panel` | Pop/gen sliders, loss toggles, mutation display, progress bar, run/pause/resume, save/load checkpoint, export genome | ⚠️ Buttons log only |
| Geometry | `show_geometry_panel` | Length/diameter/segment sliders, undo/redo, bubble/stretch dialogs, import/export JSON, bore profile plot | ✅ Mostly wired |
| Settings | `show_settings_panel` | Theme, log verbosity, default strategy/mutation, budget, export format, save/load config | ✅ Wired |
| Sidebar | `ui_system` | Strategy radios, mutation radio, budget slider, export geometry, compare strategies | ✅ Wired |

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
1. **Real optimizer loop** — run evolution in background thread / Bevy async task, publish progress to `generation_progress` and `best_loss` in real time

**P1 — Needed for usability:**
2. **Phase-aware spectrum toggle** — overlay unwrapped phase on spectrum plot
3. **Loss component weight sliders** — currently checkbox + inline slider; separate panel for fine-grained control
4. **3-D bore preview** — Bevy gizmos wireframe in geometry panel
5. **Strategy comparison in main plot** — overlay all three strategies on the simulation panel plot, not just a separate dialog

**P2 — Nice to have:**
6. **Cents-based log grid everywhere** — standardise frequency grid to cents-based spacing
7. **Tonehole UI** — add/remove toneholes from geometry panel

---

## 4. Development Plan

### 4.1 Phase A — Core Stability (Complete)
- [x] TLM cascade with complex arithmetic
- [x] Strategy dispatch (TLM / Waveguide / ComplexImpedance)
- [x] Geometry module (cone, bubble, stretch, volume)
- [x] Evolutionary optimizer (Gaussian + PrimeSequence)
- [x] Loss functions (10+ components)
- [x] Persistence (settings, checkpoints)
- [x] GUI shell (Bevy + egui, 4 panels)

### 4.2 Phase B — UI Completion (Complete)
**Owner:** Frontend / GUI  
**Blocked by:** None

| Task | File(s) | Effort | Notes |
|------|---------|--------|-------|
| Wire optimizer loop | `src/app.rs`, `src/evo/mod.rs` | Medium | Run `EvolutionaryOptimizer::evolve()` in `AsyncComputePool` or rayon thread; publish progress via callback |
| Add `rfd` file dialogs | `src/app.rs` | Small | Replace `text_edit_singleline` paths with `rfd::FileDialog` ✅ Done |
| Fix undo/redo off-by-one | `src/app.rs:901-911` | Tiny | Change `geo_history_index - 1` to `geo_history_index` ✅ Done |
| Add `prominence` to `find_peaks` | `src/sim/mod.rs`, `src/app.rs` | Small | Default `0.05` matching DidgeLab/scipy ✅ Done |
| Cents-based log grid | `src/sim/mod.rs::grid`, `src/app.rs::compute_spectrum` | Small | Use existing `grid::log_grid` ✅ Done |
| Loss caching on genome | `src/loss/mod.rs`, `src/evo/mod.rs` | Small | Add `cached_loss: Option<f64>` to `Genome` trait or `BaseGenome` ✅ Done |
| Export CSV with magnitude+phase | `src/app.rs::export_spectrum_csv` | Tiny | Add phase column ✅ Done |

### 4.3 Phase C — Physics Accuracy (Complete)
**Owner:** Simulation / Physics  
**Dependencies:** Phase B complete

| Task | File(s) | Effort | Notes |
|------|---------|--------|-------|
| Replace radiation impedance | `src/sim/mod.rs::za` | Medium | ✅ Geipel unflanged-pipe approximation |
| Add viscothermal `Tw`/`Zcw` | `src/sim/mod.rs::cadsd_ze_with_losses` | Medium | ✅ Integrated in TLM path via `viscothermal_k_complex` |
| Add `AcousticConstants` | `src/sim/mod.rs::AcousticConstants` | Small | ✅ Temperature-dependent air properties |
| Bent-shape effective-length correction | `src/sim/mod.rs::bent_effective_length` | Large | ✅ Analytical formula `dL_eff = ds * (1 - α·κ²·a²)` |

### 4.4 Phase D — Evolution Engine Enhancements (Complete)
**Owner:** Optimization  
**Dependencies:** Phase B

| Task | File(s) | Effort | Notes |
|------|---------|--------|-------|
| Add mutation operators | `src/evo/mod.rs` | Small | ✅ `SingleMutation`, `AverageCrossover`, `PartSwapCrossover`, `PartAverageCrossover` |
| Phase-based resonance finder | `src/sim/mod.rs` | Medium | ✅ `find_peaks_phase_based` using unwrapped phase derivative |
| Loss caching | `src/evo/mod.rs`, `src/loss/mod.rs` | Small | ✅ `clone_with_loss` preserves cached loss in elite selection |
| Prominence-based peak detection | `src/sim/mod.rs` | Small | ✅ `find_peaks_with_prominence` with configurable thresholds |
| Tonehole support | `src/tonehole/mod.rs` | Medium | ✅ Open/closed tonehole impedance models |

### 4.5 Phase E — Machine Learning Integration (In Progress)
**Owner:** ML / Research  
**Dependencies:** Phase C (accurate simulator) for training data

| Task | Crate | Effort | Notes |
|------|-------|--------|-------|
| Differentiable TLM prototype | `autodiff-rs` pattern or `dfdx` | Large | Wrap `Segment` params as differentiable `Value`s; backprop through cascade |
| Complex-valued NN primitives | Custom `src/nn/mod.rs` | Large | ✅ Placeholder exists; extract `Cf32` arithmetic + Wirtinger derivatives from `renplex` |
| Neural fitness predictor | `tch-rs` or `dfdx` | Large | MLP surrogate for top-5 resonance peaks; evaluate true TLM only on elite 5% |
| 3-D FDTD validator | New `src/fdtd/mod.rs` | Large | Port `fdtd-waveguide` Yee scheme to acoustics (pressure/velocity); batch validator |

**Feature flags:**
```toml
[features]
default = []
gui-bevy = ["bevy", "bevy_egui", "egui_plot", "rfd"]
nn-integration = []        # placeholder module exists; future: "tch-rs"
diff-tlm = ["dfdx"]        # differentiable TLM prototype
fdtd-validator = []        # 3-D acoustic FDTD (no external dep)
```

### 4.6 Phase F — Polish & Performance (Ongoing)
- [x] Remove duplicate `PrimeGenerator` ✅ Done
- [x] Standardize frequency grid to cents-based log spacing ✅ Done in `sim::grid`
- [ ] Add `cargo bench` benchmarks for simulator, loss, optimizer
- [ ] Expand unit tests (currently 32; target 50+)
- [ ] Add integration tests for full Geo → impedance → peaks → loss → optimizer pipeline
- [ ] Wire real optimizer loop with progress callbacks
- [ ] Add 3-D bore preview with bevy_gizmos

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
2. **12V constraint** — all new computation paths must respect `budget_ops` limit.
3. **Conservation budget** — `γ + η = C` where γ = simulation ops, η = evolution overhead.
4. **Feature flags** — ML integrations behind `nn-integration`, `diff-tlm`, `fdtd-validator`. Default build stays lightweight.
5. **Units** — geometry in mm internally; convert to m at `Segment::new`. Frequency in Hz. Impedance in Pa·s/m³ (characteristic).
6. **Complex arithmetic** — use `num_complex::Complex64` everywhere; do not introduce `Cf32` unless extracting from `renplex`.

---

## 9. Immediate Next Actions

1. **Wire real optimizer loop** — run `EvolutionaryOptimizer::evolve()` in background thread; add progress callbacks
2. **Add 3-D bore preview** — bevy_gizmos wireframe in geometry panel
3. **Prototype differentiable TLM** — wrap segment params as `autodiff-rs` `Value`s
4. **Train neural fitness predictor** — MLP surrogate for top-5 peaks
5. **Port FDTD validator** — `src/fdtd/mod.rs` Yee scheme for acoustics
6. **Standardise frequency grid** — cents-based log spacing everywhere
7. **Add integration tests** — full Geo → impedance → peaks → loss → optimizer pipeline
