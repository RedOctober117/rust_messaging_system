use async_std::io::ReadExt;
use async_std::net::TcpListener;
use async_std::stream::StreamExt;
use shared::message::{MessageData, MessageType};
use shared::user::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};
use std::net::IpAddr;

pub struct ServerSession {
    address: (IpAddr, u16),
    listener: TcpListener,
    users: HashMap<u16, User>,
}

impl ServerSession {
    pub async fn new(address: (IpAddr, u16)) -> Result<Self> {
        let listener = TcpListener::bind(address).await?;
        Ok(Self {
            address,
            listener,
            users: HashMap::new(),
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
        let mut incoming = self.listener.incoming();

        while let Some(stream) = incoming.next().await {
            let mut stream_buffer = String::new();
            stream?.read_to_string(&mut stream_buffer).await?;
            match serde_json::from_str::<MessageData>(&stream_buffer) {
                Ok(v) => match v.get_data() {
                    MessageType::Text(ve) => println!("got {:?}", ve.trim_matches('0')),
                    MessageType::File(ve) => {
                        println!("got file");
                        let mut ser_file = File::create_new("serialized_image.jpg")
                            .expect("could not create file");
                        ser_file.write_all(&ve).unwrap()
                    }
                    MessageType::User(user) => _ = self.users.insert(user.id(), user.clone()),
                },
                Err(e) => println!("could not process incoming stream, {}", e),
            };
        }
        Ok(())
    }
}
