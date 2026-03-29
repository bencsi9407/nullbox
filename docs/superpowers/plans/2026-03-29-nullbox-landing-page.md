# NullBox Landing Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a waitlist landing page for NullBox — dark, monochrome, sharp-cornered, with a Three.js ASCII 3D hero and glassmorphism panels.

**Architecture:** Astro static site with vanilla JS. Three.js AsciiEffect renders the hero 3D scene as a client-side island. All styling via Tailwind CSS v4 + CSS custom properties. No JS frameworks. Form posts to Formspree for waitlist capture.

**Tech Stack:** Astro 6.x, Three.js 0.183.x, Tailwind CSS 4.x, JetBrains Mono + Inter fonts

---

## File Structure

```
nullbox-site/
  package.json
  astro.config.mjs
  tsconfig.json
  src/
    styles/
      global.css              # CSS custom properties, Tailwind imports, font-face, base resets
    layouts/
      Layout.astro            # HTML shell, meta tags, font loading, global.css import
    components/
      Nav.astro               # Fixed nav: NULLBOX left, GITHUB right, glass-on-scroll
      Hero.astro              # Full-viewport hero with Three.js canvas + overlay content
      AsciiScene.ts           # Three.js scene: rotating ASCII cube, fracture animation
      Problem.astro           # Three problem statements, staggered fade-in
      NotExist.astro          # "What doesn't exist" grid
      Architecture.astro      # 16-layer stack with glass bars
      HowItWorks.astro        # 4 numbered steps with dotted connectors
      Manifest.astro          # AGENT.toml code block with annotations
      Hardware.astro          # 4-cell hardware grid
      Waitlist.astro          # Email capture CTA with glass panel
      Footer.astro            # License + GitHub link
      ScrollReveal.astro      # Reusable IntersectionObserver wrapper
    pages/
      index.astro             # Composes all sections
  public/
    fonts/
      JetBrainsMono-Regular.woff2
      JetBrainsMono-Bold.woff2
      Inter-Regular.woff2
      Inter-Medium.woff2
```

---

### Task 1: Scaffold Astro Project

**Files:**
- Create: `nullbox-site/package.json`
- Create: `nullbox-site/astro.config.mjs`
- Create: `nullbox-site/tsconfig.json`

- [ ] **Step 1: Create project directory and initialize**

```bash
cd /home/rsx/Desktop/projx
mkdir nullbox-site && cd nullbox-site
npm init -y
```

- [ ] **Step 2: Install dependencies**

```bash
npm install astro@latest three@latest
npm install -D @tailwindcss/vite@latest tailwindcss@latest
```

- [ ] **Step 3: Create astro.config.mjs**

```js
// astro.config.mjs
import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  vite: {
    plugins: [tailwindcss()],
  },
});
```

- [ ] **Step 4: Create tsconfig.json**

```json
{
  "extends": "astro/tsconfigs/strict"
}
```

- [ ] **Step 5: Add scripts to package.json**

Replace the `scripts` section in `package.json`:

```json
{
  "scripts": {
    "dev": "astro dev",
    "build": "astro build",
    "preview": "astro preview"
  }
}
```

- [ ] **Step 6: Download fonts to public/fonts/**

```bash
mkdir -p public/fonts
# JetBrains Mono
curl -L -o public/fonts/JetBrainsMono-Regular.woff2 "https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/webfonts/JetBrainsMono-Regular.woff2"
curl -L -o public/fonts/JetBrainsMono-Bold.woff2 "https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/webfonts/JetBrainsMono-Bold.woff2"
# Inter
curl -L -o public/fonts/Inter-Regular.woff2 "https://github.com/rsms/inter/raw/master/docs/font-files/Inter-Regular.woff2"
curl -L -o public/fonts/Inter-Medium.woff2 "https://github.com/rsms/inter/raw/master/docs/font-files/Inter-Medium.woff2"
```

- [ ] **Step 7: Verify project runs**

```bash
npx astro dev
```

Expected: Astro dev server starts on localhost:4321. Stop it with Ctrl+C.

- [ ] **Step 8: Initialize git and commit**

```bash
git init
echo "node_modules\ndist\n.astro" > .gitignore
git add -A
git commit -m "chore: scaffold astro project with tailwind and three.js"
```

---

### Task 2: Global Styles and Layout

**Files:**
- Create: `nullbox-site/src/styles/global.css`
- Create: `nullbox-site/src/layouts/Layout.astro`

- [ ] **Step 1: Create global.css with design tokens and base styles**

```css
/* src/styles/global.css */
@import "tailwindcss";

@font-face {
  font-family: 'JetBrains Mono';
  src: url('/fonts/JetBrainsMono-Regular.woff2') format('woff2');
  font-weight: 400;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'JetBrains Mono';
  src: url('/fonts/JetBrainsMono-Bold.woff2') format('woff2');
  font-weight: 700;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'Inter';
  src: url('/fonts/Inter-Regular.woff2') format('woff2');
  font-weight: 400;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'Inter';
  src: url('/fonts/Inter-Medium.woff2') format('woff2');
  font-weight: 500;
  font-style: normal;
  font-display: swap;
}

:root {
  --bg-base: #09090b;
  --bg-elevated: #141414;
  --bg-glass: rgba(255, 255, 255, 0.03);
  --border-subtle: rgba(255, 255, 255, 0.06);
  --border-glass: rgba(255, 255, 255, 0.10);
  --text-primary: #e4e4e7;
  --text-secondary: #a1a1aa;
  --text-dim: #71717a;
  --text-ghost: #3f3f46;
  --font-mono: 'JetBrains Mono', monospace;
  --font-sans: 'Inter', system-ui, sans-serif;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
  border-radius: 0;
}

