use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::{analizer::frequencies, I_BUFF};

pub fn init_output_stream(
    input_buffer: Arc<Mutex<Vec<f32>>>,
    output_buffer: Arc<Mutex<[f32; I_BUFF / 2]>>,
    channels: u16,
) {
    thread::spawn(move || loop {
        let buffer = {
            let buffer = input_buffer.lock().unwrap();
            if buffer.len() > 0 {
                Some(buffer)
            } else {
                None
            }
        };
        if let Some(buffer) = buffer {
            if buffer.len() >= I_BUFF {
                let ff = frequencies(&buffer, channels);
                let lock = output_buffer.lock();
                match lock {
                    Ok(mut output_buffer) => {
                        for i in 0..output_buffer.len() {
                            output_buffer[i] = ff[i];
                        }
                    }
                    Err(e) => {
                        panic!("{}", e)
                    }
                }
            }
        }
        // Let the CPU to take a break
        std::thread::sleep(std::time::Duration::from_millis(50));
    });
}
