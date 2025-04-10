use std::io::Write;
use std::net::Ipv4Addr;
use std::{io::Result, time::Duration};

use shared::{
    message::{Message, MessageBody, MessageBuilder, Node, Response},
    user::User,
};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{self};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
    time::sleep,
};
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod cilent_session;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut server_ip = String::with_capacity(12);
    print!("Server IP: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut server_ip)?;

    let ip: Ipv4Addr = server_ip.trim().parse().unwrap();

    let mut id = String::new();
    let mut username = String::new();

    print!("Enter an id (u16): ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut id)?;

    print!("Enter a username: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut username)?;

    let stream: TcpStream;
    let user = Node::User(User::new(
        u16::from_str_radix(&id.trim(), 10).unwrap(),
        username.trim(),
    ));

    loop {
        trace!("Awaiting connection from server...");
        println!("Awaiting connection from server...");
        if let Ok(conn) = TcpStream::connect((ip, 5000 as u16)).await {
            stream = conn;
            trace!("Established connection with server!");
            println!("Established connection with server!");
            break;
        } else {
            sleep(Duration::from_secs(1)).await;
        }
    }

    let (reader, mut writer) = stream.into_split();
    trace!("Split stream");

    let mut buf_reader = BufReader::new(reader);

    let template = Message::builder().source(user.clone());

    let (ingress_upstream, mut ingress_downstream) = mpsc::channel::<Message>(8);

    tokio::spawn(async move {
        let mut counter = 0;

        while let Some(msg) = ingress_downstream.recv().await {
            counter = counter + 1;
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
    });
    trace!("Spawned ingress writer");

    tokio::spawn(async move {
        let mut received: Vec<u8>;
        loop {
            received = buf_reader.fill_buf().await.unwrap().to_vec();
            buf_reader.consume(received.len());

            trace!("RECEIVED RAW: {:?}", received);
            if received.len() > 0 {
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
                                let time_from_server =
                                    MessageBuilder::now().unwrap() - m.timestamp();
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
    });
    trace!("Spawned ingress reader");

    let auth_req = template
        .clone()
        .destination(Node::Server)
        .body(MessageBody::Request(shared::message::Request::Connect))
        .timestamp()
        .unwrap()
        .build();

    trace!("Message to be sent downstream: {}", auth_req);

    trace!("Sending connection request...");

    match ingress_upstream.send(auth_req).await {
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
                    u16::from_str_radix(&received_id.trim(), 10).unwrap(),
                    received_username.trim(),
                ));

                print!("Message: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut received_message).unwrap();
                println!();
                let payload = MessageBody::Text(String::from(received_message.trim()));

                let template_clone = template.clone();

                match ingress_upstream
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

                received_id.clear();
                received_username.clear();
                received_message.clear();
            }
        }
        Err(e) => error!("Error establishing connection: {e}"),
    }

    Ok(())
}
