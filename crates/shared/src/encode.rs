use std::fmt::Debug;
use std::io::Result;
use std::io::Write;

pub trait Encode: Debug {
    fn write_encoded(&self, writer: &mut impl Write) -> Result<()>;
    fn as_bytes(&self) -> Result<Vec<u8>>;
}
