use std::{fs::File, net::TcpStream};

use serde::{Deserialize, Serialize};

use crate::user::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageData {
    destination: Destination,
    data: MessageType,
}

impl MessageData {
    pub fn new(destination: Destination, data: MessageType) -> Self {
        Self { destination, data }
    }

    pub fn get_data(&self) -> &MessageType {
        &self.data
    }

    pub fn get_destination(&self) -> Destination {
        self.destination
    }

    pub fn as_json(&mut self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    Text(String),
    File(Vec<u8>),
    User(User),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Destination {
    User(u16),
    Server,
}
