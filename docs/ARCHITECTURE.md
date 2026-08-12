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
  sim/mod.rs          – TLM cascade, impedance strategies, peak detection
  waveguide/mod.rs    – Digital waveguide prototype (frequency-domain)
  evo/mod.rs          – Genome trait, BaseGenome, KigaliGenome, optimizer
  loss/mod.rs         – LossComponent trait + 10+ concrete losses
  persistence/mod.rs  – AppSettings, ProjectState, OptimizerCheckpoint
  app.rs              – CadsdState, Bevy systems, panel renderers
  bin/gui.rs          – Bevy app entrypoint
  visualization/      – (future) plotters/egui_plot helpers
  integration/        – (future) external tool bridges
  export/             – (future) WAV, CSV, geometry serialization
  nn/                 – (future) differentiable TLM, PINN, fitness predictor
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
    │   ├─ Tlm        → cadsd_ze() cascade (transfer matrices)
    │   ├─ Waveguide  → WaveguideEngine::transfer_function()
    │   └─ ComplexImpedance → viscothermal approx
    ▼
Vec<Complex<f64>> spectrum
    │
    ▼
find_peaks() → Vec<(idx, freq, mag)>
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
| TLM cascade | `src/sim/mod.rs::cadsd_ze` | ✅ Working, lossless |
| Strategy dispatch | `SimulationStrategy` enum + `DidgeridooSimulator::impedance` | ✅ UI wired |
| Waveguide freq-domain | `src/waveguide/mod.rs::WaveguideEngine` | ✅ Returns complex spectrum |
| Complex impedance approx | `SimulationStrategy::ComplexImpedance` | ✅ Returns complex spectrum |
| Geometry ops | `src/geo/mod.rs` — cone, cylinder, bubble, stretch, scale, volume | ✅ All working |
| Evolutionary optimizer | `src/evo/mod.rs` — Gaussian + PrimeSequence mutation, tournament selection, elite preservation | ✅ Working |
| Loss functions | `src/loss/mod.rs` — 10+ components, `CompositeTairuaLoss` | ✅ Modular |
| Peak detection | `src/sim/mod.rs::find_peaks` | ✅ Strict local maxima |
| Persistence | `src/persistence/mod.rs` — settings, checkpoints | ✅ JSON save/load |
| GUI shell | `src/app.rs`, `src/bin/gui.rs` — Bevy + egui | ✅ Launchable |
| Strategy comparison | `run_comparison_simulation` + overlay plot | ✅ UI wired |
| Conservation budget | `CadsdState::budget_ops` slider | ✅ UI present |

### 2.2 What Exists But Is Broken / Incomplete

| Component | Issue | Location |
|-----------|-------|----------|
| Radiation impedance | Spherical placeholder `rho*c/(2πr)`; needs Levine-Schwinger or Geipel model | `src/sim/mod.rs::za` |
| Viscothermal losses | `complex_impedance` uses rough boundary-layer delta; not validated; not in TLM path | `src/sim/mod.rs::complex_impedance` |
| Undo/redo | Off-by-one in redo branch | `src/app.rs:901-911` |
| Frequency grid | Linear by default; log grid uses step_ratio but not cents-based | `src/app.rs::compute_spectrum` |
| Export CSV | Logs to file with auto-generated name; no file picker | `src/app.rs::export_spectrum_csv` |
| Loss caching | No caching; every evaluation re-simulates | `src/loss/mod.rs::CompositeTairuaLoss` |
| Peak detection robustness | No `prominence` parameter; breaks on mode switching | `src/sim/mod.rs::find_peaks` |
| Duplicate PrimeGenerator | Exists in both `src/evo/mod.rs` and `src/waveguide/mod.rs` | Two files |
| Optimizer wiring | Buttons log messages; no real parallel evaluation or progress callbacks | `src/app.rs` optimizer panel |

### 2.3 What Does Not Exist Yet

