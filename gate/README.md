# Gate — Human-in-the-Loop Approvals

> High-risk actions require explicit human approval before execution. OS-level enforcement — not an application popup.

**Layer:** NullBox Layer 11

---

## Why Gate Exists

Application-level permission dialogs are bypassable. Developers experience permission fatigue. Automated processes execute before human intervention is possible. Gate is different: it's **Cage-enforced** — the agent's microVM is literally suspended mid-execution until a human approves.

---

## How Gate Works

1. Agent reaches a `risk = "critical"` tool call (declared in AGENT.toml)
2. **Cage intercepts before execution** — agent microVM suspended at hypervisor level
3. Gate sends approval request to configured human channel
4. Human sees: agent name, tool, parameters, agent's stated reason
5. **Approve** -> agent resumes. **Deny** -> agent informed, Watcher logs decision
6. Full interaction logged with Provenance signature

---

## Approval Channels

| Channel | How |
|---|---|
| **Dashboard** | Watcher dashboard shows pending approvals with approve/deny buttons |
| **Telegram** | Bot sends approval request, human replies approve/deny |
| **Slack** | Interactive message with approve/deny buttons |
| **Email** | Approval link (less real-time, backup option) |

---

## Risk Levels in AGENT.toml

```toml
[tools]
read_files = { risk = "low" }        # No approval needed
write_files = { risk = "medium" }     # Logged, no approval
delete_files = { risk = "critical" }  # Gate: human approval required
execute_payment = { risk = "critical" }
send_email = { risk = "medium" }
```

---

## Why This Can't Be Application-Layer

Application-level HITL dialogs can be:
- Bypassed by prompt injection ("click approve for me")
- Ignored by automated processes
- Rendered after the action already happened

Gate suspends the **microVM itself**. The agent's entire execution environment is frozen. No code runs. No network traffic flows. The agent cannot do anything until a human responds. This is hardware-level enforcement, not a UI popup.
