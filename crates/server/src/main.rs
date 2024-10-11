use std::io::Result;
use std::net::{IpAddr, Ipv4Addr};

use async_std::task;
use futures::executor::block_on;
use server_session::ServerSession;

pub mod server_session;

#[async_std::main]
async fn main() -> Result<()> {
    println!("Server is initializing. . .");

    let mut session = ServerSession::new((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000)).await?;
    println!("Ready to accept connections!");

    task::block_on(session.process_connections())
}
