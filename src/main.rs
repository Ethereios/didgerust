//! Example usage of the CADSD library
//!
//! This example demonstrates how to use the CADSD library to:
//! 1. Create didgeridoo geometries
//! 2. Perform acoustic simulations
//! 3. Run evolutionary optimization
//! 4. Visualize results

use cadsd::geo::Geo;
use cadsd::sim::DidgeridooSimulator;
use cadsd::evo::{EvolutionaryOptimizer, KigaliGenome, EvolutionParameters, Genome};
use cadsd::loss::{CompositeTairuaLoss, FrequencyTuningLoss, ModalDensityLoss};
use cadsd::visualization::create_analysis_report;
use std::error::Error;


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    println!("=== CADSD Rust Example ===\n");
    
    // Example 1: Basic geometry and simulation
    example_basic_simulation()?;
    
    // Example 2: Evolutionary optimization
    example_evolutionary_optimization()?;
    
    // Example 3: Advanced parametric shapes
    example_parametric_shapes()?;
    
    Ok(())
}

fn example_basic_simulation() -> Result<(), Box<dyn Error>> {
    println!("Example 1: Basic Geometry and Simulation");
    println!("========================================");
    
    // Create a simple conical didgeridoo
    let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);
    println!("Created conical didgeridoo:");
    println!("  Length: {:.1} mm", geo.length());
    println!("  Bell diameter: {:.1} mm", geo.bellsize());
    println!("  Volume: {:.0} mm³", geo.compute_volume());
    
    // Create simulator
    let simulator = DidgeridooSimulator::from_geo(&geo.points);
    
    // Compute impedance spectrum
    println!("\nComputing impedance spectrum...");
    let freqs = cadsd::sim::grid::lin_grid(50.0, 500.0, 2.0);
    let spectrum = simulator.impedance(&freqs);
    println!("Computed {} frequency points", spectrum.len());
    
    // Find resonance peaks
    let peaks = simulator.find_resonance_peaks();
    println!("Found {} resonance peaks:", peaks.len());
    for (i, peak) in peaks.iter().enumerate() {
        println!("  Peak {}: {:.1} Hz (impedance: {:.2})", 
                i + 1, peak.frequency, peak.impedance);
    }
    
    // Create analysis report
    println!("\nGenerating analysis report...");
    create_analysis_report(&geo, &simulator, "example1_output")?;
    println!("Analysis completed (report saved to example1_output)");
    
    Ok(())
}

fn example_evolutionary_optimization() -> Result<(), Box<dyn Error>> {
    println!("\n\nExample 2: Evolutionary Optimization");
    println!("====================================");
    
    // Create a Kigali-style genome template
    let genome_template = KigaliGenome::new(
        20,     // n_segments
        32.0,   // mouth diameter
        50.0,   // min bell diameter
        80.0,   // max bell diameter
        1800.0, // max length
        1500.0, // min length
        2,      // number of bubbles
        0.3,    // smoothness
        0.2,    // bell accent
        300.0,  // bell start
    );
    
    println!("Created Kigali genome template with {} genes", genome_template.genome().len());
    
    // Create loss function for optimization
    // Target: drone in D (~73.4 Hz) with good harmonic structure
    let mut composite_loss = CompositeTairuaLoss::new(5.0); // 5 cent error tolerance
    
    composite_loss.add_component(
        "frequency_tuning".to_string(),
        Box::new(FrequencyTuningLoss::new(
            vec![f64::log2(73.4)], // Target D1 frequency
            vec![0.5],             // Target impedance
            vec![1.0],             // Weight
        ))
    );
    
    composite_loss.add_component(
        "modal_density".to_string(),
        Box::new(ModalDensityLoss::new(50.0, 0.5)) // Reward clustered peaks
    );
    
    // Create evolutionary optimizer
    let evolution_params = EvolutionParameters {
        population_size: 30,
        generation_size: 20,
        num_generations: 20,
        mutation_rate: 0.15,
        crossover_rate: 0.7,
        elite_size: 3,
    };
    
    let mut optimizer = EvolutionaryOptimizer::with_random_population(
        Box::new(composite_loss),
        &genome_template,
        evolution_params.population_size,
        evolution_params.clone(),
    );
    
    println!("Starting evolutionary optimization...");
    println!("  Population size: {}", evolution_params.population_size);
    println!("  Generations: {}", evolution_params.num_generations);
    println!("  Target frequency: 73.4 Hz (D1)");
    
    // Run optimization (this will take some time)
    let start_time = std::time::Instant::now();
    let best_genome = optimizer.evolve()?;
    let duration = start_time.elapsed();
    
    println!("Optimization completed in {:?}", duration);
    println!("Best genome ID: {}", best_genome.id());
    println!("Best loss: {:.6}", best_genome.loss().unwrap_or(f64::INFINITY));
    
    // Convert best genome to geometry and analyze
    let best_geo = best_genome.genome2geo();
    let best_simulator = DidgeridooSimulator::from_geo(&best_geo.points);
    let best_peaks = best_simulator.find_resonance_peaks();
    
    println!("\nBest design analysis:");
    println!("  Length: {:.1} mm", best_geo.length());
    println!("  Bell diameter: {:.1} mm", best_geo.bellsize());
    println!("  Found {} peaks:", best_peaks.len());
    
    for (i, peak) in best_peaks.iter().enumerate() {
        let note_name = frequency_to_note(peak.frequency);
        println!("    Peak {}: {:.1} Hz ({}) - impedance: {:.2}", 
                i + 1, peak.frequency, note_name, peak.impedance);
    }
    
    // Create analysis report for best design
    println!("\nGenerating analysis report for best design...");
    create_analysis_report(&best_geo, &best_simulator, "example2_output")?;
    println!("Best design analysis completed (report saved to example2_output)");
    
    Ok(())
}

