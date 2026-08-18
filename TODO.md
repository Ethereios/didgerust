# DidgeRust Future Goals

This file documents important features and improvements that are **not yet fully implemented**.
Status levels: 🔄 partial, ❌ missing, ⚠️ needs improvement.

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

### GUI / UX

- **GUI tonehole editor** — ⚠️ Sliders work; no drag-and-drop on bore preview.
- **3-D bore preview** — ⚠️ Wireframe exists in `src/app.rs::draw_bore_gizmos`. Add camera controls, zoom, rotation.
- **Optimizer loop** — ⚠️ Buttons log only; no real async execution with progress callbacks.
- **Frequency grid** — ⚠️ Linear by default; log grid not cents-based everywhere.

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

- **File I/O tests for persistence** — ❌ No tests for `AppSettings`, `ProjectState`, `OptimizerCheckpoint` save/load with real files.
- **CLI command tests** — ❌ No automated tests for `src/bin/cli.rs` commands.

## Completed (fully working)

- **TLM cascade** — `src/sim/mod.rs::cadsd_ze_with_losses` with full viscothermal losses.
- **Evolutionary optimizer** — `src/evo/mod.rs` with multiple mutation/crossover strategies.
- **Loss functions** — `src/loss/mod.rs` with 10+ components.
- **Peak detection** — Three modes: local maxima, prominence, phase-based.
- **CLI for experimental features** — `src/bin/cli.rs` exposes all experimental modules to non-Rust users.
- **Persistence** — JSON save/load for settings, checkpoints, project state.
- **Geometry ops** — cone, cylinder, bubble, stretch, scale, volume, Kigali, Mbeya.
