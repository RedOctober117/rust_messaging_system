use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use shared::message::{MessageBody, MessageBuilder, Node};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Interest, Result};
use tokio::net::TcpStream;
use tokio::time::sleep;
pub mod client_session;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let server_addr: (IpAddr, u16) = (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);

    let client_1_handle = tokio::spawn(async move {
        let mut stream = TcpStream::connect(server_addr).await.unwrap();

        let msg_template = MessageBuilder::new()
            .source(Node::UserID(0))
            .destination(Node::Server);

        let greeting = msg_template
            .clone()
            .body(MessageBody::Text("Hello from client 0".into()))
            .timestamp()
            .unwrap()
            .build();

        info!("Sending hello from client 1. . .");
        stream
            .write(greeting.as_netstring().unwrap().as_bytes())
            .await
            .unwrap();

        stream.flush().await.unwrap();

        loop {
            sleep(Duration::from_secs(1)).await;
            let stream_status = stream
                .ready(Interest::READABLE | Interest::WRITABLE)
                .await
                .unwrap();

            if stream_status.is_readable() {
                let mut buff: Vec<u8> = Vec::new();
                stream.read_buf(&mut buff).await.unwrap();

                if buff.len() > 0 {
                    info!("Received {} from server.", String::from_utf8(buff).unwrap());
                }
            }

            if stream_status.is_writable() {
                let ping = msg_template
                    .clone()
                    .body(MessageBody::Text("ping!".into()))
                    .timestamp()
                    .unwrap()
                    .build();

                stream
                    .write(ping.as_netstring().unwrap().as_bytes())
                    .await
                    .unwrap();
                stream.flush().await.unwrap();
            }
        }
    });

    client_1_handle.await.unwrap()
}
// let future_1 = tokio::spawn(async move {
//     info!("starting client 0 task");

//     let mut session = ClientSession::new(
//         User::new(
//             0,
//             "text".into(),
//             (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5001),
//         ),
//         server_addr.clone(),
//     )
//     .await
//     .unwrap();

//     // session.process_loop().await.unwrap();

//     info!("sending user 0");
//     session.send_user().await.unwrap();

//     info!("sending message to user 1");

//     let message = Message::builder();

//     sleep(Duration::from_secs(1)).await;
//     info!("sending message to user 1");
//     let greeting = message
//         .source(Node::UserID(0))
//         .destination(Node::UserID(1))
//         .data(MessageBody::Text("Hello from user 0!".into()))
//         .build();

//     session.send_message(greeting).await.unwrap();
// });

// let future_2 = tokio::spawn(async move {
//     info!("starting client 1 task");

//     let mut session = ClientSession::new(
//         User::new(
//             1,
//             "image".into(),
//             (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5002),
//         ),
//         server_addr.clone(),
//     )
//     .await
//     .unwrap();

//     // session.send_user().await.unwrap();
//     info!("sending user 1");
//     session.send_user().await.unwrap();

//     info!("user 1 sending text");
//     let msg_template = Message::builder();

//     let greeting = msg_template
//         .source(Node::UserID(1))
//         .destination(Node::UserID(0))
//         .data(MessageBody::Text("Hello from user 1!".into()))
//         .build();

//     session.send_message(greeting).await.unwrap();
//     // session
//     //     .send_message(
//     //         MessageBody::Text("Hello from user 1".into()),
//     //         Node::UserID(0),
//     //     )
//     //     .await
//     //     .unwrap();
//     // session.send_user().await.unwrap();

//     info!("user 1 waiting for connections");
//     // client_session::process_loop(session).await.unwrap()
// });

// tokio::spawn(future_1);
// tokio::spawn(future_2);
