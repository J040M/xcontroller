use std::io::{self, Write, Read};

pub fn read_from_port<T: Read>(port: &mut T) -> io::Result<()> {
    let mut serial_buffer: Vec<u8> = vec![0; 256];
    let mut timeout = 0;
    loop {
        match port.read(serial_buffer.as_mut_slice()) {
            // Incoming buffer should be parsed and stored to sent to FE
            Ok(t) => {
                match std::str::from_utf8(&serial_buffer[0..t]) {
                    Ok(res) => println!("Received: {}", res),
                    Err(err) => println!("Invalid UTF-8 sequence: {}", err),
                }
                timeout = 0;
            },
            // Check for timeout and stop the communication
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

pub fn write_to_port<T: Write>(port: &mut T, command: &[u8]) -> io::Result<()> {
    match port.write_all(command) {
        Ok(_) => {
            println!("Successfully sent command");
            Ok(())
        },
        Err(e) => Err(e),
    }
}