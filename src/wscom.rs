use futures::{stream::StreamExt, SinkExt};
use log::{debug, error, info, warn};
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
};
use tungstenite::Message;

use marlin_binary_transfer::adapters::blocking::{
    upload as binary_upload, UploadError, UploadOptions, UploadStats,
};
use marlin_binary_transfer::file_transfer::Compression;

use crate::commands::g_command;
use crate::serialcom::SerialConnection;

use crate::parser::{m105, m114, m115, m119, m20, m27, m31, m33};
use crate::structs::{MessageSender, UploadProgress, UploadRequest, UploadResult};
use crate::Config;
use crate::MessageType;
use crate::MessageWS;

type SharedSerial = Arc<Mutex<SerialConnection>>;
pub type UploadInFlight = Arc<AtomicBool>;

/// Max wait for the next binary frame while collecting an upload payload.
/// A stalled client must not hold the process-global `upload_in_flight`
/// flag (and thus block every other upload) indefinitely.
const UPLOAD_IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// RAII guard that clears the upload-in-flight flag on drop. Constructed
/// only after a successful `compare_exchange(false, true)` so it has
/// exclusive ownership of the flag for its lifetime. Using a guard
/// rather than manual `store(false)` calls means error short-circuits
/// (`?` on a failing `send_message_back`) can't leak the flag.
struct InFlightGuard(UploadInFlight);

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn error_message(kind: &str, detail: &str) -> MessageSender {
    MessageSender {
        message_type: kind.to_string(),
        message: detail.to_string(),
        raw_message: detail.to_string(),
        timestamp: now_ts(),
    }
}

/// Server-side validation for `UploadRequest::dest_filename`. Marlin's
/// SD layer allows up to 64 chars including the NUL terminator, so 63 is
/// the usable cap. We also reject directory separators (no traversal)
/// and require a recognised g-code extension.
fn valid_dest_filename(name: &str) -> bool {
    if name.is_empty() || name.len() > 63 {
        return false;
    }
    if name.contains('/') || name.contains('\0') {
        return false;
    }
    let lower = name.to_lowercase();
    lower.ends_with(".gco") || lower.ends_with(".gcode") || lower.ends_with(".g")
}

/// Translate the request's compression string into a `Compression` value,
/// returning `Err(reason)` if the requested mode isn't available in this
/// build. The `heatshrink` feature gates everything except `"none"`.
fn resolve_compression(requested: Option<&str>) -> std::result::Result<Compression, String> {
    match requested.unwrap_or("none") {
        "none" => Ok(Compression::None),
        "heatshrink" => {
            #[cfg(feature = "heatshrink")]
            {
                Ok(Compression::Heatshrink {
                    window: 8,
                    lookahead: 4,
                })
            }
            #[cfg(not(feature = "heatshrink"))]
            {
                Err("compression unavailable: rebuild with --features heatshrink".into())
            }
        }
        "auto" => {
            #[cfg(feature = "heatshrink")]
            {
                Ok(Compression::Auto)
            }
            #[cfg(not(feature = "heatshrink"))]
            {
                Err("compression auto unavailable: rebuild with --features heatshrink".into())
            }
        }
        other => Err(format!("unknown compression mode: {}", other)),
    }
}

/// Stable string token for a `Compression` value, used in the `UploadDone`
/// payload instead of debug-formatting the enum. All variants exist
/// regardless of the `heatshrink` feature — that feature only gates whether
/// `resolve_compression` will *accept* a request for them.
fn compression_label(c: &Compression) -> &'static str {
    match c {
        Compression::None => "none",
        Compression::Heatshrink { .. } => "heatshrink",
        Compression::Auto => "auto",
    }
}

/**
 * Accept incoming connection from client
 * @param peer: SocketAddr, peer address
 * @param stream: TcpStream, stream from client
 * @param configuration: Config, configuration for the server
 * @return Result<(), Error>, return Ok(())
 * @throws Error
 */
pub async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    configuration: Config,
    serial: SharedSerial,
    upload_in_flight: UploadInFlight,
) -> Result<(), Error> {
    match handle_connection(peer, stream, configuration, serial, upload_in_flight).await {
        Ok(_) => Ok(()),
        Err(e) => match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8(_) => Ok(()),
            err => {
                error!("Error processing connection: {}", err);
                Err(err)
            }
        },
    }
}

