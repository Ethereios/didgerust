# Quick Start Guide - CADSD GUI

## Running the GUI

### Option 1: Development Build (Faster to start, good for testing)
```bash
cd rust-cadsd/rust-cadsd-accurate
cargo run --features gui -- gui
```

### Option 2: Release Build (Better performance, recommended for actual use)
```bash
cd rust-cadsd/rust-cadsd-accurate
cargo build --features gui --release
./target/release/cadsd gui
```

### Option 3: Test Minimal GUI (Debug purposes)
```bash
cargo run --features gui -- test-gui
```

## What You'll See

When the GUI opens successfully, you should see:

### Left Panel - Controls
- **Geometry Parameters**: Length, Top Diameter, Bottom Diameter, Segments
- **Style Type**: Choose from Cone, Cylinder, Exponential, Kigali, Mbeya
- **Display Options**: Show 3D Model, Auto Update
- **Run Simulation Button**: Execute acoustic analysis
- **Results Section**: Shows fundamental frequency and resonance details

### Center Panel - Visualization
- **3D View**: Real-time rendering of your didgeridoo design
- **Impedance Chart**: Visual representation of acoustic response
- **Tuning Information**: Open and closed tuning suggestions
- **Status Indicators**: Green "✓ GUI Active" means everything is working

## Basic Workflow

1. **Adjust Geometry** - Use sliders in left panel to modify dimensions
2. **Select Style** - Choose bore profile type (cone, exponential, etc.)
3. **Run Simulation** - Click button to compute acoustic properties
4. **View Results** - Check impedance chart and resonance frequencies
5. **Iterate** - Adjust parameters and re-run to refine design

## Troubleshooting

### White Screen or Black Screen
- ✅ **Fixed!** The recent fixes address this issue
- If it persists, check that your graphics drivers are updated
- Ensure Vulkan support (required by Bevy)

### Window Won't Open
- Verify you have the GUI feature enabled: `--features gui`
- Check terminal for error messages
- Try the minimal GUI test: `cargo run --features gui -- test-gui`

### Slow Performance
- Use release build: `cargo build --release`
- Reduce segments count (try 10-20 instead of 50)
- Disable auto-update if enabled

### No 3D Model Visible
- Check "Show 3D Model" checkbox in Display Options
- Try rotating view with mouse
- Run simulation to ensure data exists

## Keyboard Shortcuts

While the GUI is running:
- **Mouse Drag** - Rotate camera around model
- **Scroll Wheel** - Zoom in/out
- **Right-click Drag** - Pan camera
- **Escape** - Close window

## Tips for Best Results

1. **Start Simple**: Begin with cone shape, then try advanced profiles
2. **Watch Segments**: Higher = more accurate but slower (20-30 is good)
3. **Check Lighting**: The model should be clearly visible with good shadows
4. **Use Ground Plane**: Provides scale reference for your design
5. **Save Configurations**: Note successful parameter combinations

## Example Settings

### Traditional Didgeridoo
- Length: 1200-1500 mm
- Top Diameter: 28-32 mm
- Bottom Diameter: 55-70 mm
- Style: Cone or Exponential
- Bore Curve: 0.3-0.7 (for exponential)

### Modern Design
- Length: 1000-1800 mm
- Top Diameter: 25-40 mm
- Bottom Diameter: 60-90 mm
- Style: Kigali or Mbeya
- Bore Curve: -0.5 to 1.0

## Next Steps

Once comfortable with the GUI:
1. Experiment with different shape profiles
2. Analyze how geometry changes affect acoustics
3. Compare open vs closed tuning characteristics
4. Use Tairua Loss metric to optimize designs
5. Export successful configurations for further analysis

---

**Note**: This GUI uses the same acoustic simulation engine as the Python DidgeLab toolkit, ensuring accurate predictions matching real-world behavior.