| Feature | Notes |
|---------|-------|
| Moist-air `AcousticConstants` | DidgeLab computes density/viscosity/speed from temp/humidity/pressure |
| Missing mutation operators | DidgeLab has 7; we have 2 (Gaussian, PrimeSequence) |
| Differentiable TLM | No autodiff integration; `Value`-based gradient computation not prototyped |
| Neural fitness predictor | No `nn-integration` feature flag code yet |
| 3-D FDTD module | No acoustic FDTD; `fdtd-waveguide` is EM-only reference |
| Phase-based resonance finder | Ernoult et al. 2020 alternative to `find_peaks` |
| Tonehole support | No side-hole geometry or scattering junction |
| Time-domain synthesis | `WaveguideEngine` is frequency-domain only |
| 3-D bore preview | Bevy gizmos wireframe mentioned in TODO but not implemented |

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
2. **Loss caching** — cache `shape.loss` on genome to avoid re-simulation of unchanged individuals
3. **Radiation impedance fix** — invisible to user but affects all spectrum accuracy

**P1 — Needed for usability:**
4. **File dialogs** — replace all text-entry path fields with `rfd` native file pickers (already in `Cargo.toml`)
5. **Undo/redo fix** — redo branch reads `geo_history_index - 1` instead of `geo_history_index`
6. **Prominence in peak detection** — add `prominence` parameter to `find_peaks`; expose in UI
7. **Cents-based log grid** — replace ad-hoc `step_ratio` log grid with `grid::log_grid(min_cents, max_cents, step_cents)`

**P2 — Nice to have:**
8. **Phase-aware spectrum toggle** — overlay unwrapped phase on spectrum plot
9. **Loss component weight sliders** — currently checkbox + inline slider; separate panel for fine-grained control
10. **3-D bore preview** — Bevy gizmos wireframe in geometry panel
11. **Strategy comparison in main plot** — overlay all three strategies on the simulation panel plot, not just a separate dialog

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

### 4.2 Phase B — UI Completion (In Progress)
**Owner:** Frontend / GUI  
**Blocked by:** None

| Task | File(s) | Effort | Notes |
|------|---------|--------|-------|
| Wire optimizer loop | `src/app.rs`, `src/evo/mod.rs` | Medium | Run `EvolutionaryOptimizer::evolve()` in `AsyncComputePool` or rayon thread; publish progress via callback |
| Add `rfd` file dialogs | `src/app.rs` | Small | Replace `text_edit_singleline` paths with `rfd::FileDialog` |
| Fix undo/redo off-by-one | `src/app.rs:901-911` | Tiny | Change `geo_history_index - 1` to `geo_history_index` |
| Add `prominence` to `find_peaks` | `src/sim/mod.rs`, `src/app.rs` | Small | Default `0.05` matching DidgeLab/scipy |
| Cents-based log grid | `src/sim/mod.rs::grid`, `src/app.rs::compute_spectrum` | Small | Use existing `grid::log_grid` |
| Loss caching on genome | `src/loss/mod.rs`, `src/evo/mod.rs` | Small | Add `cached_loss: Option<f64>` to `Genome` trait or `BaseGenome` |
| Export CSV with magnitude+phase | `src/app.rs::export_spectrum_csv` | Tiny | Add phase column |

### 4.3 Phase C — Physics Accuracy (Next)
**Owner:** Simulation / Physics  
**Dependencies:** Phase B complete

| Task | File(s) | Effort | Notes |
|------|---------|--------|-------|
| Replace radiation impedance | `src/sim/mod.rs::za` | Medium | Implement Levine-Schwinger IIR or Geipel approx from DidgeLab `tlm_python.py::Za` |
| Add viscothermal `Tw`/`Zcw` | `src/sim/mod.rs::cadsd_ze` | Medium | Align with DidgeLab's `rvw`, `Tw`, `Zcw` formulas |
| Add `AcousticConstants` | `src/sim/mod.rs` or new `src/acoustics/mod.rs` | Small | Temp/humidity/pressure → density, viscosity, speed of sound |
| Bent-shape effective-length correction | `src/geo/mod.rs`, `src/sim/mod.rs` | Large | Add `Centreline` struct, curvature integral `dL_eff = ds * (1 - α·κ²·a²)` |

### 4.4 Phase D — Evolution Engine Enhancements (Parallel with C)
**Owner:** Optimization  
**Dependencies:** Phase B

| Task | File(s) | Effort | Notes |
|------|---------|--------|-------|
| Add mutation operators | `src/evo/mod.rs` | Small | `SingleMutation`, `AverageCrossover`, `PartSwapCrossover`, `PartAverageCrossover` from DidgeLab |
| Phase-based resonance finder | `src/sim/mod.rs` | Medium | Unwrapped phase of `R(f) = (Z_in - Z_c)/(Z_in + Z_c)`; peaks at `angle = -2π(n-1)` |
| Loss caching | `src/evo/mod.rs`, `src/loss/mod.rs` | Small | Cache on `BaseGenome.loss` or in optimizer |

