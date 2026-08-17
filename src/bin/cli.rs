//! User-friendly CLI for DidgeRust CADSD
//!
//! Designed for non-Rust developers to test scientific features via command line.
//!
//! Examples:
//!   cargo run --bin cli -- simulate cone --length 1500 --top 32 --bottom 65 --json
//!   cargo run --bin cli -- optimize cone --length 1500 --top 32 --bottom 65 --generations 20
//!   cargo run --bin cli -- validate cone --length 1500 --top 32 --bottom 65
//!   cargo run --bin cli -- ml primes --max-prime 17 --input 10
//!   cargo run --bin cli -- primes list --max-prime 100
//!   cargo run --bin cli -- waveguide cone --length 1500 --top 32 --bottom 65
//!   cargo run --bin cli -- tonehole --diameter 10 --depth 5 --freq 500

use cadsd::{
    Geo, sim::{DidgeridooSimulator, SimulationStrategy, AcousticConstants},
    evo::{EvolutionaryOptimizer, EvolutionParameters, MutationStrategy, CrossoverStrategy},
    loss::CompositeTairuaLoss,
    prime_conv::{PrimeGenerator, PrimeConvBlock},
    waveguide::{WaveguideSimulator},
    tonehole::Tonehole,
};
use num_complex::Complex64;
use std::time::Instant;

#[derive(Debug, Clone)]
struct SimConfig {
    geo_type: String,
    length: f64,
    top_diameter: f64,
    bottom_diameter: f64,
    segments: usize,
    freqs: Vec<f64>,
    strategy: SimulationStrategy,
    temperature: f64,
    pressure: f64,
    humidity: f64,
    json: bool,
}

#[derive(Debug, Clone)]
struct OptimizeConfig {
    geo_type: String,
    length: f64,
    top_diameter: f64,
    bottom_diameter: f64,
    segments: usize,
    generations: usize,
    population: usize,
    strategy: SimulationStrategy,
    temperature: f64,
    pressure: f64,
    humidity: f64,
    json: bool,
}

#[derive(Debug, Clone)]
struct ValidateConfig {
    geo_type: String,
    length: f64,
    top_diameter: f64,
    bottom_diameter: f64,
    segments: usize,
    freqs: Vec<f64>,
    json: bool,
}

fn parse_sim_config(args: &[String]) -> SimConfig {
    let mut config = SimConfig {
        geo_type: "cone".to_string(),
        length: 1500.0,
        top_diameter: 32.0,
        bottom_diameter: 65.0,
        segments: 30,
        freqs: (20..=2000).step_by(20).map(|x| x as f64).collect(),
        strategy: SimulationStrategy::Tlm,
        temperature: 20.0,
        pressure: 101325.0,
        humidity: 0.0,
        json: false,
    };

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--geo" => {
                config.geo_type = args.get(i + 1).unwrap_or(&"cone".to_string()).clone();
                i += 2;
            }
            "--length" => {
                config.length = args.get(i + 1).unwrap_or(&"1500".to_string()).parse().unwrap_or(1500.0);
                i += 2;
            }
            "--top" => {
                config.top_diameter = args.get(i + 1).unwrap_or(&"32".to_string()).parse().unwrap_or(32.0);
                i += 2;
            }
            "--bottom" => {
                config.bottom_diameter = args.get(i + 1).unwrap_or(&"65".to_string()).parse().unwrap_or(65.0);
                i += 2;
            }
            "--segments" => {
                config.segments = args.get(i + 1).unwrap_or(&"30".to_string()).parse().unwrap_or(30);
                i += 2;
            }
            "--temp" => {
                config.temperature = args.get(i + 1).unwrap_or(&"20".to_string()).parse().unwrap_or(20.0);
                i += 2;
            }
            "--pressure" => {
                config.pressure = args.get(i + 1).unwrap_or(&"101325".to_string()).parse().unwrap_or(101325.0);
                i += 2;
            }
            "--humidity" => {
                config.humidity = args.get(i + 1).unwrap_or(&"0".to_string()).parse().unwrap_or(0.0);
                i += 2;
            }
                "--strategy" => {
                config.strategy = match args.get(i + 1).unwrap_or(&String::from("tlm")).as_str() {
                    "tlm" => SimulationStrategy::Tlm,
                    "waveguide" => SimulationStrategy::Waveguide,
                    "complex" => SimulationStrategy::ComplexImpedance,
                    _ => SimulationStrategy::Tlm,
                };
                i += 2;
            }
            "--freq-start" => {
                if let Ok(v) = args.get(i + 1).unwrap_or(&"20".to_string()).parse::<f64>() {
                    config.freqs = config.freqs.iter().map(|&f| if f < v { v } else { f }).collect();
                }
                i += 2;
            }
            "--freq-end" => {
                if let Ok(v) = args.get(i + 1).unwrap_or(&"2000".to_string()).parse::<f64>() {
                    config.freqs = config.freqs.iter().map(|&f| if f > v { v } else { f }).collect();
                }
                i += 2;
            }
            "--json" => {
                config.json = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    config
}

