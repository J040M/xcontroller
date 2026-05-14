use log::{debug, info};
use serialport::{ClearBuffer, SerialPort};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

static TIMEOUT: u64 = 1;

pub struct SerialConnection {
    port: Box<dyn SerialPort>,
}

impl SerialConnection {
    pub fn open(serial_port: &str, baud_rate: u32) -> io::Result<Self> {
        let port = serialport::new(serial_port, baud_rate)
            .timeout(Duration::from_secs(TIMEOUT))
            .open()
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(Self { port })
    }

    pub fn send_command(&mut self, cmd: &str) -> io::Result<String> {
        // Drain unsolicited bytes (auto-report temp/pos, SD status, busy
        // pings) so they don't get read as the reply to *this* command.
        let _ = self.port.clear(ClearBuffer::Input);

        let response = round_trip(&mut self.port, cmd)?;
        info!("{}", response);
        Ok(response)
    }

    /// Run `f` with the port's read timeout temporarily set to `dur`,
    /// restoring the original timeout on exit. The binary-transfer
    /// adapter expects short-timeout `TimedOut` to drive its retransmit
    /// `tick()`; the default 1 s timeout would starve that loop.
    ///
    /// Generic over the closure's return type so callers can pass back a
    /// domain-specific `Result` (e.g. `Result<UploadStats, UploadError>`)
    /// without an extra layer of nesting. Failures to flip the timeout
    /// are logged but not surfaced — degrading to the existing timeout
    /// is preferable to aborting the upload.
    pub fn with_short_read_timeout<F, R>(&mut self, dur: Duration, f: F) -> R
    where
        F: FnOnce(&mut dyn SerialPort) -> R,
    {
        let previous = self.port.timeout();
        if let Err(e) = self.port.set_timeout(dur) {
            debug!("set_timeout({:?}) failed: {}", dur, e);
        }
        // Drain unsolicited bytes before switching modes — leftover ASCII
        // auto-reports would corrupt the binary handshake.
        let _ = self.port.clear(ClearBuffer::Input);
        let result = f(&mut *self.port);
        if let Err(e) = self.port.set_timeout(previous) {
            debug!("restore set_timeout({:?}) failed: {}", previous, e);
        }
        result
    }
}

fn round_trip<P: Read + Write>(port: &mut P, cmd: &str) -> io::Result<String> {
    let framed = format!("{}\r\n", cmd);
    write_to_port(port, framed.as_bytes())?;
    read_from_port(port)
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

    /// Mock with separate read and write buffers — Cursor alone can't model a
    /// duplex port because writes and reads share its position cursor.
    struct MockPort {
        write_buf: Vec<u8>,
        read_buf: Cursor<Vec<u8>>,
    }

    impl Read for MockPort {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.read_buf.read(buf)
        }
    }

    impl Write for MockPort {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_buf.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn round_trip_frames_command_with_crlf() {
        let mut mock = MockPort {
            write_buf: Vec::new(),
            read_buf: Cursor::new(b"ok\n".to_vec()),
        };
        let response = round_trip(&mut mock, "M105").unwrap();
        assert_eq!(mock.write_buf, b"M105\r\n");
        assert_eq!(response, "ok\n");
    }

    #[test]
    fn round_trip_returns_multiline_reply_verbatim() {
        let reply = "Begin file list\nfile1.GCO\nEnd file list\nok\n";
        let mut mock = MockPort {
            write_buf: Vec::new(),
            read_buf: Cursor::new(reply.as_bytes().to_vec()),
        };
        let response = round_trip(&mut mock, "M20").unwrap();
        assert_eq!(mock.write_buf, b"M20\r\n");
        assert_eq!(response, reply);
    }

    #[test]
    fn round_trip_propagates_write_error() {
        struct ErrorPort;
        impl Read for ErrorPort {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Ok(0)
            }
        }
        impl Write for ErrorPort {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "unplugged"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut port = ErrorPort;
        let err = round_trip(&mut port, "M105").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
    }
}
