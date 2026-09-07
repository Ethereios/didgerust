# Test Plan

This document defines the regression smoke tests, physics parity tests, conversion tests, optimizer tests, and integration tests required to validate the didgerust / cadsd-accurate alignment.

## Test categories

1. Physics regression tests (geometry + simulation + peak detection)
2. Conversion / note-frequency tests
3. Optimizer smoke tests
4. Loss function parity tests
5. Adapter / reconciliation tests
6. Visualization regression tests

## 1. Physics regression tests

### 1.1 Geometry equivalence

**Goal**: Wrapper and accurate geometries produce equivalent segment structures after mm→m conversion.

| Test ID | Description | Method | Acceptance |
|---------|-------------|--------|------------|
| PHY-GEO-01 | `Geo::make_cone(1000, 32, 60, 10)` produces same number of segments in both crates | Compare wrapper `points.len()` vs accurate `geo.len()` after `create_segments_from_geo` | Equal lengths |
| PHY-GEO-02 | `create_segments_from_geo` converts mm→m identically | Convert same input in both crates, compare segment `l`, `d0`, `d1` | Equal within 1e-12 |
| PHY-GEO-03 | Zero-length duplicate x removal is consistent | Input `[[0,32], [0,35], [100,40], [100,45], [1000,60]]` to both `Geo::new()` | Both produce 3 points |
| PHY-GEO-04 | Bubble insertion preserves monotonic x | Insert bubble in both crates, check `points`/`geo` sorted by x | All `x[i] < x[i+1]` |

### 1.2 Simulation parity

**Goal**: Both crates compute finite, stable impedance values for typical didgeridoo geometries.

| Test ID | Description | Method | Acceptance |
|---------|-------------|--------|------------|
| PHY-SIM-01 | Cylindrical tube impedance is finite | `geo = [[0,32], [1000,32]]`, freq = 100 Hz, both crates | `imp > 0 && imp.is_finite()` |
| PHY-SIM-02 | Conical tube impedance is finite | `geo = [[0,32], [1000,60]]`, freq = 100 Hz, both crates | `imp > 0 && imp.is_finite()` |
| PHY-SIM-03 | Impedance magnitude is `Vec<f64>` aligned to input frequencies | `freqs = [50, 100, 200, 400]`, both crates | `result.len() == freqs.len()` |
| PHY-SIM-04 | Wrapper `impedance()` returns complex values whose `.norm()` matches accurate magnitude | Same geometry and freqs, wrapper `impedance().iter().map(|c| c.norm())` vs accurate magnitudes | Equal within relative tolerance `1e-6` |
| PHY-SIM-05 | No NaN/Inf across wide frequency range | Sweep 20..2000 Hz in 20 Hz steps | All values finite and positive |
| PHY-SIM-06 | Frequency grid generation matches accurate defaults | Wrapper `grid::log_grid(20.0, 2000.0, 1.0)` vs accurate `get_log_simulation_frequencies()` | Equal within floating-point tolerance |

### 1.3 Peak detection parity

**Goal**: Resonance peaks are defined and detected identically.

| Test ID | Description | Method | Acceptance |
|---------|-------------|--------|------------|
| PHY-PEAK-01 | Local maxima definition is consistent | Both crates use `imp[i] > imp[i-1] && imp[i] > imp[i+1]` | Same peak indices for identical magnitude arrays |
| PHY-PEAK-02 | Synthetic spectrum peak detection | Input `freqs=[100,200,300,400,500]`, `imps=[1,5,2,8,1.5]` | Both return peaks at 200 Hz and 400 Hz |
| PHY-PEAK-03 | Real geometry peak count is stable | `Geo::make_cone(1500, 32, 65, 20)`, log grid | Peak count > 0 and stable across reruns |

## 2. Conversion / note-frequency tests

**Goal**: `conv` module functions behave identically to Python DidgeLab reference.

| Test ID | Description | Method | Acceptance |
|---------|-------------|--------|------------|
| CONV-01 | `note_to_freq(69) == 440.0` | Direct call | Equal within 1e-10 |
| CONV-02 | Round-trip `note → freq → note` | Loop over MIDI 40..80 | All round-trips equal |
| CONV-03 | `freq_to_note_and_cent(440.0) == ("A4", 0.0)` | Direct call | Equal within 1e-10 |
| CONV-04 | `cent_diff(100, 200) == 1200.0` (one octave) | Direct call | Equal within 1e-10 |
| CONV-05 | Flat note names convert correctly | `note_name_to_number("Bb4") == 70` | Exact equality |
| CONV-06 | A4=440 reference is correct | `note_to_freq(69)` | Equal within 1e-10 |

## 3. Optimizer smoke tests

**Goal**: Evolutionary optimization runs to completion without panics and produces valid geometries.

