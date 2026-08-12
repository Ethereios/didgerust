use plotters::prelude::*;
use std::f64::consts::PI;
use num_complex::Complex64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a chart showing Gaussian vs Prime mutation patterns
    let root = BitMapBackend::new("mutation_comparison.png", (1000, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Mutation Strategy Comparison: Gaussian vs Prime Sequence", ("sans-serif", 20).into_font())
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0..100, 0f64..1f64)?;
    
    chart.configure_mesh()
        .x_desc("Gene Index")
        .y_desc("Mutation Amplitude")
        .x_labels(5)
        .y_labels(10)
        .draw()?;
    
    // Generate Gaussian mutation pattern (normal distribution)
    let rng = rand::thread_rng();
    let gaussian_mutations: Vec<f64> = (0..100)
        .map(|i| {
            let normal = rand_distr::Normal::new(0.0, 0.1).unwrap();
            let noise: f64 = normal.sample(&rng);
            noise.abs() // Take absolute value for visualization
        })
        .collect();
    
    // Generate Prime mutation pattern
    let primes = generate_primes(100);
    let prime_mutations: Vec<f64> = (0..100)
        .map(|i| {
            let prime = primes[i % primes.len()];
            let base_amplitude = prime as f64 / 100.0;
            let noise = rand::random::<f64>() * base_amplitude;
            noise
        })
        .collect();
    
    // Plot Gaussian mutations
    chart.draw_series(LineSeries::new(
        (0..100).zip(gaussian_mutations.iter()),
        &RED,
    ))?
    .label("Gaussian Mutation")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    // Plot Prime mutations
    chart.draw_series(LineSeries::new(
        (0..100).zip(prime_mutations.iter()),
        &BLUE,
    ))?
    .label("Prime Sequence Mutation")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
    chart.configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE)
        .draw()?;
    
    root.present()?;
    println!("Saved mutation_comparison.png");
    
    // Also create a frequency distribution comparison
    let root2 = BitMapBackend::new("mutation_distribution.png", (1000, 600)).into_drawing_area();
    root2.fill(&WHITE)?;
    
    let mut chart2 = ChartBuilder::on(&root2)
        .caption("Mutation Amplitude Distribution Comparison", ("sans-serif", 20).into_font())
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..0.2, 0..100)?;
    
    chart2.configure_mesh()
        .x_desc("Mutation Amplitude")
        .y_desc("Frequency (count)")
        .draw()?;
    
    // Create histograms for both distributions
    let gaussian_counts = create_histogram(&gaussian_mutations, 20);
    let prime_counts = create_histogram(&prime_mutations, 20);
    
    chart2.draw_series(ColumnSeries::new(
        gaussian_counts.iter().map(|(x, y)| (*x, *y as u32)),
        &RED,
    ))?.label("Gaussian").style(&RED);
    
    chart2.draw_series(ColumnSeries::new(
        prime_counts.iter().map(|(x, y)| (*x, *y as u32)),
        &BLUE,
    ))?.label("Prime").style(&BLUE);
    
    chart2.configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE)
        .draw()?;
    
    root2.present()?;
    println!("Saved mutation_distribution.png");
    
    Ok(())
}

fn generate_primes(limit: usize) -> Vec<usize> {
    let mut sieve = vec![true; limit + 1];
    sieve[0] = false;
    sieve[1] = false;
    
    for i in 2..=((limit as f64).sqrt() as usize) {
        if sieve[i] {
            for j in (i * i..=limit).step_by(i) {
                sieve[j] = false;
            }
        }
    }
    
    sieve.into_iter().enumerate()
        .filter(|(_, is_prime)| *is_prime)
        .map(|(p, _)| p)
        .collect()
}

fn create_histogram(data: &[f64], bins: usize) -> Vec<(f64, usize)> {
    let min = 0.0;
    let max = 0.2;
    let bin_width = (max - min) / bins as f64;
    
    let mut counts = vec![0usize; bins];
    
    for &value in data {
        if value >= min && value < max {
            let bin = ((value - min) / bin_width) as usize;
            if bin < bins {
                counts[bin] += 1;
            }
        }
    }
    
    (0..bins)
        .map(|i| ((min + i as f64 * bin_width), counts[i]))
        .collect()
}