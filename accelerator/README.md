# Accelerator Manager — GPU / NPU / TPU Isolation

> Hardware acceleration is the difference between an agent that thinks in 50ms vs 5 seconds. NullBox handles this at the hypervisor level with proper isolation.

**Layer:** NullBox Layer 3

---

## What It Does

- Agents declare `accelerator = "npu:hailo"` or `"cuda:0"` or `"metal"` in AGENT.toml
- Cage pins the declared accelerator to that specific microVM — **other agents cannot access it**
- Accelerator quotas: `max_gpu_memory_mb`, `max_compute_percent`
- Hardware drivers compiled into kernel as signed built-ins (CONFIG_MODULES=n still holds)
- ctxgraph vector index operations offloaded to NPU/GPU when available

---

## Built-In Local LLM (Ollama)

Ollama is a first-class OS service in NullBox:

- `nullctl llm enable ollama` — starts as nulld service at boot
- Model storage in EPHEMERAL partition — survives reboots, cleared on full reset
- Agents use `LOCAL_LLM_URL=http://localhost:11434`
- Warden applies credential proxy to local LLM traffic too (rate limiting, audit)
- **Automatic cloud fallback:** When cloud API is unavailable (offline mode), agents automatically fall back to local Ollama

---

## Supported Hardware

| Hardware | Accelerator | Notes |
|---|---|---|
| **RPi 5 + AI HAT+ 2** | NPU, **40 TOPS** (Hailo-10H), 8GB dedicated RAM | Best edge target. Supports LLMs, VLMs, generative AI on Pi 5. Major leap from original 13 TOPS Hailo-8L. |
| **RPi 5 + AI HAT+** | NPU, 13-26 TOPS (Hailo-8L) | Budget edge option. Small models. |
| **Jetson AGX Thor** | CUDA, **2,070 FP4 TFLOPS**, 128GB memory | New flagship (Aug 2025). Blackwell GPU. 7.5x compute over Orin. 40-130W. |
| **Jetson Orin Nano** | CUDA, 40 TOPS | Previous gen, still excellent for local inference. |
| **x86 + NVIDIA GPU** | CUDA | VPS with GPU, or dedicated workstation. |
| **x86 no GPU** | CPU only | Ollama on CPU, smaller models (Qwen2.5 1.5B). |
| **RISC-V + NPU** | NPU (emerging) | Not yet viable for production. MIPS S8200 demonstrated 1-3B parameter models. Monitor for 2027. |
