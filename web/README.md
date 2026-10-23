# TIDE Website (`web/`)

The official marketing and technical architecture website for **TIDE** (Terminal Integrated Development Environment).

Built with **Astro** and **Tailwind CSS** using a zero-bloat, engineering-first philosophy (zero client JavaScript by default, pixel-accurate Ratatui emulation, 100/100 Lighthouse score).

---

## Getting Started

### Prerequisites
- Node.js `22.x` or later
- npm `10.x` or later

### Local Development
```bash
cd web
npm run dev
```
Open [http://localhost:4321](http://localhost:4321) in your browser.

### Production Build
```bash
cd web
npm run build
```
Generates a static HTML/CSS distribution in `dist/` ready for Cloudflare Pages, GitHub Pages, or Vercel.

### Preview Build Locally
```bash
cd web
npm run preview
```

---

## Component Architecture

- [`src/pages/index.astro`](src/pages/index.astro) — Main landing page.
- [`src/components/TerminalMockup.astro`](src/components/TerminalMockup.astro) — 1:1 pixel-accurate Ratatui interface showing parallel agent sessions, ConPTY output, and diff review.
- [`src/components/ArchitectureFlow.astro`](src/components/ArchitectureFlow.astro) — Visual comparison: The Naked CLI Chaos vs. TIDE Orchestration Flow.
- [`src/components/EngineeringPillars.astro`](src/components/EngineeringPillars.astro) — Systems deep dives into Windows Job Objects (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`), Worktree bootstrapping, 60 FPS PTY frame pacing, and honest telemetry.
- [`src/components/ThemePreviewer.astro`](src/components/ThemePreviewer.astro) — Live browser theme engine previewing all 6 palettes from `src/theme.rs`.
- [`src/components/AgentAdapters.astro`](src/components/AgentAdapters.astro) — Declarative `tide.toml` configuration spec.
- [`src/components/BenchmarkSpec.astro`](src/components/BenchmarkSpec.astro) — Real benchmark comparisons against Electron-based IDEs.
- [`src/components/InstallSection.astro`](src/components/InstallSection.astro) — Installation channels (`cargo`, git source, standalone binaries).
- [`src/styles/global.css`](src/styles/global.css) — Custom theme tokens and CSS variables.
