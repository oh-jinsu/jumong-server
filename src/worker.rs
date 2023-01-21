use reqwest::{header::AUTHORIZATION, StatusCode};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

use std::{
    collections::{BinaryHeap, HashMap},
    error::Error,
    net::SocketAddr,
};

use crate::{
    connection::Connection, http_response::AuthResponse, incoming_packet::Incoming, job::Protocol,
    outgoing_packet::Outgoing, url::endpoint, Job, Readable, Schedule, ScheduleQueue,
};

pub struct Worker {
    tcp_listener: TcpListener,
    waitings: Vec<TcpStream>,
    connections: HashMap<String, Connection>,
    udp_socket: UdpSocket,
    schedule_queue: BinaryHeap<Schedule<Job>>,
}

impl Worker {
    pub fn new(tcp_listener: TcpListener, udp_socket: UdpSocket) -> Self {
        Worker {
            tcp_listener,
            waitings: Vec::new(),
            connections: HashMap::new(),
            udp_socket,
            schedule_queue: BinaryHeap::new(),
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let job = self.select_job().await;

            if let Err(e) = self.handle_job(job).await {
                eprintln!("{e}");
            }
        }
    }

    async fn select_job(&mut self) -> Job {
        if self.schedule_queue.is_first_urgent() {
            return self.schedule_queue.pop().unwrap().job;
        }

        tokio::select! {
            Ok((stream, addr)) = self.tcp_listener.accept() => {
                Job::AcceptFromTcp(stream, addr)
            }
            Ok(i) = self.waitings.readable() => {
                Job::ReadableFromWaiting(i)
            }
            Ok(addr) = self.connections.readable() => {
                Job::ReadableFromTcp(addr)
            }
            Ok(_) = self.udp_socket.readable() => {
                Job::ReadableFromUdp
            }
            Ok(_) = self.schedule_queue.wait_for_first() => {
                self.schedule_queue.pop().unwrap().job
            },
        }
    }

    async fn handle_job(&mut self, job: Job) -> Result<(), Box<dyn Error>> {
        match job {
            Job::AcceptFromTcp(stream, addr) => {
                let id = addr.to_string();

                let conn = Connection::new(stream);

                self.connections.insert(id, conn);

                Ok(())
            }
            Job::ReadableFromWaiting(i) => {
                if let Some(stream) = self.waitings.get(i) {
                    let mut buf = [0; 2];

                    let n = stream.try_read(&mut buf)?;

                    if n == 0 {
                        self.waitings.remove(i);

                        return Ok(());
                    }

                    if n < 2 {
                        return Err("buffer too short".into());
                    }

                    let len = usize::from(u16::from_le_bytes(buf));

                    if len > 4096 {
                        return Err("packet too big".into());
                    }

                    let mut buf = Vec::with_capacity(len);

                    let n = stream.try_read(&mut buf)?;

                    if n == 0 {
                        self.waitings.remove(i);

                        return Ok(());
                    }

                    if n != len {
                        return Err("length not matched".into());
                    }

                    let incoming = Incoming::deserialize(&buf)?;

                    if let Err(_) = self.handle_incoming_from_waiting(incoming, i).await {
                        self.waitings.remove(i);

                        return Ok(());
                    }

                    Ok(())
                } else {
                    Err("no tcp stream".into())
                }
            }
            Job::ReadableFromTcp(id) => {
                if let Some(conn) = self.connections.get(&id) {
                    let mut buf = [0; 2];

                    let n = conn.tcp_stream.try_read(&mut buf)?;

                    if n == 0 {
                        let schedule = Schedule::instant(Job::Drop(id));

                        self.schedule_queue.push(schedule);

                        return Ok(());
                    }

                    if n < 2 {
                        return Err("buffer too short".into());
                    }

                    let len = usize::from(u16::from_le_bytes(buf));

                    if len > 4096 {
                        return Err("packet too big".into());
                    }

                    let mut buf = Vec::with_capacity(len);

                    let n = conn.tcp_stream.try_read(&mut buf)?;

                    if n == 0 {
                        let schedule = Schedule::instant(Job::Drop(id));

                        self.schedule_queue.push(schedule);

                        return Ok(());
                    }

                    if n != len {
                        return Err("length not matched".into());
                    }

                    let incoming = Incoming::deserialize(&buf)?;

                    if let Err(_) = self.handle_incoming_from_tcp(incoming, id.clone()).await {
                        let schedule = Schedule::instant(Job::Drop(id));

                        self.schedule_queue.push(schedule);

                        return Ok(());
                    }

                    Ok(())
                } else {
                    Err("no tcp stream".into())
                }
            }
            Job::Drop(id) => {
                self.connections.remove(&id);

                Ok(())
            }
            Job::ReadableFromUdp => {
                let mut buf = [0; 4096];

                let (n, addr) = self.udp_socket.try_recv_from(&mut buf)?;

                if n < 2 {
                    return Err("buffer too short".into());
                }

                let len = usize::from(u16::from_le_bytes([buf[0], buf[1]]));

                if n != len {
                    return Err("length not matched".into());
                }

                let incoming = Incoming::deserialize(&buf[2..n])?;

                self.handle_incoming_from_udp(incoming, addr).await
            }
            Job::Send(_, _, _) => todo!(),
            Job::Broadcast(_, _, _) => todo!(),
        }
    }

    async fn handle_incoming_from_udp(
        &mut self,
        incoming: Incoming,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn Error>> {
        match incoming {
            Incoming::Hello { token } => {
                let response = reqwest::Client::new()
                    .get(endpoint("auth"))
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .send()
                    .await?;

                let response = match response.status() {
                    StatusCode::CREATED => response.json::<AuthResponse>().await?,
                    _ => return Err(response.text().await?.into()),
                };

                if let Some(conn) = self.connections.get_mut(&response.id) {
                    let _ = conn.udp_addr.insert(addr);

                    Ok(())
                } else {
                    Err("no connection".into())
                }
            }
        }
    }

    async fn handle_incoming_from_waiting(
        &mut self,
        incoming: Incoming,
        i: usize,
    ) -> Result<(), Box<dyn Error>> {
        match incoming {
            Incoming::Hello { token } => {
                let response = reqwest::Client::new()
                    .get(endpoint("auth"))
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .send()
                    .await?;

                let response = match response.status() {
                    StatusCode::CREATED => response.json::<AuthResponse>().await?,
                    _ => return Err(response.text().await?.into()),
                };

                let stream = self.waitings.remove(i);

                let conn = Connection::new(stream);

                self.connections.insert(response.id.clone(), conn);

                let id = response.id;

                let packet = Outgoing::Hello { id: id.clone() };

                let schedule = Schedule::instant(Job::Send(packet, id, Protocol::Tcp));

                self.schedule_queue.push(schedule);

                Ok(())
            }
        }
    }

    async fn handle_incoming_from_tcp(
        &mut self,
        incoming: Incoming,
        id: String,
    ) -> Result<(), Box<dyn Error>> {
        match incoming {
            _ => todo!(),
        }
    }
}
