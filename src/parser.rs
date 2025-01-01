use log::debug;
use regex::Regex;

use crate::structs::{AxePositions, EndstopStatus, PrinterInfo, Temperatures};

/**
 *  List SD card
 * Command param "L" to list long filenames, "T" to list with timestamps
 * @param message: String, return message from firmware
 * @return Vec<String>, list of files on the SD card
 */
pub fn m20(message: String) -> Vec<String> {
    let parts: Vec<&str> = message.split("\n").collect();
    let mut files: Vec<String> = Vec::new();

    for (_i, part) in parts.iter().enumerate() {
        let file_parts: Vec<&str> = part.split_whitespace().collect();
        for (_i, part) in file_parts.iter().enumerate() {
            if part.contains(".gcode") {
                files.push(part.to_string());
            }
        }
    }

    files
}

/**
 * Get long path of a single file
 * @param message: String, return message from firmware
 * @return String, return long path of single file
 */
pub fn m33(message: String) -> String {
    let parts: Vec<&str> = message.split("\n").collect();
    let mut file_path = String::new();

    for (_i, part) in parts.iter().enumerate() {
        let file_parts: Vec<&str> = part.split_whitespace().collect();
        for (_i, part) in file_parts.iter().enumerate() {
            if part.contains(".gcode") {
                file_path = part.to_string();
            }
        }
    }

    file_path
}

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


/*****************/
/*     Tests     */
/*****************/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_m20_parser() {
        let sample_response = "Begin file list\nfile1.gcode\nfile2.gcode\nsubdir/file3.gcode\nEnd file list".to_string();
        let files = m20(sample_response);
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"file1.gcode".to_string()));
        assert!(files.contains(&"file2.gcode".to_string()));
        assert!(files.contains(&"subdir/file3.gcode".to_string()));
    }

    #[test]
    fn test_m33_parser() {
        let sample_response = "Path: /test/long/path/file.gcode\nok".to_string();
        let file_path = m33(sample_response);
        assert_eq!(file_path, "/test/long/path/file.gcode");
    }

    #[test]
    fn test_m105_parser() {
        let sample_response = "ok T:185.4 /200.0 B:55.2 /60.0 @:127 B@:0".to_string();
        let temps = m105(sample_response);
        assert_eq!(temps.e0, 185);
        assert_eq!(temps.e0_set, 200);
        assert_eq!(temps.bed, 55);
        assert_eq!(temps.bed_set, 60);
    }

    #[test]
    fn test_m114_parser() {
        let sample_response = "X:10 Y:20 Z:30 E:0 Count X:10 Y:20 Z:30".to_string();
        let axes = m114(sample_response).unwrap();
        assert_eq!(axes.x, 10);
        assert_eq!(axes.y, 20);
        assert_eq!(axes.z, 30);

        // Test error case
        let error_response = "Count X:10 Y:20 Z:30".to_string();
        assert!(m114(error_response).is_err());
    }

    #[test]
    fn test_m115_parser() {
        let sample_response = "FIRMWARE_NAME:Marlin 2.0.1\nCap:SERIAL_XON_XOFF:1\nCap:EEPROM:1\nCap:VOLUMETRIC:1\nCap:AUTOREPORT_TEMP:1\nCap:PROGRESS:1\nCap:PRINT_JOB:1\nCap:AUTOLEVEL:1\nCap:Z_PROBE:1\nCap:LEVELING_DATA:1\nCap:BUILD_PERCENT:1\nCap:SOFTWARE_POWER:1\nok".to_string();
        
        let info = m115(sample_response).unwrap();
        assert_eq!(info.firmware_name, "Marlin");
        assert_eq!(info.firmware_version, "2.0.1");
        assert_eq!(info.serial_xon_xoff, 1);
        assert_eq!(info.eeprom, 1);
        assert_eq!(info.volumetric, 1);
        assert_eq!(info.autoreport_temp, 1);
        assert_eq!(info.progress, 1);
        assert_eq!(info.print_job, 1);
        assert_eq!(info.autolevel, 1);
        assert_eq!(info.z_probe, 1);
        assert_eq!(info.leveling_data, 1);
        assert_eq!(info.build_percent, 1);
        assert_eq!(info.software_power, 1);
    }

    #[test]
    fn test_m119_parser() {
        let sample_response = "Reporting endstop status\nx_min: TRIGGERED\ny_min: open\nz_min: open\nok".to_string();
        let status = m119(sample_response);
        assert_eq!(status.x_min, "TRIGGERED");
        assert_eq!(status.y_min, "open");
        assert_eq!(status.z_min, "open");
    }
}
