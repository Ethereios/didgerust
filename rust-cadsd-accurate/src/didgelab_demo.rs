//! DidgeLab-style inverse design demo
//!
//! This demonstrates the exact same workflow as didgelab.com:
//! 1. Describe the sound you want (key, toots, overtones)
//! 2. Specify bore shape preferences
//! 3. Run evolutionary optimization
//! 4. Get matching geometry

use cadsd_accurate::inverse_design::InverseDesigner;
use cadsd_accurate::evo::{TargetSound, BoreShapePreference};
use cadsd_accurate::geo::Geo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          CADSD DidgeLab - Inverse Design Demo            ║");
    println!("║     Design the didgeridoo you hear in your head!         ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    // Example 1: Simple fundamental frequency design
    example_simple_design()?;
    
    // Example 2: Design with specific toots
    example_design_with_toots()?;
    
    // Example 3: Design with bore shape preference
    example_design_with_bore_shape()?;
    
    // Example 4: Complex multi-objective design
    example_complex_design()?;
    
    Ok(())
}

fn example_simple_design() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(60));
    println!("Example 1: Simple Design - Cylindrical Drone in D");
    println!("{}\n", "=".repeat(60));
    
    println!("🎵 Sound Description:");
    println!("   Key: D1 (≈73.4 Hz)");
    println!("   Style: Cylindrical drone");
    println!();
    
    // Create target sound
    let target = TargetSound::new(73.4)
        .with_bore_shape(BoreShapePreference::Cylindrical)
        .with_length_range(1200.0, 1800.0);
    
    // Run inverse design
    let designer = InverseDesigner::new()
        .with_population_size(30)
        .with_generations(50)
        .with_verbose(true);
    
    let result = designer.design(target)?;
    
    println!("\n📐 Resulting Geometry:");
    println!("   Length: {:.1} mm", result.geometry.length());
    println!("   Bell diameter: {:.1} mm", result.geometry.bellsize());
    println!("   Start diameter: {:.1} mm", result.geometry.diameter_at_x(0.0));
    println!("   Volume: {:.0} mm³", result.geometry.compute_volume());
    
    // Save to file
    result.geometry.to_file("example1_cone_d.txt")?;
    println!("\n💾 Geometry saved to: example1_cone_d.txt");
    
    Ok(())
}

fn example_design_with_toots() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(60));
    println!("Example 2: Design with Toots - D1 with D and A overtones");
    println!("{}\n", "=".repeat(60));
    
    println!("🎵 Sound Description:");
    println!("   Fundamental: D1 (≈73.4 Hz)");
    println!("   Toot 1: D4 (≈293.7 Hz)");
    println!("   Toot 2: A4 (≈440.0 Hz)");
    println!();
    
    // Create target sound with toots
    let target = TargetSound::new(73.4)
        .with_toot(293.7) // D4
        .with_toot(440.0)  // A4
        .with_bore_shape(BoreShapePreference::Conical)
        .with_overtones(vec![2, 3, 4, 5, 6]);
    
    // Run inverse design
    let designer = InverseDesigner::new()
        .with_population_size(40)
        .with_generations(60)
        .with_verbose(true);
    
    let result = designer.design(target)?;
    
    println!("\n📐 Resulting Geometry:");
    println!("   Length: {:.1} mm", result.geometry.length());
    println!("   Bell diameter: {:.1} mm", result.geometry.bellsize());
    println!("   Resonances found: {}", result.resonances.len());
    
    for (i, (freq, imp)) in result.resonances.iter().take(6).enumerate() {
        println!("     Peak {}: {:.1} Hz (impedance: {:.2e})", i + 1, freq, imp);
    }
    
    // Save to file
    result.geometry.to_file("example2_toots_da.txt")?;
    println!("\n💾 Geometry saved to: example2_toots_da.txt");
    
    Ok(())
}

