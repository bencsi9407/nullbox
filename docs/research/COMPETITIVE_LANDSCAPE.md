# Competitive Landscape

> What exists, what doesn't, and why NullBox is architecturally distinct.

---

## Direct Competitors (Agent Sandbox / Execution)

### E2B — Cloud Sandbox Runner

- **What it is:** Cloud-hosted sandboxes for AI agents using Firecracker microVMs
- **Funding:** $21M Series A (July 2025). 88% of Fortune 100 signed up. Grew from 40K to 15M sandboxes/month in one year.
- **Architecture:** Firecracker microVMs. ~150ms cold starts. Dedicated kernel per sandbox. Cloud-hosted only (US/EU regions). BYOC on AWS for enterprise.
- **Pricing:** Per-second. ~$0.05/hr/vCPU. Hobby: free ($100 credit). Pro: $150/mo. Enterprise: custom.
- **Limitations:** No GPU support (Firecracker lacks PCIe passthrough). 24-hour max session (5-10 min typical). No local option. No credential isolation. No PII redaction. No prompt inspection.
- **Where NullBox wins:** Local-first. Full security stack (Warden + Sentinel + CloakPipe). GPU isolation. Persistent agents. The OS itself.

### Daytona — Composable Sandbox Infrastructure

- **What it is:** "Composable computers for AI agents." Pivoted from dev environments to agent infrastructure early 2025.
- **Funding:** $24M Series A (Feb 2026). $1M ARR in under 3 months, doubled it in 6 weeks. Customers: LangChain, Turing, Writer, SambaNova.
- **Architecture:** Docker containers (not Firecracker). Sub-90ms creation. CPU, memory, storage, GPU all configurable. Start, pause, snapshot at any point.
- **Pricing:** Per-second. ~$0.05-0.067/hr/vCPU. $200 free credit. Startups can apply for up to $50K in credits.
- **Limitations:** Docker isolation (shared kernel — weaker security boundary). No credential vault. No prompt inspection. No PII redaction. Cloud-only.
- **Where NullBox wins:** Hardware-level isolation (microVMs > Docker containers). Integrated security stack. Immutable OS. Local-first.

### microsandbox — Local-First MicroVM Sandbox

