# Edge Power Manager — Offline & Low-Power Mode

> Running 24/7 on a Raspberry Pi shouldn't cost $50/month in electricity. Going offline on a plane shouldn't break your agents.

**Layer:** NullBox Layer 15

---

## Power Management

- **Kernel-level cpufreq governor** tuned for agent workloads: aggressive idle, burst on activation
- **Idle agent suspension:** Cage microVMs for idle agents suspended (state in memory, CPU released)
- **Watcher batch-writes** to reduce EPHEMERAL partition writes and extend SD card lifespan
- `nullctl power set economic` — reduces all CPU quotas by 50%, halves network polling
- **Target:** RPi 5 running 3 background agents at under 3W average

---

## Offline Mode

When internet connectivity is lost (detected by Egress):

1. Warden stops injecting cloud API credentials (no cloud APIs to reach). Agents using local-only tools continue unaffected.
2. All cloud LLM calls **automatically re-routed to local Ollama**
3. ctxgraph continues locally — reads and writes work normally
4. Agents requiring specific external APIs are paused with state snapshot
5. When connectivity restored: queued actions replayed, agents resumed, Watcher synced

---

## Travel / Air-Gap Mode

```bash
nullctl offline --duration 8h
```

Pre-downloads models, caches necessary data, prepares agents for extended disconnection. Deterministic behavior, no surprise failures.
