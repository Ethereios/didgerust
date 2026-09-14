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

---

## 11. DidgeLab Workflow Review and App Menu Architecture 

[certain] This revision corrects the prior "tec" typo: the top menu is **File, Edit, Theme, Help** (standard desktop app structure), NOT a "Tech" section. The analysis below maps each DidgeLab web-feature to our app's correct placement: main window, auxiliary window, or future work.

### 11.1 Current GUI Feature Inventory (verified against src/bin/gui.rs)

The minimum working GUI window currently contains:

| Component | Location in GUI | Status |
|---|---|---|
| Header (title + hint) | SolidView, top 44px | ✅ Ready for menu bar replacement |
| Sidebar | SolidView, 320px left | ✅ Complete |
| Geometry controls | Sidebar | ✅ Length, top/bell diameter, segments, bore style, bore curve |
| Bubble insertion | Sidebar | ✅ Position/width/height inputs + add/remove buttons |
| Segment editor | Sidebar (toggle) | ✅ Start/end/offset + sort |
| Mouthpiece controls | Sidebar | ✅ UI only (backend incomplete — tonehole simulation) |
| Finger hole controls | Sidebar | ✅ UI only (backend incomplete — scattering junction) |
| Loss breakdown | Sidebar | ✅ Weight sliders + BarChart + target freq |
| Optimization panel | Sidebar | ✅ Start/Stop + async thread + Nuevolution backend |
| 3D viewport | Main area | ✅ BoreViewport (orbit/zoom) |
| Impedance spectrum | Main area | ✅ LineChart |
| Cross-section view | Main area | ✅ Diameter vs position LineChart |
| Geometry summary | Main area | ✅ Grid of: length, bell, volume, taper, segments, max d |
| Resonance analysis | Main area | ✅ DataGrid (10 rows) + resonance_list label |
| Export CSV/JSON | Sidebar | ✅ rfd dialog |

**State fields confirmed** (App struct, lines 1192–1289): impedance_data, fundamental_freq, peaks, geo_*, bubbles, mouthpiece_*, hole_*, loss_*, optimization_*, cross_section_data — all verified present.

### 11.2 Feature Placement Decision: Main Window vs Auxiliary Window vs Future

[certain] The following table maps each DidgeLab feature to its correct location. This is a design decision, not yet implemented.

| DidgeLab Feature | Placement | Rationale |
|---|---|---|
| **Acoustic Target** (multi-peak, scale tuning, weights) | **Auxiliary window** | Single target freq slider in sidebar covers basic case; full target builder (per-peak note, impedance amplitude, scale tuning) needs dedicated dialog to avoid cluttering sidebar |
| **Shape Configuration with ranges** | **Main window** (extend sidebar) | Min/max sliders can extend existing geometry controls in sidebar |
| **Forced diameters for joints** | **Main window** (viewport overlay) | Interacts directly with 3D viewport; can be inline overlay |
| **Evolution settings** (clone, name, duration) | **Auxiliary window** | Clone from previous run + name + duration tiers need a configuration dialog, not inline sliders |
| **Real-time loss chart over generations** | **Main window** (replace/augment BarChart) | Can extend the existing loss breakdown BarChart into a LineChart with generational history |
| **Multi-shape results with tuning table** | **Auxiliary window** | 10 best shapes + per-shape tuning table (cents) needs dedicated result browser |
| **Export pipeline** | **Main window** (extend File menu) | CSV/JSON already in sidebar; Blender/3D print prep as File → Export submenu |
| **Advanced 3D editor** | **Main window** (viewport) | Drag handles, control points integrate directly with BoreViewport; right-click context menu |
| **AI/ML features** | **Auxiliary window** | Neural predictor training and time-domain synthesis need dedicated panel |
| **Documentation window** | **Auxiliary window** | Triggered via Help → Documentation; multi-page guided tour |
| **Shape preview before simulation** | **Main window** (viewport) | Already have real-time 3D preview in viewport |
| **Phase overlay on impedance** | **Main window** (impedance chart) | Extend existing LineChart |
| **Community queue / etiquette** | Not applicable | Desktop app, not multi-user online service |
| **Donate** | Not applicable | Not a web service |

### 11.3 Proposed Top Menu Architecture

[certain] Replacing the current simple header (title + hint) with a proper desktop-style menu bar:

```
File        Edit          View        Theme       Help
```