html {
  scroll-behavior: smooth;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  background: var(--bg-base);
  color: var(--text-secondary);
  font-family: var(--font-sans);
  line-height: 1.6;
  overflow-x: hidden;
}

::selection {
  background: rgba(255, 255, 255, 0.15);
  color: var(--text-primary);
}

.glass {
  background: var(--bg-glass);
  border: 1px solid var(--border-glass);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

@supports not (backdrop-filter: blur(12px)) {
  .glass {
    background: var(--bg-elevated);
  }
}

.section-label {
  font-family: var(--font-mono);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.15em;
  color: var(--text-dim);
  margin-bottom: 3rem;
}

.reveal {
  opacity: 0;
  transform: translateY(12px);
  transition: opacity 0.4s cubic-bezier(0.25, 0.1, 0.25, 1),
              transform 0.4s cubic-bezier(0.25, 0.1, 0.25, 1);
}

.reveal.visible {
  opacity: 1;
  transform: translateY(0);
}
```

- [ ] **Step 2: Create Layout.astro**

```astro
---
// src/layouts/Layout.astro
interface Props {
  title: string;
  description: string;
}

const { title, description } = Astro.props;
---

<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="description" content={description} />
    <title>{title}</title>
    <link rel="preload" href="/fonts/JetBrainsMono-Regular.woff2" as="font" type="font/woff2" crossorigin />
    <link rel="preload" href="/fonts/Inter-Regular.woff2" as="font" type="font/woff2" crossorigin />
    <style>
      @import '../styles/global.css';
    </style>
  </head>
  <body>
    <slot />
  </body>
</html>
```

- [ ] **Step 3: Create a minimal index.astro to verify**

```astro
---
// src/pages/index.astro
import Layout from '../layouts/Layout.astro';
---

<Layout title="NullBox" description="The operating system for AI agents.">
  <main>
    <div style="height: 100vh; display: flex; align-items: center; justify-content: center;">
      <h1 style="font-family: var(--font-mono); color: var(--text-primary); letter-spacing: 0.2em;">NULLBOX</h1>
    </div>
  </main>
</Layout>
```

- [ ] **Step 4: Verify dev server shows styled page**

```bash
cd /home/rsx/Desktop/projx/nullbox-site && npx astro dev
```

Expected: Dark background, monospace "NULLBOX" centered, Inter/JetBrains Mono fonts loaded.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add global styles, design tokens, and base layout"
```

---

### Task 3: Nav Component

**Files:**
- Create: `nullbox-site/src/components/Nav.astro`

- [ ] **Step 1: Create Nav.astro**

```astro
---
// src/components/Nav.astro
---

<nav id="nav" class="fixed top-0 left-0 right-0 z-50 border-b transition-all duration-300" style="border-color: var(--border-subtle); background: transparent;">
  <div class="max-w-6xl mx-auto px-6 h-14 flex items-center justify-between">
    <a href="/" class="font-bold tracking-[0.2em] text-sm" style="font-family: var(--font-mono); color: var(--text-primary);">
      NULLBOX
    </a>
    <a
      href="https://github.com/nullbox-os/nullbox"
      target="_blank"
      rel="noopener noreferrer"
      class="text-xs tracking-[0.15em] uppercase transition-colors duration-200"
      style="font-family: var(--font-mono); color: var(--text-dim);"
      onmouseover="this.style.color='var(--text-primary)'"
      onmouseout="this.style.color='var(--text-dim)'"
    >
      GITHUB ↗
    </a>
  </div>
</nav>

<script>
  const nav = document.getElementById('nav');
  if (nav) {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          nav.style.background = 'transparent';
          nav.style.backdropFilter = 'none';
          nav.style.webkitBackdropFilter = 'none';
        } else {
          nav.style.background = 'rgba(9, 9, 11, 0.8)';
          nav.style.backdropFilter = 'blur(12px)';
          nav.style.webkitBackdropFilter = 'blur(12px)';
        }
      },
      { threshold: 0.1 }
    );

    const hero = document.getElementById('hero');
    if (hero) observer.observe(hero);
  }
</script>
```

- [ ] **Step 2: Add Nav to index.astro**

Replace `index.astro` content:

```astro
---
import Layout from '../layouts/Layout.astro';
import Nav from '../components/Nav.astro';
---

<Layout title="NullBox" description="The operating system for AI agents.">
  <Nav />
  <main>
    <section id="hero" style="height: 100vh; display: flex; align-items: center; justify-content: center;">
      <h1 style="font-family: var(--font-mono); color: var(--text-primary); letter-spacing: 0.2em;">NULLBOX</h1>
    </section>
    <section style="height: 100vh;"></section>
  </main>
</Layout>
```

- [ ] **Step 3: Verify nav glass effect on scroll**

```bash
cd /home/rsx/Desktop/projx/nullbox-site && npx astro dev
```

Expected: Nav transparent at top, gains glass backdrop on scroll past hero.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add fixed nav with glass-on-scroll effect"
```

---

### Task 4: ScrollReveal Component

**Files:**
- Create: `nullbox-site/src/components/ScrollReveal.astro`

- [ ] **Step 1: Create ScrollReveal.astro**

```astro
---
// src/components/ScrollReveal.astro
interface Props {
  delay?: number;
  class?: string;
}

const { delay = 0, class: className = '' } = Astro.props;
---

<div class={`reveal ${className}`} data-reveal-delay={delay}>
  <slot />
</div>

<script>
  const reveals = document.querySelectorAll('.reveal');

  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          const el = entry.target as HTMLElement;
          const delay = parseInt(el.dataset.revealDelay || '0', 10);
          setTimeout(() => {
            el.classList.add('visible');
          }, delay);
          observer.unobserve(el);
        }
      });
    },
    { threshold: 0.1, rootMargin: '0px 0px -40px 0px' }
  );

  reveals.forEach((el) => observer.observe(el));
