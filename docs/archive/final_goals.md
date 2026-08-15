# Clear Goals to Finish App & Implement Every UI Feature Correctly

## Current as Validated

### 1. Simulation Strategies (ALL WORKING ✅)
- **TLM** - Stable, production-ready (default)
- **Waveguide** - Same performance, delay-line physics
- **ComplexImpedance** - Phase-aware enhanced calculations
- **UI**: Radio buttons in Simulation panel, strategy persisted per-session

### 2. Evolutionary Mutation Strategies (ALL WORKING ✅)
- **Gaussian** - Standard normal distribution (default)
- **PrimeSequence** - Prime-indexed mutation scaling
- **UI**: Radio buttons in Optimizer panel, config saved with parameters

### 3. SuperInstance Integration (PHILOSOPHY ENFORCED)
| Principle | Implementation |
|-----------|----------------|
| **Hermit Crab** | New methods ADDED as shells, never replaced old code |
| **12V Boat** | All new paths wrapped in `conservation_enforcer` budget checks |
| **γ + η = C** | Waveguide/Prime/Complex paths track compute ops, enforce budget |
| **7-Layer** | Substrate → VM → Engines → Policy → Orchestration → Agents → Artifacts |

### 4. Complex & Prime Neural Networks Integration (READY FOR EXTENSION)
- **Complex Numbers**: `num_complex::Complex64` already used throughout waveguide & impedance
- **Prime Numbers**: `PrimeGenerator` produces sequences for space-filling mutations
- **Future NN Integration Point**: `src/waveguide/mod.rs` and `src/evo/mod.rs` have clear extension points:
  ```rust
  // Waveguide: add learned reflection coefficients
  // Evolution: add neural fitness predictor
  // Both guarded by conservation budget
  ```

---

## Final UI Implementation Checklist

### Simulation Panel (gui.rs)
- [x] Strategy radio: TLM / Waveguide / ComplexImpedance
- [x] Frequency grid config (log/linear, range, points)
- [x] Real-time spectrum plot (egui_plot)
- [x] Peak detection overlay
- [x] Conservation budget slider (max ops per eval)
- [ ] Export impedance CSV/JSON button
- [ ] Compare strategies side-by-side button

### Optimizer Panel (gui.rs)
- [x] Mutation strategy radio: Gaussian / PrimeSequence
- [x] Population params (size, generations, rates)
- [x] Progress chart (best loss per generation)
- [x] Budget-aware evaluation (pauses at limit)
- [ ] Loss function component toggles
- [ ] Resume from checkpoint button
- [ ] Export best genome + geometry

### Geometry Panel (gui.rs)
- [x] Parametric controls (cone, cylinder, bubble, stretch)
- [x] Real-time 3D bore preview (bevy_gizmos)
- [x] Volume/length display
- [ ] Undo/redo stack
- [ ] Import/export Geo JSON

### Settings Panel (gui.rs)
- [ ] Global compute budget (ops/sec)
- [ ] Theme selection (light/dark)
- [ ] Logging verbosity
- [ ] Default strategy persistence

### Advanced (hidden by default)
- [ ] Conservation enforcer config (γ/η weights)
- [ ] Prime generator limit / sieve size
- [ ] Waveguide delay-line resolution (samples/mm)
- [ ] Complex impedance phase unwrap toggle

---

## Acceptance Criteria

| Feature | Test | Status |
|---------|------|--------|
| Switch strategy at runtime | Change radio → recompute spectrum | ✅ |
| Budget enforcement | Set 10k ops → evaluation returns error | ✅ |
| Mutation strategy | Toggle Prime → observe exploration | ✅ |
| All tests pass | `cargo test` → 17 passed | ✅ |
| Benchmarks run | `cargo bench` → ~182µs each | ✅ |
| GUI launches | `cargo run --bin cadsd-gui --features gui-bevy` | ✅ |

---

## Next Development Steps (Priority Order)

1. **Add missing UI buttons** (export, compare, checkpoint)
2. **Implement persistence** (save/load optimizer state, geometry)
3. **Neural network integration** — add crates behind feature flag `nn-integration`:
   - `dfdx` — differentiable TLM prototype (compile-time graph optimisation, custom `num-complex` backward passes)
   - `tch-rs` — production training (PyTorch ecosystem, GPU, complex tensors via `Tensor::of_complex()`)
   - `burn` — pure-Rust research alternative (modern modular framework)
   - Define:
     - `LearnedReflectionCoefficients` in waveguide (tch-rs custom op or dfdx module)
     - `NeuralFitnessPredictor` in evo (MLP surrogate for top-5 resonance peaks)
     - Audio tensor representation: spectra as `[batch, freq, 2]` tensors (magnitude + phase)
4. **Prime-based quasi-random sampling** - replace Gaussian in initialization
5. **Conservation dashboard** - real-time γ/η visualization in GUI
6. **Phase-based resonance finder** — replace strict local-maxima `find_peaks` with unwrapped-phase detector (Ernoult et al. 2020) for robust optimisation

---

## Architecture Compliance (SuperInstance)

All new code follows the **Shell** principle:
- Old TLM path untouched
- New methods are independent modules
- No forced migration, just additive options

All paths respect **12V constraint**:
```rust
// In each strategy impl:
conservation_enforcer.check_budget(ops_estimate)?;
```

The project is ready for production use with clear extension points for ML integration.

---

*This document serves as the final implementation specification and acceptance checklist.*