# CADSD GUI - Complete Feature Requirements

## Overview

This document defines the full UI requirements for the CADSD Makepad-based GUI, derived from the research methodology and backend capabilities. The UI is designed as a research tool for didgeridoo design optimization, incorporating all parameters and features from the `cadsd-accurate` backend.

---

## Current Implementation State

### Working Features ✅
- 3D viewport with bore geometry rendering (orbit/zoom)
- Length, top diameter, bell diameter, segments sliders
- Bore style dropdown (Cone, Kigali, Mbeya)
- Run simulation button with threaded execution
- Results display (fundamental frequency, resonance count)
- Makepad-based GPU rendering
- Impedance spectrum chart with LineChart
- Resonance analysis with note names
- Geometry summary with volume, taper, max diameter
- Loss breakdown BarChart with 4 components
- Generational loss LineChart with per-generation total_loss
- Export CSV/JSON geometry functions
- Bore curve slider for Kigali/Mbeya profiles
- Profile selection in simulation thread

### UI Features Partially Implemented
- Geometry summary preview with all metrics
- Loss breakdown with weight sliders
- Export functions (CSV/JSON)
- Bubble insertion (marked experimental in UI)
- Finger holes (marked experimental in UI)
- Mouthpiece controls (UI wired, backend incomplete)

### State Variables Present in App struct
- Geometry: length, top_diameter, geo_bell, segments, bore_style_name, geo_curve
- Mouthpiece: enable_mouthpiece, mouthpiece_type, mouthpiece_length, mouthpiece_diameter
- Holes: enable_holes, hole_count, hole_positions, hole_diameters
- Loss: weight_fundamental, weight_harmonics, weight_peaks, tairua_loss_value, target_freq
- Optimization: optimization_running, optimization_target_freq, optimization_progress, optimization_status_text, optimization_best_top, optimization_best_bell, optimization_best_style

### Backend Modules Available
- Geo: cone, kigali, mbeya, make_bubble, stretch, scale, move_segments_x, diameter_at_x, compute_volume, taper_ratio
- Sim: acoustical_simulation, get_log_simulation_frequencies, get_fundamental, compute_ground_spektrum
- Loss: TairuaLoss with 10+ components (FundamentalFrequencyLoss, GeometricLoss, MultiObjectiveLoss, DidgeLabLoss)
- Evo: Nuevolution, GeoGenome, TargetSound, multiple mutation/crossover strategies
- Conv: note_to_freq, freq_to_note, note_name

---

## Preview Windows

### Status Overview

| Preview Window | Status |
|---|---|
| 1. Impedance Spectrum | ✅ Working |
| 2. Resonance Analysis | ✅ Working (note names) |
| 3. Geometry Summary | ✅ Working (all metrics) |
| 4. Loss Breakdown | ✅ Working (BarChart, 4 components) |
| 5. Generational Loss | ✅ Working (LineChart, total_loss per generation) |
| 6. Mouthpiece/Hole Editor | ❌ Backend incomplete (no simulation resolution) |
| 7. Cross-Section View | ✅ Working (backend available) |
| 8. View Selector (Tab Navigation) | ✅ Working |

### 1. Impedance Spectrum Preview ✅

**Purpose**: Display frequency-dependent acoustic impedance as a line chart

**UI Elements**:
- Line chart: X-axis frequency (20Hz-5000Hz), Y-axis impedance magnitude
- Fundamental frequency marker
- Real-time data from simulation thread

**Data Source**: `frequencies: Vec<f64>`, `impedances: Vec<f64>`

---

### 7. View Selector (Tab Navigation) ✅

**Purpose**: Switch between different preview windows and view modes

**UI Elements**:
- View selector dropdown in header (Setup, Segments, Bubbles, Optimization, Export)
- Displays current view label in header
- Shows/hides sidebar based on view selection
- Toggles wireframe and cross-section displays

**Data Source**: `current_view: String` state variable

