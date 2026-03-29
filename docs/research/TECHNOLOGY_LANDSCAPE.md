# Technology Landscape: Agent OS Infrastructure (2025-2026)

> Research findings informing NullBox's architecture decisions.

---

## 1. Linux Kernel State

### LTS Kernels Available

| Kernel | Released | LTS Until | Agent-Relevant Features |
|--------|----------|-----------|------------------------|
| 6.12 LTS | Nov 2024 | ~2030 | Talos-validated, KSPP, KVM, eBPF |
| 6.13 | Jan 2025 | Non-LTS | ARM CCA confidential compute, AArch64 shadow stacks (GCS), lazy preemption |
| 6.14 | Mar 2025 | Non-LTS | Kernel Lockdown for high-security, improved live patching |
| 6.18 LTS | Nov 2025 | ~2029 | **Signed eBPF programs**, Rust Binder driver, Sheaves allocator, dm-pcache |
| 6.19 | Feb 2026 | Non-LTS | 30% AMD GPU perf boost, PCIe link encryption |

**NullBox uses 6.18 LTS.** Signed eBPF programs provide tamper-proof audit probes. The Sheaves allocator improves microVM spawn throughput.

### Rust in the Linux Kernel

Rust is **no longer experimental** as of December 2025 (Kernel Maintainer Summit, Tokyo):
- 600,000+ lines of production Rust in drivers, filesystem abstractions, core subsystem bindings
- Android 16 ships with Rust-built ashmem allocator (millions of devices)
- Greg Kroah-Hartman confirmed Rust drivers are proving safer than C counterparts
- **Implication for NullBox:** nulld and kernel-adjacent components can be written in Rust with confidence. Safety-critical paths (Cage microVM management, Egress network filtering) benefit directly.

### io_uring: Unresolved Threat

io_uring remains the most dangerous attack surface in the Linux kernel:
- ARMO's **"Curing" rootkit PoC** (2026) demonstrated io_uring completely bypasses system call monitoring — invisible to Falco, Tetragon, and all eBPF-based security tools
- Google restricts io_uring internally
- 2026 CVEs: CVE-2026-23259 (memory leak), CVE-2026-23113 (kernel panic), CVE-2025-38106 (use-after-free privilege escalation)
- **NullBox decision: CONFIG_IO_URING=n.** Agent workloads are network-bound, not disk-bound. The I/O performance sacrifice is acceptable.

### eBPF Security

