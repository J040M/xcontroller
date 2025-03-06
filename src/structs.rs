use serde::{Deserialize, Serialize};

/// Used for identifying the type of incoming message
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    GCommand,
    SerialConfig,
    Unsafe,
    Terminal,
}

/// Used for received messages
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageWS<'a> {
    pub message_type: MessageType,
    pub message: &'a str,
}

/// M115 - Firmware and Capabilities
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PrinterInfo {
    pub firmware_name: String,
    pub firmware_version: String,
    pub serial_xon_xoff: u8,
    pub eeprom: u8,
    pub volumetric: u8,
    pub autoreport_pos: u8,
    pub autoreport_temp: u8,
    pub progress: u8,
    pub print_job: u8,
    pub autolevel: u8,
    pub runout: u8,
    pub z_probe: u8,
    pub leveling_data: u8,
    pub build_percent: u8,
    pub software_power: u8,
    pub toggle_lights: u8,
    pub case_light_brightness: u8,
    pub emergency_parser: u8,
    pub most_action_commands: u8,
    pub prompt_support: u8,
    pub sdcard: u8,
    pub repeat: u8,
    pub sd_write: u8,
    pub auto_report_sd_status: u8,
    pub long_filename: u8,
    pub thermal_protection: u8,
    pub motion_modes: u8,
    pub arcs: u8,
    pub babystepping: u8,
    pub chamber_temperature: u8,
    pub cooler_temperature: u8,
    pub meatpack: u8,
}

/**
 * M114 - Get Current Position
 * also used printer object
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct AxePositions {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

/// M105 - Get Extruder Temperature
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Temperatures {
    pub bed: u8,
    pub bed_set: u8,
    pub e0: u8,
    pub e0_set: u8,
    pub e1: u8,
    pub e1_set: u8,
    pub e2: u8,
    pub e2_set: u8,
    pub e3: u8,
    pub e3_set: u8,
}

/// Com configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub test_mode: bool,
    pub serial_port: String,
    pub baud_rate: u32,
    pub ws_port: String,
}

// Used for sending messages back to clients
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageSender {
    pub message_type: String,
    pub message: String,
    pub raw_message: String,
    pub timestamp: u64,
}

/// M119 - Get Endstop Status
#[derive(Debug, Serialize, Deserialize)]
pub struct EndstopStatus {
    pub x_min: String,
    pub y_min: String,
    pub z_min: String,
}
