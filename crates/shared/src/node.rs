use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::user::User;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Node {
    Server,
    User(User),
    NoNode,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Server => write!(f, "Server"),
            Node::User(user) => write!(f, "{}", user),
            Node::NoNode => write!(f, "No Node"),
        }
    }
}
