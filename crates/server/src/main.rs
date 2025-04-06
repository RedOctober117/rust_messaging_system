use std::io::Result;
use std::net::{IpAddr, Ipv4Addr};

use server_session::TcpOrchestrator;

pub mod server_session;

// pwsh cmd: $env:RUST_LOG="trace"
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    info!("Server is initializing. . .");

    let mut session = TcpOrchestrator::new((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000));

    info!("Ready to accept connections!");
    session.process_loop().await
}
