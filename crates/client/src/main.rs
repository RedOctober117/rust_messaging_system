use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

use client_session::ClientSession;
use futures::join;
use shared::message::{Destination, MessageData, MessageType};
use shared::user::User;

pub mod client_session;

#[async_std::main]
async fn main() -> Result<()> {
    let future_1 = async {
        println!("starting client 0 thread");
        let mut text_user = User::new(0, "text".into());

        let connection_addr = (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
        let conn = establish_connection(connection_addr).unwrap();

        let mut session = ClientSession::new(
            &mut text_user,
            (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5001),
            connection_addr,
        )
        .unwrap();

        // session.process_loop().await.unwrap();

        let test_message = MessageData::new(
            Destination::User(1),
            MessageType::Text("Hello from client!".into()),
        );

        text_user.send_message(&conn, test_message).unwrap();
        println!("user 0 sending hello to user 1");
    }
    .await;

    let future_2 = async {
        println!("starting client 1 thread");
        let mut image_user = User::new(1, "image".into());

        let connection_addr = (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
        let conn = establish_connection(connection_addr).unwrap();

        let mut session = ClientSession::new(
            &mut image_user,
            (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5002),
            connection_addr,
        )
        .unwrap();
        // session.process_loop().await.unwrap();

        let mut file = File::open("test_image.jpg").unwrap();
        let mut file_buffer: Vec<u8> = vec![];
        file.read_to_end(&mut file_buffer)
            .expect("couldnt read file to vec");

        let test_message = MessageData::new(Destination::User(0), MessageType::File(file_buffer));

        println!("user 1 is sending image to user 0");
        image_user.send_message(&conn, test_message).unwrap();
    }
    .await;

    Ok(())
}

pub fn establish_connection(address: (IpAddr, u16)) -> Result<TcpStream> {
    TcpStream::connect(address)
}
