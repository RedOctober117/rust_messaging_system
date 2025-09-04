use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{message::Message, message_body::MessageBody, node::Node};

#[derive(Clone)]
pub struct MessageBuilder {
    body: MessageBody,
    timestamp: u64,
    source: Node,
    destination: Node,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            source: Node::NoNode,
            destination: Node::NoNode,
            timestamp: 0,
            body: MessageBody::Text(String::with_capacity(140)),
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

    pub fn body(mut self, message_body: MessageBody) -> Self {
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
        self.timestamp = Self::now();
        self
    }

    pub fn build(&self) -> Message {
        Message {
            source: self.source.clone(),
            destination: self.destination.clone(),
            timestamp: self.timestamp,
            body: self.body.to_owned(),
        }
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self {
            source: Node::NoNode,
            destination: Node::NoNode,
            timestamp: Self::now(),
            body: MessageBody::Text("".into()),
        }
    }
}
