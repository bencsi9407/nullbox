# Market Analysis

> Where the moat actually lives, who pays, and when.

---

## Market Position: Nobody Is Building This

As of March 2026, the agent infrastructure market breaks down as:

| Layer | Who's There | What's Missing |
|-------|-------------|----------------|
| Container images | Chainguard ($892M raised), Wolfi | Agent-specific images |
| Host OS | Talos, Bottlerocket | Agent-aware immutable OS |
| Sandbox runtime | E2B, Daytona, microsandbox, Fly.io | OS-level integration |
| Agent framework | AIOS, LangChain, CrewAI | Security-first design |
| K8s orchestration | K8s Agent Sandbox | Not a standalone solution |

**Nobody is building a full OS purpose-built for agents.** Everyone builds sandboxes *on top of* general-purpose Linux. NullBox collapses the entire stack into a single bootable image.

---

## Competitive Funding Landscape

| Company | Total Raised | Valuation | Stage |
|---------|-------------|-----------|-------|
| Chainguard | $892M | $3.5B | Series D |
| Modal | $111M+ | $1.1B (seeking $2.5B) | Series B+ |
| Daytona | $31M total ($24M Series A) | Undisclosed | Series A |
| E2B | $25M+ | Undisclosed | Series A |
| microsandbox/Zerocore | YC X26 | Early | Seed |

Agent infrastructure attracted ~$2.8B in H1 2025. Big Tech projected to invest $500B+ in AI infrastructure in 2026.

---

## Direct Competitors

### E2B — Cloud Sandbox Runner
- **Funding:** $21M Series A (July 2025). 40K to 15M sandboxes/month in one year.
- **Architecture:** Firecracker microVMs. ~150ms cold starts. Cloud-only.
- **Pricing:** Per-second. ~$0.05/hr/vCPU. $100 free credit.
- **Limitations:** No GPU (Firecracker lacks PCIe passthrough). 24-hour max session. No local option. No credential isolation. No PII redaction.
- **Where NullBox wins:** Local-first. Full security stack. GPU isolation. Persistent agents. The OS itself.

### Daytona — Composable Sandbox Infrastructure
- **Funding:** $24M Series A (Feb 2026). $1M ARR in under 3 months.
- **Architecture:** Docker containers (not microVMs). Sub-90ms creation. GPU support.
- **Pricing:** Per-second. ~$0.05-0.067/hr/vCPU. $200 free credit.
- **Limitations:** Docker isolation (shared kernel). No credential vault. No prompt inspection.
- **Where NullBox wins:** Hardware-level isolation (microVMs). Integrated security. Immutable OS.

### microsandbox — Local-First MicroVM Sandbox
- **Funding:** YC X26 batch. Early stage.
- **Architecture:** libkrun microVMs. Sub-200ms boot. **Network-layer secret injection** — sandbox sees random placeholder; real credential swapped at network layer only for verified TLS.
- **Limitations:** A daemon on a host OS, not an OS primitive. No prompt injection firewall. No audit trail.
- **Where NullBox wins:** Full OS integration. Sentinel + Watcher + CloakPipe + Provenance layers don't exist in microsandbox.

### Fly.io Sprites — Persistent Sandbox VMs
- **Architecture:** Firecracker. 1-2s creation. 100GB persistent NVMe. Auto-idle when inactive.
- **Pricing:** $0.07/CPU-hr. No charge when idle.
- **Limitations:** Cloud-only. No integrated security. No credential vault.
- **Where NullBox wins:** Local-first. Full security stack.

---

## Specific Gaps NullBox Fills

1. **No OS-level agent runtime exists.** Talos removed SSH for Kubernetes. Nobody has done the equivalent for agents.

2. **No unified security model.** microsandbox has secret injection (one feature). Nobody offers: immutable filesystem + secret injection + network policy + audit logging + cryptographic attestation as an integrated OS.

3. **Local-first + cloud parity is underserved.** microsandbox is the only local-first option. E2B, Daytona, Fly.io are cloud-only.

4. **GPU isolation for agents is unsolved.** Firecracker has no GPU passthrough. Daytona supports GPU but uses Docker.

5. **Compliance gap.** Bottlerocket has CIS/FIPS/HIPAA. Nobody in agent sandboxing offers compliance-ready infrastructure for regulated industries.

---

## Target Market Segments

### Now — Cohort 1: Indie Developers & AI Hackers
- Power users running agents 24/7
- **Pain:** API key theft, no audit trail, compliance anxiety
- **WTP:** $0-19/mo
- **Size:** ~30,000+ active agent users, growing

### 6-12 Months — Cohort 2: SMBs & Agencies
- Marketing agencies, dev shops deploying internal agents
- **Pain:** Compliance requirements, agent visibility, cost control
- **WTP:** $79-299/mo
- **Size:** Rapidly expanding

### 12-24 Months — Cohort 3: Enterprise
- Fortune 500 deploying autonomous agents at scale
- **Pain:** SOC2 compliance, supply chain security, agent isolation
- **WTP:** Custom pricing ($1,000+/mo)
- **Trigger:** By end of 2026, 40% of enterprise apps expected to run embedded AI agents

---

## Revenue Streams

### VibeGuard (Pre-OS)

| Tier | Price | Target |
|---|---|---|
| **Free** | $0 | Indie devs (1 agent) |
| **Builder** | $19/mo | Multi-agent devs |
| **Team** | $79/mo | Agencies, 5 seats |
| **Studio** | $299/mo | Dev shops |

### NullBox (Post-OS)

| Stream | Description |
|---|---|
| **Open source core** | MIT/Apache licensed. Security story = growth engine. |
| **Managed cloud** | Hybrid Cloud Bridge, managed Swarm, cloud Harbor |
| **Enterprise** | Forge compliance reports, Phoenix SLAs, HSM Provenance, private Harbor |
| **Hardware** | Pre-flashed NullBox Appliance: RPi 5 + AI HAT+ 2 (40 TOPS) + NullBox |

---

## Key Market Signal

48% of cybersecurity professionals identify agentic AI as the most dangerous attack vector (2026 survey). OWASP published an AI Agent Security Top 10 for 2026. The market is begging for security-first agent infrastructure.
