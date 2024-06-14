use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use anyhow::Result;

use input_stream::{device, AudioInput};
use minifb::{Key, Window, WindowOptions};
use output_stream::init_output_stream;
use visualizer::Visualizer;

pub mod analizer;
pub mod input_stream;
pub mod output_stream;
pub mod visualizer;

const WIDTH: usize = 512;
const HEIGHT: usize = WIDTH;
const DELTA: f32 = 2.0;
const BUF_LEN: usize = 1024;
const DB_LEN: usize = 128;
const SCALE_FACTOR: usize = 2;

fn main() -> Result<()> {
    let (device, config) = device()?;
    let channels = config.channels();

    // Input buffer (samples)
    let i_buffer = Arc::new(Mutex::new(Vec::new()));
    let i_buffer_clone = Arc::clone(&i_buffer);

    // Setup input stream
    let stream = AudioInput::new(i_buffer_clone, device, &config)?;
    stream.play()?;

    //Output buffer (frequencies)
    let i_buffer_clone = Arc::clone(&i_buffer);
    let o_buffer = Arc::new(Mutex::new(Vec::new()));
    let o_buffer_clone = Arc::clone(&o_buffer);

    //Setup output stream
    init_output_stream(i_buffer_clone, o_buffer_clone, channels);

    let mut window = Window::new(
        "Frequency Spectrum",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    let visualizer = Visualizer::new(WIDTH, HEIGHT, SCALE_FACTOR, DELTA);
    window.set_target_fps(30);
    let mut start = SystemTime::now();
    let mut curr = vec![0.0; DB_LEN];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap();
        // println!("Elapsed: {:?}", elapsed.as_millis());
        // precission issues
        let millis = elapsed.as_nanos() as f64 / 1_000_000.0;

        //Get visualiation buffer
        let shrd_ff_cln = Arc::clone(&o_buffer);
        let b = visualizer.get_live_buffer(&mut curr, shrd_ff_cln, millis);
        if let Some(b) = b {
            window.update_with_buffer(&b, WIDTH, HEIGHT).unwrap();
        }

        start = end;
    }
    Ok(())
}
