// TCP and UDP logic can go here

use crate::crypto::{complete_handshake, start_handshake};
use serde::Deserialize;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_json::{from_slice, to_vec};
use snow::TransportState;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::u8;

struct LoginLoad {
    r#type: String,
    username: String,
    password: String,
}

impl Serialize for LoginLoad {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("LoginLoad", 3)?;

        s.serialize_field("type", &self.r#type)?;
        s.serialize_field("username", &self.username)?;
        s.serialize_field("password", &self.password)?;
        s.end()
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginResponse {
    pub r#type: String,
    pub success: bool,
    pub message: String,
}

pub fn establish_command(addr: &str) -> std::io::Result<(TcpStream, TransportState)> {
    let mut stream = TcpStream::connect(addr)?;

    // prep and send initial handshake message
    let (handshake, init_msg) = start_handshake();
    stream.write_all(&init_msg)?;

    //recieve sever response
    let mut init_response = [0u8; 1024];
    let len = stream.read(&mut init_response)?;
    let session = complete_handshake(handshake, &init_response[..len]);

    Ok((stream, session))
}

pub fn send_message(
    stream: &mut TcpStream,
    session: &mut TransportState,
    plaintext: &[u8],
) -> std::io::Result<()> {
    let mut buf = [0u8; 1024];
    let len = session.write_message(plaintext, &mut buf).unwrap();
    stream.write_all(&buf[..len]).unwrap();
    Ok(())
}

pub fn build_login_payload() -> Vec<u8> {
    use std::io::{Write, stdin, stdout};

    let mut user = String::new();
    let mut pass = String::new();

    println!("Username: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut user).unwrap();

    println!("Password: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut pass).unwrap();

    let msg = LoginLoad {
        r#type: "login_request".to_string(),
        username: user.trim().to_string(),
        password: pass.trim().to_string(),
    };

    to_vec(&msg).unwrap()
}

pub fn read_message(
    stream: &mut TcpStream,
    session: &mut TransportState,
) -> std::io::Result<LoginResponse> {
    let mut buf = [0u8; 1024];
    let len = stream.read(&mut buf)?;

    let mut out = [0u8; 1024];
    let msg_len = session.read_message(&buf[..len], &mut out).unwrap();

    let json = &out[..msg_len];
    let parsed: LoginResponse = from_slice(json).unwrap();

    Ok(parsed)
}
