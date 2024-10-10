use serde::{Deserialize, Serialize};
use std::{io::Write, net::TcpStream};

use crate::message::{Destination, MessageData, MessageType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    id: u16,
    name: String,
}

impl User {
    pub fn new(id: u16, name: String) -> Self {
        Self { id, name }
    }

    pub fn as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn send_message(
        &mut self,
        mut connection: &TcpStream,
        mut message: MessageData,
    ) -> std::io::Result<()> {
        let ser_message = message.as_json()?;
        connection.write_all(&ser_message.as_bytes())?;
        connection.flush().unwrap();
        connection.shutdown(std::net::Shutdown::Both)
    }

    pub fn send_self(&mut self, mut connection: &TcpStream) -> std::io::Result<()> {
        let ser_message =
            MessageData::new(Destination::Server, MessageType::User(self.clone())).as_json()?;

        connection.write_all(&ser_message.as_bytes())?;
        connection.flush().unwrap();
        connection.shutdown(std::net::Shutdown::Both)
    }
}
