# CADSD GUI - Complete Feature Requirements

## Overview

This document defines the full UI requirements for the CADSD Makepad-based GUI, derived from the research methodology and backend capabilities. The UI is designed as a research tool for didgeridoo design optimization, incorporating all parameters and features from the `cadsd-accurate` backend.

---

## Current Implementation State

### Working Features
- 3D viewport with bore geometry rendering (orbit/zoom)
- Length, top diameter, bell diameter, segments sliders
- Bore style dropdown (Cone, Cylinder, Exponential)
- Run simulation button with threaded execution
- Results display (fundamental frequency, resonance count)
- Makepad-based GPU rendering

### State Variables Already Present (from app.rs)
The following parameters already exist in the backend state but may not have UI controls:

**Geometry State:**
- `length`, `top_diameter`, `bottom_diameter`, `segments` - sliders exist
- `style_type`, `bore_curve` - dropdown/slider exist
- `make_bubble()`, `stretch()`, `scale()`, `move_segments_x()` - backend functions available

**Mouthpiece State:**
- `enable_mouthpiece` - toggle needed
- `mouthpiece_type` - dropdown: "none", "reed", "embouchure_hole", "fipple", "cup"
- `mouthpiece_length`, `mouthpiece_diameter` - sliders needed

**Hole State:**
- `enable_holes` - toggle needed
- `hole_count` - slider (0-12)
- `hole_positions` - sliders per hole (mm from top)
- `hole_diameters` - sliders per hole (mm)

**Advanced Parameters:**
- `wall_thickness` - slider (already in state)
- `temperature` - slider (already in state)

**Visualization Options:**
- `show_3d`, `show_wireframe`, `show_cross_section` - toggles
- `mesh_rotation_enabled`, `mesh_rotation_speed` - controls
- `color_scheme` - dropdown: "wood", "metal", "custom"

**Simulation Results:**
- `frequencies`, `impedances` - for preview windows
- `fundamental_freq` - display
- `resonance_notes` - table
- `tairua_loss_value` - display

---

## Preview Windows

### 1. Impedance Spectrum Preview

**Purpose**: Display frequency-dependent acoustic impedance as a line chart

**UI Elements**:
- Line chart: X-axis frequency (20Hz-5000Hz), Y-axis impedance magnitude
- Toggle: log scale / linear scale
- Fundamental frequency marker (vertical line, different color)
- Resonance peak annotations on hover
- Export chart as PNG button

**Data Source**: `frequencies: Vec<f64>`, `impedances: Vec<f64>`

**Implementation**: Use Makepad's chart widgets or custom drawing

---

### 2. Resonance Analysis Preview

**Purpose**: Display detected resonances with musical note analysis

**UI Elements**:
- Collapsible table with columns:
  - Peak # (1, 2, 3...)
  - Frequency (Hz)
  - Note Name (e.g., "D3", "A4+15¢")
  - Cent Deviation (e.g., "+15¢", "-38¢")
  - Harmonic # (fundamental=1, 2, 3...)
- Toggle: show all / fundamental only
- Highlight rows by harmonic type (even=open tuning, odd=closed tuning)

**Data Source**: `resonance_notes: Vec<(f64, f64)>`, `frequencies`

**Conversion**: Use `note_name(freq_to_note(freq))` utilities

---

### 3. Geometry Summary Preview

**Purpose**: Show computed geometry metrics in real-time

**UI Elements**:
- Length: display in mm
- Bell diameter: display in mm  
- Taper ratio: display (max_d / min_d)
- Volume: display in mm³ (computed via trapezoidal rule)
- Segment count: display
- Max diameter: display in mm
- Bore curve: display current value

**Data Source**: `geo.length()`, `geo.bellsize()`, `geo.taper_ratio()`, `geo.compute_volume()`, `geo.get_max_d()`

---

### 4. Loss Breakdown Preview

**Purpose**: Show Tairua loss component decomposition

**UI Elements**:
- Pie chart or stacked bar showing:
  - Fundamental frequency loss (weight: `weight_fundamental`)
  - Harmonic alignment loss (weight: `weight_harmonics`)
  - Peak alignment loss (weight: `weight_peaks`)
- Weight sliders for each component
- Target frequency display with current deviation
- Total Tairua loss value (0-10 scale)

**Data Source**: `tairua_loss_value`, loss computation internals

---

### 5. Mouthpiece/Hole Editor Preview

**Purpose**: Configure mouthpiece and finger hole modifications

**UI Elements**:

**Mouthpiece Section**:
- Toggle: Enable mouthpiece
- Dropdown: Type ("none", "reed", "embouchure_hole", "fipple", "cup")
- Slider: Length (mm) - visible when enabled
- Slider: Diameter (mm) - visible when enabled

