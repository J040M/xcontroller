use futures::stream::StreamExt;
use std::time::Duration;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
};
use serde::{Serialize, Deserialize};

mod serialcom;
mod commands;

// Defined structure for messages between the server and client
#[derive(Debug, Serialize, Deserialize)]
enum MessageType {
    Operation,
    Movement,
    Tools,
    Config,
    Information,
    Special,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message<'a> {
    message_type: MessageType,
    message: &'a str,
}

// These should be used as defaults
// But different values should be accepted from incoming
// connections to insure that other configs are accepted
static SERIAL_PORT: &str = "/dev/ttyUSB0";
static BAUD_RATE: u32 = 115200;
static TIMEOUT: u64 = 1;

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

    //Socket addresses can be validated to insure only valide peers can connect and send commands
    println!("New WebSocket connection: {}", peer);

    // Loop over received messages
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        // can also check for binary values
        if msg.is_text() {
            // The data is directly going to the serial_com.
            // Parse and validate the commands.
            let data = msg.to_text()?;
            create_serialcom(data);

            // Defining types and parsing that removes the possibility
            // of having direct commands from FE. +1!
            match serde_json::from_str::<Message>(&data) {
                Ok(message) => {
                    match message.message_type {
                        MessageType::Config => {
                            println!("Config: {}", message.message);
                            let result = commands::config(message.message)?;
                            create_serialcom(&result)
                        },
                        MessageType::Movement => {
                            println!("Movement: {}", message.message);
                            let result = commands::movement(message.message)?;
                            create_serialcom(&result)
                        },
                        MessageType::Operation => {
                            println!("Operation: {}", message.message);
                            let result = commands::operation(message.message)?;
                            create_serialcom(&result)
                        },
                        MessageType::Tools => {
                            println!("Tools: {}", message.message);
                            let result = commands::tools(message.message)?;
                            create_serialcom(&result)
                        },
                        MessageType::Information => {
                            println!("Information: {}", message.message);
                            let result = commands::information(message.message)?;
                            create_serialcom(&result)
                        },
                        MessageType::Special => {
                            println!("Special: {}", message.message);
                            let result = commands::special(message.message)?;
                            create_serialcom(&result)
                        },
                    }
                },
                Err(_) => eprintln!("Failed to parse Message from JSON"),
            }
        } else {
            eprintln!("No valid text received")
        }
    }

    Ok(())
}

// This  creates a serial connection for every command
// The connection can be kept temporarily open to avoid this
fn create_serialcom(cmd: &str) {
    //Validate the Gcode in &command before converting it
    let command = format!("{}\r\n", cmd);
    let c_inbytes =  command.into_bytes();
    
    // Spawning an async task here could avoid freezing the program
    match serialport::new(SERIAL_PORT, BAUD_RATE)
        .timeout(Duration::from_secs(TIMEOUT)).open() {
            Ok(mut port) => {
                if let Err(e) = serialcom::write_to_port(&mut port, &c_inbytes) {
                    eprintln!("Failed to send command. Error: {}", e);
                    return;
                }
                if let Err(e) = serialcom::read_from_port(&mut port) {
                    eprintln!("Failed to read port. Error: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Failed to open \"{}\". Error: {}", SERIAL_PORT, e);
            },
    }
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        println!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream));
    }
}