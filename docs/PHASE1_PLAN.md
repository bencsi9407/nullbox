# Implementation Plan: NullBox v0.1 Phase 1 — First Bootable ISO

> Build order: Kernel -> nulld -> Cage -> Egress -> ctxgraph -> First ISO

---

## Requirements

### Hard Requirements (First Bootable Milestone)
- Custom Linux 6.18 LTS kernel with KSPP hardening, Clang/ThinLTO/CFI build
- CONFIG_MODULES=n, CONFIG_IO_URING=n, CONFIG_KVM=y, CONFIG_VIRTIO=y, CONFIG_SQUASHFS=y, CONFIG_OVERLAY_FS=y
- nulld: Rust binary that runs as PID 1, mounts filesystems, starts services in dependency order
- Cage: Rust wrapper around libkrun that reads AGENT.toml and spawns one microVM
- Egress: Default-deny nftables/eBPF network controller with DNS-to-IP binding
- ctxgraph: SQLite-backed content-addressed key-value graph
- SquashFS read-only rootfs (~80-100MB) with overlay for /var and tmpfs for /tmp
- UKI (Unified Kernel Image) packaging
- Bootable ISO for x86_64 (QEMU testable), then RPi 5

### Deferred (Not v0.1)
- Warden OS mode (dm-crypt vault, TPM 2.0 binding)
- Sentinel OS mode (eBPF LLM firewall)
- CloakPipe (PII redaction)
- Watcher (Merkle-chained audit log)
- A/B OTA updates
- Secure Boot / UKI signature verification
- Provenance (Ed25519 per agent)
- Confidential computing (SEV-SNP/TDX)

---

## Workspace Layout

```
nullbox/
  Cargo.toml                    # Workspace root
  kernel/
    config/
      x86_64_defconfig          # KSPP-hardened kernel config for x86_64
      aarch64_defconfig         # KSPP-hardened kernel config for ARM64
    scripts/
      build-kernel.sh           # Clang/ThinLTO kernel build script
      build-initramfs.sh        # initramfs generation
  nulld/
    Cargo.toml
    src/
      main.rs                   # PID 1 entry point
      mount.rs                  # Filesystem mounting (squashfs, overlay, tmpfs)
      service.rs                # Service definition and dependency graph
      supervisor.rs             # Process supervision (start, restart, health)
      config.rs                 # /system/config/nulld.toml parser
      signal.rs                 # Signal handling (SIGCHLD reaping, SIGTERM)
  cage/
    Cargo.toml
    src/
      lib.rs                    # Public API
      manifest.rs               # AGENT.toml parser and validator
      vm.rs                     # libkrun VM lifecycle
      network.rs                # Virtual NIC setup per manifest
      filesystem.rs             # Per-agent rootfs mount setup
      resources.rs              # CPU/memory cgroup limits
  egress/
    Cargo.toml
    src/
      lib.rs                    # Public API
      firewall.rs               # nftables rule generation (default-deny)
      dns.rs                    # DNS resolver with IP binding
      blocklist.rs              # Cloud metadata, RFC-1918 block rules
  ctxgraph/
    Cargo.toml
    src/
      lib.rs                    # Public API
      store.rs                  # SQLite storage backend
      entry.rs                  # Content-addressed entry type
      query.rs                  # Read/write/query operations
  nullctl/
    Cargo.toml
    src/
      main.rs                   # CLI entry point
      commands/
        agent.rs                # nullctl agent start|stop|pause|resume
        status.rs               # nullctl status
  image/
    scripts/
      build-squashfs.sh         # SquashFS rootfs builder
      build-uki.sh              # UKI assembly
      build-iso.sh              # ISO generation (xorriso)
      build-rpi.sh              # RPi 5 image (dd-flashable)
    rootfs/
      etc/                      # Minimal /etc
      system/                   # NullBox binaries
      agent/                    # Empty, Cage populates per-agent
    initramfs/
      init                      # Mounts squashfs, pivots, execs nulld
```

---

## Phase 1: Build Infrastructure and Kernel (Weeks 1-2)

**Goal:** Compile a custom KSPP-hardened Linux 6.18 kernel that boots in QEMU.

| Step | Action | Risk |
|------|--------|------|
| 1 | Initialize Cargo workspace with members: nulld, cage, egress, ctxgraph, nullctl | Low |
| 2 | Create x86_64 defconfig — start from `tinyconfig`, enable KSPP flags + KVM + VirtIO + SquashFS + overlay + EFI + essential drivers | **High** — iterative, expect 3-5 rounds |
| 3 | Write `build-kernel.sh` — downloads Linux 6.18 LTS, applies defconfig, builds with `CC=clang LLVM=1` + ThinLTO | Medium |
| 4 | **Validate:** Boot kernel in QEMU, expect clean panic ("No init found") | Low |

