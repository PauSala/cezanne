use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use anyhow::Result;

use cezanne::{
    input_stream::{device, AudioInput},
    output_stream::init_output_stream,
    visualizer::Visualizer,
    FF_BUFF, HEIGHT, I_BUFF, SCALE_FACTOR, WIDTH,
};
use minifb::{HasWindowHandle, Key, Window, WindowOptions};

fn main() -> Result<()> {
    let (device, config) = device()?;
    let channels = config.channels();

    // Input buffer (samples)
    let ib = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
    let ib_clone = Arc::clone(&ib);

    // Setup input stream
    let stream = AudioInput::new(ib_clone, device, &config)?;
    stream.play()?;

    //Output buffer (frequencies)
    let ob = Arc::new(Mutex::new([0.0; I_BUFF / 2]));
    let ob_clone = Arc::clone(&ob);

    //Setup output stream
    let terminate_flag = Arc::new(Mutex::new(false));
    let terminate_flag_clone = Arc::clone(&terminate_flag);
    let ib_clone = Arc::clone(&ib);
    let thread_handle = init_output_stream(ib_clone, ob_clone, channels, terminate_flag_clone);

    let mut window = Window::new(
        "Frequency Spectrum",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    let visualizer = Visualizer::new();
    window.set_target_fps(60);
    let mut start = SystemTime::now();
    let mut db = [0.0; FF_BUFF];
    let mut wb = [0; WIDTH * HEIGHT];
    let mut scaled_b = vec![0; WIDTH * SCALE_FACTOR * HEIGHT * SCALE_FACTOR];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap();
        // precission issues
        let millis = elapsed.as_nanos() as f64 / 1_000_000.0;

        //Get visualiation buffer
        let ob_clone = Arc::clone(&ob);
        let b = visualizer.update_window_buffer(&mut db, &mut wb, &mut scaled_b, ob_clone, millis);
        if let Some(_) = b {
            window.update_with_buffer(&wb, WIDTH, HEIGHT).unwrap();
        }
        start = end;
    }

    // Set termination flag to signal thread to return
    {
        let mut should_terminate = terminate_flag.lock().unwrap();
        *should_terminate = true;
    }
    let _ = thread_handle.join();
    drop(window);
    stream.pause()?;
    drop(stream);
    Ok(())
}
