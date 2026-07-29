//! Integration tests for the cadsd wrapper crate.
//! These tests exercise the end-to-end workflow:
//! geometry creation → simulation → analysis → report generation.

use cadsd::geo::Geo;
use cadsd::sim::DidgeridooSimulator;
use cadsd::visualization::{create_analysis_report, generate_text_report};
use std::fs;
use std::path::Path;

#[test]
fn test_full_workflow_cone() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    let simulator = DidgeridooSimulator::from_geo(&geo.geo);

    let output_dir = "test_integration_cone";
    let result = create_analysis_report(&geo, &simulator, output_dir);
    assert!(result.is_ok(), "Analysis report creation failed");

    assert!(Path::new(&format!("{}/geometry.png", output_dir)).exists());
    assert!(Path::new(&format!("{}/spectrum.png", output_dir)).exists());
    assert!(Path::new(&format!("{}/report.txt", output_dir)).exists());

    let report = fs::read_to_string(&format!("{}/report.txt", output_dir)).unwrap();
    assert!(report.contains("CADSD Analysis Report"));
    assert!(report.contains("Geometry Summary"));
    assert!(report.contains("Resonance Analysis"));

    let _ = fs::remove_dir_all(output_dir);
}

#[test]
fn test_full_workflow_with_bubble() {
    let mut geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    geo.add_bubble(500.0, 100.0, 50.0);

    let simulator = DidgeridooSimulator::from_geo(&geo.geo);

    let output_dir = "test_integration_bubble";
    let result = create_analysis_report(&geo, &simulator, output_dir);
    assert!(result.is_ok(), "Analysis report creation failed with bubble");

    let report = fs::read_to_string(&format!("{}/report.txt", output_dir)).unwrap();
    assert!(report.contains("CADSD Analysis Report"));

    let _ = fs::remove_dir_all(output_dir);
}

#[test]
fn test_text_report_generation() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    let simulator = DidgeridooSimulator::from_geo(&geo.geo);
    let peaks = simulator.find_resonance_peaks();

    let report = generate_text_report(&geo, &peaks);
    assert!(report.contains("CADSD Analysis Report"));
    assert!(report.contains("Length:"));
    assert!(report.contains("Bell diameter:"));
}

#[test]
fn test_geometry_to_simulator_workflow() {
    let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
    let simulator = DidgeridooSimulator::from_geo(&geo.geo);
    let freqs = cadsd::sim::grid::lin_grid(50.0, 500.0, 2.0);
    let spectrum = simulator.impedance(&freqs);
    assert!(!spectrum.is_empty(), "Spectrum should not be empty");
}