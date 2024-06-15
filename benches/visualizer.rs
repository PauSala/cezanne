use std::sync::{Arc, Mutex};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use cezanne::visualizer;
use rand::Rng;

const MARGIN: usize = 8;
const WIDTH: usize = 512 + 2 * MARGIN;
const HEIGHT: usize = WIDTH;
const I_BUFF: usize = 2014;
const FF_BUFF: usize = 128;

fn populate_fixed_sized_array<T, const N: usize>(array: &mut [T; N])
where
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    let mut rng = rand::thread_rng();
    for elem in array.iter_mut() {
        *elem = rng.gen();
    }
}

fn generate_samples<T>(num_samples: usize) -> Vec<T>
where
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    let mut rng = rand::thread_rng();
    let samples: Vec<T> = (0..num_samples).map(|_| rng.gen()).collect();
    samples
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let v = visualizer::Visualizer::new();
    let mut prev_buffer: [f32; FF_BUFF] = [0.0; FF_BUFF];
    populate_fixed_sized_array(&mut prev_buffer);

    let mut window_buffer: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];
    populate_fixed_sized_array(&mut window_buffer);

    let mut scaled_buffer: Vec<u32> = generate_samples(WIDTH * HEIGHT * 2 * 2);

    let mut ff = [0.0; I_BUFF / 2];
    populate_fixed_sized_array(&mut ff);
    let ob = Arc::new(Mutex::new(ff));

    c.bench_function("update_window_buffer", |b| {
        b.iter(|| {
            let ob_clone = Arc::clone(&ob);
            v.update_window_buffer(
                black_box(&mut prev_buffer),
                black_box(&mut window_buffer),
                black_box(&mut scaled_buffer),
                black_box(ob_clone),
                black_box(20.0),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
