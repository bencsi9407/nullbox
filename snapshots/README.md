# Snapshots — Agent State Backup & Restore

> Agent memory and accumulated state is valuable. It should be backed up, versioned, and restorable.

**Layer:** NullBox Layer 16

---

## What It Does

- `nullctl snapshot create [agent-name]` — atomic snapshot of microVM state + ctxgraph namespace + Warden vault references (not the secrets themselves)
- Snapshots are **content-addressed, Provenance-signed, encrypted** with the machine's TPM-bound key
- `nullctl snapshot restore [snapshot-id]` — restore agent to any prior state
- Automated snapshot schedule configurable per agent in AGENT.toml
- Full-machine snapshots for disaster recovery: entire NullBox state exportable as encrypted archive

---

## Cross-Hardware Portability

Structured, versioned, encrypted, portable across hardware. A snapshot taken on an RPi 5 can be restored on an x86 VPS.

---

## Used By Other Layers

| Layer | How it uses Snapshots |
|---|---|
| **Phoenix** | Snapshots agent state before restart attempts; restores on failure |
| **Forge** | Snapshot mid-execution for simulation replay and edge case testing |
| **Swarm** | Consistent initial state across nodes when deploying swarm agents |
| **Gate** | Snapshot agent state before VM suspension, enabling state recovery if the suspended action is denied |

---

## CLI

```bash
# Create snapshot
nullctl snapshot create researcher
# Output: snapshot-researcher-2026-03-25-143201 (encrypted, signed)

# List snapshots
nullctl snapshot list researcher
# Output: 12 snapshots, oldest: 2026-03-10, newest: 2026-03-25

# Restore
nullctl snapshot restore snapshot-researcher-2026-03-25-143201

# Full machine backup
nullctl snapshot full --output /mnt/usb/backup.enc

# Automated schedule in AGENT.toml
# [snapshots]
# schedule = "0 */6 * * *"    # Every 6 hours
# retention = 30               # Keep last 30 snapshots
```
