use log::warn;
use std::env;

use crate::Config;

pub fn get_configuration() -> Config {
    
    // Set defaults in case arguments are not provided
    let mut configuration = Config {
        test_mode: false,
        serial_port: "/dev/ttyUSB0".to_string(),
        baud_rate: 115200,
        ws_port: "9002".to_string(),
    };

    // Set config from start params
    let args: Vec<String> = env::args().collect();
    if args.len() > 4 {
        let ws_port = args[1].clone(); // Convert String to &str
        let serial_port = args[2].clone();
        let baudrate = args[3].clone();
        let test_arg = args[4].clone();

        configuration = Config {
            test_mode: matches!(test_arg.to_lowercase().as_str(), "true"),
            baud_rate: match baudrate.parse::<u32>() {
                Ok(br) => br,
                Err(_) => {
                    warn!("Failed to parse baudrate. Using default baudrate 115200");
                    115200
                }
            },
            serial_port,
            ws_port,
        };
    }

    return configuration; 
}