### 4.5 Phase E — Machine Learning Integration (Later)
**Owner:** ML / Research  
**Dependencies:** Phase C (accurate simulator) for training data

| Task | Crate | Effort | Notes |
|------|-------|--------|-------|
| Differentiable TLM prototype | `autodiff-rs` pattern or `dfdx` | Large | Wrap `Segment` params as differentiable `Value`s; backprop through cascade |
| Complex-valued NN primitives | Custom `src/nn/mod.rs` | Large | Extract `Cf32` arithmetic + Wirtinger derivatives from `renplex`; do not depend on archived crate |
| Neural fitness predictor | `tch-rs` or `dfdx` | Large | MLP surrogate for top-5 resonance peaks; evaluate true TLM only on elite 5% |
| 3-D FDTD validator | New `src/fdtd/mod.rs` | Large | Port `fdtd-waveguide` Yee scheme to acoustics (pressure/velocity); batch validator |

**Feature flags:**
```toml
[features]
default = []
gui-bevy = ["bevy", "bevy_egui", "egui_plot", "rfd"]
nn-integration = ["tch-rs"]        # production GPU training
diff-tlm = ["dfdx"]                # differentiable TLM prototype
fdtd-validator = []                # 3-D acoustic FDTD (no external dep)
```

### 4.6 Phase F — Polish & Performance (Ongoing)
- Remove duplicate `PrimeGenerator`
- Standardize frequency grid everywhere to cents-based
- Add `cargo bench` benchmarks for simulator, loss, optimizer
- Expand unit tests (currently ~17; target 50+)
- Add integration tests for full Geo → impedance → peaks → loss → optimizer pipeline

---

## 5. UI Requirements (Detailed)

### 5.1 Simulation Panel

**Current:**
- Frequency grid config (min/max/points, log toggle)
- Compute spectrum button
- Find peaks button
- Export CSV button
- Spectrum plot with hover tooltip
- Fundamental frequency display

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
- Save / Load checkpoint
- Export best genome

**Missing:**
- **Real async execution** — run optimizer in background; update `best_loss`, `current_generation`, `generation_progress` live
- **Loss curve plot** — best loss per generation (time series)
- **Resume from checkpoint** — file picker + state restoration
- **Mutation operator selector** — dropdown or radio for all 7 operators (currently only Gaussian / PrimeSequence in sidebar)
- **Convergence stop condition** — early stopping if best loss plateaus
- **Population diversity metric** — e.g., average pairwise genome distance

### 5.3 Geometry Panel

**Current:**
- Length / top-diameter / bottom-diameter / segment sliders
- Undo / redo buttons (redo broken)
- Add bubble dialog
- Stretch dialog
- Import / export JSON
- Bore profile preview (2D cross-section)

**Missing:**
- **3-D wireframe preview** — Bevy gizmos showing bore as rotated cylinder/conical frustum
- **Segment editor** — table view of individual `[x, d]` points with add/remove/reorder
- **Parametric shape presets** — Kigali, Mbeya, cone, cylinder one-click generators
- **Curvature editor** — for bent-shape correction (Phase C)
- **Volume / surface-area readout** — already have volume; add surface area

### 5.4 Settings Panel

**Current:**
- Theme (light/dark)
- Log verbosity (0-4)
- Default strategy / mutation
- Budget ops slider
- Export format radio
- Save / load config
- Reset to defaults

**Missing:**
- **Default mutation operator** — extend beyond Gaussian / PrimeSequence
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

1. **Fix undo/redo** — one-line fix in `src/app.rs:904`
2. **Wire optimizer loop** — run `EvolutionaryOptimizer::evolve()` in `AsyncComputePool`; add progress callbacks
3. **Add `rfd` dialogs** — replace 8 text-entry path fields with native file pickers
4. **Replace radiation impedance** — implement Geipel approximation from DidgeLab `tlm_python.py::Za`
5. **Add prominence to `find_peaks`** — `src/sim/mod.rs` + expose in UI
6. **Remove duplicate `PrimeGenerator`** — keep the one in `src/evo/mod.rs`, delete `src/waveguide/mod.rs` copy
7. **Add missing mutation operators** — `SingleMutation`, `AverageCrossover` at minimum