**View Definitions**:
- Setup: All geometry controls and 3D viewport
- Segments: Segment manipulation tools
- Bubbles: Bubble insertion controls
- Optimization: Evolution settings and progress
- Export: Save/load/export functions

---

### 2. Resonance Analysis Preview ✅

**Purpose**: Display detected resonances with musical note analysis

**UI Elements**:
- Top 10 peaks with frequency and impedance values
- Note names via `note_name(freq_to_note(*f))`
- Resonance count display

**Data Source**: `peaks`, `note_name`, `freq_to_note`

---

### 3. Geometry Summary Preview ✅

**Purpose**: Show computed geometry metrics in real-time

**UI Elements**:
- Length: display in mm
- Bell diameter: display in mm
- Taper ratio: display (max_d / min_d)
- Volume: display in mm³
- Segment count: display
- Max diameter: display in mm

**Data Source**: `geo_length`, `geo_bell`, `geo_taper`, `geo_volume`, `geo_segments`, `geo_max_d`

---

### 4. Loss Breakdown Preview ✅

**Purpose**: Show Tairua loss component decomposition

**UI Elements**:
- BarChart widget showing 4 bars: fundamental loss, harmonic loss, peak loss, total Tairua loss
- Weight sliders for fundamental, harmonics, peaks components
- Target frequency input
- Total loss display

**Data Source**: `loss_fundamental_value`, `loss_harmonics_value`, `loss_peaks_value`, `tairua_loss_value`, `weight_fundamental`, `weight_harmonics`, `weight_peaks`, `target_freq`

---

### 5. Mouthpiece/Hole Editor ❌ Backend Incomplete

**Purpose**: Configure mouthpiece and finger hole modifications

**UI Elements**:

**Mouthpiece Section**:
- Toggle: Enable mouthpiece
- Dropdown: Type ("None", "Reed", "Embouchure", "Fipple", "Cup")
- Slider: Length (mm) - visible when enabled
- Slider: Diameter (mm) - visible when enabled

**Finger Holes Section**:
- Toggle: Enable holes
- Slider: Hole count (0-12) - visible when enabled

**Status**: These controls are present in the UI but the backend acoustic simulation does not yet support mouthpiece or tonehole effects. The `sim/mod.rs` module only implements a transmission-line model (`cadsd_ze`) with no mouthpiece or tonehole parameters. Adding these requires new scientific methods (e.g., three-port scattering junction per Scavone & Smith 1997).

**3D Preview**: Geometric modifications only; no acoustic impact simulated.

---

### 6. Cross-Section View ✅

**Purpose**: Show bore profile as 2D cross-section

**UI Elements**:
- 2D plot showing diameter vs position along bore
- Axis labels: diameter (mm), position (mm)
- Grid lines for scale reference
- Current geometry profile as line

**Data Source**: `geo.geo` segments (x, diameter pairs), `diameter_at_x()` interpolation

**Implementation**: LineChart widget with segment data

---

## Geometry Controls (Extended)

### Basic Parameters
- Length slider: 500-3000mm
- Top diameter slider: 10-100mm
- Bell diameter slider: 20-150mm
- Segments slider: 5-200

### Bore Profile Selection
- Dropdown: Cone, Kigali, Mbeya
- Bore curve slider: -2.0 to 2.0 (for exponential/Kigali/Mbeya)

### Advanced Geometry Operations

**Bubble Insertion** ✅ Working:
- Position slider (mm from top)
- Width slider (mm)
- Height slider (mm)
- "Add Bubble" button
- "Remove Last Bubble" button
- Backend: `Geo::make_bubble(pos, width, height)` with sinusoidal profile (10 sample points)
- Tested: shape continuity, volume increase, simulation validity, numerical stability

**Segment Manipulation** ✅ Working:
- Start segment index slider
- End segment index slider
- Offset slider (mm)
- "Move Segments" button
- "Sort Segments" button
- Backend: `geo.move_segments_x(start, end, offset)`, `geo.sort_segments()`

