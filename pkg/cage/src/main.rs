//! cage daemon — per-agent microVM manager for NullBox.
//!
//! On startup:
//! 1. Verifies KVM is available
//! 2. Scans /agent/*.toml for agent manifests
//! 3. Listens on a Unix socket for agent lifecycle commands
//!
//! v0.1: No libkrun yet — validates manifests and waits for commands.

use cage::manifest;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::Path;

const SOCKET_PATH: &str = "/run/cage.sock";
const AGENT_DIR: &str = "/agent";

fn main() {
    let result = run();
    if let Err(e) = result {
        log(&format!("cage: fatal: {e}"));
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    log("cage: starting microVM manager");

    // Check for KVM support
    if Path::new("/dev/kvm").exists() {
        log("cage: KVM available");
    } else {
        log("cage: WARNING — /dev/kvm not found, microVMs will not work");
    }

    // Scan for agent manifests
    let agent_dir = Path::new(AGENT_DIR);
    if agent_dir.is_dir() {
        let mut count = 0;
        for entry in std::fs::read_dir(agent_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "toml") {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match manifest::parse(&content) {
                        Ok(m) => {
                            log(&format!(
                                "cage: found agent '{}' v{}",
                                m.agent.name, m.agent.version
                            ));
                            count += 1;
                        }
                        Err(e) => {
                            log(&format!(
                                "cage: invalid manifest {}: {e}",
                                path.display()
                            ));
                        }
                    },
                    Err(e) => {
                        log(&format!(
                            "cage: cannot read {}: {e}",
                            path.display()
                        ));
                    }
                }
            }
        }
        log(&format!("cage: {count} agent manifest(s) loaded"));
    } else {
        log("cage: no /agent directory, no manifests to load");
    }

    // Clean up stale socket
    let sock_path = Path::new(SOCKET_PATH);
    if sock_path.exists() {
        std::fs::remove_file(sock_path)?;
    }

    // Listen for lifecycle commands
    let listener = UnixListener::bind(SOCKET_PATH)?;
    log(&format!("cage: listening on {SOCKET_PATH}"));

    for stream in listener.incoming() {
        match stream {
            Ok(_conn) => {
                log("cage: received connection (no-op in v0.1)");
            }
            Err(e) => {
                log(&format!("cage: accept error: {e}"));
            }
        }
    }

    Ok(())
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
