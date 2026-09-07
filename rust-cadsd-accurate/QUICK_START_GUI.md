# Quick Start Guide - CADSD Makepad GUI

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
./target/release/cadsd-gui
```

### Option 3: Test Minimal GUI (Debug purposes)
```bash
cargo run --features gui -- test-gui
```

## What You'll See

When the GUI opens successfully, you should see:

### Left Panel - Controls
- **Geometry Parameters**: Length, Top Diameter, Bottom Diameter, Segments
- **Bore Style**: Choose from Cone, Cylinder, Exponential (Kigali, Mbeya planned)
- **Run Simulation Button**: Execute acoustic analysis
- **Results Section**: Shows fundamental frequency and resonance details

### Center Panel - Visualization
- **3D Viewport**: Real-time GPU-rendered bore geometry with orbit/zoom controls
- **Status Indicators**: Green "Ready" means everything is working

## Basic Workflow

1. **Adjust Geometry** - Use sliders in left panel to modify dimensions
2. **Select Bore Style** - Choose bore profile type (cone, exponential, cylinder)
3. **Run Simulation** - Click button to compute acoustic properties (runs on background thread)
4. **View Results** - Check fundamental frequency, resonance count, and musical note
5. **Iterate** - Adjust parameters and re-run to refine design

## Troubleshooting

### White Screen or Black Screen
- The Makepad GPU-driven UI handles initialization differently - no white screen issues
- If it persists, check that your graphics drivers support Vulkan/DX12/Metal
- Ensure makepad-widgets and makepad-xr features are enabled

### Window Won't Open
- Verify you have the GUI feature enabled: `--features gui`
- Check terminal for error messages
- Try the test-gui option: `cargo run --features gui -- test-gui`

### Slow Performance
- Use release build: `cargo build --release`
- Reduce segments count (try 10-20 instead of 50)
- Makepad GPU rendering is highly optimized for 60 FPS

### No 3D Model Visible
- Try rotating view with mouse drag
- Scroll wheel to zoom in/out
- Run simulation to ensure data exists

## Keyboard Shortcuts

While the GUI is running:
- **Mouse Drag** - Orbit camera around model
- **Scroll Wheel** - Zoom in/out
- **Escape** - Close window

## Tips for Best Results

1. **Start Simple**: Begin with cone shape, then try exponential
2. **Watch Segments**: Higher = more accurate but slower (20-30 is good)
3. **Check Lighting**: The model renders with PBR materials and proper lighting
4. **Use Ground Plane**: Provides scale reference for your design
5. **Save Configurations**: Note successful parameter combinations

## Example Settings

### Traditional Didgeridoo
- Length: 1200-1500 mm
- Top Diameter: 28-32 mm
- Bottom Diameter: 55-70 mm
- Style: Cone or Exponential
- Segments: 30-50

### Modern Design
- Length: 1000-1800 mm
- Top Diameter: 25-40 mm
- Bottom Diameter: 60-90 mm
- Style: Exponential
- Segments: 50-100

## Next Steps

Once comfortable with the GUI:
1. Experiment with different shape profiles
2. Analyze how geometry changes affect acoustics
3. Compare open vs closed tuning characteristics
4. Use Tairua Loss metric to optimize designs
5. Export successful configurations for further analysis

---

**Note**: This GUI uses the same acoustic simulation engine as the Python DidgeLab toolkit, ensuring accurate predictions matching real-world behavior. The GUI is built with Makepad - a GPU-driven immediate-mode UI framework for Rust.