# didgerust - Performance Report

## Hotspots

### 1. Segment conversion (`create_segments_from_geo`)
- **Location**: `sim/mod.rs` in both crates
- **Cost**: O(n) per call where n = number of segments
- **Impact**: Called for every frequency in the impedance sweep; for a 512-point grid, this is 512 allocations + conversions
- **Mitigation**: Cache the converted segment vector and reuse across frequency evaluations

### 2. Impedance recomputation (`impedance()`)
- **Current**: Recomputes the full cascade for every frequency point
- **Cost**: O(n * m) where n = segments, m = frequency points (typically 500+)
- **Impact**: Dominates simulation time in optimization loops where geometry changes slightly between generations
- **Mitigation**: Cache segment conversion; consider batching frequency evaluation

### 3. Peak scanning (`find_resonance_peaks()`)
- **Location**: `sim/mod.rs` in wrapper, `analysis/mod.rs` in accurate
- **Cost**: O(m) for m frequency points
- **Impact**: Minor relative to simulation; but called frequently in loss computation
- **Mitigation**: Not a primary bottleneck; can be deferred

### 4. Loss component calculations (`LossComponent::calculate()`)
- **Cost**: Each component iterates over peak arrays; composite loss sums N components
- **Impact**: N * peaks * spectrum_length per genome evaluation
- **Mitigation**: Pre-normalize peak impedances once per spectrum evaluation

### 5. Geometry operations (`make_bubble`, `stretch`, `scale`)
- **Cost**: O(n) for bubble insertion (shifts elements), O(n) for stretch/scale
- **Impact**: Negligible for moderate segment counts (<200)
- **Mitigation**: Not a bottleneck

## Caching Strategy

### Recommended cache layers:

```
Level 1: Segment Cache (per Geo)
  - Convert Geo -> Segment<T> once
  - Invalidate on: stretch(), scale(), add_bubble()
  - Use a dirty flag or version counter on Geo

Level 2: Impedance Spectrum Cache (per Geo)
  - Cache full impedance spectrum for a given frequency grid
  - Invalidate on: any geometry change, frequency grid change
  - Key: (geo_hash, freq_grid_hash)

Level 3: Peak Cache (per spectrum)
  - Cache peak extraction results
  - Invalidate on: spectrum data change
```

### Batch computation plan:
1. Convert geometry to segments once per generation
2. Compute impedance for all frequencies in one pass through the cascade
3. Extract peaks once from the full spectrum
4. Feed normalized peaks to all loss components

This reduces redundant work from O(generations * freqs * segments) to O(generations * segments + generations * freqs).

## Profiling Steps

1. **Baseline**: Run `cargo test --release` and measure wall-clock time for 1000-generation evolution
2. **criterion**: Insert criterion benchmarks for `impedance()`, `find_resonance_peaks()`, `compute_loss()`
3. **flamegraph**: Use `cargo flamegraph` to identify CPU hotspots
4. **valgrind/cachegrind**: Profile cache misses in the cascade multiplication

## Optimization Priority

| Priority | Hotspot | Expected Speedup | Effort |
|----------|---------|-----------------|--------|
| High | Segment conversion cache | 2-3x | Medium |
| High | Impedance batch eval | 1.5-2x | Medium |
| Medium | Peak extraction cache | 1.2-1.5x | Low |
| Low | Loss component pre-normalization | 1.1-1.3x | Low |