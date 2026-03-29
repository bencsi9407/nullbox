# Kernel — Hardened Linux Base

> The foundation. Everything above depends on getting this right.

**Layer:** NullBox Layer 1

---

## Why This Kernel Configuration

If a machine exists solely to execute AI agents, why must it carry the attack surface of a general-purpose operating system? NullBox answers this by stripping Linux down to the minimum viable kernel for agent workloads, then hardening what remains.

---

## Kernel Selection: Linux 6.18 LTS

Linux 6.18 LTS (released Nov 30, 2025) is the recommended kernel for NullBox. Selected over 6.12 LTS for:

- **Cryptographically signed eBPF programs** — runtime bytecode integrity verification for Watcher audit probes. This means eBPF programs cannot be tampered with post-load, a critical guarantee for security-sensitive audit.
- **Production Rust in-kernel** — Rust is no longer experimental as of December 2025. 600,000+ lines of production Rust in the Linux kernel. Android 16 ships Rust kernel drivers to millions of devices. NullBox can write safety-critical kernel components in Rust.
- **Rust Binder driver** — first production Rust IPC driver, proving the Rust-in-kernel ecosystem is real
- **"Sheaves" memory allocator** — per-CPU caching layer for slab allocations, reducing locking overhead. Improves microVM spawn throughput.
- **KVM support** — required for libkrun microVMs (Cage). Without it, per-agent hardware isolation is impossible.
- **ARM64 + x86_64 first-class** — RPi 5 and x86 VPS without separate kernel builds
- **NPU/accelerator support** — Hailo-10H (40 TOPS), Coral TPU, improved NVIDIA open modules

### Kernel Version Decision Matrix

| Kernel | LTS Until | Key Advantage | Key Risk |
|---|---|---|---|
| **6.12 LTS** | ~2030 | Proven stable, Talos 1.12 validated, full KSPP | No signed eBPF, fewer Rust APIs |
| **6.13** | Non-LTS | ARM CCA confidential compute, AArch64 shadow stacks | No long-term support |
| **6.14** | Non-LTS | Kernel Lockdown for high-security, improved live patching | No long-term support |
| **6.18 LTS** | ~2029 | Signed eBPF, production Rust, Sheaves allocator | Newer, less battle-tested than 6.12 |
| **6.19** | Non-LTS | 30% AMD GPU perf boost, PCIe link encryption | Current stable, no LTS |

**Decision:** 6.18 LTS. Signed eBPF programs and production Rust support are essential for NullBox's security model. If stability concerns arise during early development, fall back to 6.12 LTS (supported until 2030).

### io_uring: Disabled by Default

io_uring remains a significant unresolved attack surface:
- ARMO's "Curing" rootkit PoC (2026) demonstrated io_uring **completely bypasses system call monitoring** — invisible to Falco, Tetragon, and other eBPF-based security tools
- Google restricts io_uring internally due to persistent vulnerabilities
- 2026 CVEs: CVE-2026-23259 (memory leak), CVE-2026-23113 (kernel panic), CVE-2025-38106 (use-after-free privilege escalation)

**NullBox disables io_uring entirely** (`CONFIG_IO_URING=n`). The I/O performance sacrifice is acceptable for agent workloads that are network-bound, not disk-bound. This eliminates an entire class of kernel exploits.

---

## KSPP Hardening Configuration

Following Kernel Self-Protection Project guidelines (same profile as Talos Linux):

```
# === THE MOST IMPORTANT FLAG ===
CONFIG_MODULES=n          # No loadable kernel modules. Ever.
                          # All drivers compiled in at build time.
                          # Eliminates: insmod, modprobe, eBPF rootkits,
                          # custom kernel modules. Attack surface fixed
                          # permanently at compilation.

# === Memory Protection ===
CONFIG_STRICT_KERNEL_RWX=y    # Kernel text/rodata non-writable at runtime
CONFIG_RANDOMIZE_BASE=y        # KASLR — kernel address space randomization
CONFIG_STACKPROTECTOR_STRONG=y # Stack canaries

# === Control Flow Integrity ===
CONFIG_CFI_CLANG=y            # CFI (requires Clang build, impossible with GCC)
CONFIG_SHADOW_CALL_STACK=y    # ARM64: shadow call stack for return address protection

# === Syscall Filtering ===
CONFIG_SECCOMP=y              # Syscall filtering used by Cage per-agent profiles
CONFIG_SECCOMP_FILTER=y       # BPF-based seccomp filters

# === Mandatory Access Control ===
CONFIG_SECURITY_APPARMOR=y    # AppArmor LSM for per-agent MAC policies
CONFIG_SECURITY_LANDLOCK=y    # Landlock LSM for filesystem sandboxing

# === eBPF (Watcher) ===
CONFIG_BPF_SYSCALL=y          # eBPF for kernel-level audit
CONFIG_BPF_JIT=y              # eBPF JIT for performance

# === Filesystem ===
CONFIG_SQUASHFS=y             # Read-only rootfs
CONFIG_OVERLAY_FS=y           # Writable ephemeral layers (tmpfs on squashfs)

# === Virtualization (Cage) ===
CONFIG_KVM=y                  # Hardware virtualization for microVMs
CONFIG_VIRTIO=y               # VirtIO drivers for microVM communication

# === Encryption (Warden) ===
CONFIG_DM_CRYPT=y             # dm-crypt for vault encryption at rest

# === Power Management (Edge) ===
CONFIG_SUSPEND=y              # Offline/suspend mode
CONFIG_HIBERNATION=y          # Agent state snapshot + resume
CONFIG_CPU_FREQ=y             # cpufreq for edge power management

# === io_uring: DISABLED (unresolved attack surface) ===
CONFIG_IO_URING=n             # Bypasses all syscall monitoring. Not needed for agent workloads.

# === Confidential Computing (production-ready on AMD/Intel as of 2025) ===
CONFIG_AMD_MEM_ENCRYPT=y      # AMD SEV-SNP: encrypted VM memory (GA on Azure, GCP, AWS)
CONFIG_INTEL_TDX_GUEST=y      # Intel TDX: trusted execution domains (GA on GCP)
# CONFIG_ARM_CCA=y            # ARM CCA: enable when broadly available (2027+)
```

