use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cadsd::loss::{CompositeTairuaLoss, BentEffectiveLengthLoss, LossComponent};
use cadsd::sim::create_segments_from_geo_with_curvature;
use cadsd::{Geo, evo::{KigaliGenome, LossFunction}};

fn bench_composite_loss(c: &mut Criterion) {
    let _geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);
    let loss = CompositeTairuaLoss::with_default_components(50.0);
    let genome = KigaliGenome::new(30, 32.0, 50.0, 80.0, 1800.0, 1500.0, 0, 0.3, 0.0, 300.0, 0);

    c.bench_function("composite_loss_via_genome", |b| {
        b.iter(|| {
            let _ = loss.calculate(black_box(&genome));
        })
    });
}

fn bench_bent_effective_length(c: &mut Criterion) {
    let loss = BentEffectiveLengthLoss::new(1.0, 2.0, 0.016, 0.25);
    let freqs: Vec<f64> = vec![100.0, 200.0, 400.0];
    let amps: Vec<f64> = vec![1.0, 0.8, 0.6];
    let indices: Vec<usize> = vec![0, 1, 2];

    c.bench_function("bent_effective_length_loss", |b| {
        b.iter(|| {
            let _ = loss.calculate(
                black_box(&freqs),
                black_box(&amps),
                black_box(&freqs),
                black_box(&amps),
                black_box(&indices),
            );
        })
    });
}

fn bench_segment_creation_with_curvature(c: &mut Criterion) {
    let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);
    c.bench_function("create_segments_with_curvature", |b| {
        b.iter(|| {
            let _ = create_segments_from_geo_with_curvature(
                black_box(&geo.geo),
                black_box(0.016),
                black_box(0.25),
            );
        })
    });
}

criterion_group!(
    benches,
    bench_composite_loss,
    bench_bent_effective_length,
    bench_segment_creation_with_curvature
);
criterion_main!(benches);