**Mitigation:** Keep a "known-good" defconfig without CFI; layer CFI after basic kernel boots. If 6.18 is unavailable, fall back to 6.12 LTS.

---

## Phase 2: nulld — PID 1 in Rust (Weeks 2-4)

**Goal:** Rust binary that runs as PID 1, mounts filesystem hierarchy, supervises child processes.

| Step | Action | Risk |
|------|--------|------|
| 5 | Create nulld crate — deps: `nix`, `toml`, `log` + `simplelog`. Target: `x86_64-unknown-linux-musl` (static linking) | Low |
| 6 | PID 1 bootstrap — signal handlers (SIGCHLD, SIGTERM, SIGINT), `/dev/kmsg` logging, mount sequence, supervisor loop. Wrap main in `catch_unwind` | **High** — PID 1 panic = kernel panic |
| 7 | Filesystem mounting — `/proc`, `/sys`, `/dev`, `/tmp`, `/var` (overlay), `/dev/pts`, `/dev/shm`. Order matters. | Medium |
| 8 | Service supervisor — reads service defs, topological sort by deps, starts/monitors children via `waitpid`, restart with backoff (1s→30s) | Medium |
| 9 | Service config — `Service` struct (name, binary, args, deps, restart policy). Parsed from `/system/config/nulld.toml` | Low |
| 10 | Signal handling — SIGCHLD reaping loop (waitpid + WNOHANG), SIGTERM triggers graceful shutdown, use `signalfd` or `signal_hook` crate | Medium |
| 11 | **Validate:** Build static musl binary, put in initramfs as `/init`, boot in QEMU. Verify mounts, logging, SIGCHLD reaping | Low |

---

## Phase 3: Cage — microVM Isolation (Weeks 3-8)

**Goal:** Rust wrapper around libkrun that spawns a microVM from an AGENT.toml manifest.

> **This is the highest-risk phase.** libkrun integration in this context is unproven.

| Step | Action | Risk |
|------|--------|------|
| 12 | Build libkrun from source — static library targeting musl. Generate FFI bindings via bindgen. Verify against custom kernel headers. | **Very High** — single biggest risk |
| 13 | AGENT.toml parser — support: name, version, network.allow, filesystem.read/write, max_cpu_percent, max_memory_mb | Low |
| 14 | VM lifecycle — wrap libkrun: `krun_create_ctx`, `krun_set_vm_config`, `krun_set_root`, `krun_set_exec`, `krun_start_enter`. Implement start/pause/resume/stop | High |
| 15 | Minimal guest rootfs — guest kernel + busybox or minimal Rust init + agent binary. For v0.1: agent = Rust binary that makes HTTPS request and writes to ctxgraph | Medium |
| 16 | Per-VM network — TAP device per VM, bridge/NAT, nftables rules restricting TAP to `network.allow` domains | High |
| 17 | Per-VM filesystem — create `/agent/<name>/` with declared paths, bind mounts or virtio-fs into guest | Medium |
| 18 | **Validate:** On dev host, Cage spawns libkrun VM from AGENT.toml, runs command, exits. Measure boot time (target: <200ms) | Medium |

**Fallback:** If libkrun is unviable, evaluate Cloud Hypervisor or Firecracker. Ultimate fallback: isolated processes (cgroups v2 + namespaces + seccomp) to prove the rest of the stack.

---

## Phase 4: Egress — Network Controller (Weeks 4-5, parallel with Phase 3)

**Goal:** Default-deny firewall that only allows declared domains.

| Step | Action | Risk |
|------|--------|------|
| 19 | Create egress crate — deps: nftables bindings, `trust-dns-resolver` | Low |
| 20 | Default-deny firewall — nftables: DROP all, allow loopback + established, block 169.254.169.254, block RFC-1918 from agent traffic | Medium |
| 21 | DNS resolver with IP binding — resolve declared domains, store domain→IP map, reject rebinding to RFC-1918/blocklist, update rules on legitimate IP rotation | Medium |
| 22 | Blocklist management — hardcoded for v0.1 (cloud metadata, RFC-1918). API for Cage to register per-agent allow rules | Low |
| 23 | **Validate:** Start Egress, test 4 cases: metadata blocked, RFC-1918 blocked, allowed domain works, undeclared domain blocked | Low |

---

## Phase 5: ctxgraph — Shared Memory Graph (Weeks 4-5, parallel with Phase 3)

**Goal:** SQLite-backed content-addressed store for inter-agent communication.

