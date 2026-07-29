# didgerust - Implementation Plan Checklist

## Phase 0: Inventory + API coverage
- [x] Create API coverage matrix for both crates (`didgerust` and `cadsd-accurate`)
- [x] Identify public exports and map to implemented internals
- [x] Identify gaps (missing modules, partial implementations, mismatched semantics)

## Phase 1: Module-by-module code reading (cadsd-accurate)
- [x] `src/geo/` (geometry invariants, mm units, parametric generators)
- [x] `src/sim/` (acoustical simulation pipeline + backends)
- [x] `src/conv/` (note↔freq conversion, cent calculations)
- [x] `src/analysis/` (peak detection, note labeling, report helpers)
- [x] `src/evo/` (genome encoding, mutation/crossover, selection loop)
- [x] `src/loss/` (each loss component, intermediate requirements)
- [x] `src/export/`, `src/persistence/`, `src/audio/` (IO/state serialization)
- [x] `src/ui/` + app entrypoints (wiring to core logic)

Deliverable after Phase 1:
- [ ] Create `DESIGN_NOTES.md` with per-module call graphs + invariants

## Phase 2: Module-by-module code reading (top-level didgerust crate)
- [x] `src/sim/`
- [x] `src/geo/`
- [x] `src/evo/`
- [x] `src/loss/`
- [x] `src/visualization/`
- [x] `src/lib.rs`, `src/main.rs`, examples/bin

Deliverable after Phase 2:
- [x] Update API coverage matrix for differences vs `cadsd-accurate`

## Phase 3: Reconciliation / alignment plan
- [x] Decide target architecture: single backend vs adapters
- [x] Propose adapter layer between `didgerust` and `cadsd-accurate` APIs
- [x] Identify duplicated logic and deprecation steps

Deliverables:
- [x] `RECONCILIATION_PLAN.md`

## Phase 4: Testing + validation plan
- [x] Create physics regression test plan (geometry + simulation + peak detection)
- [x] Create conversion regression test plan
- [x] Create optimizer smoke test plan

Deliverables:
- [x] `ACCURACY_PARITY.md` (already existed, validated)
- [x] `TEST_PLAN.md`

## Phase 5: Performance plan
- [ ] Identify hotspots (segments conversion, impedance recompute, peak scanning)
- [ ] Propose caching + batching strategy
- [ ] Plan profiling steps

Deliverables:
- [ ] `PERF_REPORT.md`

## Tracking
- [ ] Mark tasks completed after each phase

