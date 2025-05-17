// TCP and UDP logic can go here

use crate::crypto::complete_handshake;
// use cpal::Stream;
// use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string};
use snow::{Builder, HandshakeState, TransportState, params::NoiseParams};
// use std::io::{Read, Write};
// use std::net::TcpStream;
use std::u8;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Deserialize, Debug, Serialize)]
pub struct Message {
    pub r#type: String,
    pub success: bool,
    pub message: String,
}

pub async fn establish_command(
    addr: &str,
) -> Result<(tokio::net::TcpStream, snow::TransportState), Box<dyn std::error::Error>> {
    let mut stream = tokio::net::TcpStream::connect(addr).await?;

    let handshake = establish_client_handshake().unwrap();
    let session = complete_handshake(&mut stream, handshake).await?;

    Ok((stream, session))
}

pub async fn send_message(
    stream: &mut TcpStream,
    session: &mut TransportState,
    msg: &Message,
) -> std::io::Result<()> {
    let json = to_string(msg).unwrap(); //coud use .map_err for cleaner handling ig
    let plaintext = json.as_bytes();
    let mut encrypted = [0u8; 1024];
    let len = session.write_message(plaintext, &mut encrypted).unwrap();

    stream.write_u16(len as u16).await?;
    stream.write_all(&encrypted[..len]).await?;

    Ok(())
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

pub async fn read_message(
    stream: &mut TcpStream,
    session: &mut TransportState,
) -> std::io::Result<Message> {
    let mut buf = [0u8; 1024];

    // Read a 2-byte length prefix
    let len = {
        let mut len_bytes = [0u8; 2];
        stream.read_exact(&mut len_bytes).await?;
        u16::from_be_bytes(len_bytes) as usize
    };

    // Read the actual encrypted message
    stream.read_exact(&mut buf[..len]).await?;

    let mut out = [0u8; 1024];
    let msg_len = session.read_message(&buf[..len], &mut out).unwrap();

    let json = &out[..msg_len];
    let parsed: Message = serde_json::from_slice(json).unwrap();

    Ok(parsed)
}

pub fn establish_client_handshake() -> Result<HandshakeState, Box<dyn std::error::Error>> {
    let noise_params: NoiseParams = "Noise_XX_25519_ChaChaPoly_BLAKE2b".parse()?;

    let static_key = Builder::new(noise_params.clone()).generate_keypair()?;

    let builder = Builder::new(noise_params).local_private_key(&static_key.private);

    let handshake = builder.build_initiator()?;

    Ok(handshake)
}
