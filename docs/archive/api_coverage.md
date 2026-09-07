# API Coverage Matrix

This document maps the public API surface of both crates and identifies gaps, overlaps, and semantic differences.

## Crates

| Crate | Path | Purpose |
|-------|------|---------|
| `cadsd` (wrapper) | `didgerust/src/*` | Higher-level Rust API; simplified physics, parametric genomes, composite loss |
| `cadsd-accurate` (accurate) | `didgerust/rust-cadsd-accurate/src/*` | Exact Python DidgeLab parity; full viscothermal physics, GUI, CLI, audio, export |

## Module-level mapping

| Wrapper module | Accurate module | Status |
|----------------|-----------------|--------|
| `geo` | `geo` | Partial overlap; different struct names and helper methods |
| `sim` | `sim` | Partial overlap; wrapper is simplified, accurate is full physics |
| `evo` | `evo` | **Diverged**; different genome/optimizer designs |
| `loss` | `loss` | Partial overlap; wrapper has composite components, accurate has Tairua/DidgeLab |
| `visualization` | `analysis`, `ui` | **Improved**; wrapper now exports `get_notes` from accurate's `analysis` |
| *missing* | `conv` | Not present in wrapper |
| `integration` (minimal) | `integration` | Minimal stubs: `DefaultSimulator`, `DefaultOptimizer` |
| *missing* | `inverse_design` | Not present in wrapper |
| *missing* | `audio` | Not present in wrapper |
| `export` (minimal) | `export` | Minimal stubs: `GeometryExporter`, `DataExporter` |
| *missing* | `persistence` | Not present in wrapper |
| *missing* | `cli` (main.rs) | Not present in wrapper |

## Public type / function mapping

### Geometry (`geo`)

| Wrapper (`cadsd::geo`) | Accurate (`cadsd_accurate::geo`) | Gap / Difference |
|------------------------|----------------------------------|------------------|
| `Geo { points: Vec<[f64; 2]> }` | `Geo { geo: Vec<[f64; 2]> }` | **Field name differs** (`points` vs `geo`); wrapper exports `BoreGeometry = Geo` |
| `Geo::new()` | `Geo::new()` | Accurate removes zero-length duplicate-x segments; wrapper sorts but does not dedup |
| `length()` | `length()` | Compatible (mm) |
| `bellsize()` | `bellsize()` | Compatible (mm) |
| `diameter_at(x)` | `diameter_at_x(x)` | Wrapper: linear interpolation; Accurate: atan-based interpolation (different semantics) |
| `diameter_at_x(x)` | `diameter_at_x(x)` | Wrapper alias; accurate uses it directly |
| `scale_diameter(factor)` | `scale_diameter(max_d)` | **Different signature**: wrapper scales by factor, accurate scales to absolute max diameter |
| `stretch(factor)` | `stretch(factor)` | Compatible (x only) |
| `add_bubble(center, width, height)` | `make_bubble(pos, width, height)` | Different insertion strategy and point count |
| `compute_volume()` | `compute_volume()` | Different formulas: wrapper uses `PI*dx*(r0^2+r0*r1+r1^2)/3`; accurate uses `PI*dx*(d1^2+d1*d2+d2^2)/12` |
| `get_max_diameter()` | `get_max_d()` | Alias; different names |
| *missing* | `scale(factor)` | Accurate method scales both x and diameter |
| *missing* | `scale_length(max_length)` | Accurate method |
| *missing* | `sort_segments()` | Accurate method |
| *missing* | `copy()` | Accurate method |
| *missing* | `from_file()` / `to_file()` | Accurate JSON serialization |
| *missing* | `print_summary()` | Accurate method |
| *missing* | `make_kigali()` | Accurate parametric generator |
| *missing* | `make_mbeya()` | Accurate parametric generator |
| *missing* | `taper_ratio()` | Accurate method |

### Simulation (`sim`)

