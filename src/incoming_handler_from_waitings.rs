use std::error::Error;

use reqwest::{header::AUTHORIZATION, StatusCode};

use crate::{
    http_response::AuthResponse, incoming_packet::Incoming, job::Job, outgoing_packet::Outgoing,
    schedule::Schedule, url::endpoint, Context,
};

pub async fn handle_incoming_from_waitings(
    incoming: Incoming,
    i: usize,
    context: &mut Context,
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

            let stream = context.waitings.remove(i);

            context.tcp_streams.insert(response.id.clone(), stream);

            let id = response.id;

            let packet = Outgoing::HelloFromTcp { id: id.clone() };

            let schedule = Schedule::instant(Job::SendToTcp(packet, id));

            context.schedule_queue.push(schedule);

            Ok(())
        }
        _ => Ok(()),
    }
}
