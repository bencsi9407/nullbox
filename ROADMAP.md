# NullBox Build Roadmap

> Build the shield first. The fortress comes after.

---

## Prerequisites

NullBox ships **after** VibeGuard has users and revenue. The VibeGuard components (Warden, Sentinel, CloakPipe, Watcher) are promoted to OS-level primitives in NullBox.

---

## Phase 1 — NullBox v0.1 (Months 6-12 post-VibeGuard)

**Goal:** First flashable ISO. Requires kernel engineer (hire or cofounder).

| Deliverable | Details |
|---|---|
| Kernel | Linux 6.18 LTS, KSPP hardening, Clang/CFI build, CONFIG_MODULES=n, CONFIG_IO_URING=n |
| nulld | PID 1 in Rust, replaces systemd, manages all OS services |
| Cage | Per-agent microVM isolation via libkrun, AGENT.toml capability manifests |
| Egress | Default-deny network controller, DNS-to-IP binding |
| ctxgraph | Shared agent memory graph, content-addressed entries |
| Warden OS mode | dm-crypt vault, TPM 2.0 binding |
| Sentinel OS mode | eBPF integration, Cage enforcement |
| First ISO | RPi 5 target first, x86 second |

**Success metric:** Working ISO that boots on RPi 5, runs one agent end-to-end.

---

## Phase 2 — NullBox v0.5 (Months 12-15)

| Deliverable | Details |
|---|---|
| Accelerator Manager | GPU/NPU/TPU isolation per agent, Ollama as first-class service |
| Swarm | Multi-agent orchestration, declarative manifests, ctxgraph state bus |
| Forge | Pre-deploy agent testing, capability proofs, Harbor integration |
| Phoenix | Self-healing, eBPF health monitoring, automatic recovery |
| Full Watcher | Merkle-chained audit log, eBPF syscall audit |
| Provenance | Ed25519 per agent, TPM-bound, delegation chain signing |
| Snapshots | Agent state backup/restore, cross-hardware portability |

---

## Phase 3 — NullBox v1.0 (Months 15-18)

| Deliverable | Details |
|---|---|
| Hybrid Cloud Bridge | Local NullBox nodes + cloud overflow |
| Full Harbor | Community submissions, Forge proofs, reputation scores |
| Enterprise | SOC2 exports, HSM Provenance, private Harbor, SSO |
| NullBox Appliance | Pre-flashed hardware: RPi 5 + Hailo-8L + NullBox |
| Gate | Human-in-the-loop approvals, Cage-enforced VM suspension |
| Edge Power | Offline mode, cpufreq tuning, travel/air-gap mode |

---

## Build Dependencies

```
VibeGuard (Warden + Sentinel + CloakPipe + Watcher)
         |
         v
Revenue + Users + Kernel Engineer
         |
         v
NullBox v0.1 (Kernel + nulld + Cage + Egress + ctxgraph)
         |
         v
NullBox v0.5 (Accelerator + Swarm + Forge + Phoenix)
         |
         v
NullBox v1.0 (Cloud Bridge + Enterprise + Appliance)
```

---

## Revenue Streams

| Stream | Description |
|---|---|
| **Open source core** | MIT/Apache licensed. Security story = growth engine. |
| **Managed cloud** | Hybrid Cloud Bridge nodes, managed Swarm, cloud Harbor |
| **Enterprise** | Forge compliance reports, Phoenix SLAs, HSM Provenance, private Harbor |
| **Hardware** | Pre-flashed "NullBox Appliance": RPi 5 + Hailo-8L + NullBox. Jetson variant. |

---

## The Pivot Moment

You'll know it's time to build NullBox when users tell you:

- "A skill bypassed VibeGuard by spawning a subprocess"
- "I need this to work even when the agent is running on a server I don't control"
- "Can you isolate agents from each other, not just from the internet?"

Every one of those is a problem only the OS solves.
