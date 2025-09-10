use std::{io::Result, net::IpAddr, sync::Arc};

use shared::{
    message::Message, message_data::MessageData, node::Node, request::Request, response::Response,
};
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
            Ok(listener) => {
                trace!("Listening on {}:{}", addr.0, addr.1);
                Ok(Self {
                    user_map: UserMap::new(),
                    listener,
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
            tokio::spawn(parse_stream(Arc::clone(&self.user_map), conn));
        }
    }
}

enum MessageDecision {
    RemoveUser(Node),
    SendUpstream(Message),
}

async fn handle_message_decision(users_handle: Arc<UserMap>, decision: MessageDecision) {
    match decision {
        MessageDecision::RemoveUser(node) => users_handle.remove(&node).map_or_else(
            || trace!("User {} was not present in map.", node),
            |_| trace!("Removed user {} successfully.", node),
        ),
        MessageDecision::SendUpstream(message) => {
            if let Some(upstream) = users_handle.get(&message.destination()) {
                upstream.send(message).await.map_or_else(
                    |e| error!("Could not send ping downstream: {e}"),
                    |_| trace!("Ping sent downstream successfully."),
                )
            }
        }
    }
}

async fn handle_message(msg: Message) -> MessageDecision {
    match msg.data() {
        MessageData::Text(_) => todo!(),
        MessageData::File(_) => todo!(),
        MessageData::User(_) => todo!(),
        MessageData::Request(Request::Connect) => todo!(),
        MessageData::Request(Request::Disconnect) => MessageDecision::RemoveUser(msg.source()),
        MessageData::Request(Request::Ping) => {
            let time_received = msg.timestamp();

            let payload = Message::builder()
                .source(Node::Server)
                .destination(msg.source())
                .data(MessageData::Response(Response::Ping(time_received)))
                .timestamp()
                .build();

            MessageDecision::SendUpstream(payload)
        }
        MessageData::Request(Request::Echo(echo_msg)) => {
            let payload = Message::builder()
                .source(Node::Server)
                .destination(msg.source().to_owned())
                .data(MessageData::Response(Response::Echo(echo_msg.to_owned())))
                .timestamp()
                .build();

            trace!("Attempting echo to user {}", msg.source());

            MessageDecision::SendUpstream(payload)
        }

        MessageData::Response(Response::Ping(_)) => todo!(),
        MessageData::Response(Response::ConnectSuccess) => todo!(),
        MessageData::Response(Response::ConnectFail) => todo!(),
        MessageData::Response(Response::UserNotFound(_)) => todo!(),
        MessageData::Response(Response::UserID(_)) => todo!(),
        MessageData::Response(Response::Echo(_)) => todo!(),
    }
}

async fn spawn_server_thread(users_handle: Arc<UserMap>) {
    let (server_upstream, mut server_downstream) = mpsc::channel::<Message>(32);

    users_handle.insert(Node::Server, server_upstream);

    // spawn the server task to display received messages
    while let Some(msg) = server_downstream.recv().await {
        let handle_clone = Arc::clone(&users_handle);

        tokio::spawn(async move {
            let decision = handle_message(msg).await;
            handle_message_decision(handle_clone, decision).await;
        });
    }
}

async fn spawn_writer(mut downstream: Receiver<Message>, mut tx: OwnedWriteHalf) {
    while let Some(msg) = downstream.recv().await {
        let id = msg.destination();

        trace!("Attempting to write {} to {}", msg, id);

        match serde_json::to_vec(&msg) {
            Ok(byte_msg) => {
                tx.write_all(&byte_msg).await.map_or_else(
                    |e| error!("Error writing to buffer: {e}"),
                    |()| trace!("Wrote to buffer successfully."),
                );

                tx.flush().await.map_or_else(
                    |e| error!("Error flushing writer: {e}"),
                    |()| trace!("Flushed buffer successfully."),
                );
            }
            Err(e) => error!("Error serializing message: {e}"),
        }
    }
}

