# Accuracy / Parity Plan

This document defines what “parity” means between the two implementations in this repository:

- **Wrapper crate**: `didgerust/` (modules: `src/sim`, `src/geo`, `src/loss`, `src/visualization`)
- **Accurate crate**: `didgerust/rust-cadsd-accurate/` (`cadsd-accurate`, modules: `src/sim`, `src/geo`, `src/conv`, `src/analysis`, `src/loss`, `src/evo`, `src/integration`, plus GUI entrypoints)

Parity work is currently dominated by:
- **peak semantics** (how “resonances” are detected),
- **frequency grid semantics** (log vs linear + resolution),
- **impedance scaling** (raw magnitude vs normalized).

---

## 1) Simulation parity

### 1.1 API-level parity
**Goal:** both crates accept equivalent geometries and align outputs with the passed-in frequency vectors.

- Accurate crate:
  - `acoustical_simulation(geo, frequencies, method) -> Vec<f64>` returns impedance magnitudes aligned 1:1 with `frequencies`.
  - `get_log_simulation_frequencies() -> Vec<f64>` provides the default log grid.
- Wrapper crate:
  - `DidgeridooSimulator::impedance(freqs) -> Vec<Complex<f64>>` (downstream uses `norm()`).
  - `find_resonance_peaks()` constructs a **linear** grid based on `SimulationParams::default()`.

**Acceptance criteria**
- Both produce impedance arrays aligned to the input frequency order.
- Geometry conversion (mm segments → internal meter segments) is consistent.

### 1.2 Physics parity (based on inspected code)
**Transfer matrices**
- Accurate crate (`rust-cadsd-accurate/src/sim/mod.rs`): explicit conical vs cylindrical transfer logic inside `ap_segment()`.
- Wrapper crate (`didgerust/src/sim/mod.rs`): simpler tube propagation model plus simplified radiation impedance.

**Viscothermal losses**
- Accurate crate: `ap_segment()` includes viscothermal-style complex terms (`rvw`, `tw`, `zcw`) and uses them in the segment transfer matrices.
- Wrapper crate: lossless propagation constant stub (`k = ω/c`) and reduced physics.

**Radiation impedance**
- Accurate crate: `za(w, segments)` uses a Geipel/derived formulation combining `zcw` and geometry-dependent terms.
- Wrapper crate: `za()` is a placeholder-like spherical radiation model.

**Acceptance criteria**
- No requirement for numeric equality initially.
- Must maintain:
  - finite impedance values (no NaN/Inf),
  - stable resonance peak finding in typical didgeridoo geometries,
  - correct behavior across conical vs cylindrical geometry inputs.

### 1.3 Backend parity (`simulation_method`)
- Accurate crate accepts `"tlm_python"` / `"tlm_cython"` but currently both appear routed to the same Rust implementation in the inspected file.
- Wrapper crate’s sim layer (we inspected) doesn’t heavily rely on backend selection.

**Acceptance criteria**
- Backend strings must be accepted.
- Outputs should be identical (or documented as non-differentiated).

---

## 2) Analysis / resonance extraction parity

### 2.1 Peak picking semantics (confirmed)
Both crates (in the inspected modules) use strict local-maxima detection:

> `imp[i] > imp[i-1] && imp[i] > imp[i+1]`

- Accurate crate:
  - `analysis::get_notes()` uses local maxima.
  - `sim::compute_ground_spektrum()` and `sim::get_fundamental()` use local maxima.
  - `loss::TairuaLoss::find_peaks()` uses local maxima (and also “top N by sorting” in `DidgeLabLoss`).
- Wrapper crate:
  - `sim::find_peaks()` uses local maxima.
  - Wrapper `loss::CompositeTairuaLoss` depends on wrapper peak extraction.

**Acceptance criteria**
- “Resonance peaks” are defined consistently as local maxima of impedance magnitude.
- Peak ordering must be stable and deterministic within each crate.

### 2.2 Ordering & impedance normalization differences (risk)
- Wrapper loss normalizes peak impedances by `max_imp`.
- Accurate `TairuaLoss` (as inspected) uses harmonic deviations and does not show equivalent normalization in the harmonic-loss section.

**Acceptance criteria**
- Do not compare absolute loss values across crates until impedance scaling and peak extraction are aligned.

---

## 3) Loss parity

### 3.1 Accurate crate: `TairuaLoss` (inspected)
- Uses log frequencies.
- Runs simulation (twice).
- Peak extraction: internal `find_peaks()` uses local maxima.
- Fundamental loss:
  - `get_fundamental(…, min_peak_f=20.0)` and quadratic penalty with `frequency_tolerance=5.0` (Hz).
- Harmonic loss:
  - uses base frequency = first peak returned by peak finder
  - expected harmonic = `base_freq * (i + 1)`
  - deviation = relative abs(freq-expected)/expected

### 3.2 Wrapper crate: `CompositeTairuaLoss` (inspected)
- Builds a frequency grid derived from `max_error`.
- Computes impedance and uses wrapper peaks (`find_peaks` in wrapper sim).
- Normalizes peak impedances by max impedance before evaluating components.

**Acceptance criteria**
- Align peak list selection and impedance scaling before validating optimizer improvements or loss comparisons.

---

## 4) Visualization / export parity

### 4.1 Wrapper crate visualization
- `didgerust/src/visualization/mod.rs` writes stub “mock png content”.
- `report.txt` is generated from computed peaks and geometry metrics.

**Acceptance criteria**
- Output files exist with correct filenames and report structure.
- Numeric fidelity depends on correct underlying simulation/peaks.

### 4.2 Accurate crate GUI
- Bevy+egui exists and routes UI actions to export/settings panels.
- Geometry mapping must be single-sourced (avoid duplicated helpers).

**Acceptance criteria**
- Export outputs match GUI state-derived geometry.

---

## Practical parity targets (what to fix/align first)
1) Peak extraction consistency (definition + ordering).
2) Frequency grid consistency (log vs linear + resolution).
3) Impedance magnitude scaling (raw vs normalized).
4) Geometry mapping consistency between GUI state and `Geo` segment lists.
