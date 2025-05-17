// TCP and UDP logic can go here

use crate::crypto::{complete_handshake, establish_client_handshake};
// use cpal::Stream;
// use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use snow::TransportState;
use std::io::Error;
use tokio::task::JoinHandle;
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

pub async fn spawn_main_session_task(
    mut stream: TcpStream,
    mut session: TransportState,
) -> Result<JoinHandle<()>, Error> {
    let handle = tokio::spawn(async move {
        loop {
            let msg = match read_message(&mut stream, &mut session).await {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("❌ Connection lost: {}", e);
                    break;
                }
            };

            println!("📥 {:?}", msg);
        }
    });
    Ok(handle)
}
