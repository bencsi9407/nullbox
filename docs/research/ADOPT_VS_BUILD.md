# NullBox: Adopt vs Build Analysis

Research into open-source projects NullBox could adopt or integrate instead of building from scratch.

**Date:** 2026-03-29

---

## 1. PID 1 Init System (nulld)

NullBox needs: service supervision, dependency resolution, process reaping, TOML config.

### Horust — RECOMMENDED ADOPT

- **URL:** https://github.com/FedericoPonzi/Horust
- **Language:** Rust
- **License:** MIT
- **Stars:** 274 | **Last updated:** 2026-03-28 | **Status:** Beta, actively maintained
- **What it does:** Supervisor / init system designed for containers. Supports service dependency ordering (`start-after`), parallel startup, `die-if-failed` cascading shutdown, TOML service definitions, process reaping.
- **What NullBox can reuse:** Core supervision loop, dependency DAG resolution, TOML service config parsing, signal handling, process reaping logic. The entire crate can be used as a library.
- **Integration effort:** **Moderate.** Horust's design closely matches nulld's requirements. It would need extensions for: JSON-RPC control socket integration, custom health check protocols, and NullBox-specific service lifecycle hooks. But the core supervision + dependency engine is solid and avoids reimplementing PID 1 boilerplate.
- **Risk:** Beta maturity. Small contributor base (primarily one author).

### rustysd

- **URL:** https://github.com/KillingSpark/rustysd
- **Language:** Rust
- **License:** MIT
- **Stars:** 573 | **Last updated:** 2026-03-26
- **What it does:** Drop-in replacement for a subset of systemd. Parses systemd unit files, supports socket activation, dependency ordering, parallel startup.
- **What NullBox can reuse:** Dependency resolution engine, socket activation support, unit file parsing patterns.
- **Integration effort:** **Heavy.** Tied to systemd unit file format. NullBox uses TOML, so the config layer would need replacement. However, the dependency resolution and service lifecycle state machine are well-tested and could be extracted.

### rinit

- **URL:** https://github.com/rinit-org/rinit
- **Language:** Rust
- **License:** GPL-3.0 (INCOMPATIBLE)
- **Stars:** 69 | **Last updated:** 2026-03-20
- **What it does:** Async init inspired by s6/66/daemontools.
- **Verdict:** **Skip.** GPL-3.0 is incompatible with NullBox's likely MIT/Apache-2.0 licensing. Also low maturity.

### dinit (reference architecture)

- **URL:** https://github.com/davmac314/dinit
- **Language:** C++
- **License:** Apache-2.0
- **Stars:** 962 | **Last updated:** 2026-03-29
- **What it does:** Mature service manager with proper dependency resolution (hard deps, milestone deps, soft deps), parallel startup, `dinitcheck` static analysis tool, process supervision with rollback.
- **What NullBox can reuse:** Not code directly (C++), but dinit's **dependency model is the gold standard** for small init systems. Its dependency types (depends-on, depends-ms, waits-for) and rollback/restart semantics should be the design reference for nulld. The `dinitcheck` concept (static validation of service graphs) is worth porting.
- **Integration effort:** **Design reference only.** Port the dependency model concepts into Rust.

### Recommendation

**Adopt Horust as the starting point for nulld.** Fork it, extend with JSON-RPC control socket and dinit-style dependency semantics. This saves 2-3 months of PID 1 boilerplate (signal handling, zombie reaping, service state machine).

---

## 2. microVM Isolation (cage)

NullBox needs: per-agent lightweight VMs via KVM, fast boot (<200ms), minimal footprint, programmatic API.

### rust-vmm crates — RECOMMENDED ADOPT

- **URL:** https://github.com/rust-vmm (org with 30+ crates)
- **Language:** Rust
- **License:** Apache-2.0 (all crates)
- **Key crates and stars:**
  - `vm-virtio` — 444 stars, virtio device implementations
  - `kvm` (kvm-ioctls + kvm-bindings) — 393 stars, safe KVM wrappers
  - `vm-memory` — 353 stars, guest memory abstraction
  - `linux-loader` — 219 stars, kernel loading
  - `vmm-reference` — 164 stars, reference VMM combining all crates
  - `vhost` — 159 stars, vhost-user support
  - `seccompiler` — 108 stars, seccomp-bpf jailing
  - `event-manager` — 50 stars, event loop
  - `vm-allocator` — 34 stars, resource allocation
