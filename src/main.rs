use log::{error, info};
use simplelog::*;
use std::env;
use std::fs::{self, File};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;

mod commands;
mod configuration;
mod serialcom;
mod structs;
mod wscom;
mod parser;

use crate::configuration::get_configuration;
use crate::structs::{Config, MessageType, MessageWS};
use crate::wscom::accept_connection;

#[tokio::main]
async fn main() {
    setup_logs().expect("Failed to setup logs");

    info!("Starting xcontroller...");

    // Set config from start params
    let args: Vec<String> = env::args().collect();
    let configuration = get_configuration(args);

    let addr = format!("0.0.0.0:{}", configuration.ws_port);

    info!("Listening on {}", addr);
    info!("Running with config: {:?}", configuration);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("TCP fail to open connection");

    // Start serial connection and listen for incoming connections
    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("Connected peers should have an address");

        let cloned_configuration = configuration.clone();

        // Spawn a new thread for each connection for async handling
        tokio::spawn(async move {
            if let Err(e) = accept_connection(peer, stream, cloned_configuration).await {
                error!("Connection error from {}: {}", peer, e);
            }
        });
    }
}

fn setup_logs() -> Result<(), std::io::Error> {
    // setup logs folder
    if !std::path::Path::new("./logs").exists() {
        match fs::create_dir("./logs") {
            Ok(()) => {
                println!("Setup logs folder");
            }
            Err(err) => {
                println!("Error setting up logs folder");
                return Err(err);
            }
        }
    }

    // Set timestamp
    let current_time = SystemTime::now();
    let duration_since_epoch = current_time.duration_since(UNIX_EPOCH).unwrap();
    let timestamp = duration_since_epoch.as_secs().to_string();

    let log_file_path = format!("./logs/log_{}.log", timestamp);

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            ConfigBuilder::new().build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            ConfigBuilder::new().build(),
            File::create(log_file_path).unwrap(),
        ),
    ])
    .unwrap();

    Ok(())
}
