use futures::{stream::StreamExt, SinkExt};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_async, 
    tungstenite::{Error, Result},
};
use tungstenite::Message;
use log::{ info, debug, error };

use crate::commands::g_command;
use serde_json;
use crate::serialcom::create_serialcom;

use crate::Config;
use crate::MessageWS;
use crate::MessageType;


pub struct MessageSender<'a> {
    message_type:&'a str,
    message: &'a str,
    raw_message: &'a str,
    timestamp: u64,
}
// Accept incoming connection from client
pub async fn accept_connection(peer: SocketAddr, stream: TcpStream, configuration: Config<'_>) {
    match handle_connection(peer, stream, configuration).await {
        Ok(_) => {
        }
        Err(e) => match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        },
    }
}

// Get stream message and validate it and send back command
async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    configuration: Config<'_>
) -> Result<(), Error> {
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept incoming connection");

    // Socket addresses can be validated to insure only valide peers can connect and send commands
    info!("New client | {}", peer);
    let (mut ws_write, mut ws_read) = ws_stream.split();

    let send_message_to_client = |message_to_send: MessageSender| async move {

        let json_str = serde_json::to_string(message_to_send).expect("Failed to serialize myvar into JSON");
                    let resp_message = Message::Text(json_str.into());

        if let Err(e) = ws_write.send(resp_message).await {
            error!("{}", e);
        }
    };
    // Send message to port and return response to WS client
    let send_command = |command: &str| async move {
        match create_serialcom(
            command,
            configuration.serial_port.to_string(),
            configuration.baud_rate,
            configuration.test_mode) {
                Ok(response) => {
                    debug!("{}", response);
                    //return response to WS clients
                    send_message_to_client(response)
                }
                Err(e) => {
                    error!("{}", e)
                }
            }
    };

    
    // Loop over received messages
    while let Some(msg) = ws_read.next().await {
        let msg = msg?;

        // can also check for binary values
        if msg.is_text() && !msg.is_empty() {
            // The data is directly going to the serial_com.
            // Parse and validate the commands.
            let data = msg.to_text()?;

            let mut cmd: &str;
            let mut command: &str;

            match serde_json::from_str::<MessageWS>(&data) {
                Ok(message) => {
                    match message.message_type {
                        MessageType::GCommand => {
                            debug!("Config: {}", message.message);
                            Ok((cmd, command)) = g_command(message.message);
                            // Handle serialcom response
                        }
                        MessageType::SerialConfig => {
                            debug!("SerialConfig: {}", message.message);
                            // Test GCode for printer info
                            cmd = "M115";
                            //Expects message.message to be ex: /dev/USBtty01;119200
                        }
                        MessageType::Unsafe => todo!(),
                        MessageType::MessageSender => todo!(),
                    }
                    send_command(cmd);
                    
                }
                Err(_) => {
                    error!("Failed to parse message from JSON");
                }
            }
        } else {
            error!("No valid text received");
        }
    }


    // Can add a timeout to regularly send status updates

    error!("ConnectionClosed for {}", peer);
    Err(Error::ConnectionClosed)
}

// Send message to connected clients
async fn broadcast_message(message: String) {
    debug!("BROADCAST | {}", message)


}