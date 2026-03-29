//! nullctl — CLI for NullBox.
//!
//! Communicates with nulld via Unix socket to manage agents and services.

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process;

const NULLD_SOCKET: &str = "/run/nulld.sock";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        process::exit(1);
    }

    let result = match args[0].as_str() {
        "status" => handle_status(),
        "shutdown" => handle_shutdown(),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        other => {
            eprintln!("nullctl: unknown command '{other}'");
            print_usage();
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("nullctl: error: {e}");
        process::exit(1);
    }
}

fn handle_status() -> Result<(), Box<dyn std::error::Error>> {
    let response = send_request("status")?;
    let parsed: serde_json::Value = serde_json::from_str(&response)?;

    if let Some(services) = parsed.get("services").and_then(|s| s.as_array()) {
        println!("{:<15} {:<12} {:<8} {}", "SERVICE", "STATE", "PID", "RESTARTS");
        for svc in services {
            let name = svc.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let state = svc.get("state").and_then(|v| v.as_str()).unwrap_or("?");
            let pid = svc
                .get("pid")
                .and_then(|v| v.as_u64())
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string());
            let restarts = svc.get("restart_count").and_then(|v| v.as_u64()).unwrap_or(0);
            println!("{:<15} {:<12} {:<8} {}", name, state, pid, restarts);
        }
    } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
        eprintln!("nulld: {err}");
    }

    Ok(())
}

fn handle_shutdown() -> Result<(), Box<dyn std::error::Error>> {
    let response = send_request("shutdown")?;
    let parsed: serde_json::Value = serde_json::from_str(&response)?;

    if parsed.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        println!("nulld: shutdown initiated");
    } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
        eprintln!("nulld: {err}");
    }

    Ok(())
}

fn send_request(method: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(NULLD_SOCKET).map_err(|e| {
        format!("cannot connect to nulld at {NULLD_SOCKET}: {e}")
    })?;

    let request = serde_json::json!({"method": method});
    writeln!(stream, "{}", serde_json::to_string(&request)?)?;
    stream.shutdown(std::net::Shutdown::Write)?;

    let reader = BufReader::new(&stream);
    let line = reader
        .lines()
        .next()
        .ok_or("no response from nulld")??;

    Ok(line)
}

fn print_usage() {
    eprintln!("nullctl — NullBox CLI");
    eprintln!();
    eprintln!("usage:");
    eprintln!("  nullctl status      Show service status");
    eprintln!("  nullctl shutdown    Initiate clean shutdown");
}