| Step | Action | Risk |
|------|--------|------|
| 24 | Create ctxgraph crate — deps: `rusqlite`, `serde` + `serde_json`, `sha2`, `tokio` | Low |
| 25 | Content-addressed store — Entry: `{hash, agent_id, key, value, timestamp}`. SQLite schema. Write = compute SHA-256 hash + insert-or-ignore | Low |
| 26 | Query interface — write(agent_id, key, value)→hash, read(hash)→Entry, query(key_prefix)→Vec, history(key)→Vec | Low |
| 27 | ctxgraph daemon — Unix socket `/var/run/ctxgraph.sock`, JSON-RPC. For VMs: virtio-vsock (needs CONFIG_VHOST_VSOCK) or TCP fallback | Medium |
| 28 | **Validate:** Unit tests + integration: two mock agents share data through ctxgraph | Low |

---

## Phase 6: Image Build and ISO (Weeks 8-12)

**Goal:** Assemble everything into a bootable ISO.

| Step | Action | Risk |
|------|--------|------|
| 29 | initramfs builder — minimal cpio: init script mounts SquashFS, `switch_root`, execs nulld | Medium |
| 30 | SquashFS rootfs builder — staging dir with filesystem layout, copy static binaries to `/system/bin/`, `nulld.toml`, minimal `/etc`, empty mount points. `mksquashfs` with zstd. Target: <100MB | Low |
| 31 | UKI assembly — `ukify` to combine: vmlinuz + initramfs + cmdline. Output: single EFI PE binary ~120MB | Medium |
| 32 | x86_64 ISO — xorriso: EFI partition with UKI or GRUB, data partition with SquashFS | Medium |
| 33 | RPi 5 image — dd-flashable: FAT32 boot (kernel8.img, initramfs, config.txt) + SquashFS partition | **High** — RPi boot differs from UEFI |
| 34 | nullctl CLI — communicates with nulld via Unix socket. Commands: `agent start/stop`, `status` | Low |
| 35 | **End-to-end validation in QEMU** — full boot → nulld → services start → agent VM spawns → HTTPS request → ctxgraph write → VM stop | Medium |

---

## Dependency Graph

```
Phase 1: Kernel ─────────────────────────────────────────────┐
  defconfig -> build script -> QEMU boot test                │
                                                              │
Phase 2: nulld ──────────────────────────────────────────────┤
  crate -> PID 1 bootstrap -> mounts, supervisor, signals    │
  -> QEMU PID 1 test (depends on kernel + nulld)             │
                                                              │
Phase 3: Cage ───────────────────────── CRITICAL PATH ───────┤
  libkrun build (highest risk) -> VM lifecycle                │
  AGENT.toml parser (parallel)                                │
  guest rootfs -> per-VM network + filesystem -> validation   │
                                                              │
Phase 4: Egress ────────── (parallel with Phase 3) ──────────┤
  firewall -> DNS binding -> blocklist -> validation          │
                                                              │
Phase 5: ctxgraph ─────── (parallel with Phase 3) ───────────┤
  store -> query -> daemon -> validation                      │
                                                              │
Phase 6: Image ──────────────────────────────────────────────┘
  initramfs -> SquashFS -> UKI -> ISO -> RPi 5 -> E2E test
```

**Parallelism:** Phases 3, 4, and 5 can proceed in parallel after Phase 1. Egress and ctxgraph are off the critical path.

**Critical path:** Kernel → nulld → Cage/libkrun → Image assembly → E2E test

---

## Risks

### CRITICAL

| Risk | Mitigation |
|------|-----------|
| libkrun doesn't work with CONFIG_MODULES=n or requires undocumented kernel features | Test libkrun on host first. Maintain compatibility matrix. Fallback: Cloud Hypervisor, Firecracker, or isolated processes (cgroups+namespaces+seccomp) |
| Kernel defconfig missing critical drivers, no boot | Start from known-good config (Talos-derived), not tinyconfig. Validate each change |
| PID 1 panic = kernel panic, no debugging | Wrap in `catch_unwind`. Log to `/dev/kmsg`. Keep rescue initramfs with busybox during dev |

### HIGH

| Risk | Mitigation |
|------|-----------|
| RPi 5 boot with CONFIG_MODULES=n requires dozens of compiled-in SoC drivers | Build x86_64 first. For RPi 5, start from RPi Foundation config and subtract |
| libkrun guest networking (TAP + nftables per-VM) is complex | Isolate behind a trait. Start with simple NAT before per-domain filtering |
| virtio-vsock for ctxgraph may not work with libkrun | Fall back to virtio-serial or TCP over TAP |

### MEDIUM

| Risk | Mitigation |
|------|-----------|
| Static musl binaries too large, SquashFS > 100MB | `strip`, `opt-level = "z"`, `lto = true`. SquashFS zstd achieves ~3:1 |
| DNS-to-IP binding breaks with CDN IP rotation | Allow rotation within same ASN/CIDR. Only block rebinding to RFC-1918 |

