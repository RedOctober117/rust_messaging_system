use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::TcpStream;

use shared::message::MessageData;
use shared::message::MessageType;

fn main() -> Result<()> {
    let connection_addr = (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
    let mut conn = establish_connection(connection_addr)?;

    let mut file = File::open("test_image.jpg").unwrap();
    let mut file_buffer: Vec<u8> = vec![];
    file.read_to_end(&mut file_buffer)
        .expect("couldnt read file to vec");

    let mut test_message = MessageData::new(MessageType::File(file_buffer));
    let mut ser_test_message = serde_json::to_string(&test_message)?;
    println!("serialized file, sending. . .");
    conn.write_all(ser_test_message.as_bytes())?;

    conn.flush()?;
    conn.shutdown(std::net::Shutdown::Both)?;

    conn = establish_connection(connection_addr)?;
    test_message = MessageData::new(MessageType::Text("Hello from client!".into()));
    ser_test_message = serde_json::to_string(&test_message)?;
    println!("serialized \"{}\", sending. . .", ser_test_message);

    conn.write_all(ser_test_message.as_bytes())?;
    conn.flush()?;
    conn.shutdown(std::net::Shutdown::Both)?;

    Ok(())
}

pub fn establish_connection(address: (IpAddr, u16)) -> Result<TcpStream> {
    TcpStream::connect(address)
}
