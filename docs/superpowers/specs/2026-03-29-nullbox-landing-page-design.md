# NullBox Landing Page — Design Spec

## Overview

A waitlist landing page for NullBox, the immutable Linux OS for AI agents. Separate Astro project at `/home/rsx/Desktop/projx/nullbox-site`.

**Goals (priority order):**
1. Build hype — "something powerful is coming" for DevOps/AI agent engineers
2. Establish technical credibility — show architectural depth, attract cofounders/kernel engineers
3. Light community touch — GitHub link, open-source positioning

**Brand voice:** Dark luxury as the dominant vibe, with cold/militant edge, quiet confidence substance, and monospace hacker-ethos in details.

---

## Tech Stack

- **Framework:** Astro (static site generation)
- **3D/ASCII:** Three.js with AsciiEffect for the hero visual
- **Styling:** Tailwind CSS v4
- **Fonts:** JetBrains Mono (headings, labels, nav, code) + Inter (body text)
- **Waitlist:** Form posts to Formspree, Buttondown, or a Cloudflare Worker (TBD by user)
- **Hosting:** Static deploy (Vercel, Cloudflare Pages, or Netlify)
- **No JS frameworks.** Vanilla JS + Astro islands for Three.js canvas only.

---

## Visual System

### Palette — Pure Monochrome

| Token | Value | Usage |
|-------|-------|-------|
| `--bg-base` | `#09090b` | Page background |
| `--bg-elevated` | `#141414` | Cards, panels |
| `--bg-glass` | `rgba(255,255,255, 0.03)` | Glassmorphism fill |
| `--border-subtle` | `rgba(255,255,255, 0.06)` | Default borders |
| `--border-glass` | `rgba(255,255,255, 0.10)` | Glass panel borders |
| `--text-primary` | `#e4e4e7` | Headings, primary text |
| `--text-secondary` | `#a1a1aa` | Body text, descriptions |
| `--text-dim` | `#71717a` | Labels, metadata |
| `--text-ghost` | `#3f3f46` | Decorative numbers, bg text |

Zero chromatic color. Anywhere. No green, no cyan, no purple. Depth through opacity only.

### Borders

- All corners sharp. `border-radius: 0` everywhere.
- Border width: `1px solid var(--border-subtle)` default, `var(--border-glass)` for interactive/glass elements.

### Glassmorphism

- `backdrop-filter: blur(12px)` on cards and panels
- `background: var(--bg-glass)`
- `border: 1px solid var(--border-glass)`
- Extremely subtle — smoke-grey tint, not frosted white
- Fallback: solid `var(--bg-elevated)` for browsers without `backdrop-filter`

### Depth

- Radial gradients: `radial-gradient(ellipse at <offset>, rgba(255,255,255, 0.02-0.04), transparent)` placed off-center behind content sections
- No box-shadows. Depth through opacity contrast and layering only.

### Typography

- **Headings/Nav/Labels:** JetBrains Mono, all caps where appropriate, `letter-spacing: 0.1-0.2em`
- **Body:** Inter, regular weight, `--text-secondary` color
- **Code:** JetBrains Mono, monochrome syntax (white, grey, dim grey — no colored syntax highlighting)
- **Section labels:** `font-size: 11px`, uppercase, `letter-spacing: 0.15em`, `--text-dim`

### Animations

- **Scroll reveals:** Fade-in via IntersectionObserver. `opacity: 0 → 1`, `translateY: 12px → 0`, `duration: 0.4s`, `ease: cubic-bezier(0.25, 0.1, 0.25, 1)`. Staggered for grouped items.
- **Hero 3D:** Continuous slow rotation of ASCII cube, periodic fracture into microVM grid.
- **No bouncing, spring physics, or playful motion.** Things appear — they don't perform.
- **Hover states:** Border opacity increase (`0.06 → 0.15`), subtle. No color shifts.

---

## Page Sections

### 1. Nav (fixed)

```
[NULLBOX]                                            [GITHUB ↗]
─────────────────────────────────────────────────────────────────
```

- Fixed top, `z-index: 50`
- `NULLBOX` in JetBrains Mono, left-aligned, letter-spaced
- `GITHUB` link right-aligned, opens in new tab
- Thin bottom border `var(--border-subtle)`
- Transparent on load, glass background (`backdrop-filter`) on scroll past hero