fn parse_optimize_config(args: &[String]) -> OptimizeConfig {
    let mut config = OptimizeConfig {
        geo_type: "cone".to_string(),
        length: 1500.0,
        top_diameter: 32.0,
        bottom_diameter: 65.0,
        segments: 30,
        generations: 20,
        population: 50,
        strategy: SimulationStrategy::Tlm,
        temperature: 20.0,
        pressure: 101325.0,
        humidity: 0.0,
        json: false,
    };

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--geo" => {
                config.geo_type = args.get(i + 1).unwrap_or(&"cone".to_string()).clone();
                i += 2;
            }
            "--length" => {
                config.length = args.get(i + 1).unwrap_or(&"1500".to_string()).parse().unwrap_or(1500.0);
                i += 2;
            }
            "--top" => {
                config.top_diameter = args.get(i + 1).unwrap_or(&"32".to_string()).parse().unwrap_or(32.0);
                i += 2;
            }
            "--bottom" => {
                config.bottom_diameter = args.get(i + 1).unwrap_or(&"65".to_string()).parse().unwrap_or(65.0);
                i += 2;
            }
            "--segments" => {
                config.segments = args.get(i + 1).unwrap_or(&"30".to_string()).parse().unwrap_or(30);
                i += 2;
            }
            "--generations" => {
                config.generations = args.get(i + 1).unwrap_or(&"20".to_string()).parse().unwrap_or(20);
                i += 2;
            }
            "--population" => {
                config.population = args.get(i + 1).unwrap_or(&"50".to_string()).parse().unwrap_or(50);
                i += 2;
            }
            "--temp" => {
                config.temperature = args.get(i + 1).unwrap_or(&"20".to_string()).parse().unwrap_or(20.0);
                i += 2;
            }
            "--pressure" => {
                config.pressure = args.get(i + 1).unwrap_or(&"101325".to_string()).parse().unwrap_or(101325.0);
                i += 2;
            }
            "--humidity" => {
                config.humidity = args.get(i + 1).unwrap_or(&"0".to_string()).parse().unwrap_or(0.0);
                i += 2;
            }
            "--json" => {
                config.json = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    config
}

fn parse_validate_config(args: &[String]) -> ValidateConfig {
    let mut config = ValidateConfig {
        geo_type: "cone".to_string(),
        length: 1500.0,
        top_diameter: 32.0,
        bottom_diameter: 65.0,
        segments: 30,
        freqs: vec![200.0, 400.0, 600.0, 800.0],
        json: false,
    };

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--geo" => {
                config.geo_type = args.get(i + 1).unwrap_or(&"cone".to_string()).clone();
                i += 2;
            }
            "--length" => {
                config.length = args.get(i + 1).unwrap_or(&"1500".to_string()).parse().unwrap_or(1500.0);
                i += 2;
            }
            "--top" => {
                config.top_diameter = args.get(i + 1).unwrap_or(&"32".to_string()).parse().unwrap_or(32.0);
                i += 2;
            }
            "--bottom" => {
                config.bottom_diameter = args.get(i + 1).unwrap_or(&"65".to_string()).parse().unwrap_or(65.0);
                i += 2;
            }
            "--segments" => {
                config.segments = args.get(i + 1).unwrap_or(&"30".to_string()).parse().unwrap_or(30);
                i += 2;
            }
            "--json" => {
                config.json = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    config
}

fn make_geo(config: &SimConfig) -> Geo {
    match config.geo_type.as_str() {
        "cone" => Geo::make_cone(config.length, config.top_diameter, config.bottom_diameter, config.segments),
        "kigali" => Geo::make_kigali(config.length, config.top_diameter, config.bottom_diameter, 0.3, config.segments),
        "mbeya" => Geo::make_mbeya(config.length, config.top_diameter, config.bottom_diameter, 0.3, config.segments),
        _ => Geo::make_cone(config.length, config.top_diameter, config.bottom_diameter, config.segments),
    }
}

