use async_std::net::TcpStream;
use serde::{Deserialize, Serialize};

use crate::message::{Message, MessageData, Node};

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

    pub async fn send_message(
        &mut self,
        connection: &TcpStream,
        data: MessageData,
        destination: Node,
    ) -> async_std::io::Result<()> {
        let mut message = Message::new(Node::UserID(self.id), destination, data);
        message.send(&connection).await
    }

    pub async fn send_self(
        &mut self,
        connection: &TcpStream,
        server: Node,
    ) -> async_std::io::Result<()> {
        self.send_message(connection, MessageData::User(self.to_owned()), server)
            .await
    }
}
