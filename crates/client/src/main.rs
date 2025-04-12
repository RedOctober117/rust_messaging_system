use client_session::ClientSession;
use shared::{message::Node, user::User};
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
    loop {
        print!("Server IP: ");
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut buffer)?;

        if let Ok(ip) = buffer.trim().parse::<Ipv4Addr>() {
            return Ok(ip);
        }
    }
}

fn gather_user() -> Result<Node> {
    let mut buffer = String::with_capacity(32);

    let id: u16;

    loop {
        print!("Enter an id (0-65_000): ");
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut buffer)?;

        if let Ok(parsed_id) = buffer.trim().parse::<u16>() {
            id = parsed_id;
            buffer.clear();
            break;
        } else {
            print!("Invalid id type. ");
            std::io::stdout().flush()?;
            buffer.clear();
        }
    }

    print!("Username (c<32): ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut buffer)?;

    let username = buffer.trim().to_string();
    buffer.clear();

    Ok(Node::User(User::new(id, username)))
}

// enum ClientError {
//     ParseError(String),
//     GeneralError(String),
// }