| Test ID | Description | Method | Acceptance |
|---------|-------------|--------|------------|
| OPT-01 | Wrapper optimizer with `TestLossFunction` runs 3 generations | `EvolutionaryOptimizer::with_random_population(...)` | Returns `Ok(best_genome)` |
| OPT-02 | Wrapper optimizer with `CompositeTairuaLoss` runs small evolution | Population=10, generations=3, `KigaliGenome` | Loss is finite and non-negative |
| OPT-03 | Accurate optimizer (`Nuevolution`) with simple loss runs 5 generations | `LossFunctionType::Custom` | Returns `Ok(population)` with correct length |
| OPT-04 | Optimizer produces non-decreasing best loss | Track `best_loss` per generation | Final best ≤ initial best (or equal) |
| OPT-05 | Optimized geometry is valid | Final `genome.genome2geo()` | `geo.length() > 0 && geo.points.len() >= 2` |

## 4. Loss function parity tests

**Goal**: Loss values are finite, non-negative, and behaviorally consistent.

| Test ID | Description | Method | Acceptance |
|---------|-------------|--------|------------|
| LOSS-01 | `CompositeTairuaLoss` with cone geometry returns finite non-negative value | `geo = Geo::make_cone(1500, 32, 65, 20)` | `loss >= 0 && loss.is_finite()` |
| LOSS-02 | Accurate `TairuaLoss` with same cone returns finite non-negative value | Same geometry and target | `loss >= 0 && loss.is_finite()` |
| LOSS-03 | Wrapper loss components unit tests pass | Run `cargo test` in wrapper | All pass |
| LOSS-04 | Accurate loss unit tests pass | Run `cargo test` in accurate | All pass |
| LOSS-05 | `FrequencyTuningLoss` penalizes frequency mismatch | Target `log2(73.4)`, actual `log2(100.0)` | Loss > 0 |
| LOSS-06 | `IntegerHarmonicLoss` is zero for perfect harmonics | Peaks at 100, 200, 300 Hz | Loss == 0 |
| LOSS-07 | `PeakAmplitudeLoss` increases when peaks are weaker | Peak amps [1.0, 0.8, 0.6] vs [1.0, 1.0, 1.0] | First loss < second loss |

## 5. Adapter / reconciliation tests

**Goal**: After reconciliation steps, adapters preserve behavior.

| Test ID | Description | Method | Acceptance |
|---------|-------------|--------|------------|
| ADPT-01 | `cadsd::Geo` wraps and unwraps `cadsd_accurate::Geo` correctly | Create wrapper Geo, extract accurate Geo, compare points | Equal |
| ADPT-02 | `DidgeridooSimulator::impedance()` wraps accurate simulation | Compare wrapper `impedance()` norms vs accurate `acoustical_simulation()` | Equal within 1e-6 |
| ADPT-03 | Wrapper `find_resonance_peaks()` uses accurate `get_notes()` under the hood | After reconciliation, compare wrapper peaks vs accurate `compute_ground_spektrum()` first N peaks | Equal frequencies within grid tolerance |
| ADPT-04 | `cadsd::conv` re-exports match accurate exactly | `cadsd::conv::note_to_freq(69)` vs `cadsd_accurate::conv::note_to_freq(69)` | Equal within 1e-10 |
| ADPT-05 | `cadsd::evolutionary_optimizer` delegates to accurate without panic | Run full evolution via wrapper using accurate `DefaultOptimizer` | `result.is_ok()` |

## 6. Visualization regression tests

**Goal**: Report generation and plotting produce expected output files.

| Test ID | Description | Method | Acceptance |
|---------|-------------|--------|------------|
| VIS-01 | `create_analysis_report()` creates output directory and files | Run with cone geometry, check paths | `report.txt`, `geometry.png`, `spectrum.png` exist |
| VIS-02 | `report.txt` contains expected sections | Read generated report | Contains "=== CADSD Analysis Report ===" and peak count |
| VIS-03 | `report.txt` fundamental frequency matches first peak | Parse report | `fundamental == peaks[0].frequency` within tolerance |
| VIS-04 | Stub PNG bytes are replaced with real PNG headers after reconciliation | Inspect first 8 bytes | Starts with PNG magic `137 80 78 71 13 10 26 10` |

## Test execution strategy

### Unit tests

Run with `cargo test` in each crate:

```bash
cd didgerust && cargo test
cd didgerust/rust-cadsd-accurate && cargo test --features gui-bevy
```

### Property / regression tests

Add `#[cfg(test)]` modules in each source file that test cross-crate behavior via the adapter layer.

### CI integration

If a CI pipeline exists, add a matrix:
- `cargo check` (both crates, default features)
- `cargo test` (both crates, `gui-bevy` where applicable)
- `cargo clippy -- -D warnings` (both crates)
- `cargo fmt -- --check` (both crates)

## Acceptance criteria for the project

- **No regressions**: All existing unit tests pass after reconciliation.
- **Physics parity**: Wrapper simulation produces the same impedance values as accurate for the same geometry and frequency grid (within floating-point tolerance).
- **Peak parity**: Peak detection yields the same set of resonance frequencies for identical inputs.
- **Loss sanity**: All loss values are finite, non-negative, and do not produce NaN/Inf for valid geometries.
- **Optimizer completeness**: Optimizer runs to completion for at least 10 generations with population size 20 in under 60 seconds on a modern laptop.
