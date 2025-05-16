// use std::io::BufRead;

// all crypto stuff can go here
use snow::{Builder, HandshakeState, TransportState};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
fn create_noise_client() -> HandshakeState {
    let builder: Builder<'_> = Builder::new("Noise_XX_25519_ChchaPoly_Blake2s".parse().unwrap());
    builder.build_initiator().unwrap()
}

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
