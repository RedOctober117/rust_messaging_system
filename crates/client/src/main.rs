use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

use async_std::task;
use async_std::task::block_on;
use client_session::ClientSession;
use futures::join;
use futures::TryFutureExt;
use shared::message::{Message, MessageData, Node};
use shared::user::User;

pub mod client_session;

#[async_std::main]
async fn main() -> Result<()> {
    let server_addr = (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
    let future_1 = task::spawn(async move {
        println!("starting client 0 task");

        let mut session = ClientSession::new(
            User::new(0, "text".into()),
            (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5001),
            server_addr.clone(),
        )
        .await
        .unwrap();

        // session.process_loop().await.unwrap();

        println!("sending user 0");
        session.send_user().await.unwrap();

        println!("sending message to user 1");
        session
            .send_message(
                MessageData::Text("Hello from user 0".into()),
                Node::UserID(1),
            )
            .await
            .unwrap();

        // session.process_loop().await.unwrap();
    });

    let future_2 = task::spawn(async move {
        println!("starting client 1 task");

        let mut session = ClientSession::new(
            User::new(1, "image".into()),
            (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5002),
            server_addr.clone(),
        )
        .await
        .unwrap();

        println!("sending user 1");
        session.send_user().await.unwrap();

        println!("user 1 waiting for connections");
        session.process_loop().await.unwrap();
    });

    block_on(future_1);
    block_on(future_2);

    Ok(())
}

pub fn establish_connection(address: (IpAddr, u16)) -> Result<TcpStream> {
    TcpStream::connect(address)
}
