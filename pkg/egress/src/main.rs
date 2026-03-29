//! egress daemon — default-deny nftables network controller for NullBox.
//!
//! On startup:
//! 1. Generates the base default-deny ruleset (no agents yet)
//! 2. Applies it via `nft -f`
//! 3. Listens on a Unix socket for agent add/remove commands
//! 4. Regenerates and reapplies rules atomically on each change

use egress::firewall;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::Command;

const SOCKET_PATH: &str = "/run/egress.sock";
const NFT_RULES_PATH: &str = "/run/egress-rules.nft";

fn main() {
    let result = run();
    if let Err(e) = result {
        log(&format!("egress: fatal: {e}"));
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    log("egress: starting default-deny network controller");

    // Generate base ruleset with no agent rules
    let ruleset = firewall::generate_ruleset(&[]);

    // Write ruleset to file
    std::fs::write(NFT_RULES_PATH, &ruleset.content)?;
    log("egress: wrote base ruleset");

    // Apply via nft (skip if nft not available — e.g., in QEMU without nftables)
    match apply_ruleset() {
        Ok(()) => log("egress: nftables rules applied"),
        Err(e) => log(&format!("egress: nft apply skipped: {e}")),
    }

    // Clean up stale socket
    let sock_path = Path::new(SOCKET_PATH);
    if sock_path.exists() {
        std::fs::remove_file(sock_path)?;
    }

    // Listen for commands
    let listener = UnixListener::bind(SOCKET_PATH)?;
    log(&format!("egress: listening on {SOCKET_PATH}"));

    // Block on accept — nulld will send SIGTERM to shut us down
    for stream in listener.incoming() {
        match stream {
            Ok(_conn) => {
                // v0.1: log and ignore — agent rule management comes later
                log("egress: received connection (no-op in v0.1)");
            }
            Err(e) => {
                log(&format!("egress: accept error: {e}"));
            }
        }
    }

    Ok(())
}

fn apply_ruleset() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("nft")
        .args(["-f", NFT_RULES_PATH])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("nft failed: {stderr}").into());
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
