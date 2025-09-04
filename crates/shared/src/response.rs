use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    UserID(u16),
    ConnectSuccess,
    ConnectFail,
    Echo(Vec<u8>),
    Ping(u64),
    UserNotFound(Node),
}