fn example_design_with_bore_shape() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(60));
    println!("Example 3: Flared Horn Design - Custom Resonance");
    println!("{}\n", "=".repeat(60));
    
    println!("🎵 Sound Description:");
    println!("   Fundamental: F1 (≈87.3 Hz)");
    println!("   Style: Flared horn with strong bell");
    println!("   Bell range: 80-120 mm");
    println!();
    
    // Create target sound with flared bore
    let target = TargetSound::new(87.3)
        .with_bore_shape(BoreShapePreference::Flared)
        .with_bell_range(80.0, 120.0)
        .with_length_range(1000.0, 1600.0)
        .with_overtones(vec![2, 3, 4]);
    
    // Run inverse design
    let designer = InverseDesigner::new()
        .with_population_size(35)
        .with_generations(50)
        .with_verbose(true);
    
    let result = designer.design(target)?;
    
    println!("\n📐 Resulting Geometry:");
    println!("   Length: {:.1} mm", result.geometry.length());
    println!("   Bell diameter: {:.1} mm", result.geometry.bellsize());
    println!("   Start diameter: {:.1} mm", result.geometry.diameter_at_x(0.0));
    
    // Show bore profile
    println!("\n📊 Bore Profile:");
    let positions = vec![0.0, 0.25, 0.5, 0.75, 1.0];
    for &pos in &positions {
        let x = pos * result.geometry.length();
        let diam = result.geometry.diameter_at_x(x);
        println!("   At {:.0}% ({:.0} mm): {:.1} mm diameter", 
                pos * 100.0, x, diam);
    }
    
    // Save to file
    result.geometry.to_file("example3_flared_f.txt")?;
    println!("\n💾 Geometry saved to: example3_flared_f.txt");
    
    Ok(())
}

fn example_complex_design() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(60));
    println!("Example 4: Complex Multi-Objective Design");
    println!("{}\n", "=".repeat(60));
    
    println!("🎵 Sound Description:");
    println!("   Fundamental: A1 (≈55.0 Hz)");
    println!("   Toots: A2 (110 Hz), E3 (164.8 Hz), A3 (220 Hz)");
    println!("   Overtones: 2, 3, 4, 5, 6");
    println!("   Style: Conical with moderate flare");
    println!("   Length: 1500-2200 mm");
    println!("   Bell: 70-100 mm");
    println!();
    
    // Create complex target sound
    let target = TargetSound::new(55.0)
        .with_toot(110.0)   // A2
        .with_toot(164.8)   // E3
        .with_toot(220.0)   // A3
        .with_bore_shape(BoreShapePreference::Conical)
        .with_overtones(vec![2, 3, 4, 5, 6])
        .with_length_range(1500.0, 2200.0)
        .with_bell_range(70.0, 100.0);
    
    // Run inverse design with larger population
    let designer = InverseDesigner::new()
        .with_population_size(50)
        .with_generations(80)
        .with_top_n(5)
        .with_verbose(true);
    
    let result = designer.design(target)?;
    
    println!("\n📐 Best Result:");
    println!("   Final loss: {:.6}", result.loss);
    println!("   Generations: {}", result.generations);
    println!("   Fundamental: {:.2} Hz", result.fundamental_freq);
    println!("   Length: {:.1} mm", result.geometry.length());
    println!("   Bell diameter: {:.1} mm", result.geometry.bellsize());
    println!("   Volume: {:.0} mm³", result.geometry.compute_volume());
    
    println!("\n🎼 Resonance Analysis:");
    for (i, (freq, imp)) in result.resonances.iter().take(8).enumerate() {
        println!("   Peak {}: {:.1} Hz (impedance: {:.2e})", i + 1, freq, imp);
    }
    
    // Show top candidates
    println!("\n🏆 Top {} Candidates:", result.candidates.len());
    for (i, geo) in result.candidates.iter().enumerate() {
        println!("   Candidate {}: L={:.0}mm, Bell={:.0}mm, Vol={:.0}mm³",
                i + 1, geo.length(), geo.bellsize(), geo.compute_volume());
    }
    
    // Save all candidates
    for (i, geo) in result.candidates.iter().enumerate() {
        let filename = format!("example4_candidate_{}.txt", i + 1);
        geo.to_file(&filename)?;
    }
    println!("\n💾 All candidates saved to: example4_candidate_*.txt");
    
    // Save best geometry
    result.geometry.to_file("example4_best.txt")?;
    println!("💾 Best geometry saved to: example4_best.txt");
    
    Ok(())
}
