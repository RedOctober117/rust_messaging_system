use std::clone;
use std::io::Write;
use std::net::IpAddr;
use std::sync::Arc;
use std::{io::Result, time::Duration};

use shared::{
    message::{Message, MessageBody, MessageBuilder, Node},
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
    listener: TcpStream,
}

impl ClientSession {
    pub async fn new<S: Into<String>>(id: u16, username: S, addr: (IpAddr, u16)) -> Result<Self> {
        loop {
            warn!("Awaiting connection from server...");
            if let Ok(s) = TcpStream::connect(addr).await {
                info!("Connection successful.");
                return Ok(Self {
                    user: Node::User(User::new(id, username)),
                    listener: s,
                });
            }

            sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn start(self) {
        let (reader, writer): (OwnedReadHalf, OwnedWriteHalf) = self.listener.into_split();
        trace!("Split stream");

        let buf_reader = BufReader::new(reader);

        let template = Message::builder().source(self.user.clone());

        let (ingress_upstream, ingress_downstream) = mpsc::channel::<Message>(8);
        // self.upstream = Some(ingress_upstream);

        tokio::spawn(spawn_ingress_writer(ingress_downstream, writer));
        trace!("Spawned ingress writer");

        tokio::spawn(spawn_ingress_reader(buf_reader));
        trace!("Spawned ingress reader");

        tokio::spawn(spawn_client_interface(template, ingress_upstream));
        trace!("Spawned client interface");

        // match ingress_upstream.send(auth_req.clone()).await {
        //     Ok(_) => {
        //         trace!("Sent connection req");

        //         tokio::spawn(spawn_client_interface(template.clone(), ingress_upstream));
        //         trace!("Spawned client interface");
        //     }
        //     Err(e) => error!("error at line 73 {e}"),
        // };
    }
}

async fn spawn_client_interface(
    message_template: MessageBuilder,
    ingress_upstream: Sender<Message>,
) {
    let cloned_upstream = Arc::new(ingress_upstream);

    let auth_req = message_template
        .clone()
        .destination(Node::Server)
        .body(MessageBody::Request(shared::message::Request::Connect))
        .timestamp()
        .unwrap()
        .build();

    trace!("Sending connection request...");
    match Arc::clone(&cloned_upstream).send(auth_req).await {
        Ok(_) => {
            trace!("Connection Established.");
            let mut received_id = String::new();
            let mut received_username = String::new();
            let mut received_message = String::new();

            loop {
                trace!("upstream: {:?}", cloned_upstream);
                let cloned_upstream = Arc::clone(&cloned_upstream);

                print!("Destination ID: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut received_id).unwrap();

                print!("Destination Username: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut received_username).unwrap();

                let dest_node = Node::User(User::new(
                    u16::from_str_radix(&received_id.trim(), 10).unwrap(),
                    &received_username,
                ));

                print!("Message: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut received_message).unwrap();
                let payload = MessageBody::Text(received_message.clone());

                match cloned_upstream
                    .send(
                        message_template
                            .clone()
                            .body(payload)
                            .destination(dest_node)
                            .timestamp()
                            .unwrap()
                            .build(),
                    )
                    .await
                {
                    Ok(_) => info!("Sent message"),
                    Err(e) => error!("Error sending message: {e}"),
                }

                received_id.clear();
                received_username.clear();
                received_message.clear();
            }
        }
        Err(e) => error!("Error establishing connection: {e}"),
    }
}

async fn spawn_ingress_reader(mut buf_reader: BufReader<OwnedReadHalf>) {
    let mut received: Vec<u8>;
    loop {
        trace!("buf_reader: {:?}", buf_reader);

        received = buf_reader.fill_buf().await.unwrap().to_vec();
        if received.len() > 0 {
            match serde_json::from_slice::<Message>(&received) {
                Ok(m) => {
                    info!("RECEIVED: {:?}", m);
                    match m.body() {
                        MessageBody::File(_) => todo!(),
                        MessageBody::Response(response) => match response {
                            shared::message::Response::UserID(_) => todo!(),
                            shared::message::Response::ConnectSuccess => {
                                info!("Connection established successfully with server.");
                            }
                            shared::message::Response::ConnectFail => {
                                warn!("Failed to establish connection with server.");
                            }
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
                        MessageBody::Text(t) => {
                            info!("Received \"{}\" from {}.", t, m.source());
                        }
                        MessageBody::User(user) => todo!(),
                        MessageBody::Request(request) => todo!(),
                    }
                }
                Err(e) => warn!("Could not parse buffer, {e}"),
            }
        }
        buf_reader.consume(received.len());
    }
}

async fn spawn_ingress_writer(mut downstream: Receiver<Message>, mut writer: OwnedWriteHalf) {
    while let Some(msg) = downstream.recv().await {
        trace!("buf_writer: {:?}", writer);

        warn!("Attempting to write {}", msg);

        match writer.write_all(&serde_json::to_vec(&msg).unwrap()).await {
            Ok(_) => {
                info!("Successfully wrote message.");
            }
            Err(e) => error!("Error in ingress writer: {e}"),
        }

        writer.flush().await.unwrap();
    }
}
