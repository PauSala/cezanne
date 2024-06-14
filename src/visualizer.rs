use std::sync::{Arc, Mutex};

use crate::{DELTA, FF_BUFF, HEIGHT, I_BUFF, MARGIN, SCALE_FACTOR, WIDTH};
const AA_SQRT: usize = 4;
const AA_PIXEL_SIZE: usize = AA_SQRT.pow(2);

pub struct Visualizer {
    colors: Vec<u32>,
    circles: Vec<usize>,
}

impl Visualizer {
    pub fn new() -> Self {
        let colors = Visualizer::gradient(FF_BUFF);
        Self {
            colors,
            circles: Visualizer::classify_circles(WIDTH * SCALE_FACTOR, HEIGHT * SCALE_FACTOR),
        }
    }

    /// Precomputes at which circle belongs every pixel in an upscaled buffer
    pub fn classify_circles(width: usize, height: usize) -> Vec<usize> {
        let circles = FF_BUFF;
        let max_radius = (width.min(height) / 2) - MARGIN;
        let radius_step = max_radius / circles;

        let mut all_circles = [0; FF_BUFF];
        for i in 0..FF_BUFF {
            let radius = (circles - i) * radius_step;
            all_circles[i] = radius;
        }

        let mut res: Vec<usize> = vec![FF_BUFF; width * height];

        let cx = width / 2;
        let cy = height / 2;
        for y in 0..height {
            for x in 0..width {
                let dx = x as isize - cx as isize;
                let dy = y as isize - cy as isize;
                let dist_sq = dx * dx + dy * dy;
                let index = y * width + x;
                for (i, circle) in all_circles.iter().rev().enumerate() {
                    if dist_sq <= (circle * circle) as isize {
                        res[index] = i;
                        break;
                    }
                }
            }
        }
        res
    }

    pub fn update_window_buffer(
        &self,
        prev_buffer: &mut [f32; FF_BUFF],
        window_buffer: &mut [u32; WIDTH * HEIGHT],
        scaled_buffer: &mut Vec<u32>,
        freqs: Arc<Mutex<[f32; I_BUFF / 2]>>,
        elapsed_milis: f64,
    ) -> Option<()> {
        let ff = freqs.lock().unwrap();
        if ff.len() < 1 {
            return None;
        }
        for i in 0..prev_buffer.len() {
            prev_buffer[i] +=
                (ff[i] - prev_buffer[i]) as f32 * (elapsed_milis / 1000.0) as f32 * DELTA;
        }
        self.draw_circles(&prev_buffer, window_buffer, scaled_buffer, &self.colors);
        Some(())
    }

    /// Draws circles based on precomputations in the upscaled buffer
    fn draw_circles(
        &self,
        freqs: &[f32; FF_BUFF],
        window_buffer: &mut [u32; WIDTH * HEIGHT],
        scaled_buffer: &mut Vec<u32>,
        colors: &Vec<u32>,
    ) {
        // Map each frequency to its corresponding color
        let color_map: Vec<u32> = freqs
            .iter()
            .map(|&sample| {
                let color_index =
                    ((sample * (colors.len() as f32)).round() as usize) % colors.len();
                colors[color_index]
            })
            .collect();

        // Use the precomputed circle classification to set buffer colors
        for (i, &circle_index) in self.circles.iter().enumerate() {
            if circle_index < color_map.len() {
                scaled_buffer[i] = color_map[circle_index];
            }
        }

        self.downscale(&scaled_buffer, window_buffer);
    }

    fn average_colors(&self, colors: &[u32]) -> u32 {
        let mut sum_r = 0u32;
        let mut sum_g = 0u32;
        let mut sum_b = 0u32;
        let count = colors.len() as u32;

        for &color in colors {
            sum_r += (color >> 16) & 0xFF;
            sum_g += (color >> 8) & 0xFF;
            sum_b += color & 0xFF;
        }

        let avg_r = sum_r / count;
        let avg_g = sum_g / count;
        let avg_b = sum_b / count;

        (avg_r << 16) | (avg_g << 8) | avg_b
    }

    /// Downscale for Supersampling Antialiasing (SSAA)
    fn downscale(&self, buffer: &[u32], window_buffer: &mut [u32; WIDTH * HEIGHT]) {
        let length = buffer.len() as f64;
        let width = length.sqrt() as usize;
        let mut colors = [0; AA_PIXEL_SIZE];

        for r in 0..WIDTH {
            for c in 0..HEIGHT {
                let mut idx = 0;
                for dy in 0..AA_SQRT {
                    for dx in 0..AA_SQRT {
                        let orig_row = r * SCALE_FACTOR + dx;
                        let orig_col = c * SCALE_FACTOR + dy;
                        let orig_index = orig_row * width + orig_col;
                        if orig_index < buffer.len() {
                            // Some indices are out of scope, just ignore them
                            colors[idx] = buffer[orig_index];
                        }
                        idx += 1;
                    }
                }
                window_buffer[r * WIDTH + c] = self.average_colors(&colors);
            }
        }
    }

    fn gradient(len: usize) -> Vec<u32> {
        let g = colorgrad::magma();
        let c = g.colors(len);
        c.iter()
            .map(|color| {
                let r = (color.r * 255.0) as u32;
                let g = (color.g * 255.0) as u32;
                let b = (color.b * 255.0) as u32;
                (r << 16) | (g << 8) | b
            })
            .collect::<Vec<u32>>()
    }
}
