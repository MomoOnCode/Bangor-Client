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
// TODO [x] First get starbuck and check if libsodium is actually Salsa20
//      - make UDP / TCP Sender/Reciever
//     [x] - use libsodium+noise-xx for TCP
//
// =======================
//  UDP VOICE PROTOCOL TODO
// =======================

//  Core stack:
// -  Opus for audio encoding/decoding
// -  ring for AEAD encryption (ChaCha20-Poly1305)
// -  snow (Noise) for session key exchange

// 1. [ ] After successful TCP login:
//      - Derive UDP session key via `session.export(..., Some(b"udp-key"))`
//      - Bind UDP socket on client
//      - Send encrypted "register" message to server

// 2. [ ] On server:
//      - Bind UDP socket (e.g., 0.0.0.0:5555)
//      - Listen for "register" packets
//      - Decrypt using key tied to login session
//      - Map `SocketAddr → session key`

// 3. [ ] Decide on crypto backend:
//      -  Use `ring::aead` with `CHACHA20_POLY1305`
//      -   Generate new UDP key per Noise session

// 4. [ ] Implement counter-based nonce system:
//      - 12-byte nonce = 4-byte prefix + 8-byte counter
//      - Reset counter for each new session
//      - Ensure nonce never repeats per key

// 5. [ ] Build encrypt/decrypt helpers:
//      - `fn encrypt_udp(payload: &[u8], key: &LessSafeKey, nonce: Nonce) -> Vec<u8>`
//      - `fn decrypt_udp(packet: &[u8], key: &LessSafeKey) -> Result<Vec<u8>, _>`

// 6. [ ] Implement UDP send loop on client:
//      - Capture + encode Opus frame
//      - Encrypt with current nonce/key
//      - Send [nonce || ciphertext] to server
//      - Increment nonce counter

// 7. [ ] Implement UDP receive loop on client:
//      - Read [nonce || ciphertext]
//      - Decrypt using current key
//      - Decode Opus and play audio

// 8. [ ] On server:
//      - Receive from multiple clients
//      - Verify/decrypt voice packet using mapped key
//      - Forward to other peers in same room

// 9. [ ] Later: add replay protection (if needed)
//      - Optional: store recent nonces per addr
//      - Prevent duplicate packet injection

// =======================
//       Let’s go.
// =======================
mod audio;
mod crypto;
mod net;

use audio::AudioLoopback;
use net::{establish_command, read_message, send_message, spawn_main_session_task};
use std::io::{self, Write};
// use tokio::net::UdpSocket;
#[tokio::main]
async fn main() {
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
                //Main login loop, it can use a little refactoring
                // but just establishes a handhskae then spawns
                // new tokio worker on success

                let payload = crypto::build_login_payload();

                let login_result = tokio::spawn(async move {
                    let (mut stream, mut session) = establish_command("192.168.0.19:42069")
                        .await
                        .expect("Connection to server failed..");

                    send_message(&mut stream, &mut session, &payload)
                        .await
                        .unwrap();

                    let response = read_message(&mut stream, &mut session).await.unwrap();
                    (response, stream, session)
                })
                .await
                .unwrap(); // unwrap the JoinHandle

                // unpack returned stuff into local bindings
                let (response, mut stream, mut session) = login_result;

                if response.r#type != "login_response" {
                    println!("❌ Login failed: {}", response.message);
                } else if response.success {
                    println!("✅ Login OK: {}", response.message);
                    //start main tokio worker for command
                    let join_handle =
                        spawn_main_session_task(stream, session).expect("Main session task failed");
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
