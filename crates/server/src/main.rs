use server_session::ServerSession;
use std::io::Result;
use std::net::{IpAddr, Ipv4Addr};

// pwsh cmd: $env:RUST_LOG="trace"
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod server_session;
pub mod user_map;

const SERVER_ADDR: (IpAddr, u16) = (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 5000);

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let server = ServerSession::new(SERVER_ADDR).await?;
    server.start().await;

    Ok(())
}
