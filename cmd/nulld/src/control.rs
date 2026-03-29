//! Control socket for nulld.
//!
//! Listens on /run/nulld.sock for JSON commands from nullctl.
//! Non-blocking: polled from the main supervisor loop.
//!
//! Protocol (newline-delimited JSON):
//!   Request:  {"method": "status"}
//!   Response: {"services": [...]}
//!
//!   Request:  {"method": "shutdown"}
//!   Response: {"ok": true}

use crate::supervisor::{ServiceStatus, Supervisor};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

const SOCKET_PATH: &str = "/run/nulld.sock";

#[derive(Debug, Deserialize)]
struct Request {
    method: String,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    services: Vec<ServiceStatus>,
}

#[derive(Debug, Serialize)]
struct OkResponse {
    ok: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// The control socket listener, set to non-blocking mode.
pub struct ControlSocket {
    listener: UnixListener,
}

impl ControlSocket {
    /// Create and bind the control socket.
    pub fn bind() -> Result<Self, std::io::Error> {
        let path = Path::new(SOCKET_PATH);
        if path.exists() {
            std::fs::remove_file(path)?;
        }

        let listener = UnixListener::bind(path)?;
        listener.set_nonblocking(true)?;

        crate::log_kmsg(&format!("nulld: control socket listening on {SOCKET_PATH}"));

        Ok(Self { listener })
    }

    /// Poll for incoming connections and handle them.
    /// Called once per main loop tick. Non-blocking.
    pub fn poll(&self, supervisor: &Supervisor, shutdown_flag: &AtomicBool) {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    self.handle_connection(stream, supervisor, shutdown_flag);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No pending connections
                    return;
                }
                Err(e) => {
                    crate::log_kmsg(&format!("nulld: control socket accept error: {e}"));
                    return;
                }
            }
        }
    }

    fn handle_connection(
        &self,
        stream: std::os::unix::net::UnixStream,
        supervisor: &Supervisor,
        shutdown_flag: &AtomicBool,
    ) {
        // Set a short timeout so we don't block the main loop
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(100)));
        let _ = stream.set_write_timeout(Some(std::time::Duration::from_millis(100)));

        let reader = BufReader::new(&stream);

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => return,
            };

            if line.is_empty() {
                continue;
            }

            let request: Request = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let _ = send_response(
                        &stream,
                        &ErrorResponse {
                            error: format!("invalid request: {e}"),
                        },
                    );
                    return;
                }
            };

            match request.method.as_str() {
                "status" => {
                    let status = supervisor.status();
                    let _ = send_response(&stream, &StatusResponse { services: status });
                }
                "shutdown" => {
                    crate::log_kmsg("nulld: shutdown requested via control socket");
                    shutdown_flag.store(true, Ordering::SeqCst);
                    let _ = send_response(&stream, &OkResponse { ok: true });
                }
                other => {
                    let _ = send_response(
                        &stream,
                        &ErrorResponse {
                            error: format!("unknown method: {other}"),
                        },
                    );
                }
            }
        }
    }
}

impl Drop for ControlSocket {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}

fn send_response<T: Serialize>(
    mut stream: &std::os::unix::net::UnixStream,
    response: &T,
) -> Result<(), std::io::Error> {
    let json = serde_json::to_string(response)?;
    writeln!(stream, "{json}")?;
    stream.flush()
}
