use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{request::Request, response::Response, user::User};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageBody {
    // refactor text file and user as enum communication with aforementioned as impls
    Text(String),
    File(Vec<u8>),
    User(User),
    Request(Request),
    Response(Response),
}

impl Display for MessageBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageBody::Text(t) => write!(f, "{{ Text: {t} }}"),
            MessageBody::File(_) => write!(f, "{{ <file> }}"),
            MessageBody::User(user) => write!(f, "{{ User: {user} }}"),
            MessageBody::Request(request) => write!(f, "{{ Request: {request:?} }}"),
            MessageBody::Response(response) => write!(f, "{{ Response: {response:?} }}"),
        }
    }
}
