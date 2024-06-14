use std::sync::{Arc, Mutex};

use crate::{FF_BUFF, SCALE_FACTOR};

pub struct Visualizer {
    width: usize,
    height: usize,
    scale_factor: usize,
    delta: f32,
    colors: Vec<u32>,
    circles: Vec<usize>,
}

impl Visualizer {
    pub fn new(width: usize, height: usize, scale_factor: usize, delta: f32) -> Self {
        let colors = Visualizer::gradient(FF_BUFF);
        Self {
            width,
            height,
            scale_factor,
            delta,
            colors,
            circles: Visualizer::classify_circles(width * SCALE_FACTOR, height * SCALE_FACTOR),
        }
    }

    pub fn classify_circles(width: usize, height: usize) -> Vec<usize> {
        let circles = FF_BUFF;
        let max_radius = (width.min(height) / 2) - 1;
        let radius_step = max_radius / circles;

        let mut all_circles: Vec<usize> = Vec::new();

        for i in 0..FF_BUFF {
            let radius = (circles - i) * radius_step;
            all_circles.push(radius);
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

    pub fn get_live_buffer(
        &self,
        prev_buffer: &mut Vec<f32>,
        freqs: Arc<Mutex<Vec<f32>>>,
        elapsed_milis: f64,
    ) -> Option<Vec<u32>> {
        let ff = freqs.lock().unwrap();

        if ff.len() < 1 {
            return None;
        }
        for i in 0..prev_buffer.len() {
            prev_buffer[i] +=
                (ff[i] - prev_buffer[i]) as f32 * (elapsed_milis / 1000.0) as f32 * self.delta;
        }
        return Some(self.draw_circles(&prev_buffer, &self.colors));
    }

    fn draw_circles(&self, freqs: &[f32], colors: &Vec<u32>) -> Vec<u32> {
        let mut buffer: Vec<u32> =
            vec![0x000000; self.width * SCALE_FACTOR * self.height * SCALE_FACTOR];

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
                buffer[i] = color_map[circle_index];
            }
        }

        self.downscale(&buffer)
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

    fn downscale(&self, buffer: &[u32]) -> Vec<u32> {
        let mut new_buffer = vec![0xFFFFFF; self.width * self.height];

        for r in 0..self.width {
            for c in 0..self.height {
                let mut colors = Vec::new();
                for dy in 0..4 {
                    for dx in 0..4 {
                        let orig_row = r * self.scale_factor + dx;
                        let orig_col = c * self.scale_factor + dy;
                        let length = buffer.len() as f64;
                        let width = length.sqrt() as usize;
                        let orig_index = orig_row * width + orig_col;
                        if orig_index >= buffer.len() {
                            continue;
                        }
                        let c = buffer[orig_index].clone();
                        colors.push(c);
                    }
                }
                new_buffer[r * self.width + c] = self.average_colors(colors.as_slice());
            }
        }
        new_buffer
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
