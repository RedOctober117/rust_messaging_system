use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageData {
    data: MessageType,
}

impl MessageData {
    pub fn new(data: MessageType) -> Self {
        Self { data }
    }

    pub fn get_data(&self) -> &MessageType {
        &self.data
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    Text(String),
    File(Vec<u8>),
}
