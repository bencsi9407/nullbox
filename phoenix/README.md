# Phoenix — Self-Healing & Auto-Recovery

> Agents crash. Networks drop. Models hallucinate infinite loops. At 3 AM nobody is watching. Phoenix makes NullBox self-healing.

**Layer:** NullBox Layer 14

---

## What Phoenix Does

- **eBPF probes** monitor agent health: CPU anomalies, memory growth, stuck syscalls, network timeouts
- **Anomaly detection** -> pause agent -> snapshot microVM state -> attempt restart from snapshot
- **Escalation:** If restart fails twice -> reclaim microVM -> Watcher logs full context -> alert to human
- **ctxgraph state preserved** across recovery — agent picks up where it left off with full memory

---

## Self-Healing Scenarios

### Infinite LLM Retry Loop
```
Phoenix detects: CPU >95% for >5 minutes
Action: Pause -> snapshot -> check ctxgraph for loop detection
        -> resume with rate limit temporarily halved -> Watcher alert
```

### Network Partition
```
Egress detects: all outbound connections failing for >2 minutes -> notifies Phoenix
Phoenix action: Offline mode engaged -> agent switches to local Ollama
               -> resumes operation -> Watcher logs network event
```

### Skill Bug Causes OOM
```
Phoenix detects: memory limit hit -> microVM killed by hypervisor
Action: Restore from last snapshot -> resume with reduced memory budget
        -> alert if it happens 3x in 24h
```

---

## OTA State Migration

When NullBox updates to a new version, Phoenix migrates running agent states to the new microVM format. Agents don't need to restart from scratch after an OS update.

---

## Integration Points

| Layer | Connection |
|---|---|
| **Cage** | Phoenix pauses/resumes/reclaims microVMs |
| **Watcher** | Every recovery event logged with full context |
| **ctxgraph** | Memory preserved across recovery cycles |
| **Snapshots** | Phoenix uses snapshot primitives for state preservation |
| **Edge Power** | Network partition triggers offline mode via Phoenix |
