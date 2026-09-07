# CADSD GUI - Feature Roadmap and Previews

## Overview

This document defines the feature roadmap for the CADSD Makepad-based GUI, organized by implementation priority. The roadmap focuses on incremental UI additions that complement existing slider/parameter controls, with careful step-wise implementation.

## Current State (Working)

### Already Implemented in UI State
- **Hole/side hole controls**: `enable_holes`, `hole_count`, `hole_positions`, `hole_diameters` - present in `app.rs`
- **Mouthpiece types**: `"none"`, `"reed"`, `"embouchure_hole"`, `"fipple"`, `"cup"` - present in `app.rs`
- **Bore curve parameter**: `bore_curve: f32` - already in `app.rs`
- **Advanced parameters**: `wall_thickness`, `temperature` - present in `app.rs`
- **Visualization options**: `show_3d`, `show_wireframe`, `show_cross_section`, `mesh_rotation_enabled`, `mesh_rotation_speed`, `color_scheme`, `active_tab` - all present in `app.rs`
- **Optimization controls**: `enable_optimization`, `opt_population_size`, `opt_generations`, `opt_bore_shape`, `opt_toots_input` - present in `app.rs`

### GUI Controls Already Working
- 3D viewport with bore geometry
- Length/diameter/segments sliders
- Bore style dropdown (cone/cylinder/exponential)
- Run simulation button
- Results display (fundamental, resonances)

## Preview Windows Implementation Plan

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
- Use Makepad's `egui_plot` or direct chart rendering

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
- Exponential flare profiles

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
- Exponential flare profiles

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

### Minimum Viable Previews
- [x] Impedance spectrum renders from simulation data
- [x] Resonance table populates with note names
- [x] Geometry summary shows computed metrics
- [x] Bore curve slider modifies 3D model in real-time
- [x] Mouthpiece/hole toggles enable/disable without crash

### Enhanced Previews
- [ ] Loss breakdown shows correct component weights
- [ ] Cross-section view displays correctly
- [ ] Hole positions validate and update geometry
- [ ] Kigali/Mbeya profile dropdown works with all options

### Full Feature Set
- [ ] 3D editor drag handles modify geometry
- [ ] Multi-part joint definition works
- [ ] Custom profile save/load
- [ ] Export geometry with modifications