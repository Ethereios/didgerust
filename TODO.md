# DidgeRust Future Goals

This file documents important features and improvements that are **not yet fully implemented**.
Status levels: ✅ working, 🔄 partial, ⚠️ needs improvement, ❌ missing, 🧪 experimental.

## Partial Implementations (concrete improvements needed)

### Acoustics / Simulation

- **Radiation impedance** — ⚠️ Geipel approximation in `src/sim/mod.rs::za`. Replace with Levine-Schwinger IIR; validate against published unflanged-pipe data.
- **Viscothermal losses** — ⚠️ Full Tw/Zcw system in `cadsd_ze_with_losses`. Validate against Scavone 1997 data in 100 Hz–2 kHz range.
- **Bent-shape correction** — 🔄 `bent_effective_length()` exists with tests. Wire into `Segment::effective_length`; integrate into optimizer loss; show in GUI bore preview.
- **Tonehole models** — ⚠️ Open/closed impedance in `src/tonehole/mod.rs`. Add three-port scattering junction (Scavone & Smith 1997) for chromatic design.
- **ComplexImpedance strategy** — ⚠️ Basic implementation exists. Add full viscothermal model; validate against TLM for non-cylindrical geometries.
- **Differentiable TLM** — ⚠️ Analytical gradients + Adam in `src/diff_tlm.rs`. Implement real backprop through cascade using Wirtinger calculus; test against numerical gradients.
- **FDTD validator** — 🔄 3-D Yee grid with PML in `src/fdtd/`. Increase grid resolution; validate against analytical cylinder; add bent-geometry study.
- **Prime-conv ML** — 🔄 `PrimeConvBlock` forward pass in `src/prime_conv/`. Build training pipeline; generate dataset from TLM; train surrogate for top-5 peaks.
- **DWM prototypes** — 🔄 2-D/3-D mesh in `src/dwm/`. Integrate with `DidgeridooSimulator` as alternative strategy; validate against TLM.

### GUI / UX (Makepad)

- **GUI tonehole editor** — ⚠️ Sliders work; no drag-and-drop on bore preview.
- **3-D bore preview** — ⚠️ Wireframe exists in `src/app.rs::draw_bore_gizmos`. Add camera controls, zoom, rotation.
- **Optimizer loop** — ✅ Wired: cancellable background thread with Nuevolution, progress callbacks, Start/Stop buttons and status labels
- **Loss breakdown preview** — ✅ Implemented: TairuaLoss weights (fundamental, harmonics, peaks) exposed via sliders, BarChart showing 4 components, target freq input
- **Cross-section view** — ✅ LineChart rendering diameter vs position from current Geo segments
- **Export functions** — ✅ CSV and JSON export working via rfd file dialog

## Missing Implementations

### Neural / Differentiable

- **Neural fitness predictor** — ❌ Placeholder struct only in `src/nn/mod.rs`. No MLP, no training loop, no dataset.
- **Time-domain synthesis** — ❌ Frequency-domain only. No sample-by-sample waveguide loop; no cpal audio output from simulation.
- **PINN surrogate for bent geometries** — ❌ No physics-informed neural network for bent-bore correction.

### Audio

- **Real-time audio backend** — ❌ No time-domain synthesis → no cpal audio output from actual simulation.
- **WAV export from waveguide** — ❌ Current export uses simple sine wave; not from time-domain simulation.

### Geometry

- **RBF constraint systems** — ❌ No radial-basis-function constraints for smooth geometry deformation during optimization.

### Testing

- **Geo bubble shape/simulation validation** — ✅ Added 4 tests in `rust-cadsd-accurate/src/geo/mod.rs`: `test_make_bubble_shape_continuity`, `test_make_bubble_volume_increase`, `test_make_bubble_simulation_valid`, `test_make_bubble_numerical_stability`. Confirmed `Geo::make_bubble` sinusoidal profile matches `KigaliGenome::make_bubble` in `src/evo/mod.rs`.
- **Slow test suite** — ✅ Fixed infinite loop in `get_log_simulation_frequencies_with_points` (sim/mod.rs:266) that caused tests to hang. Refactored TairuaLoss, FundamentalFrequencyLoss, and MultiObjectiveLoss to use single acoustic simulation per call instead of 3 redundant simulations. All 32 lib tests now pass in ~11s (was 60s+ timeout). Full frequency grid preserved — no accuracy compromise.

## Completed (fully working)

- **TLM cascade** — `src/sim/mod.rs::cadsd_ze_with_losses` with full viscothermal losses.
- **Evolutionary optimizer** — `src/evo/mod.rs` with multiple mutation/crossover strategies.
- **Loss functions** — `src/loss/mod.rs` with 10+ components.
- **Peak detection** — Three modes: local maxima, prominence, phase-based.
- **CLI for experimental features** — `src/bin/cli.rs` exposes all experimental modules to non-Rust users.
- **Persistence** — JSON save/load for settings, checkpoints, project state.
- **Geometry ops** — cone, cylinder, bubble, stretch, scale, volume, Kigali, Mbeya.
- **Geo::make_bubble consistency** — Fixed from triangular to sinusoidal profile (10 sample points) matching `KigaliGenome::make_bubble`. Fixed unsorted geometry bug from bubble insertion. Added shape continuity, volume, simulation, and numerical stability tests.

### Makepad GUI (working)

- **Geometry controls** — Length, top/bell diameter, segments sliders
- **Bore style dropdown** — ✅ Fixed: Cone, Kigali, Mbeya all available in dropdown
- **Bore curve slider** — -2.0 to 2.0 (affects Kigali/Mbeya power parameter)
- **3D viewport** — Real-time bore geometry with orbit/zoom controls (XrCamera)
- **Run simulation button** — Background thread execution
- **Results display** — Fundamental frequency, resonance count
- **Impedance spectrum chart** — LineChart widget rendering impedance spectrum
- **Geometry summary preview** — Length, bell, volume, taper ratio, segments, max diameter
- **Resonance analysis preview** — Top 10 peaks with frequency and impedance values
- **Profile selection in simulation thread** — Cone/Kigali/Mbeya properly routed to `Geo::make_*` builders
- **Optimizer panel** — Start/Stop/Progress buttons, async Nuevolution thread, TairuaLoss weights and target frequency
- **Loss breakdown preview** — Tairua loss components exposed via sliders and BarChart
- **Cross-section view** — Diameter-vs-position LineChart from current Geo segments
- **Export functions** — CSV and JSON export via rfd file dialog

## Experimental Features 🧪

The following features are implemented but not fully validated. Marked as experimental in the GUI UI with "(Experimental)" labels:

- **Bubble insertion** — Insert bulges at positions in bore; backend available but acoustic impact not fully tested
- **Finger holes** — UI controls wired (toggle + count slider), but backend equations incomplete — no acoustic simulation resolution yet
- **Mouthpiece** — UI controls present; backend equations incomplete — no acoustic simulation resolution yet
- **AI/ML integration** — Neural fitness predictor and time-domain synthesis not yet implemented
- **Optimization panel** — UI fully wired with Start/Stop/Progress and async thread using real `acoustical_simulation` + `Nuevolution` backend

---