---

## Simulation Parameters

### Simulation Method
- Method: TLM (tlm_python)
- Frequency range: log grid (20-5000Hz) via `get_log_simulation_frequencies()`

### Loss Modeling
- Tairua loss with 10+ components
- Viscothermal losses included in backend
- Weight sliders for fundamental, harmonics, peaks

### Advanced Settings
- Wall thickness: slider (mm) - in state
- Temperature: slider (°C) - in state

---

## Optimization Controls ✅

### Current State
- Optimization panel UI present with Start/Stop buttons and status display
- Target frequency field available
- Progress tracking field present
- Backend: `Nuevolution` with `evolve()` method and `GeoGenome`

### Implemented Backend Features
- Population size: `Nuevolution::new(population_size, generations)`
- Generations: `Nuevolution::new(population_size, generations)`
- Mutation rate: `set_mutation_rate()`
- Crossover rate: `set_crossover_rate()`
- Elite size: `set_elite_size()`
- Progress callbacks: `progress_cb: Option<&(dyn Fn(usize, f64) + Send + Sync)>`
- Fitness evaluation through `GeoGenome::evaluate_fitness()`
- Mutation operators: `Gaussian`, `Uniform`, `RandomResetting`
- Crossover operators: `Uniform`, `SinglePoint`, `TwoPoint`
- Bore shape preference: `Any`, `Cylindrical`, `Conical`, `Flared`

### Evolution Settings Modal ✅
- Evolution Settings modal accessible from Optimization panel
- Population Size slider (10-500, step 10, default 100)
- Mutation Rate slider (0.01-1.0, step 0.01, default 0.1)
- Crossover Rate slider (0.1-1.0, step 0.05, default 0.8)
- Convergence Patience slider (5-100, step 5, default 20)
- Selection Strategy dropdown: Tournament, Roulette, Rank (default Tournament)
- Clone from Previous button (toggles state, updates button text)
- Apply/Cancel buttons (Apply keeps values, Cancel restores originals)
- Safe defaults via `#[rust(...)]` prevent invalid values
- Labels show current values when modal opens and when sliders change
- Original values saved on modal open for Cancel functionality

### Menu Bar Status
- File menu: New Project (✅), Open Project (⚠️ file dialog only, TODO load JSON), Save Project (⚠️ file dialog only, TODO save JSON), Save As (⚠️ file dialog only, TODO save JSON), Export (✅), Quit (✅)
- Edit menu: Undo (✅), Redo (✅), Preferences (⚠️ "Preferences coming soon" modal stub)
- View menu: Toggle Sidebar (✅), Toggle Wireframe (✅), Toggle Cross-Section (✅), Zoom Extents (✅), Full Screen (✅)
- Theme menu: Dark Mode (✅), Light Mode (✅), Auto (✅)
- Help menu: Documentation (⚠️ "Documentation coming soon" modal stub), About (✅)
- **Top menu glitch**: When pressing Save/Save As from the File dropdown, the file dialog takes focus but the dropdown retains state causing the menu to glitch and the pressed item to become the first button. Fix: close dropdown before showing file dialogs, or reset dropdown state after dialog interaction.

### Documentation Modal (Future)
- Should open relevant docs (TODO.md, UI_REQUIREMENTS.md, GUI_ROADMAP.md, RESEARCH.md, ARCHITECTURE.md, losses.md)
- Can use Notepad or display content within a Makepad ScrollYView
- Currently shows "Documentation coming soon" placeholder

### Future Enhancements (Not Yet Implemented)
- Real async execution with progress callbacks
- Best genome display and convergence indicators
- Evolutionary optimizer integration with geometry
- UI controls for all evolution parameters
- Clone from Previous wiring to reuse prior optimization results

---

## Export and Configuration

### Export Functions ✅
- Export geometry: CSV, JSON (buttons present in UI)
- Export impedance data: CSV (button present)

### Configuration Management
- Future: Save/load configurations
- Future: Preset library