//! ctxgraph daemon — content-addressed shared agent memory for NullBox.
//!
//! On startup:
//! 1. Opens (or creates) the SQLite database at /var/lib/ctxgraph/db.sqlite
//! 2. Listens on a Unix socket for read/write queries
//! 3. Agents interact with ctxgraph through this socket

use ctxgraph::store::Store;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::Path;
#[allow(unused_imports)]
use std::io::Read;

const SOCKET_PATH: &str = "/run/ctxgraph.sock";
const DB_DIR: &str = "/var/lib/ctxgraph";
const DB_PATH: &str = "/var/lib/ctxgraph/db.sqlite";

fn main() {
    let result = run();
    if let Err(e) = result {
        log(&format!("ctxgraph: fatal: {e}"));
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    log("ctxgraph: starting shared agent memory");

    // Ensure database directory exists
    std::fs::create_dir_all(DB_DIR)?;

    // Open database
    let _store = Store::open(Path::new(DB_PATH))?;
    log("ctxgraph: database initialized");

    // Clean up stale socket
    let sock_path = Path::new(SOCKET_PATH);
    if sock_path.exists() {
        std::fs::remove_file(sock_path)?;
    }

    // Listen for queries
    let listener = UnixListener::bind(SOCKET_PATH)?;
    log(&format!("ctxgraph: listening on {SOCKET_PATH}"));

    for stream in listener.incoming() {
        match stream {
            Ok(_conn) => {
                // v0.1: log and ignore — full query protocol comes later
                log("ctxgraph: received connection (no-op in v0.1)");
            }
            Err(e) => {
                log(&format!("ctxgraph: accept error: {e}"));
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
