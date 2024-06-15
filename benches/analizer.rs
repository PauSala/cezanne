use cezanne::analizer::frequencies;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn generate_samples(num_samples: usize) -> Vec<f32> {
    // Example: Generate random samples for benchmarking
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let samples: Vec<f32> = (0..num_samples).map(|_| rng.gen::<f32>()).collect();
    samples
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let samples = generate_samples(1000); // Generate 1000 samples for benchmarking
    let channels = 2; // Example: Number of channels
    c.bench_function("frequencies", |b| {
        b.iter(|| frequencies(black_box(&samples), black_box(channels)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
