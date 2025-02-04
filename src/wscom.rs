use futures::{stream::StreamExt, SinkExt};
use log::{error, info};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
};
use futures_util;
use tokio_tungstenite::WebSocketStream;

// use crate::commands::g_command;
use crate::serialcom::SerialConnection;

// use crate::parser::{m105, m114, m115, m119, m20, m33};
use crate::structs::MessageSender;
use crate::Config;
// use crate::MessageType;
use crate::MessageWS;

// #[derive(Serialize, Deserialize)]
async fn send_message_back(
    message: MessageSender,
    ws_write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, tungstenite::Message>
) -> Result<(), tungstenite::Error> {
    let message_json = serde_json::to_string(&message).unwrap();
    ws_write.send(tungstenite::Message::Text(message_json)).await?;
    Ok(())
}

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
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New client | {}", peer);
    
    // Create serial connection once
    let mut serial = match SerialConnection::new(
        &configuration.serial_port,
        configuration.baud_rate
    ) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to open serial port: {}", e);
            return Err(e);
        }
    };

    let (mut ws_write, mut ws_read) = ws_stream.split();

    while let Some(msg) = ws_read.next().await {
        let msg = msg?;

        if msg.is_text() && !msg.is_empty() {
            let data = msg.to_text()?;
            
            match serde_json::from_str::<MessageWS>(data) {
                Ok(message) => {
                    // Use the existing serial connection
                    match serial.send_command(&message.message) {
                        Ok(response) => {
                            // Handle response...
                            let message_sender = MessageSender {
                                message_type: "MessageSender".to_string(), 
                                message: response.clone(),
                                raw_message: response,
                                timestamp: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            };
                            send_message_back(message_sender, &mut ws_write).await?;
                        }
                        Err(e) => {
                            error!("Serial command failed: {:?}", e);
                            // Handle error...
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse message: {:?}", e);
                }
            }
        }
    }

    info!("Connection lost for {}", peer);
    Err(Error::ConnectionClosed)
}
