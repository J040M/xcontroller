# Rust Serial Communication for 3D printers

3D controller written in Rust. Serialcom to send commands to the main board. Websocket server for further communication.

## Usage

1. Compile the Rust program using Cargo:

```cargo build --release```

2. Run the application (dev) - (prod)


```cargo run -- <test_mode_boolean>```
```RUST_LOG=debug cargo run -- <test_mode_boolean>```
```./xcontroller -- <test_mode_boolean>```


### Commands

- Operation: Control the activity of the machine. Includes start/stop operation, pause/resume operation.

- Movement: Control the movement of the machine or tool, such as it's speed, location, or direction.

- Tools: Control the machine's tools, like tool selection, tool changer operation, speed/spindle control.

- Config: Configure the machine, such as setting feed rate, units of measurement, or endstop behaviour.

- Information: Output information about the machine or its status.

- Special: All commands that don't fit into the other categories and perform specific unique functions.