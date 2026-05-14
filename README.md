# Xcontroller

<img src=".github/logo.png" alt="xcontroller logo" width="400"/>

A 3D-printer controller written in Rust. It bridges a serial connection to the
printer's main board and exposes a WebSocket server so clients can drive the
printer, read back status, and upload G-code to the SD card.

## Features

- **Serial bridge** — sends validated G/M/T-code to a Marlin board and reads
  back responses.
- **WebSocket server** — JSON message protocol for commands and responses, with
  structured parsing of common status replies (temperatures, position,
  endstops, firmware info, SD listing).
- **Authentication** — optional shared-token handshake gating every connection.
- **Concurrency limits** — caps simultaneous clients and refuses excess
  connections at accept time.
- **Binary SD-card upload** — streams G-code files to the printer using Marlin's
  Binary File Transfer protocol, with optional `heatshrink` compression.
- **Command allow-list** — only known-safe G/M/T codes reach the printer;
  dangerous ones (e.g. `M997` firmware flash) are blocked.
- **Runs as a service** — systemd unit and installer, running as a dedicated
  unprivileged user.

## Build

```
cargo build --release
```

To enable `heatshrink`/`auto` compression for binary SD-card uploads, build with
the optional feature:

```
cargo build --release --features heatshrink
```

## Run

During development:

```
cargo run
RUST_LOG=debug cargo run
```

Windows: `$env:RUST_LOG="debug"; cargo run`

The release binary takes either no arguments (all defaults) or all four
positional arguments:

```
./xcontroller <websocket_port> <serial_port> <baudrate> <test_mode>
./xcontroller 9002 /dev/ttyUSB0 115200 false
```

Passing fewer than four arguments silently falls back to the defaults:

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

## Configuration

Security and concurrency knobs are read from environment variables, so the
positional CLI and the systemd unit stay unchanged:

| Variable | Default | Purpose |
|----------|---------|---------|
| `XCONTROLLER_BIND_ADDR` | `127.0.0.1` | Listen address. Set to `0.0.0.0` for LAN access. The daemon logs a warning if you bind to a non-loopback address without setting `XCONTROLLER_AUTH_TOKEN`. |
| `XCONTROLLER_AUTH_TOKEN` | unset | Shared secret enabling the auth handshake (see [Authentication](#authentication)). Empty string is treated as unset. |
| `XCONTROLLER_MAX_CLIENTS` | `8` | Maximum concurrent WebSocket clients. Extra connections are refused at accept time. |
| `XCONTROLLER_MAX_UPLOAD_BYTES` | `67108864` (64 MiB) | Maximum accepted binary upload payload. `UploadBegin` requests larger than this are rejected. |

## Install as a service

Installs xcontroller as a systemd service so it restarts with the correct
parameters on reboot:

```
chmod +x install_service.sh
./install_service.sh 8080 "/dev/ttyUSB0" 115200 true
```

The installer creates a dedicated unprivileged `xcontroller` user (in the
`dialout` group for serial access) and writes a systemd unit that reads optional
environment variables from `/etc/xcontroller/xcontroller.env` (chmod `0600`):

```
XCONTROLLER_BIND_ADDR=0.0.0.0
XCONTROLLER_AUTH_TOKEN=<a-long-random-string>
XCONTROLLER_MAX_CLIENTS=8
XCONTROLLER_MAX_UPLOAD_BYTES=67108864
```

Manage the service:

```
sudo systemctl start|restart|stop xcontroller
sudo systemctl enable|disable xcontroller
journalctl -u xcontroller -f
```

## WebSocket protocol

Clients exchange JSON text frames with the server.

### Message envelope

**Client → server** (`MessageWS`):

```json
{ "message_type": "<type>", "message": "<payload>" }
```

**Server → client** (`MessageSender`):

```json
{ "message_type": "<type>", "message": "<parsed>", "raw_message": "<raw>", "timestamp": 1700000000 }
```

`timestamp` is Unix epoch seconds.

### Authentication

When `XCONTROLLER_AUTH_TOKEN` is set, the **first** message on every new
connection must be:

```json
{ "message_type": "Auth", "message": "<token>" }
```

The server replies with an `Auth` message — `message: "ok"` on success,
`"fail"` otherwise — and closes the connection on failure. After a successful
handshake the rest of the protocol works normally; sending `Auth` again later is
rejected. When no token is configured, no handshake is required.

### Message types

| `message_type` | `message` payload | Behaviour |
|----------------|-------------------|-----------|
| `GCommand` | a G/M/T-code, e.g. `M105` | Validated against the command allow-list and sent to the printer. The reply's `message_type` is the command, `raw_message` is the raw serial response, and `message` is structured JSON for `M20`, `M27`, `M31`, `M33`, `M105`, `M114`, `M115`, `M119` (raw text otherwise, empty when the printer reply is just `ok`). |
| `Terminal` | a G/M/T-code | Same allow-list validation as `GCommand`; the reply's `message_type` is `terminal` with the raw response in `message`. |
| `Unsafe` | a G/M/T-code | Currently validated against the same allow-list as `GCommand`/`Terminal`; the reply's `message_type` is `Unsafe`. |
| `SerialConfig` | — | Reserved for runtime serial reconfiguration; not yet implemented. |
| `Auth` | token | Only valid as the first message — see [Authentication](#authentication). |
| `UploadBegin` | JSON `UploadRequest` | Starts a binary SD-card upload — see [Binary file upload](#binary-file-upload). |

Only known-safe G/M/T codes are accepted; dangerous commands such as `M997`
(firmware flash) are deliberately excluded from the allow-list. Invalid JSON,
disallowed commands, and protocol misuse are answered with a `MessageSenderError`
(or `UploadError` during an upload).

### Binary file upload

G-code files can be streamed to the printer's SD card using Marlin's Binary File
Transfer protocol: an `UploadBegin` message, followed by raw binary WebSocket
frames, with progress and result messages streamed back. The handshake, the
`UploadRequest` JSON schema, and server-side limits are documented in
[`docs/upload.md`](docs/upload.md).

## Commands

G/M-code reference, grouped by purpose:

- **Operation** — start/stop and pause/resume machine activity.
- **Movement** — speed, location, and direction of the machine or tool.
- **Tools** — tool selection, tool changer operation, spindle/speed control.
- **Config** — feed rate, units of measurement, endstop behaviour.
- **Information** — machine and status output.
- **Special** — commands that don't fit the other categories.

Per-category lists live under [`docs/`](docs/):
[operation](docs/operation.md),
[movement](docs/movement.md),
[tool](docs/tool.md),
[config](docs/config.md),
[information](docs/information.md),
[special](docs/special.md).
Sample raw Marlin responses are in [`docs/marlin_cmd_rsp.md`](docs/marlin_cmd_rsp.md).

## External docs

- Marlin G-code reference: https://marlinfw.org/meta/gcode/
