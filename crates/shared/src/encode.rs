use std::io::Result;
use std::io::Write;

pub trait Encode {
    fn write_encoded(&self, writer: &mut impl Write) -> Result<()>;
    fn as_bytes(&self) -> Result<Vec<u8>>;
}
