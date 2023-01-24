use std::{
    collections::{BinaryHeap, HashMap},
    net::SocketAddr,
};

use tokio::net::{TcpListener, TcpStream, UdpSocket};

use crate::{collection::BiMap, job::Job, schedule::Schedule};

pub struct Context {
    pub tcp_listener: TcpListener,
    pub waitings: Vec<TcpStream>,
    pub tcp_streams: HashMap<String, TcpStream>,
    pub udp_socket: UdpSocket,
    pub udp_addrs: BiMap<String, SocketAddr>,
    pub schedule_queue: BinaryHeap<Schedule<Job>>,
}

impl Context {
    pub fn new(tcp_listener: TcpListener, udp_socket: UdpSocket) -> Self {
        Context {
            tcp_listener,
            waitings: Vec::new(),
            tcp_streams: HashMap::new(),
            udp_socket,
            udp_addrs: BiMap::new(),
            schedule_queue: BinaryHeap::new(),
        }
    }
}
