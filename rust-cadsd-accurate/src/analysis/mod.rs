//! Analysis tools for didgeridoo acoustics
//!
//! This module provides spectrum analysis and visualization functions
//! that match the Python DidgeLab analysis module.

use crate::geo::Geo;

/// Get resonance notes from frequency and impedance data (same as Python)
pub fn get_notes(frequencies: &[f64], impedances: &[f64]) -> Vec<(f64, f64)> {
    let mut peaks = Vec::new();
    
    for i in 1..impedances.len() - 1 {
        if impedances[i] > impedances[i-1] && impedances[i] > impedances[i+1] {
            peaks.push((frequencies[i], impedances[i]));
        }
    }
    
    peaks
}

/// Visualize didgeridoo geometry (placeholder - same interface as Python)
pub fn vis_didge(geo: &Geo) {
    println!("Visualizing didgeridoo geometry:");
    println!("  Length: {:.1} mm", geo.length());
    println!("  Bell size: {:.1} mm", geo.bellsize());
    println!("  Segments: {}", geo.geo.len());
}

/// Plot bore profile (placeholder - same interface as Python)
pub fn plot_bore(geo: &Geo) {
    // In Python, this would create a 2D cross-section plot
    // For now, just print basic info
    println!("Bore profile for didgeridoo ({} segments):", geo.geo.len());
    for (i, &[x, d]) in geo.geo.iter().enumerate().take(10) {
        println!("  {}: {:.1} mm -> {:.1} mm diameter", i, x, d);
    }
    if geo.geo.len() > 10 {
        println!("  ... and {} more segments", geo.geo.len() - 10);
    }
}

/// Plot impedance spectrum (placeholder - same interface as Python)
pub fn plot_impedance_spectrum(frequencies: &[f64], impedances: &[f64]) {
    // In Python, this would create a spectrum plot
    // For now, just print basic info
    println!("Impedance spectrum ({} points):", frequencies.len());
    for (i, (freq, imp)) in frequencies.iter().zip(impedances.iter()).enumerate().take(10) {
        println!("  {}: {:.1} Hz -> {:.2e} impedance", i, freq, imp);
    }
    if frequencies.len() > 10 {
        println!("  ... and {} more points", frequencies.len() - 10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::Geo;
    use crate::sim::{acoustical_simulation, get_log_simulation_frequencies};
    
    #[test]
    fn test_get_notes() {
        // Test with a simple synthetic spectrum
        let frequencies = vec![100.0, 200.0, 300.0, 400.0, 500.0];
        let impedances = vec![1.0, 5.0, 2.0, 8.0, 1.5];  // Peaks at 200Hz and 400Hz
        
        let notes = get_notes(&frequencies, &impedances);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].0, 200.0);
        assert_eq!(notes[1].0, 400.0);
    }
    
    #[test]
    fn test_analysis_with_real_data() {
        let geo = Geo::make_cone(1500.0, 32.0, 65.0, 20);
        let frequencies = get_log_simulation_frequencies();
        let impedances = acoustical_simulation(&geo, &frequencies, "tlm_python").unwrap();
        
        let notes = get_notes(&frequencies, &impedances);
        assert!(!notes.is_empty());
        
        // Basic visualization functions should not panic
        vis_didge(&geo);
        plot_bore(&geo);
        plot_impedance_spectrum(&frequencies, &impedances);
    }
}