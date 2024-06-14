use rustfft::{num_complex::Complex, FftPlanner};

pub fn frequencies(samples: &[f32], channels: u16) -> Vec<f32> {
    let mut channel = Vec::with_capacity(samples.len() / channels as usize);
    let mut v: f32 = 0.0;
    for i in 0..samples.len() {
        v += samples[i];
        if (i + 1) % channels as usize == 0 {
            channel.push(v / channels as f32);
            v = 0.0;
        }
    }

    apply_hann_window(&mut channel);
    let mut frequencies = analyze_frequencies(&channel);
    normalize_freqs(&mut frequencies);
    frequencies
}

fn hann_window(n: usize) -> Vec<f32> {
    let mut window = Vec::with_capacity(n);
    for i in 0..n {
        let value = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n as f32 - 1.0)).cos());
        window.push(value);
    }
    window
}
fn apply_hann_window(samples: &mut [f32]) {
    let n = samples.len();
    let window = hann_window(n);

    for (i, sample) in samples.iter_mut().enumerate() {
        *sample *= window[i];
    }
}

fn analyze_frequencies(samples: &[f32]) -> Vec<f32> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(samples.len());
    let mut spectrum: Vec<Complex<f32>> = samples
        .iter()
        .map(|&sample| Complex::new(sample, 0.0))
        .collect();
    fft.process(&mut spectrum);
    let half = spectrum.len() / 2;
    spectrum
        .iter()
        .skip(1)
        .take(half)
        .map(|sample| sample.norm())
        .collect()
}

fn normalize_freqs(freqs: &mut Vec<f32>) {
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    // Apply a logarithmic function to compress the range
    // for row in freqs.iter_mut() {
    //     for value in row.iter_mut() {
    //         *value = (*value + 1.0).ln();
    //     }
    // }
    for &value in freqs.iter() {
        if value > max {
            max = value;
        }
        if value < min {
            min = value;
        }
    }
    for value in freqs.iter_mut() {
        *value = (*value - min) / (max - min);
    }
}