</script>
```

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "feat: add scroll reveal component with staggered delays"
```

---

### Task 5: Three.js ASCII Hero Scene

**Files:**
- Create: `nullbox-site/src/components/AsciiScene.ts`
- Create: `nullbox-site/src/components/Hero.astro`

- [ ] **Step 1: Create AsciiScene.ts — the Three.js ASCII cube with fracture animation**

```ts
// src/components/AsciiScene.ts
import * as THREE from 'three';
import { AsciiEffect } from 'three/addons/effects/AsciiEffect.js';

export function initAsciiScene(container: HTMLElement): () => void {
  const width = container.clientWidth;
  const height = container.clientHeight;

  // Scene
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x09090b);

  // Camera
  const camera = new THREE.PerspectiveCamera(50, width / height, 0.1, 100);
  camera.position.set(0, 0, 6);

  // Renderer
  const renderer = new THREE.WebGLRenderer();
  renderer.setSize(width, height);

  // ASCII Effect
  const effect = new AsciiEffect(renderer, ' .:-=+*#%@01', { invert: true });
  effect.setSize(width, height);
  effect.domElement.style.color = 'var(--text-primary)';
  effect.domElement.style.backgroundColor = 'var(--bg-base)';
  effect.domElement.style.fontFamily = 'var(--font-mono)';
  effect.domElement.style.fontSize = '6px';
  effect.domElement.style.lineHeight = '6px';
  container.appendChild(effect.domElement);

  // Lighting
  const ambient = new THREE.AmbientLight(0xffffff, 0.4);
  scene.add(ambient);

  const directional = new THREE.DirectionalLight(0xffffff, 0.8);
  directional.position.set(2, 3, 4);
  scene.add(directional);

  // Main cube group
  const mainGroup = new THREE.Group();
  scene.add(mainGroup);

  // Materials
  const mainMat = new THREE.MeshStandardMaterial({
    color: 0xcccccc,
    roughness: 0.6,
    metalness: 0.2,
  });

  const miniMat = new THREE.MeshStandardMaterial({
    color: 0x999999,
    roughness: 0.7,
    metalness: 0.1,
  });

  // Main cube
  const cubeGeo = new THREE.BoxGeometry(1.8, 1.8, 1.8);
  const mainCube = new THREE.Mesh(cubeGeo, mainMat);
  mainGroup.add(mainCube);

  // Wireframe overlay
  const wireGeo = new THREE.BoxGeometry(1.82, 1.82, 1.82);
  const wireMat = new THREE.MeshBasicMaterial({
    color: 0x666666,
    wireframe: true,
  });
  const wireframe = new THREE.Mesh(wireGeo, wireMat);
  mainGroup.add(wireframe);

  // Mini cubes (for fracture state)
  const miniCubes: THREE.Mesh[] = [];
  const miniGeo = new THREE.BoxGeometry(0.35, 0.35, 0.35);
  const gridSize = 3;
  const spacing = 0.55;

  for (let x = 0; x < gridSize; x++) {
    for (let y = 0; y < gridSize; y++) {
      for (let z = 0; z < gridSize; z++) {
        const mini = new THREE.Mesh(miniGeo, miniMat);
        mini.userData.targetPos = new THREE.Vector3(
          (x - 1) * spacing,
          (y - 1) * spacing,
          (z - 1) * spacing
        );
        mini.userData.explodedPos = new THREE.Vector3(
          (x - 1) * spacing * 2.5,
          (y - 1) * spacing * 2.5,
          (z - 1) * spacing * 2.5
        );
        mini.position.copy(mini.userData.targetPos);
        mini.visible = false;
        mainGroup.add(mini);
        miniCubes.push(mini);
      }
    }
  }

  // Animation state
  let phase: 'solid' | 'fracturing' | 'fractured' | 'coalescing' = 'solid';
  let phaseTimer = 0;
  const SOLID_DURATION = 6;
  const FRACTURE_DURATION = 2;
  const FRACTURED_DURATION = 4;
  const COALESCE_DURATION = 2;

  const clock = new THREE.Clock();
  let animId: number;

  function animate() {
    animId = requestAnimationFrame(animate);
    const delta = clock.getDelta();
    const elapsed = clock.getElapsedTime();

    // Slow rotation
    mainGroup.rotation.y = elapsed * 0.15;
    mainGroup.rotation.x = Math.sin(elapsed * 0.1) * 0.2;

    phaseTimer += delta;

    switch (phase) {
      case 'solid':
        if (phaseTimer > SOLID_DURATION) {
          phase = 'fracturing';
          phaseTimer = 0;
          mainCube.visible = false;
          wireframe.visible = false;
          miniCubes.forEach((m) => {
            m.visible = true;
            m.position.copy(m.userData.targetPos);
          });
        }
        break;

      case 'fracturing': {
        const t = Math.min(phaseTimer / FRACTURE_DURATION, 1);
        const eased = t * t * (3 - 2 * t); // smoothstep
        miniCubes.forEach((m) => {
          m.position.lerpVectors(m.userData.targetPos, m.userData.explodedPos, eased);
          m.rotation.x = eased * Math.PI * 0.5;
          m.rotation.y = eased * Math.PI * 0.3;
        });
        if (t >= 1) {
          phase = 'fractured';
          phaseTimer = 0;
        }
        break;
      }

      case 'fractured':
        // Gentle floating
        miniCubes.forEach((m, i) => {
          m.position.x = m.userData.explodedPos.x + Math.sin(elapsed * 0.5 + i) * 0.05;
          m.position.y = m.userData.explodedPos.y + Math.cos(elapsed * 0.4 + i * 0.5) * 0.05;
        });
        if (phaseTimer > FRACTURED_DURATION) {
          phase = 'coalescing';
          phaseTimer = 0;
        }
        break;

      case 'coalescing': {
        const t = Math.min(phaseTimer / COALESCE_DURATION, 1);
        const eased = t * t * (3 - 2 * t);
        miniCubes.forEach((m) => {
          m.position.lerpVectors(m.userData.explodedPos, m.userData.targetPos, eased);
          m.rotation.x *= 1 - eased * 0.05;
          m.rotation.y *= 1 - eased * 0.05;
        });
        if (t >= 1) {
          phase = 'solid';
          phaseTimer = 0;
          mainCube.visible = true;
          wireframe.visible = true;
          miniCubes.forEach((m) => (m.visible = false));
        }
        break;
      }
    }

    effect.render(scene, camera);
  }

  animate();

  // Resize handler
  function onResize() {
    const w = container.clientWidth;
    const h = container.clientHeight;
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    renderer.setSize(w, h);
    effect.setSize(w, h);
  }

  window.addEventListener('resize', onResize);

  // Cleanup
  return () => {
    cancelAnimationFrame(animId);
    window.removeEventListener('resize', onResize);
    renderer.dispose();
    container.removeChild(effect.domElement);
  };
}
```

