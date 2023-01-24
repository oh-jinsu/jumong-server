use std::{collections::HashSet, error::Error, net::SocketAddr};

use reqwest::{header::AUTHORIZATION, StatusCode};

use crate::{
    http_response::AuthResponse, incoming_packet::Incoming, job::Job, outgoing_packet::Outgoing,
    schedule::Schedule, url::endpoint, Context,
};

pub async fn handle_incoming_from_udp(
    incoming: Incoming,
    addr: SocketAddr,
    context: &mut Context,
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

            context.udp_addrs.insert(id.clone(), addr);

            let packet = Outgoing::HelloFromUdp { id: id.clone() };

            let schedule = Schedule::instant(Job::SendToTcp(packet, id));

            context.schedule_queue.push(schedule);

            Ok(())
        }
        Incoming::UpdateOrigin { origin } => {
            if let Some(id) = context.udp_addrs.get_by_val(&addr) {
                let packet = Outgoing::UpdateOrigin {
                    id: id.to_owned(),
                    origin,
                };

                let ex = HashSet::from_iter([id.to_owned()]);

                let schedule = Schedule::instant(Job::BroadcastToUdp(packet, ex));

                context.schedule_queue.push(schedule);

                Ok(())
            } else {
                Err("no addr".into())
            }
        }
        Incoming::UpdateRotation { y } => {
            if let Some(id) = context.udp_addrs.get_by_val(&addr) {
                let packet = Outgoing::UpdateRotation {
                    id: id.to_owned(),
                    y,
                };

                let ex = HashSet::from_iter([id.to_owned()]);

                let schedule = Schedule::instant(Job::BroadcastToUdp(packet, ex));

                context.schedule_queue.push(schedule);

                Ok(())
            } else {
                Err("no addr".into())
            }
        }
        _ => Ok(()),
    }
}
