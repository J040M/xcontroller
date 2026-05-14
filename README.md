# Xcontroller

<img src=".github/logo.png" alt="xcontroller logo" width="400"/>

3D controller written in Rust. Serialcom to send commands to the main board. Websocket server for communication with the controller.

## Usage

1. Compile the Rust program using Cargo:

```cargo build --release```

To enable `heatshrink`/`auto` compression for binary SD-card uploads, build
with the optional feature:

```cargo build --release --features heatshrink```

2. Run the application — Development

Defaults only:
```
cargo run
RUST_LOG=debug cargo run
```
Windows: ``` $env:RUST_LOG="debug"; cargo run```

3. Run the application with defined params

The binary takes either no arguments (all defaults) or all four positional arguments:

```./xcontroller <websocket_port> <serial_port> <baudrate> <test_mode>```

Example:

```./xcontroller 9002 /dev/ttyUSB0 115200 false```

Note: passing fewer than four arguments silently falls back to the defaults below.

Default configuration:
```
Config {
    test_mode: false,
    serial_port: "/dev/ttyUSB0",
    baud_rate: 115200,
    ws_port: "9002",
    bind_addr: "127.0.0.1",
    auth_token: None,
    max_clients: 8,
    max_upload_bytes: 67108864,
}
```

### Configuration via environment variables

Security and concurrency knobs are read from the environment so the
positional CLI stays unchanged:

| Variable | Default | Purpose |
|----------|---------|---------|
| `XCONTROLLER_BIND_ADDR` | `127.0.0.1` | Listen address. Set to `0.0.0.0` for LAN access. The daemon logs a warning if you bind to a non-loopback address without setting `XCONTROLLER_AUTH_TOKEN`. |
| `XCONTROLLER_AUTH_TOKEN` | unset | Shared secret enabling the auth handshake (see below). Empty string is treated as unset. |
| `XCONTROLLER_MAX_CLIENTS` | `8` | Maximum concurrent WebSocket clients. Extra connections are refused at accept time. |
| `XCONTROLLER_MAX_UPLOAD_BYTES` | `67108864` (64 MiB) | Maximum accepted binary upload payload. `UploadBegin` requests larger than this are rejected. See [`docs/upload.md`](docs/upload.md). |

### Authentication

When `XCONTROLLER_AUTH_TOKEN` is set, the very first message a client
sends on a new WebSocket connection must be:

```json
{"message_type":"Auth","message":"<token>"}
```

The server replies with an `Auth` MessageSender — `message: "ok"` on
success, `message: "fail"` otherwise — and closes the connection on
failure. After a successful handshake the rest of the protocol works as
normal.

When `XCONTROLLER_AUTH_TOKEN` is not set, no handshake is required.

4. Install or update as a service

This will allow the service to restart with the correct params on reboot:

```./install_service.sh 8080 "/dev/ttyUSB0" 115200 true```

The installer creates a dedicated unprivileged `xcontroller` user (in
the `dialout` group for serial access) and writes a systemd unit that
reads optional environment variables from
`/etc/xcontroller/xcontroller.env` (chmod `0600`). Put your token there:

```
XCONTROLLER_BIND_ADDR=0.0.0.0
XCONTROLLER_AUTH_TOKEN=<a-long-random-string>
XCONTROLLER_MAX_CLIENTS=8
```

Then `sudo systemctl restart xcontroller`.

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

Per-category G/M-code lists live under [`docs/`](docs/):
[operation](docs/operation.md),
[movement](docs/movement.md),
[tool](docs/tool.md),
[config](docs/config.md),
[information](docs/information.md),
[special](docs/special.md).
Sample raw Marlin responses are in [`docs/marlin_cmd_rsp.md`](docs/marlin_cmd_rsp.md).

### Binary file upload

G-code files can be streamed to the printer's SD card over the WebSocket
connection using Marlin's Binary File Transfer protocol (an `UploadBegin`
message followed by raw binary frames). The handshake, the `UploadRequest`
JSON schema, progress/result messages, and server-side limits are
documented in [`docs/upload.md`](docs/upload.md).

### External docs

- Marlin GCode docs: https://marlinfw.org/meta/gcode/
