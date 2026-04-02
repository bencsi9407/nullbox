//! nullctl — CLI for NullBox.
//!
//! Communicates with nulld, cage, sentinel, watcher, and egress via Unix sockets.

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process;

const NULLD_SOCKET: &str = "/run/nulld.sock";
const CAGE_SOCKET: &str = "/run/cage.sock";
const WARDEN_SOCKET: &str = "/run/warden.sock";
const SENTINEL_SOCKET: &str = "/run/sentinel.sock";
const WATCHER_SOCKET: &str = "/run/watcher.sock";
const EGRESS_SOCKET: &str = "/run/egress.sock";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        process::exit(1);
    }

    let result = match args[0].as_str() {
        "status" => handle_status(),
        "shutdown" => handle_shutdown(),
        "cage" => handle_cage(&args[1..]),
        "vault" => handle_vault(&args[1..]),
        "deploy" => handle_deploy(&args[1..]),
        "sentinel" => handle_sentinel(&args[1..]),
        "watcher" => handle_watcher(&args[1..]),
        "egress" => handle_egress(&args[1..]),
        "logs" => handle_logs(&args[1..]),
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
    let response = send_nulld_request("status")?;
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
    let response = send_nulld_request("shutdown")?;
    let parsed: serde_json::Value = serde_json::from_str(&response)?;

    if parsed.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        println!("nulld: shutdown initiated");
    } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
        eprintln!("nulld: {err}");
    }

    Ok(())
}

fn handle_cage(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("nullctl cage: missing subcommand");
        eprintln!();
        eprintln!("usage:");
        eprintln!("  nullctl cage list                List running agent VMs");
        eprintln!("  nullctl cage start <agent>       Start an agent microVM");
        eprintln!("  nullctl cage stop <agent>        Stop an agent microVM");
        process::exit(1);
    }

    match args[0].as_str() {
        "list" => {
            let resp = send_cage_request(serde_json::json!({"method": "list"}))?;
            let parsed: serde_json::Value = serde_json::from_str(&resp)?;

            if let Some(agents) = parsed.get("agents").and_then(|a| a.as_array()) {
                if agents.is_empty() {
                    println!("no running agents");
                } else {
                    println!("{:<20} {:<10} {:<8} {}", "AGENT", "PID", "VCPUS", "RAM_MIB");
                    for vm in agents {
                        let name = vm.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let pid = vm.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                        let vcpus = vm.get("vcpus").and_then(|v| v.as_u64()).unwrap_or(0);
                        let ram = vm.get("ram_mib").and_then(|v| v.as_u64()).unwrap_or(0);
                        println!("{:<20} {:<10} {:<8} {}", name, pid, vcpus, ram);
                    }
                }
            } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                eprintln!("cage: {err}");
            }
        }
        "start" => {
            let name = args.get(1).ok_or("usage: nullctl cage start <agent>")?;
            let resp = send_cage_request(serde_json::json!({"method": "start", "agent": name}))?;
            let parsed: serde_json::Value = serde_json::from_str(&resp)?;

            if parsed.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                let pid = parsed.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("cage: started '{name}' (PID {pid})");
            } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                eprintln!("cage: {err}");
                process::exit(1);
            }
        }
        "stop" => {
            let name = args.get(1).ok_or("usage: nullctl cage stop <agent>")?;
            let resp = send_cage_request(serde_json::json!({"method": "stop", "agent": name}))?;
            let parsed: serde_json::Value = serde_json::from_str(&resp)?;

            if parsed.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                println!("cage: stopped '{name}'");
            } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                eprintln!("cage: {err}");
                process::exit(1);
            }
        }
        other => {
            eprintln!("nullctl cage: unknown subcommand '{other}'");
            process::exit(1);
        }
    }

    Ok(())
}

