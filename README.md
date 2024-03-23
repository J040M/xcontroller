# Rust Serial Communication with Marlin Firmware

This Rust program demonstrates serial communication with Marlin firmware. It sends a command to the firmware via a serial port and prints the response.

## Usage

1. Compile the Rust program using Cargo:

```cargo build --release```

### Commands

- Operation: Control the activity of the machine. Includes start/stop operation, pause/resume operation, etc. Example: M0 (Stop or Unconditional Halt)

- Movement: Control the movement of the machine or tool, such as it's speed, location, or direction. Example: G01 (Move in a straight line at a specified speed), G02 & G03 (Move in a clockwise or counter-clockwise circular arc)

- Coordinate: Coordinate system, or specify movements in terms of coordinates. Example: G90 & G91 (Set to Absolute or Incremental Positioning), G92 (Set Position)

- Tools: Control the machine's tools, like tool selection, tool changer operation, speed/spindle control. Example: T (Select tool), M03 & M04 (Start or stop spindle turning)

- Config: Configure the machine, such as setting feed rate, units of measurement, or endstop behaviour. Example: G20 & G21 (Set units), F (Feedrate)

- Information: Output information about the machine or its status. Example: M114 (Get Current Position), M115 (Get Firmware Version and Capabilities)

- Special: All commands that don't fit into the other categories and perform specific unique functions. Example: M48 (Measure Z Probe repeatability), G28 (Move to Origin/Home).
