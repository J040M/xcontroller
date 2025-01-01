use serde::{Deserialize, Serialize};

// Used for identifying the type of incoming message
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    GCommand,
    SerialConfig,
    Unsafe,
}

// Used for receiving messages
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageWS<'a> {
    pub message_type: MessageType,
    pub message: &'a str,
}

// M115 - Firmware and Capabilities
#[derive(Debug, Serialize, Deserialize)]
pub struct PrinterInfo<'a> {
    firmware_name: &'a str,
    firmware_version: &'a str,
    serial_xon_xoff: u8,
    eeprom: u8,
    volumetric: u8,
    autoreport_pos: u8,
    autoreport_temp: u8,
    progress: u8,
    print_job: u8,
    autolevel: u8,
    runout: u8,
    z_probe: u8,
    leveling_data: u8,
    build_percent: u8,
    software_power: u8,
    toggle_lights: u8,
    case_light_brightness: u8,
    emergency_parser: u8,
    most_action_commands: u8,
    prompt_support: u8,
    sdcard: u8,
    repeat: u8,
    sd_write: u8,
    auto_report_sd_status: u8,
    long_filename: u8,
    thermal_protection: u8,
    motion_modes: u8,
    arcs: u8,
    babystepping: u8,
    chamber_temperature: u8,
    cooler_temperature: u8,
    meatpack: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AxePositions {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Temperatures {
    bed: u8,
    bed_set: u8,
    e0: u8,
    e0_set: u8,
    e1: u8,
    e1_set: u8,
    e2: u8,
    e2_set: u8,
    e3: u8,
    e3_set: u8,
}

// xcontroller configuration on startup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub test_mode: bool,
    pub serial_port: String,
    pub baud_rate: u32,
    pub ws_port: String,
}

// Used for sending messages back to clients
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageSender<'a> {
    pub message_type: &'a str,
    pub message: &'a str,
    pub raw_message: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndstopStatus {
    pub x_min: String,
    pub y_min: String,
    pub z_min: String,
}