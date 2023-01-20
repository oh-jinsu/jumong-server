use tokio::net::UdpSocket;

use std::{
    collections::{BinaryHeap, HashMap},
    error::Error,
    io,
    net::SocketAddr,
};

use crate::{incoming_packet::Incoming, Job, Schedule, ScheduleQueue};

pub struct Worker {
    udp_socket: UdpSocket,
    udp_addrs: HashMap<String, SocketAddr>,
    schedule_queue: BinaryHeap<Schedule<Job>>,
}

impl Worker {
    pub fn new(udp_socket: UdpSocket) -> Self {
        Worker {
            udp_socket,
            udp_addrs: HashMap::new(),
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
            Ok(_) = self.udp_socket.readable() => {
                Job::Readable
            }
            Ok(_) = self.schedule_queue.wait_for_first() => {
                self.schedule_queue.pop().unwrap().job
            },
        }
    }

    async fn handle_job(&mut self, job: Job) -> Result<(), Box<dyn Error>> {
        match job {
            Job::Readable => {
                let mut buf = [0; 4096];

                let (n, _) = self.udp_socket.try_recv_from(&mut buf)?;

                if n < 2 {
                    return Err("buffer too short".into());
                }

                let len = u16::from_le_bytes([buf[0], buf[1]]).into();

                if n != len {
                    return Err("length not matched".into());
                }

                let incoming = Incoming::deserialize(&buf[2..n])?;

                self.handle_incoming(incoming).await
            }
        }
    }

    async fn handle_incoming(&mut self, incoming: Incoming) -> Result<(), Box<dyn Error>> {
        match incoming {
            Incoming::Hello { token } => Ok(()),
        }
    }
}