- **What NullBox can reuse:** This is the **exact building block set** for cage. Firecracker and Cloud Hypervisor are both built on these crates. Use `kvm-ioctls` for KVM interaction, `vm-memory` for guest memory, `linux-loader` for kernel loading, `seccompiler` for sandboxing, `vm-virtio` for virtio-net/block/vsock devices.
- **Integration effort:** **Moderate.** The `vmm-reference` project is literally a template VMM built from these crates. Clone it, strip what you don't need, add NullBox-specific features (capability manifests, ctxgraph integration). This is the standard approach — it's what Firecracker did.

### Firecracker (reference architecture)

- **URL:** https://github.com/firecracker-microvm/firecracker
- **Language:** Rust
- **License:** Apache-2.0
- **Stars:** 33,349 | **Last updated:** 2026-03-29 | **Production:** Powers AWS Lambda + Fargate
- **What it does:** Purpose-built microVM with <125ms boot, <5MB memory footprint, 5 emulated devices only, jailer companion for host-side sandboxing.
- **What NullBox can reuse:** Firecracker itself is a complete VMM, not a library. However:
  - Its **jailer** design (seccomp + cgroups + chroot) is the reference for cage's host-side isolation.
  - Its REST API over Unix socket is a proven control plane pattern.
  - Its device model (only virtio-net, virtio-block, virtio-vsock, serial, keyboard) proves that minimal device emulation is sufficient.
- **Integration effort:** **Design reference + shared rust-vmm crates.** Don't fork Firecracker itself — it's tightly coupled to AWS's use case. Instead, build cage from the same rust-vmm crates using vmm-reference as the skeleton.

### Cloud Hypervisor

- **URL:** https://github.com/cloud-hypervisor/cloud-hypervisor
- **Language:** Rust
- **License:** Apache-2.0
- **Stars:** 5,425 | **Last updated:** 2026-03-29
- **What it does:** Full-featured VMM with hotplug, Windows support, vhost-user, VFIO passthrough. Built on rust-vmm crates.
- **What NullBox can reuse:** More feature-rich than Firecracker but heavier. Its vhost-user device offloading and virtio-fs support could be useful if agents need shared filesystem access.
- **Integration effort:** **Heavy as a whole**, but individual subsystems (e.g., virtio-fs integration) can be studied.

### libkrun

- **URL:** https://github.com/containers/libkrun
- **Language:** Rust
- **License:** Apache-2.0
- **Stars:** 1,774 | **Last updated:** 2026-03-28
- **What it does:** Dynamic library providing virtualization-based process isolation. Wraps KVM into a library call — you literally call `krun_start_enter()` and your process is now in a microVM. Uses virtio-fs for filesystem, virtio-vsock for networking. Built on rust-vmm crates.
- **What NullBox can reuse:** libkrun is the **closest existing project to cage's vision** — per-process VM isolation as a library call. It could potentially be used directly as cage's backend, with NullBox adding the capability manifest layer on top.
- **Integration effort:** **Drop-in to Moderate.** If libkrun's isolation model fits (it uses virtio-fs for root filesystem, virtio-vsock for networking), cage could wrap libkrun directly. The main gap is that libkrun is designed for single-process isolation, while cage may need more VM lifecycle control.

### Hyperlight

- **URL:** https://github.com/hyperlight-dev/hyperlight
- **Language:** Rust
- **License:** Apache-2.0
- **Stars:** 4,182 | **Last updated:** 2026-03-28
- **What it does:** Microsoft's embeddable micro-VMM for executing untrusted functions with hypervisor-based isolation. Designed for embedding in applications, not full OS isolation.
- **What NullBox can reuse:** Interesting for function-level isolation but probably too fine-grained for agent-level VMs. Better suited if NullBox ever needs to sandbox individual tool calls within an agent.
- **Integration effort:** **Heavy** for cage's use case. Different isolation granularity.

### crosvm

- **URL:** https://github.com/google/crosvm
- **Language:** Rust
- **License:** BSD-3-Clause
- **Stars:** 1,167 | **Last updated:** 2026-03-28
- **What it does:** Chrome OS VMM. Full-featured, supports GPU passthrough, Wayland, audio. Built on rust-vmm crates.
- **Verdict:** Overkill for NullBox. Too many desktop-oriented features.

### Kata Containers (reference architecture)

- **URL:** https://github.com/kata-containers/kata-containers
- **Language:** Rust (runtime) + Go
- **License:** Apache-2.0
- **Stars:** 7,676 | **Last updated:** 2026-03-28
- **What it does:** OCI-compatible container runtime that runs each container in its own lightweight VM. Uses Firecracker, Cloud Hypervisor, or QEMU as the VMM backend.
- **What NullBox can reuse:** Kata's architecture (shimv2 → VMM → agent inside VM) is a proven pattern. Its guest agent protocol and VM lifecycle management are worth studying.
- **Integration effort:** **Design reference.** Kata is Go-heavy and OCI-coupled.

