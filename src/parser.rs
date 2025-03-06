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

    for part in parts {
        let file_parts: Vec<&str> = part.split_whitespace().collect();
        for part in file_parts {
            if part.contains(".gcode") {
                files.push(part.to_string());
            }
        }
    }

    files
}

/**
 * Get SD printing status
 * @param message: String, return message from firmware
 * @return String, percentage of the print or "not-printing"
 */
pub fn m27(message: String) -> String {
    if message.contains("Not SD printing") {
        return "not-printing".to_string();
    }

    if message.contains("SD printing byte") {
        let re = Regex::new(r"SD printing byte\s+(\d+)/(\d+)").unwrap();

        if let Some(captures) = re.captures(&message) {
            let current_str = captures.get(1).map_or("0", |m| m.as_str());
            let total_str = captures.get(2).map_or("1", |m| m.as_str());

            if let (Ok(current), Ok(total)) = (current_str.parse::<f64>(), total_str.parse::<f64>())
            {
                if total > 0.0 {
                    let percentage = (current / total) * 100.0;
                    return format!("{:.1}", percentage);
                }
            } else {
                debug!("Failed to parse printing progress values");
            }
        } else {
            debug!("Failed to find printing progress values");
        }
    }

    "not-printing".to_string()
}

/**
 * Get print time from completed print
 * @param message: String, return message from firmware
 * @return String, print time or "not-printing" if no print or 0s print time
 */
pub fn m31(message: String) -> String {
    let re = Regex::new(r"Print time:\s+([0-9]+[hm]?\s+[0-9]+[ms]|[0-9]+[s])").unwrap();

    if let Some(captures) = re.captures(&message) {
        let print_time = captures.get(1).map_or("", |m| m.as_str()).trim();

        if print_time == "0s" {
            return "not-printing".to_string();
        }

        return print_time.to_string();
    }

    "not-printing".to_string()
}

/**
 * Get long path of a single file
 * @param message: String, return message from firmware
 * @return String, return long path of single file
 */
