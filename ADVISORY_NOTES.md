# Advisory Notes

- You are not an assistant but my advisor who happens to be smarter but not more intelligent.
- Your first response should not be agreement but challenge my assumptions.
- Point out what I'm missing or identify gaps in my thinking.
- Rate your confidence.
- Before any claim tag it [certain] if you have concrete evidence, [likely] if it's a strong inference, [guessing] if you are filling gaps.
- Kill phrases: "excellent question", "you are absolutely right", "that makes a lot of sense", "absolutely", "definitely". If you catch yourself behaving like this, delete and rewrite.

## General Instruction

- If you read the crucial docs before starting a task, especially this one, we can work much better.
- Start with reading AGENTS.md or instructions files before beginning any task.

## Session Notes (2026-01-17)

- **Geo::make_bubble consistency**: The `Geo::make_bubble` method in `rust-cadsd-accurate/src/geo/mod.rs` was using a triangular profile, while `KigaliGenome::make_bubble` in `src/evo/mod.rs` uses a sinusoidal profile. I fixed `Geo::make_bubble` to use the same sinusoidal profile (10 sample points) as `KigaliGenome::make_bubble`.
- **Geo::make_bubble geometry bug**: The original `make_bubble` implementation could create unsorted geometry arrays, which broke `diameter_at_x` and volume calculations. I fixed it to properly filter points inside the bubble range and insert the sinusoidal bubble points in sorted order.
- **Validation tests**: Added 4 tests in `rust-cadsd-accurate/src/geo/mod.rs`: `test_make_bubble_shape_continuity`, `test_make_bubble_volume_increase`, `test_make_bubble_simulation_valid`, `test_make_bubble_numerical_stability`. All 13 geo tests pass.
- **Full test suite**: `cargo test -p cadsd-accurate` passes all 32 lib tests in ~11s (was 60s+ timeout). Root cause was an infinite loop in `get_log_simulation_frequencies_with_points` (sim/mod.rs:266) that caused tests to hang. Fixed by restructuring the loop to break when no frequencies are added in an octave. Also refactored TairuaLoss, FundamentalFrequencyLoss, and MultiObjectiveLoss to use a single acoustic simulation per call instead of 3 redundant simulations. Full frequency grid preserved — no accuracy compromise.
- **Unused import warning**: Removed `use approx::assert_abs_diff_eq;` from sim/mod.rs tests section (was unused).

## ContentStudio Migration (Legacy Notes - from previous session)

- **Content Studio migration**: The content studio was migrated from `src/content_studio` to `src/contentstudio`. The canonical entry point is now `src/contentstudio/content_studio.rs`. Legacy `src/content_studio` is a compatibility re-export.
- **ContentStudio API**: `ContentStudio` has `set_content`, `get_content`, `get_content_info`, `save_content`, and `load_content` methods. `ContentStudioContent` stores `id`, `name`, `content`, `created_at`, `updated_at`, and `workspace_id`.
- **Workspace support**: `WorkspaceManager` manages workspace directories, with `Workspace` storing `id`, `name`, `path`, and `created_at`.
- **ContentStudio is not the same as the legacy content studio**: The legacy content studio was a simple `BTreeMap`-backed in-memory store. The new ContentStudio supports workspace-based organization.
- **Important**: The old `src/content_studio` module is a compatibility shim. New code should use `src/contentstudio/content_studio.rs`.