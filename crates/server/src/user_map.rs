use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use shared::{message::Message, source_or_destination::SourceOrDestination};
use tokio::sync::mpsc::Sender;

pub struct UserMap {
    upstream_map: Mutex<HashMap<SourceOrDestination, Sender<Message>>>,
}

impl UserMap {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            upstream_map: Mutex::new(HashMap::new()),
        })
    }

    pub fn insert(
        &self,
        id: SourceOrDestination,
        upstream: Sender<Message>,
    ) -> Option<Sender<Message>> {
        let mut lock = self.upstream_map.lock().unwrap();
        lock.insert(id, upstream)
    }

    pub fn get(&self, id: &SourceOrDestination) -> Option<Sender<Message>> {
        let lock = self.upstream_map.lock().unwrap();
        lock.get(id).cloned()
    }

    pub fn contains_key(&self, id: &SourceOrDestination) -> bool {
        let lock = self.upstream_map.lock().unwrap();
        lock.contains_key(id)
    }

    pub fn remove(&self, id: &SourceOrDestination) -> Option<Sender<Message>> {
        let mut lock = self.upstream_map.lock().unwrap();
        lock.remove(id)
    }
}
