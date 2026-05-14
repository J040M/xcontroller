use crate::structs::Config;
use log::warn;
use std::env;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1";
const DEFAULT_MAX_CLIENTS: usize = 8;
const DEFAULT_MAX_UPLOAD_BYTES: u64 = 64 * 1024 * 1024;

pub fn get_configuration(args: Vec<String>) -> Config {
    // Set defaults in case arguments are not provided
    let mut configuration = Config {
        test_mode: false,
        serial_port: "/dev/ttyUSB0".to_string(),
        baud_rate: 115200,
        ws_port: "9002".to_string(),
        bind_addr: DEFAULT_BIND_ADDR.to_string(),
        auth_token: None,
        max_clients: DEFAULT_MAX_CLIENTS,
        max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
    };

    if args.len() > 4 {
        let ws_port = args[1].clone();
        let serial_port = args[2].clone();
        let baudrate = args[3].clone();
        let test_arg = args[4].clone();

        configuration.ws_port = ws_port;
        configuration.serial_port = serial_port;
        configuration.test_mode = matches!(test_arg.to_lowercase().as_str(), "true");
        configuration.baud_rate = match baudrate.parse::<u32>() {
            Ok(br) => br,
            Err(_) => {
                warn!("Failed to parse baudrate. Using default baudrate 115200");
                115200
            }
        };
    }

    // Security/networking knobs come from env vars so existing positional
    // CLI invocations and the systemd unit don't need to change.
    if let Ok(addr) = env::var("XCONTROLLER_BIND_ADDR") {
        configuration.bind_addr = addr;
    }
    if let Ok(token) = env::var("XCONTROLLER_AUTH_TOKEN") {
        if !token.is_empty() {
            configuration.auth_token = Some(token);
        }
    }
    if let Ok(max) = env::var("XCONTROLLER_MAX_CLIENTS") {
        match max.parse::<usize>() {
            Ok(n) if n > 0 => configuration.max_clients = n,
            _ => warn!(
                "Invalid XCONTROLLER_MAX_CLIENTS={}, using default {}",
                max, DEFAULT_MAX_CLIENTS
            ),
        }
    }
    if let Ok(max) = env::var("XCONTROLLER_MAX_UPLOAD_BYTES") {
        match max.parse::<u64>() {
            Ok(n) if n > 0 => configuration.max_upload_bytes = n,
            _ => warn!(
                "Invalid XCONTROLLER_MAX_UPLOAD_BYTES={}, using default {}",
                max, DEFAULT_MAX_UPLOAD_BYTES
            ),
        }
    }

    if configuration.bind_addr != "127.0.0.1"
        && configuration.bind_addr != "localhost"
        && configuration.auth_token.is_none()
    {
        warn!(
            "Listening on non-loopback address {} without XCONTROLLER_AUTH_TOKEN set; \
             any host on the network can drive the printer.",
            configuration.bind_addr
        );
    }

    configuration
}

#[cfg(test)]
mod tests {
    use super::*;

    // Env vars are process-global; gate test access behind a mutex so
    // parallel `cargo test` runs don't see each other's mutations.
    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        env::remove_var("XCONTROLLER_BIND_ADDR");
        env::remove_var("XCONTROLLER_AUTH_TOKEN");
        env::remove_var("XCONTROLLER_MAX_CLIENTS");
        env::remove_var("XCONTROLLER_MAX_UPLOAD_BYTES");
    }

    #[test]
    fn test_get_configuration_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();

        let args: Vec<String> = vec![];
        let config = get_configuration(args);

        assert!(!config.test_mode);
        assert_eq!(config.serial_port, "/dev/ttyUSB0");
        assert_eq!(config.baud_rate, 115200);
        assert_eq!(config.ws_port, "9002");
        assert_eq!(config.bind_addr, "127.0.0.1");
        assert!(config.auth_token.is_none());
        assert_eq!(config.max_clients, DEFAULT_MAX_CLIENTS);
        assert_eq!(config.max_upload_bytes, DEFAULT_MAX_UPLOAD_BYTES);
    }

    #[test]
    fn test_get_configuration_with_args() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();

        let args: Vec<String> = vec![
            "program_name".to_string(),
            "8080".to_string(),
            "/dev/ttyS0".to_string(),
            "9600".to_string(),
            "true".to_string(),
        ];
        let config = get_configuration(args);

        assert!(config.test_mode);
        assert_eq!(config.serial_port, "/dev/ttyS0");
        assert_eq!(config.baud_rate, 9600);
        assert_eq!(config.ws_port, "8080");
    }

    #[test]
    fn test_get_configuration_invalid_baudrate() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();

        let args: Vec<String> = vec![
            "program_name".to_string(),
            "8080".to_string(),
            "/dev/ttyS0".to_string(),
            "invalid_baudrate".to_string(),
            "false".to_string(),
        ];
        let config = get_configuration(args);

        assert!(!config.test_mode);
        assert_eq!(config.serial_port, "/dev/ttyS0");
        assert_eq!(config.baud_rate, 115200); // Default baud rate
        assert_eq!(config.ws_port, "8080");
    }

    #[test]
    fn test_env_overrides() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("XCONTROLLER_BIND_ADDR", "0.0.0.0");
        env::set_var("XCONTROLLER_AUTH_TOKEN", "s3cret");
        env::set_var("XCONTROLLER_MAX_CLIENTS", "16");
        env::set_var("XCONTROLLER_MAX_UPLOAD_BYTES", "1048576");

        let config = get_configuration(vec![]);

        assert_eq!(config.bind_addr, "0.0.0.0");
        assert_eq!(config.auth_token.as_deref(), Some("s3cret"));
        assert_eq!(config.max_clients, 16);
        assert_eq!(config.max_upload_bytes, 1_048_576);

        clear_env();
    }

    #[test]
    fn test_max_upload_bytes_invalid_falls_back_to_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("XCONTROLLER_MAX_UPLOAD_BYTES", "not-a-number");

        let config = get_configuration(vec![]);

        assert_eq!(config.max_upload_bytes, DEFAULT_MAX_UPLOAD_BYTES);
        clear_env();
    }

    #[test]
    fn test_empty_token_treated_as_unset() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("XCONTROLLER_AUTH_TOKEN", "");

        let config = get_configuration(vec![]);

        assert!(config.auth_token.is_none());

        clear_env();
    }
}
