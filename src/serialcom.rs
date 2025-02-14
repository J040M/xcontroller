use log::{debug, error, info};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

static TIMEOUT: u64 = 1;

// TODO: This  creates a serial connection for every command
// The connection can be kept temporarily open to avoid this
pub fn create_serialcom(cmd: &str, serial_port: String, baud_rate: u32) -> Result<String, ()> {
    // Validate the Gcode in &command before converting it
    let command = format!("{}\r\n", cmd);
    let c_inbytes = command.into_bytes();

    match serialport::new(&serial_port, baud_rate)
        .timeout(Duration::from_secs(TIMEOUT))
        .open()
    {
        Ok(mut port) => {
            if let Err(e) = write_to_port(&mut port, &c_inbytes) {
                error!("Failed to write_to_port | {}", e);
                return Err(());
            }

            if let Ok(response) = read_from_port(&mut port) {
                info!("{}", response);
                Ok(response)
            } else {
                error!("Failed to read read_from_port");
                Err(())
            }
        }
        Err(e) => {
            error!("Failed to open COM \"{}\". Error: {}", serial_port, e);
            Err(())
        }
    }
}

fn read_from_port<T: Read>(port: &mut T) -> io::Result<String> {
    let mut serial_buffer = [0u8; 1024];
    let mut response_buffer = String::new();
    let timeout_duration = Duration::from_millis(100); // Adjust as needed
    let start_time = Instant::now();
    let mut last_char_time = Instant::now();

    loop {
        match port.read(serial_buffer.as_mut_slice()) {
            Ok(bytes_read) if bytes_read > 0 => {
                match std::str::from_utf8(&serial_buffer[0..bytes_read]) {
                    Ok(res) => {
                        response_buffer.push_str(res);
                        last_char_time = Instant::now();
                    }
                    Err(err) => {
                        debug!("Invalid UTF-8 sequence: {}", err);
                    }
                }
            }
            Ok(_) => {
                // No bytes read
                if last_char_time.elapsed() > timeout_duration {
                    // If we have data and no new chars for timeout_duration, message is complete
                    if !response_buffer.is_empty() {
                        return Ok(response_buffer);
                    }
                }
                if start_time.elapsed() > timeout_duration * 3 {
                    // Global timeout - either return what we have or NO RESPONSE
                    return if response_buffer.is_empty() {
                        Ok("NO RESPONSE".to_string())
                    } else {
                        Ok(response_buffer)
                    };
                }
            }
            Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                // Handle same as Ok(0)
                if last_char_time.elapsed() > timeout_duration && !response_buffer.is_empty() {
                    return Ok(response_buffer);
                }
                if start_time.elapsed() > timeout_duration * 3 {
                    return if response_buffer.is_empty() {
                        Ok("NO RESPONSE".to_string())
                    } else {
                        Ok(response_buffer)
                    };
                }
            }
            Err(e) => return Err(e),
        }
    }
}

fn write_to_port<T: Write>(port: &mut T, command: &[u8]) -> io::Result<()> {
    match port.write_all(command) {
        Ok(_) => {
            info!("{}", std::str::from_utf8(command).unwrap());
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn write_file_to_sd_card(
    file_content: &str,
    serial_port: String,
    baud_rate: u32,
) -> Result<String, ()> {
    // get the first line
    let first_line = file_content.lines().next().unwrap();
    // remove the first character that will always be ";"
    let first_line = &first_line[1..];
    //remove everything after the first "."
    let file_name = first_line.split('.').next().unwrap();

    let start_command = format!("M28 {}.gcode\r\n", file_name);
    let end_command = format!("M29\r\n");

    // Open serial port and store the connection
    let mut port = match serialport::new(&serial_port, baud_rate)
        .timeout(Duration::from_secs(TIMEOUT))
        .open()
    {
        Ok(port) => port,
        Err(e) => {
            error!("Failed to open COM port: {}", e);
            return Err(());
        }
    };

    // Start file transfer
    if let Err(e) = port.write_all(start_command.as_bytes()) {
        error!("Failed to write start command: {}", e);
        return Err(());
    }
    info!("Sent: {}", start_command.trim());

    // Write file content but this should go in chunks (per line) and line number (Nx) and checksum (*x)
    for (i, line) in file_content.lines().enumerate() {
        let line_number = format!("N{}", i + 1);
        let checksum = format!("*{}", xor_checksum(line));

        let mut command = "".to_string();

        //if line starts with ; the it should not have line number and checksum
        if line.starts_with(";") {
            // print!("{}", line);
            command = format!("{}", line);
        } else {
            // print!("Sent: {}", line);
            command = format!("{} {}{}", line_number, line, checksum);
        }

        if let Err(e) = port.write_all(command.as_bytes()) {
            error!("Failed to write checksum: {}", e);
            return Err(());
        }

        // Wait for "ok" response
        match read_from_port(&mut port) {
            Ok(response) => {
                info!("Received: {}", response.trim());
                if !response.trim().starts_with("ok") {
                    error!("Unexpected response: {}", response);
                    return Err(());
                }
            }
            Err(e) => {
                error!("Failed to read response: {}", e);
                return Err(());
            }
        }
    }

    // End file transfer
    if let Err(e) = port.write_all(end_command.as_bytes()) {
        error!("Failed to write end command: {}", e);
        return Err(());
    }

    info!("Sent: {}", end_command.trim());
    info!("File transfer completed");
    Ok("File transfer completed".to_string())
}

fn xor_checksum(cmd: &str) -> u8 {
    let mut checksum = 0u8;
    for c in cmd.chars() {
        checksum ^= c as u8; // XOR with the ASCII value of the character
    }
    checksum
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
    #[test]
    fn test_read_from_port_partial_ok() {
        let data = b"data and more data";
        let mut cursor = Cursor::new(data);
        let result = read_from_port(&mut cursor).unwrap();
        assert_eq!(result, "data and more data");
    }

    #[test]
    fn test_read_from_port_timeout() {
        struct TimeoutReader;
        impl Read for TimeoutReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::TimedOut, "timeout"))
            }
        }

        let mut reader = TimeoutReader;
        let result = read_from_port(&mut reader).unwrap();
        assert_eq!(result, "NO RESPONSE");
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
