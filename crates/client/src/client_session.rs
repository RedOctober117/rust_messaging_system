use std::io::Result;
use std::net::{IpAddr, TcpListener, TcpStream};

use shared::message::MessageData;
use shared::user::User;

pub struct ClientSession<'a> {
    user: &'a mut User,
    location: (IpAddr, u16),
    listener: TcpListener,
    server: TcpStream,
}

impl<'a> ClientSession<'a> {
    pub fn new(user: &'a mut User, location: (IpAddr, u16), server: (IpAddr, u16)) -> Result<Self> {
        Ok(Self {
            user,
            location,
            listener: TcpListener::bind(location)?,
            server: TcpStream::connect(server)?,
        })
    }

    pub fn send_user(&mut self) -> Result<()> {
        self.user.send_self(&self.server)?;
        Ok(())
    }

    pub async fn process_loop(&mut self) -> Result<()> {
        for connection in self.listener.incoming() {
            match serde_json::from_reader::<TcpStream, MessageData>(connection?) {
                Ok(data) => match data.get_data() {
                    shared::message::MessageType::Text(v) => println!("got text! {}", v),
                    shared::message::MessageType::File(_) => println!("got image!"),
                    shared::message::MessageType::User(user) => todo!(),
                },
                Err(e) => println!("Could not process connection, {}", e),
            }
        }
        Ok(())
    }
}
