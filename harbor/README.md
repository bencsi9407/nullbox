# Harbor — Verified Skill Registry

> The npm for AI agents, except every package has a cryptographic capability proof and enforcement at install time.

**Layer:** NullBox (Harbor) — spans multiple phases

---

## Why Harbor Exists

Unverified skill registries have been compromised at scale. Harbor is the fix: every skill is signed, every skill has a Forge capability proof, and every skill's declared capabilities are enforced at runtime by Cage.

---

## Trust Chain

```
Author signs skill with Ed25519 keypair
    -> Forge dry-runs skill in simulated microVM
        -> Forge generates capability proof (signed)
            -> Harbor publishes skill + proof
                -> User installs skill
                    -> Cage enforces declared capabilities at runtime
```

---

## Skill Format

```
harbor://researcher@1.2.0

researcher/
  AGENT.toml          # Capability manifest
  skill.wasm          # Skill binary (WASM for portability)
  README.md           # Documentation
  FORGE_PROOF.sig     # Forge capability proof (signed)
  SIGNATURE.sig       # Author Ed25519 signature
```

---

## Version Update Safety

When a skill publishes a new version, Harbor generates a **capability diff:**

```
researcher@1.2.0 -> researcher@1.3.0
  + network.allow: api.newservice.com    (NEW — not in previous version)
  ~ max_memory_mb: 512 -> 768            (INCREASED)
  = filesystem.read: /data/research      (UNCHANGED)
```

User must explicitly approve the diff before the update applies. Silent permission escalation is structurally impossible.

---

## Phased Build

### Phase 1: Harbor Lite
- Curated directory of 20-30 hand-picked, verified skills
- Ed25519 signing on every skill
- Basic capability manifests (AGENT.toml)
- No community submissions yet

### Phase 2: Full Harbor
- Community submissions
- Forge verification proofs on every submission
- Capability diff alerts on version updates
- Reputation scores for authors
- Search and discovery

### Phase 3: Cloud Harbor
- Enterprise: private skill registry with internal audit
- Cloud + local registry — same Ed25519 trust chain
