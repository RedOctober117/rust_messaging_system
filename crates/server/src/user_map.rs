use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use shared::message::Message;
use tokio::sync::mpsc::Sender;

pub struct UserMap {
    map: Mutex<HashMap<u16, Sender<Message>>>,
}

impl UserMap {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            map: Mutex::new(HashMap::new()),
        })
    }

    pub fn insert(&self, id: u16, upstream: Sender<Message>) -> Option<Sender<Message>> {
        let mut lock = self.map.lock().unwrap();
        lock.insert(id, upstream)
    }

    pub fn get(&self, id: &u16) -> Option<Sender<Message>> {
        let lock = self.map.lock().unwrap();
        match lock.get(&id) {
            Some(upstream) => Some(upstream.clone()),
            None => None,
        }
    }
}