### Recommendation

**Build cage from rust-vmm crates using vmm-reference as the skeleton.** Evaluate libkrun as a potential shortcut — if its isolation model fits, wrap it directly. Use Firecracker's jailer and Kata's guest agent as design references.

---

## 3. Network Firewall (egress)

NullBox needs: default-deny nftables, per-agent allow rules, programmatic rule management from Rust.

### nftables-rs — RECOMMENDED ADOPT

- **URL:** https://github.com/nftables-rs/nftables-rs
- **Language:** Rust
- **License:** Apache-2.0
- **Stars:** 80 | **Last updated:** 2026-03-18
- **What it does:** Safe abstraction over nftables JSON API. Create, read, apply rulesets. Batch operations. JSON Schema generation. Typed Rust structs for all nftables objects (tables, chains, rules, sets, maps).
- **What NullBox can reuse:** **Directly.** This crate does exactly what egress needs — programmatically build nftables rulesets in Rust and apply them. Use it to: create per-agent chains, add allow rules from capability manifests, apply default-deny policies.
- **Integration effort:** **Drop-in.** Add as a dependency, build egress's policy engine on top. The crate handles all nftables JSON API serialization/deserialization.

### nftnl-rs (Mullvad)

- **URL:** https://github.com/mullvad/nftnl-rs
- **Language:** Rust
- **License:** Apache-2.0
- **Stars:** 98 | **Last updated:** 2026-03-18
- **What it does:** Low-level Rust bindings for libnftnl (netlink-based nftables API). Used in production by Mullvad VPN.
- **What NullBox can reuse:** Lower-level than nftables-rs. Uses netlink directly instead of JSON API. More performant for high-frequency rule updates but harder to use.
- **Integration effort:** **Moderate.** More boilerplate than nftables-rs but battle-tested in production (Mullvad VPN). Consider if egress needs high-performance rule updates.

### rustables

- **URL:** https://gitlab.com/rustwall/rustables
- **Language:** Rust
- **License:** Apache-2.0/MIT
- **What it does:** Fork of nftnl-rs with additional abstractions. Netlink-based.
- **Verdict:** Less mature than either nftables-rs or nftnl-rs. Rough edges acknowledged by maintainer.

### Recommendation

**Use nftables-rs as the foundation for egress.** It provides the right abstraction level — typed Rust structs for nftables objects, batch operations, JSON API integration. If performance becomes an issue later, consider dropping to nftnl-rs for hot paths.

---

## 4. Control Socket (nullctl <-> nulld)

NullBox needs: Unix socket JSON-RPC for managing services.

### varlink (Rust implementation) — RECOMMENDED EVALUATE

- **URL:** https://github.com/varlink/rust
- **Language:** Rust
- **License:** Apache-2.0
- **Stars:** 116 | **Last updated:** 2026-02-26
- **What it does:** Rust implementation of the Varlink protocol — a typed IPC protocol over Unix sockets. Used by systemd and (formerly) Podman. Supports interface definitions with code generation, introspection, error types.
- **What NullBox can reuse:** Varlink provides typed IPC with schema validation — stronger than raw JSON-RPC. Interface definitions generate Rust client/server code. systemd's adoption proves it works for service management.
- **Integration effort:** **Moderate.** Varlink has a learning curve but provides better type safety than raw JSON-RPC. You define `.varlink` interface files and generate Rust code.

### zlink

- **URL:** https://github.com/z-galaxy/zlink
- **Language:** Rust
- **License:** (check repo)
- **Stars:** 48
- **What it does:** Modern async-first Varlink implementation. Tokio-native.
- **Integration effort:** **Moderate.** Newer and async-native, but less proven.

### tonic with UDS (gRPC over Unix sockets)

- **URL:** https://github.com/hyperium/tonic (part of the broader tonic ecosystem)
- **Language:** Rust
- **License:** MIT
- **What it does:** tonic supports Unix domain socket transport natively. Define services in protobuf, get generated Rust client/server with streaming support.
- **What NullBox can reuse:** If NullBox wants protobuf-based RPC instead of JSON-RPC, tonic over UDS is the standard Rust approach. Stronger typing, streaming, and a massive ecosystem.
- **Integration effort:** **Moderate.** Requires protobuf toolchain. Heavier than JSON-RPC for simple request/response but better for streaming (e.g., log tailing, event subscriptions).

