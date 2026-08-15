# didgerust - Design Notes (module call graphs & invariants)

This document consolidates what was learned from the repository code and docs.
It focuses on invariants of:
- geometry representation (mm-based segments),
- frequency grid generation (linear vs log),
- the call graph across simulation → peak picking/analysis → loss,
- and where current implementations are simplified/stubbed.

---

## 1) Two-crate architecture

### A) Wrapper crate: `didgerust/` (top-level)
Path: `didgerust/src/*`

Main subsystems (from code we read):
- `geo` (geometry operations in mm)
- `sim` (impedance computation + peak picking)
- `loss` (loss components + composite loss)
- `visualization` (analysis report generation; plotting is stubbed)

### B) Accurate crate: `didgerust/rust-cadsd-accurate/` (`cadsd-accurate`)
Path: `didgerust/rust-cadsd-accurate/src/*`

Main subsystems (from code we read):
- `geo/mod.rs` (Geo invariants + generators)
- `sim/mod.rs` (acoustical_simulation; transfer matrices; frequency grid)
- `analysis/mod.rs` (get_notes; placeholder plotting)
- `loss/mod.rs` (TairuaLoss + additional losses)
- `ui/*` and `app.rs` (Bevy GUI; includes wiring to `crate::ui::*`)

---

## 2) Geometry invariants (mm segment model)

### 2.1 Accurate crate `Geo` (`rust-cadsd-accurate/src/geo/mod.rs`)
- Geometry is `Vec<[f64; 2]>` where each element is:
  - `[x_mm, d_mm]`
  - `x` = distance from mouthpiece in mm
  - `d` = bore diameter in mm
- `Geo::new()` removes adjacent duplicate x values (zero-length segment cleanup).
- `make_cone(length, d1, d2, n_segments)` builds a conical bore with `n_segments` subdivisions.
- `stretch(factor)` scales **x only**; `scale(factor)` scales **x and diameter**.
- `make_bubble(pos, width, height)` inserts a bulge by adding 3 points:
  - left edge at `pos - width/2`
  - peak at `pos`
  - right edge at `pos + width/2`
- `diameter_at_x(x)` interpolates diameter along the segment containing `x`.
- `compute_volume()` uses a trapezoidal-rule approximation of frustum volume:
  - sums: `length * π * (d1^2 + d1*d2 + d2^2) / 12`

### 2.2 Wrapper crate geometry
- Wrapper `Geo(pub AccurateGeo)` wraps the accurate crate's `Geo` with `Deref`/`DerefMut`.
- Used consistently as a segment list in mm (passed into simulator functions that convert mm→m internally).
- `add_bubble(pos, width, height)` delegates to `self.0.make_bubble`.

---

## 3) Frequency grid invariants

### 3.1 Accurate crate (log frequency grid)
In `rust-cadsd-accurate/src/sim/mod.rs`:
- `get_log_simulation_frequencies_with_params(fmin, fmax, grid_size)`
- Uses "cents per 1200" style stepping:
  - `stepsize = grid_size / 1200.0`
  - builds `0..1200` frequency points per "octave" block
- Default uses `get_log_simulation_frequencies()` = (20..2000) with `grid_size=1.0`.

### 3.2 Wrapper crate (linear frequency grid for resonance search)
In `didgerust/src/sim/mod.rs`:
- `find_resonance_peaks()` uses `SimulationParams::default()`:
  - `freq_range=(20,2000)` and `points=512`
  - and builds an inclusive linear grid.
- The impedance evaluation itself accepts any `freqs` list, but resonance extraction uses the above linear grid.

---

## 4) Simulation call graph

### 4.1 Accurate crate simulation pipeline (`rust-cadsd-accurate/src/sim/mod.rs`)
Public entry:
- `acoustical_simulation(geo, frequencies, simulation_method) -> Result<Vec<f64>>`

Internal:
1. Convert `Geo` to internal segment model (mm→m):
   - `create_segments_from_geo(&geo.geo)`
2. For each frequency:
   - compute `cadsd_ze(&segments, freq)` where:
     - builds transfer matrix cascade:
       - `ap(w, segments)` → multiplies per-segment `ap_segment(w, segment)`
     - includes viscothermal-style complex terms:
       - `rvw`, complex `tw`, complex `zcw`
     - computes radiation impedance via `za(w, segments)`:
       - uses `zcw` and terms dependent on last segment `d1`, `l`, `a01`, `r0`
     - returns input impedance magnitude:
       - `(numerator/denominator).norm()`

Backend note:
- `tlm_python` and `tlm_cython` both route to the same Rust implementation in the code we read.

### 4.2 Wrapper crate simulation pipeline (`didgerust/src/sim/mod.rs`)
Public entry (wrapper simulator):
- `DidgeridooSimulator::impedance(&self, freqs) -> Vec<Complex<f64>>`

