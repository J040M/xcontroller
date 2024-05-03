use futures::stream::StreamExt;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
};

use crate::commands::g_command;
use crate::serialcom::create_serialcom;

use crate::Config;
use crate::Message;
use crate::MessageType;

// Accept incoming connection from client
pub async fn accept_connection(peer: SocketAddr, stream: TcpStream, configuration: Config<'_>) {
    match handle_connection(peer, stream).await {
        Ok(command) => {
            println!("Returned message: {}", command);

            create_serialcom(
                &command,
                configuration.serial_port.to_string(),
                configuration.baud_rate,
                configuration.test_mode,
            );
        }
        Err(e) => match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => eprintln!("Error processing connection: {}", err),
        },
    }
}

// Get stream message and validate it and send back command
pub async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
) -> Result<String, Error> {
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

    // Socket addresses can be validated to insure only valide peers can connect and send commands
    println!("New client connection: {}", peer);

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
                            command_result = g_command(message.message)?;
                        }
                        MessageType::SerialConfig => {
                            println!("SerialConfig: {}", message.message);
                            // Test GCode for printer info
                            command_result = "M115";
                            //Expects message.message to be ex: /dev/USBtty01;119200
                        }
                        MessageType::Unsafe => todo!(),
                    }
                    return Ok(command_result.to_string());
                }
                Err(_) => {
                    eprintln!("Failed to parse message from JSON");
                }
            }
        } else {
            eprintln!("No valid text received")
        }
    }

    Err(Error::ConnectionClosed)
}
