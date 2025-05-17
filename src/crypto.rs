// use std::io::BufRead;

// all crypto stuff can go here
use snow::{Builder, HandshakeState, TransportState, params::NoiseParams};

use crate::net::Message;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub async fn complete_handshake(
    stream: &mut tokio::net::TcpStream,
    mut handshake: HandshakeState,
) -> Result<TransportState, Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1024];
    let mut msg_out = [0u8; 1024];

    // Noise_XX always has 3 steps: write → read → write
    for step in 0..3 {
        if step % 2 == 0 {
            // write step
            let len = handshake.write_message(&[], &mut msg_out)?;
            stream.write_u16(len as u16).await?;
            stream.write_all(&msg_out[..len]).await?;
        } else {
            // read step
            let len = stream.read_u16().await? as usize;
            stream.read_exact(&mut buf[..len]).await?;
            let _ = handshake.read_message(&buf[..len], &mut msg_out)?;
        }
    }

    Ok(handshake.into_transport_mode()?)
}

pub fn establish_client_handshake() -> Result<HandshakeState, Box<dyn std::error::Error>> {
    let noise_params: NoiseParams = "Noise_XX_25519_ChaChaPoly_BLAKE2b".parse()?;

    let static_key = Builder::new(noise_params.clone()).generate_keypair()?;

    let builder = Builder::new(noise_params).local_private_key(&static_key.private);

    let handshake = builder.build_initiator()?;

    Ok(handshake)
}

pub fn build_login_payload() -> Message {
    use std::io::{Write, stdin, stdout};

    let mut user = String::new();
    let mut pass = String::new();

    println!("Username: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut user).unwrap();

    println!("Password: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut pass).unwrap();

    let login_data = json!({
        "username": user.trim(),
        "password": pass.trim(),
    });

    Message {
        r#type: "login_request".to_string(),
        success: false,
        message: login_data.to_string(),
    }
}
