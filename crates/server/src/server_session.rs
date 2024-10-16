use async_std::io::{BufReadExt, BufReader, ReadExt, Result};
use async_std::net::{TcpListener, TcpStream};
use async_std::stream::StreamExt;
use async_std::sync::RwLock;
use async_std::task;

use shared::message::{self, Message, Node};
use shared::user::*;
use std::collections::HashMap;
use std::io::Read;
use std::net::IpAddr;
use std::ops::DerefMut;
use std::sync::Arc;

pub struct ServerSession {
    address: (IpAddr, u16),
    listener: TcpListener,
    users: RwLock<HashMap<u16, User>>,
    router: RwLock<Router>,
}

impl ServerSession {
    pub async fn new(address: (IpAddr, u16)) -> Result<Self> {
        let listener = TcpListener::bind(address).await?;
        Ok(Self {
            address,
            listener,
            users: RwLock::new(HashMap::new()),
            router: RwLock::new(Router::new()),
        })
    }

    // pub fn incoming(&self) -> Incoming {
    //     self.listener.incoming()
    // }

    pub fn get_address(&self) -> (IpAddr, u16) {
        self.address
    }

    // pub fn get_users(&self) -> &HashMap<u16, User> {
    //     self.users.read()
    // }
}

pub async fn process_connections(session: Arc<ServerSession>) -> Result<()> {
    println!("waiting. . .");
    let session_1 = Arc::clone(&session);
    let mut incoming = session_1.listener.incoming();

    while let Some(stream) = incoming.next().await {
        let session_clone = Arc::clone(&session);
        println!();
        task::spawn(async move {
            let mut stream = async_std::io::BufReader::new(stream.unwrap());
            let mut stream_buffer: Vec<u8> = vec![];

            stream.read_to_end(&mut stream_buffer).await.unwrap();
            let incoming_message = String::from_utf8(stream_buffer.into()).unwrap();

            let incoming_mess_len_raw = incoming_message.split(':').nth(0).unwrap();
            let incoming_message_len: u16 = incoming_mess_len_raw.parse().unwrap();

            let offset = incoming_mess_len_raw.len() + 1;
            // add 3 to offset for the xx: part of the stream_buffer
            let incoming_message_data = String::from_utf8(
                incoming_message[offset..(incoming_message_len + offset as u16).into()].into(),
            )
            .unwrap();

            let deserialized_data = serde_json::from_str::<Message>(&incoming_message_data);

            // println!(
            //     "raw data: {:?}\n data component: {}",
            //     incoming_message, incoming_message_data
            // );

            match deserialized_data {
                Ok(message) => match message.get_destination() {
                    Node::UserID(id) => {
                        let session = Arc::clone(&session_clone);
                        let mut writer = session.router.write_blocking();
                        writer.deref_mut().send_message(message).await.unwrap();
                    }
                    Node::Server => {
                        println!("Server received: {:?}", message);
                        match message.get_data() {
                            message::MessageBody::Text(s) => println!(""),
                            message::MessageBody::File(vec) => todo!(),
                            message::MessageBody::User(user) => {
                                let session = Arc::clone(&session_clone);
                                let mut writer = session.router.write_blocking();
                                writer.deref_mut().route_user(user.id(), user.location());
                            }
                            message::MessageBody::Failure(_) => todo!(),
                        }
                    }
                },
                // ::Body(v) => match v.get_destination() {
                //     Node::UserID(id) => println!("destination: {}", id),
                //     Node::Server => println!("destination: server"),
                //     // Node::Server => match v.get_data() {
                //     //     MessageData::User(user) => {
                //     //         _ = session
                //     //             .users
                //     //             .write()
                //     //             .await
                //     //             .insert(user.id(), user.to_owned());
                //     //         session.router.write().await.route_user(user.id(), stream);
                //     //         println!("inserted user {}", user.id())
                //     //     }
                Err(e) => println!(
                    "Could not process data '{:?}': {}",
                    incoming_message_data, e
                ),
            };
        });
    }
    Ok(())
}

struct Router {
    addresses: HashMap<u16, (IpAddr, u16)>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            addresses: HashMap::new(),
        }
    }
    pub fn route_user(&mut self, id: u16, loc: (IpAddr, u16)) {
        _ = self.addresses.insert(id, loc)
    }

    fn get_location(&mut self, id: &u16) -> Option<(IpAddr, u16)> {
        self.addresses.get(id).copied()
    }

    pub async fn send_message(&mut self, message: Message) -> shared::message::MessageResult<()> {
        if let Node::UserID(i) = &message.get_destination() {
            match self.get_location(i) {
                Some(i) => {
                    let conn = TcpStream::connect(i).await?;
                    message::send_message(&conn, message).await?;
                }
                None => println!("wanted to send to user {}", i),
            }
        }

        Ok(())
    }
}
