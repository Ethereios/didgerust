use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cadsd::Geo;
use cadsd::sim::{DidgeridooSimulator, SimulationStrategy};

fn benchmark_tlm(c: &mut Criterion) {
    let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);
    let simulator = DidgeridooSimulator::from_geo(&geo.geo);
    // Test frequency range
    let freqs: Vec<f64> = (20..=2000).step_by(10).map(|x| x as f64).collect();
    
    c.bench_function("TLM impedance calculation", |b| {
        b.iter(|| {
            let _ = simulator.impedance(black_box(&freqs));
        })
    });
}

fn benchmark_waveguide(c: &mut Criterion) {
    let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);
    let simulator = DidgeridooSimulator::with_strategy(
        &geo.geo,
        SimulationStrategy::Waveguide
    );
    // Test frequency range
    let freqs: Vec<f64> = (20..=2000).step_by(10).map(|x| x as f64).collect();
    
    c.bench_function("Waveguide impedance calculation", |b| {
        b.iter(|| {
            let _ = simulator.impedance(black_box(&freqs));
        })
    });
}

fn benchmark_complex_impedance(c: &mut Criterion) {
    let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);
    let simulator = DidgeridooSimulator::with_strategy(
        &geo.geo,
        SimulationStrategy::ComplexImpedance
    );
    // Test frequency range
    let freqs: Vec<f64> = (20..=2000).step_by(10).map(|x| x as f64).collect();
    
    c.bench_function("Complex Impedance calculation", |b| {
        b.iter(|| {
            let _ = simulator.impedance(black_box(&freqs));
        })
    });
}

criterion_group!(benches, benchmark_tlm, benchmark_waveguide, benchmark_complex_impedance);
criterion_main!(benches);