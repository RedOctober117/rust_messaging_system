use shared::message::{Message, MessageBody, MessageBuilder, Node};
use shared::user::User;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Result};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::time::sleep;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

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
                        match m.body() {
                            MessageBody::File(_) => todo!(),
                            MessageBody::Response(response) => match response {
                                shared::message::Response::UserID(_) => todo!(),
                                shared::message::Response::ConnectSuccess => todo!(),
                                shared::message::Response::ConnectFail => todo!(),
                                shared::message::Response::Echo(t) => info!("Server echoed {t:?}"),
                                shared::message::Response::Ping(t) => {
                                    let time_to_server = m.timestamp() - t;
                                    let time_from_server =
                                        MessageBuilder::now().unwrap() - m.timestamp();
                                    info!(
                                        "Ping responded. Time to server: {}, Time from server: {}",
                                        time_to_server, time_from_server
                                    );
                                }
                            },
                            MessageBody::Text(_) => todo!(),
                            MessageBody::User(user) => todo!(),
                            MessageBody::Request(request) => todo!(),
                        }
                    }
                    Err(e) => warn!("Could not parse buffer, {e}"),
                }
            }

            buf_reader.consume(received.len());
        }
    });

    let writer_handle = tokio::spawn(async move {
        let user = User::new(1, "test");

        let msg_template = MessageBuilder::new()
            .source(Node::User(user))
            .destination(Node::Server);

        let auth = msg_template
            .clone()
            .body(MessageBody::Request(shared::message::Request::Connect))
            .timestamp()
            .unwrap()
            .build();

        info!("Sending hello from client 1. . .");
        buf_writer
            .write_all(&serde_json::to_vec(&auth).unwrap())
            .await
            .unwrap();

        buf_writer.flush().await.unwrap();

        loop {
            sleep(Duration::from_secs(3)).await;
            let ping = msg_template
                .clone()
                .body(MessageBody::Request(shared::message::Request::Ping))
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
}
