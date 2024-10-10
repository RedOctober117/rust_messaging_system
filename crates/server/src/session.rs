use shared::user::*;
use std::io::Result;
use std::net::{Incoming, IpAddr, TcpListener};

pub struct Session {
    address: (IpAddr, u16),
    listener: TcpListener,
    users: Vec<User>,
}

impl Session {
    pub fn new(address: (IpAddr, u16)) -> Result<Self> {
        let listener = TcpListener::bind(address)?;
        Ok(Self {
            address,
            listener,
            users: vec![],
        })
    }

    pub fn incoming(&mut self) -> Incoming {
        self.listener.incoming()
    }

    pub fn get_address(&self) -> (IpAddr, u16) {
        self.address
    }

    pub fn get_users(&self) -> &Vec<User> {
        &self.users
    }
}
