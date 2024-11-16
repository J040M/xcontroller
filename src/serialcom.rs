use log::{debug, error, info};
use std::io::{self, Read, Write};
use std::time::Duration;

static TIMEOUT: u64 = 1;

// TODO: This  creates a serial connection for every command
// The connection can be kept temporarily open to avoid this
pub fn create_serialcom(cmd: &str, serial_port: String, baud_rate: u32) -> Result<String, ()> {
    //Validate the Gcode in &command before converting it
    let command = format!("{}\r\n", cmd);
    let c_inbytes = command.into_bytes();

    match serialport::new(&serial_port, baud_rate)
        .timeout(Duration::from_secs(TIMEOUT))
        .open()
    {
        Ok(mut port) => {
            if let Err(e) = write_to_port(&mut port, &c_inbytes) {
                //Send this message back to WS for broadcast
                error!("Failed to write_to_port | {}", e);
                return Err(());
            }

            if let Ok(response) = read_from_port(&mut port) {
                // Parse message and send this response back to WS for broadcast
                info!("{}", response);
                Ok(response)
            } else {
                //Send this message back to WS for broadcast
                error!("Failed to read comport. Error");
                Err(())
            }
        }
        Err(e) => {
            error!("Failed to open \"{}\". Error: {}", serial_port, e);
            Err(())
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

                        // TODO: This will lead to wrong return message!
                        // "ok" can also be in the beginning or middle and not only at the end of the message
                        if res.contains("ok") {
                            return Ok(response_buffer);
                        }
                    }
                    Err(err) => {
                        debug!("Invalid UTF-8 sequence: {}", err)
                    }
                }
                timeout = 0;
            }
            // Check for timeout and stop the communication
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                timeout += 1;
                if timeout > 10 {
                    error!("Timeout on COM exceeded");
                    return Err(io::Error::new(e.kind(), "Timeout limit exceeded"));
                }
            }
            Err(err) => return Err(err),
        }
    }
}

fn write_to_port<T: Write>(port: &mut T, command: &[u8]) -> io::Result<()> {
    match port.write_all(command) {
        Ok(_) => {
            debug!("Successfully sent command");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_from_port_ok() {
        let data = b"ok\n";
        let mut cursor = Cursor::new(data);
        let result = read_from_port(&mut cursor).unwrap();
        assert_eq!(result, "ok\n");
    }

    // TODO: For this to work a end of message delimiter is needed
    
    // #[test]
    // fn test_read_from_port_partial_ok() {
    //     let data = b"data and more data";
    //     let mut cursor = Cursor::new(data);
    //     let result = read_from_port(&mut cursor).unwrap();
    //     assert_eq!(result, "data and more data");
    // }

    #[test]
    fn test_read_from_port_timeout() {
        struct TimeoutReader;
        impl Read for TimeoutReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::TimedOut, "timeout"))
            }
        }

        let mut reader = TimeoutReader;
        let result = read_from_port(&mut reader);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn test_write_to_port_success() {
        let mut buffer = Vec::new();
        let command = b"test command";
        let _result = write_to_port(&mut buffer, command).unwrap();
        assert_eq!(buffer, command);
    }

    #[test]
    fn test_write_to_port_error() {
        struct ErrorWriter;
        impl Write for ErrorWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Other, "write error"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut writer = ErrorWriter;
        let command = b"test command";
        let result = write_to_port(&mut writer, command);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Other);
    }
}
