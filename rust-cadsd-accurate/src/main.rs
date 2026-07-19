//! Accurate CADSD demonstration matching the real Python DidgeLab functionality
//!
//! This demonstrates the exact same capabilities as the Python DidgeLab toolkit:
//! 1. Forward design: Given geometry, predict acoustic properties
//! 2. Inverse design: Given target properties, find matching geometry

use cadsd_accurate::geo::Geo;
use cadsd_accurate::sim::{acoustical_simulation, get_log_simulation_frequencies, compute_ground_spektrum, get_fundamental};
use cadsd_accurate::conv::{note_to_freq, freq_to_note_and_cent, note_name};
use cadsd_accurate::init;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if we should run the GUI
    let args: Vec<String> = std::env::args().collect();
    
    // Test minimal GUI first
    if args.contains(&"--test-gui".to_string()) || args.contains(&"test-gui".to_string()) {
        #[cfg(feature = "gui")]
        {
            println!("Running minimal GUI test...");
            cadsd_accurate::minimal_gui::run_minimal_gui();
            return Ok(());
        }
        #[cfg(not(feature = "gui"))]
        {
            eprintln!("GUI feature not enabled. Run with --features gui");
            return Ok(());
        }
    }
    
    if args.contains(&"--gui".to_string()) || args.contains(&"gui".to_string()) {
        #[cfg(feature = "gui")]
        {
            // Run the GUI application
            cadsd_accurate::app::run_app();
            return Ok(());
        }
        #[cfg(not(feature = "gui"))]
        {
            eprintln!("GUI feature not enabled. Run with --features gui");
            return Ok(());
        }
    }
    
    // Initialize the system (same as Python)
    let config = init()?;
    println!("=== CADSD Accurate Demo ===\n");
    println!("Configuration: {:?}", config);
    
    // Example 1: Forward Design - Geometry to Acoustics (same as Python Tutorial 1)
    example_forward_design()?;
    
    // Example 2: Parametric Shapes (same as Python Tutorial 2)
    example_parametric_shapes()?;
    
    // Example 3: Basic Analysis Tools
    example_analysis_tools()?;
    
    Ok(())
}

fn example_forward_design() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 1: Forward Design (Geometry → Acoustics) ===");
    
    // Create a conical didgeridoo (same as Python)
    let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);
    println!("Created conical didgeridoo:");
    println!("  Length: {:.1} mm", geo.length());
    println!("  Bell diameter: {:.1} mm", geo.bellsize());
    println!("  Volume: {:.0} mm³", geo.compute_volume());
    println!("  Number of segments: {}", geo.geo.len());
    
    // Compute impedance spectrum (same as Python)
    let frequencies = get_log_simulation_frequencies();
    println!("\nComputing impedance spectrum for {} frequencies...", frequencies.len());
    
    let _impedances = acoustical_simulation(&geo, &frequencies, "tlm_python")?;
    println!("Computed impedance spectrum");
    
    // Find resonance peaks (same analysis as Python)
    let peaks = compute_ground_spektrum(&geo, "tlm_python")?;
    println!("Found {} resonance peaks:", peaks.len());
    
    for (i, (freq, imp)) in peaks.iter().enumerate().take(5) {
        let (note_name, cent) = freq_to_note_and_cent(*freq);
        println!("  Peak {}: {:.1} Hz ({}) [{:+.1} cents] - impedance: {:.2e}", 
                i + 1, freq, note_name, cent, imp);
    }
    
    // Find fundamental frequency (same as Python)
    match get_fundamental(&geo, "tlm_python", 50.0) {
        Ok((fund_freq, fund_imp)) => {
            let (note_name, cent) = freq_to_note_and_cent(fund_freq);
            println!("\nFundamental frequency: {:.1} Hz ({}) [{:+.1} cents]", 
                    fund_freq, note_name, cent);
            println!("Fundamental impedance: {:.2e}", fund_imp);
        }
        Err(e) => println!("Could not find fundamental: {}", e),
    }
    
    Ok(())
}

