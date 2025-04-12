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
            } else {
                sleep(Duration::from_secs(1)).await;
            }
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
    let mut counter = 0;

    while let Some(msg) = downstream.recv().await {
        counter += 1;
        trace!("receiver alive for {} cycles", counter);

        warn!("Attempting to write {}", msg);

        match writer
            .write_all(&serde_json::to_vec(&msg).expect("Failed to parse msg"))
            .await
        {
            Ok(_) => {
                info!("Successfully wrote message.");
            }
            Err(e) => error!("Error in ingress writer: {e}"),
        }

        writer.flush().await.unwrap();
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
                error!("Error filling buffer: {e}");
                return;
            }
        }

        trace!("RECEIVED RAW: {:?}", received);
        if !received.is_empty() {
            if let Ok(m) = serde_json::from_slice::<Message>(&received) {
                info!("RECEIVED PARSED: {:?}", m);
                match m.body() {
                    MessageBody::File(_) => todo!(),
                    MessageBody::Response(response) => match response {
                        Response::UserID(_) => todo!(),
                        Response::ConnectSuccess => {
                            info!("Connection established successfully with server.");
                        }
                        Response::ConnectFail => {
                            warn!("Failed to establish connection with server.");
                        }
                        Response::Echo(t) => info!("Server echoed {t:?}"),
                        Response::Ping(t) => {
                            let time_to_server = m.timestamp() - t;
                            let time_from_server = MessageBuilder::now() - m.timestamp();
                            info!(
                                "Ping responded. Time to server: {}, Time from server: {}",
                                time_to_server, time_from_server
                            );
                        }
                        Response::UserNotFound(u) => {
                            trace!("Server returned user {} not found.", u);
                            println!("SERVER: User {} not found!", u);
                        }
                    },
                    MessageBody::Text(t) => {
                        info!("Received \"{}\" from {}.", t, m.source());
                        if let Node::User(u) = m.source() {
                            if let MessageBody::Text(t) = m.body() {
                                println!("{}: {}", u.format(), t);
                            }
                        }
                    }
                    MessageBody::User(_) => todo!(),
                    MessageBody::Request(_) => todo!(),
                }
            } else {
                warn!("Could not parse buffer");
                sleep(Duration::from_secs(2)).await;
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

            let mut received_id = String::new();
            let mut received_username = String::new();
            let mut received_message = String::new();

            loop {
                print!("Destination ID: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut received_id).unwrap();

                print!("Destination Username: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut received_username).unwrap();

                let dest_node = Node::User(User::new(
                    // u16::from_str_radix(&received_id.trim(), 10).unwrap(),
                    received_id.trim().parse::<u16>().unwrap(),
                    received_username.trim(),
                ));

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

                received_id.clear();
                received_username.clear();
                received_message.clear();
            }
        }
        Err(e) => error!("Error establishing connection: {e}"),
    }
}
