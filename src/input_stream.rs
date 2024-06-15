use anyhow::Result;
use std::sync::{Arc, Condvar, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, InputCallbackInfo, SampleFormat, StreamConfig, SupportedStreamConfig,
};

use crate::I_BUFF;

pub fn device() -> Result<(Device, SupportedStreamConfig)> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("Failed to get default input device");
    let config = device.default_input_config()?;
    dbg!(&config);
    Ok((device, config))
}

pub struct AudioInput {
    stream: cpal::Stream,
}

impl AudioInput {
    pub fn new(
        shared_buffer: Arc<(Mutex<Vec<f32>>, Condvar)>,
        device: Device,
        config: &SupportedStreamConfig,
    ) -> Result<Self> {
        let shared_buffer_clone = Arc::clone(&shared_buffer);

        let sample_format = config.sample_format();
        let config: StreamConfig = config.clone().into();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match sample_format {
            SampleFormat::F32 => {
                let callback = move |data: &[f32], _: &InputCallbackInfo| {
                    let (lock, cvar) = &*shared_buffer_clone;
                    let mut buffer = lock.lock().unwrap();
                    if buffer.len() >= I_BUFF {
                        buffer.clear();
                    }
                    buffer.extend_from_slice(data);
                    cvar.notify_one();
                };
                device.build_input_stream(&config, callback, err_fn, None)?
            }
            _ => panic!("Unsupported sample format"),
        };

        Ok(AudioInput { stream })
    }

    pub fn play(&self) -> Result<()> {
        self.stream.play()?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.stream.pause()?;
        Ok(())
    }
}