**File menu** (dropdown):
- New Project — reset current session
- Open Project — load JSON config
- Save Project — save current settings to JSON
- Save As — file dialog for save location
- Export → Impedance CSV, Geometry CSV, Geometry JSON, Shape JSON
- Print — print screenshot of current view
- Quit / Ctrl+Q

**Edit menu** (dropdown):
- Undo / Ctrl+Z
- Redo / Ctrl+Y
- Copy View — copy current viewport snapshot to clipboard
- Find Resonance — search peaks by note name
- Preferences — units, simulation defaults

**View menu** (dropdown):
- Toggle Sidebar — show/hide sidebar
- Toggle Wireframe — show/hide wireframe overlay (uses existing `show_wireframe`)
- Toggle Cross-Section — show/hide cross-section chart (uses existing `show_cross_section`)
- Zoom Extents — reset camera
- Full Screen — maximize window

**Theme menu** (dropdown):
- Dark Mode — current default (#x0d1116 background)
- Light Mode — alternate theme
- Auto (System) — follow OS preference

**Help menu** (dropdown):
- Documentation — opens multi-page guided workflow window
- Research Papers — list of relevant acoustics citations
- About — version, authors, build info

### 11.4 Auxiliary Windows Required

Three auxiliary windows for features that don't fit the minimum main window:

1. **Acoustic Target & Optimization Config Window** — Contains: Acoustic Target builder (per-peak note, impedance amplitude, scale tuning, target weights), Evolution Settings (clone from previous, optimization name, duration tiers: Medium/Long), and Results Browser (10 best shapes, tuning table with cents, individual loss breakdown).

2. **AI/ML Features Window** — Contains: neural fitness predictor training panel, PINN surrogate configuration for bent geometries, time-domain synthesis controls (audio output), differentiable TLM parameters. Marked "Experimental" since backend is incomplete.

3. **Documentation Window** — Multi-page guided workflow mirroring DidgeLab's docs: Introduction, Setup Optimization (Acoustic Target, Shape Configuration, Evolution Settings), Run Optimization (loss trends, results analysis), Build a Didgeridoo (export, Blender prep).

### 11.5 Advanced 3D Editor Placement in Main Window

[certain] These features integrate directly with the 3D viewport in the main window — no separate window needed:

- Drag handles on 3D viewport segments (interact with existing BoreViewport)
- Right-click context menu: add bubble, stretch, scale (context menu on viewport)
- Control points for bore curve shape (overlay on viewport)
- Multi-part joint definition at specific x positions (forced diameter markers on viewport)
- Preserve/global modify toggle (sidebar toggle that affects viewport behavior)

### 11.6 Feature That Must Stay in Sidebar (Main Window)

- Acoustic target frequency input (existing single-target slider) — extend to multi-peak via auxiliary window
- Loss weight sliders (existing) — extend to generational trend in main area
- Geometry controls (existing sliders) — add min/max range sliders inline

### 11.7 Implementation Priority (Revised)

#### Immediate (Next UI Session)
1. **Top menu bar** — replace current header with File/Edit/View/Theme/Help dropdowns
2. **View menu toggles** — wire existing `show_wireframe`, `show_cross_section` to View menu items
3. **Theme menu** — dark/light mode toggle (extend current dark theme)
4. **Help → Documentation** — open auxiliary docs window with existing research docs

#### Mid-term (2-3 UI Sessions)
5. **Acoustic Target window** — per-peak note, scale tuning, weights builder
6. **Evolution settings** — clone from previous, name, duration tiers
7. **Generational loss chart** — replace single BarChart with LineChart showing freq/scale/q/total trends
8. **Multi-shape results browser** — 10 best shapes + tuning table with cents
9. **Advanced 3D viewport tools** — drag handles, control points, right-click menu (inline, main window)

#### Lower Priority
10. **AI/ML feature window** — neural predictor, PINN, time-domain synthesis
11. **Export pipeline extension** — Blender/3D print prep (File → Export submenu)
12. **Phase overlay** — extend impedance LineChart

### 11.8 Makepad Verification Required

[certain] Before implementing any new Makepad UI elements, verify against source in `D:/didgeridoo/makepad/`:
- `DropDown` widget usage — confirm dropdown sub-menu API pattern
- `Menu` widget — check if makepad provides a Menu/MenuItem widget, or custom SolidView pattern
- `Window` widget — confirm multi-window creation pattern for auxiliary windows
- `LineChart` widget — confirm API for multiple series (for generational loss chart)
- `DataGrid` widget — confirm API for tuning table

### 11.9 Summary: Menu Structure vs Feature Placement

```
Main Window (minimum working GUI):
├── Top Menu Bar (File, Edit, View, Theme, Help)
├── 3D Viewport (with advanced 3D tools inline: drag handles, control points, right-click menu)
├── Sidebar (geometry controls, loss weights, optimization panel)
└── Main Area (impedance chart with phase overlay, geometry summary, resonance analysis, cross-section view, generational loss chart)

Auxiliary Windows (triggered from menus/toolbar):
├── Documentation (Help → Documentation)
├── Acoustic Target & Optimization Config (Tools → Optimization Settings)
├── AI/ML Features (Tools → AI/ML)
└── Results Browser (Run → Results Browser)
```

Based on thorough review of the DidgeLab website (didgelab.com), the following major UI/UX features are present in DidgeLab but **completely missing** from our current GUI. These must be planned before any further implementation.

### 11.1 Top Menu Architecture (Corrected: etc, not tec)

[certain] The top menu is **File, Edit, Theme, Help** — standard desktop app structure. Not a website nav bar.

**File menu** (dropdown):
- New Project — reset current session
- Open Project — load JSON config
- Save Project — save current settings to JSON
- Save As — file dialog for save location
- Export → Impedance CSV, Geometry CSV, Geometry JSON, Shape JSON (extend existing CSV/JSON export)
- Quit / Ctrl+Q

**Edit menu** (dropdown):
- Undo / Ctrl+Z, Redo / Ctrl+Y
- Preferences — units, simulation defaults

**Theme menu** (dropdown):
- Dark Mode — current default (#x0d1116)
- Light Mode — alternate

**Help menu** (dropdown):
- Documentation — opens auxiliary window with existing research docs
- About — version, authors

### 11.2 Menu Integration with Current Page Structure

DidgeLab's documentation is a **multi-page guided tour**:
- **Introduction** — Overview, features (fundamental note, toots, shape config), experimental acoustic targets (harmonic/inharmonic/shimmering resonances)
- **Setup Optimization** — 3 sections: Acoustic Target, Shape Configuration, Evolution Settings
- **Run Optimization** — Status monitoring, real-time loss chart, results analysis
- **Build a Didgeridoo** — Export shape data, Blender scripts for 3D printing, building methods

**Our gap**: No documentation window, no guided workflow, no explanation of experimental features. User notes: "we already have research and methodology document to navigate for users" — this should be integrated as a Documentation window.

### 11.3 Acoustic Target Definition

DidgeLab's **Setup Optimization** page has a full Acoustic Target builder:

**Basic Targets:**
- **Frequency Tuning** — Per-peak: tune to specific note (e.g., B1, A2), set impedance amplitude ("Ignore" option), weight (priority)
- **Scale Tuning** — Auto-tune all peaks to a scale (e.g., harmonic major)
- **Peak Quantity** — Encourage high number of resonant peaks
- **Peak Amplitude** — Encourage higher, more pronounced peaks

**Weights & Tips:**
- Weights determine priority when optimizer encounters conflicts
- "Prioritize the drone" — high weight on fundamental for specific key
- "Keep it simple" — start with few targets

**Our gap**: We only have a single target frequency slider. No multi-peak targeting, no scale tuning, no weight system per-target, no impedance amplitude targets.

### 11.4 Shape Configuration with Ranges & Forced Diameters

DidgeLab's **Shape Configuration** section:
- **Ranges** — Min/max for length, bell, bore sections (not fixed values)
- **Forced Diameters** — Define specific diameters at certain x-positions for multi-part joint design
- **Preview** — Visualize generated shape before running optimization

**Our gap**: Only fixed sliders. No min/max ranges, no forced diameters for joints, no shape preview before simulation.

### 11.5 Evolution Settings

DidgeLab's **Evolution Settings**:
- **Clone from existing optimization** — Copy shape, targets, or results from previous run to continue refining
- **Optimization name** — Custom name or auto-generated from movie quotes
- **Duration** — Generations: Medium (50 gens, ~30 min), Long (1000 gens, up to 10 hours)
- **Community Etiquette** — Global limit: 3 parallel optimizations; run one at a time; start with test run

**Our gap**: Basic Start/Stop only. No cloning, no naming, no duration tiers, no queue/etiquette system.

### 11.6 Real-Time Loss Chart Over Generations

DidgeLab's **Run Optimization** page shows:
- **Loss chart** tracking freq/scale/q factor/total loss over generations (line chart)
- Visualizes diminishing returns (flattens after ~150 generations)
- "Balancing Weights" analysis: if scale loss high while freq low → optimizer sacrificing scale for note

**Our gap**: Single BarChart showing current loss breakdown. No generational history, no trend visualization, no weight-balancing guidance.

### 11.7 Multi-Shape Results with Tuning Table

DidgeLab's **Results** section:
- **10 best shapes** generated so far (smallest total loss)
- Toggle between Individual 1, 2, 3...
- Per-individual: bore visualization, **tuning table** (frequency, note, cents deviation), impedance spectrum, individual loss breakdown
- **Cents deviation** — e.g., "D (-38 cents)" means between C# and D; <5 cents excellent, >10 cents audibly out of tune
- Download shape data: JSON, Excel, CSV

**Our gap**: Single result only. No tuning table with cents, no multi-shape comparison, no individual loss breakdown, no Excel export.

### 11.8 Build/Export Pipeline

DidgeLab's **Build a Didgeridoo** page:
- Download shape data (JSON, Excel, CSV)
- **Blender scripts** to turn results into 3D-printable meshes ([github.com/didgitaldoo/didge2blender](https://github.com/didgitaldoo/didge2blender))
- External guide: Didgeridoo-Physik building methods

**Our gap**: Basic CSV/JSON only. No Blender export, no 3D print preparation, no manufacturing guidance.

### 11.9 Advanced 3D Editor for Generated Model

User requirement: "more 3D editing of generated model" — DidgeLab only shows static bore visualization. We need:
- Drag handles on 3D viewport to move selected segment
- Scroll wheel to stretch/lengthen geometry
- Right-click menu: add bubble, stretch, scale
- Control points for bore curve shape (local exponent modification)
- Multi-part joint definition at specific x positions
- Forced diameter at joints
- Profile editing with preserve/global modify toggle

### 11.10 AI Features Section

User requirement: "separate section needed for the AI features to control and manipulate"
- Neural fitness predictor training panel
- PINN surrogate for bent geometries
- Time-domain synthesis with audio output
- Differentiable TLM / Adam optimizer integration
- ML backend training pipeline controls

### 11.11 Summary: View Modes (NOT separate pages — single main window)

[certain] These are **view modes** that switch the main area layout via the top menu — not separate web pages. The entire app is ONE window:

| View Mode | Top Menu | Main Area Shows | Sidebar Shows |
|---|---|---|---|
| **Home/Start** | File → New | Quick action buttons, recent projects | Basic geometry controls |
| **Setup** | Edit → Setup | Acoustic target builder, shape config, evolution settings | Target weights, shape ranges |
| **Run** | Run → Start | Real-time loss chart, generational trends, status | Optimization controls, stop |
| **Results** | Run → Results | 10 best shapes, tuning table (cents), impedance per shape | Shape selector, compare toggle |
| **Build/Export** | File → Export | Blender export, 3D print prep, shape data download | Export format options |
| **Documentation** | Help → Docs | Guided workflow steps | N/A |
| **AI/ML** | Tools → AI/ML | Neural predictor training, PINN config, audio synthesis | ML backend controls |

**Key point**: The main window is ALWAYS one window. The top menu switches which panels are visible in the main area. The sidebar context adapts to the active view mode.

---

### Updated Implementation Priority

#### Immediate (Next UI Session)
1. **Top menu bar** replacing header — File/Edit/Theme/Help dropdowns
2. **View mode switching** — Setup, Run, Results, Documentation views in main area
3. **Documentation window** — integrate existing research/methodology docs as Help menu
4. **Acoustic Target builder** — multi-peak, scale tuning, weights (auxiliary window)
5. **Shape Configuration** — ranges, forced diameters (sidebar extension)

#### Mid-term (2-3 UI Sessions)
6. **Evolution settings** — clone from previous, name, duration tiers (auxiliary window)
7. **Generational loss chart** — LineChart replacing single BarChart
8. **Multi-shape results view** — 10 best shapes, tuning table with cents (auxiliary window)
9. **Advanced 3D editor** — drag handles, control points, right-click menu (inline in viewport)

#### Lower Priority
10. **AI/ML feature window** — neural predictor, PINN, differentiable TLM
11. **Time-domain synthesis** — audio output
12. **Blender/3D print export** — File → Export pipeline extension

