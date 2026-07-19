# Loss Functions Documentation for `rust-cadsd`

## Overview
The `rust-cadsd` crate provides a flexible loss function system for evaluating didgeridoo geometries during evolutionary optimization. The core abstractions are `LossComponent` and `CompositeTairuaLoss`. Individual components capture specific acoustic criteria, and the composite aggregates them.

## Loss Components
| Component | Description | Constructor | Typical Weight |
|-----------|-------------|-------------|----------------|
| `IntegerHarmonicLoss` | Penalizes deviation from integer harmonic relationships. | `IntegerHarmonicLoss::new(weight)` | 5.0 |
| `NearIntegerLoss` | Allows near‑integer harmonics within a tolerance. | `NearIntegerLoss::new(tolerance, weight)` | 5.0 (tolerance 0.05) |
| `StretchedOddLoss` | Targets stretched odd harmonics (1,3,5…) with a stretch factor. | `StretchedOddLoss::new(stretch_factor, weight)` | 5.0 |
| `HarmonicSplittingLoss` | Drives adjacent peak ratios away from integer values, encouraging richer spectra. | `HarmonicSplittingLoss::new(weight)` | 5.0 |
| `PeakQuantityLoss` | Encourages a target number of resonance peaks. | `PeakQuantityLoss::new(target_quantity, weight)` | 2.0 |
| `PeakAmplitudeLoss` | Rewards stronger resonance peaks. | `PeakAmplitudeLoss::new(weight)` | 2.0 |
| `ScaleTuningLoss` | Pulls peaks toward standard chromatic notes (MIDI). | `ScaleTuningLoss::new(weight)` | 5.0 |
| `FrequencyTuningLoss` | Aligns peaks to a set of explicit target frequencies. | `FrequencyTuningLoss::new(target_freqs_log, target_impedances, weights)` | User‑defined |
| `QFactorLoss` | Controls resonance sharpness via Q‑factor. | `QFactorLoss::new(target_q, weight)` | User‑defined |
| `ModalDensityLoss` | Encourages clusters of peaks (shimmer) within a cents range. | `ModalDensityLoss::new(cluster_range_cents, weight)` | User‑defined |
| `HighInharmonicLoss` | Rewards high inharmonicity for more dissonant timbres. | `HighInharmonicLoss::new(weight)` | 1.0 |

## Composite Loss Builder
`CompositeTairuaLoss` aggregates any number of components. For a typical optimisation you can start with a sensible default set:

```rust
use rust_cadsd::loss::{CompositeTairuaLoss, IntegerHarmonicLoss, NearIntegerLoss,
    StretchedOddLoss, HarmonicSplittingLoss, PeakQuantityLoss, PeakAmplitudeLoss,
    ScaleTuningLoss};

// `max_error` is the frequency‑grid resolution in cents.
let mut loss = CompositeTairuaLoss::with_default_components(5.0);
// Add custom components if desired
loss.add_component("my_custom".to_string(), Box::new(IntegerHarmonicLoss::new(10.0)));
```

The `with_default_components` method builds a loss comprising the seven most common components with reasonable weights.

## Using the Composite Loss in the Optimizer
```rust
use rust_cadsd::{evo::{EvolutionaryOptimizer, EvolutionParameters}, loss::CompositeTairuaLoss, Genome};

let loss = CompositeTairuaLoss::with_default_components(5.0);
let optimizer = EvolutionaryOptimizer::with_random_population(
    Box::new(loss),
    &genome_template,
    50, // population size
    EvolutionParameters::default(),
);
```

## Testing
The crate ships a `TestLossFunction` that simply sums genome values – useful for quick sanity checks.

```rust
let test_loss = TestLossFunction::new();
assert_eq!(test_loss.calculate(&genome), /* expected value */);
```

## Re‑exports
All loss components are re‑exported from the crate root for convenient access:

```rust
use rust_cadsd::loss::{CompositeTairuaLoss, IntegerHarmonicLoss, NearIntegerLoss, /* … */};
```

---

*Generated on 2026‑05‑26.*
