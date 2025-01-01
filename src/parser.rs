use log::debug;
use regex::Regex;

use crate::structs::{AxePositions, EndstopStatus, PrinterInfo, Temperatures};

/**
 *  List SD card
 * Command param "L" to list long filenames, "T" to list with timestamps
 * @param message: String, return message from firmware
 * @return Vec<String>, list of files on the SD card
 */
pub fn m20(message: String) {}

/**
 * Get long path of a file
 * @param message: String, return message from firmware
 * @return String, return long path of single file
 */
pub fn m33(message: String) {}

/**
 * Report temperatures
 * @param message: String, return message from firmware
 * @return Temperatures, temperatures of bed and extruders
 */
pub fn m105(message: String) -> Temperatures {
    let re = Regex::new(r"T:([\d.]+)\s/([\d.]+)\sB:([\d.]+)\s/([\d.]+)").unwrap();

    // TODO: Printer with more extredures won't work with this
    let mut temperatures = Temperatures::default();

    if let Some(captures) = re.captures(&message) {
        let nozzle_temp = captures.get(1).unwrap().as_str();
        let nozzle_def_temp = captures.get(2).unwrap().as_str();
        let bed_temp = captures.get(3).unwrap().as_str();
        let bed_def_temp = captures.get(4).unwrap().as_str();

        temperatures.bed = bed_temp.parse().unwrap_or(0);
        temperatures.bed_set = bed_def_temp.parse().unwrap_or(0);
        temperatures.e0 = nozzle_temp.parse().unwrap_or(0);
        temperatures.e0_set = nozzle_def_temp.parse().unwrap_or(0);
    }

    temperatures
}

/**
 * Report current axe position
 * @param message: String, return message from firmware
 * @return AxePositions, current position of the axes
 */
pub fn m114(message: String) -> Result<AxePositions, ()> {
    let parts: Vec<&str> = message.split("\n").collect();
    let mut axes = AxePositions { x: 0, y: 0, z: 0 };

    for (_i, part) in parts.iter().enumerate() {
        let set_parts: Vec<&str> = part.split_whitespace().collect();
        for (_i, part) in set_parts.iter().enumerate() {
            if part.contains("X") || part.contains("Y") || part.contains("Z") {
                let axe_parts: Vec<&str> = part.split(":").collect();
                let axis = axe_parts[0];
                let value: i8 = axe_parts[1].parse().unwrap(); // Parse the value to an integer

                match axis {
                    "X" => axes.x = value,
                    "Y" => axes.y = value,
                    "Z" => axes.z = value,
                    _ => {
                        debug!("Unmanged axis value: {:?}", axe_parts);
                    }
                }
            // We are returning so we don't have to deal with the values after "Count"
            } else if part.contains("Count") || part.contains("ok") {
                return Err(());
            }
        }
    }

    Ok(axes)
}

/**
 * Get printer info
 * @param message: String, return message from firmware
 * @return PrinterInfo, printer information
 */