fn run_simulate(config: SimConfig) {
    let geo = make_geo(&config);
    let mut sim = DidgeridooSimulator::from_geo(&geo.geo);
    sim.strategy = config.strategy;
    sim.acoustic_constants = AcousticConstants::for_conditions(config.temperature, config.pressure, config.humidity);

    println!("Running {:?} simulation...", config.strategy);
    let start = Instant::now();
    let spectrum = sim.impedance(&config.freqs);
    let elapsed = start.elapsed();

    if config.json {
        let mut results = Vec::new();
        for (f, z) in config.freqs.iter().zip(spectrum.iter()) {
            results.push(serde_json::json!({
                "frequency_hz": f,
                "impedance_magnitude": z.norm(),
                "impedance_real": z.re,
                "impedance_imag": z.im,
            }));
        }
        let output = serde_json::json!({
            "strategy": format!("{:?}", config.strategy),
            "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
            "results": results,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("Simulation completed in {:.2?}", elapsed);
        println!("Frequency (Hz) | Impedance (Pa·s/m³) | Real | Imag");
        println!("--------------|----------------------|------|------");
        for (f, z) in config.freqs.iter().zip(spectrum.iter()) {
            println!("{:>13.2} | {:>20.6} | {:>5.3} | {:>5.3}", f, z.norm(), z.re, z.im);
        }
    }
}

fn run_optimize(config: OptimizeConfig) {
    let _geo = make_geo(&SimConfig {
        geo_type: config.geo_type.clone(),
        length: config.length,
        top_diameter: config.top_diameter,
        bottom_diameter: config.bottom_diameter,
        segments: config.segments,
        freqs: Vec::new(),
        strategy: config.strategy,
        temperature: config.temperature,
        pressure: config.pressure,
        humidity: config.humidity,
        json: config.json,
    });

    println!("Running evolutionary optimization for {} generations...", config.generations);
    let start = Instant::now();

    let _constants = AcousticConstants::for_conditions(config.temperature, config.pressure, config.humidity);
    let loss_fn = CompositeTairuaLoss::with_default_components(50.0);
    let genome_template = cadsd::evo::KigaliGenome::new(
        config.segments,
        config.top_diameter,
        config.bottom_diameter,
        config.bottom_diameter * 1.5,
        config.length,
        config.length * 0.5,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );

    let params = EvolutionParameters {
        population_size: config.population,
        generation_size: (config.population / 2).max(1),
        num_generations: config.generations,
        mutation_rate: 0.1,
        crossover_rate: 0.7,
        elite_size: 5,
        mutation_strategy: MutationStrategy::Gaussian,
        crossover_strategy: CrossoverStrategy::SinglePoint,
        convergence_patience: 10,
        convergence_threshold: 1e-6,
    };

    let mut optimizer = EvolutionaryOptimizer::with_random_population(
        Box::new(loss_fn),
        &genome_template,
        config.population,
        params,
    );

    let result = optimizer.evolve();
    let elapsed = start.elapsed();

    let best_loss = match result {
        Ok(ref genome) => genome.loss().unwrap_or(f64::INFINITY),
        Err(_) => f64::INFINITY,
    };

    if config.json {
        let output = serde_json::json!({
            "generations": config.generations,
            "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
            "best_loss": best_loss,
            "status": result.is_ok(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("Optimization completed in {:.2?}", elapsed);
        println!("Best loss: {:.6}", best_loss);
    }
}

fn run_validate(config: ValidateConfig) {
    let geo = make_geo(&SimConfig {
        geo_type: config.geo_type.clone(),
        length: config.length,
        top_diameter: config.top_diameter,
        bottom_diameter: config.bottom_diameter,
        segments: config.segments,
        freqs: config.freqs.clone(),
        strategy: SimulationStrategy::Tlm,
        temperature: 20.0,
        pressure: 101325.0,
        humidity: 0.0,
        json: config.json,
    });

    println!("Running TLM vs analytical validation...");
    let start = Instant::now();

    let segments = cadsd::sim::create_segments_from_geo(&geo.geo);
    let constants = AcousticConstants::default();
    let tlm_results: Vec<(f64, num_complex::Complex<f64>)> = config.freqs.iter()
        .map(|&f| {
            let z = cadsd::sim::cadsd_ze_with_losses(&segments, f, &constants, true, &[]);
            (f, z)
        })
        .collect();

    let analytical_results: Vec<(f64, num_complex::Complex<f64>)> = config.freqs.iter()
        .map(|&f| {
            let z = cadsd::validation::analytical_impedance_cylinder(config.length / 1000.0, config.top_diameter / 2000.0, f, &constants);
            (f, z)
        })
        .collect();

    let elapsed = start.elapsed();

    if config.json {
        let comparisons: Vec<serde_json::Value> = tlm_results.iter().zip(analytical_results.iter())
            .map(|((_f_tlm, z_tlm), (_f_ana, z_ana))| {
                let rel_error = if z_ana.norm() > 1e-12 {
                    ((z_tlm - z_ana).norm() / z_ana.norm()) as f64
                } else {
                    0.0
                };
                serde_json::json!({
                    "frequency_hz": f_tlm,
                    "tlm_magnitude": z_tlm.norm(),
                    "analytical_magnitude": z_ana.norm(),
                    "relative_error": rel_error,
                })
            })
            .collect();
        let output = serde_json::json!({
            "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
            "comparisons": comparisons,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("Validation completed in {:.2?}", elapsed);
        println!("Frequency (Hz) | TLM | Analytical | Rel Error");
        println!("--------------|-----|------------|----------");
        for ((_f_tlm, z_tlm), (_f_ana, z_ana)) in tlm_results.iter().zip(analytical_results.iter()) {
            let rel_error = if z_ana.norm() > 1e-12 {
                ((z_tlm - z_ana).norm() / z_ana.norm()) as f64
            } else {
                0.0
            };
            println!("{:>13.2} | {:>5.3} | {:>10.3} | {:>8.4}", f_tlm, z_tlm.norm(), z_ana.norm(), rel_error);
        }
    }
}

// ==================== ADVANCED FEATURE COMMANDS ====================

fn run_ml_primes(args: &[String]) {
    let mut max_prime = 17;
    let mut input_len = 10;
    let mut json = false;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--max-prime" => {
                max_prime = args.get(i + 1).unwrap_or(&"17".to_string()).parse().unwrap_or(17);
                i += 2;
            }
            "--input" => {
                input_len = args.get(i + 1).unwrap_or(&"10".to_string()).parse().unwrap_or(10);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    println!("Running ComplexPrimeMLP demo with prime kernels up to {}", max_prime);
    let start = Instant::now();

    let model = PrimeConvBlock::new(max_prime, 1, 2);
    let input: Vec<Complex64> = (0..input_len)
        .map(|i| Complex64::new(i as f64, (i as f64).sin()))
        .collect();

    let output = model.forward(&input);
    let elapsed = start.elapsed();
    let prime_gen = PrimeGenerator::new(max_prime);

    if json {
        let output_json: Vec<serde_json::Value> = output.iter()
            .map(|c| serde_json::json!({
                "real": c.re,
                "imag": c.im,
                "norm": c.norm(),
            }))
            .collect();
        let result = serde_json::json!({
            "max_prime": max_prime,
            "input_len": input_len,
            "output_len": output.len(),
            "primes_used": prime_gen.prime_list(),
            "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
            "output": output_json,
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else {
        println!("PrimeConvBlock forward pass completed in {:.2?}", elapsed);
        println!("Input length: {}", input.len());
        println!("Output length: {}", output.len());
        println!("Primes used: {:?}", prime_gen.prime_list());
        println!("\nFirst 10 output values:");
        for (i, val) in output.iter().take(10).enumerate() {
            println!("  [{}] real={:8.4} imag={:8.4} norm={:8.4}", i, val.re, val.im, val.norm());
        }
    }
}

fn run_primes_list(args: &[String]) {
    let mut max_prime = 100;
    let mut json = false;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--max" => {
                max_prime = args.get(i + 1).unwrap_or(&"100".to_string()).parse().unwrap_or(100);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let generator = PrimeGenerator::new(max_prime);
    let primes = generator.prime_list();

    if json {
        let primes_json: Vec<serde_json::Value> = primes.iter().map(|&p| serde_json::json!(p)).collect();
        let result = serde_json::json!({
            "max_prime": max_prime,
            "count": primes.len(),
            "primes": primes_json,
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else {
        println!("Primes up to {}:", max_prime);
        println!("Count: {}", primes.len());
        println!("{:?}", primes);
    }
}

fn run_waveguide(args: &[String]) {
    let mut geo_type = "cone".to_string();
    let mut length = 1500.0;
    let mut top_diameter = 32.0;
    let mut bottom_diameter = 65.0;
    let mut segments = 30;
    let freqs: Vec<f64> = (20..=2000).step_by(20).map(|x| x as f64).collect();
    let mut json = false;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--geo" => {
                geo_type = args.get(i + 1).unwrap_or(&"cone".to_string()).clone();
                i += 2;
            }
            "--length" => {
                length = args.get(i + 1).unwrap_or(&"1500".to_string()).parse().unwrap_or(1500.0);
                i += 2;
            }
            "--top" => {
                top_diameter = args.get(i + 1).unwrap_or(&"32".to_string()).parse().unwrap_or(32.0);
                i += 2;
            }
            "--bottom" => {
                bottom_diameter = args.get(i + 1).unwrap_or(&"65".to_string()).parse().unwrap_or(65.0);
                i += 2;
            }
            "--segments" => {
                segments = args.get(i + 1).unwrap_or(&"30".to_string()).parse().unwrap_or(30);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let geo = match geo_type.as_str() {
        "cone" => Geo::make_cone(length, top_diameter, bottom_diameter, segments),
        "kigali" => Geo::make_kigali(length, top_diameter, bottom_diameter, 0.3, segments),
        "mbeya" => Geo::make_mbeya(length, top_diameter, bottom_diameter, 0.3, segments),
        _ => Geo::make_cone(length, top_diameter, bottom_diameter, segments),
    };

    println!("Running 3D waveguide simulation...");
    let start = Instant::now();

    let sim = WaveguideSimulator::new(&geo);
    let spectrum = sim.compute_impedance(&freqs);
    let elapsed = start.elapsed();

    if json {
        let results: Vec<serde_json::Value> = freqs.iter().zip(spectrum.iter())
            .map(|(&f, z)| {
                serde_json::json!({
                    "frequency_hz": f,
                    "impedance_magnitude": z.norm(),
                    "impedance_real": z.re,
                    "impedance_imag": z.im,
                })
            })
            .collect();
        let output = serde_json::json!({
            "geo_type": geo_type,
            "length": length,
            "segments": sim.n_segments(),
            "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
            "results": results,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("Waveguide simulation completed in {:.2?}", elapsed);
        println!("Geometry: {} ({} segments, {:.1} mm)", geo_type, sim.n_segments(), sim.total_length() * 1000.0);
        println!("Frequency (Hz) | Impedance | Real | Imag");
        println!("--------------|-----------|------|------");
        for (f, z) in freqs.iter().zip(spectrum.iter()) {
            println!("{:>13.2} | {:>9.4} | {:>5.3} | {:>5.3}", f, z.norm(), z.re, z.im);
        }
    }
}

fn run_tonehole(args: &[String]) {
    let mut diameter = 10.0;
    let mut depth = 5.0;
    let mut is_open = true;
    let freqs: Vec<f64> = (20..=2000).step_by(20).map(|x| x as f64).collect();
    let mut json = false;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--diameter" => {
                diameter = args.get(i + 1).unwrap_or(&"10".to_string()).parse().unwrap_or(10.0);
                i += 2;
            }
            "--depth" => {
                depth = args.get(i + 1).unwrap_or(&"5".to_string()).parse().unwrap_or(5.0);
                i += 2;
            }
            "--closed" => {
                is_open = false;
                i += 1;
            }
            "--open" => {
                is_open = true;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    println!("Computing tonehole impedance spectrum (d={}mm, depth={}mm, {})...", 
        diameter, depth, if is_open { "open" } else { "closed" });
    let start = Instant::now();

    let th = Tonehole::new(500.0, diameter, depth, is_open);
    let constants = AcousticConstants::default();
    let spectrum: Vec<Complex64> = freqs.iter()
        .map(|&f| if is_open { th.open_impedance(f, &constants) } else { th.closed_impedance(f, &constants) })
        .collect();

    let elapsed = start.elapsed();

    if json {
        let results: Vec<serde_json::Value> = freqs.iter().zip(spectrum.iter())
            .map(|(&f, z)| {
                serde_json::json!({
                    "frequency_hz": f,
                    "impedance_magnitude": z.norm(),
                    "impedance_real": z.re,
                    "impedance_imag": z.im,
                })
            })
            .collect();
        let output = serde_json::json!({
            "diameter_mm": diameter,
            "depth_mm": depth,
            "is_open": is_open,
            "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
            "results": results,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("Tonehole impedance computed in {:.2?}", elapsed);
        println!("Diameter: {}mm, Depth: {}mm, Type: {}", diameter, depth, if is_open { "open" } else { "closed" });
        println!("Frequency (Hz) | Impedance | Real | Imag");
        println!("--------------|-----------|------|------");
        for (f, z) in freqs.iter().zip(spectrum.iter()) {
            println!("{:>13.2} | {:>9.4} | {:>5.3} | {:>5.3}", f, z.norm(), z.re, z.im);
        }
    }
}

fn print_help() {
    println!("DidgeRust CADSD CLI");
    println!("==================");
    println!();
    println!("USAGE:");
    println!("    cli <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    simulate    Run acoustic impedance simulation");
    println!("    optimize    Run evolutionary optimization");
    println!("    validate    Validate TLM against analytical solution");
    println!("    ml          Run ML / prime-conv demo");
    println!("    primes      List prime numbers");
    println!("    waveguide   Run 3D waveguide simulation");
    println!("    tonehole    Compute tonehole impedance spectrum");
    println!("    help        Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Simulate a cone and print results");
    println!("    cli simulate cone --length 1500 --top 32 --bottom 65");
    println!();
    println!("    # Simulate with JSON output");
    println!("    cli simulate cone --length 1500 --top 32 --bottom 65 --json");
    println!();
    println!("    # Optimize a cone for 20 generations");
    println!("    cli optimize cone --length 1500 --top 32 --bottom 65 --generations 20");
    println!();
    println!("    # Validate TLM vs analytical for a cone");
    println!("    cli validate cone --length 1500 --top 32 --bottom 65");
    println!();
    println!("    # Run ML prime-conv demo");
    println!("    cli ml primes --max-prime 17 --input 10");
    println!();
    println!("    # List prime numbers");
    println!("    cli primes list --max 100");
    println!();
    println!("    # Run 3D waveguide simulation");
    println!("    cli waveguide cone --length 1500 --top 32 --bottom 65");
    println!();
    println!("    # Compute tonehole impedance");
    println!("    cli tonehole --diameter 10 --depth 5 --freq 500");
    println!();
    println!("OPTIONS:");
    println!("    --geo <TYPE>        Geometry type: cone, kigali, mbeya (default: cone)");
    println!("    --length <MM>       Bore length in mm (default: 1500)");
    println!("    --top <MM>          Top diameter in mm (default: 32)");
    println!("    --bottom <MM>       Bottom diameter in mm (default: 65)");
    println!("    --segments <N>      Number of segments (default: 30)");
    println!("    --generations <N>   Number of optimization generations (default: 20)");
    println!("    --population <N>    Population size for optimization (default: 50)");
    println!("    --temp <C>          Temperature in Celsius (default: 20)");
    println!("    --pressure <PA>     Pressure in Pa (default: 101325)");
    println!("    --humidity <0-1>    Relative humidity (default: 0)");
    println!("    --strategy <TYPE>   Simulation strategy: tlm, fdtd, waveguide, complex (default: tlm)");
    println!("    --max-prime <N>     Max prime kernel size for ML demo (default: 17)");
    println!("    --input <N>         Input vector length for ML demo (default: 10)");
    println!("    --diameter <MM>     Tonehole diameter in mm (default: 10)");
    println!("    --depth <MM>        Tonehole depth in mm (default: 5)");
    println!("    --open / --closed   Tonehole type (default: open)");
    println!("    --json              Output results as JSON");
    println!("    --help              Show this help message");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    let command = args[1].as_str();
    match command {
        "simulate" => {
            let config = parse_sim_config(&args);
            run_simulate(config);
        }
        "optimize" => {
            let config = parse_optimize_config(&args);
            run_optimize(config);
        }
        "validate" => {
            let config = parse_validate_config(&args);
            run_validate(config);
        }
        "ml" => {
            run_ml_primes(&args);
        }
        "primes" => {
            run_primes_list(&args);
        }
        "waveguide" => {
            run_waveguide(&args);
        }
        "tonehole" => {
            run_tonehole(&args);
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Run 'cli help' for usage information.");
            std::process::exit(1);
        }
    }
}