### 2. Hero (full viewport)

**Background:** Three.js canvas, full viewport. A rotating cube built from ASCII digits (0, 1, and box-drawing characters) rendered via Three.js `AsciiEffect`. Monochrome — white characters on `#09090b`. The cube periodically fractures: faces break apart and reform into a grid of smaller cubes (representing microVMs/isolated agents), then coalesce back. Slow, hypnotic, continuous.

**Overlay content (centered):**
```
the operating system
for AI agents

no ssh. no shell. no attack surface.
just agents.

[email input        ] [JOIN WAITLIST]
```

- Headline: JetBrains Mono, large (clamp ~3-5rem), `--text-primary`, lowercase
- Subline: Inter, `--text-secondary`, smaller
- Email input: sharp borders, glass background, monospace placeholder text
- Button: sharp borders, `--text-primary` on `--bg-elevated`, border `var(--border-glass)`

### 3. The Problem

Three statements stacked vertically, large monospace type, generous vertical spacing (~120px between):

```
your agents run on an OS built for humans.

ssh exists. a shell exists. an attack surface exists.

every layer you don't need is a layer that can break.
```

- JetBrains Mono, `clamp(1.2rem, 2.5vw, 2rem)`
- `--text-primary`
- Fade-in on scroll, staggered

### 4. What Doesn't Exist

Section label: `00 — WHAT DOESN'T EXIST`

A grid (2-3 columns on desktop, 1 on mobile) of items. Each item is a sharp-bordered cell:

```
┌─────────────────────┐
│ NO CRON             │
│ NO DBUS             │
│ NO SYSTEMD          │
│ NO SSHD             │
│ NO SHELL            │
│ NO INTERACTIVE LOGIN│
│ NO PACKAGE MANAGER  │
│ NO MUTABLE ROOTFS   │
│ ~15 BINARIES TOTAL  │
└─────────────────────┘
```

- Monospace, uppercase, `--text-dim` for "NO", `--text-primary` for the item name
- Each item in its own bordered cell, or a single glass panel with a stacked list
- Optional: strike-through animation on scroll reveal (line draws through each "NO X")

### 5. Architecture Stack

Section label: `01 — ARCHITECTURE`

The 16-layer stack from ARCHITECTURE.md, rendered as thin horizontal bars stacked vertically. Each bar is a glass panel:

```
┌──────────────────────────────────────────────────┐
│ 16  HARBOR          Verified skill registry      │
├──────────────────────────────────────────────────┤
│ 15  EDGE POWER      Offline mode, cpufreq        │
├──────────────────────────────────────────────────┤
│ ...                                              │
├──────────────────────────────────────────────────┤
│ 02  CAGE            Per-agent microVM isolation   │
├──────────────────────────────────────────────────┤
│ 01  KERNEL          Linux 6.18 LTS, KSPP hardened │
└──────────────────────────────────────────────────┘
```

- Layer number in `--text-ghost` (large, decorative)
- Layer name in `--text-primary`, monospace, uppercase
- Description in `--text-dim`, Inter
- Radial gradient glow behind the stack
- On scroll: layers animate in from bottom to top, staggered

### 6. How It Works

Section label: `02 — HOW IT WORKS`

