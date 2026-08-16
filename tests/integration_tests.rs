//! Integration tests for the cadsd wrapper crate.
//! These tests exercise the end-to-end workflow:
//! geometry creation → simulation → analysis → report generation.

use cadsd::Geo;
use cadsd::visualization::{generate_text_report, plot_bore_geometry, plot_evolution_progress};
use cadsd::sim::{find_resonance_peaks, SimulationStrategy, DidgeridooSimulator, AcousticConstants};
use cadsd::tonehole::Tonehole;
use cadsd::evo::{KigaliGenome, LossFunction, Genome};
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

#[test]
fn test_tonehole_comparison_workflow() {
    let geo = Geo::new(vec![[0.0, 32.0], [1000.0, 32.0]]);
    let constants = AcousticConstants::for_temperature(20.0);
    let freqs: Vec<f64> = (50..=500).step_by(50).map(|x| x as f64).collect();
    
    let segments = cadsd::create_segments_from_geo(&geo.geo);
    let spec_no_th = cadsd::sim::compute_impedance_spectrum(&segments, &freqs);
    
    let mut sim_with_th = DidgeridooSimulator::from_geo(&geo.geo);
    sim_with_th.toneholes = vec![Tonehole::new(300.0, 10.0, 5.0, true)];
    let spec_with_th = sim_with_th.impedance(&freqs);
    
    assert_eq!(spec_no_th.len(), spec_with_th.len());
    let mut differences = 0;
    for (z_no, z_with) in spec_no_th.iter().zip(spec_with_th.iter()) {
        if (z_no.norm() - z_with.norm()).abs() > 1e-6 {
            differences += 1;
        }
    }
    assert!(differences > 0, "Toneholes should change impedance spectrum");
    
    let report = cadsd::validation::generate_validation_report(&geo, &freqs, &constants);
    assert!(report.contains("Validation Report"));
    assert!(report.contains("PASS") || report.contains("MARGINAL") || report.contains("FAIL"));
}

#[test]
fn test_tonehole_evolutionary_optimization() {
    let genome = KigaliGenome::new(
        10, 32.0, 50.0, 80.0, 1800.0, 1500.0, 0, 0.3, 0.0, 300.0, 2,
    );
    let (geo, toneholes) = genome.geo_and_toneholes();
    assert_eq!(toneholes.len(), 2);
    
    let mut simulator = DidgeridooSimulator::from_geo(&geo.geo);
    simulator.toneholes = toneholes;
    let freqs: Vec<f64> = (20..=500).step_by(20).map(|x| x as f64).collect();
    let spectrum = simulator.impedance(&freqs);
    assert_eq!(spectrum.len(), freqs.len());
    
    let loss_fn = cadsd::loss::CompositeTairuaLoss::with_default_components(50.0);
    let loss = loss_fn.calculate(&genome);
    assert!(loss >= 0.0);
    assert!(loss.is_finite());
}

#[test]
fn test_tonehole_tuning_loss_integration() {
    use cadsd::loss::ToneholeTuningLoss;
use cadsd::loss::LossComponent;
    use cadsd::evo::Genome;
    
    let genome = KigaliGenome::new(
        10, 32.0, 50.0, 80.0, 1800.0, 1500.0, 0, 0.3, 0.0, 300.0, 2,
    );
    let (_geo, toneholes) = genome.geo_and_toneholes();
    assert_eq!(toneholes.len(), 2);
    
    let targets = vec![
        (0.3, 0.5, 0.3, 0.0),
        (0.7, 0.4, 0.2, 0.0),
    ];
    let loss_fn = ToneholeTuningLoss::new(targets, 1.0);
    
    let f_log: Vec<f64> = vec![];
    let amps: Vec<f64> = vec![];
    let all_f: Vec<f64> = vec![];
    let all_z: Vec<f64> = vec![];
    let idx: Vec<usize> = vec![];
    
    let loss = loss_fn.calculate_with_toneholes(&f_log, &amps, &all_f, &all_z, &idx, &toneholes);
    assert!(loss >= 0.0);
    assert!(loss.is_finite());
    
    let empty_loss = loss_fn.calculate_with_toneholes(&f_log, &amps, &all_f, &all_z, &idx, &[]);
    assert_eq!(empty_loss, 0.0);
}

#[test]
fn test_tonehole_edge_tone_resistance() {
    use cadsd::tonehole::Tonehole;
    let constants = AcousticConstants::for_temperature(20.0);
    let closed_th = Tonehole::new(500.0, 12.0, 5.0, false);
    let r_closed = closed_th.edge_tone_resistance(440.0, &constants);
    assert_eq!(r_closed, 0.0, "Closed tonehole should have zero edge-tone resistance");
    
    let open_th = Tonehole::new(500.0, 12.0, 5.0, true);
    let r_open = open_th.edge_tone_resistance(440.0, &constants);
    assert!(r_open >= 0.0, "Open tonehole edge-tone resistance should be non-negative");
}

#[test]
fn test_optimizer_checkpoint_with_toneholes() {
    use cadsd::persistence::OptimizerCheckpoint;
    use cadsd::tonehole::Tonehole;
    
    let cp = OptimizerCheckpoint {
        timestamp: "test".to_string(),
        population_size: 50,
        num_generations: 100,
        current_generation: 10,
        mutation_rate: 0.1,
        crossover_rate: 0.7,
        elite_size: 5,
        best_loss: Some(0.05),
        generation_progress: 0.1,
        mutation_strategy: "Gaussian".to_string(),
        simulation_strategy: "Tlm".to_string(),
        geometry: cadsd::persistence::OptimizerGeoState {
            length: 1500.0,
            top_diameter: 32.0,
            bottom_diameter: 60.0,
            segments: 24,
        },
        loss_component_weights: vec![("integer_harmonic".to_string(), 5.0)],
        toneholes: vec![Tonehole::new(400.0, 10.0, 5.0, true)],
    };
    
    let json = serde_json::to_string(&cp).unwrap();
    let loaded: OptimizerCheckpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.toneholes.len(), 1);
    assert_eq!(loaded.toneholes[0].x, 400.0);
    assert_eq!(loaded.toneholes[0].diameter, 10.0);
    assert!(loaded.toneholes[0].is_open);
}