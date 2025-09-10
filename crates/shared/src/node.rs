use std::io::{Read, Write};

// use serde::{Deserialize, Serialize};

use crate::{decode::Decode, encode::Encode, varuint::VarUInt, NO_STATE};

pub const SERVER_NODE: [u8; 2] = [1, 0];

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node(Vec<u8>);

impl Node {
    pub fn new(name: impl Into<Vec<u8>>) -> Self {
        Self(name.into())
    }
}

impl Default for Node {
    fn default() -> Self {
        Self(vec![])
    }
}

impl Encode for Node {
    fn write_encoded(&self, _state: u8, writer: &mut impl Write) -> std::io::Result<()> {
        writer
            .write(&mut VarUInt(self.0.len() as u64).encode())
            .unwrap();
        writer.write_all(&self.0)
    }

    fn as_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![];
        self.write_encoded(NO_STATE, &mut buffer).unwrap();
        Ok(buffer)
    }
}

impl Decode for Node {
    fn decode_reader(
        _state: u8,
        reader: &mut impl std::io::BufRead,
    ) -> Result<Box<Node>, Box<(dyn std::error::Error)>> {
        let length = *VarUInt::decode_reader(NO_STATE, reader)?;

        let mut take = reader.take(u64::from(length));
        let mut buf: Vec<u8> = vec![];

        take.read_to_end(&mut buf)?;

        Ok(Box::new(Node(buf)))
    }
}

#[cfg(test)]
mod test {
    use crate::{encode::Encode, node::Node};

    #[test]
    fn user_encoding() {
        let user = "TestUser";
        let user_node = Node::new(user);

        assert_eq!(user_node.as_bytes().unwrap().len(), user.len() + 1)
    }
}
