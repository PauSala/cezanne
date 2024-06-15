use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
};

use crate::{analizer::frequencies, I_BUFF};

pub fn init_output_stream(
    input_buffer: Arc<(Mutex<Vec<f32>>, Condvar)>,
    output_buffer: Arc<Mutex<[f32; I_BUFF / 2]>>,
    channels: u16,
    terminate: Arc<Mutex<bool>>,
) -> thread::JoinHandle<()> {
    let join_handle = thread::spawn(move || loop {
        {
            let should_terminate = terminate.lock().unwrap();
            if *should_terminate {
                break;
            }
        }
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
    join_handle
}
