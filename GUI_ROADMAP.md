# CADSD GUI - Feature Roadmap and Previews

## Overview

This document defines the feature roadmap for the CADSD Makepad-based GUI, organized by implementation priority. The roadmap focuses on incremental UI additions that complement existing slider/parameter controls, with careful step-wise implementation.

## Current State (Working)

### Flexible UI Shell (Makepad 2.0)

The active GUI uses a split-shell layout:

- **Sidebar**: `ScrollYView` with parameter controls, export buttons, and optimization panel. Fixed width (`Fill`), height `Fit`.
- **Main Area**: `View` with `flow: Down` containing fixed-height panels arranged vertically. Each panel has `height: Fit` to avoid circular dependency.
- **Panel Structure**:
  - Top row: Geometry controls + Simulation controls
  - Middle: Results area (impedance chart, resonance table)
  - Bottom: Optimization panel + sidebar controls

This layout ensures every container has `height: Fit` (the #1 bug preventer) and the root container has `width: Fill` with no fixed pixel widths.

**Backend Integration**:
- Parameters live in `App` struct (`src/bin/gui.rs`)
- Background simulation uses `std::thread::spawn` with result channels
- `needs_viewport_update` flag triggers `vp.update_bore()` when params change

### Already Implemented in UI State ✅
- **Hole/side hole controls**: `enable_holes`, `hole_count`, `hole_positions`, `hole_diameters` - present in `App` struct (`src/bin/gui.rs`)
- **Mouthpiece types**: `"none"`, `"reed"`, `"embouchure_hole"`, `"fipple"`, `"cup"` - present in `App` struct (`src/bin/gui.rs`)
- **Bore curve parameter**: `bore_curve: f32` - already in `App` struct (`src/bin/gui.rs`)
- **Advanced parameters**: `wall_thickness`, `temperature` - present in `App` struct (`src/bin/gui.rs`)
- **Visualization options**: `show_3d`, `show_wireframe`, `show_cross_section`, `mesh_rotation_enabled`, `mesh_rotation_speed`, `color_scheme`, `active_tab` - all present in `App` struct (`src/bin/gui.rs`)
- **Optimization controls**: `enable_optimization`, `opt_population_size`, `opt_generations`, `opt_bore_shape`, `opt_toots_input` - present in `App` struct (`src/bin/gui.rs`)
- **Loss breakdown controls**: weight sliders for fundamental, harmonics, peaks
- **Export functions**: CSV and JSON export buttons present in UI

### GUI Controls Already Working ✅
- 3D viewport with bore geometry
- Length/diameter/segments sliders
- Bore style dropdown (cone/cylinder/exponential)
- Run simulation button
- Results display (fundamental, resonances)
- Impedance spectrum chart
- Geometry summary preview
- Loss breakdown BarChart
- Export CSV/JSON buttons
- Bore curve slider for Kigali/Mbeya profiles

### Experimental Features ❌ Backend Incomplete
The following features have UI controls but incomplete backend support:

- **Mouthpiece** — UI controls present (toggle, type dropdown, length/diameter sliders); backend has no acoustic simulation support (requires new tonehole methods)
- **Finger holes** — UI controls present (toggle, count slider); backend has no tonehole simulation without new scattering junction implementation
- **AI/ML integration** — Not implemented; requires neural surrogate training pipeline
- **Time-domain synthesis** — Not implemented; requires cpal audio backend

---

### Preview Window 1: Impedance Spectrum

**Purpose**: Display frequency-dependent acoustic impedance

**UI Elements**:
- Line chart showing impedance magnitude vs frequency (20Hz-5000Hz)
- Fundamental frequency marker (vertical line)
- Resonance peaks highlighted with annotations
- Toggle: log scale / linear scale
- Toggle: show/hide fundamental marker
- Tooltip on hover: frequency, impedance value, note name

**Backend Integration**:
- Draw from `frequencies: Vec<f64>` and `impedances: Vec<f64>` state
- Use Makepad `LineChart` widget or custom drawing

### Preview Window 2: Resonance Analysis

**Purpose**: Display detected resonances with musical analysis

**UI Elements**:
- Table of resonances: frequency (Hz), note name, cent deviation, Q-factor, harmonic number
- Expandable/collapsible entries
- Toggle: show fundamental only / show all peaks
- Auto-update when simulation changes

**Backend Integration**:
- Draw from `resonance_notes: Vec<(f64, f64)>` and `frequencies: Vec<f64>` state
- Compute note names via `note_name(freq_to_note(freq))` utilities

### Preview Window 3: Loss Breakdown

**Purpose**: Show Tairua loss component decomposition

**UI Elements**:
- Progress bar or pie chart showing: fundamental loss, harmonic loss, peak alignment loss
- Weight sliders for each component (fundamental, harmonics, peaks)
- Target frequency input with current deviation display
- "Improve this component" suggestions

**Backend Integration**:
- Use `tairua_loss_value: f64` from state
- Draw from loss computation components available in `cadsd-accurate/src/loss/`

### Preview Window 4: Geometry Summary

**Purpose**: Show computed geometry metrics in real-time

**UI Elements**:
- Length (mm) display
- Bell diameter (mm) display
- Taper ratio display
- Volume calculation (mm³)
- Segment count display
- Max diameter display

**Backend Integration**:
- Draw from geometry generation functions: `geo.length()`, `geo.bellsize()`, `geo.taper_ratio()`, `geo.compute_volume()`

### Preview Window 5: Mouthpiece/Hole Editor

**Purpose**: Enable/disable finger holes and mouthpiece modifications

**UI Elements**:
- **Mouthpiece section**:
  - Toggle: enable mouthpiece
  - Dropdown: type (`none`, `reed`, `embouchure_hole`, `fipple`, `cup`)
  - Length slider (mm)
  - Diameter slider (mm)
  
- **Finger holes section**:
  - Toggle: enable holes
  - Count slider (0-12)
  - Position sliders (mm from top, one per hole)
  - Diameter sliders (mm, one per hole)

- **Visual preview**: 3D model updates in real-time as parameters change

**Backend Integration**:
- Read/write `enable_mouthpiece`, `mouthpiece_type`, `mouthpiece_length`, `mouthpiece_diameter`
- Read/write `enable_holes`, `hole_count`, `hole_positions`, `hole_diameters`
- Geometry updates via `make_bubble` and segment manipulation functions

## Complex Shape Support

### Already Available Backend Functions
- `make_bubble(pos, width, height)` - Insert bulge at position
- `stretch(factor)` - Scale length only
- `scale(factor)` - Scale all dimensions
- `move_segments_x(start, end, offset)` - Shift segment positions
- `sort_segments()` - Sort by x position
- `diameter_at_x(x)` - Interpolate diameter
- Kigali/Mbeya parametric profiles

### UI Implementation Steps (Step-Wise)

**Step 1: Bubble Controls** (Immediate)
- Add bore curve slider (-2.0 to 2.0) for exponential shaping
- Add "Insert Bubble" button with position/width/height inputs
- Real-time 3D viewport update

**Step 2: Segment Manipulation** (Near-term)
- Add "Move Segments" controls: start segment, end segment, offset
- Add "Sort Segments" button
- Real-time geometry cleanup

**Step 2.5: Kigali/Mbeya Controls** (Near-term)
- Add power exponent slider for Kigali profile
- Add profile dropdown: cone/cylinder/exponential/kigali/mbeya
- Dynamic parameter display based on selected profile

**Step 3: Hole Editor** (Mid-term)
- Enable mouthpiece controls from existing state fields
- Enable finger hole controls from existing state fields
- Real-time 3D viewport updates
- Validation: hole positions must be within [0, length], diameters must be positive

**Step 4: Advanced Geometry** (Mid-term)
- Wall thickness parameter (already in state)
- Temperature parameter (already in state, affects sound speed)
- Custom profile editor (drag control points)

## 3D Editor Features

### Available Backend Geometry Operations
- Bubble insertion at any position
- Length/stretch scaling
- Diameter scaling
- Segment repositioning
- Kigali/Mbeya parametric profiles

### UI 3D Editor Plan

**Phase 1: Interactive Geometry Controls**
- Drag handle in 3D viewport to move selected segment
- Scroll wheel to stretch/lengthen geometry
- Right-click menu: add bubble, stretch, scale
- Real-time parameter updates displayed in sidebar

**Phase 2: Cross-section View**
- Toggle: `show_cross_section` (already in state)
- Vertical plane through bore center
- Interpolated diameter at plane position
- Export cross-section data

**Phase 3: Profile Editing**
- Control points for bore curve shape
- Drag to modify curve exponent locally
- Preserve/global modify toggle
- Save modified profiles

**Phase 4: Multi-part Didgeridoo**
- Joint definition at specific x positions
- Forced diameter at joints
- Multi-segment geometry export/import

## Implementation Priority

### Immediate (Next UI Session)
1. Impedance spectrum preview window
2. Resonance analysis preview window  
3. Geometry summary preview window
4. Bore curve slider addition
5. Kigali/Mbeya profile dropdown integration

### Mid-term (2-3 UI Sessions)
6. Mouthpiece/hole editor panel
7. Loss breakdown preview window
8. Cross-section view toggle
9. Wall thickness parameter display

### Lower Priority (Feature Completeness)
10. Profile editing control points
11. Multi-part didgerijoit joint support
12. Custom profile export/import
13. Real-time synthesis preview (requires audio backend)

## Technical Notes

### Makepad-Specific Implementation

#### Preview Window Pattern
```rust
// Each preview window is a SolidView in the content area
content := SolidView{
    width: Fill
    height: Fill
    flow: Down
    spacing: 8
    padding: 10
    draw_bg +: {color: #x12161d}
    
    // Window title
    title := Label{text: "Impedance Spectrum" ...}
    
    // Chart or table
    chart := Plot{...}  // or Table{...}
    
    // Controls
    controls := View{flow: Right ...}
}
```

#### Real-time Updates
- Use `needs_viewport_update` flag from `handle_actions`
- Call `vp.update_bore()` when parameters change
- Background simulation uses `std::thread::spawn` with result channels

### State Management
All parameters live in `AppState` (or `App` struct in Makepad port):
- Geometry: length, diameters, segments, style_type, bore_curve
- Holes: enable_holes, hole_count, hole_positions, hole_diameters
- Mouthpiece: enable_mouthpiece, mouthpiece_type, mouthpiece_length, mouthpiece_diameter
- Advanced: wall_thickness, temperature
- Visualization: show_3d, show_wireframe, show_cross_section, color_scheme
- Simulation: frequencies, impedances, fundamental_freq, resonance_notes, tairua_loss_value
- Optimization: enable_optimization, opt_* fields

## Files to Update/Modify

### Primary: `src/bin/gui.rs`
- Add preview window SolidViews in `startup()` 
- Add event handling for preview window controls
- Integrate with existing `handle_actions` flow

### Supporting: `src/bin/gui.rs` (state extensions)
- Extend `App` struct with preview window state flags
- Add geometry computation functions for display
- Wire backend functions to UI controls

### Documentation Updates
- `UI_REQUIREMENTS.md` - Update with preview windows section
- `QUICK_START_GUI.md` - Add preview window usage
- `GUI_SOLUTION.md` - Document step-wise implementation

## Success Criteria

### ✅ Verified Working
- [x] Impedance spectrum renders from simulation data
- [x] Resonance table populates with note names
- [x] Geometry summary shows computed metrics (length, bell, volume, taper, segments, max diameter)
- [x] Bore curve slider modifies 3D model in real-time
- [x] Mouthpiece/hole toggles enable/disable without crash
- [x] Loss breakdown BarChart displays 4 components
- [x] Export CSV/JSON buttons present and functional
- [x] Profile selection in simulation thread (Cone/Kigali/Mbeya)
- [x] 3D viewport with orbit/zoom controls (XrCamera)
- [x] Background simulation thread with result channels
- [x] Bubble insertion works without geometry corruption (Geo::make_bubble tested)
- [x] Segment manipulation works (move/sort)
- [x] Optimization backend available (Nuevolution with evolve())

### ❌ Backend Incomplete (UI present but no simulation support)
- [ ] Mouthpiece acoustic effects (requires new tonehole methods)
- [ ] Finger hole acoustic effects (requires new scattering junction)
- [ ] AI/ML integration (neural surrogate not implemented)
- [ ] Time-domain synthesis (CPAL backend not available)

### To Be Implemented
- [ ] 3D editor drag handles modify geometry
- [ ] Multi-part joint definition works
- [ ] Custom profile save/load
- [ ] Export geometry with modifications

## 10. Current Makepad GUI State and Future UI Needs (Addendum)

This addendum records the actual state of the new Makepad GUI and the future features that must be planned for now so the UI does not need to be rebuilt later. The older roadmap sections above are kept as historical context.

### What the current GUI already has ✅

The current Makepad GUI is the active UI. It already contains:

- Geometry controls: length, top diameter, bell diameter, segments, bore curve, and a bore-style dropdown.
- A 3-D viewport that redraws the bore in real time with orbit/zoom controls.
- A run button that starts a background simulation thread.
- An impedance spectrum chart (LineChart widget) that renders simulation data.
- A resonance analysis display showing frequency and impedance values.
- A geometry summary showing length, bell diameter, volume, taper ratio, segment count, and max diameter.
- A loss breakdown BarChart showing 4 components: fundamental, harmonics, peaks, total.
- Export CSV and JSON buttons for geometry data.
- Bubble insertion controls for bubble position, width and height.
- Finger hole controls (experimental — backend acoustic simulation not implemented).
- Mouthpiece controls for length and diameter.
- Optimization controls (evolutionary optimizer with Nuevolution).

### What is missing but must be planned for now ⚠️

The following features are research-driven future work. They are not yet in the backend, but the UI should reserve space for them now:

- Full acoustic simulation resolution for finger holes and mouthpiece (UI controls present but backend incomplete)
- Cross-section view toggle and 2D plot
- Phase overlay on impedance chart showing real/imaginary components
- Bent-shape correction preview showing effective length
- Differentiable TLM / Adam optimization backend
- Neural surrogate training panel (ML backend)
- Time-domain synthesis with audio output (CPAL backend)
- Multi-fidelity validation with FDTD/FEM backends

### Design rule

Do not change the existing layout just to fit one new backend feature. Add new panels or extend existing panels with stable IDs. The GUI should be a shell that can accept new backend modules later.

### Future UI requirements

- Reserve a sidebar area for simulation method and loss-component controls.
- Keep the impedance chart capable of showing magnitude and phase.
- Keep the resonance panel able to show a structured table later.
- Keep a cross-section panel slot even if the backend does not support it yet.
- Keep an optimizer panel slot even if the optimizer is not wired yet.

### Future backend features that need UI slots

| Future backend feature | UI slot to reserve now | Why it matters |
|------------------------|------------------------|----------------|
| Mouthpiece / finger holes | Sidebar panel | Backend state present but simulation resolution incomplete |
| Loss breakdown | Optimizer panel | Future loss component visualization |
| Cross-section view | Main area panel | Future 2-D backend |
| Phase overlay | Impedance chart | Future complex impedance |
| Differentiable TLM | Optimizer panel | Future gradient-based design |
| Neural surrogate | Training panel | Future ML backend |
| Time-domain synthesis | Audio panel | Future `cpal` backend |
| Multi-fidelity validation | Validation panel | Future FDTD/FEM backend |