- **What it is:** Open-source, local-first sandbox platform for AI agents. YC X26 batch.
- **Architecture:** Uses libkrun microVMs (same as NullBox Cage). Sub-200ms boot. OCI-compatible. Rust-based.
- **Key innovation:** **Network-layer secret injection** — sandbox sees a random placeholder token for API keys; real credentials swapped at the network layer only for verified TLS connections to allowed hosts. Secrets never exposed inside sandbox.
- **Limitations:** Runs as a daemon on host OS — not an OS primitive. No prompt injection firewall. No PII redaction. No audit trail. No agent lifecycle management.
- **Where NullBox wins:** Full OS integration. Warden (richer than microsandbox's secret injection). Sentinel (prompt firewall). CloakPipe (PII). Watcher (audit). Provenance (attestation). Phoenix (self-healing).
- **Note:** microsandbox validates NullBox's core architecture (libkrun microVMs + credential isolation). NullBox goes further by making it an OS primitive and adding 10+ additional security layers.

### Fly.io Sprites — Persistent Sandbox VMs

- **What it is:** Persistent stateful sandbox VMs for AI agents (launched Jan 2026)
- **Architecture:** Firecracker microVMs. 1-2s creation. Checkpoint/restore ~300ms. 100GB persistent NVMe per sandbox. Auto-idle when inactive.
- **Pricing:** $0.07/CPU-hr, $0.04375/GB-hr memory. No charge when idle.
- **CEO quote:** "Ephemeral sandboxes are obsolete. Claude doesn't want a stateless container. Claude wants a computer."
- **Limitations:** Cloud-only. No integrated security. No credential vault. No prompt inspection.
- **Where NullBox wins:** Local-first. Full security stack. Immutable OS. Per-agent network isolation.

### Modal — Serverless AI Compute

- **Funding:** $111M total, $87M Series B (Oct 2025, $1.1B valuation). In talks for $2.5B valuation. ~$50M ARR.
- **Architecture:** Serverless functions + sandboxes. Container-based. GPU support (NVIDIA).
- **Pricing:** Starter: $30/mo free credit. ~$0.142/CPU-core/hr.
- **Relevance:** General-purpose AI compute platform, not agent-specific. Offers sandbox primitives but focused on batch inference and model serving.
- **Where NullBox wins:** Purpose-built for agents. Security-first. Local-first.

---

## Security-Focused Infrastructure

### Chainguard / Wolfi OS

- **Funding:** $892M total. $356M Series D (April 2025) at $3.5B valuation. ~$40M ARR.
- **What it is:** 1,300+ container images built on Wolfi OS with zero known CVEs. Average CVE remediation under 20 hours.
- **Difference:** Secures the container image supply chain, not the runtime. Solves "what goes into the image" not "how it runs securely."
- **Potential synergy:** NullBox could use Chainguard images as base for Cage microVMs.

### Bottlerocket (AWS)

- **What it is:** AWS's immutable, minimal Linux OS for hosting containers.
- **Architecture:** Read-only root with dm-verity. No SSH, no package manager. Atomic updates with rollback. API-managed.
- **Certifications:** CIS Benchmark, FIPS 140-3, PCI DSS, HIPAA eligible.
- **Difference:** Designed for running containers on Kubernetes, not running AI agents.

### Talos Linux (Sidero Labs)

- **What it is:** Immutable, API-driven Linux OS for Kubernetes. Latest: v1.12 (Jan 2026).
- **Architecture:** 12 binaries total. No SSH. Signed kernel modules only. CNCF AI Conformance certified. NVIDIA GPU support via extensions.
- **Our analogy:** NullBox is Talos Linux for AI agents — same philosophy (subtractive OS), different workload target.
- **Key lesson:** Talos took 5 years with veteran kernel engineers.

---

## Emerging "Agent OS" Concepts

### Kubernetes Agent Sandbox (kubernetes-sigs/agent-sandbox)

- Official Kubernetes SIG Apps subproject backed by Google (KubeCon Atlanta, Nov 2025)
- Sandbox CRD for AI agent workloads. gVisor + Kata Containers. Scale-to-zero.
- Not an OS — a Kubernetes orchestration layer.

### AIOS (Rutgers University)

- 5,403 GitHub stars. Published at COLM 2025. "LLM is the kernel, agents are apps."
- AIOS kernel provides scheduling, context management, memory management.
- Academic/research only. Not security-focused. Software framework, not an actual OS.

### AgenticOS 2026 Workshop (ASPLOS 2026)

- 1st Workshop on OS Design for AI Agents
- Signals agent OS is being recognized as a distinct research field

---

## Why No Existing Product Can Replicate NullBox

NullBox is not a feature. It's an architecture.

| Product | Isolation | Credentials | PII | Prompt Firewall | Audit | Local-First | Immutable OS |
|---|---|---|---|---|---|---|---|
| **E2B** | microVM | -- | -- | -- | -- | -- | -- |
| **Daytona** | Docker | -- | -- | -- | -- | -- | -- |
| **microsandbox** | microVM | Secret injection | -- | -- | -- | Yes | -- |
| **Fly.io** | microVM | -- | -- | -- | -- | -- | -- |
| **Talos** | N/A (K8s host) | -- | -- | -- | -- | -- | Yes |
| **NullBox** | **microVM** | **Warden vault** | **CloakPipe** | **Sentinel** | **Watcher** | **Yes** | **Yes** |

The moat is integration + enforcement at the OS level. Any single feature can be replicated. The full stack — Warden + Sentinel + CloakPipe + Watcher + Cage + Egress + ctxgraph + Forge + Harbor, all integrated and enforced at the kernel — cannot be assembled from parts.

---

## Funding Landscape Summary

| Company | Total Raised | Valuation | Stage |
|---------|-------------|-----------|-------|
| Chainguard | $892M | $3.5B | Series D |
| Modal | $111M+ | $1.1B (seeking $2.5B) | Series B+ |
| Daytona | $31M | Undisclosed | Series A |
| E2B | $25M+ | Undisclosed | Series A |
| microsandbox/Zerocore | YC X26 | Early | Seed |

Agent infrastructure attracted ~$2.8B in H1 2025. The category is well-funded and growing fast. NullBox occupies a unique position: nobody else is building the OS layer.
