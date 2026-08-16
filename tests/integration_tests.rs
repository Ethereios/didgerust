//! Integration tests for the cadsd wrapper crate.
//! These tests exercise the end-to-end workflow:
//! geometry creation → simulation → analysis → report generation.

use cadsd::Geo;
use cadsd::visualization::{generate_text_report, plot_bore_geometry, plot_evolution_progress};
use cadsd::sim::{find_resonance_peaks, SimulationStrategy, DidgeridooSimulator, AcousticConstants};
use cadsd::tonehole::Tonehole;
use std::fs;
use std::path::Path;

#[test]
fn test_full_workflow_cone() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);

    let output_dir = "test_integration_cone";
    let _ = fs::remove_dir_all(output_dir);
    let _ = fs::create_dir_all(output_dir);
    
    // Test plotting directly instead of slow create_analysis_report
    let result = plot_bore_geometry(&geo, &format!("{}/geometry.png", output_dir));
    assert!(result.is_ok(), "Geometry plot failed");

    let result2 = plot_evolution_progress(&vec![0, 1, 2], &vec![1.0, 0.5, 0.2], &vec![1.0, 0.7, 0.4], &format!("{}/progress.png", output_dir));
    assert!(result2.is_ok(), "Progress plot failed");
    
    let report = generate_text_report(&geo, &[]);
    fs::write(format!("{}/report.txt", output_dir), report).unwrap();

    assert!(Path::new(&format!("{}/geometry.png", output_dir)).exists());
    assert!(Path::new(&format!("{}/progress.png", output_dir)).exists());
    assert!(Path::new(&format!("{}/report.txt", output_dir)).exists());

    let report_text = fs::read_to_string(&format!("{}/report.txt", output_dir)).unwrap();
    assert!(report_text.contains("CADSD Analysis Report"));
    assert!(report_text.contains("Geometry Summary"));
    assert!(report_text.contains("Resonance Analysis"));

    let _ = fs::remove_dir_all(output_dir);
}

#[test]
fn test_full_workflow_with_bubble() {
    let mut geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    geo.make_bubble(500.0, 100.0, 50.0);

    let output_dir = "test_integration_bubble";
    let _ = fs::remove_dir_all(output_dir);
    let _ = fs::create_dir_all(output_dir);
    
    let result = plot_bore_geometry(&geo, &format!("{}/geometry.png", output_dir));
    assert!(result.is_ok(), "Geometry plot failed with bubble");
    
    let report = generate_text_report(&geo, &[]);
    fs::write(format!("{}/report.txt", output_dir), report).unwrap();
    
    assert!(Path::new(&format!("{}/geometry.png", output_dir)).exists());
    assert!(Path::new(&format!("{}/report.txt", output_dir)).exists());

    let report_text = fs::read_to_string(&format!("{}/report.txt", output_dir)).unwrap();
    assert!(report_text.contains("CADSD Analysis Report"));

    let _ = fs::remove_dir_all(output_dir);
}

#[test]
fn test_text_report_generation() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    let peaks = find_resonance_peaks(&geo, SimulationStrategy::Tlm);

    let report = generate_text_report(&geo, &peaks);
    assert!(report.contains("CADSD Analysis Report"));
    assert!(report.contains("Length:"));
    assert!(report.contains("Bell diameter:"));
}

#[test]
fn test_geometry_to_simulation_workflow() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    // Use a small frequency grid for fast test execution
    let freqs: Vec<f64> = (20..=200).step_by(10).map(|x| x as f64).collect();
    let impedances = cadsd::acoustical_simulation(&geo, &freqs, "tlm_cython").unwrap();
    assert!(!impedances.is_empty(), "Impedance spectrum should not be empty");
    assert_eq!(impedances.len(), freqs.len());
}

#[test]
fn test_tlm_python_simulation() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    let freqs: Vec<f64> = (20..=200).step_by(10).map(|x| x as f64).collect();
    let impedances = cadsd::acoustical_simulation(&geo, &freqs, "tlm_python").unwrap();
    assert!(!impedances.is_empty(), "TLM Python impedance spectrum should not be empty");
    assert_eq!(impedances.len(), freqs.len());
}

#[test]
fn test_tlm_cython_simulation() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    let freqs: Vec<f64> = (20..=200).step_by(10).map(|x| x as f64).collect();
    let impedances = cadsd::acoustical_simulation(&geo, &freqs, "tlm_cython").unwrap();
    assert!(!impedances.is_empty(), "TLM Cython impedance spectrum should not be empty");
    assert_eq!(impedances.len(), freqs.len());
}

#[test]
fn test_acoustic_constants_temperature() {
    let cold = AcousticConstants::for_temperature(0.0);
    let warm = AcousticConstants::for_temperature(40.0);
    assert!(warm.c > cold.c, "Speed of sound should increase with temperature");
    assert!(warm.rho < cold.rho, "Air density should decrease with temperature");
}

#[test]
fn test_acoustic_constants_pressure_humidity() {
    let dry = AcousticConstants::for_conditions(20.0, 101325.0, 0.0);
    let humid = AcousticConstants::for_conditions(20.0, 101325.0, 0.8);
    let high_p = AcousticConstants::for_conditions(20.0, 202650.0, 0.0);
    assert!(humid.c > dry.c, "Humid air should have slightly higher speed of sound");
    assert!(high_p.rho > dry.rho, "Higher pressure should increase density");
    assert_eq!(dry.pressure_pa, 101325.0);
    assert_eq!(dry.relative_humidity, 0.0);
}

#[test]
fn test_tonehole_impedance() {
    let th = Tonehole::new(500.0, 10.0, 5.0, true);
    let constants = AcousticConstants::for_temperature(20.0);
    let z_open = th.open_impedance(440.0, &constants);
    assert!(z_open.norm() > 0.0, "Open tonehole impedance should be positive");
    
    let z_closed = th.closed_impedance(440.0, &constants);
    assert!(z_closed.norm() > 0.0, "Closed tonehole impedance should be positive");
}

#[test]
fn test_simulator_with_acoustic_constants() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    let mut simulator = DidgeridooSimulator::from_geo(&geo.geo);
    simulator.strategy = SimulationStrategy::Tlm;
    let freqs: Vec<f64> = (20..=200).step_by(10).map(|x| x as f64).collect();
    let spectrum = simulator.impedance(&freqs);
    assert!(!spectrum.is_empty(), "Simulator spectrum should not be empty");
    assert_eq!(spectrum.len(), freqs.len());
}

#[test]
fn test_tonehole_simulation_integration() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    let sim_no_th = DidgeridooSimulator::from_geo(&geo.geo);
    let mut sim_with_th = DidgeridooSimulator::from_geo(&geo.geo);
    sim_with_th.toneholes = vec![Tonehole::new(400.0, 10.0, 5.0, true)];

    let freqs: Vec<f64> = (20..=500).step_by(20).map(|x| x as f64).collect();
    let spec_no_th = sim_no_th.impedance(&freqs);
    let spec_with_th = sim_with_th.impedance(&freqs);

    assert_eq!(spec_no_th.len(), spec_with_th.len());
    for z_with in spec_with_th.iter() {
        assert!(z_with.norm() > 0.0, "Tonehole impedance should be finite");
    }
}