**Finger Holes Section**:
- Toggle: Enable holes
- Slider: Hole count (0-12) - visible when enabled
- Per-hole controls (dynamic, shown for each hole):
  - Slider: Position (mm from top)
  - Slider: Diameter (mm)
- Validation: positions must be in [0, length], diameters > 0

**3D Preview**: Model updates in real-time as parameters change

---

### 6. Cross-Section View

**Purpose**: Show bore profile as 2D cross-section

**UI Elements**:
- 2D plot showing diameter vs position along bore
- Axis labels: diameter (mm), position (mm)
- Grid lines for scale reference
- Current geometry profile as line
- Optional: show multiple profiles overlaid (for optimization comparisons)

**Implementation**: Simple 2D plot widget

---

## Geometry Controls (Extended)

### Basic Parameters (Existing)
- Length slider: 500-3000mm
- Top diameter slider: 10-100mm  
- Bell diameter slider: 20-150mm
- Segments slider: 5-200

### Bore Profile Selection (Extended)
- Dropdown: Cone, Cylinder, Exponential, Kigali, Mbeya
- Bore curve slider: -2.0 to 2.0 (for exponential/Kigali/Mbeya)

### Profile-Specific Parameters

**Kigali Profile**:
- Power exponent slider
- Q1, Q2, Q3: quarter length ratios
- D1, D2, D3: diameters at quarter points

**Mbeya Profile**:
- Power exponent slider
- Straight section length ratio
- Opening section length ratio
- Bell section length ratio

**Exponential Profile**:
- Base diameter
- Bell ratio
- Curve parameter

### Advanced Geometry Operations

**Bubble Insertion**:
- Position slider (mm from top)
- Width slider (mm)
- Height slider (mm)
- "Add Bubble" button
- "Remove Last Bubble" button

**Segment Manipulation**:
- Start segment index slider
- End segment index slider
- Offset slider (mm)
- "Move Segments" button
- "Sort Segments" button

---

## Simulation Parameters

### Simulation Method
- Dropdown: TLM (Python), TLM (Cython), Digital Waveguide, Complex Impedance
- Frequency range: fmin/fmax sliders (20-5000Hz)
- Grid resolution: slider (1-100 cents per semitone)

### Loss Modeling
- Viscothermal losses: toggle
- Radiation impedance model: dropdown (Geipel, spherical, placeholder)
- Boundary conditions: radio buttons (open end / closed end)

### Advanced Settings
- Wall thickness: slider (mm)
- Temperature: slider (°C, affects sound speed)
- Parallel jobs: slider (1-4)

---

## Optimization Controls

### Target Sound Definition
- Target fundamental frequency: input field or note selector
- Target notes: multi-select for toots/overtones
- Overtone series: checkboxes for harmonics 2-10
- Bore shape preference: Any, Cylindrical, Conical, Flared

### Evolution Parameters
- Population size: slider (10-200)
- Generations: slider (1-1000)
- Mutation rate: slider (0.0-1.0)
- Crossover rate: slider (0.0-1.0)
- Elite size: slider (1-20)
- Mutation strategy: Gaussian, PrimeSequence, Uniform

### Optimization Controls
- Start/Stop/Pause buttons
- Progress bar: generation count, current best fitness
- Best genome display: parameters, loss value, geometry preview
- Convergence indicators

---

## Visualization Options

### 3D Viewport Controls
- Orbit: mouse drag
- Zoom: scroll wheel
- Pan: right-click drag
- Reset view button

### Display Toggles
- Show 3D model: toggle
- Show wireframe: toggle
- Show cross-section: toggle
- Show ground plane: toggle
- Auto-update: toggle (real-time geometry updates)
- Mesh rotation: toggle
- Rotation speed: slider

### Color Schemes
- Dropdown: Wood, Metal, Custom
- Custom: color pickers for primary, secondary, highlight

---

## Export and Configuration

### Export Functions
- Export geometry: CSV, JSON, Excel
- Export impedance data: CSV
- Export results report: text, JSON
- Export chart: PNG

### Configuration Management
- Save configuration with name
- Load saved configurations
- Preset library: Traditional, Modern, Experimental
- Import/Export all settings

---

## Backend Feature Reference

