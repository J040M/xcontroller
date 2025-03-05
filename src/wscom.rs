use futures::{stream::StreamExt, SinkExt};
use log::{debug, error, info};
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

use crate::parser::{m105, m114, m115, m119, m20, m33};
use crate::structs::MessageSender;
use crate::Config;
use crate::MessageType;
use crate::MessageWS;

/**
 * Accept incoming connection from client
 * @param peer: SocketAddr, peer address
 * @param stream: TcpStream, stream from client
 * @param configuration: Config, configuration for the server
 * @return Result<(), Error>, return Ok(())
 * @throws Error
 */
pub async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    configuration: Config,
) -> Result<(), Error> {
    match handle_connection(peer, stream, configuration).await {
        Ok(_) => Ok(()),
        Err(e) => match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => Ok(()),
            err => {
                error!("Error processing connection: {}", err);
                Err(err)
            }
        },
    }
}

/**
 * Get stream message and validate it and send back command
 * @param peer: SocketAddr, peer address
 * @param stream: TcpStream, stream from client
 * @param configuration: Config, configuration for the server
 * @return Result<(), Error>, return Ok(())
 * @throws Error
 */
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

    // Broadcast response message to clients
    async fn send_message_back(
        message: MessageSender,
        ws_write: &mut futures::prelude::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<TcpStream>,
            Message,
        >,
    ) -> Result<()> {
        let json_str =
            serde_json::to_string(&message).expect("Failed to serialize message into JSON");
        let resp_message = Message::Text(json_str);

        if let Err(e) = ws_write.send(resp_message).await {
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
                                            let mut message_sender = MessageSender {
                                                message_type: cmd.to_string(),
                                                message: "".to_string(),
                                                raw_message: response.clone(),
                                                timestamp,
                                            };

                                            if &response != "ok" {
                                                message_sender.message = match cmd.trim() {
                                                    "M20" => {
                                                        let response = m20(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M33" => {
                                                        let response = m33(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    },
                                                    "M105" => {
                                                        let response = m105(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M114" => {
                                                        let response = m114(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M115" => {
                                                        let response = m115(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M119" => {
                                                        let response = m119(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    _ => response.to_string(),
                                                };
                                            }
                                            // Return response to WS clients
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
                        MessageType::Terminal => {
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
                                        message_type: "terminal".to_string(),
                                        message: response.to_string().clone(),
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
                                        message_type: "MessageSenderError".to_string(),
                                        message: "Error executing command".to_string(),
                                        raw_message: "Error executing command".to_string(),
                                        timestamp,
                                    };

                                    send_message_back(message_sender, &mut ws_write).await?;
                                }
                            }
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
                                        message_type: "Unsafe".to_string(),
                                        message: response.to_string().clone(),
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
                                        message_type: "MessageSenderError".to_string(),
                                        message: "Error executing command".to_string(),
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

    info!("Connection lost for {}", peer);
    Err(Error::ConnectionClosed)
}
