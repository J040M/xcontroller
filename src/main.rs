use std::time::Duration;
use std::io::{self, Write, Read};
use serialport::SerialPort;

static SERIAL_PORT: &str = "/dev/ttyUSB0";
static BAUD_RATE: u32 = 115200;
static TIMEOUT: u64 = 1;

fn read_from_port<T: Read>(port: &mut T) -> io::Result<()> {
    let mut serial_buffer: Vec<u8> = vec![0; 256];
    let mut timeout = 0;
    loop {
        match port.read(serial_buffer.as_mut_slice()) {
            Ok(t) => {
                match std::str::from_utf8(&serial_buffer[0..t]) {
                    Ok(res) => println!("Received: {}", res),
                    Err(err) => println!("Invalid UTF-8 sequence: {}", err),
                }
                timeout = 0;
            },
            // Check for timeout and stop the program
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                timeout += 1;
                if timeout > 10 {
                    return Err(io::Error::new(e.kind().clone(), "Timeout limit exceeded"));
                }
            },
            Err(err) => return Err(err),
        }
    }
}

fn write_to_port<T: Write>(port: &mut T, command: &[u8]) -> io::Result<()> {
    match port.write_all(command) {
        Ok(_) => {
            println!("Successfully sent command");
            Ok(())
        },
        Err(e) => Err(e),
    }
}

fn main() {
    match serialport::new(SERIAL_PORT, BAUD_RATE)
        .timeout(Duration::from_secs(TIMEOUT)).open() {
            Ok(mut port) => {
                if let Err(e) = write_to_port(&mut port, b"G0 Z100\r\n") {
                    eprintln!("Failed to send command. Error: {}", e);
                    return;
                }
                if let Err(e) = read_from_port(&mut port) {
                    eprintln!("Failed to read port. Error: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Failed to open \"{}\". Error: {}", SERIAL_PORT, e);
            },
    }
}