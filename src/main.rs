use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::net::TcpListener;

mod commands;
mod serialcom;
mod wscom;

use crate::wscom::accept_connection;

// Defined structure for messages between the server and client
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    GCommand,
    SerialConfig,
    Unsafe,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageWS<'a> {
    message_type: MessageType,
    message: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrinterInfo<'a> {
    firmware_name: &'a str,
    firmware_version: &'a str,
    serial_xon_xoff: u8,
    eeprom: u8,
    volumetric: u8,
    autoreport_pos: u8,
    autoreport_temp: u8,
    progress: u8,
    print_job: u8,
    autolevel: u8,
    runout: u8,
    z_probe: u8,
    leveling_data: u8,
    build_percent: u8,
    software_power: u8,
    toggle_lights: u8,
    case_light_brightness: u8,
    emergency_parser: u8,
    most_action_commands: u8,
    prompt_support: u8,
    sdcard: u8,
    repeat: u8,
    sd_write: u8,
    auto_report_sd_status: u8,
    long_filename: u8,
    thermal_protection: u8,
    motion_modes: u8,
    arcs: u8,
    babystepping: u8,
    chamber_temperature: u8,
    cooler_temperature: u8,
    meatpack: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AxePositions {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Temperatures {
    bed: u8,
    bed_set: u8,
    e0: u8,
    e0_set: u8,
    e1: u8,
    e1_set: u8,
    e2: u8,
    e2_set: u8,
    e3: u8,
    e3_set: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    test_mode: bool,
    serial_port: String,
    baud_rate: u32,
    ws_port: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Starting application...");

    let mut configuration = Config {
        test_mode: false,
        serial_port: "/dev/ttyUSB0".to_string(),
        baud_rate: 115200,
        ws_port: "9002".to_string(),
    };

    // Define configuration values
    let args: Vec<String> = env::args().collect();
    if args.len() > 4 {
        let ws_port = args[1].clone(); // Convert String to &str
        let sp_arg = args[2].clone();
        let br_arg = args[3].clone();
        let test_arg = &args[4].clone();

        configuration = Config {
            test_mode: match test_arg.to_lowercase().as_str() {
                "true" => true,
                _ => false,
            },
            baud_rate: match br_arg.parse::<u32>() {
                Ok(br) => br,
                Err(_) => {
                    warn!("Failed to parse baudrate for the configuration.");
                    warn!("Using default baudrate 115200.");
                    115200 // Default baud rate
                }
            },
            serial_port: sp_arg,
            ws_port: ws_port,
        };


        println!("{:?}", configuration)
    }
    // Define 127 to accept only local connection.
    // let addr = "127.0.0.1:9002";

    let addr = format!("0.0.0.0:{}", configuration.ws_port);
    info!("Listening on ws://{}", addr);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("TCP fail to open connection");

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("Connected streams should have a peer address");

        let cloned_configuration = configuration.clone();

        println!("Running with config: {:?}", cloned_configuration);
    
        tokio::spawn(async move {
            accept_connection(peer, stream, cloned_configuration).await;
        });
    }
}
