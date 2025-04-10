use cilent_session::ClientSession;
use std::io::{Result, Write};
use std::net::{IpAddr, Ipv4Addr};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod cilent_session;

const CLIENT_ADDR: (IpAddr, u16) = (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut id = String::new();
    let mut username = String::new();

    print!("Enter a username: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut username)?;

    print!("Enter an id (u16): ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut id)?;

    let client = ClientSession::new(
        u16::from_str_radix(&id.trim(), 10).unwrap(),
        username,
        CLIENT_ADDR,
    )
    .await?;

    client.start().await;

    Ok(())
}
