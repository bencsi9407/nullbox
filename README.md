# NullBox - The Operating System for AI Agents

> "If all you're running is agents, why manage a full OS?"

NullBox is an immutable, minimal, flashable Linux OS where every layer exists to serve autonomous AI agents — and nothing else exists at all.

---

## The One-Sentence Pitch

NullBox is Talos Linux for AI agents — a hardened, immutable OS that makes per-agent hardware isolation, credential vaulting, PII redaction, prompt injection defense, and cryptographic audit a kernel-level guarantee, not an application-layer hope.

---

## Core Design Philosophy

| Principle | In Practice |
|---|---|
| **Subtractive, not additive** | No SSH, no shell, no package manager, no mutable rootfs |
| **Immutable by default** | SquashFS read-only rootfs. Root cannot remount writable. |
| **Agents are OS processes** | Spawn, suspend, resume, reclaim — first-class primitives |
| **Every layer integrated** | Warden talks to Cage talks to Watcher talks to CloakPipe |
| **Zero maintenance** | OTA A/B atomic updates. Flash once, manages itself. |
| **Runtime-agnostic** | Runs any MCP workload, Claude Code sessions, custom agent frameworks |

---

## Layer Index

| # | Layer | Doc | Description |
|---|---|---|---|
| 1 | Kernel | [kernel/README.md](kernel/README.md) | Linux 6.18 LTS, KSPP hardened, Clang/CFI, CONFIG_MODULES=n |
| 2 | Cage | [cage/README.md](cage/README.md) | Per-agent microVM isolation via libkrun |
| 3 | Accelerator | [accelerator/README.md](accelerator/README.md) | GPU/NPU/TPU manager, Ollama first-class |
| 4 | Egress | [egress/README.md](egress/README.md) | Default-deny network controller |
| 5 | Warden | (promoted from VibeGuard) | Credential vault, dm-crypt, TPM-bound |
| 6 | Sentinel | (promoted from VibeGuard) | LLM firewall + eBPF integration |
| 7 | CloakPipe | (promoted from VibeGuard) | Mandatory OS-level PII redaction |
| 8 | ctxgraph | [ctxgraph/README.md](ctxgraph/README.md) | Shared agent memory graph |
| 9 | Provenance | [provenance/README.md](provenance/README.md) | Ed25519 per agent, TPM-bound |
| 10 | Watcher | (promoted from VibeGuard) | Merkle-chained audit, eBPF syscall audit |
| 11 | Gate | [gate/README.md](gate/README.md) | HITL approvals, Cage-enforced VM suspend |
| 12 | Swarm | [swarm/README.md](swarm/README.md) | Multi-agent orchestration, A2A protocol |
| 13 | Forge | [forge/README.md](forge/README.md) | Pre-deploy testing, capability proofs |
| 14 | Phoenix | [phoenix/README.md](phoenix/README.md) | Self-healing, eBPF health monitoring |
| 15 | Edge Power | [edge-power/README.md](edge-power/README.md) | Offline mode, cpufreq, travel/air-gap |
| 16 | Snapshots | [snapshots/README.md](snapshots/README.md) | Agent state backup/restore |
| -- | Harbor | [harbor/README.md](harbor/README.md) | Verified skill registry |

---

## Target Hardware

| Hardware | Role | Notes |
|---|---|---|
| **Raspberry Pi 5 (8GB)** | Primary edge target | AI HAT+ 2 (Hailo-10H, 40 TOPS, 8GB RAM). Flashable ISO. |
| **Jetson AGX Thor / Orin** | Heavy local inference | Blackwell GPU (Thor), CUDA-accelerated Ollama. |
| **x86_64 VPS ($6-20/mo)** | Primary cloud target | DigitalOcean, Hetzner, Vultr 1-click templates. |
| **Old laptop/desktop** | Reclaimed hardware | USB-bootable ISO. |
| **ARM servers** | Future | ARM64 first-class from day one. |

---

## What Doesn't Exist in NullBox

- No cron
- No dbus
- No systemd
- No sshd
- No shell
- No interactive login
- No package manager
- No mutable root filesystem
- ~15 binaries in PATH total

---

## nulld — PID 1 in Rust

nulld replaces systemd. It is the init system, service manager, and agent lifecycle controller.

**Manages:** Cage, Warden, Sentinel, CloakPipe, Watcher, Egress, Phoenix, ctxgraph, Gate

**Boot sequence:**
```
EFI/BIOS
  -> UKI (Unified Kernel Image, ~120MB)
    -> initramfs
      -> SquashFS root (read-only, 80-100MB)
        -> nulld (PID 1)
          -> All agent services
```

---

## CLI

```bash
# Agent lifecycle
nullctl agent start researcher
nullctl agent pause researcher
nullctl agent resume researcher
nullctl agent stop researcher

# Swarm orchestration
nullctl swarm deploy research-team.toml

# Snapshots
nullctl snapshot create researcher
nullctl snapshot restore snapshot-researcher-2026-03-25

# Power management
nullctl power set economic
nullctl offline --duration 8h

# Local LLM
nullctl llm enable ollama
```

---

## Relationship to VibeGuard

VibeGuard is the application-layer security middleware that runs on any OS. NullBox is the purpose-built OS that promotes VibeGuard's components (Warden, Sentinel, CloakPipe, Watcher) to OS-enforced primitives — structurally mandatory services managed by nulld, with Watcher using kernel-level eBPF probes for syscall audit. VibeGuard ships first, proves the market, and generates revenue. NullBox ships after.

---

## Full Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the complete technical architecture.
