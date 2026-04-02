//! test-harness — TCP-to-Unix-socket bridge for e2e testing.
//!
//! Runs as a NullBox service on TCP 9200. Accepts JSON commands that
//! specify a target service, forwards the request to its Unix socket,
//! and returns the response. This lets the external e2e test exercise
//! ALL services (warden, sentinel, watcher, egress, cage) via QEMU
//! TCP port forwarding.
//!
//! Protocol:
//!   -> {"service": "warden", "request": {"method": "list"}}
//!   <- {"ok": true, "response": {...}}
//!
//! Only included in dev builds (--production flag in build-squashfs.sh
//! excludes test-harness).

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::os::unix::net::UnixStream;

const LISTEN_ADDR: &str = "0.0.0.0:9200";

const SERVICES: &[(&str, &str)] = &[
    ("warden", "/run/warden.sock"),
    ("sentinel", "/run/sentinel.sock"),
    ("watcher", "/run/watcher.sock"),
    ("egress", "/run/egress.sock"),
    ("cage", "/run/cage.sock"),
    ("nulld", "/run/nulld.sock"),
];

fn main() {
    log("test-harness: starting on TCP 9200");

    let listener = match TcpListener::bind(LISTEN_ADDR) {
        Ok(l) => l,
        Err(e) => {
            log(&format!("test-harness: bind failed: {e}"));
            std::process::exit(1);
        }
    };

    log(&format!("test-harness: listening on {LISTEN_ADDR}"));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => log(&format!("test-harness: accept error: {e}")),
        }
    }
}

fn handle_connection(stream: std::net::TcpStream) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));

    let reader = BufReader::new(&stream);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => return,
        };

        let request: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let _ = respond(&stream, &serde_json::json!({"error": format!("invalid JSON: {e}")}));
                return;
            }
        };

        let service = request
            .get("service")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        let inner_request = request
            .get("request")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let response = match service {
            "ping" => serde_json::json!({"ok": true, "service": "test-harness"}),
            _ => forward_to_service(service, &inner_request),
        };

        if respond(&stream, &response).is_err() {
            return;
        }
    }
}

fn forward_to_service(service: &str, request: &serde_json::Value) -> serde_json::Value {
    let socket_path = match SERVICES.iter().find(|(name, _)| *name == service) {
        Some((_, path)) => *path,
        None => {
            return serde_json::json!({
                "error": format!("unknown service: {service}"),
                "available": SERVICES.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
            });
        }
    };

    // Connect to Unix socket
    let mut unix_stream = match UnixStream::connect(socket_path) {
        Ok(s) => s,
        Err(e) => {
            return serde_json::json!({
                "error": format!("connect to {service} ({socket_path}): {e}"),
            });
        }
    };

    let _ = unix_stream.set_write_timeout(Some(std::time::Duration::from_secs(2)));
    let _ = unix_stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));

    // Forward request
    let msg = serde_json::to_string(request).unwrap_or_default();
    if writeln!(unix_stream, "{msg}").is_err() {
        return serde_json::json!({"error": format!("write to {service} failed")});
    }
    let _ = unix_stream.flush();

    // Read response
    let mut reader = BufReader::new(&unix_stream);
    let mut response_line = String::new();
    match reader.read_line(&mut response_line) {
        Ok(0) => serde_json::json!({"error": "empty response from service"}),
        Ok(_) => {
            match serde_json::from_str::<serde_json::Value>(response_line.trim()) {
                Ok(v) => serde_json::json!({"ok": true, "service": service, "response": v}),
                Err(_) => serde_json::json!({"ok": true, "service": service, "raw": response_line.trim()}),
            }
        }
        Err(e) => serde_json::json!({"error": format!("read from {service}: {e}")}),
    }
}

fn respond(stream: &std::net::TcpStream, response: &serde_json::Value) -> std::io::Result<()> {
    let mut writer = stream;
    writeln!(writer, "{}", serde_json::to_string(response).unwrap_or_default())?;
    writer.flush()
}

fn log(msg: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/kmsg")
    {
        let _ = writeln!(f, "{msg}");
    } else {
        eprintln!("{msg}");
    }
}
