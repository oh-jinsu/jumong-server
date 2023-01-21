use std::error::Error;

use jumong_server::{env, Worker};
use tokio::net::{TcpListener, UdpSocket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env::init();

    let tcp_listener = TcpListener::bind("0.0.0.0:3000").await?;

    let udp_socket = UdpSocket::bind("0.0.0.0:3000").await?;

    let worker = Worker::new(tcp_listener, udp_socket);

    worker.run().await
}
