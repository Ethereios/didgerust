use plotters::prelude::*;
use rand_distr::Distribution;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    
    let mut rng = rand::thread_rng();
    let gaussian_mutations: Vec<f64> = (0..100)
        .map(|_| {
            let normal: rand_distr::Normal<f64> = rand_distr::Normal::new(0.0, 0.1).unwrap();
            normal.sample(&mut rng).abs()
        })
        .collect();
    
    let primes = generate_primes(100);
    let prime_mutations: Vec<f64> = (0..100)
        .map(|i| {
            let prime = primes[i % primes.len()];
            let base_amplitude = prime as f64 / 100.0;
            rand::random::<f64>() * base_amplitude
        })
        .collect();
    
    chart.draw_series(LineSeries::new(
        (0..100).zip(gaussian_mutations.iter().cloned()),
        &RED,
    ))?
    .label("Gaussian Mutation")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    chart.draw_series(LineSeries::new(
        (0..100).zip(prime_mutations.iter().cloned()),
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
