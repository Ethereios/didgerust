# didgerust - Visualization — Replaced stubs with plotters PNG output

## Implementation

1. `plot_bore_geometry`:
   - Directly renders PNG using `plotters::BitMapBackend`
   - Properly handles xy coordinates
   - Outputs to specified path

2. `plot_impedance_spectrum`:
   - Uses log-scaled frequency grid (wrapper crate)
   - Matches accurate crate's impedance calculation semantics

3. `generate_text_report`:
   - Now writes actual text files with detailed resonance data

## Example Outputs
- Example `geometry.png` showing cones with bubbles
- Example `spectrum.png` with impedance curves
- Example `report.txt` containing resonance peaks

## Integration
- Works with both crates' `Geo` implementations
- Supports linear/non-linear frequency grids
- All plots match semantic requirements in `DESIGN_NOTES.md`