- $228,200 grant from Alpha-Omega Foundation for eBPF security hardening
- KASAN enablement for JIT-compiled programs
- Security audits of x86-64, arm64, riscv64 JIT compilers
- Linux 6.18: **cryptographically signed BPF programs** — critical for Watcher audit integrity
- **Tetragon** (Cilium): 1M syscalls/sec at 0.3% CPU (vs Falco's 11.2% — 37x improvement)
- AWS EKS adopted Cilium as default CNI in 2025

### Landlock LSM

- **ABI v6**: Restrict abstract UNIX socket connections and signal sending
- **ABI v7**: Audit event logging controls
- **landrun** tool: 2,156 GitHub stars since March 2025
- **Implication:** Landlock provides additional filesystem sandboxing for Cage VMs. Complements AppArmor.

---

## 2. MicroVM / Isolation Technologies

### Production-Ready MicroVM Runtimes

| Technology | Latest | Boot Time | Key Feature | Language |
|-----------|--------|-----------|-------------|---------|
| **Firecracker** | v1.14 (Dec 2025) | ~125ms | 150 VMs/sec/host, memory hot-plug | Rust |
| **Cloud Hypervisor** | v51.0 (Feb 2026) | ~150ms | Nested virtualization, QCOW2 compression | Rust |
| **libkrun** | Active | <200ms | Library-based KVM, macOS support | Rust |
| **gVisor** | Active | ~ms | Userspace kernel, syscall interception | Go |
| **Kata Containers** | Active | ~300ms | Full VM isolation, Intel TDX/AMD SNP | Go/Rust |

**NullBox uses libkrun** for Cage microVMs. Library-based integration (vs daemon-based Firecracker) simplifies the architecture and reduces overhead. libkrun also supports macOS via Hypervisor.framework for future cross-platform VibeGuard mode.

### microsandbox Innovation: Network-Layer Secret Injection

microsandbox (Zerocore AI, YC X26) introduced **network-layer secret injection**:
- Sandbox receives a random placeholder token for API keys
- microsandbox swaps the real credential at the network layer during verified TLS connections to allowed hosts
- Secrets never exist inside the sandbox environment
- Combined with DNS rebinding protection and cloud metadata blocking

**This is directly analogous to NullBox's Warden + Cage architecture.** NullBox's Warden provides the same placeholder-to-real credential swap, but at the OS level rather than as a daemon.

### Google Agent Sandbox (Kubernetes-Native)

Announced at KubeCon NA 2025 as a Kubernetes SIG Apps subproject:
- Sandbox CRD for AI agent workloads
- Supports gVisor and Kata Containers
- Scale-to-zero for idle agents
- Targets tens of thousands of parallel sandboxes
- **Not an OS** — an orchestration layer on top of Kubernetes

### Docker Sandboxes (Desktop 4.58)

Docker Desktop 4.58 added MicroVM-based isolation:
- Claude Code can run in isolated Docker sandbox environments
- Brings microVM concepts to developer desktops
- Validates the market demand for agent isolation

---

## 3. Immutable OS Landscape

### Purpose-Built Immutable OSes

| OS | Target Workload | Latest | Key Feature |
|----|----------------|--------|-------------|
| **Talos Linux** | Kubernetes | v1.12 (Jan 2026) | No SSH, API-only, 12 binaries total |
| **Bottlerocket** | AWS containers | Active | dm-verity, CIS/FIPS/HIPAA certified |
| **Flatcar Container Linux** | Containers (CNCF) | Active | USR-A/USR-B partition flipping |
| **Fedora CoreOS** | Clusters | Active | Auto-updating, minimal |
| **NullBox** | **AI Agents** | **Planned** | **No SSH, per-agent microVMs, integrated security** |

**NullBox is the only immutable OS targeting AI agents.** Every other immutable OS targets containers or Kubernetes clusters.

### Talos Linux (Closest Architectural Analogue)

Talos v1.12 (Jan 2026) is NullBox's architectural reference:
- Fully immutable, API-driven (no SSH)
- userspace OOM handler
- Read-only registry via talosctl
- Phased provisioning via multi-document configuration
- **Key lesson:** Talos took 5 years with veteran kernel engineers to reach production stability

---

## 4. Agent-Specific OS Research (Emerging Field)

### AIOS (Rutgers University)

Published at COLM 2025 — "LLM Agent Operating System":
- LLM kernel with scheduling, context management, memory management, access control
- Claims 2.1x faster agent execution via OS-level optimization
- 5,403 GitHub stars
- **Academic/research only.** Not focused on security or hardware isolation.

### Agent-OS Blueprint (Academic Paper, Sep 2025)

Proposes a layered architecture for agent operating systems:
- Kernel -> Services -> Agent Runtime -> Orchestration -> User
- Introduces **immutable contracts** for high-stakes operations
- Defines latency classes: Hard Real-Time, Soft Real-Time, Delay-Tolerant
- **Validates NullBox's architectural approach** from an academic perspective

### AgenticOS 2026 Workshop (ASPLOS 2026)

1st Workshop on OS Design for AI Agents, co-located with ASPLOS 2026:
- Exploring OS primitives, isolation models, scheduling for agent workloads
- **Signals that agent OS is being recognized as a distinct research field**

---

## 5. Confidential Computing

### Production Status (March 2026)

| Technology | Vendor | Status | GA Clouds |
|-----------|--------|--------|-----------|
| **AMD SEV-SNP** | AMD | **Production-ready** | Azure, GCP, AWS |
| **Intel TDX** | Intel | **Production-ready** | GCP (c3-standard), expanding |
| **ARM CCA** | ARM | **Pre-production** | Broader deployment expected 2027 |

Market: ~$5.8B in 2025, 38% CAGR, projected >$28B by 2030.

**Implication for NullBox:** Cage microVMs can leverage SEV-SNP/TDX for encrypted VM memory on cloud deployments. Agent code and data protected even from the host hypervisor. Enables enterprise use cases where cloud infrastructure is untrusted.

---

## 6. WebAssembly as Agent Runtime

### WASI Progress

- **WASI Preview 2** (2025): Async ops, better resource management
- **WASI 0.3** in progress
- Potential **WASI 1.0 by end of 2026 or early 2027**
- Capability-based security: explicit capability handles, no direct host access

### Microsoft Wassette (WASM + MCP)

- Agents execute WASM components from OCI registries
- Digital signing and provenance tracking
- Isolation with fine-grained capabilities
- **Validates NullBox's Harbor skill format** (skill.wasm)

### NullBox Position

WASM is used for **Harbor skill portability** (skill.wasm binaries), not as the primary agent runtime. Agents need full Linux userspace inside Cage VMs (filesystem, network stacks, GPU access, process spawning). WASM provides the packaging and distribution format.

---

## 7. Agent Ecosystem Standards

### A2A (Agent-to-Agent Protocol)

- Announced April 2025, current **v0.3**
- Built on HTTP, SSE, JSON-RPC; v0.3 adds gRPC, security card signing
- **100+ technology partners** (Atlassian, Salesforce, SAP, ServiceNow, PayPal)
- Donated to **Linux Foundation** as standalone project
- **NullBox's Swarm layer implements A2A** for cross-agent task delegation

### MCP (Model Context Protocol)

- Donated to **Agentic AI Foundation (AAIF)** under Linux Foundation, Dec 2025
- Co-founded by Anthropic, Block, OpenAI
- Key features: Tasks, OAuth 2.1 (June 2025), Streamable HTTP (March 2025)
- 2026 priorities: stateless HTTP, MCP Server Cards (.well-known), enterprise SSO
- **NullBox's ctxgraph and all modules expose MCP interfaces**

### AAIF (Agentic AI Foundation)

Linux Foundation fund co-founded by Anthropic, Block, OpenAI:
- Governs MCP specification
- Emerging as central body for agent interoperability
- A2A (agent-to-agent) + MCP (agent-to-tool) are complementary standards

### OpenAI Agents SDK

- Replaced Swarm in March 2025; open-source Python + TypeScript, provider-agnostic
- Handoffs, guardrails, sessions, tracing
- Pushed AGENTS.md spec for agent capability declaration
- Participates in AAIF

---

## 8. Edge Hardware Advances

### Raspberry Pi 5 AI Accelerators

| Accessory | NPU | TOPS | RAM | Notes |
|-----------|-----|------|-----|-------|
| AI HAT+ | Hailo-8L | 13-26 | Shared | Original, good for small models |
| **AI HAT+ 2** | **Hailo-10H** | **40** | **8GB dedicated** | Supports LLMs, VLMs, generative AI. Major leap. |

The AI HAT+ 2 makes RPi 5 a viable NullBox edge platform for real AI workloads.

### NVIDIA Jetson

| Model | GPU | Compute | Memory | Power | Released |
|-------|-----|---------|--------|-------|----------|
| Orin Nano | Ampere | 40 TOPS | 8GB | 7-15W | 2023 |
| **AGX Thor** | **Blackwell** | **2,070 FP4 TFLOPS** | **128GB** | **40-130W** | **Aug 2025** |

AGX Thor is 7.5x more powerful than Orin. Makes on-device inference of large models practical.

### RISC-V

Not yet viable for production agent workloads:
- MIPS S8200: RISC-V NPU for transformer/agentic models at edge (Jan 2026)
- 1-3B parameter LLMs demonstrated with NPU acceleration
- ARM performance parity projected end of 2026
- **Monitor for NullBox support in 2027**

---

## 9. Key Technology Decisions for NullBox

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Kernel | Linux 6.18 LTS | Signed eBPF, production Rust, newer allocator |
| io_uring | Disabled | Unresolved security risk, agent workloads are network-bound |
| MicroVM | libkrun | Library-based (not daemon), macOS future, Rust native |
| Skill format | WASM (skill.wasm) | Portable, sandboxed, OCI-compatible |
| Agent communication | A2A v0.3 | Linux Foundation backed, 100+ partners |
| Tool protocol | MCP | AAIF standard, Anthropic/OpenAI/Block backed |
| Confidential computing | AMD SEV-SNP / Intel TDX | Production-ready, enables zero-trust cloud |
| Edge NPU | Hailo-10H (RPi), CUDA (Jetson) | 40 TOPS on Pi 5, 2070 TFLOPS on Thor |
| Build toolchain | Clang + ThinLTO | Required for CFI, ~5% perf improvement |
| Init system | nulld (Rust) | Replaces systemd, manages all agent services |
