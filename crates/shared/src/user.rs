use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    id: u16,
    name: String,
    location: (IpAddr, u16),
}

impl User {
    pub fn new(id: u16, name: String, location: (IpAddr, u16)) -> Self {
        Self { id, name, location }
    }

    pub fn as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn location(&self) -> (IpAddr, u16) {
        self.location
    }
}