### jsonrpc crate (raw JSON-RPC over UDS)

- **URL:** https://crates.io/crates/jsonrpc
- **What it does:** Synchronous JSON-RPC transport over Unix domain sockets via `UnixStream`.
- **Integration effort:** **Drop-in** for the simplest possible approach. But synchronous and feature-light.

### Recommendation

**Evaluate varlink vs raw JSON-RPC.** Varlink gives you typed interfaces, introspection, and systemd ecosystem alignment — good fit for a service management protocol. If you want the simplest possible path, use the `jsonrpc` crate over UDS. If you want streaming (log tailing, event subscriptions), consider tonic+UDS with protobuf.

For an immutable OS targeting simplicity, **varlink is the sweet spot** — typed but lightweight, Unix-native, proven in systemd.

---

## 5. Shared Memory Store (ctxgraph)

NullBox needs: SQLite content-addressed key-value store, BLAKE3 hashing, shared across agents.

### iroh-blobs — RECOMMENDED EVALUATE

- **URL:** https://github.com/n0-computer/iroh-blobs
- **Language:** Rust
- **License:** Apache-2.0
- **Stars:** 108 (iroh-blobs) + 8,113 (iroh main) | **Last updated:** 2026-03-27
- **What it does:** BLAKE3-based content-addressed blob storage with verified streaming. Supports persistent and in-memory stores. Range requests with cryptographic verification.
- **What NullBox can reuse:** The content-addressing scheme (BLAKE3 tree hashing), blob store traits, and verification logic. iroh-blobs is designed as a standalone crate separate from iroh's networking stack.
- **Integration effort:** **Moderate.** iroh-blobs is oriented toward network blob transfer, not local SQLite storage. You'd use the content-addressing primitives (BLAKE3 hashing, tree structure) but implement your own SQLite-backed store conforming to iroh-blobs' store traits. Alternatively, just use the BLAKE3 hashing approach and build a simpler store.

### redb

- **URL:** https://github.com/cberner/redb
- **Language:** Rust
- **License:** Apache-2.0
- **Stars:** 4,367 | **Last updated:** 2026-03-29
- **What it does:** Embedded key-value database in pure Rust. ACID transactions, MVCC, zero-copy reads. Inspired by LMDB.
- **What NullBox can reuse:** If ctxgraph doesn't strictly need SQLite, redb is a pure-Rust embedded KV store with excellent performance. No C dependency (unlike rusqlite/SQLite). ACID guarantees.
- **Integration effort:** **Moderate.** Would need to add content-addressing (BLAKE3 hashing) on top. redb provides the storage engine; you add the content-addressed semantics.

### rusqlite + BLAKE3 (build your own)

- **rusqlite URL:** https://crates.io/crates/rusqlite
- **blake3 URL:** https://crates.io/crates/blake3
- **What it does:** rusqlite is the standard Rust SQLite wrapper. blake3 is the official BLAKE3 hash crate.
- **Integration effort:** **Low complexity, more custom code.** Combine rusqlite + blake3 to build a content-addressed KV store. Schema: `CREATE TABLE blobs (hash BLOB PRIMARY KEY, data BLOB)`. This is the simplest path if ctxgraph's requirements are straightforward.

### Recommendation

**Build ctxgraph with rusqlite + blake3.** The requirements (content-addressed KV store over SQLite) are simple enough that adopting a larger framework adds unnecessary complexity. Use iroh-blobs' BLAKE3 tree hashing design as a reference if you need verified streaming later. Consider redb if you want to drop the SQLite dependency for a pure-Rust stack.

---

## 6. Immutable OS References

### Bottlerocket (reference architecture)

- **URL:** https://github.com/bottlerocket-os/bottlerocket
- **Language:** Rust
- **License:** Apache-2.0 / MIT (dual)
- **Stars:** 9,560 | **Last updated:** 2026-03-28 | **Production:** Used by AWS ECS/EKS
- **What it does:** Immutable Linux OS for containers. API-driven configuration (no SSH by default), dm-verity for root filesystem integrity, A/B partition updates, Rust-based system agents.
- **What NullBox can reuse:**
  - **Update mechanism:** A/B partition scheme with dm-verity verification
  - **API-driven model:** Bottlerocket is configured entirely through an API, not config files — same philosophy as NullBox
  - **System agent patterns:** Rust-based agents that manage the OS (settings API, update agent, host-containers)
  - **Security model:** SELinux policies, dm-verity, no shell by default
