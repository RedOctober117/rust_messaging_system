use std::io::Write;
use std::net::IpAddr;
use std::sync::Arc;
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
    server_addr: (IpAddr, u16), // listener: TcpStream,
}

impl ClientSession {
    pub async fn new<S: Into<String>>(id: u16, username: S, addr: (IpAddr, u16)) -> Self {
        Self {
            user: Node::User(User::new(id, username)),
            server_addr: addr,
        }
        // loop {
        //     warn!("Awaiting connection from server...");
        //     if let Ok(s) = TcpStream::connect(addr).await {
        //         info!("Connection successful.");
        //         return Ok(Self {
        //             user: Node::User(User::new(id, username)),
        //             listener: s,
        //         });
        //     }

        // }
    }

    pub async fn start(self) -> Result<()> {
        let stream: TcpStream;

        loop {
            trace!("Awaiting connection from server...");
            if let Ok(conn) = TcpStream::connect(self.server_addr).await {
                stream = conn;
                trace!("Established connection with server!");
                break;
            } else {
                sleep(Duration::from_secs(1)).await;
            }
        }

        let (reader, writer) = stream.into_split();
        trace!("Split stream");

        let buf_reader = BufReader::new(reader);

        let template = Message::builder().source(self.user.clone());

        let (ingress_upstream, ingress_downstream) = mpsc::channel::<Message>(8);
        // self.upstream = Some(ingress_upstream);

        tokio::spawn(spawn_ingress_writer(ingress_downstream, writer));
        trace!("Spawned ingress writer");

        tokio::spawn(spawn_ingress_reader(buf_reader));
        trace!("Spawned ingress reader");

        // loop {}
        // tokio::spawn(spawn_client_interface(template, ingress_upstream));
        // trace!("Spawned client interface");

        // match ingress_upstream.send(auth_req.clone()).await {
        //     Ok(_) => {
        //         trace!("Sent connection req");

        spawn_client_interface(template.clone(), ingress_upstream);

        Ok(())
        //         trace!("Spawned client interface");
        //     }
        //     Err(e) => error!("error at line 73 {e}"),
        // };
    }
}

pub fn spawn_client_interface(message_template: MessageBuilder, ingress_upstream: Sender<Message>) {
    let cloned_upstream = Arc::new(ingress_upstream);
    // let cloned_upstream_2 = Arc::clone(&cloned_upstream);

    // tokio::spawn(async move {
    //     loop {
    //         println!("upstream status: {}", cloned_upstream_2.is_closed());
    //     }
    // });

    let auth_req = message_template
        .clone()
        .destination(Node::Server)
        .body(MessageBody::Request(shared::message::Request::Connect))
        .timestamp()
        .unwrap()
        .build();

    trace!("Message to be sent downstream: {}", auth_req);

    trace!("Sending connection request...");

    tokio::spawn(async move {
        match cloned_upstream.clone().send(auth_req).await {
            Ok(_) => {
                trace!("Connection Established.");

                let mut received_id = String::new();
                let mut received_username = String::new();
                let mut received_message = String::new();

                loop {
                    // trace!("upstream: {:?}", cloned_upstream);
                    // let cloned_upstream = Arc::clone(&cloned_upstream);

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

                    let sender_clone = Arc::clone(&cloned_upstream);
                    let template_clone = message_template.clone();
                    tokio::spawn(async move {
                        match sender_clone
                            .send(
                                template_clone
                                    .body(payload)
                                    .destination(dest_node)
                                    .timestamp()
                                    .unwrap()
                                    .build(),
                            )
                            .await
                        {
                            Ok(_) => info!("Sent message"),
                            Err(e) => error!("Error sending message downstream: {e}"),
                        }
                    });

                    received_id.clear();
                    received_username.clear();
                    received_message.clear();
                }
            }
            Err(e) => error!("Error establishing connection: {e}"),
        }
    });
}

pub async fn spawn_ingress_reader(mut buf_reader: BufReader<OwnedReadHalf>) {
    let mut received: Vec<u8>;
    loop {
        // trace!("buf_reader: {:?}", buf_reader);

        received = buf_reader.fill_buf().await.unwrap().to_vec();
        buf_reader.consume(received.len());

        if let Ok(m) = serde_json::from_slice::<Message>(&received) {
            info!("RECEIVED: {:?}", m);
            // match m.body() {
            //     MessageBody::File(_) => todo!(),
            //     MessageBody::Response(response) => match response {
            //         Response::UserID(_) => todo!(),
            //         Response::ConnectSuccess => {
            //             info!("Connection established successfully with server.");
            //         }
            //         Response::ConnectFail => {
            //             warn!("Failed to establish connection with server.");
            //         }
            //         Response::Echo(t) => info!("Server echoed {t:?}"),
            //         Response::Ping(t) => {
            //             let time_to_server = m.timestamp() - t;
            //             let time_from_server = MessageBuilder::now().unwrap() - m.timestamp();
            //             info!(
            //                 "Ping responded. Time to server: {}, Time from server: {}",
            //                 time_to_server, time_from_server
            //             );
            //         }
            //     },
            //     MessageBody::Text(t) => {
            //         info!("Received \"{}\" from {}.", t, m.source());
            //     }
            //     MessageBody::User(_) => todo!(),
            //     MessageBody::Request(_) => todo!(),
            // }
        } else {
            warn!("Could not parse buffer");
        }
    }
}

pub async fn spawn_ingress_writer(mut downstream: Receiver<Message>, mut writer: OwnedWriteHalf) {
    let mut counter = 0;

    while let Some(msg) = downstream.recv().await {
        counter = counter + 1;
        trace!("receiver alive for {} cycles", counter);
        trace!("buf_writer: {:?}", writer);

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
// while let Some(msg) = downstream.recv().await {
//     trace!("buf_writer: {:?}", writer);

//     warn!("Attempting to write {}", msg);

//     match writer
//         .write_all(&serde_json::to_vec(&msg).expect("Failed to parse msg"))
//         .await
//     {
//         Ok(_) => {
//             info!("Successfully wrote message.");
//         }
//         Err(e) => error!("Error in ingress writer: {e}"),
//     }

//     writer.flush().await.unwrap();
// }
