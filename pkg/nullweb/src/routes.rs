//! API route handlers for the NullBox web dashboard.
//!
//! Each handler maps an HTTP path to a Unix socket service call,
//! translating the HTTP request into a JSON message and returning
//! the service response as HTTP JSON.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

/// Service socket paths.
const SOCK_NULLD: &str = "/run/nulld.sock";
const SOCK_CAGE: &str = "/run/cage.sock";
const SOCK_SENTINEL: &str = "/run/sentinel.sock";
const SOCK_WATCHER: &str = "/run/watcher.sock";
const SOCK_WARDEN: &str = "/run/warden.sock";
const SOCK_EGRESS: &str = "/run/egress.sock";

// ---------------------------------------------------------------------------
// Unix socket helper
// ---------------------------------------------------------------------------

/// Send a JSON request to a Unix socket and return the JSON response.
/// Uses the same newline-delimited JSON protocol as all NullBox services.
fn proxy_to_service(
    socket_path: &str,
    request: &serde_json::Value,
) -> serde_json::Value {
    let mut stream = match UnixStream::connect(socket_path) {
        Ok(s) => s,
        Err(e) => {
            return serde_json::json!({
                "error": format!("connect {socket_path}: {e}")
            });
        }
    };

    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(2)));
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));

    let msg = serde_json::to_string(request).unwrap_or_default();
    if writeln!(stream, "{msg}").is_err() {
        return serde_json::json!({"error": format!("write to {socket_path} failed")});
    }
    let _ = stream.flush();

    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => serde_json::json!({"error": "empty response"}),
        Ok(_) => {
            serde_json::from_str::<serde_json::Value>(line.trim())
                .unwrap_or_else(|_| serde_json::json!({"raw": line.trim()}))
        }
        Err(e) => serde_json::json!({"error": format!("read: {e}")}),
    }
}

// ---------------------------------------------------------------------------
// Route handlers — each returns a JSON value
// ---------------------------------------------------------------------------

/// GET /api/status — aggregate status from nulld
pub fn handle_status() -> serde_json::Value {
    proxy_to_service(SOCK_NULLD, &serde_json::json!({"method": "status"}))
}

/// GET /api/agents — list agents from cage
pub fn handle_agents() -> serde_json::Value {
    proxy_to_service(SOCK_CAGE, &serde_json::json!({"method": "list"}))
}

/// POST /api/agents/deploy — deploy agent bundle via cage
pub fn handle_agents_deploy(body: &[u8]) -> serde_json::Value {
    // The body is the raw upload. For the socket protocol we
    // base64-encode the bundle and send it as a JSON field.
    let encoded = base64_encode(body);
    proxy_to_service(
        SOCK_CAGE,
        &serde_json::json!({"method": "deploy", "bundle_b64": encoded}),
    )
}

/// GET /api/sentinel/stats
pub fn handle_sentinel_stats() -> serde_json::Value {
    proxy_to_service(SOCK_SENTINEL, &serde_json::json!({"method": "stats"}))
}

/// POST /api/sentinel/inspect
pub fn handle_sentinel_inspect(body: &[u8]) -> serde_json::Value {
    let val: serde_json::Value =
        serde_json::from_slice(body).unwrap_or(serde_json::json!({}));
    proxy_to_service(SOCK_SENTINEL, &serde_json::json!({"method": "inspect", "payload": val}))
}

/// GET /api/watcher/:agent — audit log
pub fn handle_watcher_log(agent: &str) -> serde_json::Value {
    proxy_to_service(
        SOCK_WATCHER,
        &serde_json::json!({"method": "log", "agent": agent}),
    )
}

/// GET /api/watcher/:agent/verify — chain verification
pub fn handle_watcher_verify(agent: &str) -> serde_json::Value {
    proxy_to_service(
        SOCK_WATCHER,
        &serde_json::json!({"method": "verify", "agent": agent}),
    )
}

/// GET /api/warden/list — vault key names
pub fn handle_warden_list() -> serde_json::Value {
    proxy_to_service(SOCK_WARDEN, &serde_json::json!({"method": "list"}))
}

/// GET /api/egress/list — egress rules per agent
pub fn handle_egress_list() -> serde_json::Value {
    proxy_to_service(SOCK_EGRESS, &serde_json::json!({"method": "list"}))
}

/// GET /api/logs/:agent — agent console logs
pub fn handle_logs(agent: &str) -> serde_json::Value {
    proxy_to_service(
        SOCK_CAGE,
        &serde_json::json!({"method": "logs", "agent": agent, "lines": 50}),
    )
}

/// GET /api/registry — available agents
pub fn handle_registry() -> serde_json::Value {
    // Serve the embedded registry or read from /system/config/registry.json
    let registry_path = "/system/config/registry.json";
    if let Ok(content) = std::fs::read_to_string(registry_path) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
            return serde_json::json!({"ok": true, "registry": parsed});
        }
    }
    // Fallback: return a minimal registry
    serde_json::json!({
        "ok": true,
        "registry": {
            "version": 1,
            "agents": []
        }
    })
}

/// POST /api/update/check — check for available updates
pub fn handle_update_check() -> serde_json::Value {
    let current_version = env!("CARGO_PKG_VERSION");

    // Read the update manifest from a well-known URL (cached locally)
    // For v0.1, just report current version
    serde_json::json!({
        "ok": true,
        "current_version": current_version,
        "latest_version": current_version,
        "update_available": false,
        "note": "OTA updates require a persistent partition. Use install.sh to reflash for now."
    })
}

/// POST /api/update/apply — apply an OTA update
pub fn handle_update_apply() -> serde_json::Value {
    // For v0.1: download new SquashFS to overlay, mark for next boot
    // This requires:
    // 1. Persistent storage (overlay on ext4, not tmpfs)
    // 2. New SquashFS downloaded to /overlay/update/nullbox.squashfs
    // 3. On next boot, initramfs detects the update and uses the new image

    // Check if we have persistent storage
    let persistent = std::path::Path::new("/overlay/.nullbox-data").exists()
        || std::path::Path::new("/data/.nullbox-data").exists();

    if !persistent {
        return serde_json::json!({
            "ok": false,
            "error": "OTA updates require a persistent partition. Create one with: mkfs.ext4 -L nullbox-data /dev/sdX && touch /mnt/.nullbox-data"
        });
    }

    serde_json::json!({
        "ok": false,
        "error": "OTA update download not yet implemented. Reflash the ISO for now."
    })
}

// ---------------------------------------------------------------------------
// Minimal base64 encoder (avoid pulling in a crate)
// ---------------------------------------------------------------------------

const B64_CHARS: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(B64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn base64_encode_hello() {
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
    }

    #[test]
    fn base64_encode_padding() {
        assert_eq!(base64_encode(b"Hi"), "SGk=");
        assert_eq!(base64_encode(b"Hey"), "SGV5");
    }
}
