# NullBox — Technical Architecture

---

## Full Stack

```
+--------------------------------------------------------------------+
|                Harbor (Skill Registry + Forge Testing)              |
+--------------------------------------------------------------------+
|            Swarm (Multi-Agent + Multi-Node Orchestration)           |
+----------+---------------+--------------+--------------------------+
| Sentinel |   Watcher     |  ctxgraph    |  Provenance Vault        |
|(Firewall)|(Observability)|(Memory Graph)| (Identity + Signing)     |
+----------+-------+-------+--------------+--------------------------+
|    CloakPipe     |         Gate (HITL Approvals)                   |
|   (PII Redact)   |                                                 |
+------------------+-------------------------------------------------+
|              Warden (Credential Isolation Proxy)                   |
+--------------------------------------------------------------------+
|                Egress (Network Controller)                          |
+--------------------------------------------------------------------+
|     Cage (Per-Agent MicroVM) + Accelerator Manager (GPU/NPU)       |
+--------------------------------------------------------------------+
|        Phoenix (Self-Healing) + Edge Power Manager                  |
+--------------------------------------------------------------------+
|          Linux 6.18 LTS . KSPP Hardened . CONFIG_MODULES=n         |
|         SquashFS rootfs . Clang/ThinLTO/CFI . UKI . A/B OTA       |
+--------------------------------------------------------------------+
```

---

## Filesystem Layout

```
/           -> Read-only SquashFS (~80-100MB, the entire OS)
/system     -> Read-only, NullBox binaries only (~15 total)
/tmp        -> tmpfs (ephemeral, wiped on reboot)
/var        -> Writable overlay: agent data, logs, ctxgraph state
/agent      -> Per-agent microVM root filesystems (managed by Cage)
/vault      -> Warden encrypted credential store (dm-crypt, TPM-bound)
/snapshots  -> Phoenix + snapshot store (encrypted)
```

---

## Partition Layout

```
[EFI] [BOOT-A] [BOOT-B] [META] [STATE] [EPHEMERAL]

BOOT-A / BOOT-B  -> A/B immutable UKI images (active / standby)
META              -> Machine UUID, install config, hardware fingerprint
STATE             -> nulld config, Warden vault (AES-256-GCM at rest)
EPHEMERAL         -> ctxgraph, agent memory, Watcher audit logs, snapshots
```

---

## Kernel Configuration (KSPP)

```
# THE MOST IMPORTANT FLAG
CONFIG_MODULES=n          # No loadable kernel modules. EVER.

# Memory Protection
CONFIG_STRICT_KERNEL_RWX=y
CONFIG_RANDOMIZE_BASE=y        # KASLR
CONFIG_STACKPROTECTOR_STRONG=y

# Control Flow Integrity (requires Clang)
CONFIG_CFI_CLANG=y
CONFIG_SHADOW_CALL_STACK=y     # ARM64

# Syscall Filtering (Cage)
CONFIG_SECCOMP=y
CONFIG_SECCOMP_FILTER=y

# MAC (per-agent policies)
CONFIG_SECURITY_APPARMOR=y
CONFIG_SECURITY_LANDLOCK=y

# eBPF (Watcher)
CONFIG_BPF_SYSCALL=y
CONFIG_BPF_JIT=y

# Filesystem
CONFIG_SQUASHFS=y
CONFIG_OVERLAY_FS=y

# Virtualization (Cage)
CONFIG_KVM=y
CONFIG_VIRTIO=y

# Encryption (Warden)
CONFIG_DM_CRYPT=y
```

---

## OTA Update Flow (A/B Atomic)

```
1. Download new UKI to inactive partition (BOOT-B if BOOT-A active)
2. Reboot
3. UEFI Secure Boot verifies new image cryptographically
4. Switch active partition
5. If new image fails 3x -> automatic rollback to previous partition
```

Zero manual intervention. Zero maintenance.

---

## Agent Lifecycle (via Cage)

```
nullctl agent start researcher
  -> Cage reads AGENT.toml
  -> Creates microVM with declared capabilities
  -> Boots in <200ms (libkrun)
  -> Agent process starts inside VM
  -> All traffic routes through Warden -> Sentinel -> CloakPipe

nullctl agent pause researcher
  -> VM suspended (state in memory, CPU released)

nullctl agent resume researcher
  -> VM resumed from suspended state

nullctl agent stop researcher
  -> VM destroyed, ephemeral data wiped
```

