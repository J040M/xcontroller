use log::info;
use std::env;
use tokio::net::TcpListener;
use log::error;

mod configuration;
mod commands;
mod serialcom;
mod structs;
mod wscom;

use crate::structs::{Config, MessageType, MessageWS};
use crate::configuration::get_configuration;
use crate::wscom::accept_connection;

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Starting xcontroller...");

    // Set config from start params
    let args: Vec<String> = env::args().collect();
    let configuration = get_configuration(args);

    let addr = format!("0.0.0.0:{}", configuration.ws_port);
    info!("Listening on {}", addr);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("TCP fail to open connection");

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("Connected peers should have an address");

        let cloned_configuration = configuration.clone();
        info!("Running with config: {:?}", cloned_configuration);

        tokio::spawn(async move {
            if let Err(e) = accept_connection(peer, stream, cloned_configuration).await {
                error!("Connection error from {}: {}", peer, e);
            }
        });
    }
}
