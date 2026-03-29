//! test-agent — minimal binary for verifying microVM isolation.
//!
//! Runs as PID 1 inside a libkrun microVM. Prints a heartbeat
//! every 5 seconds to prove the VM is alive.

fn main() {
    let name = std::env::var("AGENT_NAME").unwrap_or_else(|_| "unknown".into());
    println!("test-agent: booted inside microVM (AGENT_NAME={name})");

    let mut tick: u64 = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        tick += 1;
        println!("test-agent: heartbeat {tick}");
    }
}
