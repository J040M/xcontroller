use futures::{stream::StreamExt, SinkExt};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
};
use tungstenite::Message;

use crate::commands::g_command;
use crate::serialcom::create_serialcom;

use crate::Config;
use crate::MessageType;
use crate::MessageWS;

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageSender<'a> {
    pub message_type: &'a str,
    pub message: &'a str,
    pub raw_message: String,
    pub timestamp: u64,
}

// Accept incoming connection from client
pub async fn accept_connection(peer: SocketAddr, stream: TcpStream, configuration: Config) {
    match handle_connection(peer, stream, configuration).await {
        Ok(_) => {}
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
    configuration: Config,
) -> Result<(), Error> {
    let ws_stream = accept_async(stream)
        .await
        .expect("Failed to accept incoming connection");

    // Socket addresses can be validated to insure only valide peers can connect and send commands
    info!("New client | {}", peer);
    let (mut ws_write, mut ws_read) = ws_stream.split();

    //Broadcast message to clients
    // Broadcast message to clients
    async fn send_message_back(
        message: MessageSender<'_>,
        ws_write: &mut futures::prelude::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<TcpStream>,
            Message,
        >,
    ) -> Result<()> {
        let json_str =
            serde_json::to_string(&message).expect("Failed to serialize myvar into JSON");
        let resp_message = Message::Text(json_str);

        if let Err(e) = ws_write.send(resp_message).await {
            // Handle the error here
            error!("{:?}", e)
        }

        Ok(())
    }

    // Loop over received messages
    while let Some(msg) = ws_read.next().await {
        let msg = msg?;

        // can also check for binary values
        if msg.is_text() && !msg.is_empty() {
            // The data is directly going to the serial_com.
            // Parse and validate the commands.
            let data = msg.to_text()?;

            match serde_json::from_str::<MessageWS>(data) {
                Ok(message) => {
                    info!("Message received: {}", message.message);

                    // Starting timestamp
                    let now = SystemTime::now();

                    match message.message_type {
                        MessageType::GCommand => {
                            debug!("Config: {}", message.message);
                            let result = g_command(message.message);
                            match result {
                                Ok(cmd) => {
                                    match create_serialcom(
                                        cmd,
                                        configuration.serial_port.to_string(),
                                        configuration.baud_rate,
                                    ) {
                                        Ok(response) => {
                                            debug!("{:?}", response);

                                            // Set timestamp
                                            let since_epoch = now
                                                .duration_since(UNIX_EPOCH)
                                                .expect("Time went backwards");
                                            let timestamp = since_epoch.as_secs();

                                            // Define response message
                                            let message_sender = MessageSender {
                                                message_type: "MessageSender",
                                                message: &response.clone(),
                                                raw_message: response,
                                                timestamp,
                                            };

                                            //return response to WS clients
                                            send_message_back(message_sender, &mut ws_write)
                                                .await?;
                                        }
                                        Err(e) => {
                                            error!("{:?}", e)
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("{:?}", e)
                                }
                            }
                        }
                        MessageType::SerialConfig => {
                            debug!("SerialConfig: {}", message.message);
                            // Test GCode for printer info
                            // cmd = "M115";
                            //Expects message.message to be ex: /dev/USBtty01;119200
                        }
                        MessageType::Unsafe => {
                            let cmd = message.message;
                            match create_serialcom(
                                cmd,
                                configuration.serial_port.to_string(),
                                configuration.baud_rate,
                            ) {
                                Ok(response) => {
                                    debug!("{:?}", response);

                                    // Get timestamp
                                    let since_epoch = now
                                        .duration_since(UNIX_EPOCH)
                                        .expect("Time went backwards");
                                    let timestamp = since_epoch.as_secs();

                                    let message_sender = MessageSender {
                                        message_type: "MessageSender",
                                        message: &response.clone(),
                                        raw_message: response,
                                        timestamp,
                                    };

                                    send_message_back(message_sender, &mut ws_write).await?;
                                }
                                Err(e) => {
                                    error!("{:?}", e);

                                    let since_epoch = now
                                        .duration_since(UNIX_EPOCH)
                                        .expect("Time went backwards");
                                    let timestamp = since_epoch.as_secs();

                                    // Define response message
                                    let message_sender = MessageSender {
                                        message_type: "MessageSenderError",
                                        message: "Error executing command",
                                        raw_message: "Error executing command".to_string(),
                                        timestamp,
                                    };

                                    send_message_back(message_sender, &mut ws_write).await?;
                                }
                            }
                        }
                    }
                }
                Err(_) => todo!(),
            }
        }
    }

    error!("ConnectionClosed for {}", peer);
    Err(Error::ConnectionClosed)
}
