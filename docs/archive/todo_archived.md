# didgerust - Implementation Progress

## Phase A: ✅ Core Implementation Fixes (COMPLETE)

## Phase B: ✅ UI Feature Completion (COMPLETE)

### B0 — Persistence Module (Foundation)
- [x] Implement real JSON save/load for `AppSettings`
- [x] Implement real JSON save/load for `ProjectState`
- [x] Wire optimizer checkpoint save/load to actual files

### B1 — Simulation Panel
- [x] Wire "Export CSV" button to file dialog + `DataExporter`
- [x] Implement "Compare Strategies" overlay with multi-line plot (TLM/WG/CI)
- [x] Add real-time spectrum tooltips on hover

### B2 — Optimizer Panel
- [x] Add loss function component toggles (checkboxes)
- [x] Wire "Resume from Checkpoint" button to file dialog
- [x] Wire "Export Best Genome" button to file dialog
- [x] Wire real generation progress tracking

### B3 — Geometry Panel
- [x] Implement undo/redo stack (history buffer)
- [x] Wire "Import Geometry" / "Export Geometry" to file dialogs
- [x] Implement "Add Bubble" dialog
- [x] Implement "Stretch Geometry" dialog
- [ ] Add 3D bore preview (bevy_gizmos wireframe)

### B4 — Settings Panel
- [x] Add theme selection (light/dark toggle)
- [x] Add logging verbosity control
- [x] Add default strategy persistence
- [x] Wire "Save/Load Configuration" to file dialogs

## Phase C: 🟠 Persistence & State Management (later)
## Phase D: 🔵 Advanced Integrations (in progress)

### D1 — Physics Model Improvements
- [x] Replace radiation impedance placeholder (`src/sim/mod.rs::za`) with Levine-Schwinger IIR or Geipel approximation from DidgeLab
- [x] Implement full viscothermal loss model (`Tw`, `Zcw`) in `cadsd_ze` to match DidgeLab's `tlm_python.py`
- [x] Add `AcousticConstants` struct with temperature/humidity/pressure-dependent moist-air properties
- [x] Implement bent-shape effective-length correction (`dL_eff = ds * (1 - α·κ²·a²)`) from DidgeLab

### D2 — Evolution Engine Enhancements
- [x] Add missing mutation operators: `SingleMutation`, `AverageCrossover`, `PartSwapCrossover`, `PartAverageCrossover` (from DidgeLab `operators.py`)
- [x] Implement loss result caching on genome objects to avoid redundant simulation
- [x] Add `prominence` parameter to `find_peaks` for robust peak detection
- [x] Implement phase-based resonance finder (Ernoult et al. 2020) as alternative to strict local maxima
- [ ] Add `prominence` parameter to `find_peaks` for robust peak detection
- [ ] Implement phase-based resonance finder (Ernoult et al. 2020) as alternative to strict local maxima

### D3 — Machine Learning Integration (behind `nn-integration` feature flag)
- [ ] Prototype differentiable TLM using scalar `Value` engine (`autodiff-rs` pattern) for gradient-based optimisation
- [ ] Add `dfdx` as dev-dependency for compile-time graph optimisation of complex transfer matrices
- [ ] Add `tch-rs` behind `nn-integration` feature flag for production GPU training
- [ ] Extract complex-valued NN primitives from `renplex` (Cf32 arithmetic, Wirtinger derivatives) into `src/nn/mod.rs`
- [ ] Implement neural fitness predictor (MLP surrogate for top-5 resonance peaks) to speed up evolution
- [ ] Port `fdtd-waveguide` Yee scheme to acoustics (`src/fdtd/mod.rs`) for 3-D bent-geometry validation

### D4 — Code Quality
- [x] Fix undo/redo off-by-one in geometry panel redo logic
- [x] Remove duplicate `PrimeGenerator` in `src/waveguide/mod.rs` (already exists in `src/evo/mod.rs`)
- [x] Standardise frequency grid to cents-based log spacing for tuning accuracy

## New Modules Added
- [x] `src/tonehole/mod.rs` — Tonehole modeling (open/closed impedance)
- [x] `PeakDetectionMode` in `src/loss/mod.rs` — configurable peak detection for loss evaluation

## Phase 5: Performance plan
- [x] Identify hotspots (segments conversion, impedance recompute, peak scanning)
- [x] Propose caching + batching strategy
- [x] Plan profiling steps

Deliverables:
- [x] `PERF_REPORT.md`

## Tracking
- [x] Mark tasks completed after each phase