fn handle_vault(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("nullctl vault: missing subcommand");
        eprintln!();
        eprintln!("usage:");
        eprintln!("  nullctl vault list                  List stored secret keys");
        eprintln!("  nullctl vault set <KEY> <VALUE>      Set a secret");
        eprintln!("  nullctl vault delete <KEY>            Delete a secret");
        process::exit(1);
    }

    match args[0].as_str() {
        "list" => {
            let resp = send_to_socket(WARDEN_SOCKET, &serde_json::json!({"method": "list"}))?;
            let parsed: serde_json::Value = serde_json::from_str(&resp)?;

            if let Some(keys) = parsed.get("keys").and_then(|k| k.as_array()) {
                if keys.is_empty() {
                    println!("no secrets stored");
                } else {
                    println!("{:<30} {}", "KEY", "STATUS");
                    for key in keys {
                        if let Some(name) = key.as_str() {
                            println!("{:<30} set", name);
                        }
                    }
                }
            } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                eprintln!("warden: {err}");
                process::exit(1);
            }
        }
        "set" => {
            let key = args.get(1).ok_or("usage: nullctl vault set <KEY> <VALUE>")?;
            let value = args.get(2).ok_or("usage: nullctl vault set <KEY> <VALUE>")?;
            let resp = send_to_socket(
                WARDEN_SOCKET,
                &serde_json::json!({"method": "set", "key": key, "value": value}),
            )?;
            let parsed: serde_json::Value = serde_json::from_str(&resp)?;

            if parsed.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                println!("warden: secret '{key}' set");
            } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                eprintln!("warden: {err}");
                process::exit(1);
            }
        }
        "delete" => {
            let key = args.get(1).ok_or("usage: nullctl vault delete <KEY>")?;
            let resp = send_to_socket(
                WARDEN_SOCKET,
                &serde_json::json!({"method": "delete", "key": key}),
            )?;
            let parsed: serde_json::Value = serde_json::from_str(&resp)?;

            if parsed.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                println!("warden: secret '{key}' deleted");
            } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                eprintln!("warden: {err}");
                process::exit(1);
            }
        }
        other => {
            eprintln!("nullctl vault: unknown subcommand '{other}'");
            process::exit(1);
        }
    }

    Ok(())
}

fn handle_deploy(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use base64::Engine;

    let bundle_path = args.first().ok_or("usage: nullctl deploy <bundle.tar.gz>")?;

    let bundle_data = std::fs::read(bundle_path)
        .map_err(|e| format!("cannot read bundle '{bundle_path}': {e}"))?;

    let bundle_b64 = base64::engine::general_purpose::STANDARD.encode(&bundle_data);

    let resp = send_cage_request(serde_json::json!({
        "method": "deploy",
        "bundle": bundle_b64,
    }))?;

    let parsed: serde_json::Value = serde_json::from_str(&resp)?;

    if parsed.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        let agent = parsed.get("agent").and_then(|a| a.as_str()).unwrap_or("?");
        println!("cage: deployed and started agent '{agent}'");
    } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
        eprintln!("cage: {err}");
        process::exit(1);
    }

    Ok(())
}

fn handle_sentinel(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("nullctl sentinel: missing subcommand");
        eprintln!();
        eprintln!("usage:");
        eprintln!("  nullctl sentinel stats            Show sentinel statistics");
        process::exit(1);
    }

    match args[0].as_str() {
        "stats" => {
            let resp = send_to_socket(
                SENTINEL_SOCKET,
                &serde_json::json!({"method": "stats"}),
            )?;
            println!("{resp}");
        }
        other => {
            eprintln!("nullctl sentinel: unknown subcommand '{other}'");
            process::exit(1);
        }
    }

    Ok(())
}