4 numbered steps in sharp-bordered boxes, connected by dotted vertical lines (like screenpipe's format):

```
 01 ─── FLASH
┌──────────────────────────────┐
│ boot the immutable image.    │
│ squashfs rootfs. ~100MB.     │
│ zero configuration.          │
└──────────────────────────────┘
         :
         :
 02 ─── DECLARE
┌──────────────────────────────┐
│ write AGENT.toml.            │
│ declare capabilities.        │
│ nothing implicit.            │
└──────────────────────────────┘
         :
         :
 03 ─── ISOLATE
┌──────────────────────────────┐
│ each agent gets a microVM.   │
│ own kernel. own filesystem.  │
│ hardware-level isolation.    │
└──────────────────────────────┘
         :
         :
 04 ─── ENFORCE
┌──────────────────────────────┐
│ default-deny everything.     │
│ network, filesystem, shell.  │
│ only what's declared runs.   │
└──────────────────────────────┘
```

### 7. Capability Manifest

Section label: `03 — CAPABILITY MANIFEST`

A glass-bordered code block showing AGENT.toml with monochrome syntax:

```toml
[agent]
name = "researcher"
version = "1.2.0"

[capabilities]
network.allow = ["api.openai.com", "api.exa.ai"]
filesystem.read = ["/data/research"]
filesystem.write = ["/data/research/output"]
shell = false
max_memory_mb = 512

[tools]
read_files = { risk = "low" }
delete_files = { risk = "critical" }    # triggers Gate
```

- JetBrains Mono, `font-size: 13-14px`
- Syntax colors: white for values, `--text-secondary` for keys, `--text-dim` for comments
- No colored syntax — monochrome only
- Optional: faint annotations on the right margin pointing to key lines ("← default deny", "← human approval required")

### 8. Target Hardware

Section label: `04 — TARGET HARDWARE`

4 cells in a horizontal grid (stacks on mobile):

```
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ RASPBERRY    │ │ JETSON       │ │ x86_64 VPS   │ │ OLD LAPTOP   │
│ PI 5         │ │ AGX THOR     │ │              │ │              │
│              │ │              │ │              │ │              │
│ 8GB + AI HAT │ │ Blackwell GPU│ │ $6-20/mo     │ │ USB bootable │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
```

- Sharp borders, glass background
- Device name in `--text-primary`, monospace, uppercase
- Spec note in `--text-dim`

### 9. Waitlist CTA

Centered glass panel, large:

```
┌─────────────────────────────────────────────────┐
│                                                 │
│    nothing unnecessary. everything enforced.     │
│                                                 │
│    [your@email.com          ] [JOIN WAITLIST]    │
│                                                 │
│              join 0 others waiting               │
│                                                 │
└─────────────────────────────────────────────────┘
```

- Headline: JetBrains Mono, `--text-primary`
- Input + button: same style as hero
- Counter: `--text-dim`, honest count, updates if backend is wired
- Radial gradient glow behind the panel

### 10. Footer

```
─────────────────────────────────────────────────────────────────
MIT / Apache-2.0                          nullbox is open source.
```

- Thin top border
- License badge left, statement right
- `--text-dim`, small monospace
- GitHub link

---

## Responsive Behavior

- **Desktop (>1024px):** Full layout as described. Three.js hero at full resolution.
- **Tablet (768-1024px):** 2-column grids collapse to 1 where needed. Hero still has 3D canvas.
- **Mobile (<768px):** Single column. Three.js canvas at reduced resolution (or static ASCII fallback for performance). Waitlist input stacks vertically.

---

## Project Structure

```
nullbox-site/
  astro.config.mjs
  package.json
  tailwind.config.mjs
  src/
    layouts/
      Layout.astro          # Base layout, fonts, meta
    pages/
      index.astro           # Landing page
    components/
      Nav.astro
      Hero.astro
      AsciiScene.ts         # Three.js + AsciiEffect (client-side island)
      Problem.astro
      NotExist.astro
      Architecture.astro
      HowItWorks.astro
      Manifest.astro
      Hardware.astro
      Waitlist.astro
      Footer.astro
    styles/
      global.css            # Tailwind base + custom properties + fonts
  public/
    fonts/                  # JetBrains Mono, Inter (self-hosted)
```

---

## Waitlist Backend

For v1, the simplest option: form submits to **Formspree** (free tier, 50 submissions/month) or **Buttondown** (email list with API). No custom backend needed. The form action URL is the only configuration.

If the user wants a counter, a Cloudflare Worker with KV storage can track submission count and return it on page load.

---

## Performance Budget

- **First paint:** <1s (static HTML, critical CSS inlined)
- **Three.js bundle:** Loaded async, hero shows a static ASCII art fallback until canvas initializes
- **Total page weight:** <500KB excluding fonts
- **Lighthouse target:** 90+ performance on mobile (Three.js may reduce this; static fallback handles it)

---

## Out of Scope

- Blog / docs / changelog
- User accounts or dashboard
- Multi-page site (single page only)
- CMS integration
- Analytics (can be added later with Plausible or similar)
