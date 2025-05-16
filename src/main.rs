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
// __________________________
// TODO First get starbuck and check if libsodium is actually Salsa20
//      - make UDP / TCP Sender/Reciever
//      - use libsodium+noise-xx for TCP
mod audio;
mod crypto;
mod net;

use audio::AudioLoopback;
use net::read_message;
use std::io::{self, Write};

fn main() {
    // TODO Webcam buddy
    // let mut video = AudioLoopback::new();
    let mut loopback = AudioLoopback::new();

    loop {
        println!("\nMenu:");
        println!("1. Commence Audio Test");
        println!("2. Stop the Audio");
        println!("3. Mute/Unmute");
        println!("4. Connect to server TCP");
        println!("5. Show status");
        println!("6. Exit...");
        println!("Choose an option: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => loopback.start(),
            "2" => loopback.stop(),
            "3" => loopback.toggle_mute(),
            "4" => {
                let payload = net::build_login_payload();
                let (mut stream, mut session) = net::establish_command("192.168.0.19:420").unwrap();
                net::send_message(&mut stream, &mut session, &payload).unwrap();
                let response = read_message(&mut stream, &mut session).unwrap();
                println!("{:?}", response);
                if response.r#type != "login_response" {
                    println!("❌ Login failed: {}", response.message);
                } else if response.success {
                    println!("✅ Login OK: {}", response.message);
                } else {
                    println!("❌ Login failed: {}", response.message);
                }
            }
            "5" => loopback.status(),
            "6" => {
                loopback.stop();
                break;
            }
            _ => println!("Invalid selection."),
        }
    }
}
