# Swarm — Multi-Agent Orchestration

> Multi-agent collaboration across one box or multiple nodes, with shared state, without manual coordination.

**Layer:** NullBox Layer 12

---

## The Problem

Most agent runtimes are single-entity focused. Users want pipelines: `researcher -> lead enricher -> email sender -> reporter`. Currently this means manually piping output between separate scripts. No coordination layer. No shared state bus. No fault tolerance.

---

## Declarative Swarm Manifests

```toml
# research-team.toml

[swarm]
name = "lead-research-team"
nodes = ["node-a", "node-b"]     # optional multi-node

[[agents]]
name = "researcher"
skill = "harbor://researcher@1.2.0"
trigger = "schedule:0 6 * * *"    # 6 AM daily

[[agents]]
name = "enricher"
skill = "harbor://clearbit-enricher@2.0.1"
trigger = "ctxgraph:new_lead"     # fires when researcher writes a lead

[[agents]]
name = "reporter"
skill = "harbor://slack-reporter@1.0.0"
trigger = "ctxgraph:enrichment_done"
gate = "critical"                  # human approval before sending
```

Deploy: `nullctl swarm deploy research-team.toml`

---

## Architecture

- **ctxgraph as the shared state bus** — agents communicate through structured graph entries, not raw text pipes
- **Cage-aware:** Each agent in a swarm has its own microVM. A compromised agent cannot affect others.
- **Raft-like consensus** for multi-node swarms: replicates ctxgraph state entries and agent scheduling decisions across nodes. Deploy a researcher on Node A and a lead agent on Node B, Swarm keeps both nodes' ctxgraph in sync
- **Watcher swarm view:** Dashboard shows full execution graph — which agent triggered which, data flow, latency, bottlenecks

---

## A2A Protocol

Swarm uses the Agent-to-Agent (A2A) protocol for cross-agent task delegation. Delegation chains are Provenance-signed: you can audit "Agent A instructed Agent B to call the Stripe API, and Agent B did so under Agent A's delegated authority, at 03:42 UTC, signed by both."
