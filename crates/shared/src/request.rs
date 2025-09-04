use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    Connect,
    Disconnect,
    Echo(Vec<u8>),
    Ping,
}
