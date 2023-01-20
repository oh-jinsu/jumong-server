use std::{collections::HashSet, error::Error, io};

use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let udp_socket = UdpSocket::bind("0.0.0.0:3001").await?;

    let mut udp_addrs = HashSet::new();

    let mut buf = [0; 1024];

    loop {
        udp_socket.readable().await?;

        let (n, addr) = match udp_socket.try_recv_from(&mut buf) {
            Ok(x) => x,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => return Err(e.into()),
        };

        udp_addrs.insert(addr);

        if n == 0 {
            continue;
        }

        let buf = &buf[..n];

        match &buf[..2] {
            [1, 0] => {
                println!("Yep!");
            }
            [2, 0] => {
                println!("Yep!");
            }
            _ => {}
        }
    }
}
