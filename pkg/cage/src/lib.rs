//! Cage — Per-agent microVM isolation via libkrun.
//!
//! Each agent gets its own microVM with hardware-level isolation.
//! Capabilities are declared in AGENT.toml and enforced at the hypervisor level.

pub mod krun;
pub mod manifest;
pub mod vm;
