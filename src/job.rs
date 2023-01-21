use std::{collections::HashSet, net::SocketAddr};

use tokio::net::TcpStream;

use crate::outgoing_packet::Outgoing;

pub enum Protocol {
    Udp,
    Tcp,
}

pub enum Job {
    AcceptFromTcp(TcpStream, SocketAddr),
    Drop(String),
    ReadableFromWaiting(usize),
    ReadableFromTcp(String),
    ReadableFromUdp,
    Send(Outgoing, String, Protocol),
    Broadcast(Outgoing, HashSet<String>, Protocol),
}
