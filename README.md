# Rust Serial Communication with Marlin Firmware

This Rust program demonstrates serial communication with Marlin firmware. It sends a command to the firmware via a serial port and prints the response.

## Usage

1. Compile the Rust program using Cargo:

```cargo build --release```

### Commands

- Operation: Control the activity of the machine. Includes start/stop operation, pause/resume operation.

- Movement: Control the movement of the machine or tool, such as it's speed, location, or direction.

- Tools: Control the machine's tools, like tool selection, tool changer operation, speed/spindle control.

- Config: Configure the machine, such as setting feed rate, units of measurement, or endstop behaviour.

- Information: Output information about the machine or its status.

- Special: All commands that don't fit into the other categories and perform specific unique functions.