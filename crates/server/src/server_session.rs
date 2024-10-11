use async_std::io::ReadExt;
use async_std::net::{TcpListener, TcpStream};
use async_std::stream::StreamExt;
use async_std::task;
use shared::message::Node;
use shared::message::{Message, MessageData};
use shared::user::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};
use std::net::IpAddr;

pub struct ServerSession {
    address: (IpAddr, u16),
    listener: TcpListener,
    users: HashMap<u16, User>,
    router: Router,
}

impl ServerSession {
    pub async fn new(address: (IpAddr, u16)) -> Result<Self> {
        let listener = TcpListener::bind(address).await?;
        Ok(Self {
            address,
            listener,
            users: HashMap::new(),
            router: Router::new(),
        })
    }

    // pub fn incoming(&self) -> Incoming {
    //     self.listener.incoming()
    // }

    pub fn get_address(&self) -> (IpAddr, u16) {
        self.address
    }

    pub fn get_users(&self) -> &HashMap<u16, User> {
        &self.users
    }

    pub async fn process_connections(&mut self) -> Result<()> {
        println!("waiting. . .");
        let mut incoming = self.listener.incoming();

        while let Some(stream) = incoming.next().await {
            task::block_on(async {
                let mut stream = stream.unwrap();
                let mut stream_buffer = String::new();
                stream.read_to_string(&mut stream_buffer).await.unwrap();
                println!("server received {}", stream_buffer);

                match serde_json::from_str::<Message>(
                    &stream_buffer.trim_end_matches('0').to_string(),
                ) {
                    Ok(v) => match v.get_data() {
                        MessageData::User(user) => {
                            _ = self.users.insert(user.id(), user.to_owned());
                            self.router.route_user(user.id(), stream);
                            println!("inserted user {}", user.id())
                        }
                        _ => {
                            println!("forwarding");
                            self.router.send_message(v).await.unwrap()
                        }
                    },
                    Err(e) => println!("could not process incoming stream, {}", e),
                };
            })
        }
        Ok(())
    }
}

struct Router {
    streams: HashMap<u16, TcpStream>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }
    pub fn route_user(&mut self, id: u16, stream: TcpStream) {
        _ = self.streams.insert(id, stream)
    }

    fn get_stream(&mut self, id: u16) -> Option<&mut TcpStream> {
        self.streams.get_mut(&id)
    }

    pub async fn send_message(&mut self, mut message: Message) -> Result<()> {
        if let Node::UserID(i) = &message.get_destination() {
            match self.get_stream(i.to_owned()) {
                Some(i) => message.send(&i).await?,
                None => todo!(),
            }
        }

        Ok(())
    }
}
