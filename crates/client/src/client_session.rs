use std::io::Write;
use std::net::Ipv4Addr;
use std::{io::Result, time::Duration};

use shared::{
    message::{Message, MessageBody, MessageBuilder, Node, Response},
    user::User,
};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    time::sleep,
};

use crate::gather_user;

pub struct ClientSession {
    user: Node,
    stream: TcpStream,
}

impl ClientSession {
    pub async fn new(user: Node, server_addr: (Ipv4Addr, u16)) -> Result<Self> {
        let stream: TcpStream;

        loop {
            trace!("Awaiting connection from server...");
            println!("Awaiting connection from server...");
            if let Ok(conn) = TcpStream::connect(server_addr).await {
                stream = conn;
                trace!("Established connection with server!");
                println!("Established connection with server!");
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }

        Ok(Self { user, stream })
    }

    pub async fn start(self) -> Result<()> {
        let (reader, writer) = self.stream.into_split();
        trace!("Split stream");

        let buf_reader = BufReader::new(reader);

        let template = Message::builder().source(self.user.clone());

        let (ingress_upstream, ingress_downstream) = mpsc::channel::<Message>(8);

        tokio::spawn(spawn_writer(ingress_downstream, writer));
        trace!("Spawned ingress writer");

        tokio::spawn(spawn_reader(buf_reader));
        trace!("Spawned ingress reader");

        spawn_interface(template, ingress_upstream).await;

        Ok(())
    }
}

pub async fn spawn_writer(mut downstream: Receiver<Message>, mut writer: OwnedWriteHalf) {
    while let Some(msg) = downstream.recv().await {
        warn!("Attempting to write {}", msg);

        writer
            .write_all(&serde_json::to_vec(&msg).expect("Failed to parse msg"))
            .await
            .map_or_else(
                |e| error!("Error in ingress writer: {e}"),
                |()| info!("Successfully wrote message."),
            );

        writer.flush().await.map_or_else(
            |e| error!("Error flushing buffer: {e}"),
            |()| trace!("Flushed buffer successfully."),
        );
    }
}

pub async fn spawn_reader(mut buf_reader: BufReader<OwnedReadHalf>) {
    let mut received: Vec<u8>;
    loop {
        match buf_reader.fill_buf().await.map(|r| r.to_vec()) {
            Ok(v) => {
                buf_reader.consume(v.len());
                received = v;
            }
            Err(e) => {
                error!("Error in buffer: {e}");
                return;
            }
        }

        trace!("RECEIVED RAW: {:?}", received);
        match serde_json::from_slice::<Message>(&received) {
            Ok(m) => {
                info!("RECEIVED PARSED: {:?}", m);
                match m.body() {
                    MessageBody::File(_) => todo!(),
                    MessageBody::Response(Response::ConnectSuccess) => {
                        info!("Connection established successfully with server.");
                    }
                    MessageBody::Response(Response::ConnectFail) => {
                        warn!("Failed to establish connection with server.");
                    }
                    MessageBody::Response(Response::Echo(t)) => {
                        info!("Server echoed {t:?}");
                    }
                    MessageBody::Response(Response::Ping(t)) => {
                        let time_to_server = m.timestamp() - t;
                        let time_from_server = MessageBuilder::now() - m.timestamp();
                        info!(
                            "Ping responded. Time to server: {}, Time from server: {}",
                            time_to_server, time_from_server
                        );
                    }
                    MessageBody::Response(Response::UserNotFound(u)) => {
                        trace!("Server returned user {} not found.", u);
                        println!("SERVER: User {} not found!", u);
                    }
                    MessageBody::Response(Response::UserID(_)) => todo!(),
                    MessageBody::Text(t) => {
                        info!("Received \"{}\" from {}.", t, m.source());
                        match (m.source(), m.body()) {
                            (Node::User(u), MessageBody::Text(t)) => {
                                println!("{}: {}", u.format(), t)
                            }
                            _ => todo!(),
                        }
                    }
                    MessageBody::User(_) => todo!(),
                    MessageBody::Request(_) => todo!(),
                }
            }

            Err(e) => {
                error!("Error deseralizing message: {e}")
            }
        }
    }
}

pub async fn spawn_interface(msg_template: MessageBuilder, upstream: Sender<Message>) {
    let auth_req = msg_template
        .clone()
        .destination(Node::Server)
        .body(MessageBody::Request(shared::message::Request::Connect))
        .timestamp()
        .build();

    trace!("Message to be sent downstream: {}", auth_req);

    trace!("Sending connection request...");

    match upstream.send(auth_req).await {
        Ok(_) => {
            trace!("Connection Established.");

            let mut received_message = String::new();

            loop {
                let dest_node = match gather_user() {
                    Ok(u) => u,
                    Err(e) => {
                        error!("Error in gather_user: {e}");
                        return;
                    }
                };

                print!("Message: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut received_message).unwrap();

                println!();

                let payload = MessageBody::Text(String::from(received_message.trim()));

                match upstream
                    .send(
                        msg_template
                            .clone()
                            .body(payload)
                            .destination(dest_node)
                            .timestamp()
                            .build(),
                    )
                    .await
                {
                    Ok(_) => info!("Sent message"),
                    Err(e) => error!("Error sending message downstream: {e}"),
                }

                received_message.clear();
            }
        }
        Err(e) => error!("Error establishing connection: {e}"),
    }
}
