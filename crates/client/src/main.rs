use shared::message::{Message, MessageBody, MessageBuilder, Node};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Result};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::time::sleep;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod client_session;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let server_addr: (IpAddr, u16) = (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);

    // let client_1_handle = tokio::spawn(async move {
    let (reader, writer): (OwnedReadHalf, OwnedWriteHalf);

    let mut buf_reader: BufReader<OwnedReadHalf>;
    let mut buf_writer: BufWriter<OwnedWriteHalf>;

    loop {
        if let Ok(s) = TcpStream::connect(server_addr).await {
            (reader, writer) = s.into_split();
            buf_reader = BufReader::new(reader);
            buf_writer = BufWriter::new(writer);

            break;
        }

        warn!("No server found, retrying in 1 second.");
        sleep(Duration::from_secs(1)).await;
    }

    let reader_handle = tokio::spawn(async move {
        loop {
            let received = buf_reader.fill_buf().await.unwrap().to_vec();
            if received.len() > 0 {
                match serde_json::from_slice::<Message>(&received) {
                    Ok(m) => {
                        info!("RECEIVED: {:?}", m);
                    }
                    Err(e) => warn!("Could not parse buffer, {e}"),
                }
            }

            buf_reader.consume(received.len());
        }
    });

    let writer_handle = tokio::spawn(async move {
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

        buf_writer
            .write(&serde_json::to_vec(&greeting).unwrap())
            .await
            .unwrap();

        buf_writer.flush().await.unwrap();

        loop {
            sleep(Duration::from_secs(1)).await;
            let ping = msg_template
                .clone()
                .body(MessageBody::Text("ping!".into()))
                .timestamp()
                .unwrap()
                .build();

            buf_writer
                .write(&serde_json::to_vec(&ping).unwrap())
                .await
                .unwrap();

            buf_writer.flush().await.unwrap();
            info!("SENT: {:?}", ping);
        }
    });

    reader_handle.await.unwrap();
    writer_handle.await.unwrap();

    Ok(())
    // });

    // client_1_handle.await.unwrap()
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
