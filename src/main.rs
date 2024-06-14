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

const MARGIN: usize = 8;
const WIDTH: usize = 512 + 2 * MARGIN;
const HEIGHT: usize = WIDTH;
const DELTA: f32 = 2.0;
const I_BUFF: usize = 1024;
const FF_BUFF: usize = 128;
const SCALE_FACTOR: usize = 2;

fn main() -> Result<()> {
    let (device, config) = device()?;
    let channels = config.channels();

    // Input buffer (samples)
    let ib = Arc::new(Mutex::new(Vec::new()));
    let ib_clone = Arc::clone(&ib);

    // Setup input stream
    let stream = AudioInput::new(ib_clone, device, &config)?;
    stream.play()?;

    //Output buffer (frequencies)
    let ob = Arc::new(Mutex::new([0.0; I_BUFF / 2]));
    let ob_clone = Arc::clone(&ob);

    //Setup output stream
    let ib_clone = Arc::clone(&ib);
    init_output_stream(ib_clone, ob_clone, channels);

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
    let mut db = [0.0; FF_BUFF];
    let mut wb = [0; WIDTH * HEIGHT];
    let mut scaled_b = vec![0; WIDTH * SCALE_FACTOR * HEIGHT * SCALE_FACTOR];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap();
        // println!("Elapsed top: {:?}ms", elapsed.as_millis());
        // precission issues
        let millis = elapsed.as_nanos() as f64 / 1_000_000.0;

        //Get visualiation buffer
        let ob_clone = Arc::clone(&ob);
        // let xstart = SystemTime::now();
        let b = visualizer.update_window_buffer(&mut db, &mut wb, &mut scaled_b, ob_clone, millis);
        // let xend = SystemTime::now();
        // let elapsed = xend.duration_since(xstart).unwrap();
        // println!("Elapsed: {:?}ms", elapsed.as_nanos() as f64 / 1_000_000.0);
        if let Some(_) = b {
            window.update_with_buffer(&wb, WIDTH, HEIGHT).unwrap();
        }
        start = end;
    }
    Ok(())
}
