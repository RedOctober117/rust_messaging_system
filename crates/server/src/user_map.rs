use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use shared::{message::Message, user::User};
use tokio::sync::mpsc::Sender;

pub struct UserMap {
    upstream_map: Mutex<HashMap<User, Sender<Message>>>,
}

impl UserMap {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            upstream_map: Mutex::new(HashMap::new()),
        })
    }

    pub fn insert(&self, id: User, upstream: Sender<Message>) -> Option<Sender<Message>> {
        let mut lock = self.upstream_map.lock().unwrap();
        lock.insert(id, upstream)
    }

    pub fn get(&self, id: &User) -> Option<Sender<Message>> {
        let lock = self.upstream_map.lock().unwrap();
        match lock.get(&id) {
            Some(upstream) => Some(upstream.clone()),
            None => None,
        }
    }
}