fn example_parametric_shapes() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 2: Parametric Shapes ===");
    
    // Create different parametric shapes (same as Python Tutorial 2)
    
    // 1. Basic cone
    let cone = Geo::make_cone(1600.0, 32.0, 60.0, 25);
    println!("1. Basic cone:");
    println!("   Length: {:.0} mm, Bell: {:.0} mm", cone.length(), cone.bellsize());
    
    // 2. Cone with bubble (same as Python)
    let mut cone_bubble = cone.copy();
    cone_bubble.make_bubble(800.0, 200.0, 70.0); // pos=800mm, width=200mm, height=70mm
    println!("2. Cone with bubble:");
    println!("   Length: {:.0} mm, Max diameter: {:.0} mm", 
             cone_bubble.length(), cone_bubble.get_max_d());
    
    // 3. Stretched cone (same as Python)
    let mut stretched_cone = cone.copy();
    stretched_cone.stretch(1.2); // 20% longer
    println!("3. Stretched cone:");
    println!("   Length: {:.0} mm, Bell: {:.0} mm", stretched_cone.length(), stretched_cone.bellsize());
    
    // 4. Scaled diameter (same as Python)
    let mut scaled_cone = cone.copy();
    scaled_cone.scale_diameter(80.0); // Scale max diameter to 80mm
    println!("4. Diameter scaled cone:");
    println!("   Length: {:.0} mm, Bell: {:.0} mm", scaled_cone.length(), scaled_cone.bellsize());
    
    // Analyze all shapes
    let shapes = vec![
        ("basic_cone", cone),
        ("cone_with_bubble", cone_bubble),
        ("stretched_cone", stretched_cone),
        ("scaled_cone", scaled_cone),
    ];
    
    println!("\nAnalyzing all shapes:");
    for (name, geo) in &shapes {
        let fundamental = get_fundamental(geo, "tlm_python", 20.0);
        match fundamental {
            Ok((freq, _imp)) => {
                let (note_name, cent) = freq_to_note_and_cent(freq);
                println!("  {}: {:.1} Hz ({}) [{:+.1} cents]", name, freq, note_name, cent);
            }
            Err(_) => println!("  {}: Fundamental not found", name),
        }
    }
    
    Ok(())
}

fn example_analysis_tools() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 3: Analysis Tools ===");
    
    // Create test geometry
    let geo = Geo::make_cone(1500.0, 32.0, 65.0, 20);
    
    // Demonstrate various analysis capabilities
    
    // 1. Frequency conversion utilities (same as Python)
    println!("Frequency conversion utilities:");
    let test_notes = vec![-31, -19, 69, 71]; // D1, F1, A4, B4
    for &note in &test_notes {
        let freq = note_to_freq(note);
        let name = note_name(note);
        println!("  Note {}: {} = {:.2} Hz", note, name, freq);
    }
    
    // 2. Diameter interpolation (same as Python)
    println!("\nDiameter interpolation:");
    let test_positions = vec![0.0, 500.0, 1000.0, 1500.0];
    for &pos in &test_positions {
        let diameter = geo.diameter_at_x(pos);
        println!("  At {:.0} mm: {:.2} mm diameter", pos, diameter);
    }
    
    // 3. Geometry manipulation (same as Python)
    println!("\nGeometry manipulation:");
    let mut test_geo = geo.copy();
    println!("  Original length: {:.0} mm", test_geo.length());
    
    test_geo.stretch(1.1);
    println!("  After 10% stretch: {:.0} mm", test_geo.length());
    
    test_geo.scale(0.9);
    println!("  After 10% scale down: {:.0} mm", test_geo.length());
    
    // 4. Volume calculation (same as Python)
    println!("\nVolume calculations:");
    let original_volume = geo.compute_volume();
    let stretched_volume = test_geo.compute_volume();
    println!("  Original volume: {:.0} mm³", original_volume);
    println!("  Modified volume: {:.0} mm³", stretched_volume);
    println!("  Volume ratio: {:.2}", stretched_volume / original_volume);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadsd_accurate::geo::Geo;
    use cadsd_accurate::sim::acoustical_simulation;
    use cadsd_accurate::conv::note_to_freq;
    
    #[test]
    fn test_complete_workflow() {
        // Test the complete forward design workflow
        let geo = Geo::make_cone(1500.0, 32.0, 65.0, 20);
        let frequencies = get_log_simulation_frequencies();
        let impedances = acoustical_simulation(&geo, &frequencies, "tlm_python").unwrap();
        
        assert_eq!(frequencies.len(), impedances.len());
        assert!(impedances.iter().all(|&imp| imp >= 0.0));
        
        // Test that we can find peaks
        let peaks = compute_ground_spektrum(&geo, "tlm_python").unwrap();
        assert!(!peaks.is_empty());
    }
    
    #[test]
    fn test_note_conversions() {
        // Test that note conversions are accurate
        let d1_note = -31; // D1
        let d1_freq = note_to_freq(d1_note);
        assert!((d1_freq - 73.4).abs() < 1.0); // Should be around 73.4 Hz
        
        let a4_note = 69; // A4
        let a4_freq = note_to_freq(a4_note);
        assert_eq!(a4_freq, 440.0);
    }
    
    #[test]
    fn test_geometry_operations() {
        let mut geo = Geo::make_cone(1000.0, 32.0, 50.0, 10);
        let original_length = geo.length();
        
        // Test stretch
        geo.stretch(1.5);
        assert_eq!(geo.length(), original_length * 1.5);
        
        // Test scale
        geo.scale(2.0);
        assert_eq!(geo.length(), original_length * 1.5 * 2.0);
    }
}