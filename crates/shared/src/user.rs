use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, Ord)]
pub struct User {
    id: u16,
    name: String,
}

impl User {
    pub fn new<S: Into<String>>(id: u16, name: S) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    pub fn as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn format(&self) -> String {
        format!("{}[{}]", self.name, self.id)
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for User {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.id.partial_cmp(&other.id) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.name.partial_cmp(&other.name)
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ ID: {{ {} }}, Username: {{ {} }} }}",
            self.id, self.name
        )
    }
}
