// hocus pocus here is where i put opus

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
// use net::read_message;
use std::collections::VecDeque;
// use std::io::{self, Write};
use std::sync::{Arc, Mutex};

pub struct AudioLoopback {
    pub input_stream: Option<cpal::Stream>,
    pub output_stream: Option<cpal::Stream>,
    pub shared_buffer: Option<Arc<Mutex<VecDeque<f32>>>>,
    pub muted: bool,
}

impl AudioLoopback {
    pub fn new() -> Self {
        Self {
            input_stream: None,
            output_stream: None,
            shared_buffer: None,
            muted: false,
        }
    }

    pub fn start(&mut self) {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No input found.");
        let supported_audio_config = device.default_input_config().unwrap();
        let config = supported_audio_config.config();
        let sample_rate = config.sample_rate.0 as usize;

        if self.input_stream.is_none() && self.output_stream.is_none() {
            println!("Starting audio loopback...");
            let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(sample_rate)));
            self.input_stream = Some(create_microphone_stream(
                device,
                config,
                buffer.clone(),
                sample_rate,
            ));
            self.output_stream = Some(create_output_stream(buffer.clone()));
            self.shared_buffer = Some(buffer);

            self.input_stream.as_ref().unwrap().play().unwrap();
            self.output_stream.as_ref().unwrap().play().unwrap();
            self.muted = false;
        } else {
            println!("Audio is already running.");
        }
    }

    pub fn stop(&mut self) {
        if self.input_stream.is_some() || self.output_stream.is_some() {
            println!("Stopping audio loopback...");
            self.input_stream = None;
            self.output_stream = None;
            self.shared_buffer = None;
            self.muted = false;
        } else {
            println!("Loopback is not running.");
        }
    }

    pub fn toggle_mute(&mut self) {
        if let Some(ref stream) = self.output_stream {
            if self.muted {
                println!("Unmuting...");
                if let Some(ref buffer) = self.shared_buffer {
                    buffer.lock().unwrap().clear();
                }
                stream.play().unwrap();
                self.muted = false;
            } else {
                println!("Muting...");
                stream.pause().unwrap();
                self.muted = true;
            }
        } else {
            println!("Audio not started.");
        }
    }

    pub fn status(&self) {
        println!(
            "Input stream:  {}",
            if self.input_stream.is_some() {
                "running"
            } else {
                "not running"
            }
        );
        println!(
            "Output stream: {}",
            if self.output_stream.is_some() {
                "running"
            } else {
                "not running"
            }
        );
        println!("Muted:         {}", if self.muted { "yes" } else { "no" });
    }
}

fn create_microphone_stream(
    device: cpal::Device,
    config: cpal::StreamConfig,
    shared_buff: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: usize,
) -> cpal::Stream {
    let input_buffer = shared_buff.clone();

    // Define error callback
    let err_fn = |err| eprintln!("Stream error: {}", err);

    // Build and start the stream
    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _| {
                let mut buffer = input_buffer.lock().unwrap();
                for &sample in data {
                    buffer.push_back(sample);
                }
                while buffer.len() > sample_rate {
                    buffer.pop_front();
                }
            },
            err_fn,
            None,
        )
        .unwrap();
    // stream.play().unwrap();
    stream
}

fn create_output_stream(shared_buffer: Arc<Mutex<VecDeque<f32>>>) -> cpal::Stream {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device found.");
    let supported_config = device.default_output_config().unwrap();
    let config = supported_config.config();

    let output_buff = shared_buffer.clone();

    let err_fn = |err| eprintln!("Stream output error: {}", err);

    let stream = device
        .build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                let mut buffer = output_buff.lock().unwrap();
                for sample in output.iter_mut() {
                    *sample = buffer.pop_front().unwrap_or(0.0);
                }
            },
            err_fn,
            None,
        )
        .unwrap();
    // stream.play().unwrap();
    stream
}
