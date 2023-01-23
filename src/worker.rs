use reqwest::{header::AUTHORIZATION, StatusCode};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

use std::{
    collections::{BinaryHeap, HashMap},
    error::Error,
    io,
    net::SocketAddr,
};

use crate::{
    collection::BiMap, http_response::AuthResponse, incoming_packet::Incoming, job::Protocol,
    net::Reader, outgoing_packet::Outgoing, url::endpoint, Job, Readable, Schedule, ScheduleQueue,
};

pub struct Worker {
    tcp_listener: TcpListener,
    waitings: Vec<TcpStream>,
    tcp_streams: HashMap<String, TcpStream>,
    udp_socket: UdpSocket,
    udp_addrs: BiMap<String, SocketAddr>,
    schedule_queue: BinaryHeap<Schedule<Job>>,
}

impl Worker {
    pub fn new(tcp_listener: TcpListener, udp_socket: UdpSocket) -> Self {
        Worker {
            tcp_listener,
            waitings: Vec::new(),
            tcp_streams: HashMap::new(),
            udp_socket,
            udp_addrs: BiMap::new(),
            schedule_queue: BinaryHeap::new(),
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let job = self.select_job().await;

            if let Err(e) = self.handle_job(job).await {
                eprintln!("job failed for {e}");
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
            Ok(addr) = self.tcp_streams.readable() => {
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

                self.tcp_streams.insert(id, stream);

                Ok(())
            }
            Job::ReadableFromWaiting(index) => {
                if let Some(stream) = self.waitings.get(index) {
                    let mut buf = [0; 4096];

                    let n = match stream.try_read_packet_buf(&mut buf) {
                        Ok(n) => n,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                        Err(e) => {
                            let job = Job::DropFromWaiting(index, Some(e.into()));

                            let schedule = Schedule::instant(job);

                            self.schedule_queue.push(schedule);

                            return Ok(());
                        }
                    };

                    let incoming = match Incoming::deserialize(&buf[..n]) {
                        Ok(incoming) => incoming,
                        Err(e) => {
                            let job = Job::DropFromWaiting(index, Some(e.into()));

                            let schedule = Schedule::instant(job);

                            self.schedule_queue.push(schedule);

                            return Ok(());
                        }
                    };

                    if let Err(e) = self.handle_incoming_from_waiting(incoming, index).await {
                        let job = Job::DropFromWaiting(index, Some(e.into()));

                        let schedule = Schedule::instant(job);

                        self.schedule_queue.push(schedule);

                        return Ok(());
                    }

                    Ok(())
                } else {
                    Err("no tcp stream".into())
                }
            }
            Job::ReadableFromTcp(id) => {
                if let Some(stream) = self.tcp_streams.get(&id) {
                    let mut buf = [0; 4096];

                    let n = match stream.try_read_packet_buf(&mut buf) {
                        Ok(n) => n,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                        Err(e) => {
                            let schedule = Schedule::instant(Job::DropFromTcp(id, Some(e.into())));

                            self.schedule_queue.push(schedule);

                            return Ok(());
                        }
                    };

                    let incoming = match Incoming::deserialize(&buf[..n]) {
                        Ok(incoming) => incoming,
                        Err(e) => {
                            let schedule = Schedule::instant(Job::DropFromTcp(id, Some(e)));

                            self.schedule_queue.push(schedule);

                            return Ok(());
                        }
                    };

                    if let Err(e) = self.handle_incoming_from_tcp(incoming, id.clone()).await {
                        let schedule = Schedule::instant(Job::DropFromTcp(id, Some(e)));

                        self.schedule_queue.push(schedule);
                    }

                    Ok(())
                } else {
                    Err("no tcp stream".into())
                }
            }
            Job::ReadableFromUdp => {
                let mut buf = [0; 4096];

                let (n, addr) = self.udp_socket.try_recv_from(&mut buf)?;

                let incoming = match Incoming::deserialize(&buf[..n]) {
                    Ok(incoming) => incoming,
                    Err(e) => {
                        let schedule = Schedule::instant(Job::DropFromUdp(addr, Some(e.into())));

                        self.schedule_queue.push(schedule);

                        return Ok(());
                    }
                };

                if let Err(e) = self.handle_incoming_from_udp(incoming, addr).await {
                    let schedule = Schedule::instant(Job::DropFromUdp(addr, Some(e)));

                    self.schedule_queue.push(schedule);
                }

                Ok(())
            }
            Job::DropFromWaiting(index, e) => {
                if let Some(e) = e {
                    eprintln!("waiting dropped for {e:?}");
                }

                self.waitings.remove(index);

                Ok(())
            }
            Job::DropFromTcp(id, e) => {
                if let Some(e) = e {
                    eprintln!("tcp stream dropped for {e:?}");
                }

                self.tcp_streams.remove(&id);

                self.udp_addrs.remove_by_key(&id);

                Ok(())
            }
            Job::DropFromUdp(addr, e) => {
                if let Some(e) = e {
                    eprintln!("udp addr dropped for {e:?}");
                }

                self.udp_addrs.remove_by_value(&addr);

                Ok(())
            }
            Job::Send(_, _, _) => Ok(()),
            Job::Broadcast(_, _, _) => Ok(()),
        }
    }

    async fn handle_incoming_from_udp(
        &mut self,
        incoming: Incoming,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        match incoming {
            Incoming::UdpHello { token } => {
                let response = reqwest::Client::new()
                    .get(endpoint("auth"))
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .send()
                    .await?;

                let response = match response.status() {
                    StatusCode::CREATED => response.json::<AuthResponse>().await?,
                    _ => return Err(response.text().await?.into()),
                };

                let id = response.id;

                self.udp_addrs.insert(id.clone(), addr);

                let packet = Outgoing::UdpHello { id: id.clone() };

                let schedule = Schedule::instant(Job::Send(packet, id, Protocol::Tcp));

                self.schedule_queue.push(schedule);

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn handle_incoming_from_waiting(
        &mut self,
        incoming: Incoming,
        i: usize,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        match incoming {
            Incoming::TcpHello { token } => {
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

                self.tcp_streams.insert(response.id.clone(), stream);

                let id = response.id;

                let packet = Outgoing::TcpHello { id: id.clone() };

                let schedule = Schedule::instant(Job::Send(packet, id, Protocol::Tcp));

                self.schedule_queue.push(schedule);

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn handle_incoming_from_tcp(
        &mut self,
        incoming: Incoming,
        _: String,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        match incoming {
            _ => Ok(()),
        }
    }
}
