use std::{collections::HashSet, error::Error, io};

use crate::{
    incoming_handler_from_tcp::handle_incoming_from_tcp,
    incoming_handler_from_udp::handle_incoming_from_udp,
    incoming_handler_from_waitings::handle_incoming_from_waitings,
    incoming_packet::Incoming,
    job::Job,
    net::{wrap_tcp_packet, Reader},
    outgoing_packet::Outgoing,
    schedule::Schedule,
    Context,
};

pub async fn handle(job: Job, context: &mut Context) -> Result<(), Box<dyn Error>> {
    match job {
        Job::AcceptFromTcp(stream, _) => {
            context.waitings.push(stream);

            Ok(())
        }
        Job::ReadableFromWaiting(index) => {
            if let Some(stream) = context.waitings.get(index) {
                let mut buf = [0; 4096];

                let n = match stream.try_read_packet_buf(&mut buf) {
                    Ok(n) => n,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                    Err(e) => {
                        let job = Job::DropFromWaiting(index, Some(e.into()));

                        let schedule = Schedule::instant(job);

                        context.schedule_queue.push(schedule);

                        return Ok(());
                    }
                };

                let incoming = match Incoming::deserialize(&buf[..n]) {
                    Ok(incoming) => incoming,
                    Err(e) => {
                        let job = Job::DropFromWaiting(index, Some(e.into()));

                        let schedule = Schedule::instant(job);

                        context.schedule_queue.push(schedule);

                        return Ok(());
                    }
                };

                if let Err(e) = handle_incoming_from_waitings(incoming, index, context).await {
                    let job = Job::DropFromWaiting(index, Some(e.into()));

                    let schedule = Schedule::instant(job);

                    context.schedule_queue.push(schedule);

                    return Ok(());
                }

                Ok(())
            } else {
                Err("no tcp stream".into())
            }
        }
        Job::ReadableFromTcp(id) => {
            if let Some(stream) = context.tcp_streams.get(&id) {
                let mut buf = [0; 4096];

                let n = match stream.try_read_packet_buf(&mut buf) {
                    Ok(n) => n,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                    Err(e) => {
                        let schedule = Schedule::instant(Job::DropFromTcp(id, Some(e.into())));

                        context.schedule_queue.push(schedule);

                        return Ok(());
                    }
                };

                let incoming = match Incoming::deserialize(&buf[..n]) {
                    Ok(incoming) => incoming,
                    Err(e) => {
                        let schedule = Schedule::instant(Job::DropFromTcp(id, Some(e)));

                        context.schedule_queue.push(schedule);

                        return Ok(());
                    }
                };

                if let Err(e) = handle_incoming_from_tcp(incoming, id.clone(), context).await {
                    let schedule = Schedule::instant(Job::DropFromTcp(id, Some(e)));

                    context.schedule_queue.push(schedule);
                }

                Ok(())
            } else {
                Err("no tcp stream".into())
            }
        }
        Job::ReadableFromUdp => {
            let mut buf = [0; 4096];

            let (n, addr) = match context.udp_socket.try_recv_from(&mut buf) {
                Ok(x) => x,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                Err(e) => return Err(e.into()),
            };

            let incoming = match Incoming::deserialize(&buf[..n]) {
                Ok(incoming) => incoming,
                Err(e) => {
                    let schedule = Schedule::instant(Job::DropFromUdp(addr, Some(e.into())));

                    context.schedule_queue.push(schedule);

                    return Ok(());
                }
            };

            if let Err(e) = handle_incoming_from_udp(incoming, addr, context).await {
                let schedule = Schedule::instant(Job::DropFromUdp(addr, Some(e)));

                context.schedule_queue.push(schedule);
            }

            Ok(())
        }
        Job::DropFromWaiting(index, e) => {
            if let Some(e) = e {
                eprintln!("waiting dropped for {e:?}");
            }

            context.waitings.remove(index);

            Ok(())
        }
        Job::DropFromTcp(id, e) => {
            if let Some(e) = e {
                eprintln!("tcp stream dropped for {e:?}");
            }

            context.tcp_streams.remove(&id);

            context.udp_addrs.remove_by_key(&id);

            let packet = Outgoing::GoodBye { id };

            let schedule = Schedule::instant(Job::BroadcastToTcp(packet, HashSet::new()));

            context.schedule_queue.push(schedule);

            Ok(())
        }
        Job::DropFromUdp(addr, e) => {
            if let Some(e) = e {
                eprintln!("udp addr dropped for {e:?}");
            }

            context.udp_addrs.remove_by_value(&addr);

            Ok(())
        }
        Job::SendToTcp(packet, id) => {
            if let Some(stream) = context.tcp_streams.get(&id) {
                let buf = packet.serilaize()?;

                let buf = wrap_tcp_packet(&buf);

                stream.try_write(&buf)?;

                Ok(())
            } else {
                Err("no stream to send".into())
            }
        }
        Job::SendToUdp(packet, id) => {
            if let Some(addr) = context.udp_addrs.get_by_key(&id) {
                let buf = packet.serilaize()?;

                context.udp_socket.try_send_to(&buf, addr.clone())?;

                Ok(())
            } else {
                Err("no stream to send".into())
            }
        }
        Job::BroadcastToTcp(packet, ex) => {
            let buf = packet.serilaize()?;

            let buf = wrap_tcp_packet(&buf);

            for (id, stream) in context.tcp_streams.iter() {
                if ex.contains(id) {
                    continue;
                }

                stream.try_write(&buf)?;
            }

            Ok(())
        }
        Job::BroadcastToUdp(packet, ex) => {
            let buf = packet.serilaize()?;

            for (id, addr) in context.udp_addrs.iter() {
                if ex.contains(id) {
                    continue;
                }

                context.udp_socket.try_send_to(&buf, addr.clone())?;
            }

            Ok(())
        }
    }
}
