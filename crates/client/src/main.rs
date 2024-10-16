use std::net::{IpAddr, Ipv4Addr};

use async_std::io::Result;
use async_std::net::TcpStream;
use async_std::task;
use async_std::task::block_on;
use async_std::task::sleep;
use client_session::ClientSession;
use futures::join;
use futures::TryFutureExt;
use shared::message::{MessageBody, Node};
use shared::user::User;
use std::time::Duration;

pub mod client_session;

#[async_std::main]
async fn main() -> Result<()> {
    let server_addr: (IpAddr, u16) = (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
    let future_1 = task::spawn(async move {
        println!("starting client 0 task");

        let mut session = ClientSession::new(
            User::new(
                0,
                "text".into(),
                (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5001),
            ),
            server_addr.clone(),
        )
        .await
        .unwrap();

        // session.process_loop().await.unwrap();

        println!("sending user 0");
        session.send_user().await.unwrap();

        println!("sending message to user 1");
        loop {
            sleep(Duration::from_secs(1)).await;
            println!("sending message to user 1");
            session
                .send_message(
                    MessageBody::Text("Hello from user 0".into()),
                    Node::UserID(1),
                )
                .await
                .unwrap();
        }

        // client_session::process_loop(session).await.unwrap();
    });

    let future_2 = task::spawn(async move {
        println!("starting client 1 task");

        let mut session = ClientSession::new(
            User::new(
                1,
                "image".into(),
                (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5002),
            ),
            server_addr.clone(),
        )
        .await
        .unwrap();

        // session.send_user().await.unwrap();
        println!("sending user 1");
        session.send_user().await.unwrap();

        println!("user 1 sending text");
        session
            .send_message(
                MessageBody::Text("Hello from user 1".into()),
                Node::UserID(0),
            )
            .await
            .unwrap();
        // session.send_user().await.unwrap();

        // println!("user 1 waiting for connections");
        client_session::process_loop(session).await.unwrap()
    });

    block_on(future_1);
    block_on(future_2);

    Ok(())
}
