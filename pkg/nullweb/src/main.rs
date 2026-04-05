//! nullweb — NullBox web dashboard and API proxy.
//!
//! Single binary HTTP server on TCP 8080. Serves an embedded HTML
//! dashboard and proxies API calls to NullBox services over Unix
//! sockets. No external web framework — raw `TcpListener` with
//! hand-rolled HTTP/1.1 parsing.

mod html;
mod routes;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

const LISTEN_ADDR: &str = "0.0.0.0:8080";
const MAX_BODY_SIZE: usize = 64 * 1024 * 1024; // 64 MB (agent bundles)

fn main() {
    log("nullweb: starting on TCP 8080");

    let listener = match TcpListener::bind(LISTEN_ADDR) {
        Ok(l) => l,
        Err(e) => {
            log(&format!("nullweb: bind failed: {e}"));
            std::process::exit(1);
        }
    };

    log(&format!("nullweb: listening on {LISTEN_ADDR}"));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(10)));
                let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(10)));
                handle_connection(stream);
            }
            Err(e) => log(&format!("nullweb: accept error: {e}")),
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP request parsing
// ---------------------------------------------------------------------------

struct HttpRequest {
    method: String,
    path: String,
    _headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn parse_request(stream: &TcpStream) -> Option<HttpRequest> {
    let mut reader = BufReader::new(stream);

    // Request line
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).ok()? == 0 {
        return None;
    }
    let parts: Vec<&str> = request_line.trim().splitn(3, ' ').collect();
    if parts.len() < 2 {
        return None;
    }
    let method = parts[0].to_uppercase();
    let path = parts[1].to_string();

    // Headers
    let mut headers = Vec::new();
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some((key, val)) = trimmed.split_once(':') {
            let key_lower = key.trim().to_lowercase();
            let val_trimmed = val.trim().to_string();
            if key_lower == "content-length" {
                content_length = val_trimmed.parse().unwrap_or(0);
            }
            headers.push((key_lower, val_trimmed));
        }
    }

    // Body
    let mut body = Vec::new();
    if content_length > 0 && content_length <= MAX_BODY_SIZE {
        body.resize(content_length, 0);
        let _ = reader.read_exact(&mut body);
    }

    Some(HttpRequest {
        method,
        path,
        _headers: headers,
        body,
    })
}

// ---------------------------------------------------------------------------
// Connection handler — route dispatch
// ---------------------------------------------------------------------------

fn handle_connection(mut stream: TcpStream) {
    let req = match parse_request(&stream) {
        Some(r) => r,
        None => {
            let _ = send_response(&mut stream, 400, "text/plain", b"Bad Request");
            return;
        }
    };

    match (req.method.as_str(), req.path.as_str()) {
        // Dashboard
        ("GET", "/") | ("GET", "/index.html") => {
            let _ = send_response(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                html::DASHBOARD_HTML.as_bytes(),
            );
        }

        // API routes
        ("GET", "/api/status") => send_json(&mut stream, routes::handle_status()),
        ("GET", "/api/agents") => send_json(&mut stream, routes::handle_agents()),
        ("POST", "/api/agents/deploy") => {
            send_json(&mut stream, routes::handle_agents_deploy(&req.body));
        }
        ("GET", "/api/sentinel/stats") => {
            send_json(&mut stream, routes::handle_sentinel_stats());
        }
        ("POST", "/api/sentinel/inspect") => {
            send_json(&mut stream, routes::handle_sentinel_inspect(&req.body));
        }
        ("GET", "/api/warden/list") => {
            send_json(&mut stream, routes::handle_warden_list());
        }
        ("GET", "/api/egress/list") => {
            send_json(&mut stream, routes::handle_egress_list());
        }
        ("GET", "/api/registry") => {
            send_json(&mut stream, routes::handle_registry());
        }
        ("POST", "/api/update/check") => {
            send_json(&mut stream, routes::handle_update_check());
        }
        ("POST", "/api/update/apply") => {
            send_json(&mut stream, routes::handle_update_apply());
        }

        // Parameterised routes: /api/watcher/:agent and /api/logs/:agent
        (method, path) => {
            if let Some(rest) = path.strip_prefix("/api/watcher/") {
                if method == "GET" {
                    if let Some(agent) = rest.strip_suffix("/verify") {
                        send_json(
                            &mut stream,
                            routes::handle_watcher_verify(agent),
                        );
                    } else {
                        send_json(
                            &mut stream,
                            routes::handle_watcher_log(rest),
                        );
                    }
                    return;
                }
            }

            if let Some(agent) = path.strip_prefix("/api/logs/") {
                if method == "GET" {
                    send_json(&mut stream, routes::handle_logs(agent));
                    return;
                }
            }

            let _ = send_response(&mut stream, 404, "text/plain", b"Not Found");
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP response helpers
// ---------------------------------------------------------------------------

fn send_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    let header = format!(
        "HTTP/1.1 {status} {status_text}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Access-Control-Allow-Origin: *\r\n\
         \r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn send_json(stream: &mut TcpStream, value: serde_json::Value) {
    let body = serde_json::to_vec(&value).unwrap_or_default();
    let _ = send_response(stream, 200, "application/json", &body);
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
