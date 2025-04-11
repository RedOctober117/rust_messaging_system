use cilent_session::ClientSession;
use shared::{message::Node, user::User};
use std::io::Result;
use std::io::Write;
use std::net::Ipv4Addr;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod cilent_session;

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

        if let Ok(ip) = buffer.parse::<Ipv4Addr>() {
            return Ok(ip);
        }
    }
}

fn gather_user() -> Result<Node> {
    let mut id = String::new();
    let mut username = String::new();

    print!("Enter an id (u16): ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut id)?;

    print!("Enter a username: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut username)?;

    Ok(Node::User(User::new(
        id.trim().parse::<u16>().unwrap(),
        username.trim(),
    )))
}

// enum ClientError {
//     ParseError(String),
//     GeneralError(String),
// }