- [ ] **Step 2: Create Hero.astro**

```astro
---
// src/components/Hero.astro
---

<section id="hero" class="relative w-full" style="height: 100vh; min-height: 600px;">
  <!-- Three.js canvas container -->
  <div id="ascii-canvas" class="absolute inset-0 z-0"></div>

  <!-- Overlay gradient to improve text readability -->
  <div class="absolute inset-0 z-10" style="background: radial-gradient(ellipse at 50% 50%, rgba(9,9,11,0.5) 0%, rgba(9,9,11,0.85) 70%);"></div>

  <!-- Content -->
  <div class="relative z-20 flex flex-col items-center justify-center h-full px-6 text-center">
    <h1 class="mb-4" style="font-family: var(--font-mono); color: var(--text-primary); font-size: clamp(2rem, 5vw, 4rem); line-height: 1.15; letter-spacing: 0.02em;">
      the operating system<br />for AI agents
    </h1>
    <p class="mb-10 max-w-lg" style="color: var(--text-secondary); font-size: clamp(0.875rem, 1.5vw, 1.125rem);">
      no ssh. no shell. no attack surface.<br />just agents.
    </p>

    <!-- Waitlist form (hero instance) -->
    <form id="hero-form" class="flex flex-col sm:flex-row gap-3 w-full max-w-md" action="https://formspree.io/f/REPLACE_ME" method="POST">
      <input
        type="email"
        name="email"
        required
        placeholder="you@company.com"
        class="glass flex-1 px-4 py-3 text-sm outline-none transition-colors duration-200"
        style="font-family: var(--font-mono); color: var(--text-primary); border-color: var(--border-glass);"
        onfocus="this.style.borderColor='rgba(255,255,255,0.2)'"
        onblur="this.style.borderColor='var(--border-glass)'"
      />
      <button
        type="submit"
        class="px-6 py-3 text-xs font-bold tracking-[0.15em] uppercase transition-colors duration-200"
        style="font-family: var(--font-mono); color: var(--text-primary); background: var(--bg-elevated); border: 1px solid var(--border-glass);"
        onmouseover="this.style.borderColor='rgba(255,255,255,0.2)'"
        onmouseout="this.style.borderColor='var(--border-glass)'"
      >
        JOIN WAITLIST
      </button>
    </form>
  </div>
</section>

<script>
  import { initAsciiScene } from './AsciiScene';

  const container = document.getElementById('ascii-canvas');
  if (container) {
    // Reduce resolution on mobile
    const isMobile = window.innerWidth < 768;
    if (isMobile) {
      container.style.opacity = '0.5';
    }
    initAsciiScene(container);
  }
</script>
```

- [ ] **Step 3: Update index.astro to use Hero**

```astro
---
import Layout from '../layouts/Layout.astro';
import Nav from '../components/Nav.astro';
import Hero from '../components/Hero.astro';
---

<Layout title="NullBox — The Operating System for AI Agents" description="Immutable, minimal, flashable Linux OS where every layer exists to serve autonomous AI agents.">
  <Nav />
  <main>
    <Hero />
    <section style="height: 100vh;"></section>
  </main>
</Layout>
```

- [ ] **Step 4: Verify hero renders**

```bash
cd /home/rsx/Desktop/projx/nullbox-site && npx astro dev
```

Expected: Full-viewport hero with rotating ASCII cube, headline text, and waitlist form overlay. Cube fractures into grid of smaller cubes every ~6 seconds.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add three.js ASCII hero with rotating cube and fracture animation"
```

---

### Task 6: Problem Section

**Files:**
- Create: `nullbox-site/src/components/Problem.astro`

- [ ] **Step 1: Create Problem.astro**

```astro
---
// src/components/Problem.astro
import ScrollReveal from './ScrollReveal.astro';

