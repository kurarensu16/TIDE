<div align="center">

<img src="./TIDE.png" alt="TIDE Logo" width="160" />

# TIDE
### Terminal Integrated Development Environment

**A native, keyboard-first terminal workspace for parallel coding-agent orchestration.**

[![Rust](https://img.shields.io/badge/Rust-1.85%2B%20(2024%20Edition)-orange?logo=rust&style=flat-square)](https://www.rust-lang.org)
[![TUI](https://img.shields.io/badge/TUI-Ratatui%200.29-blue?style=flat-square)](https://ratatui.rs)
[![Platform](https://img.shields.io/badge/Platform-Windows%2011%20%7C%20Linux%20%7C%20macOS-informational?style=flat-square)](#platform-support)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-green?style=flat-square)](LICENSE)
[![Website](https://img.shields.io/badge/Docs%20%26%20Landing-Astro%20%2B%20Tailwind-purple?style=flat-square)](./web)

</div>

---

## 🌊 Overview

**TIDE** is a native, terminal-integrated workspace built for the era of AI coding agents. 

Rather than juggling countless detached terminal windows, brittle tmux sessions, or bloated Electron-based IDEs, TIDE allows developers to **launch, isolate, monitor, and review multiple coding-agent sessions in parallel** alongside everyday shell sessions, a direct code editor, and full Git operations — all running inside your terminal with ultra-low latency.

TIDE does not host an AI model; it **orchestrates locally installed coding-agent CLIs** (such as [Antigravity](https://github.com/google-deepmind/antigravity), Claude Code, or custom scripts) within strictly isolated Git worktrees and robust OS-level process lifecycles.

```
┌─ Explorer [F2] ─────────┬─ Editor / Preview [F5] ──────────────────┬─ Agents [F4] ──────────┐
│ 📁 src/                 │ 1  use anyhow::Result;                   │ ● Antigravity [RUN]    │
│   ├── app.rs            │ 2  use ratatui::Terminal;                │   tree: .tide/agy-auth │
│   ├── job_object.rs     │ 3                                        │   task: Implement OAuth│
│   ├── main.rs           │ 4  pub fn start_workspace() -> Result<()>{│ ● Claude Code  [IDLE]   │
│   └── ui.rs             │ 5      // 60 FPS frame-paced render loop │   tree: .tide/claude-2 │
│ 📁 web/                 │ 6  }                                     ├─ Git Status [F3] ──────┤
│                         ├─ Terminal [Ctrl+t new | F7/F8 tab] ──────┤ ⎇ main (3 staged)      │
│                         │ > cargo check                            │ + src/app.rs           │
│                         │    Compiling tide v0.1.0                 │ + src/job_object.rs    │
│                         │    Finished `dev` profile in 0.42s       │ ~ src/ui.rs            │
└─────────────────────────┴──────────────────────────────────────────┴────────────────────────┘
 [F1] Help  [F6] Focus Pane  [Ctrl+P] Palette  [Ctrl+Shift+F] Search  [F12] Themes  [Ctrl+A] Launch Agent
```

---

## ✨ Core Architecture Pillars

### 1. 🛡️ Windows Job Object Process Lifecycle
Coding agents frequently spawn deep child processes (compilers, language servers, test suites, background watchers) that can easily become orphaned when a shell exits, holding file locks on Windows and bricking Git operations.
- TIDE binds each terminal and agent session to an OS-level **Windows Job Object** configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
- When an agent is stopped or closed, the entire process hierarchy is terminated atomically — guaranteeing zero orphaned processes and zero locked files.

### 2. 🌲 Dedicated Git Worktree Isolation
Running parallel coding agents on a single branch causes destructive race conditions, dirty working directories, and accidental file overwrites.
- Every agent session is automatically bootstrapped into its own dedicated Git worktree and branch under `.tide/worktrees/`.
- Inspect per-agent diffs before merging (`d`), merge clean changes (`m`), or safely discard rejected sessions (`x`) with automated worktree cleanup.

### 3. ⚡ 60 FPS Frame-Paced PTY Rendering
High-throughput command outputs (e.g., verbose compiler logs, test floods) can easily overwhelm TUI event loops, causing severe interface freezes.
- Embedded terminals run on background threads using `portable-pty` (ConPTY on Windows) and `vt100` terminal emulation.
- Reader threads update screen grids in-place and coalesce dirty flags, while the main UI thread renders at a steady, frame-paced **~60 FPS (16ms)** cadence. Keypresses stay responsive regardless of child output volume.

### 4. 🧭 Honest Agent Telemetry
Detecting autonomous agent states via fragile screen scraping or regex guesswork is unreliable.
- TIDE reports agent state alongside its confidence level:
  - **Confirmed**: Reported via structured CLI exit codes, active PTY process status, or adapter signals.
  - **Inferred**: Derived from idle output heuristics and recognized prompt patterns (surfaced as non-blocking hints).
  - **Unknown**: Displayed transparently when signals are indeterminate.

---

## 🚀 Quick Start

### Prerequisites
- **Rust**: Stable toolchain (1.85+ recommended, 2024 Edition)
- **Git**: Installed and available in your `PATH`
- **OS**: Windows 11 (fully native ConPTY support), Linux, or macOS

### Installation & Build

```bash
# Clone the repository
git clone https://github.com/kurarensu16/TIDE.git
cd TIDE

# Build release binary
cargo build --release

# Run TIDE in your current workspace
./target/release/tide.exe

# Or open a specific project directory
./target/release/tide.exe "C:\path\to\project"
```

---

## ⌨️ Keyboard Shortcuts & Navigation

TIDE is designed from the ground up for frictionless, keyboard-first navigation. All universal function keys are conflict-free and bypass child PTY shells.

### Universal Navigation & Panes
| Key | Action |
| --- | --- |
| `F1` | Toggle Shortcuts & Help Overlay |
| `F2` | Toggle File Explorer panel |
| `F3` | Toggle Git Status & Staging panel |
| `F4` | Toggle Coding Agents panel |
| `F5` | Toggle File Editor / Preview panel |
| `F6` / `Tab` | Cycle active pane focus |
| `F11` | Maximize / Restore current active pane |
| `F12` | Open Theme Selection Modal |
| `Shift + F12` | Quickly cycle to next color theme |
| `Ctrl + P` | Open Command Palette (fuzzy actions) |
| `Ctrl + Shift + F` | Workspace-wide text search |

### Embedded Terminal (`ActivePane::Terminal`)
| Key | Action |
| --- | --- |
| `Ctrl + B` | **Escape chord**: Return focus to TIDE workspace navigation |
| `Ctrl + T` | New Terminal tab (PowerShell, Cmd, Git Bash, or Custom Shell) |
| `Ctrl + W` | Close active terminal tab |
| `F7` / `F8` | Switch to previous / next terminal tab |
| `Ctrl + Tab` / `Ctrl + Shift + Tab` | Next / previous terminal tab |
| `Alt + 1` .. `Alt + 9` | Jump directly to terminal tab `N` |
| `Shift + PageUp` / `Shift + PageDown` | Scroll terminal buffer history up / down |
| `Shift + End` | Jump back to bottom of live terminal output |
| `Ctrl + =` / `Ctrl + -` / `Ctrl + 0` | Zoom in / Zoom out / Reset terminal scaling |

### File Explorer (`ActivePane::Explorer`)
| Key | Action |
| --- | --- |
| `j` / `Down` & `k` / `Up` | Navigate file tree items |
| `Space` / `Enter` | Toggle folder expand/collapse or open file in preview |
| `h` / `Left` & `l` / `Right` | Collapse / Expand directory |
| `a` or `n` | Create new file or directory |
| `r` or `F2` | Rename selected item |
| `d` or `Delete` | Delete selected file or directory (with confirmation) |
| `Ctrl + R` / `R` | Refresh file tree |
| `Ctrl + E` | Open selected file in external editor (e.g., VS Code) |

### Direct Code Editor (`ActivePane::FilePreview`)
| Key | Action |
| --- | --- |
| `Arrow Keys` | Move cursor through document |
| `Type characters` | Direct inline editing |
| `Ctrl + S` | Save file changes to disk |
| `Ctrl + Z` / `Ctrl + Y` | Undo / Redo edits |
| `Ctrl + F` | Search within open file |
| `Ctrl + Tab` / `Ctrl + Shift + Tab` | Next / previous open editor tab |
| `Ctrl + W` | Close current editor tab |
| `Ctrl + O` | Open file in system default editor |

### Git Operations (`ActivePane::Git`)
| Key | Action |
| --- | --- |
| `j` / `Down` & `k` / `Up` | Navigate changed files |
| `Space` / `s` / `u` | Stage / Unstage selected file |
| `a` / `S` | Stage all changes (`git add -A`) |
| `U` | Unstage all changes |
| `c` | Open Commit Dialog |
| `p` / `P` | Push (`git push`) / Pull (`git pull`) |
| `d` / `Enter` | View interactive colored diff |
| `b` | Open Branch Manager (switch, create, delete) |
| `z` | Open Git Stash Manager (apply, pop, create, drop, view diff) |
| `r` | Refresh repository status |

### Parallel Agent Orchestration (`ActivePane::Agents`)
| Key | Action |
| --- | --- |
| `Ctrl + A` or `a` | Open **Launch Agent Modal** |
| `j` / `Down` & `k` / `Up` | Select agent session |
| `Enter` | Jump directly into agent's interactive PTY terminal |
| `d` | Inspect agent's isolated worktree diff |
| `m` | Merge agent worktree into current branch |
| `s` | Stop active agent session (kills full process tree via Job Object) |
| `x` | Discard agent session and clean up worktree/branch |
| `Space` | Toggle waiting-for-input acknowledge hint |

### Card & Panel Resizing
| Key | Action |
| --- | --- |
| `Alt + [` / `Alt + ]` | Decrease / increase Left Explorer column width |
| `Alt + {` / `Alt + }` | Decrease / increase Right Panel (Git / Agents) width |
| `Alt + -` / `Alt + =` | Adjust vertical split between Editor and Terminal |
| *Mouse Drag* | Click and drag any border divider to resize cards interactively |

---

## 🎨 Built-in Color Themes

TIDE features a centralized design system with 10 meticulously crafted color palettes. Switch themes on the fly via `F12` with instant live preview:

1. **TIDE Ocean** *(Default)* — Deep midnight cyan and ocean blues.
2. **Tokyo Night** — Sleek Japanese cyberpunk indigo and neon accents.
3. **Catppuccin Mocha** — Soothing pastel palette with soft contrast.
4. **Nord** — Arctic, north-bluish clean aesthetic.
5. **Gruvbox Dark** — Warm retro groove with earthy contrast.
6. **Cyberpunk** — High-contrast neon yellow and electric pinks.
7. **Dracula** — Classic gothic purple and vibrant highlights.
8. **Rosé Pine** — Minimalist warm pine, rose, and gold tones.
9. **Monokai Pro** — Modernized iconic developer palette.
10. **One Dark Pro** — Iconic balanced dark palette.

---

## 🌐 Documentation & Web Landing

The [`web/`](./web) directory contains the official TIDE product site and interactive technical documentation:

- **Framework**: [Astro](https://astro.build) + [Tailwind CSS](https://tailwindcss.com)
- **Features**:
  - Interactive Ratatui 1:1 terminal simulation
  - Visual Architecture Flow (*The Naked CLI Chaos vs. TIDE Orchestration*)
  - Systems deep dives on Windows Job Objects and worktree lifecycle
  - Live Theme Engine previewer
  - Zero client JavaScript by default with 100/100 Lighthouse performance

To run the documentation site locally:
```bash
cd web
npm install
npm run dev
```

---

## 📂 Project Structure

```
TIDE/
├── build.rs                  # Windows resource compilation (app icon embed)
├── Cargo.toml                # Rust 2024 edition crate configuration
├── tide.ico                  # Embedded Windows application icon
├── TIDE.png                  # High-resolution TIDE branding logo
│
├── src/
│   ├── main.rs               # CLI entrypoint, raw terminal setup & cleanup
│   ├── app.rs                # Core App state, keyboard/mouse event dispatch
│   ├── ui.rs                 # Ratatui layout engine, widget rendering & modals
│   ├── theme.rs              # 10 color themes, palette tokens & theme modal
│   ├── terminal.rs           # PTY session management & vt100 grid integration
│   ├── job_object.rs         # Windows Job Object process tree lifecycle
│   ├── git.rs                # Repository status, staging, diffs, branches & stashes
│   ├── explorer.rs           # File tree traversal, sorting & file operations
│   ├── agents.rs             # Agent session metadata & confidence tracking
│   ├── worktree.rs           # Git worktree creation, isolation & cleanup
│   ├── icons.rs              # File type icons and glyphs
│   └── logo.rs               # ANSI wave logo rasterization
│
└── web/                      # Astro documentation & marketing landing page
    ├── src/
    │   ├── pages/            # Landing page (index.astro) & docs
    │   ├── components/       # Interactive components & architecture diagrams
    │   └── styles/           # Global styles and design tokens
    └── package.json
```

---

## 🔒 Safety & Design Guarantees

- **No Silent Discards**: Unsaved editor changes or uncommitted worktree diffs will never be deleted without an explicit confirmation dialog.
- **No Automatic Merges**: Agent worktrees are never merged into parent branches automatically. Every integration requires a deliberate user review (`m`).
- **Full Process Teardown**: Closing an agent or terminal guarantees that background child processes, test runners, or watchers are cleanly terminated via Windows Job Objects.
- **Transparent Permissions**: Agents run with the user's standard permissions in designated worktrees. TIDE does not run hidden background services or claim artificial sandbox security.

---

## 📄 License

TIDE is open-source software licensed under either the [MIT License](LICENSE) or [Apache License (Version 2.0)](LICENSE) at your option.
