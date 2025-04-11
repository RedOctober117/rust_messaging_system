use std::{io::Result, net::IpAddr, sync::Arc};

use shared::message::{Message, MessageBody, Node, Request, Response};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    sync::mpsc::{self, Receiver},
};

use crate::user_map::UserMap;

pub struct ServerSession {
    user_map: Arc<UserMap>,
    listener: TcpListener,
}

impl ServerSession {
    pub async fn new(addr: (IpAddr, u16)) -> Result<Self> {
        match TcpListener::bind(addr).await {
            Ok(l) => {
                info!("Listening on {}:{}", addr.0, addr.1);
                Ok(Self {
                    user_map: UserMap::new(),
                    listener: l,
                })
            }
            Err(e) => {
                error!("Could not open server at {addr:?}, {e}");
                Err(e)
            }
        }
    }

    pub async fn start(self) {
        tokio::spawn(spawn_server_thread(Arc::clone(&self.user_map)));

        while let Ok((conn, _)) = self.listener.accept().await {
            tokio::spawn(handle_connection(Arc::clone(&self.user_map), conn));
        }
    }
}

async fn handle_message(users_handle: Arc<UserMap>, msg: Message) {
    match msg.body() {
        MessageBody::Request(req) => match req {
            Request::Echo(t) => {
                let payload = Message::builder()
                    .source(Node::Server)
                    .destination(msg.source().to_owned())
                    .body(MessageBody::Response(Response::Echo(t.to_owned())))
                    .timestamp()
                    .unwrap()
                    .build();

                warn!("Attempting echo to user {}", msg.source());

                match users_handle.get(&msg.source()) {
                    Some(upstream) => {
                        _ = upstream.send(payload).await;
                        info!("Echoed {t:?}\n");
                    }
                    None => todo!(),
                }
            }
            Request::Ping => {
                let time_received = msg.timestamp();

                let payload = Message::builder()
                    .source(Node::Server)
                    .destination(msg.source())
                    .body(MessageBody::Response(Response::Ping(time_received)))
                    .timestamp()
                    .unwrap()
                    .build();

                warn!("Attempting return ping to user {}", msg.source());

                match users_handle.get(&msg.source()) {
                    Some(upstream) => {
                        _ = upstream.send(payload).await;
                        info!("Returned ping to user {}\n", msg.source());
                    }
                    None => todo!(),
                }
            }
            Request::Connect => todo!(),
            Request::Disconnect => {
                trace!("Attempting to drop {}", msg.source());
                match users_handle.remove(&msg.source()) {
                    Some(_) => warn!("Removed user successfully.\n"),
                    None => warn!("User was not present in map."),
                }
            }
        },
        _ => trace!("Server received {}", msg),
    }
}

async fn spawn_server_thread(users_handle: Arc<UserMap>) {
    let (server_upstream, mut server_downstream) = mpsc::channel::<Message>(32);

    users_handle.insert(Node::Server, server_upstream);

    // spawn the server task to display received messages
    while let Some(msg) = server_downstream.recv().await {
        tokio::spawn(handle_message(Arc::clone(&users_handle), msg));
    }
}

async fn spawn_writer(mut downstream: Receiver<Message>, mut tx: OwnedWriteHalf) {
    while let Some(msg) = downstream.recv().await {
        let id = msg.destination();

        warn!("Attempting to write {} to {}", msg, id);

        match tx.write(&serde_json::to_vec(&msg).unwrap()).await {
            Ok(_) => info!("Successfully wrote message to {}\n", id),
            Err(e) => error!("Error in writer: {e}"),
        }

        tx.flush().await.unwrap();
    }
}

async fn spawn_reader(
    users_handle: Arc<UserMap>,
    mut buf_reader: BufReader<OwnedReadHalf>,
    user: Node,
) {
    let mut received: Vec<u8>;
    loop {
        if let Ok(r) = buf_reader.fill_buf().await {
            received = r.to_vec();
        } else {
            warn!("Dropping user {}", user);
            users_handle.remove(&user);
            return;
        }
        if let Ok(msg) = serde_json::from_slice::<Message>(&received) {
            trace!("Received {}", msg);

            match users_handle.get(&msg.destination()) {
                Some(upstream) => match msg.destination() {
                    Node::User(user) => {
                        warn!("Attempting to send message downstream to {}", user.id());
                        {
                            upstream.send(msg).await.unwrap();
                            info!("Forwarded message to user id {}\n", user.id());
                        };
                    }
                    Node::Server => {
                        warn!("Server received {}\n", msg,);
                    }
                    Node::NoNode => warn!("Message has no node specified.\n"),
                },
                None => {
                    trace!("implement a response of 'user not found'\n");
                    if let Some(upstream) = users_handle.get(&msg.source()) {
                        _ = upstream
                            .send(
                                Message::builder()
                                    .source(Node::Server)
                                    .destination(msg.source())
                                    .body(MessageBody::Response(Response::UserNotFound(
                                        msg.destination(),
                                    )))
                                    .timestamp()
                                    .unwrap()
                                    .build(),
                            )
                            .await;
                    }
                }
            };
        }
        buf_reader.consume(received.len());
    }
}

async fn handle_connection(users_handle: Arc<UserMap>, conn: TcpStream) {
    trace!("Processing connection from {:?}", conn);
    let (rx, mut tx) = conn.into_split();

    let mut buf_reader = BufReader::new(rx);
    let auth = buf_reader.fill_buf().await.unwrap().to_vec();
    buf_reader.consume(auth.len());

    trace!("Received {:?}", auth);

    let incoming_msg = serde_json::from_slice::<Message>(&auth).unwrap();
    trace!("Received connection request: {incoming_msg}\n");

    if let MessageBody::Request(Request::Connect) = incoming_msg.body() {
        let user = incoming_msg.source();
        let (w_upstream, w_downstream) = mpsc::channel::<Message>(8);

        if !users_handle.contains_key(&user) {
            users_handle.insert(user.to_owned(), w_upstream);
            trace!("Inserted {user} to hashmap\n");

            let acceptance_payload = Message::builder()
                .source(Node::Server)
                .destination(user.to_owned())
                .body(MessageBody::Response(Response::ConnectSuccess))
                .timestamp()
                .unwrap()
                .build();

            // spawn a thread that auto sends any messages it receives from unstream to the ownedreadhalf
            tokio::spawn(spawn_writer(w_downstream, tx));

            // spawn a thread to auto forward messages to the appropriate upstreams in the hashmap
            tokio::spawn(spawn_reader(
                Arc::clone(&users_handle),
                buf_reader,
                user.to_owned(),
            ));

            match users_handle.get(&user) {
                Some(u) => u.send(acceptance_payload).await.unwrap(),
                None => error!("Could not find user {}", user),
            }
        } else {
            warn!("{user} already exists!\n");
            let rejection_payload = Message::builder()
                .source(Node::Server)
                .destination(user.to_owned())
                .body(MessageBody::Response(Response::ConnectFail))
                .timestamp()
                .unwrap()
                .build();

            tx.write_all(&serde_json::to_vec(&rejection_payload).unwrap())
                .await
                .unwrap();

            tx.flush().await.unwrap();
        }
    }
}