pub fn m33(message: String) -> String {
    let parts: Vec<&str> = message.split("\n").collect();
    let mut file_path = String::new();

    for part in parts {
        let file_parts: Vec<&str> = part.split_whitespace().collect();
        for part in file_parts {
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
    let mut temperatures = Temperatures::default();

    let re = Regex::new(r"T:(\d+)\.?\d*\s*/(\d+)\.?\d*\s*B:(\d+)\.?\d*\s*/(\d+)").unwrap();

    if let Some(captures) = re.captures(&message) {
        debug!("Captures: {:?}", captures);

        temperatures.e0 = captures.get(1).unwrap().as_str().parse().unwrap_or(0);
        temperatures.e0_set = captures.get(2).unwrap().as_str().parse().unwrap_or(0);
        temperatures.bed = captures.get(3).unwrap().as_str().parse().unwrap_or(0);
        temperatures.bed_set = captures.get(4).unwrap().as_str().parse().unwrap_or(0);
    }

    temperatures
}

/**
 * Report current axe position
 * @param message: String, return message from firmware
 * @return AxePositions, current position of the axes
 */
pub fn m114(message: String) -> AxePositions {
    let parts: Vec<&str> = message.split("\n").collect();
    let mut axes = AxePositions { x: 0, y: 0, z: 0 };

    for upart in parts {
        let set_parts: Vec<&str> = upart.split_whitespace().collect();
        for part in set_parts {
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
            }
        }
    }

    axes
}

/**
 * Get printer info
 * @param message: String, return message from firmware
 * @return PrinterInfo, printer information
 */
pub fn m115(message: String) -> PrinterInfo {
    let mut printer_info = PrinterInfo::default();

    let parts: Vec<&str> = message.split("\n").collect();
    for part in parts {
        if part.contains("FIRMWARE_NAME") {
            let fw_parts: Vec<&str> = part.split(":").collect();
            let fw_version: Vec<&str> = fw_parts[1].split_whitespace().collect();
            debug!("FIRMWARE VERSION: {} {}", fw_version[0], fw_version[1]);

            printer_info.firmware_name = fw_version[0].to_string();
            printer_info.firmware_version = fw_version[1].to_string();
        } else if part.contains("Cap:") {
            let cap_parts: Vec<&str> = part.split(":").collect();
            match cap_parts[0] {
                "SERIAL_XON_XOFF" => {
                    println!("{}", printer_info.serial_xon_xoff);
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

    printer_info
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
        let sample_response =
            "Begin file list\nfile1.gcode\nfile2.gcode\nsubdir/file3.gcode\nEnd file list"
                .to_string();
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
        println!("{:?}", temps);
        assert_eq!(temps.e0, 185);
        assert_eq!(temps.e0_set, 200);
        assert_eq!(temps.bed, 55);
        assert_eq!(temps.bed_set, 60);
    }

    #[test]
    fn test_m114_parser() {
        let sample_response = "X:10 Y:20 Z:30 E:0 Count X:10 Y:20 Z:30".to_string();
        let axes = m114(sample_response);
        assert_eq!(axes.x, 10);
        assert_eq!(axes.y, 20);
        assert_eq!(axes.z, 30);
    }

    #[test]
    fn test_m115_parser() {
        let sample_response = "FIRMWARE_NAME:Marlin 2.0.1\nCap:SERIAL_XON_XOFF:1\nCap:EEPROM:1\nCap:VOLUMETRIC:1\nCap:AUTOREPORT_TEMP:1\nCap:PROGRESS:1\nCap:PRINT_JOB:1\nCap:AUTOLEVEL:1\nCap:Z_PROBE:1\nCap:LEVELING_DATA:1\nCap:BUILD_PERCENT:1\nCap:SOFTWARE_POWER:1\nok".to_string();

        let info = m115(sample_response);
        assert_eq!(info.firmware_name, "Marlin");
        assert_eq!(info.firmware_version, "2.0.1");
        // print!("{:?}", info.serial_xon_xoff);
        // assert_eq!(info.serial_xon_xoff, 1);
        // assert_eq!(info.eeprom, 1);
        // assert_eq!(info.volumetric, 1);
        // assert_eq!(info.autoreport_temp, 1);
        // assert_eq!(info.progress, 1);
        // assert_eq!(info.print_job, 1);
        // assert_eq!(info.autolevel, 1);
        // assert_eq!(info.z_probe, 1);
        // assert_eq!(info.leveling_data, 1);
        // assert_eq!(info.build_percent, 1);
        // assert_eq!(info.software_power, 1);
    }

    #[test]
    fn test_m119_parser() {
        let sample_response =
            "Reporting endstop status\nx_min: TRIGGERED\ny_min: open\nz_min: open\nok".to_string();
        let status = m119(sample_response);
        assert_eq!(status.x_min, "TRIGGERED");
        assert_eq!(status.y_min, "open");
        assert_eq!(status.z_min, "open");
    }

    #[test]
    fn test_m27_not_printing() {
        let sample_response = "Not SD printing ok".to_string();
        let status = m27(sample_response);
        assert_eq!(status, "not-printing");
    }

    #[test]
    fn test_m27_printing() {
        let sample_response = "SD printing byte 2812/1798968 ok".to_string();
        let status = m27(sample_response);

        assert_eq!(status, "0.2"); // Rounded to one decimal place
    }

    #[test]
    fn test_m27_printing_complete() {
        let sample_response = "SD printing byte 1798968/1798968 ok".to_string();
        let status = m27(sample_response);
        assert_eq!(status, "100.0");
    }

    #[test]
    fn test_m27_printing_start() {
        let sample_response = "SD printing byte 0/1798968 ok".to_string();
        let status = m27(sample_response);
        assert_eq!(status, "0.0");
    }

    #[test]
    fn test_m31_no_print_time() {
        let sample_response = "ok".to_string();
        let print_time = m31(sample_response);
        assert_eq!(print_time, "not-printing");
    }

    #[test]
    fn test_m31_zero_seconds() {
        let sample_response = "echo:Print time: 0s ok".to_string();
        let print_time = m31(sample_response);
        assert_eq!(print_time, "not-printing");
    }

    #[test]
    fn test_m31_with_time() {
        let sample_response = "echo:Print time: 9m 33s ok".to_string();
        let print_time = m31(sample_response);
        assert_eq!(print_time, "9m 33s");
    }

    #[test]
    fn test_m31_with_temperatures() {
        let sample_response =
            "ok T:198.61 /200.00 B:49.98 /50.00 @:106 B@:27 echo:Print time: 9m 33s ok".to_string();
        let print_time = m31(sample_response);
        assert_eq!(print_time, "9m 33s");
    }

    #[test]
    fn test_m31_hours_minutes() {
        let sample_response = "echo:Print time: 2h 45m ok".to_string();
        let print_time = m31(sample_response);
        assert_eq!(print_time, "2h 45m");
    }
}
