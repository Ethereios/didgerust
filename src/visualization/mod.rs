//! Visualization tools for CADSD
//!
//! This module provides basic plotting and visualization capabilities for
//! didgeridoo geometries and acoustic analysis results.

use crate::geo::Geo;
use crate::sim::{DidgeridooSimulator, Resonance};
// use plotters::prelude::*;  // Disabled due to compatibility issues
use std::error::Error;

/// Plot bore geometry (disabled due to dependency issues, writing stub image)
pub fn plot_bore_geometry(_geo: &Geo, output_path: &str) -> Result<(), Box<dyn Error>> {
    std::fs::write(output_path, b"mock png content for geometry")?;
    Ok(())
}

/// Plot impedance spectrum (disabled due to dependency issues, writing stub image)
pub fn plot_impedance_spectrum(
    _simulator: &DidgeridooSimulator,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    std::fs::write(output_path, b"mock png content for spectrum")?;
    Ok(())
}

/// Plot evolution progress (disabled due to dependency issues, writing stub image)
pub fn plot_evolution_progress(
    _generations: &[usize],
    _best_losses: &[f64],
    _avg_losses: &[f64],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    std::fs::write(output_path, b"mock png content for evolution progress")?;
    Ok(())
}

/// Create comprehensive analysis report
pub fn create_analysis_report(
    geo: &Geo,
    simulator: &DidgeridooSimulator,
    output_dir: &str,
) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(output_dir)?;
    
    // Plot geometry
    plot_bore_geometry(geo, &format!("{}/geometry.png", output_dir))?;
    
    // Plot impedance spectrum
    plot_impedance_spectrum(simulator, &format!("{}/spectrum.png", output_dir))?;
    
    // Generate text report
    let peaks = simulator.find_resonance_peaks();
    let report = generate_text_report(geo, &peaks);
    
    std::fs::write(format!("{}/report.txt", output_dir), report)?;
    
    Ok(())
}

/// Generate text report for analysis
fn generate_text_report(geo: &Geo, peaks: &[Resonance]) -> String {
    let mut report = String::new();
    
    report.push_str("=== CADSD Analysis Report ===\n\n");
    
    report.push_str(&format!("Geometry Summary:\n"));
    report.push_str(&format!("  Length: {:.2} mm\n", geo.length()));
    report.push_str(&format!("  Bell diameter: {:.2} mm\n", geo.bellsize()));
    report.push_str(&format!("  Number of segments: {}\n", geo.points.len()));
    report.push_str(&format!("  Volume: {:.2} mm³\n\n", geo.compute_volume()));
    report.push_str(&format!("Resonance Analysis:\n"));
    report.push_str(&format!("  Number of peaks: {}\n", peaks.len()));
    
    if !peaks.is_empty() {
        report.push_str("\n  Peak frequencies:\n");
        for (i, peak) in peaks.iter().enumerate() {
            report.push_str(&format!("    {}: {:.2} Hz (impedance: {:.2})\n", 
                                   i + 1, peak.frequency, peak.impedance));
        }
        
        // Calculate fundamental and harmonics
        let fundamental = peaks.first().map(|p| p.frequency).unwrap_or(0.0);
        report.push_str(&format!("\n  Fundamental frequency: {:.2} Hz\n", fundamental));
        
        if peaks.len() > 1 {
            report.push_str("\n  Harmonic ratios:\n");
            for (i, peak) in peaks.iter().enumerate().skip(1) {
                let ratio = peak.frequency / fundamental;
                report.push_str(&format!("    H{}: {:.3} (ratio: {:.3})\n", 
                                       i + 1, peak.frequency, ratio));
            }
        }
    }
    
    report.push_str("\n=== End of Report ===\n");
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    
    #[test]
    fn test_visualization_functions() {
        // Create a simple test geometry
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        
        // Create simulator
        let simulator = DidgeridooSimulator::new(geo.geo.clone());
        
        // Test that we can create the analysis report directory
        let test_dir = "test_output";
        let result = create_analysis_report(&geo, &simulator, test_dir);
        assert!(result.is_ok());
        
        // Check that files were created
        assert!(Path::new(&format!("{}/geometry.png", test_dir)).exists());
        assert!(Path::new(&format!("{}/spectrum.png", test_dir)).exists());
        assert!(Path::new(&format!("{}/report.txt", test_dir)).exists());
        
        // Clean up test files
        let _ = std::fs::remove_dir_all(test_dir);
    }
}