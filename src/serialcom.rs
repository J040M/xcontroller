use std::io::{self, Read, Write};
use std::time::Duration;
use std::u32;

static TIMEOUT: u64 = 1;
// This  creates a serial connection for every command
// The connection can be kept temporarily open to avoid this
pub fn create_serialcom(cmd: &str, serial_port: String, baud_rate: u32, test_mode: bool) {
    //return without comm with printer
    if test_mode {
        return;
    }

    //Validate the Gcode in &command before converting it
    let command = format!("{}\r\n", cmd);
    let c_inbytes = command.into_bytes();

    // Spawning an async task here could avoid freezing the program
    match serialport::new(&serial_port, baud_rate)
        .timeout(Duration::from_secs(TIMEOUT))
        .open()
    {
        Ok(mut port) => {
            if let Err(e) = write_to_port(&mut port, &c_inbytes) {
                eprintln!("Failed to send command. Error: {}", e);
                return;
            }
            if let Ok(response) = read_from_port(&mut port) {
                println!("{}", response);
            } else {
                eprintln!("Failed to read port. Error");
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", serial_port, e);
        }
    }
}

fn read_from_port<T: Read>(port: &mut T) -> io::Result<String> {
    let mut serial_buffer: Vec<u8> = vec![0; 256];
    let mut timeout = 0;
    let mut response_buffer = String::new();

    loop {
        match port.read(serial_buffer.as_mut_slice()) {
            // Incoming buffer should be parsed and stored to sent to FE
            Ok(t) => {
                match std::str::from_utf8(&serial_buffer[0..t]) {
                    Ok(res) => {
                        response_buffer.push_str(res);

                        // This can lead to wrong return message
                        // "ok" can also be in the beginning or middle and not only at the end of the message
                        if res.contains("ok") {
                            return Ok(response_buffer);
                        }
                    }
                    Err(err) => println!("Invalid UTF-8 sequence: {}", err),
                }
                timeout = 0;
            }
            // Check for timeout and stop the communication
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                timeout += 1;
                if timeout > 10 {
                    return Err(io::Error::new(e.kind().clone(), "Timeout limit exceeded"));
                }
            }
            Err(err) => return Err(err),
        }
    }
}

fn write_to_port<T: Write>(port: &mut T, command: &[u8]) -> io::Result<()> {
    match port.write_all(command) {
        Ok(_) => {
            println!("Successfully sent command");
            Ok(())
        }
        Err(e) => Err(e),
    }
}
