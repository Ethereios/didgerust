# DidgeRust Future Goals

This file documents important features and improvements that are **not yet implemented**.
These are intentional future goals — not placeholders or stubs.

## Acoustics / Simulation

- **Webster Horn Equation solver** — explicit standalone solver for varying cross-section tubes, complementing the current TLM approach.
- **Real-time waveguide synthesis** — true time-domain digital waveguide audio output, replacing the current single-frequency tone generator.
- **ComplexImpedance strategy completion** — bring it to feature parity with TLM and Waveguide: tonehole support, proper `AcousticConstants`, and Levine-Schwinger radiation impedance.
- **Analytical validation for non-cylindrical geometries** — reference solutions for conical and exponential bores to cross-check TLM accuracy.
- **Edge-tone physics refinement** — more accurate tonehole edge-tone resistance models based on published aeroacoustic data.

## Geometry

- **Expose `make_mbeya` and `make_kigali`** — parametric didgeridoo shape generators exist in the accurate crate but are not yet exposed through the wrapper API.
- **RBF constraint systems** — radial-basis-function constraints for smooth geometry deformation during optimization.
- **Geo utility method tests** — `scale_length`, `scale_diameter`, `get_max_d`, `make_bubble`, `diameter_at_x` edge cases.

## Audio

- **WAV export from waveguide synthesis** — current WAV export uses a simple sine wave; future version should render from the actual waveguide time-domain simulation.
- **cpal real-time audio tests** — test actual audio stream creation and callback pipeline on supported platforms.

## Neural / Differentiable

- **Consolidate `diff_tlm` and `nn-integration`** — merge `DiffSegment` / `NeuralFitnessPredictor` into a single, correct differentiable TLM with real backpropagation (Wirtinger calculus through the full cascade).
- **Neural fitness predictor training** — replace the current mean-predictor stub with a real MLP trained on simulation data.

## Visualization

- **3D mesh rendering** — replace wireframe gizmos with proper 3D meshes (Bevy mesh rendering).
- **Offline PNG/SVG report generation** — automated bore geometry and impedance spectrum plots for documentation.

## GUI / UX

- **Wire up Settings Panel buttons** — "Play Note" should trigger `AudioProcessor`; "Run Simulation" should compute spectrum; "Refresh Preview" should redraw 3D view; Bore Style ComboBox should map to actual geometry presets.
- **Tonehole drag-and-drop on bore visualization** — currently toneholes are edited via scroll wheel and sliders; direct drag interaction on the bore plot would improve UX.
- **Strategy-aware tonehole editing** — when a non-TLM strategy is selected, tonehole controls should be disabled rather than just warned.

## Performance

- **SIMD optimization** — vectorized acoustic calculations for spectrum evaluation.
- **Parallel Rayon evaluation** — parallelize genome fitness evaluation in the evolutionary optimizer.
- **Geometry caching** — cache segment computations for unchanged bore geometries.
- **`acoustical_simulation` backend unification** — the accurate crate still has `tlm_python` / `tlm_cython` backends; the wrapper should eventually replace these entirely.

## Testing

- **File I/O tests for persistence** — test `AppSettings`, `ProjectState`, and `OptimizerCheckpoint` save/load with real files.
- **`insert_toneholes` direct tests** — test the tonehole insertion logic with edge cases (multiple toneholes, toneholes at segment boundaries).
- **`bent_effective_length` parameterized tests** — cover zero, negative, and large curvature values.
- **`AcousticConstants::for_conditions` edge cases** — extreme temperatures, pressures, and humidities.
- **`EvolutionaryOptimizer` pause/resume tests** — cover `set_pause_flag` and `evolve_with_progress` callback.
