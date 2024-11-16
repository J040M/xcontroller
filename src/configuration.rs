use crate::Config;
use log::warn;

pub fn get_configuration(args: Vec<String>) -> Config {
    // Set defaults in case arguments are not provided
    let mut configuration = Config {
        test_mode: false,
        serial_port: "/dev/ttyUSB0".to_string(),
        baud_rate: 115200,
        ws_port: "9002".to_string(),
    };

    if args.len() > 4 {
        let ws_port = args[1].clone();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_configuration_defaults() {
        let args: Vec<String> = vec![];
        let config = get_configuration(args);

        assert_eq!(config.test_mode, false);
        assert_eq!(config.serial_port, "/dev/ttyUSB0");
        assert_eq!(config.baud_rate, 115200);
        assert_eq!(config.ws_port, "9002");
    }

    #[test]
    fn test_get_configuration_with_args() {
        let args: Vec<String> = vec![
            "program_name".to_string(),
            "8080".to_string(),
            "/dev/ttyS0".to_string(),
            "9600".to_string(),
            "true".to_string(),
        ];
        let config = get_configuration(args);

        assert_eq!(config.test_mode, true);
        assert_eq!(config.serial_port, "/dev/ttyS0");
        assert_eq!(config.baud_rate, 9600);
        assert_eq!(config.ws_port, "8080");
    }

    #[test]
    fn test_get_configuration_invalid_baudrate() {
        let args: Vec<String> = vec![
            "program_name".to_string(),
            "8080".to_string(),
            "/dev/ttyS0".to_string(),
            "invalid_baudrate".to_string(),
            "false".to_string(),
        ];
        let config = get_configuration(args);

        assert_eq!(config.test_mode, false);
        assert_eq!(config.serial_port, "/dev/ttyS0");
        assert_eq!(config.baud_rate, 115200); // Default baud rate
        assert_eq!(config.ws_port, "8080");
    }
}
