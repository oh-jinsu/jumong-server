use std::error::Error;

use jumong_server::{env, job_handler, selector, Context};
use tokio::net::{TcpListener, UdpSocket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env::init();

    let tcp_listener = TcpListener::bind("0.0.0.0:3000").await?;

    let udp_socket = UdpSocket::bind("0.0.0.0:3000").await?;

    let mut context = Context::new(tcp_listener, udp_socket);

    loop {
        let job = selector::select_job(&mut context).await;

        if let Err(e) = job_handler::handle(job, &mut context).await {
            eprintln!("job failed for {e}");
        }
    }
}
