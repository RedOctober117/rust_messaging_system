use client_session::ClientSession;
use shared::{source_or_destination::SourceOrDestination, user::User};
use std::io::Result;
use std::io::Write;
use std::net::Ipv4Addr;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod client_session;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let ip: Ipv4Addr = gather_ip()?;

    let user = gather_user()?;

    let session = ClientSession::new(user, (ip, 5000_u16)).await?;
    session.start().await
}

fn gather_ip() -> Result<Ipv4Addr> {
    let mut buffer = String::with_capacity(12);

    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    loop {
        print!("Server IP: ");
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;

        if let Ok(ip) = buffer.trim().parse::<Ipv4Addr>() {
            return Ok(ip);
        }
    }
}

fn gather_user() -> Result<SourceOrDestination> {
    let mut buffer = String::with_capacity(32);

    let id: u16;
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    loop {
        print!("ID (0-65_000): ");
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;

        if let Ok(parsed_id) = buffer.trim().parse::<u16>() {
            id = parsed_id;
            buffer.clear();
            break;
        } else {
            print!("Invalid id type. ");
            stdout.flush()?;
            buffer.clear();
        }
    }

    print!("Username (c<32): ");
    stdout.flush()?;
    stdin.read_line(&mut buffer)?;

    let username = buffer.trim().to_string();
    buffer.clear();

    Ok(SourceOrDestination::User(User::new(id, username)))
}

// enum ClientError {
//     ParseError(String),
//     GeneralError(String),
// }
