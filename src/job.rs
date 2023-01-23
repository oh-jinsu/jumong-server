use std::{collections::HashSet, error::Error, net::SocketAddr};

use tokio::net::TcpStream;

use crate::outgoing_packet::Outgoing;

pub enum Job {
    AcceptFromTcp(TcpStream, SocketAddr),
    DropFromWaiting(usize, Option<Box<dyn Error + Sync + Send>>),
    DropFromTcp(String, Option<Box<dyn Error + Sync + Send>>),
    DropFromUdp(SocketAddr, Option<Box<dyn Error + Sync + Send>>),
    ReadableFromWaiting(usize),
    ReadableFromTcp(String),
    ReadableFromUdp,
    SendToTcp(Outgoing, String),
    SendToUdp(Outgoing, String),
    BroadcastToTcp(Outgoing, HashSet<String>),
    BroadcastToUdp(Outgoing, HashSet<String>),
}
