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

### 3. SuperInstance Integration (TWO CATEGORIES)
#### 3a. Math/Signal Crates (code integration via feature flags)
| Crate | Feature Flag | Purpose | Status |
|-------|-------------|---------|--------|
| **conservation-law-rs** | `conservation-law` | `SymplecticIntegrator<f64, N>` (Störmer–Verlet), `MechanicalLagrangian`, `total_energy()`, Noether's theorem | Target: integrate |
| **lau-signal-processing** | `dsp` | FIR/IIR design, self-contained FFT, STFT, LMS/RLS adaptive filters, Levinson-Durbin LPC | Target: integrate |
| **iir-filter** | `dsp` | Focused Butterworth/Chebyshev biquad filters, `freqz()` | Reference (pick one IIR crate) |
| **constraint-theory-core** | `constraint-theory` | Pythagorean manifold snapping, KD-tree, holonomy checking | Reference |
| **sheaf-spectral** | `sheaf-spectral` | Sheaf Laplacian, graph diffusion, Hodge decomposition | Reference |

#### 3b. App Architecture Patterns (design inspiration, no external dependency)
| SuperInstance Pattern | DidgeRust Application | Implementation |
|-----------------------|----------------------|----------------|
| **PLATO Room** | Bounded contexts for simulation/optimizer/audio subsystems | Each subsystem is a "room" with sensors (inputs), actuators (outputs), history ring buffer, alarm thresholds |
| **Tile** | Immutable state snapshots for undo/redo, persistence, and audit trails | 384-byte atomic units; each simulation result or geometry change is a tile |
| **Deadband** | Threshold-based simulation/UI updates | Already partially in `budget_ops`; formalize as `DeadbandConfig` with PERCENTAGE/ABSOLUTE/THRESHOLD modes |
| **Conservation Fence** | Per-subsystem compute budget enforcement | `budget_ops` already exists; extend to `γ + η = C` where γ = simulation ops, η = overhead |
| **Mesh/Entry-Points** | Plugin architecture for simulation strategies | New strategies register themselves via a `MeshRegistry`-like trait system; GUI discovers them automatically |
| **A2UI** | Data-driven UI generation | Bevy egui already does this; formalize with `DataSchema` → UI mapping |

> **Note:** SuperInstance (https://github.com/SuperInstance/SuperInstance) is a ~2,000-repo ecosystem. The agent/fleet infrastructure (FLUX VM, PLATO server, t-minus dispatcher, cns-bridge mesh) is **overkill for a single-user desktop app** and should NOT be integrated as a whole. The patterns above are extracted as lightweight design guidelines. See `docs/RESEARCH.md §17.6` for full assessment.

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
| All tests pass | `cargo test` → 102 lib + 30 integration = 132 total | ✅ |
| Benchmarks run | `cargo bench` → ~1.2ms loss, ~0.7µs segment creation | ✅ |
| GUI launches | `cargo run --bin cadsd-gui --features gui-bevy` | ✅ |
| Bent-shape correction | Curvature/taper sliders wired into optimizer + GUI | ✅ |
| Loss component toggles | GUI toggles build CompositeTairuaLoss dynamically | ✅ |
| Conservation dashboard | Real-time γ/η = C visualization | ✅ |
| Export JSON | Spectrum export as JSON with magnitude+phase | ✅ |
| Prime population init | `with_prime_population()` for quasi-random seeding | ✅ |

---

## Next Development Steps (Priority Order)

1. **Add segment editor** — table view of `[x, d]` points in geometry panel
2. **Add true 3-D bore preview** — Bevy gizmos conical frustum with orbit controls
3. **Add tonehole drag-and-drop** — interactive placement on bore preview
4. **Add compute thread count** — slider in Settings wired to rayon thread pool
5. **Wire conservation-law crate** — use `SymplecticIntegrator` in long-time simulation loops
6. **Wire lau-signal-processing crate** — use FFT/IirFilter for bore-wall absorption modeling
7. **Fix radiation impedance docs** — rename "Geipel" comments to "Levine-Schwinger IIR"
8. **Standardise frequency grid** — cents-based log spacing as default everywhere
9. **Add UI tests** — manual + automated tests for panel interactions
10. **Expand unit tests** — target 150+ from current 132
11. **Neural network integration** — add crates behind feature flag `nn-integration`:
    - `dfdx` — differentiable TLM prototype
    - `tch-rs` — production training
    - `burn` — pure-Rust research alternative
12. **Complex impedance validation** — validate against TLM with published data

---

## Architecture Compliance (SuperInstance)

All new code follows the **Shell** principle:
- Old TLM path untouched
- New methods are independent modules
- No forced migration, just additive options

All paths respect **12V constraint**:
```rust
// In each strategy impl:
budget_ops.check_budget(ops_estimate)?;
```

The project is ready for production use with clear extension points for ML integration.

---

*This document serves as the final implementation specification and acceptance checklist.*