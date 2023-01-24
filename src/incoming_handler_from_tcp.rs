use std::error::Error;

use crate::{incoming_packet::Incoming, Context};

pub async fn handle_incoming_from_tcp(
    incoming: Incoming,
    _: String,
    _: &mut Context,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    match incoming {
        _ => Ok(()),
    }
}
