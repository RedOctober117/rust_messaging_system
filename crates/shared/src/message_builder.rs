use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    message::Message,
    message_data::{LoginStateData, MessageData},
    node::Node,
    varuint::VarUInt,
};

#[derive(Clone)]
pub struct MessageBuilder {
    body: MessageData,
    timestamp: VarUInt,
    source: Node,
    destination: Node,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            source: Node::default(),
            destination: Node::default(),
            timestamp: VarUInt(0),
            body: MessageData::LoginStateData(LoginStateData::RequestConnect),
        }
    }

    pub fn source(mut self, src: Node) -> Self {
        self.source = src;
        self
    }

    pub fn destination(mut self, dest: Node) -> Self {
        self.destination = dest;
        self
    }

    pub fn body(mut self, message_body: MessageData) -> Self {
        self.body = message_body;
        self
    }

    pub fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
    }

    pub fn timestamp(mut self) -> Self {
        self.timestamp = VarUInt(Self::now());
        self
    }

    pub fn build(&self) -> Message {
        Message {
            source: self.source.clone(),
            destination: self.destination.clone(),
            timestamp: self.timestamp,
            data: self.body.to_owned(),
        }
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self {
            source: Node::default(),
            destination: Node::default(),
            timestamp: VarUInt(Self::now()),
            body: MessageData::LoginStateData(LoginStateData::RequestConnect),
        }
    }
}
