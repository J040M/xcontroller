use regex::Regex;
use log::{ info, debug, error };

pub fn parsing_m20(message: String) {}
// This will only work for MARLIN. Probably only a certain number versions as well.
// Will get back to this a little later.
pub fn parsing_m114(message: String) {
    let parts: Vec<&str> = message.split("\n").collect();
    for (i, part) in parts.iter().enumerate() {
        let set_parts: Vec<&str> = part.split_whitespace().collect();
        for (i, part) in set_parts.iter().enumerate() {
            if part.contains("X") || part.contains("Y") || part.contains("Z") {
                let axe_parts: Vec<&str> = part.split(":").collect();
                let axis = axe_parts[0];
                let value: i32 = axe_parts[1].parse().unwrap(); // Parse the value to an integer
                match axis {
                    "X" => axes.x = value,
                    "Y" => axes.y = value,
                    "Z" => axes.z = value,
                    _ => {
                        debug!("Unmanged value: {}", axe_parts);
                    }
                }
                // We are returning so we don't have to deal with the values after "Count"
            } else if part.contains("Count") || part.contains("ok") {
                return;
            }
        }
    }
}

pub fn parsing_m115(message: String) {
    let mut printer_info = PrinterInfo {
        firmware_name: None,
        firmware_version: None,
        serial_xon_xoff: None,
        eeprom: None,
        volumetric: None,
        autoreport_pos: None,
        autoreport_temp: None,
        progress: None,
        print_job: None,
        autolevel: None,
        runout: None,
        z_probe: None,
        leveling_data: None,
        build_percent: None,
        software_power: None,
        toggle_lights: None,
        case_light_brightness: None,
        emergency_parser: None,
        most_action_commands: None,
        prompt_support: None,
        sdcard: None,
        repeat: None,
        sd_write: None,
        auto_report_sd_status: None,
        long_filename: None,
        thermal_protection: None,
        motion_modes: None,
        arcs: None,
        babystepping: None,
        chamber_temperature: None,
        cooler_temperature: None,
        meatpack: None,
    };

    let parts: Vec<&str> = message.split("\n").collect();
    for (i, part) in parts.iter().enumerate() {
        if part.contains("FIRMWARE_NAME") {
            let fw_parts: Vec<&str> = part.split(":").collect();
            let fw_version: Vec<&str> = fw_parts[1].split_whitespace().collect();
            debug!("FIRMWRE VERSION: {} {}", fw_version[0], fw_version[1]);

            printer_info.firmware_name = fw_version[0];
            printer_info.firmware_version = fw_version[1];
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
}

pub fn parsing_m105(message: String) {
    let re = Regex::new(r"T:([\d.]+)\s/([\d.]+)\sB:([\d.]+)\s/([\d.]+)").unwrap();

    // All these keys are not being used, YET!
    // Also a printer with more extredures won't work with this
    let mut temperatures = Temperatures {
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
    };

    if let Some(captures) = re.captures(message) {
        let nozzle_temp = captures.get(1).unwrap().as_str();
        let nozzle_def_temp = captures.get(2).unwrap().as_str();
        let bed_temp = captures.get(3).unwrap().as_str();
        let bed_def_temp = captures.get(4).unwrap().as_str();

        temperatures.bed = bed.parse().unwrap_or(0);
        temperatures.bed_set = bed_def_temp.parse().unwrap_or(0);
        temperatures.e0 = nozzle_temp.parse().unwrap_or(0);
        temperatures.e0_set = nozzle_def_temp.parse().unwrap_or(0);
    }
}