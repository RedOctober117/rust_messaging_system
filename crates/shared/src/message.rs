use async_std::{
    io::{Write, WriteExt},
    net::TcpStream,
};

use serde::{Deserialize, Serialize};

use crate::user::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    source: Node,
    destination: Node,
    data: MessageData,
}

impl Message {
    pub fn new(source: Node, destination: Node, data: MessageData) -> Self {
        Self {
            source,
            destination,
            data,
        }
    }

    pub fn get_data(&self) -> &MessageData {
        &self.data
    }

    fn as_json(&mut self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn get_destination(&self) -> Node {
        self.destination
    }

    pub fn get_source(&self) -> Node {
        self.source
    }

    pub async fn send(&mut self, mut connection: &TcpStream) -> async_std::io::Result<()> {
        connection.write_all(self.as_json()?.as_bytes()).await?;
        connection.flush().await
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageData {
    Text(String),
    File(Vec<u8>),
    User(User),
    Failure(String),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Node {
    UserID(u16),
    Server,
}
