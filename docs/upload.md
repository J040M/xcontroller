## Binary file upload

Streams a G-code file to the printer's SD card over the existing WebSocket
connection using Marlin's Binary File Transfer protocol (M28 B1 / Mark II),
via the [`marlin-binary-transfer`](https://crates.io/crates/marlin-binary-transfer)
crate. Only one upload runs at a time process-wide; while it runs, every
other client's serial command queues behind it.

### Flow

1. **Client → `UploadBegin`** — a normal text `MessageWS` whose `message`
   field is the JSON-encoded `UploadRequest` (see below).
2. **Server → `UploadAck`** (`message: "ready"`) once the request validates,
   or **`UploadError`** with a reason if it does not.
3. **Client → binary frames** — the raw file body, sent as WebSocket binary
   frames totaling exactly `size` bytes. Ping/pong frames are ignored; a text
   frame, a close frame, or more bytes than declared aborts the upload.
4. **Server → `UploadProgress`** — streamed per chunk while the transfer runs.
5. **Server → `UploadDone`** (with an `UploadResult`) on success, or
   **`UploadError`** on failure.

### `UploadRequest`

The `message` field of the `UploadBegin` `MessageWS` is this struct, JSON-encoded:

| Field | Type | Notes |
|-------|------|-------|
| `dest_filename` | string | SD-card filename. No `/`, no NUL, ≤ 63 chars, must end in `.gco`, `.gcode`, or `.g`. |
| `size` | integer | Total bytes the client will send as binary frames. Must be `1..=XCONTROLLER_MAX_UPLOAD_BYTES` (default 64 MiB). |
| `compression` | string (optional) | `"none"` (default), `"heatshrink"`, or `"auto"`. `heatshrink`/`auto` require the binary built `--features heatshrink`, otherwise the request is rejected. |
| `dummy` | bool (optional) | When `true`, Marlin's M28 B1 dummy mode — the device pretends to receive the file without writing it. Useful for protocol smoke tests without a real SD card. |
| `chunk_size` | integer (optional) | Bytes per WRITE packet. Omit or `0` to use the device-advertised maximum from SYNC. |

Example `UploadBegin` message:

```json
{"message_type":"UploadBegin","message":"{\"dest_filename\":\"part.gco\",\"size\":120480,\"compression\":\"none\"}"}
```

### Server messages

All server replies use the standard `MessageSender` envelope
(`message_type`, `message`, `raw_message`, `timestamp`).

- **`UploadAck`** — `message: "ready"`. Send the binary frames now.
- **`UploadProgress`** — `message` is a JSON `UploadProgress`
  (`bytes_sent`, `chunks_sent`, `source_bytes`).
- **`UploadDone`** — `message` is a JSON `UploadResult`
  (`source_bytes`, `bytes_sent`, `chunks_sent`, `compression`).
- **`UploadError`** — `message` is a human-readable reason.

### Limits and failure modes

- **Single-flight**: a second concurrent `UploadBegin` is rejected with
  `UploadError` ("another upload is already in progress").
- **Size**: requests above `XCONTROLLER_MAX_UPLOAD_BYTES` are rejected.
- **Idle timeout**: if no binary frame arrives for 30 s while the payload is
  being collected, the upload is aborted with `UploadError` and the
  single-flight slot is freed — a stalled client cannot block future uploads.
- **Disconnect / oversized payload / non-binary frame** mid-transfer aborts
  the upload; the connection's other message handling is unaffected if it
  survives.

**Note**: requires the serial port to be held open across the whole transfer,
which xcontroller does for the lifetime of the process.