/**
 * Get stream message and validate it and send back command
 * @param peer: SocketAddr, peer address
 * @param stream: TcpStream, stream from client
 * @param configuration: Config, configuration for the server
 * @return Result<(), Error>, return Ok(())
 * @throws Error
 */
async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    configuration: Config,
    serial: SharedSerial,
    upload_in_flight: UploadInFlight,
) -> Result<(), Error> {
    let ws_stream = accept_async(stream)
        .await
        .expect("Failed to accept incoming connection");

    // Socket addresses can be validated to insure only valide peers can connect and send commands
    info!("New client | {}", peer);
    let (mut ws_write, mut ws_read) = ws_stream.split();

    // Broadcast response message to clients
    async fn send_message_back(
        message: MessageSender,
        ws_write: &mut futures::prelude::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<TcpStream>,
            Message,
        >,
    ) -> Result<()> {
        let json_str =
            serde_json::to_string(&message).expect("Failed to serialize message into JSON");
        let resp_message = Message::Text(json_str.into());

        if let Err(e) = ws_write.send(resp_message).await {
            error!("{:?}", e)
        }

        Ok(())
    }

    // Auth handshake: when an auth_token is configured, the very first
    // message must be MessageType::Auth with the matching secret. The reply
    // is an "Auth" MessageSender ("ok" or "fail") and on failure we drop
    // the connection. When no token is configured this block is skipped.
    if let Some(expected) = configuration.auth_token.as_deref() {
        let first = match ws_read.next().await {
            Some(Ok(m)) => m,
            Some(Err(e)) => {
                warn!("Auth read error from {}: {}", peer, e);
                return Ok(());
            }
            None => {
                info!("Client {} disconnected before auth", peer);
                return Ok(());
            }
        };

        let auth_ok = first
            .to_text()
            .ok()
            .and_then(|t| serde_json::from_str::<MessageWS>(t).ok())
            .map(|m| matches!(m.message_type, MessageType::Auth) && m.message == expected)
            .unwrap_or(false);

        let reply = MessageSender {
            message_type: "Auth".to_string(),
            message: if auth_ok { "ok".into() } else { "fail".into() },
            raw_message: String::new(),
            timestamp: now_ts(),
        };
        send_message_back(reply, &mut ws_write).await?;

        if !auth_ok {
            warn!("Auth failed for {}", peer);
            return Ok(());
        }
        info!("Auth ok for {}", peer);
    }

    // Loop over received messages
    while let Some(msg) = ws_read.next().await {
        let msg = msg?;

        // can also check for binary values
        if msg.is_text() && !msg.is_empty() {
            // The data is directly going to the serial_com.
            // Parse and validate the commands.
            let data = msg.to_text()?;

            match serde_json::from_str::<MessageWS>(data) {
                Ok(message) => {
                    info!("Message received: {}", message.message);

                    // Starting timestamp
                    let now = SystemTime::now();

                    match message.message_type {
                        MessageType::GCommand => {
                            debug!("Config: {}", message.message);
                            let result = g_command(message.message);
                            match result {
                                Ok(cmd) => {
                                    let serial_clone = Arc::clone(&serial);
                                    let cmd_owned = cmd.to_string();
                                    let join_result = tokio::task::spawn_blocking(move || {
                                        serial_clone.lock().unwrap().send_command(&cmd_owned)
                                    })
                                    .await;

                                    match join_result {
                                        Ok(Ok(response)) => {
                                            debug!("{:?}", response);

                                            // Set timestamp
                                            let since_epoch = now
                                                .duration_since(UNIX_EPOCH)
                                                .expect("Time went backwards");
                                            let timestamp = since_epoch.as_secs();

                                            // Define response message
                                            let mut message_sender = MessageSender {
                                                message_type: cmd.to_string(),
                                                message: "".to_string(),
                                                raw_message: response.clone(),
                                                timestamp,
                                            };

                                            if &response != "ok" {
                                                message_sender.message = match cmd.trim() {
                                                    "M20" => {
                                                        let response = m20(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M27" => {
                                                        let response = m27(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M27 C" => {
                                                        let response = m27(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M31" => {
                                                        let response = m31(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M33" => {
                                                        let response = m33(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M105" => {
                                                        let response = m105(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M114" => {
                                                        let response = m114(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M115" => {
                                                        let response = m115(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    "M119" => {
                                                        let response = m119(response);
                                                        serde_json::to_string(&response).expect(
                                                            "Failed to serialize message into JSON",
                                                        )
                                                    }
                                                    _ => response.to_string(),
                                                };
                                            }
                                            send_message_back(message_sender, &mut ws_write)
                                                .await?;
                                        }
                                        Ok(Err(io_err)) => {
                                            error!("Serial IO error: {}", io_err);
                                        }
                                        Err(join_err) => {
                                            error!("Serial blocking task failed: {}", join_err);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("{:?}", e)
                                }
                            }
                        }
                        MessageType::SerialConfig => {
                            // Not yet implemented, changes to the config loading is required
                            debug!("SerialConfig: {}", message.message);
                        }
                        MessageType::Auth => {
                            // Auth is only valid as the first message; ignore
                            // (and don't echo the value) if a client re-sends.
                            warn!("Unexpected Auth message from {} after handshake", peer);
                            send_message_back(
                                error_message(
                                    "MessageSenderError",
                                    "Auth not allowed after handshake",
                                ),
                                &mut ws_write,
                            )
                            .await?;
                        }
                        MessageType::Terminal => {
                            // Validate against the same allow-list as GCommand —
                            // Terminal used to bypass it entirely, which made the
                            // whitelist meaningless.
                            let cmd = match g_command(message.message) {
                                Ok(c) => c,
                                Err(e) => {
                                    warn!("Terminal rejected from {}: {}", peer, e);
                                    send_message_back(
                                        error_message(
                                            "MessageSenderError",
                                            "Invalid or disallowed command",
                                        ),
                                        &mut ws_write,
                                    )
                                    .await?;
                                    continue;
                                }
                            };
                            let serial_clone = Arc::clone(&serial);
                            let cmd_owned = cmd.to_string();
                            let join_result = tokio::task::spawn_blocking(move || {
                                serial_clone.lock().unwrap().send_command(&cmd_owned)
                            })
                            .await;

                            match join_result {
                                Ok(Ok(response)) => {
                                    debug!("{:?}", response);

                                    // Get timestamp
                                    let since_epoch = now
                                        .duration_since(UNIX_EPOCH)
                                        .expect("Time went backwards");
                                    let timestamp = since_epoch.as_secs();

                                    let message_sender = MessageSender {
                                        message_type: "terminal".to_string(),
                                        message: response.clone(),
                                        raw_message: response,
                                        timestamp,
                                    };

                                    send_message_back(message_sender, &mut ws_write).await?;
                                }
                                Ok(Err(io_err)) => {
                                    error!("Serial IO error: {}", io_err);
                                    send_message_back(
                                        error_message(
                                            "MessageSenderError",
                                            "Error executing command",
                                        ),
                                        &mut ws_write,
                                    )
                                    .await?;
                                }
                                Err(join_err) => {
                                    error!("Serial blocking task failed: {}", join_err);
                                    send_message_back(
                                        error_message(
                                            "MessageSenderError",
                                            "Error executing command",
                                        ),
                                        &mut ws_write,
                                    )
                                    .await?;
                                }
                            }
                        }
                        MessageType::Unsafe => {
                            // Same validation as Terminal/GCommand: do not let
                            // a client send arbitrary text straight to Marlin.
                            let cmd = match g_command(message.message) {
                                Ok(c) => c,
                                Err(e) => {
                                    warn!("Unsafe rejected from {}: {}", peer, e);
                                    send_message_back(
                                        error_message(
                                            "MessageSenderError",
                                            "Invalid or disallowed command",
                                        ),
                                        &mut ws_write,
                                    )
                                    .await?;
                                    continue;
                                }
                            };
                            let serial_clone = Arc::clone(&serial);
                            let cmd_owned = cmd.to_string();
                            let join_result = tokio::task::spawn_blocking(move || {
                                serial_clone.lock().unwrap().send_command(&cmd_owned)
                            })
                            .await;

                            match join_result {
                                Ok(Ok(response)) => {
                                    debug!("{:?}", response);

                                    // Get timestamp
                                    let since_epoch = now
                                        .duration_since(UNIX_EPOCH)
                                        .expect("Time went backwards");
                                    let timestamp = since_epoch.as_secs();

                                    let message_sender = MessageSender {
                                        message_type: "Unsafe".to_string(),
                                        message: response.clone(),
                                        raw_message: response,
                                        timestamp,
                                    };

                                    send_message_back(message_sender, &mut ws_write).await?;
                                }
                                Ok(Err(io_err)) => {
                                    error!("Serial IO error: {}", io_err);
                                    send_message_back(
                                        error_message(
                                            "MessageSenderError",
                                            "Error executing command",
                                        ),
                                        &mut ws_write,
                                    )
                                    .await?;
                                }
                                Err(join_err) => {
                                    error!("Serial blocking task failed: {}", join_err);
                                    send_message_back(
                                        error_message(
                                            "MessageSenderError",
                                            "Error executing command",
                                        ),
                                        &mut ws_write,
                                    )
                                    .await?;
                                }
                            }
                        }
                        MessageType::UploadBegin => {
                            let req: UploadRequest = match serde_json::from_str(message.message) {
                                Ok(r) => r,
                                Err(e) => {
                                    warn!("UploadBegin bad JSON from {}: {}", peer, e);
                                    send_message_back(
                                        error_message("UploadError", "invalid UploadRequest JSON"),
                                        &mut ws_write,
                                    )
                                    .await?;
                                    continue;
                                }
                            };
                            if !valid_dest_filename(&req.dest_filename) {
                                send_message_back(
                                    error_message(
                                        "UploadError",
                                        "invalid dest_filename: must have no /, no NUL, <=63 chars, end in .gco/.gcode/.g",
                                    ),
                                    &mut ws_write,
                                )
                                .await?;
                                continue;
                            }
                            if req.size == 0 || req.size > configuration.max_upload_bytes {
                                send_message_back(
                                    error_message(
                                        "UploadError",
                                        &format!(
                                            "size {} out of range (1..={})",
                                            req.size, configuration.max_upload_bytes
                                        ),
                                    ),
                                    &mut ws_write,
                                )
                                .await?;
                                continue;
                            }
                            let compression = match resolve_compression(req.compression.as_deref())
                            {
                                Ok(c) => c,
                                Err(reason) => {
                                    send_message_back(
                                        error_message("UploadError", &reason),
                                        &mut ws_write,
                                    )
                                    .await?;
                                    continue;
                                }
                            };
                            // Single-flight: only one upload at a time. Same
                            // serial port, same mutex — concurrent uploads
                            // would just block on each other, so reject fast
                            // with a clear error instead.
                            if upload_in_flight
                                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                                .is_err()
                            {
                                send_message_back(
                                    error_message(
                                        "UploadError",
                                        "another upload is already in progress",
                                    ),
                                    &mut ws_write,
                                )
                                .await?;
                                continue;
                            }
                            // From this point on, the flag is owned by the
                            // guard — any exit path (fall-through, continue,
                            // or `?` propagation) resets it on drop.
                            let _flag_guard = InFlightGuard(Arc::clone(&upload_in_flight));
                            warn!(
                                "Upload starting (peer={}, file={}, size={}); serial commands from other clients will block until completion",
                                peer, req.dest_filename, req.size
                            );
                            send_message_back(
                                MessageSender {
                                    message_type: "UploadAck".to_string(),
                                    message: "ready".to_string(),
                                    raw_message: String::new(),
                                    timestamp: now_ts(),
                                },
                                &mut ws_write,
                            )
                            .await?;

                            // Collect binary frames until we've received `size` bytes.
                            // The idle deadline only advances on actual payload
                            // frames, so a client that sends nothing but
                            // ping/pong keep-alives still gets timed out and
                            // can't pin the single-flight slot indefinitely.
                            let mut buf: Vec<u8> = Vec::with_capacity(req.size as usize);
                            let mut aborted = false;
                            let mut idle_deadline =
                                tokio::time::Instant::now() + UPLOAD_IDLE_TIMEOUT;
                            while (buf.len() as u64) < req.size {
                                let frame = match tokio::time::timeout_at(
                                    idle_deadline,
                                    ws_read.next(),
                                )
                                .await
                                {
                                    Ok(Some(Ok(m))) => m,
                                    Ok(Some(Err(e))) => {
                                        warn!("WS read error during upload payload: {}", e);
                                        aborted = true;
                                        break;
                                    }
                                    Ok(None) => {
                                        warn!("Connection closed mid-upload");
                                        aborted = true;
                                        break;
                                    }
                                    Err(_elapsed) => {
                                        warn!(
                                            "Upload payload stalled (no data for {:?}) from {}, aborting",
                                            UPLOAD_IDLE_TIMEOUT, peer
                                        );
                                        send_message_back(
                                            error_message(
                                                "UploadError",
                                                "upload timed out waiting for payload data",
                                            ),
                                            &mut ws_write,
                                        )
                                        .await?;
                                        aborted = true;
                                        break;
                                    }
                                };
                                if frame.is_close() {
                                    aborted = true;
                                    break;
                                }
                                if frame.is_ping() || frame.is_pong() {
                                    // Keep-alive traffic deliberately does not
                                    // refresh `idle_deadline` — only real
                                    // payload progress does.
                                    continue;
                                }
                                if !frame.is_binary() {
                                    send_message_back(
                                        error_message(
                                            "UploadError",
                                            "expected binary frame during upload payload",
                                        ),
                                        &mut ws_write,
                                    )
                                    .await?;
                                    aborted = true;
                                    break;
                                }
                                let data = frame.into_data();
                                let remaining = req.size - buf.len() as u64;
                                if data.len() as u64 > remaining {
                                    send_message_back(
                                        error_message(
                                            "UploadError",
                                            "payload exceeded declared size",
                                        ),
                                        &mut ws_write,
                                    )
                                    .await?;
                                    aborted = true;
                                    break;
                                }
                                buf.extend_from_slice(&data);
                                idle_deadline = tokio::time::Instant::now() + UPLOAD_IDLE_TIMEOUT;
                            }
                            if aborted {
                                continue;
                            }

                            // Build options + mpsc-backed progress callback.
                            let (prog_tx, mut prog_rx) =
                                tokio::sync::mpsc::channel::<UploadProgress>(64);
                            let dummy = req.dummy.unwrap_or(false);
                            let chunk_size = req.chunk_size.unwrap_or(0);
                            let dest_filename = req.dest_filename.clone();
                            let opts = UploadOptions {
                                dest_filename,
                                compression,
                                dummy,
                                chunk_size,
                                progress: Some(Box::new(move |p| {
                                    // try_send so the serial loop never blocks on
                                    // backpressure; dropping an event is fine.
                                    let _ = prog_tx.try_send(UploadProgress {
                                        bytes_sent: p.bytes_sent,
                                        chunks_sent: p.chunks_sent,
                                        source_bytes: p.source_bytes,
                                    });
                                })),
                            };

                            // Run the upload on a blocking thread while
                            // forwarding progress events from this async
                            // task. The lock is held for the entire upload
                            // duration — other clients' serial commands queue.
                            let serial_for_task = Arc::clone(&serial);
                            let mut upload_handle = tokio::task::spawn_blocking(
                                move || -> std::result::Result<UploadStats, UploadError> {
                                    let mut conn = serial_for_task.lock().unwrap();
                                    conn.with_short_read_timeout(
                                        Duration::from_millis(100),
                                        |port| binary_upload(port, Cursor::new(buf), opts),
                                    )
                                },
                            );

                            let upload_result = loop {
                                tokio::select! {
                                    Some(prog) = prog_rx.recv() => {
                                        let body = serde_json::to_string(&prog)
                                            .unwrap_or_else(|_| String::from("{}"));
                                        send_message_back(
                                            MessageSender {
                                                message_type: "UploadProgress".to_string(),
                                                message: body,
                                                raw_message: String::new(),
                                                timestamp: now_ts(),
                                            },
                                            &mut ws_write,
                                        )
                                        .await?;
                                    }
                                    join = &mut upload_handle => {
                                        // Drain any progress events buffered
                                        // before the task finished.
                                        while let Ok(prog) = prog_rx.try_recv() {
                                            let body = serde_json::to_string(&prog)
                                                .unwrap_or_else(|_| String::from("{}"));
                                            send_message_back(
                                                MessageSender {
                                                    message_type: "UploadProgress".to_string(),
                                                    message: body,
                                                    raw_message: String::new(),
                                                    timestamp: now_ts(),
                                                },
                                                &mut ws_write,
                                            )
                                            .await?;
                                        }
                                        break join;
                                    }
                                }
                            };

                            match upload_result {
                                Ok(Ok(stats)) => {
                                    let result_payload = UploadResult {
                                        source_bytes: stats.source_bytes,
                                        bytes_sent: stats.bytes_sent,
                                        chunks_sent: stats.chunks_sent,
                                        compression: compression_label(&stats.compression)
                                            .to_string(),
                                    };
                                    let body = serde_json::to_string(&result_payload)
                                        .unwrap_or_else(|_| String::from("{}"));
                                    send_message_back(
                                        MessageSender {
                                            message_type: "UploadDone".to_string(),
                                            message: body,
                                            raw_message: String::new(),
                                            timestamp: now_ts(),
                                        },
                                        &mut ws_write,
                                    )
                                    .await?;
                                }
                                Ok(Err(upload_err)) => {
                                    error!("Upload failed: {}", upload_err);
                                    send_message_back(
                                        error_message("UploadError", &upload_err.to_string()),
                                        &mut ws_write,
                                    )
                                    .await?;
                                }
                                Err(join_err) => {
                                    error!("Upload blocking task panicked: {}", join_err);
                                    send_message_back(
                                        error_message("UploadError", "upload task crashed"),
                                        &mut ws_write,
                                    )
                                    .await?;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Bad JSON from {}: {}", peer, e);
                    send_message_back(
                        error_message("MessageSenderError", "Invalid JSON payload"),
                        &mut ws_write,
                    )
                    .await?;
                }
            }
        }
    }

    info!("Connection lost for {}", peer);
    Err(Error::ConnectionClosed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_dest_filename_accepts_canonical_extensions() {
        assert!(valid_dest_filename("part.gco"));
        assert!(valid_dest_filename("PART.GCODE"));
        assert!(valid_dest_filename("a.g"));
        assert!(valid_dest_filename("Some_File-01.GCo"));
    }

    #[test]
    fn valid_dest_filename_rejects_bad_inputs() {
        assert!(!valid_dest_filename(""), "empty rejected");
        assert!(!valid_dest_filename("noext"), "no extension rejected");
        assert!(!valid_dest_filename("foo.txt"), "wrong extension rejected");
        assert!(!valid_dest_filename("foo/bar.gco"), "slash rejected");
        assert!(
            !valid_dest_filename("nul\0byte.gco"),
            "embedded NUL rejected"
        );
        let too_long = format!("{}.gco", "x".repeat(60)); // 60 + 4 = 64 chars
        assert!(!valid_dest_filename(&too_long), ">63 chars rejected");
    }

    #[test]
    fn valid_dest_filename_accepts_63_char_edge() {
        let name = format!("{}.gco", "x".repeat(59)); // 59 + 4 = 63 chars
        assert_eq!(name.len(), 63);
        assert!(valid_dest_filename(&name));
    }

    #[test]
    fn resolve_compression_none_always_works() {
        assert!(matches!(
            resolve_compression(Some("none")),
            Ok(Compression::None)
        ));
        assert!(matches!(resolve_compression(None), Ok(Compression::None)));
    }

    #[test]
    fn resolve_compression_unknown_errors() {
        assert!(resolve_compression(Some("lzma")).is_err());
        assert!(resolve_compression(Some("")).is_err());
    }

    #[cfg(feature = "heatshrink")]
    #[test]
    fn resolve_compression_heatshrink_modes_with_feature() {
        assert!(matches!(
            resolve_compression(Some("heatshrink")),
            Ok(Compression::Heatshrink { .. })
        ));
        assert!(matches!(
            resolve_compression(Some("auto")),
            Ok(Compression::Auto)
        ));
    }

    #[cfg(not(feature = "heatshrink"))]
    #[test]
    fn resolve_compression_heatshrink_modes_without_feature() {
        assert!(resolve_compression(Some("heatshrink")).is_err());
        assert!(resolve_compression(Some("auto")).is_err());
    }

    #[test]
    fn compression_label_maps_every_variant() {
        // The `Compression` variants exist regardless of the `heatshrink`
        // feature, so the label mapping is exercised on both feature sets.
        assert_eq!(compression_label(&Compression::None), "none");
        assert_eq!(
            compression_label(&Compression::Heatshrink {
                window: 8,
                lookahead: 4
            }),
            "heatshrink"
        );
        assert_eq!(compression_label(&Compression::Auto), "auto");
    }
}