const statements = [
  'your agents run on an OS built for humans.',
  'ssh exists. a shell exists. an attack surface exists.',
  'every layer you don\'t need is a layer that can break.',
];
---

<section class="py-32 px-6">
  <div class="max-w-4xl mx-auto flex flex-col" style="gap: 120px;">
    {statements.map((text, i) => (
      <ScrollReveal delay={i * 150}>
        <p
          class="text-center"
          style={`font-family: var(--font-mono); color: var(--text-primary); font-size: clamp(1.2rem, 2.5vw, 2rem); line-height: 1.4;`}
        >
          {text}
        </p>
      </ScrollReveal>
    ))}
  </div>
</section>
```

- [ ] **Step 2: Add to index.astro after Hero**

Add `import Problem from '../components/Problem.astro';` and `<Problem />` after `<Hero />`.

- [ ] **Step 3: Verify staggered fade-in on scroll**

Expected: Three statements appear one by one as you scroll, with 150ms stagger.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add problem section with staggered scroll reveal"
```

---

### Task 7: What Doesn't Exist Section

**Files:**
- Create: `nullbox-site/src/components/NotExist.astro`

- [ ] **Step 1: Create NotExist.astro**

```astro
---
// src/components/NotExist.astro
import ScrollReveal from './ScrollReveal.astro';

const items = [
  'CRON',
  'DBUS',
  'SYSTEMD',
  'SSHD',
  'SHELL',
  'INTERACTIVE LOGIN',
  'PACKAGE MANAGER',
  'MUTABLE ROOTFS',
];
---

<section class="py-32 px-6 relative">
  <!-- Radial glow -->
  <div class="absolute inset-0 pointer-events-none" style="background: radial-gradient(ellipse at 40% 30%, rgba(255,255,255,0.02), transparent 60%);"></div>

  <div class="max-w-4xl mx-auto relative">
    <ScrollReveal>
      <p class="section-label">00 — WHAT DOESN'T EXIST</p>
    </ScrollReveal>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-px" style="background: var(--border-subtle);">
      {items.map((item, i) => (
        <ScrollReveal delay={i * 80}>
          <div
            class="flex items-center gap-3 px-5 py-4 transition-colors duration-200"
            style="background: var(--bg-base);"
            onmouseover="this.style.background='var(--bg-elevated)'"
            onmouseout="this.style.background='var(--bg-base)'"
          >
            <span style="font-family: var(--font-mono); font-size: 12px; color: var(--text-dim); letter-spacing: 0.1em;">NO</span>
            <span style="font-family: var(--font-mono); font-size: 14px; color: var(--text-primary); letter-spacing: 0.05em;">{item}</span>
          </div>
        </ScrollReveal>
      ))}
      <ScrollReveal delay={items.length * 80}>
        <div
          class="flex items-center gap-3 px-5 py-4"
          style="background: var(--bg-base);"
        >
          <span style="font-family: var(--font-mono); font-size: 12px; color: var(--text-dim); letter-spacing: 0.1em;">~15</span>
          <span style="font-family: var(--font-mono); font-size: 14px; color: var(--text-primary); letter-spacing: 0.05em;">BINARIES TOTAL</span>
        </div>
      </ScrollReveal>
    </div>
  </div>
</section>
```

- [ ] **Step 2: Add to index.astro after Problem**

Add `import NotExist from '../components/NotExist.astro';` and `<NotExist />` after `<Problem />`.

- [ ] **Step 3: Verify grid layout and staggered reveal**

Expected: 3-column grid on desktop, 2 on tablet, 1 on mobile. Items reveal with 80ms stagger. Grid lines formed by 1px gap with subtle border color.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add 'what doesn't exist' section with grid layout"
```

---

### Task 8: Architecture Stack Section

**Files:**
- Create: `nullbox-site/src/components/Architecture.astro`

- [ ] **Step 1: Create Architecture.astro**

```astro
---
// src/components/Architecture.astro
import ScrollReveal from './ScrollReveal.astro';

const layers = [
  { num: '16', name: 'HARBOR', desc: 'Verified skill registry' },
  { num: '15', name: 'EDGE POWER', desc: 'Offline mode, cpufreq' },
  { num: '14', name: 'PHOENIX', desc: 'Self-healing, eBPF health' },
  { num: '13', name: 'FORGE', desc: 'Pre-deploy testing, capability proofs' },
  { num: '12', name: 'SWARM', desc: 'Multi-agent orchestration' },
  { num: '11', name: 'GATE', desc: 'Human-in-the-loop approvals' },
  { num: '10', name: 'WATCHER', desc: 'Merkle-chained audit log' },
  { num: '09', name: 'PROVENANCE', desc: 'Ed25519 per agent, TPM-bound' },
  { num: '08', name: 'CTXGRAPH', desc: 'Shared agent memory graph' },
  { num: '07', name: 'CLOAKPIPE', desc: 'Mandatory PII redaction' },
  { num: '06', name: 'SENTINEL', desc: 'LLM firewall + eBPF' },
  { num: '05', name: 'WARDEN', desc: 'Credential vault, dm-crypt' },
  { num: '04', name: 'EGRESS', desc: 'Default-deny network' },
  { num: '03', name: 'ACCELERATOR', desc: 'GPU/NPU/TPU isolation' },
  { num: '02', name: 'CAGE', desc: 'Per-agent microVM isolation' },
  { num: '01', name: 'KERNEL', desc: 'Linux 6.18 LTS, KSPP hardened' },
];
---

