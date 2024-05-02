use futures::stream::StreamExt;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
};

use serde::{Deserialize, Serialize};
use std::env;

mod commands;
mod serialcom;

use crate::serialcom::create_serialcom;

// Defined structure for messages between the server and client
#[derive(Debug, Serialize, Deserialize)]
enum MessageType {
    GCommand,
    SerialConfig,
    Unsafe,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message<'a> {
    message_type: MessageType,
    message: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrinterInfo<'a> {
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
struct AxePositions {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Temperatures {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Config<'a> {
    test_mode:  bool,
    serial_port: &'a str,
    baud_rate: u32,
    ws_port: &'a str
}

// Using TCP/Websockets to get incoming connection //
async fn accept_connection(peer: SocketAddr, stream: TcpStream, configuration: Config<'_>) {
    if let Err(e) = handle_connection(peer, stream, configuration).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => eprintln!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream, configuration: Config<'_>) -> Result<String, Error> {
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

    // Socket addresses can be validated to insure only valide peers can connect and send commands
    println!("New WebSocket connection: {}", peer);

    // Loop over received messages
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;

        // can also check for binary values
        if msg.is_text() && !msg.is_empty() {
            // The data is directly going to the serial_com.
            // Parse and validate the commands.
            let data = msg.to_text()?;

            let command_result: &str;

            match serde_json::from_str::<Message>(&data) {
                Ok(message) => {
                    match message.message_type {
                        MessageType::GCommand => {
                            println!("Config: {}", message.message);
                            command_result = commands::g_command(message.message)?;
                        }
                        MessageType::SerialConfig => {
                            println!("SerialConfig: {}", message.message);
                            // Test GCode for printer info
                            command_result = "M115";
                            //Expects message.message to be ex: /dev/USBtty01;119200
                        }
                        MessageType::Unsafe => todo!(),
                    }
                    create_serialcom(
                        &command_result,
                        configuration.serial_port.to_string(),
                        configuration.baud_rate,
                        configuration.test_mode,
                    );
                }
                Err(_) => {
                    eprintln!("Failed to parse message from JSON");
                    return Err(Error::ConnectionClosed);
                },
            }

            return Ok(command_result.to_string())
        } else {
            eprintln!("No valid text received")
        }
    }
    
    Err(Error::ConnectionClosed)
}

#[tokio::main]
async fn main() {
    let configuration = Config { test_mode: false, serial_port: "dev/ttyUSB0",  baud_rate: 115200, ws_port: "9002" };

    // Define configuration values
    let args: Vec<String> = env::args().collect();
    if args.len() > 4 {
        println!("TEST_MODE {}", args[1]);
        
        let ws_port = &args[1].clone(); // Convert String to &str
        let sp_arg = &args[2].clone();
        let br_arg = args[3].clone();
        let test_arg = args[4].clone();
    
        Config {
            test_mode: match test_arg.to_lowercase().as_str() {
                "true" => true,
                _ => false
            },
            baud_rate: match br_arg.parse::<u32>() {
                Ok(br) => br,
                Err(_) => {
                    println!("Failed to parse baudrate for the configuration.");
                    115200 // Default baud rate
                }
            },
            serial_port: sp_arg,
            ws_port: &ws_port, // No longer borrowing, directly assigning
        };
    }
    // Define 127 to accept only local connection.
    // let addr = "127.0.0.1:9002";
    let addr = format!("0.0.0.0:{}", configuration.ws_port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("TCP fail to open connection");

    println!("Listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("Connected streams should have a peer address");
        
        tokio::spawn(async move {
            accept_connection(peer, stream, configuration).await;
        });
    }
}