| Wrapper (`cadsd::sim`) | Accurate (`cadsd_accurate::sim`) | Gap / Difference |
|------------------------|----------------------------------|------------------|
| `Segment { l, d0, d1, a0, a01, a1, phi, x0, x1, r0 }` | `Segment { l, d0, d1, a0, a01, _a1, phi, x0, x1, r0 }` | Wrapper exports `Segment`; accurate keeps it private |
| `create_segments_from_geo()` | `create_segments_from_geo()` | Both convert mm→m; accurate handles zero-length guard |
| `cadsd_ze(segments, freq_hz) -> Complex<f64>` | `cadsd_ze(segments, frequency) -> f64` | **Return type differs**: wrapper returns complex, accurate returns magnitude |
| `ap(m, n)` | `ap(w, segments)` | Wrapper combines two 2x2 matrices; accurate builds cascade from segments |
| `za(z, r)` | `za(w, segments)` | **Completely different**: wrapper is spherical placeholder; accurate is Geipel/derived with viscothermal terms |
| `compute_impedance_spectrum()` | `acoustical_simulation(geo, freqs, method)` | Wrapper: complex spectrum from segments; Accurate: magnitudes only, accepts `geo` directly |
| `DidgeridooSimulator { segments }` | *missing* | Wrapper-only high-level struct |
| `DidgeridooSimulator::from_geo()` | *missing* | Wrapper-only constructor |
| `DidgeridooSimulator::impedance()` | *missing* | Wrapper-only; returns `Vec<Complex<f64>>` |
| `DidgeridooSimulator::peaks()` | *missing* | Wrapper-only; returns `Vec<(usize, f64, f64)>` |
| `DidgeridooSimulator::find_resonance_peaks()` | `get_notes()`, `compute_ground_spektrum()`, `get_fundamental()` | Wrapper uses log grid; accurate uses log grid + local maxima |
| `find_peaks(freqs, spectrum)` | `get_notes(freqencies, impedances)` | Both: strict local maxima; wrapper works on `Complex`, accurate on `f64` magnitudes |
| `grid::log_grid()` | `get_log_simulation_frequencies()` / `get_log_simulation_frequencies_with_params()` | Wrapper generic cents-based grid; accurate default (20..2000, grid_size=1.0) |
| `grid::lin_grid()` | *missing* | Wrapper-only linear grid |
| `SimulationParams { freq_range, points }` | `Config { sim_fmin, sim_fmax, sim_grid_size, sim_grid, sim_backend }` | Wrapper: unused in current resonance path; Accurate: system-level config |
| `Resonance { frequency, impedance }` | *missing* | Wrapper-only result struct |
| *missing* | `acoustical_simulation()` | Accurate public entry point |
| *missing* | `get_fundamental()` | Accurate public entry point |
| *missing* | `compute_ground_spektrum()` | Accurate public entry point |

**Physics divergence note**: Wrapper `cadsd_ze` uses lossless propagation (`k = ω/c`) and a placeholder radiation impedance. Accurate `cadsd_ze` includes viscothermal losses (`rvw`, `tw`, `zcw`), conical transfer matrices, and Geipel radiation impedance.

### Evolution (`evo`)

| Wrapper (`cadsd::evo`) | Accurate (`cadsd_accurate::evo`) | Gap / Difference |
|------------------------|----------------------------------|------------------|
| `Genome` trait | `LossFunction` trait | Wrapper trait for genomes; accurate trait for loss |
| `BaseGenome` | `GeoGenome` | Wrapper: generic genome with genes; Accurate: simple gene→cone decoder |
| `KigaliGenome` | *missing* | Wrapper-only parametric shape genome |
| `EvolutionaryOptimizer` | `Nuevolution` | **Completely different** APIs and internal logic |
| `EvolutionParameters` | `Nuevolution` fields | Wrapper: struct with sizes/rates; Accurate: builder-style config |
| `LossFunction` trait (wrapper) | `LossFunctionType` enum | Wrapper: trait for `calculate(&dyn Genome) -> f64`; Accurate: enum over Tairua/DidgeLab/Custom |
| `MutationOperator` enum | Inline in `Nuevolution` / `MutationOperator` enum | Different variants and APIs |
| `CrossoverOperator` enum | `CrossoverOperator` enum | Different variants |
| `TargetSound` | `TargetSound` | Similar but different fields and builder methods |
| `BoreShapePreference` | `BoreShapePreference` | Similar enum |
| *missing* | `from_note()` | Accurate note-based target creation |
| *missing* | `with_toot_note()` | Accurate note-based toot addition |
| *missing* | `InverseDesigner` / `DesignResult` | Moved to `inverse_design` module in accurate |

### Loss (`loss`)

| Wrapper (`cadsd::loss`) | Accurate (`cadsd_accurate::loss`) | Gap / Difference |
|------------------------|----------------------------------|------------------|
| `LossComponent` trait | `LossFunction` trait | Wrapper trait takes peak arrays + full spectrum; Accurate trait takes `&Geo` |
| `TestLossFunction` | *missing* (has `Custom` in `LossFunctionType`) | Wrapper-only |
| `FrequencyTuningLoss` | *missing* | Wrapper-only component |
| `QFactorLoss` | *missing* | Wrapper-only component |
| `ModalDensityLoss` | *missing* | Wrapper-only component |
| `HighInharmonicLoss` | *missing* | Wrapper-only component |
| `IntegerHarmonicLoss` | *missing* | Wrapper-only component |
| `NearIntegerLoss` | *missing* | Wrapper-only component |
| `StretchedOddLoss` | *missing* | Wrapper-only component |
| `HarmonicSplittingLoss` | *missing* | Wrapper-only component |
| `PeakQuantityLoss` | *missing* | Wrapper-only component |
| `PeakAmplitudeLoss` | *missing* | Wrapper-only component |
| `ScaleTuningLoss` | *missing* | Wrapper-only component |
| `CompositeTairuaLoss` | `TairuaLoss` | **Different designs**: wrapper is component-based with normalized peaks; accurate is monolithic with harmonic alignment |
| *missing* | `FundamentalFrequencyLoss` | Accurate-only |
| *missing* | `GeometricLoss` | Accurate-only |
| *missing* | `MultiObjectiveLoss` | Accurate-only |
| *missing* | `DidgeLabLoss` | Accurate-only comprehensive inverse-design loss |