<section class="py-32 px-6 relative">
  <div class="absolute inset-0 pointer-events-none" style="background: radial-gradient(ellipse at 60% 50%, rgba(255,255,255,0.03), transparent 50%);"></div>

  <div class="max-w-3xl mx-auto relative">
    <ScrollReveal>
      <p class="section-label">01 — ARCHITECTURE</p>
    </ScrollReveal>

    <div class="flex flex-col" style="gap: 1px; background: var(--border-subtle);">
      {layers.map((layer, i) => (
        <ScrollReveal delay={i * 50}>
          <div
            class="glass flex items-center gap-4 px-5 py-3 transition-colors duration-200 group"
            style="background: var(--bg-glass);"
            onmouseover="this.style.borderColor='rgba(255,255,255,0.15)'"
            onmouseout="this.style.borderColor='var(--border-glass)'"
          >
            <span
              class="w-8 text-right shrink-0"
              style="font-family: var(--font-mono); font-size: 20px; color: var(--text-ghost);"
            >
              {layer.num}
            </span>
            <span
              class="w-32 shrink-0"
              style="font-family: var(--font-mono); font-size: 13px; color: var(--text-primary); letter-spacing: 0.08em;"
            >
              {layer.name}
            </span>
            <span style="font-size: 13px; color: var(--text-dim);">
              {layer.desc}
            </span>
          </div>
        </ScrollReveal>
      ))}
    </div>
  </div>
</section>
```

- [ ] **Step 2: Add to index.astro after NotExist**

Add `import Architecture from '../components/Architecture.astro';` and `<Architecture />` after `<NotExist />`.

- [ ] **Step 3: Verify stack renders with staggered bottom-to-top reveal**

Expected: 16 stacked glass bars, large ghost numbers on left, layer name and description. Reveals from top with 50ms stagger per layer.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add architecture stack section with 16 layers"
```

---

### Task 9: How It Works Section

**Files:**
- Create: `nullbox-site/src/components/HowItWorks.astro`

- [ ] **Step 1: Create HowItWorks.astro**

```astro
---
// src/components/HowItWorks.astro
import ScrollReveal from './ScrollReveal.astro';

const steps = [
  {
    num: '01',
    title: 'FLASH',
    lines: ['boot the immutable image.', 'squashfs rootfs. ~100MB.', 'zero configuration.'],
  },
  {
    num: '02',
    title: 'DECLARE',
    lines: ['write AGENT.toml.', 'declare capabilities.', 'nothing implicit.'],
  },
  {
    num: '03',
    title: 'ISOLATE',
    lines: ['each agent gets a microVM.', 'own kernel. own filesystem.', 'hardware-level isolation.'],
  },
  {
    num: '04',
    title: 'ENFORCE',
    lines: ['default-deny everything.', 'network, filesystem, shell.', 'only what\'s declared runs.'],
  },
];
---

<section class="py-32 px-6">
  <div class="max-w-2xl mx-auto">
    <ScrollReveal>
      <p class="section-label">02 — HOW IT WORKS</p>
    </ScrollReveal>

    <div class="flex flex-col items-center">
      {steps.map((step, i) => (
        <>
          <ScrollReveal delay={i * 150} class="w-full">
            <div class="flex items-start gap-4 mb-2">
              <span style="font-family: var(--font-mono); font-size: 12px; color: var(--text-dim); letter-spacing: 0.1em; padding-top: 2px;">
                {step.num}
              </span>
              <div class="w-8 border-t mt-3" style="border-color: var(--border-subtle);"></div>
              <span style="font-family: var(--font-mono); font-size: 14px; color: var(--text-primary); letter-spacing: 0.1em;">
                {step.title}
              </span>
            </div>
            <div
              class="glass w-full px-6 py-5"
              style="margin-left: 0;"
            >
              {step.lines.map((line) => (
                <p style="font-family: var(--font-mono); font-size: 13px; color: var(--text-secondary); line-height: 1.8;">
                  {line}
                </p>
              ))}
            </div>
          </ScrollReveal>

          {i < steps.length - 1 && (
            <div class="flex flex-col items-center py-4 gap-1.5">
              <div class="w-px h-2" style="background: var(--text-ghost);"></div>
              <div class="w-px h-2" style="background: var(--text-ghost);"></div>
              <div class="w-px h-2" style="background: var(--text-ghost);"></div>
            </div>
          )}
        </>
      ))}
    </div>
  </div>
</section>
```

- [ ] **Step 2: Add to index.astro after Architecture**

Add `import HowItWorks from '../components/HowItWorks.astro';` and `<HowItWorks />` after `<Architecture />`.

- [ ] **Step 3: Verify 4-step layout with dotted connectors**

Expected: 4 glass-bordered boxes with numbered labels, connected by dotted vertical lines.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add how-it-works section with 4 steps and connectors"
```

---

### Task 10: Capability Manifest Section

**Files:**
- Create: `nullbox-site/src/components/Manifest.astro`

- [ ] **Step 1: Create Manifest.astro**

```astro
---
// src/components/Manifest.astro
import ScrollReveal from './ScrollReveal.astro';

// Monochrome syntax: keys dim, values primary, comments ghost
const lines = [
  { type: 'header', text: '[agent]' },
  { type: 'kv', key: 'name', value: '"researcher"' },
  { type: 'kv', key: 'version', value: '"1.2.0"' },
  { type: 'blank' },
  { type: 'header', text: '[capabilities]' },
  { type: 'kv', key: 'network.allow', value: '["api.openai.com", "api.exa.ai"]', annotation: 'default deny' },
  { type: 'kv', key: 'filesystem.read', value: '["/data/research"]' },
  { type: 'kv', key: 'filesystem.write', value: '["/data/research/output"]' },
  { type: 'kv', key: 'shell', value: 'false' },
  { type: 'kv', key: 'max_memory_mb', value: '512' },
  { type: 'blank' },
  { type: 'header', text: '[tools]' },
  { type: 'kv', key: 'read_files', value: '{ risk = "low" }' },
  { type: 'kv', key: 'delete_files', value: '{ risk = "critical" }', annotation: 'human approval required' },
];
---