### Geo Module (`src/geo/mod.rs`)
```rust
Geo::make_cone(length, d1, d2, n_segments)     // Conical bore
Geo::make_cylinder(length, d, n_segments)       // Cylindrical bore  
Geo::make_exponential(length, d1, d2, n, power) // Exponential flare
Geo::make_kigali(length, top, bottom, power, n) // Kigali profile
Geo::make_mbeya(length, top, bottom, power, n)  // Mbeya profile
geo.make_bubble(pos, width, height)            // Insert bulge
geo.stretch(factor)                            // Scale length only
geo.scale(factor)                              // Scale all dimensions
geo.move_segments_x(start, end, offset)        // Shift segments
geo.diameter_at_x(x)                           // Interpolate diameter
geo.compute_volume()                           // Calculate volume
geo.taper_ratio()                              // Calculate taper ratio
geo.length()                                   // Total length
geo.bellsize()                                // Bell diameter
geo.get_max_d()                               // Maximum diameter
```

### Sim Module (`src/sim/mod.rs`)
```rust
acoustical_simulation(geo, frequencies, method) // Main simulation
get_log_simulation_frequencies()               // Log frequency grid
get_fundamental(geo, method, min_peak_f)        // Extract fundamental
compute_ground_spektrum(geo, method)           // Full spectrum
```

### Loss Module (`src/loss/mod.rs`)
```rust
TairuaLoss                          // Main composite loss
FundamentalFrequencyLoss            // Target frequency loss
GeometricLoss                      // Length/taper loss
MultiObjectiveLoss                 // Combined loss
DidgeLabLoss                      // Inverse design loss
```

### Evo Module (`src/evo/mod.rs`)
```rust
Nuevolution                         // Evolutionary optimizer
GeoGenome                          // Genome representation
TargetSound                       // Target sound definition
MutationOperator::Gaussian/Uniform/RandomResetting
CrossoverOperator::Uniform/SinglePoint/TwoPoint
BoreShapePreference::Any/Cylindrical/Conical/Flared
```

### Conv Module (`src/conv/mod.rs`)
```rust
note_to_freq(note)     // Note to frequency
freq_to_note(freq)    // Frequency to note
note_name(note)       // Note name string
```

---

## Implementation Priority

### Phase 1: Preview Windows
1. Impedance spectrum chart
2. Resonance analysis table
3. Geometry summary display
4. Loss breakdown preview
5. Cross-section view

### Phase 2: Extended Geometry Controls
1. Bore curve slider (exponential profiles)
2. Kigali/Mbeya profile dropdown integration
3. Bubble insertion controls
4. Segment manipulation controls
5. Profile-specific parameter panels

### Phase 3: Mouthpiece and Holes
1. Mouthpiece toggle and controls
2. Hole toggle and count slider
3. Per-hole position/diameter sliders
4. Real-time 3D updates
5. Validation and error handling

### Phase 4: Advanced Features
1. Optimization UI integration
2. Configuration save/load
3. Export functions
4. Preset library
5. Comparison tools

---

## Makepad Implementation Notes

### Widget Patterns
```rust
// Dropdown selection
if let Some(v) = self.ui.drop_down(cx, ids!(style_dropdown)).selected(actions) {
    style = v as u32;
}

// Slider changes
if let Some(v) = self.ui.slider(cx, ids!(length_slider)).slided(actions) {
    length = v;
}

// Button click
if self.ui.button(cx, ids!(run_button)).clicked(actions) {
    // handle
}

// Label update
self.ui.label(cx, ids!(value_label)).set_text(cx, &format!("{:.1}", value));
```

### Thread Safety
```rust
std::thread::spawn(move || {
    // Simulation runs on background thread
    // Results sent back via channels
});
```

### State Updates
```rust
if needs_viewport_update {
    if let Some(mut vp) = self.ui.widget(cx, ids!(viewport)).borrow_mut::<BoreViewport>() {
        vp.update_bore(cx, length, top, bell, style, segments);
    }
}
```

---

## Files Reference

### Primary Implementation
- `src/bin/gui.rs` - Main Makepad GUI (560+ lines)
  - App with Makepad macros
  - BoreViewport widget with DrawPhysMesh
  - Event handling for all controls
  - Background simulation threading

### Backend Modules
- `cadsd-accurate/src/geo/mod.rs` - Geometry generation
- `cadsd-accurate/src/sim/mod.rs` - Acoustic simulation
- `cadsd-accurate/src/conv/mod.rs` - Conversions
- `cadsd-accurate/src/loss/mod.rs` - Loss functions
- `cadsd-accurate/src/evo/mod.rs` - Evolutionary optimization
- `cadsd-accurate/src/analysis/mod.rs` - Analysis tools

### Documentation
- `GUI_ROADMAP.md` - Feature roadmap with preview windows
- `GUI_COMPLETE.md` - Makepad implementation guide
- `QUICK_START_GUI.md` - Usage guide
- `DESIGN_NOTES.md` - Backend architecture