use log::{error, info, warn};
use simplelog::*;
use std::env;
use std::fs::{self, File};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

mod commands;
mod configuration;
mod parser;
mod serialcom;
mod structs;
mod wscom;

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

    let addr = format!("{}:{}", configuration.bind_addr, configuration.ws_port);

    info!("Listening on {}", addr);
    // Don't dump auth_token into logs — Debug-print a sanitized view.
    info!(
        "Running with config: serial_port={} baud_rate={} ws_port={} bind_addr={} max_clients={} test_mode={} auth={}",
        configuration.serial_port,
        configuration.baud_rate,
        configuration.ws_port,
        configuration.bind_addr,
        configuration.max_clients,
        configuration.test_mode,
        if configuration.auth_token.is_some() { "enabled" } else { "disabled" },
    );

    let listener = TcpListener::bind(&addr)
        .await
        .expect("TCP fail to open connection");

    let connection_limit = Arc::new(Semaphore::new(configuration.max_clients));

    // Start serial connection and listen for incoming connections
    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("Connected peers should have an address");

        // Cap concurrent clients. try_acquire avoids backing the accept loop
        // up if a connection burst arrives — we'd rather refuse fast.
        let permit = match connection_limit.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                warn!(
                    "Refusing connection from {}: max_clients={} reached",
                    peer, configuration.max_clients
                );
                drop(stream);
                continue;
            }
        };

        let cloned_configuration = configuration.clone();

        tokio::spawn(async move {
            let _permit = permit;
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