- **Integration effort:** **Design reference.** Bottlerocket is too Kubernetes-coupled to adopt directly, but its architecture proves the immutable-OS-managed-by-API model works at scale.

### Talos Linux (reference architecture)

- **URL:** https://github.com/siderolabs/talos
- **Language:** Go
- **License:** MPL-2.0
- **Stars:** 10,138 | **Last updated:** 2026-03-29
- **What it does:** Immutable Linux OS for Kubernetes. No SSH, no shell, API-only management, mutual TLS for all communication, squashfs root.
- **What NullBox can reuse:** Talos is the most philosophically aligned project — API-only, no shell, immutable. Study its:
  - **Machine config model:** declarative YAML → API-applied
  - **Security posture:** mutual TLS everywhere, no shell escape
  - **Upgrade model:** staged, atomic, rollback-capable
- **Integration effort:** **Design reference only.** Go codebase, Kubernetes-specific.

### Flatcar Container Linux

- **URL:** https://github.com/flatcar/Flatcar
- **Language:** Mixed
- **License:** Apache-2.0
- **Stars:** 1,122
- **What it does:** Immutable container-optimized Linux. Auto-updates, minimal attack surface.
- **Integration effort:** **Design reference only.** Less innovative than Bottlerocket/Talos for NullBox's use case.

---

## Summary: Recommended Adoption Strategy

| NullBox Component | Adopt | Reference Design | Build Custom |
|---|---|---|---|
| **nulld** (init) | Horust (fork + extend) | dinit dependency model | Control socket integration |
| **cage** (microVM) | rust-vmm crates + vmm-reference skeleton | Firecracker jailer, Kata guest agent | Capability manifest layer |
| **egress** (firewall) | nftables-rs (direct dependency) | Mullvad nftnl-rs for perf | Policy engine, per-agent rules |
| **control socket** | varlink or jsonrpc crate | Firecracker API, containerd | NullBox-specific API surface |
| **ctxgraph** (store) | rusqlite + blake3 crates | iroh-blobs content addressing | Schema, shared memory semantics |
| **OS model** | — | Bottlerocket, Talos | NullBox-specific image build |

### Estimated Time Savings

- **nulld:** ~2-3 months saved by adopting Horust's supervision core
- **cage:** ~4-6 months saved by using rust-vmm crates instead of writing KVM wrappers from scratch
- **egress:** ~1 month saved by using nftables-rs instead of raw netlink/nftables bindings
- **control socket:** ~2-4 weeks saved by using varlink or jsonrpc crate
- **ctxgraph:** ~1-2 weeks saved (rusqlite + blake3 are well-documented; the custom part is minimal)

### Total: ~8-11 months of development time saved by adopting existing projects.

---

## Sources

- [Firecracker](https://github.com/firecracker-microvm/firecracker)
- [Cloud Hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
- [libkrun](https://github.com/containers/libkrun)
- [Hyperlight](https://github.com/hyperlight-dev/hyperlight)
- [crosvm](https://github.com/google/crosvm)
- [Kata Containers](https://github.com/kata-containers/kata-containers)
- [rust-vmm org](https://github.com/rust-vmm)
- [vmm-reference](https://github.com/rust-vmm/vmm-reference)
- [Horust](https://github.com/FedericoPonzi/Horust)
- [rustysd](https://github.com/KillingSpark/rustysd)
- [rinit](https://github.com/rinit-org/rinit)
- [dinit](https://github.com/davmac314/dinit)
- [nftables-rs](https://github.com/nftables-rs/nftables-rs)
- [nftnl-rs (Mullvad)](https://github.com/mullvad/nftnl-rs)
- [varlink Rust](https://github.com/varlink/rust)
- [iroh](https://github.com/n0-computer/iroh)
- [iroh-blobs](https://github.com/n0-computer/iroh-blobs)
- [redb](https://github.com/cberner/redb)
- [Bottlerocket](https://github.com/bottlerocket-os/bottlerocket)
- [Talos Linux](https://github.com/siderolabs/talos)
- [Flatcar](https://github.com/flatcar/Flatcar)
- [nftables-rs docs](https://docs.rs/nftables/latest/nftables/)
- [rustables](https://gitlab.com/rustwall/rustables)
- [zlink](https://github.com/z-galaxy/zlink)
- [Dinit comparison wiki](https://github.com/davmac314/dinit/wiki/Comparison)
- [rust-vmm building blocks (AWS re:Invent)](https://d1.awsstatic.com/events/reinvent/2019/Rust-VMM_Playing_Lego_with_virtualization_components_OPN403.pdf)