async fn spawn_reader(
    users_handle: Arc<UserMap>,
    mut buf_reader: BufReader<OwnedReadHalf>,
    user: Node,
) -> Result<std::io::Error> {
    let mut received: Vec<u8>;
    loop {
        match buf_reader.fill_buf().await.map(|r| r.to_vec()) {
            Ok(v) => {
                buf_reader.consume(v.len());
                received = v;
            }
            Err(e) => {
                trace!("Dropping user {}.", user);
                users_handle.remove(&user);
                trace!("User {} dropped.", user);
                return Err(e);
            }
        }

        if received.is_empty() {
            continue;
        }

        trace!("Received: {:?}", received);
        match serde_json::from_slice::<Message>(&received) {
            Ok(msg) => {
                trace!("Received {}", msg);

                if msg.destination() != Node::NoNode {
                    if let Some(upstream) = users_handle.get(&msg.destination()) {
                        trace!("Attempting to send message downstream to user {}.", user);

                        upstream.send(msg).await.map_or_else(
                            |e| error!("Failed to write to upstream: {e}"),
                            |()| trace!("Message sent downstream successfully."),
                        );
                    } else if let Some(upstream) = users_handle.get(&msg.source()) {
                        trace!("Attempting to send \"User not found\" downstream.");
                        upstream
                            .send(
                                Message::builder()
                                    .source(Node::Server)
                                    .destination(msg.source())
                                    .data(MessageData::Response(Response::UserNotFound(
                                        msg.destination(),
                                    )))
                                    .timestamp()
                                    .build(),
                            )
                            .await
                            .map_or_else(
                                |e| error!("Error sending message downstream: {e}"),
                                |()| trace!("Message sent downstream successfully."),
                            );
                    }
                } else {
                    trace!("Message has no node specified.");
                }
            }
            Err(e) => error!("Error deserializing message: {e}"),
        }
    }
}

async fn parse_stream(users_handle: Arc<UserMap>, conn: TcpStream) {
    trace!("Processing connection from {:?}", conn);
    let (rx, mut tx) = conn.into_split();

    let mut buf_reader = BufReader::new(rx);

    let auth = match buf_reader.fill_buf().await.map(|r| r.to_vec()) {
        Ok(v) => {
            buf_reader.consume(v.len());
            v
        }
        Err(e) => {
            error!("Error filling buffer: {e}");
            return;
        }
    };

    if auth.is_empty() {
        return;
    }

    trace!("Received {:?}", auth);

    match serde_json::from_slice::<Message>(&auth) {
        Ok(incoming_msg) => {
            trace!("Received connection request: {incoming_msg}\n");

            if let MessageData::Request(Request::Connect) = incoming_msg.data() {
                let user = incoming_msg.source();
                let (w_upstream, w_downstream) = mpsc::channel::<Message>(8);

                if !users_handle.contains_key(&user) {
                    users_handle.insert(user.to_owned(), w_upstream);
                    trace!("Inserted {user} to hashmap\n");

                    let acceptance_payload = Message::builder()
                        .source(Node::Server)
                        .destination(user.to_owned())
                        .data(MessageData::Response(Response::ConnectSuccess))
                        .timestamp()
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
                        Some(u) => {
                            u.send(acceptance_payload).await.map_or_else(
                                |e| error!("Could not send message downstream: {e}"),
                                |()| trace!("Sent message downstream successfully."),
                            );
                        }
                        None => error!("Cound not find user in map after insertion."),
                    }
                } else {
                    trace!("User {user} already exists!\n");
                    let rejection_payload = Message::builder()
                        .source(Node::Server)
                        .destination(user.to_owned())
                        .data(MessageData::Response(Response::ConnectFail))
                        .timestamp()
                        .build();

                    match serde_json::to_vec(&rejection_payload) {
                        Ok(byte_msg) => {
                            tx.write_all(&byte_msg)
                                .await
                                .map_err(|e| error!("Error writing to stream: {e}"))
                                .ok();

                            tx.flush()
                                .await
                                .map_err(|e| error!("Error flushing stream: {e}"))
                                .ok();
                        }
                        Err(e) => error!("Error serializing message: {e}"),
                    }
                }
            }
        }
        Err(e) => error!("Error deserializing message: {e}"),
    }
}