pub fn m115(message: String) -> Result<PrinterInfo, ()> {
    let mut printer_info = PrinterInfo::default();

    let parts: Vec<&str> = message.split("\n").collect();
    for (_i, part) in parts.iter().enumerate() {
        if part.contains("FIRMWARE_NAME") {
            let fw_parts: Vec<&str> = part.split(":").collect();
            let fw_version: Vec<&str> = fw_parts[1].split_whitespace().collect();
            debug!("FIRMWRE VERSION: {} {}", fw_version[0], fw_version[1]);

            printer_info.firmware_name = fw_version[0].to_string();
            printer_info.firmware_version = fw_version[1].to_string();
        } else if part.contains("Cap:") {
            let cap_parts: Vec<&str> = part.split(":").collect();
            match cap_parts[0] {
                "SERIAL_XON_XOFF" => {
                    printer_info.serial_xon_xoff = cap_parts[1].parse().unwrap_or(0)
                }
                "EEPROM" => printer_info.eeprom = cap_parts[1].parse().unwrap_or(0),
                "VOLUMETRIC" => printer_info.volumetric = cap_parts[1].parse().unwrap_or(0),
                "AUTOREPORT_POS" => printer_info.autoreport_pos = cap_parts[1].parse().unwrap_or(0),
                "AUTOREPORT_TEMP" => {
                    printer_info.autoreport_temp = cap_parts[1].parse().unwrap_or(0)
                }
                "PROGRESS" => printer_info.progress = cap_parts[1].parse().unwrap_or(0),
                "print_job" => printer_info.print_job = cap_parts[1].parse().unwrap_or(0),
                "autolevel" => printer_info.autolevel = cap_parts[1].parse().unwrap_or(0),
                "RUNOUT" => printer_info.runout = cap_parts[1].parse().unwrap_or(0),
                "z_probe" => printer_info.z_probe = cap_parts[1].parse().unwrap_or(0),
                "LEVELING_DATA" => printer_info.leveling_data = cap_parts[1].parse().unwrap_or(0),
                "BUILD_PERCENT" => printer_info.build_percent = cap_parts[1].parse().unwrap_or(0),
                "SOFTWARE_POWER" => printer_info.software_power = cap_parts[1].parse().unwrap_or(0),
                "TOGGLE_LIGHTS" => printer_info.toggle_lights = cap_parts[1].parse().unwrap_or(0),
                "CASE_LIGHT_BRIGHTNESS" => {
                    printer_info.case_light_brightness = cap_parts[1].parse().unwrap_or(0)
                }
                "EMERGENCY_PARSER" => {
                    printer_info.emergency_parser = cap_parts[1].parse().unwrap_or(0)
                }
                "MOST_ACTION_COMMANDS" => {
                    printer_info.most_action_commands = cap_parts[1].parse().unwrap_or(0)
                }
                "PROMPT_SUPPORT" => printer_info.prompt_support = cap_parts[1].parse().unwrap_or(0),
                "SDCARD" => printer_info.sdcard = cap_parts[1].parse().unwrap_or(0),
                "REPEAT" => printer_info.repeat = cap_parts[1].parse().unwrap_or(0),
                "SD_WRITE" => printer_info.sd_write = cap_parts[1].parse().unwrap_or(0),
                "AUTO_REPORT_SD_STATUS" => {
                    printer_info.auto_report_sd_status = cap_parts[1].parse().unwrap_or(0)
                }
                "LONG_FILENAME" => printer_info.long_filename = cap_parts[1].parse().unwrap_or(0),
                "THERMAL_PROTECTION" => {
                    printer_info.thermal_protection = cap_parts[1].parse().unwrap_or(0)
                }
                "MOTION_MODES" => printer_info.motion_modes = cap_parts[1].parse().unwrap_or(0),
                "ARCS" => printer_info.arcs = cap_parts[1].parse().unwrap_or(0),
                "BABYSTEPPING" => printer_info.babystepping = cap_parts[1].parse().unwrap_or(0),
                "CHAMBER_TEMPERATURES" => {
                    printer_info.chamber_temperature = cap_parts[1].parse().unwrap_or(0)
                }
                "COOLER_TEMPERATURE" => {
                    printer_info.cooler_temperature = cap_parts[1].parse().unwrap_or(0)
                }
                "MEATPACK" => printer_info.meatpack = cap_parts[1].parse().unwrap_or(0),
                _ => {
                    debug!("Failed to parse | {}", cap_parts[0])
                }
            }
        }
    }

    Ok(printer_info)
}

/**
 * Get endstop status
 * @param message: String, return message from firmware
 * @return EndstopStatus, status of the endstops
 */
pub fn m119(message: String) -> EndstopStatus {
    let mut endstop_status = EndstopStatus {
        x_min: "".to_string(),
        y_min: "".to_string(),
        z_min: "".to_string(),
    };

    let lines: Vec<&str> = message.split('\n').collect();

    for line in lines {
        if line.contains("x_min:") {
            let status = line.split(':').nth(1).map(|s| s.trim().to_string());
            endstop_status.x_min = status.unwrap_or("None".to_string());
        } else if line.contains("y_min:") {
            let status = line.split(':').nth(1).map(|s| s.trim().to_string());
            endstop_status.y_min = status.unwrap_or("None".to_string());
        } else if line.contains("z_min:") {
            let status = line.split(':').nth(1).map(|s| s.trim().to_string());
            endstop_status.z_min = status.unwrap_or("None".to_string());
        }
    }

    endstop_status
}