---

## Capability Manifest (AGENT.toml)

```toml
[agent]
name = "researcher"
version = "1.2.0"

[capabilities]
network.allow = ["api.perplexity.ai", "api.exa.ai", "api.openai.com"]
filesystem.read = ["/data/research"]
filesystem.write = ["/data/research/output"]
shell = false
credential_refs = ["OPENAI_KEY", "EXA_KEY"]
accelerator = "none"
max_cpu_percent = 40
max_memory_mb = 512
max_api_calls_per_hour = 200

[tools]
send_email = { risk = "low" }
read_files = { risk = "low" }
write_files = { risk = "medium" }
delete_files = { risk = "critical" }    # Triggers Gate
execute_payment = { risk = "critical" }  # Triggers Gate
```

---

## Network Security (Egress + Cage)

```
Agent declares: network.allow = ["api.openai.com"]

MicroVM virtual NIC -> only routes to declared domains
    -> packets to undeclared hosts dropped at network topology level
    -> not software policy, hardware-level virtual networking

Egress layer (OS-level, below Cage):
    -> blocks cloud metadata (169.254.169.254)
    -> blocks RFC-1918 ranges from agent traffic
    -> DNS-to-IP binding prevents rebinding attacks
    -> blocks known malicious IP ranges
```

---

## Trust Chain (Harbor -> Forge -> Cage)

```
Author signs skill with Ed25519
  -> Forge dry-runs in simulated microVM
    -> Forge generates capability proof (signed)
      -> Harbor publishes skill + proof
        -> User installs skill
          -> Cage enforces capabilities at runtime

A malicious skill must fool ALL THREE:
  1. Forge simulation
  2. Ed25519 signature forging
  3. Cage microVM escape
Each layer is independent.
```

---

## Swarm Architecture

```
Declarative manifest -> nullctl swarm deploy research-team.toml

researcher (Cage VM) --writes--> ctxgraph --triggers--> enricher (Cage VM)
                                                              |
                                                         writes ctxgraph
                                                              |
                                                         triggers reporter (Cage VM)
                                                              |
                                                         Gate approval (human)
                                                              |
                                                         sends Slack report

Each agent: own microVM, own credentials, own capabilities
Shared state: ctxgraph (content-addressed, Provenance-signed)
Coordination: Raft-like consensus for multi-node swarms (replicates ctxgraph state + scheduling decisions)
Audit: Watcher shows full execution graph
```

---

## Self-Healing (Phoenix)

| Scenario | Detection | Response |
|---|---|---|
| Infinite LLM retry loop | CPU >95% for >5min | Pause, snapshot, resume with halved rate limit |
| Network partition | All outbound timing out >2min | Offline mode, switch to local Ollama |
| Skill OOM | Memory limit hit, VM killed | Restore from snapshot, reduce memory budget |
| OS update | New UKI deployed | Migrate agent states to new VM format, zero downtime |

---

## Attack Coverage Matrix

| Attack Vector | Defeated By (VibeGuard) | Defeated By (NullBox) |
|---|---|---|
| Credential theft from .env | Warden vault | + TPM-bound, dm-crypt |
| Prompt injection via web | Sentinel inbound scan | + eBPF syscall monitoring |
| Malicious skill from registry | VibeGuard scanner | + Cage capability manifest + Forge proof |
| Agent escapes sandbox | Application-layer proxy | + Dedicated kernel per VM (container escape path gone) |
| Memory poisoning | -- | + ctxgraph content-addressed Merkle entries |
| Data exfiltration | CloakPipe redaction | + Egress DNS-to-IP binding |
| PII leak to cloud LLM | CloakPipe mandatory | + OS-level mandatory middleware |
| Unreviewed critical action | -- | + Gate: OS-level hard suspend |
| Log tampering | SQLite audit | + Watcher append-only Merkle log |
| Cost runaway | Warden rate limits | + Cage CPU quotas |
| Agent identity spoofing | -- | + Provenance Ed25519 per agent, TPM-bound |
