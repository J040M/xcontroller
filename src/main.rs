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

// Using TCP/Websockets to get incoming connection //
async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => eprintln!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
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

                            // if let Some((sp, br)) = message.message.split_once(";") {
                            //     match br.parse::<u32>() {
                            //         Ok(br) => {

                            //         }
                            //         Err(_) => println!("Failed to parse baudrate for the configuration."),
                            //     }
                            // }
                        }
                        MessageType::Unsafe => todo!(),
                    }
                    create_serialcom(
                        &command_result,
                        unsafe { SERIAL_PORT.to_string() },
                        unsafe { BAUD_RATE },
                        unsafe { TEST_MODE },
                    );
                }
                Err(_) => eprintln!("Failed to parse message from JSON"),
            }
        } else {
            eprintln!("No valid text received")
        }
    }
    Ok(())
}

static mut TEST_MODE: bool = false;
static mut SERIAL_PORT: &str = "dev/ttyUSB0";
static mut BAUD_RATE: u32 = 115200;

#[tokio::main]
async fn main() {
    static WS_PORT: &str = "9002";

    let mut ws_port = WS_PORT;
    // Define TEST_mode
    let args: Vec<String> = env::args().collect();
    if args.len() > 4 {
        println!("TEST_MODE {}", args[1]);

        let dev_arg = args[1].clone();
        let sp_arg = args[2].clone();
        let br_arg = args[3].clone();

        ws_port = &args[4];

        match dev_arg.to_lowercase().as_str() {
            "true" => unsafe { TEST_MODE = true },
            _ => {}
        }

        match br_arg.parse::<u32>() {
            Ok(br) => {
                unsafe { BAUD_RATE = br }
                unsafe { SERIAL_PORT = &sp_arg.clone() }
            },
            Err(_) => println!("Failed to parse baudrate for the configuration."),
        }


    }

    // Define 127 to accept only local connection.
    // let addr = "127.0.0.1:9002";
    let addr = format!("0.0.0.0:{}", ws_port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("TCP fail to open connection");

    println!("Listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("Connected streams should have a peer address");
        println!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream));
    }
}
