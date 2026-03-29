# Provenance Vault — Cryptographic Action Attribution

> Every action every agent takes is cryptographically attributed. Tamper-proof proof of what happened, who did it, and with what authority.

**Layer:** NullBox Layer 9

---

## The Gap

Compliance demands tamper-proof action attribution. Enterprise security teams need to answer: "Which specific agent, running which version of which skill, performed this action at this time, with what permissions?" No current agent runtime can answer this.

---

## How It Works

- Every agent gets a **persistent Ed25519 keypair** at install time, stored in the Warden vault
- Every action is signed: `{agent_id + manifest_hash + action + params + timestamp} -> Ed25519 signature`
- Signatures exported to Watcher's Merkle log
- **TPM 2.0-bound** where available: keys are hardware-rooted, cannot be extracted or impersonated even with full OS access

---

## Swarm Provenance

When agents delegate tasks (Swarm layer), the delegation chain is provenance-signed:

```
Agent A instructed Agent B to call the Stripe API
Agent B did so under Agent A's delegated authority
At 03:42 UTC
Signed by both A and B
```

Full delegation chain auditable by any party holding the public keys.

---

## Compliance Exports

Full provenance chains exportable as JSONL, verifiable by any party holding the public keys.

```jsonl
{"agent":"researcher","action":"web_fetch","params":{"url":"api.exa.ai/search"},"timestamp":"2026-03-24T14:23:01Z","manifest_hash":"abc123","signature":"base64..."}
```

---

## Enterprise Extensions

- **HSM instead of TPM** for stricter compliance requirements
- **External verification service** — third parties can verify provenance chains without accessing the OS
- **SOC2 attestation integration**
