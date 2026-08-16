//! Visualization tools for CADSD

use plotters::prelude::*;
use std::error::Error;
use crate::Geo;
use crate::sim::Resonance;
use crate::{
    get_log_simulation_frequencies,
    acoustical_simulation,
};

/// Get resonance notes from frequency and impedance data (migrated from accurate crate)
pub fn get_notes(frequencies: &[f64], impedances: &[f64]) -> Vec<(f64, f64)> {
    let mut peaks = Vec::new();

    for i in 1..impedances.len() - 1 {
        if impedances[i] > impedances[i-1] && impedances[i] > impedances[i+1] {
            peaks.push((frequencies[i], impedances[i]));
        }
    }

    peaks
}

/// Plot bore geometry using actual PNG rendering
pub fn plot_bore_geometry(geo: &Geo, output_path: &str) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Bore Geometry", ("Arial", 30))
        .margin(20)
        .build_cartesian_2d(0f64..2000.0, 0.0..200.0)?;

    chart.configure_mesh().draw()?;
    
    let points: Vec<(f64, f64)> = geo.geo.iter()
        .map(|p| (p[0], p[1]))
        .collect();
    
    chart.draw_series(LineSeries::new(points, &BLUE))?;
    
    Ok(())
}

/// Plot impedance spectrum using plotters
pub fn plot_impedance_spectrum(
    geo: &Geo,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let freqs = get_log_simulation_frequencies();
    
    // Compute impedance using the accurate crate's function
    let impedances = acoustical_simulation(geo, &freqs, "tlm_cython")?;
    
    let data: Vec<(f64, f64)> = freqs.iter()
        .zip(impedances.iter())
        .map(|(&f, &z)| (f, z))
        .collect();
    
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Impedance Spectrum", ("Arial", 30))
        .margin(20)
        .build_cartesian_2d(20.0..2000.0, 0.0..1e4)?;

    chart.configure_mesh().draw()?;
    
    chart.draw_series(LineSeries::new(data, &RED))?;
    
    Ok(())
}

/// Plot evolution progress
pub fn plot_evolution_progress(
    generations: &[usize],
    best_losses: &[f64],
    avg_losses: &[f64],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let max_gen = *generations.last().unwrap_or(&50) as u32;
    let max_loss = best_losses.iter().chain(avg_losses.iter()).cloned().fold(f64::NEG_INFINITY, f64::max);
    let max_loss = max_loss.max(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption("Evolution Progress", ("Arial", 30))
        .margin(20)
        .build_cartesian_2d(0u32..max_gen, 0.0..max_loss)?;

    chart.configure_mesh().draw()?;
    
    let best_data: Vec<(u32, f64)> = generations.iter()
        .zip(best_losses.iter())
        .map(|(&g, &l)| (g as u32, l))
        .collect();
    
    let avg_data: Vec<(u32, f64)> = generations.iter()
        .zip(avg_losses.iter())
        .map(|(&g, &l)| (g as u32, l))
        .collect();
    
    chart.draw_series(LineSeries::new(best_data, &BLUE))?;
    chart.draw_series(LineSeries::new(avg_data, &GREEN))?;
    
    Ok(())
}

/// Create comprehensive analysis report
pub fn create_analysis_report(
    geo: &Geo,
    output_dir: &str,
) -> Result<(), Box<dyn Error>> {
    use std::fs;
    
    fs::create_dir_all(output_dir)?;
    
    plot_bore_geometry(geo, &format!("{}/geometry.png", output_dir))?;
    plot_impedance_spectrum(geo, &format!("{}/spectrum.png", output_dir))?;
    
    let freqs = get_log_simulation_frequencies();
    let impedances = acoustical_simulation(geo, &freqs, "tlm_cython")?;
    
    // Find peaks locally
    let mut peaks = Vec::new();
    for i in 1..impedances.len().saturating_sub(1) {
        if impedances[i] > impedances[i - 1] && impedances[i] > impedances[i + 1] {
            peaks.push(Resonance {
                frequency: freqs[i],
                impedance: impedances[i],
            });
        }
    }
    
    let report = generate_text_report(geo, &peaks);
    
    fs::write(format!("{}/report.txt", output_dir), report)?;
    Ok(())
}

pub fn generate_text_report(geo: &Geo, peaks: &[Resonance]) -> String {
    let mut report = String::new();
    
    report.push_str("=== CADSD Analysis Report ===\n\n");
    
    report.push_str(&format!("Geometry Summary:\n"));
    report.push_str(&format!("  Length: {:.2} mm\n", geo.length())); 
    report.push_str(&format!("  Bell diameter: {:.2} mm\n", geo.bellsize()));
    report.push_str(&format!("  Number of segments: {}\n", geo.geo.len()));
    report.push_str(&format!("  Volume: {:.2} mm\u{b0}\n\n", geo.compute_volume()));
    
    report.push_str(&format!("Resonance Analysis:\n"));
    report.push_str(&format!("  Number of peaks: {}\n", peaks.len()));
    
    if !peaks.is_empty() {
        let fundamental = peaks.first().map(|p| p.frequency).unwrap_or(0.0);
        
        if !peaks.is_empty() {
            report.push_str("\n  Peak frequencies:\n");
            for (i, peak) in peaks.iter().enumerate() {
                let freq_label = if i == 0 && fundamental > 0.0 {
                    format!("{:.2} Hz (Fundamental)", peak.frequency)
                } else {
                    format!("{:.2} Hz", peak.frequency)
                };
                report.push_str(&format!("    {}: {} (impedance: {:.2})\n", 
                                        i + 1, freq_label, peak.impedance));
            }
        }
        
        if peaks.len() > 1 && fundamental > 0.0 {
            report.push_str("\n  Harmonic ratios:\n");
            for (i, peak) in peaks.iter().enumerate().skip(1) {
                let ratio = peak.frequency / fundamental;
                let harmonic_num = i + 1;
                report.push_str(&format!("    H{}: {:.3}\n", harmonic_num, ratio));
            }
        }
    }
    
    report.push_str("\n=== End of Report ===\n");
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    
    #[test]
    fn test_visualization_functions() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        
        // Create a temporary directory in the current temp folder
        let output_dir = "test_output_viz";
        let _ = fs::remove_dir_all(output_dir); // Clean up if exists
        let _ = fs::create_dir_all(output_dir); // Re-create directory
        
        // Test plotting functions directly (avoid slow acoustical_simulation)
        let result = plot_bore_geometry(&geo, &format!("{}/geometry.png", output_dir));
        assert!(result.is_ok());
        
        let result2 = plot_evolution_progress(&vec![0, 1, 2], &vec![1.0, 0.5, 0.2], &vec![1.0, 0.7, 0.4], &format!("{}/progress.png", output_dir));
        assert!(result2.is_ok());
        
        let report = generate_text_report(&geo, &[]);
        fs::write(format!("{}/report.txt", output_dir), report).unwrap();
        
        assert!(Path::new(&format!("{}/geometry.png", output_dir)).exists());
        assert!(Path::new(&format!("{}/progress.png", output_dir)).exists());
        assert!(Path::new(&format!("{}/report.txt", output_dir)).exists());
        
        // Clean up
        let _ = fs::remove_dir_all(output_dir);
    }
    
    #[test]
    fn test_get_notes() {
        let freqs = vec![100.0, 200.0, 300.0, 400.0, 500.0];
        let impedances = vec![10.0, 50.0, 30.0, 60.0, 20.0]; 
        let notes = get_notes(&freqs, &impedances);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0], (200.0, 50.0));
        assert_eq!(notes[1], (400.0, 60.0));
    }
}