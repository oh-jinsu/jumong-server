use std::net::SocketAddr;

use tokio::net::TcpStream;

pub struct Connection {
    pub tcp_stream: TcpStream,
    pub udp_addr: Option<SocketAddr>,
}

impl Connection {
    pub fn new(tcp_stream: TcpStream) -> Self {
        Connection {
            tcp_stream,
            udp_addr: None,
        }
    }
}
