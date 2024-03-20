use futures::stream::StreamExt;
use std::time::Duration;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
};

mod serialcom;

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
            err => print!("Error processing connection: {}", err),
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
            let data = msg.to_text()?;
            // The data is directly going to the serial_com.
            // Parse and validate the commands.
            create_serialcom(data)
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
    let command = format!("G0 {}\r\n", cmd);
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