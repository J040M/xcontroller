// This solution might need some more parcing and validation
// Validating the cmd to the printer data to avoid problematic commands

use std::io::{Error, ErrorKind, Result};

pub fn operation(cmd: &str) -> Result<&str> {
    let command = cmd.split_whitespace().next().unwrap();

    match command {
        "M0" | "M1" | "M2" | "M00" | "M01" | "M02" | "M06" | "M30" | "M60" | "M98" | "M99" | "M112" | "M120" | "M121" | "M226"
        | "M227" | "M228" | "M229" | "M230" | "M240" | "M245" | "M246" | "M600" | "M601" | "M602"
        | "M603" | "M605" | "M606" | "M607" | "M608" | "M650" | "M651" | "M701" | "M702" => Ok(cmd),
        _ => Err(Error::new(ErrorKind::Other, "Invalid command")),
    }
}

pub fn movement(cmd: &str) -> Result<&str> {
    let command = cmd.split_whitespace().next().unwrap();

    match command {
        "G0" | "G1" | "G2" | "G3" | "G4" | "G00" | "G01" | "G02" | "G03" | "G04" | "G28" | "G30" | "G33" | "G90" | "G91" | "G92" => Ok(cmd),
        _ => Err(Error::new(ErrorKind::Other, "Invalid command")),
    }
}

pub fn tools(cmd: &str) -> Result<&str> {
    let command = cmd.split_whitespace().next().unwrap();

    match command {
        "T" | "S" | "M3" | "M4" | "M5" | "M6" | "M7" | "M8" | "M9" | "M03" | "M04" | "M05" | "M06" | "M07" | "M08" | "M09" | "M12" | "M13" | "M14" | "G43" | "G44" | "G49" => Ok(cmd),
        _ => Err(Error::new(ErrorKind::Other, "Invalid command")),
    }
}

pub fn config(cmd: &str) -> Result<&str> {
    let command = cmd.split_whitespace().next().unwrap();

    match command {
        "F" | "G20" | "G21" | "G90" | "G91" | "G92" | "M82" | "M83" | "M106" | "M107" | "M211" | "M500" | "M501" | "M502" => Ok(cmd),
        _ => Err(Error::new(ErrorKind::Other, "Invalid command")),
    }
}

pub fn information(cmd: &str) -> Result<&str> {
    match cmd {
        "M114" | "M115" | "M119" | "M27" | "M105" | "M104" | "M112" | "M503" | "M21" | "M20" => Ok(cmd),
        _ => Err(Error::new(ErrorKind::Other, "Invalid command")),
    }
}

pub fn special(cmd: &str) -> Result<&str> {
    let command = cmd.split_whitespace().next().unwrap();

    match command {
        "M48" | "G28" | "G29" | "M108" | "M300" | "M301" | "M302" | "M303" | "M280" => Ok(cmd),
        _ => Err(Error::new(ErrorKind::Other, "Invalid command")),
    }
}
