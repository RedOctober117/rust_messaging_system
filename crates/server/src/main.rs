use std::io::Result;
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::{fs::File, io::Write};

use futures::executor::block_on;
use session::Session;
use shared::message::{MessageData, MessageType};

pub mod session;

fn main() -> Result<()> {
    println!("Server is initializing. . .");
    let mut session = Session::new((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000))?;
    println!("Ready to accept connections!");
    for conn in session.incoming() {
        let fut = process_connection(conn?);
        _ = block_on(fut);
    }

    Ok(())
}

async fn process_connection(stream: TcpStream) {
    match serde_json::from_reader::<TcpStream, MessageData>(stream) {
        Ok(v) => match v.get_data() {
            MessageType::Text(ve) => println!("got {:?}", ve.trim_matches('0')),
            MessageType::File(ve) => {
                println!("got file");
                let mut ser_file =
                    File::create_new("serialized_image.jpg").expect("could not create file");
                ser_file.write_all(&ve).unwrap()
            }
        },
        Err(e) => println!("could not process incoming stream, {}", e),
    };
}