fn handle_watcher(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("nullctl watcher: missing subcommand");
        eprintln!();
        eprintln!("usage:");
        eprintln!("  nullctl watcher verify <agent>    Verify agent audit chain");
        eprintln!("  nullctl watcher query <agent>     Query recent audit entries");
        process::exit(1);
    }

    match args[0].as_str() {
        "verify" => {
            let agent = args.get(1).ok_or("usage: nullctl watcher verify <agent>")?;
            let resp = send_to_socket(
                WATCHER_SOCKET,
                &serde_json::json!({"method": "verify", "agent": agent}),
            )?;
            let parsed: serde_json::Value = serde_json::from_str(&resp)?;

            if parsed.get("valid").and_then(|v| v.as_bool()) == Some(true) {
                println!("watcher: audit chain for '{agent}' is valid");
            } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                eprintln!("watcher: {err}");
                process::exit(1);
            } else {
                eprintln!("watcher: audit chain for '{agent}' is INVALID");
                if let Some(detail) = parsed.get("detail").and_then(|d| d.as_str()) {
                    eprintln!("  detail: {detail}");
                }
                process::exit(1);
            }
        }
        "query" => {
            let agent = args.get(1).ok_or("usage: nullctl watcher query <agent>")?;
            let resp = send_to_socket(
                WATCHER_SOCKET,
                &serde_json::json!({"method": "query", "agent": agent}),
            )?;
            let parsed: serde_json::Value = serde_json::from_str(&resp)?;

            if let Some(entries) = parsed.get("entries").and_then(|e| e.as_array()) {
                if entries.is_empty() {
                    println!("no audit entries for '{agent}'");
                } else {
                    for entry in entries {
                        let action = entry.get("action").and_then(|a| a.as_str()).unwrap_or("?");
                        let outcome = entry.get("outcome").and_then(|o| o.as_str()).unwrap_or("?");
                        let detail = entry.get("detail").and_then(|d| d.as_str()).unwrap_or("");
                        let ts = entry.get("timestamp").and_then(|t| t.as_str()).unwrap_or("?");
                        println!("[{ts}] {action} {outcome} {detail}");
                    }
                }
            } else if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                eprintln!("watcher: {err}");
                process::exit(1);
            }
        }
        other => {
            eprintln!("nullctl watcher: unknown subcommand '{other}'");
            process::exit(1);
        }
    }

    Ok(())
}

fn handle_egress(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("nullctl egress: missing subcommand");
        eprintln!();
        eprintln!("usage:");
        eprintln!("  nullctl egress list               List active firewall rules");
        process::exit(1);
    }

    match args[0].as_str() {
        "list" => {
            let resp = send_to_socket(
                EGRESS_SOCKET,
                &serde_json::json!({"method": "list"}),
            )?;
            println!("{resp}");
        }
        other => {
            eprintln!("nullctl egress: unknown subcommand '{other}'");
            process::exit(1);
        }
    }

    Ok(())
}

fn handle_logs(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let agent = args.first().ok_or("usage: nullctl logs <agent>")?;
    let log_path = format!("/var/log/cage/{agent}.log");

    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("cannot read {log_path}: {e}"))?;

    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(50);

    for line in &lines[start..] {
        println!("{line}");
    }

    Ok(())
}

fn send_nulld_request(method: &str) -> Result<String, Box<dyn std::error::Error>> {
    let request = serde_json::json!({"method": method});
    send_to_socket(NULLD_SOCKET, &request)
}

fn send_cage_request(request: serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    send_to_socket(CAGE_SOCKET, &request)
}

fn send_to_socket(path: &str, request: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(path).map_err(|e| {
        format!("cannot connect to {path}: {e}")
    })?;

    writeln!(stream, "{}", serde_json::to_string(request)?)?;
    stream.shutdown(std::net::Shutdown::Write)?;

    let reader = BufReader::new(&stream);
    let line = reader
        .lines()
        .next()
        .ok_or(format!("no response from {path}"))??;

    Ok(line)
}

fn print_usage() {
    eprintln!("nullctl — NullBox CLI");
    eprintln!();
    eprintln!("usage:");
    eprintln!("  nullctl status                    Show service status");
    eprintln!("  nullctl shutdown                  Initiate clean shutdown");
    eprintln!("  nullctl cage list                 List running agent VMs");
    eprintln!("  nullctl cage start <agent>        Start an agent microVM");
    eprintln!("  nullctl cage stop <agent>         Stop an agent microVM");
    eprintln!("  nullctl deploy <bundle.tar.gz>    Deploy agent bundle to cage");
    eprintln!("  nullctl sentinel stats            Show sentinel statistics");
    eprintln!("  nullctl watcher verify <agent>    Verify agent audit chain");
    eprintln!("  nullctl watcher query <agent>     Query recent audit entries");
    eprintln!("  nullctl egress list               List active firewall rules");
    eprintln!("  nullctl logs <agent>              Tail agent console log");
    eprintln!("  nullctl vault list                List stored secret keys");
    eprintln!("  nullctl vault set <KEY> <VALUE>   Set a secret");
    eprintln!("  nullctl vault delete <KEY>        Delete a secret");
}
