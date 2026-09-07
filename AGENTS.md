# Rust CADSD Agent Manager Documentation

This document describes the agent management system used in the Rust CADSD project for orchestrating computational tasks.

## Agent Manager Overview

The Agent Manager system allows us to:
- Launch parallel simulation tasks
- Manage evolutionary optimization runs
- Handle complex acoustic modeling operations

## Available Agents

1. **Simulation Agent**
   - Handles acoustic simulations using different strategies (TLM, Digital Waveguide, Complex Impedance)
   - Parameters:
     * Geometry data
     * Simulation strategy
     * Frequency range

2. **Evolutionary Optimizer Agent**
   - Manages genetic algorithm operations
   - Parameters:
     * Loss function
     * Population size
     * Mutation strategy
     * Crossover rate

3. **Neural Network Explorer Agent** (planned)
   - Will handle prime-number indexed networks and digital waveguide models

## Agent Manager UI Migration

Makepad-based UI replaced the broken egui+Bevy integration. Key changes:

- Removed `bevy` and `egui` dependencies from Cargo.toml
- Added `makepad-widgets` and `makepad-xr` as optional features
- Rewrote `src/bin/gui.rs` with correct Makepad API patterns:
  - `app_main!(App)` macro with `#[derive(Script, ScriptHook)]` on App struct
  - `#[source]` field removed from App (only widget structs use it)
  - `script_mod!()` macro with proper widget/DrawPhysMesh/BoreViewport registration
  - 3D viewport uses `DrawPhysMesh` shader (based on box3d example) and `BoreViewport` widget with `draw_scene()` method
- `AppState` moved to `src/lib.rs` backend, no Bevy guards needed
- `BgTaskResult` channels use `mpsc` instead of Bevy's `EventWriter`

## Agent Orchestration

All agent tasks should be initiated through the Agent Manager API. For example:

```rust
// Launch parallel simulation tasks
agent_manager! {
  mode: "local",
  tasks: [
    {
      prompt: "Run 4 parallel simulations with different strategies"
    }
  ]
}

// Start evolutionary optimization
agent_manager! {
  tasks: [
    {
      prompt: "Optimize bore shape for fundamental frequency 120Hz"
    }
  ]
}
```

## Key Protocols

1. Always specify clear prompts for agent goals
2. Use mutation strategies carefully (Gaussian vs PrimeSequence)
3. Monitor CPU usage with parallel tasks (limit to 4 jobs max)
4. Verify results with accurate crate implementations

## Example Workflows

1. **Digital Waveguide Simulation Workflow**
   - Launch simulation agent with Waveguide strategy
   - Record time-series impedance data
   - Analyze with neural network explorer

2. **Prime Indexed Optimization**
   - Configure EvolutionaryOptimizer with PrimeSequence mutation
   - Run multiple generations
   - Export best results

## Technical Notes

- All agents run in isolated Rust environments
- Communication happens through structured prompts
- Results are returned in structured format (JSON or Rust structs)
- GUI compiled with `--features gui --bin cadsd-gui` using Makepad GPU-driven UI
- Backend (`cadsd-accurate` crate) remains unchanged and fully functional

## Future Development

1. Full Digital Waveguide implementation
2. Prime-Indexed Neural Network exploration
3. GPU acceleration for parallel tasks
4. Expand Makepad controls (optimization parameters, export options)

Report any issues at https://github.com/Kilo-Org/kilocode/issues