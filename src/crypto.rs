// use std::io::BufRead;

// all crypto stuff can go here
use snow::{Builder, HandshakeState};

fn create_noise_client() -> HandshakeState {
    let builder: Builder<'_> = Builder::new("Noise_XX_25519_ChchaPoly_Blake2s".parse().unwrap());
    builder.build_initiator().unwrap()
}

pub fn start_handshake() -> (HandshakeState, Vec<u8>) {
    let mut noise = create_noise_client();
    let mut buf = [0u8; 1024];
    let len = noise.write_message(&[], &mut buf).unwrap();
    (noise, buf[..len].to_vec())
}

pub fn complete_handshake(mut noise: HandshakeState, response: &[u8]) -> snow::TransportState {
    let mut buf = [0u8; 1024];
    noise.read_message(response, &mut buf).unwrap();
    noise.into_transport_mode().unwrap()
}