---

## Development Testing Strategy

No servers, no VPS, no flashing. The entire dev loop runs on your local machine.

### Layer 1: Host-native unit/integration tests (fastest feedback, 80% of dev time)

Every Rust crate compiles and runs `cargo test` on your dev machine. No VM needed.

- AGENT.toml parsing, service dependency resolution, ctxgraph SQLite ops
- nftables rule generation (string output, not applied)
- Signal handling logic, config parsing

```bash
cargo test --workspace
```

### Layer 2: QEMU virtual machine (the core dev loop)

QEMU replaces a real server entirely. Near-native speed with KVM acceleration.

```bash
# Build everything
cargo build --workspace --target x86_64-unknown-linux-musl --release

# Rebuild ISO (< 30 seconds once scripted)
./image/scripts/build-iso.sh

# Boot in QEMU
qemu-system-x86_64 \
  -enable-kvm \
  -m 4G \
  -cpu host \
  -kernel vmlinuz \
  -initrd initramfs.cpio.gz \
  -append "console=ttyS0" \
  -nographic \
  -serial mon:stdio \
  -drive file=nullbox.iso,format=raw
```

Serial console in your terminal. Boot-to-nulld in seconds. `Ctrl+A X` to kill.

For **nested KVM** (Cage microVMs inside QEMU): `-cpu host,+vmx` (Intel) or `-cpu host,+svm` (AMD).

### Layer 3: Automated QEMU test harness (CI pipeline)

Scripts that boot the ISO, send commands over serial, assert output:

```bash
# Boot, wait for nulld, verify services started
qemu-system-x86_64 ... -serial pipe:test_pipe &
echo "nullctl status" > test_pipe.in
grep -q "egress: running" < test_pipe.out
grep -q "ctxgraph: running" < test_pipe.out
grep -q "cage: running" < test_pipe.out
```

Every commit: `cargo test` → build ISO → QEMU boot test → E2E agent test.

### Layer 4: RPi 5 (hardware-specific validation only)

Flash SD card only for RPi-specific things (device tree, GPIO, Hailo NPU). Everything else validated in QEMU first.

### Dev machine prerequisites

- **KVM** — `lscpu | grep Virtualization` (CachyOS: almost certainly enabled)
- **Nested virt** — `options kvm_intel nested=1` or `options kvm_amd nested=1`
- **QEMU** — `pacman -S qemu-full`
- **Clang/LLVM** — `pacman -S clang lld llvm`
- **musl toolchain** — `rustup target add x86_64-unknown-linux-musl`
- **squashfs-tools** — `pacman -S squashfs-tools`
- **xorriso** — `pacman -S xorriso` (ISO generation)

---

## Open Questions

1. **How does nullctl get invoked with no shell?** Recommendation: nullctl as a line reader on `/dev/ttyS0` for v0.1
2. **Guest kernel — same 6.18 or separate minimal?** Investigate during libkrun integration (Step 12)
3. **Agent-to-ctxgraph transport?** Try virtio-vsock first, fall back to TCP
4. **What is the test agent?** Minimal Rust binary: read env var for API key → HTTPS GET to allowed domain → write response to ctxgraph
5. **Linux 6.18 LTS availability?** If unavailable, use 6.12 LTS (only gap: signed eBPF, not needed for v0.1)

---

## Estimated Timeline

| Component | Effort | Complexity |
|-----------|--------|------------|
| Kernel defconfig + build | 1-2 weeks | High |
| nulld (PID 1) | 2-3 weeks | High |
| Cage (libkrun) | 3-4 weeks | **Very High** |
| Egress (network) | 1-2 weeks | Medium |
| ctxgraph (memory) | 1-2 weeks | Low-Medium |
| Image build pipeline | 2-3 weeks | Medium-High |
| nullctl (CLI) | 3-5 days | Low |
| **Total** | **11-17 weeks** | For one experienced systems engineer |

---

## Success Criteria

- [ ] Custom KSPP-hardened kernel boots in QEMU
- [ ] nulld runs as PID 1, mounts filesystems, supervises services
- [ ] Cage spawns a libkrun microVM from AGENT.toml in <500ms (200ms stretch)
- [ ] Egress blocks undeclared traffic and cloud metadata
- [ ] ctxgraph stores/retrieves content-addressed entries across agents
- [ ] Bootable SquashFS ISO under 150MB
- [ ] One agent runs end-to-end: boot → VM spawn → HTTPS → ctxgraph write → VM stop
- [ ] x86_64 ISO boots in QEMU
- [ ] RPi 5 image boots on hardware (stretch goal)
- [ ] All Rust crates have unit tests at 80%+ coverage
- [ ] Zero dynamic dependencies — every binary statically linked
