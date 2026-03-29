# Forge — Agent Testing & Simulation

> Verifiable agent behavior before production deployment. Mathematical proof of what a skill can and cannot do.

**Layer:** NullBox Layer 13

---

## The Gap

No current system tests agent behavior before it runs in production. Malicious skills can burn API quota, loop infinitely, or take unintended actions. There is no way to verify a skill's behavior before installing it.

---

## What Forge Does

1. **Hooks into Harbor** at publish time — every skill is Forge-tested before listing
2. **Dry-runs skills** in a simulated Cage microVM with:
   - Mock Warden (fake credential responses)
   - Mock Egress (logs attempted destinations without hitting them)
   - Mock ctxgraph (simulated memory state)
3. **Generates a capability proof:** a signed document stating:
   > "This skill, given these inputs, made requests only to these destinations, wrote only to these paths, never exceeded these resource limits — verified in N simulation runs"
4. **Generates anomaly baselines** used by Watcher: if the skill starts behaving outside the Forge-verified profile in production, alert immediately
5. **libkrun snapshot support:** Snapshot a microVM mid-execution and replay from that point — useful for edge case testing

---

## Community Integration with Harbor

Forge proofs are public. Before installing a skill, users can see:
- What it did in simulation
- What destinations it tried to reach
- What it wrote
- The Forge proof + Ed25519 signature

Together, these are substantially stronger than any manual code review.

---

## Why This Matters

The Forge proof + Harbor signature + Cage enforcement creates a complete trust chain:

```
Forge verifies behavior -> Harbor signs and publishes -> Cage enforces declared capabilities at runtime
```

A malicious skill has to fool Forge simulation, forge an Ed25519 signature, AND escape Cage microVM isolation. Each layer is independent. All three must be breached.
