use std::io::{Read, Write};

// use serde::{Deserialize, Serialize};

use crate::{decode::Decode, encode::Encode, varuint::VarUInt, NO_STATE};

pub const SERVER_NODE: [u8; 2] = [1, 0];

#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node(Vec<u8>);

impl Node {
    pub fn new(name: impl Into<Vec<u8>>) -> Self {
        Self(name.into())
    }
}

impl Encode for Node {
    fn write_encoded(&self, _state: u8, writer: &mut impl Write) -> std::io::Result<()> {
        VarUInt(self.0.len() as u64).write_encoded(NO_STATE, writer)?;
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
    use crate::{decode::Decode, encode::Encode, node::Node, varuint::VarUInt, NO_STATE};

    #[test]
    fn user_encoding() {
        let user: &[u8] = "TestUser".as_ref();
        let user_node = Node::new(user);
        let mut buf = vec![];

        user_node.write_encoded(NO_STATE, &mut buf).unwrap();

        let mut check_buf = vec![];
        VarUInt(user.len() as u64)
            .write_encoded(NO_STATE, &mut check_buf)
            .unwrap();
        check_buf.append(&mut Vec::from(user));

        assert_eq!(buf, check_buf)
    }

    #[test]
    fn user_decoding() {
        let mut raw_bytes: &[u8] = &[8, 84, 101, 115, 116, 85, 115, 101, 114];

        assert_eq!(
            *Node::decode_reader(NO_STATE, &mut raw_bytes).unwrap(),
            Node::new("TestUser")
        )
    }
}