Internal (from code we read):
- `create_segments_from_geo()` converts mm→m
- `cadsd_ze(segments, freq_hz)` computes complex impedance with a simple tube model:
  - cascades per-segment transfer matrices using lossless propagation:
    - `k = omega / C`
  - applies a simplified radiation impedance:
    - wrapper `za()` is essentially a placeholder

Resonance extraction in wrapper:
- `find_resonance_peaks()`:
  - builds a linear grid from `SimulationParams::default()`
  - finds local maxima only:
    - `mag[i] > mag[i-1] && mag[i] > mag[i+1]`

---

## 5) Analysis / peak semantics call graph

### 5.1 Accurate crate `get_notes()` (`rust-cadsd-accurate/src/analysis/mod.rs`)
- `get_notes(frequencies, impedances) -> Vec<(f64,f64)>`
- Peak semantics:
  - strictly local maxima in impedance:
  - `imp[i] > imp[i-1] && imp[i] > imp[i+1]`

### 5.2 Accurate crate resonance in `sim/mod.rs`
- `compute_ground_spektrum()` also uses local maxima in impedance.
- `get_fundamental()` uses local maxima and returns first peak above `min_peak_f`.

### 5.3 Wrapper analysis/report uses local maxima
- Wrapper `DidgeridooSimulator::find_resonance_peaks()` uses:
  - local maxima on impedance magnitude in a linear grid.
- Wrapper `create_analysis_report()` writes:
  - `geometry.txt` (stub)
  - `spectrum.txt` (stub)
  - `report.txt` based on wrapper peaks.

---

## 6) Loss call graph

### 6.1 Wrapper loss (`didgerust/src/loss/mod.rs`)
- `LossComponent` trait calculates component loss using:
  - `peak_freqs_log`, `peak_impedances`, and full spectrum arrays.
- `CompositeTairuaLoss`:
  1. converts `genome -> geo`
  2. evaluates impedance spectrum on a derived frequency grid
  3. extracts peaks via wrapper `simulator.peaks(&freqs)` (local maxima)
  4. normalizes peak impedances by `max_imp`
  5. calls each component's `calculate()` and sums losses

### 6.2 Accurate loss (`rust-cadsd-accurate/src/loss/mod.rs`)
Primary for GUI: `TairuaLoss`
- `compute_loss(geo)`:
  1. uses log frequencies (`get_log_simulation_frequencies()`)
  2. runs simulation (tlm_python)
  3. optional fundamental loss:
     - calls `get_fundamental(geo, "tlm_python", 20.0)`
     - penalizes (fundamental-target)/tolerance squared
  4. recomputes impedance (second simulation)
  5. extracts peaks via internal `find_peaks()` = local maxima
  6. harmonic alignment loss:
     - base_freq = first peak frequency returned by peak ordering
     - expected harmonic = base_freq*(i+1)
     - sums relative deviation

Important: `TairuaLoss` in the inspected code does **not** show peak impedance normalization matching the wrapper crate.

---

## 7) UI/app wiring notes (accurate crate)

### 7.1 Accurate GUI entrypoints (`rust-cadsd-accurate/src/app.rs`)
Observed behavior in the code we read:
- Bevy + Egui GUI
- delegates panel drawing:
  - `crate::ui::apply_visual_theme(ctx);`
  - `crate::ui::show_export_panel(ui, state);`
  - `crate::ui::show_settings_panel(ui, state);`
- geometry generation is handled by:
  - a `create_geometry` helper, but the file contains a duplicate `create_geometry`
    implementation (documented in the earlier build failure summary)

This matters for "design notes" because geometry mapping from GUI state → `Geo` must be consistent and single-sourced.

---

## 8) Current simplification/stub markers (explicit)

### Wrapper crate
- Visualization is stubbed (mock png bytes).
- Physics is simplified:
  - radiation impedance is a placeholder
  - no full viscothermal propagation terms are visible in the inspected `sim/mod.rs`

### Accurate crate
- Peak picking semantics are still simplistic local maxima (in the inspected modules).
- GUI geometry mapping risks inconsistency due to duplicate helper function.

---

## 9) Recommended next documentation steps (for completeness)
If continuing beyond this pass:
1. Read `rust-cadsd-accurate/src/{evo,integration,export,persistence,audio}/` and add:
   - how `TairuaLoss` is used by the optimizer,
   - how geometry is exported/synthesized.
2. Read wrapper crate `didgerust/src/lib.rs` and other re-exports to confirm API surface mapping.
3. Finalize parity acceptance criteria:
   - peak semantics agreement,
   - frequency grid agreement,
   - impedance scaling agreement,
   - geometry mapping agreement.