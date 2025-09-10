use std::net::Ipv4Addr;
use std::{io::Result, time::Duration};

use shared::message::Message;
use shared::message_builder::MessageBuilder;
use shared::message_data::MessageData;
use shared::node::Node;
use shared::response::Response;
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

// reader: u_upstream, w_upstream
// writer: w_downstream
//   user: u_downstream, w_upstream

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

        let (r_upstream, r_downstream) = mpsc::channel::<Message>(8);
        let (w_upstream, w_downstream) = mpsc::channel::<Message>(8);

        tokio::spawn(spawn_writer(w_downstream, writer));
        trace!("Spawned ingress writer");

        tokio::spawn(spawn_reader(buf_reader, r_upstream, w_upstream.clone()));
        trace!("Spawned ingress reader");

        spawn_core(template, r_downstream, w_upstream).await;

        Ok(())
    }
}

pub async fn spawn_writer(mut w_downstream: Receiver<Message>, mut writer: OwnedWriteHalf) {
    while let Some(msg) = w_downstream.recv().await {
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

// refactor to send msgs via broadcast, not to stdout
pub async fn spawn_reader(
    mut buf_reader: BufReader<OwnedReadHalf>,
    r_upstream: Sender<Message>,
    _w_upstream: Sender<Message>,
) {
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

        if received.is_empty() {
            continue;
        }

        trace!("RECEIVED RAW: {:?}", received);
        match serde_json::from_slice::<Message>(&received) {
            Ok(msg) => {
                trace!("Received \"{}\" from {}.", msg, msg.source());

                match msg.data() {
                    MessageData::File(_) => todo!(),
                    MessageData::Response(_) => {
                        trace!("Connection established successfully with server.");
                        r_upstream.send(msg).await.map_or_else(
                            |e| error!("Error forwarding Response to core: {e}"),
                            |()| trace!("Forwarded Response to core."),
                        );
                    }
                    // MessageBody::Response(Response::ConnectFail) => {
                    //     warn!("Failed to establish connection with server.");
                    // }
                    // MessageBody::Response(Response::Echo(t)) => {
                    //     info!("Server echoed {t:?}");
                    // }
                    // MessageBody::Response(Response::Ping(t)) => {
                    //     let time_to_server = msg.timestamp() - t;
                    //     let time_from_server = MessageBuilder::now() - msg.timestamp();
                    //     info!(
                    //         "Ping responded. Time to server: {}, Time from server: {}",
                    //         time_to_server, time_from_server
                    //     );
                    // }
                    // MessageBody::Response(Response::UserNotFound(u)) => {
                    //     trace!("Server returned user {} not found.", u);
                    //     println!("SERVER: User {} not found!", u);
                    // }
                    // MessageBody::Response(Response::UserID(_)) => todo!(),
                    MessageData::Text(_) => match (msg.source(), msg.data()) {
                        (Node::User(_), MessageData::Text(_)) => {
                            r_upstream.send(msg).await.map_or_else(
                                |e| error!("Error forwarding Text to core: {e}"),
                                |()| trace!("Forwarded Text to core."),
                            );
                        }
                        _ => todo!(),
                    },
                    MessageData::User(_) => todo!(),
                    MessageData::Request(_) => todo!(),
                }
            }

            Err(e) => {
                error!("Error deseralizing message: {e}")
            }
        }
    }
}

pub async fn spawn_core(
    msg_template: MessageBuilder,
    mut r_downstream: Receiver<Message>,
    w_upstream: Sender<Message>,
) {
    let auth_req = msg_template
        .clone()
        .destination(Node::Server)
        .data(MessageData::Request(shared::request::Request::Connect))
        .timestamp()
        .build();

    trace!("Sending {auth_req} downstream.");

    if let Ok(()) = w_upstream
        .send(auth_req)
        .await
        .map_err(|e| error!("Error sending auth request downstream: {e}"))
    {
        while let Some(msg) = r_downstream.recv().await {
            match msg.data() {
                MessageData::Response(Response::ConnectSuccess) => {
                    trace!("Successfully authenticated with server!");
                    break;
                }
                MessageData::Response(Response::ConnectFail) => {
                    warn!("Failed to authenticate with server!");
                    return;
                }
                _ => continue,
            }
        }
    }

    let mut received_message = String::new();
    let mut stdout = tokio::io::stdout();
    let stdin = std::io::stdin();

    tokio::spawn(async move {
        loop {
            if let Some(msg) = r_downstream.recv().await {
                trace!("Core downstream received {}", msg);
                match (msg.source(), msg.data()) {
                    (Node::User(user), MessageData::Text(t)) => {
                        if let Ok(()) = stdout
                            .write_all(format!("{}: {}", user.format(), t).as_bytes())
                            .await
                        {
                            stdout.flush().await.map_or_else(
                                |e| error!("Error flushing buffer: {e}"),
                                |()| trace!("Flushed buffer"),
                            );
                        }
                    }
                    _ => todo!(),
                }
            }
        }
    });

    let mut stdout = tokio::io::stdout();
    loop {
        let dest_node = match gather_user() {
            Ok(u) => u,
            Err(e) => {
                error!("Error in gather_user: {e}");
                return;
            }
        };

        if let Ok(()) = stdout.write_all(b"Message: ").await {
            stdout.flush().await.map_or_else(
                |e| error!("Error flushing buffer: {e}"),
                |()| trace!("Flushed buffer"),
            );
        }
        if let Err(e) = stdin.read_line(&mut received_message) {
            error!("Error parsing stdin: {e}");
            println!("Error processing input, please try again.");
            continue;
        }

        let payload = MessageData::Text(String::from(received_message.trim()));

        match w_upstream
            .send(
                msg_template
                    .clone()
                    .data(payload)
                    .destination(dest_node)
                    .timestamp()
                    .build(),
            )
            .await
        {
            Ok(_) => trace!("Sent message to writer."),
            Err(e) => error!("Error sending message downstream: {e}"),
        }

        received_message.clear();
    }
    //     }
    //     Err(e) => error!("Error establishing connection: {e}"),
    // }
}