<section class="py-32 px-6 relative">
  <div class="absolute inset-0 pointer-events-none" style="background: radial-gradient(ellipse at 30% 60%, rgba(255,255,255,0.02), transparent 50%);"></div>

  <div class="max-w-3xl mx-auto relative">
    <ScrollReveal>
      <p class="section-label">03 — CAPABILITY MANIFEST</p>
    </ScrollReveal>

    <ScrollReveal delay={100}>
      <div class="glass p-6 sm:p-8 overflow-x-auto">
        <pre style="font-family: var(--font-mono); font-size: 13px; line-height: 1.8;">
{lines.map((line) => {
  if (line.type === 'blank') return '\n';
  if (line.type === 'header') {
    return <span><span style="color: var(--text-primary);">{line.text}</span>{'\n'}</span>;
  }
  if (line.type === 'kv') {
    const annotation = (line as any).annotation;
    return (
      <span>
        <span style="color: var(--text-dim);">{line.key}</span>
        <span style="color: var(--text-ghost);"> = </span>
        <span style="color: var(--text-primary);">{line.value}</span>
        {annotation && (
          <span style="color: var(--text-ghost);">{'    '}# {annotation}</span>
        )}
        {'\n'}
      </span>
    );
  }
  return null;
})}
        </pre>
      </div>
    </ScrollReveal>
  </div>
</section>
```

- [ ] **Step 2: Add to index.astro after HowItWorks**

Add `import Manifest from '../components/Manifest.astro';` and `<Manifest />` after `<HowItWorks />`.

- [ ] **Step 3: Verify code block renders with monochrome syntax**

Expected: Glass-bordered code block with AGENT.toml. Keys in dim, values in primary, comments in ghost. Annotations on the right for key lines.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add capability manifest section with monochrome toml"
```

---

### Task 11: Hardware Section

**Files:**
- Create: `nullbox-site/src/components/Hardware.astro`

- [ ] **Step 1: Create Hardware.astro**

```astro
---
// src/components/Hardware.astro
import ScrollReveal from './ScrollReveal.astro';

const targets = [
  { name: 'RASPBERRY PI 5', spec: '8GB + AI HAT' },
  { name: 'JETSON AGX THOR', spec: 'Blackwell GPU' },
  { name: 'x86_64 VPS', spec: '$6-20/mo' },
  { name: 'OLD LAPTOP', spec: 'USB bootable' },
];
---

<section class="py-32 px-6">
  <div class="max-w-4xl mx-auto">
    <ScrollReveal>
      <p class="section-label">04 — TARGET HARDWARE</p>
    </ScrollReveal>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-px" style="background: var(--border-subtle);">
      {targets.map((target, i) => (
        <ScrollReveal delay={i * 100}>
          <div
            class="glass px-5 py-6 flex flex-col gap-3 transition-colors duration-200"
            onmouseover="this.style.borderColor='rgba(255,255,255,0.15)'"
            onmouseout="this.style.borderColor='var(--border-glass)'"
          >
            <span style="font-family: var(--font-mono); font-size: 14px; color: var(--text-primary); letter-spacing: 0.05em;">
              {target.name}
            </span>
            <span style="font-family: var(--font-mono); font-size: 12px; color: var(--text-dim);">
              {target.spec}
            </span>
          </div>
        </ScrollReveal>
      ))}
    </div>
  </div>
</section>
```

- [ ] **Step 2: Add to index.astro after Manifest**

Add `import Hardware from '../components/Hardware.astro';` and `<Hardware />` after `<Manifest />`.

- [ ] **Step 3: Verify 4-column grid**

Expected: 4 glass cells side by side on desktop, stacked on mobile. Sharp borders, monospace text.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add target hardware section with 4-cell grid"
```

---

### Task 12: Waitlist CTA Section

**Files:**
- Create: `nullbox-site/src/components/Waitlist.astro`

- [ ] **Step 1: Create Waitlist.astro**

```astro
---
// src/components/Waitlist.astro
import ScrollReveal from './ScrollReveal.astro';
---

<section class="py-32 px-6 relative">
  <div class="absolute inset-0 pointer-events-none" style="background: radial-gradient(ellipse at 50% 50%, rgba(255,255,255,0.03), transparent 50%);"></div>

  <div class="max-w-xl mx-auto relative">
    <ScrollReveal>
      <div class="glass px-8 py-12 sm:px-12 sm:py-16 text-center">
        <h2
          class="mb-8"
          style="font-family: var(--font-mono); font-size: clamp(1.1rem, 2vw, 1.5rem); color: var(--text-primary); letter-spacing: 0.02em; line-height: 1.4;"
        >
          nothing unnecessary.<br />everything enforced.
        </h2>

        <form
          id="waitlist-form"
          class="flex flex-col sm:flex-row gap-3 mb-6"
          action="https://formspree.io/f/REPLACE_ME"
          method="POST"
        >
          <input
            type="email"
            name="email"
            required
            placeholder="you@company.com"
            class="glass flex-1 px-4 py-3 text-sm outline-none transition-colors duration-200"
            style="font-family: var(--font-mono); color: var(--text-primary);"
            onfocus="this.style.borderColor='rgba(255,255,255,0.2)'"
            onblur="this.style.borderColor='var(--border-glass)'"
          />
          <button
            type="submit"
            class="px-6 py-3 text-xs font-bold tracking-[0.15em] uppercase transition-colors duration-200"
            style="font-family: var(--font-mono); color: var(--text-primary); background: var(--bg-elevated); border: 1px solid var(--border-glass);"
            onmouseover="this.style.borderColor='rgba(255,255,255,0.2)'"
            onmouseout="this.style.borderColor='var(--border-glass)'"
          >
            JOIN WAITLIST
          </button>
        </form>

        <p style="font-family: var(--font-mono); font-size: 12px; color: var(--text-ghost);">
          open source — MIT / Apache-2.0
        </p>
      </div>
    </ScrollReveal>
  </div>
