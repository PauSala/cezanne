use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
};

use crate::{analizer::frequencies, I_BUFF};

pub fn init_output_stream(
    input_buffer: Arc<(Mutex<Vec<f32>>, Condvar)>,
    output_buffer: Arc<Mutex<[f32; I_BUFF / 2]>>,
    channels: u16,
) {
    let input_buffer = Arc::clone(&input_buffer);
    let output_buffer = Arc::clone(&output_buffer);

    thread::spawn(move || loop {
        let (lock, cvar) = &*input_buffer;
        let mut buffer = lock.lock().unwrap();
        while buffer.len() < I_BUFF {
            buffer = cvar.wait(buffer).unwrap();
        }

        let ff = frequencies(&buffer, channels);
        let mut output_buffer = output_buffer.lock().unwrap();
        for i in 0..output_buffer.len() {
            output_buffer[i] = ff[i];
        }
        buffer.clear();
    });
}
