# Xcontroller

<img src=".github/logo.png" alt="xcontroller logo" width="400"/>

3D controller written in Rust. Serialcom to send commands to the main board. Websocket server for communication with the controller.

## Usage

1. Compile the Rust program using Cargo:

```cargo build --release```

2. Run the application - Development

```cargo run -- <test_mode_boolean>```
```RUST_LOG=debug cargo run -- <test_mode_boolean>```
Windows: ``` $env:RUST_LOG="debug"; cargo run```

Execute binary
```./xcontroller -- <test_mode_boolean>```

3. Run the application with defined params
``` ./xcontroller -- <websocket_port_value> <serial_port_string> <baudrate_value> <test_mode_boolean>```

Note:
Running the program without params it will fallback to default values.

Default configurations:
``` Config { test_mode: false, serial_port: /dev/ttyUSB0, baud_rate: 115200, ws_port: 9002} ```

4. Install or update as a service
This will allow the service to restart with the correct params on reboot

```./install_service.sh 8080 "/dev/ttyUSB0" 115200 true```

Note on how to make the script executable:
```chmod +x install_service.sh```


Start/Restart or stop the service:
```systemctl start/restart/stop xcontroller ```

Enable or disable the service
```systemctl enable/disable xcontroller ```

See the logs
```journalctl -u xcontroller```

### Commands

- Operation: Control the activity of the machine. Includes start/stop operation, pause/resume operation.

- Movement: Control the movement of the machine or tool, such as it's speed, location, or direction.

- Tools: Control the machine's tools, like tool selection, tool changer operation, speed/spindle control.

- Config: Configure the machine, such as setting feed rate, units of measurement, or endstop behaviour.

- Information: Output information about the machine or its status.

- Special: All commands that don't fit into the other categories and perform specific unique functions.

### External docs

- Marlin GCode docs: https://marlinfw.org/meta/gcode/