### Analysis / Visualization

| Wrapper (`cadsd::visualization`) | Accurate (`cadsd_accurate::analysis`) | Gap / Difference |
|----------------------------------|----------------------------------|------------------|
| `plot_bore_geometry()` (stub) | `vis_didge()` (print) | Wrapper writes mock PNG; accurate prints summary |
| `plot_impedance_spectrum()` (stub) | `plot_impedance_spectrum()` (print) | Wrapper writes mock PNG; accurate prints summary |
| `plot_evolution_progress()` (stub) | *missing* | Wrapper-only stub |
| `create_analysis_report()` | *missing* | Wrapper-only (writes geometry.png, spectrum.png, report.txt) |
| `generate_text_report()` | *missing* | Wrapper-only helper |
| `get_notes()` | `get_notes()` | **Migrated** from accurate's analysis module; peak extraction on raw magnitudes |
| *missing* | `plot_bore()` | Accurate-only print helper |
| *missing* | `vis_didge()` | Accurate-only print helper |

### Integration (new module)

| Wrapper (`cadsd::integration`) | Accurate (`cadsd_accurate::integration`) | Gap / Difference |
|----------------------------------|----------------------------------|------------------|
| `DefaultSimulator` | `AcousticSimulator` trait | Minimal stub with simplified impedance calculation |
| `DefaultOptimizer` | `EvolutionaryOptimizer` trait | Minimal stub using `Geo::make_cone` |

### Export (new module)

| Wrapper (`cadsd::export`) | Accurate (`cadsd_accurate::export`) | Gap / Difference |
|----------------------------------|----------------------------------|------------------|
| `GeometryExporter` | `DefaultExporter` | Minimal stub for OBJ/Gltf export |
| `DataExporter` | *missing* | Minimal stub for CSV spectrum export |

### Missing from wrapper (accurate-only modules)

| Module | Key Public Items |
|--------|------------------|
| `conv` | `note_to_freq`, `freq_to_note`, `note_name`, `freq_to_note_and_cent`, `freq_to_wavelength`, `cent_diff`, `note_name_to_number` |
| `integration` | `AcousticSimulator` trait, `DefaultSimulator`, `EvolutionaryOptimizer` trait, `DefaultOptimizer`, `AudioSynthesizer` trait, `DefaultSynthesizer`, `GeometryExporter` trait, `DefaultExporter` |
| `inverse_design` | `InverseDesigner`, `DesignResult` |
| `audio` | `DefaultSynthesizer` |
| `export` | `DefaultExporter` |
| `persistence` | `AppSettings`, `ProjectState` |
| `cli` / `main.rs` | Command-line entry points |
| `gui` (Bevy) | `CadsdState`, `run_app()`, `create_geometry()` |

### Missing from accurate (wrapper-only items)

| Item | Notes |
|------|-------|
| `DidgeridooSimulator` | High-level simulation wrapper |
| `KigaliGenome` | Rich parametric genome |
| `CompositeTairuaLoss` and component suite | Modular loss with normalized peaks |
| `create_analysis_report()` | Report generation with text + stub PNGs |
| `grid::lin_grid()` | Linear frequency grid helper |

## Summary of gaps

| Gap | Severity | Notes |
|-----|----------|-------|
| Wrapper `Geo` vs accurate `Geo` field names | Medium | `points` vs `geo` requires adapter or unification |
| Wrapper `Geo` missing `make_kigali`, `make_mbeya` | Medium | Wrapper CLI examples call `Geo::make_kigali` and `make_mbeya` but wrapper Geo lacks them |
| Physics divergence | High | Wrapper uses lossless + placeholder radiation; accurate has full viscothermal model |
| Peak semantics alignment | Medium | Both use local maxima, but wrapper `find_peaks` acts on `Complex` norm while accurate uses precomputed magnitudes |
| Frequency grid divergence | Medium | Wrapper default to linear for `find_resonance_peaks` in DESIGN_NOTES; currently using log. Accurate uses log. |
| Wrapper evo vs accurate evo | High | Two incompatible optimizer implementations |
| Wrapper loss vs accurate loss | Medium | Different loss architectures; wrapper normalizes peaks, accurate does not |
| Wrapper missing `conv`, `integration`, `inverse_design` | Medium | No note/frequency utilities, no service traits, no inverse designer |
| Wrapper `compute_volume()` formula | Low | Accurate uses `... / 12.0`, wrapper uses `... / 3.0` per frustum segment |
| Wrapper `diameter_at()` interpolation | Low | Accurate uses atan formula; wrapper uses linear interpolation |
| Wrapper `scale_diameter()` semantics | Medium | Wrapper scales by factor; accurate scales to absolute max diameter |
