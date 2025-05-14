// === Project Roadmap ===
// --1. CLI menu: start loopback / exit
// 2. Local audio loopback: mic input → speaker output
// 3. TCP signaling server: client join + message handling
// 4. Client → server: send Opus-encoded audio
// 5. Server → clients: forward audio streams
// 6. Add usernames and room IDs
// 7. Secure transport (TLS for control, SRTP for audio)
// ========================

// [Client] ---> TLS over TCP ---> [Server (Pi)]
// |                              |
// |-- "join room" / "mute" ---> |
// |<-- user list / stream map --|
// |
// |=== Encrypted audio(opus codec) via UDP or TCP===>
// encryption with SRTP??

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

fn main() {
    // share buffer that hold live audio data
    // let mut shared_buff: Option<Arc<Mutex<VecDeque<f32>>>> = None;
    // 2 streams variables created
    let mut input_stream: Option<cpal::Stream> = None;
    let mut output_stream: Option<cpal::Stream> = None;
    let mut muted = 0;

    loop {
        println!("\nMenu:");
        println!("1. Commence Audio");
        println!("2. Stop the Audio");
        println!("3. Mute/Unmute");
        println!("4. Exit...");
        println!("Choose an option: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => {
                if input_stream.is_none() && output_stream.is_none() {
                    println!("AUDIO IS COMMENCING...");
                    let shared_buff = Arc::new(Mutex::new(VecDeque::with_capacity(48_000)));
                    input_stream = Some(create_microphone_stream(shared_buff.clone()));
                    output_stream = Some(create_output_stream(shared_buff.clone()));
                    input_stream.as_ref().unwrap().play().unwrap();
                    output_stream.as_ref().unwrap().play().unwrap();
                } else {
                    println!("Audio has already commenced.");
                }
            }

            "2" => {
                println!("Stopping loopback");
                input_stream = None;
                output_stream = None;
                // shared_buff = None;
            }

            "3" => {
                if output_stream.is_some() && input_stream.is_some() && muted == 0 {
                    println!("Muting...");
                    muted = 1;
                    output_stream.as_ref().unwrap().pause().unwrap();
                } else if muted == 1 {
                    println!("Unmuting...");
                    muted = 0;
                    output_stream.as_ref().unwrap().play().unwrap();
                } else {
                    println!("audio not started...");
                }
            }

            "4" => {
                println!("Exiting...");
                break;
            }
            _ => println!("Invalid selection, try again."),
        }
    }
}

fn create_microphone_stream(shared_buff: Arc<Mutex<VecDeque<f32>>>) -> cpal::Stream {
    let input_buffer = shared_buff.clone();
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input found.");

    let supported_audio_config = device.default_input_config().unwrap();
    let config = supported_audio_config.config();

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
                while buffer.len() > 48_000 {
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