</section>
```

- [ ] **Step 2: Add to index.astro after Hardware**

Add `import Waitlist from '../components/Waitlist.astro';` and `<Waitlist />` after `<Hardware />`.

- [ ] **Step 3: Verify CTA renders with glass panel and form**

Expected: Centered glass panel with headline, email input, submit button. All sharp corners, monochrome.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add waitlist CTA section with email capture form"
```

---

### Task 13: Footer

**Files:**
- Create: `nullbox-site/src/components/Footer.astro`

- [ ] **Step 1: Create Footer.astro**

```astro
---
// src/components/Footer.astro
---

<footer class="px-6 py-8 border-t" style="border-color: var(--border-subtle);">
  <div class="max-w-6xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
    <span style="font-family: var(--font-mono); font-size: 11px; color: var(--text-ghost); letter-spacing: 0.1em;">
      MIT / APACHE-2.0
    </span>
    <div class="flex items-center gap-6">
      <a
        href="https://github.com/nullbox-os/nullbox"
        target="_blank"
        rel="noopener noreferrer"
        class="transition-colors duration-200"
        style="font-family: var(--font-mono); font-size: 11px; color: var(--text-dim); letter-spacing: 0.1em;"
        onmouseover="this.style.color='var(--text-primary)'"
        onmouseout="this.style.color='var(--text-dim)'"
      >
        GITHUB ↗
      </a>
      <span style="font-family: var(--font-mono); font-size: 11px; color: var(--text-ghost); letter-spacing: 0.1em;">
        NULLBOX IS OPEN SOURCE
      </span>
    </div>
  </div>
</footer>
```

- [ ] **Step 2: Add to index.astro after Waitlist**

Add `import Footer from '../components/Footer.astro';` and `<Footer />` after `</main>` (outside main, inside Layout).

- [ ] **Step 3: Verify footer renders**

Expected: Thin top border, license left, GitHub + statement right. Ghost/dim monospace text.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add footer with license and github link"
```

---

### Task 14: Final Index Assembly and Polish

**Files:**
- Modify: `nullbox-site/src/pages/index.astro`

- [ ] **Step 1: Write the final index.astro with all sections**

```astro
---
// src/pages/index.astro
import Layout from '../layouts/Layout.astro';
import Nav from '../components/Nav.astro';
import Hero from '../components/Hero.astro';
import Problem from '../components/Problem.astro';
import NotExist from '../components/NotExist.astro';
import Architecture from '../components/Architecture.astro';
import HowItWorks from '../components/HowItWorks.astro';
import Manifest from '../components/Manifest.astro';
import Hardware from '../components/Hardware.astro';
import Waitlist from '../components/Waitlist.astro';
import Footer from '../components/Footer.astro';
---

<Layout
  title="NullBox — The Operating System for AI Agents"
  description="Immutable, minimal, flashable Linux OS where every layer exists to serve autonomous AI agents — and nothing else exists at all."
>
  <Nav />
  <main>
    <Hero />
    <Problem />
    <NotExist />
    <Architecture />
    <HowItWorks />
    <Manifest />
    <Hardware />
    <Waitlist />
  </main>
  <Footer />
</Layout>
```

- [ ] **Step 2: Run dev server and scroll through entire page**

```bash
cd /home/rsx/Desktop/projx/nullbox-site && npx astro dev
```

Verify all 10 sections render correctly: Nav, Hero (with 3D), Problem, NotExist, Architecture, HowItWorks, Manifest, Hardware, Waitlist, Footer.

- [ ] **Step 3: Test responsive layout**

Resize browser to mobile width (<768px). Verify: grids stack to single column, hero text scales down, forms stack vertically, architecture bars remain readable.

- [ ] **Step 4: Run production build**

```bash
npx astro build
```

Expected: Build succeeds with no errors. Output in `dist/`.

- [ ] **Step 5: Preview production build**

```bash
npx astro preview
```

Expected: Site loads from built assets, all sections functional.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: assemble complete landing page with all sections"
```

---

## Summary

| Task | Component | Files |
|------|-----------|-------|
| 1 | Scaffold | package.json, astro.config, tsconfig |
| 2 | Styles + Layout | global.css, Layout.astro |
| 3 | Nav | Nav.astro |
| 4 | ScrollReveal | ScrollReveal.astro |
| 5 | Hero + 3D | AsciiScene.ts, Hero.astro |
| 6 | Problem | Problem.astro |
| 7 | NotExist | NotExist.astro |
| 8 | Architecture | Architecture.astro |
| 9 | HowItWorks | HowItWorks.astro |
| 10 | Manifest | Manifest.astro |
| 11 | Hardware | Hardware.astro |
| 12 | Waitlist | Waitlist.astro |
| 13 | Footer | Footer.astro |
| 14 | Assembly | index.astro, build verification |