### Why CONFIG_MODULES=n Is Paramount

Even if an attacker achieves root access within the OS, they are **structurally prevented** from:
- Loading a malicious eBPF rootkit
- Installing a custom kernel module to hide their presence
- Intercepting host system calls
- Modifying kernel behavior post-boot

The attack surface of the kernel is fixed permanently at compilation.

---

## Build Toolchain

**Clang + ThinLTO** (same as Talos 1.12+):

- Enables **Control Flow Integrity (CFI)** — impossible with GCC
- CFI restricts execution flow, drastically reducing Return-Oriented Programming (ROP) exploits
- ThinLTO gives ~5% performance improvement on aarch64
- Entire kernel built with `-fsanitize=cfi`
- **Reproducible builds** — every build of the same version produces bit-for-bit identical ISO

---

## Rootfs Architecture

### Boot Sequence

```
EFI/BIOS
  -> UKI (Unified Kernel Image, single EFI binary ~120MB)
    -> initramfs (self-contained, no disk dependency)
      -> SquashFS image mounted as read-only root
        -> nulld (PID 1, written in Rust, replaces systemd)
          -> Agent runtime services
```

### Filesystem Layout

```
/           -> Read-only SquashFS (~80-100MB, the entire OS)
/system     -> Read-only, NullBox binaries only (~15 total)
/tmp        -> tmpfs (ephemeral, wiped on reboot)
/var        -> Writable overlay: agent data, logs, ctxgraph state
/agent      -> Per-agent microVM root filesystems (managed by Cage)
/vault      -> Warden encrypted credential store (dm-crypt, TPM-bound)
/snapshots  -> Phoenix + snapshot store (encrypted)
```

### Immutability Guarantee

The SquashFS root filesystem **cannot be remounted as writable**, even by root. This means:
- Malware cannot embed itself into system binaries
- No configuration drift (identical nodes stay identical)
- The OS state is cryptographically verifiable from firmware to application layer
- Total OS footprint: 80-100MB compressed

### Partition Layout

```
[EFI] [BOOT-A] [BOOT-B] [META] [STATE] [EPHEMERAL]

BOOT-A / BOOT-B  -> A/B immutable UKI images (active / standby)
META              -> Machine UUID, install config, hardware fingerprint
STATE             -> nulld config, Warden vault (AES-256-GCM at rest)
EPHEMERAL         -> ctxgraph, agent memory, Watcher audit logs, snapshots
```

---

## OTA Updates (A/B Atomic)

No package managers. No apt. No pip. No yum. Ever.

Update flow:
1. Download new UKI to inactive partition (BOOT-B if BOOT-A is active)
2. Reboot
3. UEFI Secure Boot verifies new image cryptographically
4. Switch active partition
5. If new image fails to boot 3 times -> automatic rollback to previous partition

Zero manual intervention. Zero maintenance overhead.

---

## nulld — PID 1 in Rust

nulld replaces systemd entirely. It is the init system, the service manager, and the agent lifecycle controller.

**What nulld manages:**
- Cage (microVM lifecycle)
- Warden (credential proxy)
- Sentinel (LLM firewall)
- CloakPipe (PII redaction)
- Watcher (audit log)
- Egress (network controller)
- Phoenix (self-healing)
- ctxgraph (memory graph)
- Gate (HITL approvals)

**What doesn't exist:**
- No cron
- No dbus
- No systemd
- No sshd
- No shell
- No interactive login
- ~15 binaries in PATH total

---

## Alternatives Considered

### Unikernels (MirageOS, Unikraft)

Single-purpose VM images with no OS layer. Attractive for isolation but impractical for agent workloads that need filesystem access, network stacks, and dynamic tool execution. NullBox needs a real Linux userspace inside each Cage VM.

### WebAssembly (WASI) as Agent Runtime

WASM provides strong sandboxing but lacks the system-level capabilities agents need: raw network sockets, filesystem operations, GPU access, process spawning. WASM is used for Harbor skill portability (skill.wasm) but not as the primary runtime.

### Firecracker Instead of libkrun

Firecracker (AWS) provides similar microVM isolation but runs as a separate VMM process. libkrun embeds the VMM as a library call, reducing overhead and simplifying the Cage integration. libkrun also supports macOS (via Hypervisor.framework) for future VibeGuard cross-platform compatibility.

### NixOS / Guix for Reproducibility

Declarative OS configuration is appealing but adds massive complexity. NullBox achieves reproducibility through a simpler path: a single SquashFS image built from a deterministic Dockerfile-like pipeline. No Nix expression language needed.

---

## Build Complexity (Honest Assessment)

This is the hardest part of the entire project. Building a custom Linux kernel with:
- KSPP hardening
- SquashFS rootfs
- UKI build pipeline
- nulld as PID 1 in Rust
- Cage with libkrun
- A/B partition management
- ISO generation for RPi5 + x86

**Requires:** Deep Linux kernel and systems programming expertise.

**Reference:** Talos Linux team took 5 years with veteran kernel engineers to reach escape velocity. The OS is the right vision. The question is timing and team.
