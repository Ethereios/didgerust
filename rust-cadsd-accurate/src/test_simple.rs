//! Simple test to verify basic CADSD functionality
//!
//! This creates a simple conical didgeridoo and computes its impedance spectrum
//! to verify the core functionality works before building the GUI.

use cadsd_accurate::geo::Geo;
use cadsd_accurate::sim::{acoustical_simulation, get_log_simulation_frequencies, get_fundamental};

fn main() {
    println!("=== CADSD Basic Test ===\n");
    
    // Create a simple conical didgeridoo
    let geo = Geo::make_cone(1500.0, 32.0, 65.0, 20);
    println!("Created conical didgeridoo:");
    println!("  Length: {:.1} mm", geo.length());
    println!("  Bell diameter: {:.1} mm", geo.bellsize());
    println!("  Number of segments: {}\n", geo.geo.len());
    
    // Generate frequency grid
    let frequencies = get_log_simulation_frequencies();
    println!("Frequency range: {:.1} - {:.1} Hz ({} frequencies)\n", 
             frequencies.first().unwrap_or(&0.0), 
             frequencies.last().unwrap_or(&0.0),
             frequencies.len());
    
    // Run simulation
    println!("Computing impedance spectrum...");
    match acoustical_simulation(&geo, &frequencies, "tlm_python") {
        Ok(impedances) => {
            println!("Simulation successful! {} impedance values computed.\n", impedances.len());
            
            // Print some sample values
            println!("Sample impedance values:");
            println!("  {} Hz: {:.2e}", frequencies[100], impedances[100]);
            println!("  {} Hz: {:.2e}", frequencies[500], impedances[500]);
            println!("  {} Hz: {:.2e}", frequencies[1000], impedances[1000]);
            
            // Try to find fundamental
            match get_fundamental(&geo, "tlm_python", 20.0) {
                Ok((freq, impedance)) => {
                    println!("\nFound fundamental: {:.1} Hz, impedance: {:.2e}", freq, impedance);
                }
                Err(e) => {
                    println!("\nCould not find fundamental: {}", e);
                }
            }
            
            // Summary
            let max_imp = impedances.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            println!("\nMax impedance: {:.2e}", max_imp);
            println!("Average impedance: {:.2e}", 
                    impedances.iter().sum::<f64>() / impedances.len() as f64);
            println!("Basic test completed successfully!");
        }
        Err(e) => {
            eprintln!("Error: Failed to run simulation - {}", e);
            std::process::exit(1);
        }
    }
}