fn example_parametric_shapes() -> Result<(), Box<dyn Error>> {
    println!("\n\nExample 3: Parametric Shapes");
    println!("============================");
    
    // Create different parametric shapes
    
    // 1. Basic cone (reference)
    let cone = Geo::make_cone(1600.0, 32.0, 60.0, 25);
    println!("1. Basic cone:");
    println!("   Length: {:.0} mm, Bell: {:.0} mm", cone.length(), cone.bellsize());
    
    // 2. Cone with bubble
    let mut cone_bubble = cone.clone();
    cone_bubble.add_bubble(800.0, 200.0, 70.0); // pos=800mm, width=200mm, height=70mm
    println!("2. Cone with bubble:");
    println!("   Length: {:.0} mm, Max diameter: {:.0} mm", 
             cone_bubble.length(), cone_bubble.get_max_diameter());
    
    // 3. Stretched cone
    let mut stretched_cone = cone.clone();
    stretched_cone.stretch(1.2); // 20% longer
    println!("3. Stretched cone:");
    println!("   Length: {:.0} mm, Bell: {:.0} mm", stretched_cone.length(), stretched_cone.bellsize());
    
    // Analyze all shapes
    let shapes = vec![
        ("basic_cone", cone),
        ("cone_with_bubble", cone_bubble),
        ("stretched_cone", stretched_cone),
    ];
    
    for (name, geo) in &shapes {
        let simulator = DidgeridooSimulator::from_geo(&geo.points);
        let peaks = simulator.find_resonance_peaks();
        
        println!("\n{} analysis:", name);
        println!("  Found {} peaks", peaks.len());
        if !peaks.is_empty() {
            println!("  Fundamental: {:.1} Hz ({})", 
                    peaks[0].frequency, 
                    frequency_to_note(peaks[0].frequency));
        }
    }
    
    // Create comparison report
    println!("\nGenerating comparison reports...");
    std::fs::create_dir_all("example3_output")?;
    
    for (name, geo) in &shapes {
        let simulator = DidgeridooSimulator::from_geo(&geo.points);
        create_analysis_report(geo, &simulator, &format!("example3_output/{}", name))?;
    }
    
    println!("Comparison reports saved to example3_output/");
    
    Ok(())
}

/// Convert frequency to note name (simplified)
fn frequency_to_note(freq: f64) -> String {
    // A4 = 440 Hz reference
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    
    if freq <= 0.0 {
        return "Invalid".to_string();
    }
    
    // Calculate MIDI note number
    let midi_note = 69.0 + 12.0 * (freq / 440.0).log2();
    let note_index = (midi_note.round() as i32) % 12;
    let octave = ((midi_note.round() as i32) / 12) - 1;
    
    if note_index >= 0 && note_index < 12 {
        format!("{}{}", note_names[note_index as usize], octave)
    } else {
        format!("{:.1} Hz", freq)
    }
}