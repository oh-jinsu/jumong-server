use std::{error::Error, net::SocketAddr};

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
        _ => Ok(()),
    }
}
