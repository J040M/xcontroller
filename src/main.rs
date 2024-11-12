use log::{info, warn};
use std::env;
use tokio::net::TcpListener;

mod commands;
mod serialcom;
mod structs;
mod wscom;

use crate::structs::{Config, MessageType, MessageWS};
use crate::wscom::accept_connection;

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Starting xcontroller...");

    let mut configuration = Config {
        test_mode: false,
        serial_port: "/dev/ttyUSB0".to_string(),
        baud_rate: 115200,
        ws_port: "9002".to_string(),
    };

    // Set config from start params
    let args: Vec<String> = env::args().collect();
    if args.len() > 4 {
        let ws_port = args[1].clone(); // Convert String to &str
        let serial_port = args[2].clone();
        let baudrate = args[3].clone();
        let test_arg = args[4].clone();

        configuration = Config {
            test_mode: matches!(test_arg.to_lowercase().as_str(), "true"),
            baud_rate: match baudrate.parse::<u32>() {
                Ok(br) => br,
                Err(_) => {
                    warn!("Failed to parse baudrate. Using default baudrate 115200");
                    115200
                }
            },
            serial_port,
            ws_port,
        };
    }

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
            accept_connection(peer, stream, cloned_configuration).await;
        });
    }
}
