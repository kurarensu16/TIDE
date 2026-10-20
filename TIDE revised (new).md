# TIDE: Terminal Integrated Development Environment (Revised Project Overview)

Revision 3. Reframed around parallel coding-agent management, with hardened Windows process lifecycle, worktree bootstrapping, and rendering backpressure.

## Overview

**TIDE** is a proposed native, keyboard-first workspace that runs inside a terminal. Its purpose is to let a developer run, isolate, monitor, and review multiple coding-agent sessions in parallel, alongside ordinary shell sessions and everyday Git operations, without constantly switching between terminal windows, editors, and Git tools.

TIDE is not an AI model and does not host one. It orchestrates locally installed coding-agent CLIs. The editor and Git panels are supporting features: they exist to make agent work easy to review and integrate, and they should remain useful when no agent is running[cite: 1].

### Product identity

- **Name:** TIDE (Terminal Integrated Development Environment)[cite: 1]
- **Command:** `tide` (provisional; see Naming risk below)[cite: 1]
- **Implementation language:** Rust (stable)[cite: 1]
- **Proposed TUI stack:** Ratatui + Crossterm[cite: 1]
- **Initial platform:** Windows 11[cite: 1]
- **Planned platforms:** Linux and macOS[cite: 1]
- **Interaction model:** Keyboard-first, terminal-native[cite: 1]

### Core value proposition

A coherent workflow for parallel agent work: start an agent in an isolated Git worktree, watch its status, intervene when it needs input, review what it changed, and integrate or discard the result safely[cite: 1]. The value comes from this workflow and from reliable Windows support, not from placing many panels on one screen[cite: 1].

### Naming risk

The name `tide` may collide with existing software, including a Rust web framework crate and a shell-prompt tool that installs a `tide` command[cite: 1]. Before publishing a crate, a package, or a binary name, verify availability on crates.io, common package managers, and PATH conflicts, and choose an alternate binary/crate identifier (e.g., `tide-ws` or `riptide`) if needed[cite: 1].

## Scope

### In scope

1. **Workspace management**[cite: 1]
   - Open a local project or repository[cite: 1].
   - Navigate between resizable panes using keyboard shortcuts[cite: 1].
   - Show project, branch, and active-session status[cite: 1].
   - Support configurable layouts and preferences[cite: 1].
2. **Terminal sessions (embedded terminal)**[cite: 1]
   - Create, rename, switch, and close shell sessions[cite: 1].
   - Host real interactive programs through a PTY and a terminal emulator, rendered into TIDE panes[cite: 1].
   - Handle resizing, input, Unicode width, process exit, and full process-tree termination reliably[cite: 1].
3. **Coding-agent management**[cite: 1]
   - Launch compatible, locally installed agent CLIs from a selected project or worktree[cite: 1].
   - Display session name, status, runtime, and working directory[cite: 1].
   - Switch between, stop, restart, and inspect sessions[cite: 1].
   - Detect each agent's capabilities rather than assuming all CLIs behave alike[cite: 1].
   - Provide a consistent management interface through agent adapters[cite: 1].
4. **Git worktree isolation, bootstrapping, and review**[cite: 1]
   - Create a dedicated worktree and branch per agent session[cite: 1].
   - Support configurable worktree bootstrap hooks (e.g., copying/symlinking `.env` files, sharing build caches like `CARGO_TARGET_DIR`, or running fast dependency links) so isolated worktrees can build and test immediately.
   - Show changed files and diffs per session[cite: 1].
   - Support a review-before-integration flow: inspect, accept, request changes, or discard[cite: 1].
   - Clean up worktrees, branches, and locked child processes explicitly[cite: 1].
5. **Git tools**[cite: 1]
   - Show branch, working-tree status, and changed files[cite: 1].
   - Review diffs; stage and unstage changes; create commits; switch branches[cite: 1].
   - Confirm destructive operations and report errors clearly[cite: 1].
6. **External-editor integration**[cite: 1]
   - Open files in the user's configured editor, including inside a TIDE terminal pane[cite: 1].
   - Detect external file changes and refresh affected views[cite: 1].
7. **Configuration and reliability**[cite: 1]
   - Readable configuration file and discoverable keyboard-shortcut help[cite: 1].
   - Graceful handling of missing executables, process failures, invalid configuration, file changes, and terminal resizing[cite: 1].
   - Automated tests for core state, Git operations, and process management[cite: 1].

### Out of scope for the initial release

- Building or hosting an AI model[cite: 1].
- A built-in source editor (deferred; see Roadmap, Phase 4)[cite: 1].
- Replacing every feature of a graphical IDE[cite: 1].
- Guaranteed compatibility with every coding-agent CLI[cite: 1].
- Cloud synchronization, team collaboration, or remote workspace hosting[cite: 1].
- A universal debugger for all languages[cite: 1].
- A plugin marketplace[cite: 1].
- Silent merging or overwriting of concurrent agent changes[cite: 1].
- Project-management features unrelated to the local development workflow[cite: 1].

## Key Design Decisions

### 1. Embedded terminal: real emulator required

A full-screen interactive agent interface cannot be displayed reliably by copying raw output into a text panel[cite: 1]. Current agent CLIs are interactive terminal programs, so TIDE will host them through a PTY plus a terminal-emulation library (for example `alacritty_terminal` or `vt100`) rendered into Ratatui panes[cite: 1]. Structured or non-interactive agent modes will be used where an agent offers them, and a full-terminal handoff mode will remain available as a fallback[cite: 1]. Windows ConPTY behavior (resize, escape sequences, synchronized output, Unicode width) must be validated in Phase 0 before the architecture is finalized[cite: 1].

### 2. Keybinding model

When focus is inside a shell or agent pane, most keys must pass through to the child program[cite: 1]. TIDE will use a leader key or modal scheme to reserve a small, documented set of control chords, with an escape hatch that always returns focus to TIDE[cite: 1]. Chords intercepted by the host terminal (for example by Windows Terminal) will be catalogued and avoided[cite: 1]. The model will be designed in Phase 0 and Phase 1, before panes multiply[cite: 1].

### 3. Session persistence and process-tree lifecycle

A decision on session lifetime is required early because it affects architecture[cite: 1]:

- **Single process (initial default):** child sessions live and die with TIDE[cite: 1]. Simpler, but a TIDE crash or exit ends running agents[cite: 1].
- **Daemon/client split:** a background process owns the PTYs so sessions survive TIDE exiting, enabling detach and reattach[cite: 1].

The module boundaries will be drawn so a daemon split remains possible later, and the decision will be recorded after the Phase 0 spike[cite: 1]. Whatever is chosen, TIDE must warn the user before closing a workspace with running sessions[cite: 1]. 

Additionally, because coding agents frequently spawn child processes (test runners, compilers, dev servers) that can outlive the top-level PTY shell and lock files in a worktree, each session will be bound to an OS process group / **Windows Job Object** (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) to guarantee clean teardown of the entire process tree upon session stop.

### 4. Agent status with confidence levels

Detecting what an agent is doing from its output is inherently unreliable[cite: 1]. TIDE will therefore report status together with how it was determined[cite: 1]:

- **Confirmed:** reported by an agent-provided hook, structured output, or process state (running, exited, exit code)[cite: 1].
- **Inferred:** derived from heuristics such as output idle time or recognized prompt patterns[cite: 1].
- **Unknown:** no reliable signal; the UI says so instead of guessing[cite: 1].

Inferred states are labeled as such, and "needs input" notifications from heuristics are treated as hints, not facts[cite: 1].

### 5. Worktree lifecycle and bootstrapping

Each concurrent agent gets its own worktree and branch by default[cite: 1]. The lifecycle is explicit: create, bootstrap (copy/symlink configured untracked files like `.env` or set shared build cache env vars), run, review diff, integrate (merge, cherry-pick, or patch) or discard, then clean up[cite: 1]. TIDE never merges or overwrites concurrent agent changes silently[cite: 1]. Conflicts are surfaced to the user with the affected files named[cite: 1].

### 6. Concurrency model and frame pacing

Threads and bounded channels (`crossbeam-channel`) are the default for PTY readers, file watching, and Git subprocesses, feeding a single UI state loop[cite: 1]. To prevent high-throughput PTY output (e.g., verbose builds) from starving the UI thread, PTY reader threads update their terminal emulator grid in-place and emit coalesced dirty notifications, while the UI render loop is frame-paced (capped at ~60 FPS / 16ms intervals)[cite: 1]. Tokio is adopted only if a concrete need appears[cite: 1]. UI, terminal, agent, Git, filesystem, and configuration code live in separate modules, with platform-specific process and PTY code behind clear interfaces[cite: 1].

## Proposed Technology Stack

| Area | Proposed technology | Purpose |
| --- | --- | --- |
| Language | Rust stable | Native application and process management[cite: 1] |
| TUI | Ratatui | Layout and widgets[cite: 1] |
| Terminal events | Crossterm | Keyboard input and terminal control[cite: 1] |
| PTY management | portable-pty (ConPTY on Windows) + Windows Job Objects | Interactive shell, agent processes, and clean process-tree teardown[cite: 1] |
| Terminal emulation | alacritty\_terminal or vt100 | Parse and render child terminal output[cite: 1] |
| Git | Git CLI initially | Repository and worktree operations[cite: 1] |
| Configuration | Serde + TOML | Readable settings and worktree bootstrap rules[cite: 1] |
| File watching | notify (with debouncing) | Detect external file changes[cite: 1] |
| Concurrency | std threads + crossbeam-channel; Tokio only if needed | Background I/O and frame-paced UI signaling[cite: 1] |
| Text buffers and syntax highlighting | Ropey, Tree-sitter | Deferred with the built-in editor[cite: 1] |

The stack is provisional[cite: 1]. PTY and emulator behavior must be validated on Windows 11 in Phase 0[cite: 1].

## Safety Requirements

- Never silently discard unsaved work, uncommitted changes, or an agent's changes[cite: 1].
- Never merge concurrent agent changes without explicit user action[cite: 1].
- Require confirmation for destructive operations (discarding changes, deleting worktrees or branches, force operations), and show what will be lost[cite: 1].
- Make agent permissions visible: agents run with the user's own permissions, and TIDE shows each session's working directory and launch command[cite: 1]. TIDE does not claim to sandbox agents[cite: 1].
- Do not auto-approve agent prompts or inject input into an agent without an explicit user action[cite: 1].
- Keep secrets out of logs and configuration examples; do not log full terminal contents by default[cite: 1].
- Show process state and errors rather than hiding failures[cite: 1].

## Non-Functional Requirements and Targets

- **Lightweight:** no bundled browser engine[cite: 1].
- **Responsive:** keyboard input and rendering stay responsive while child processes run[cite: 1].
- **Cross-platform:** platform-specific behavior isolated behind interfaces; Windows first[cite: 1].
- **Reliable:** handle child exits, orphaned child processes, invalid configuration, missing executables, file changes, and resizing[cite: 1].
- **Maintainable:** clear module separation[cite: 1].
- **Transparent:** visible process state and clear errors[cite: 1].

Proposed initial targets, to be validated and adjusted after Phase 0 measurements (these are design goals, not benchmark claims)[cite: 1]:

| Metric | Initial target |
| --- | --- |
| Startup to interactive UI | under 500 ms on a typical development machine[cite: 1] |
| Keypress to rendered echo in a shell pane | under 50 ms at the 95th percentile[cite: 1] |
| Idle CPU with 3 idle sessions | near zero (under 1%)[cite: 1] |
| TIDE memory excluding child processes | to be measured, then budgeted[cite: 1] |
| Output flood (large log stream) | UI stays responsive; rendering is throttled/coalesced at ~60 Hz[cite: 1] |

Agent processes may use far more memory than TIDE itself, so measurements report TIDE and child processes separately[cite: 1].

## Roadmap

### Phase 0: Terminal and agent feasibility spike

- Run a PTY plus terminal emulator inside a Ratatui pane on Windows 11[cite: 1].
- Host a real agent CLI and a terminal editor; test resize, Unicode width, colors, mouse, and clean process-tree termination via Windows Job Objects[cite: 1].
- Prototype the keybinding model and confirm which chords reach TIDE[cite: 1].
- Validate frame-paced rendering under an output flood[cite: 1].
- Record findings, choose the emulator library, and decide the persistence approach[cite: 1].
- **Exit criterion:** an interactive agent CLI is usable inside a TIDE pane on Windows 11 with clean process-tree teardown, or the architecture is revised before continuing[cite: 1].

### Phase 1: Workspace and terminal sessions

- Launch from a project directory and render the main layout[cite: 1].
- Implement pane focus, keyboard navigation, and the keybinding model[cite: 1].
- Add multiple shell sessions with create, rename, switch, and close[cite: 1].
- Add configuration loading, a file explorer, and shortcut help[cite: 1].

### Phase 2: One agent integration

- Implement a single agent adapter first[cite: 1].
- Detect capabilities; manage launch, stop, restart, and exit handling[cite: 1].
- Display session status with confidence levels[cite: 1].
- Validate interactive workflows (input, resize, long-running output) before adding more agents[cite: 1].

### Phase 3: Worktree isolation and review

- Create per-session worktrees and branches[cite: 1].
- Implement worktree bootstrap hooks (symlink/copy `.env`, shared build cache configuration).
- Show per-session changed files and diffs[cite: 1].
- Implement the review-before-integration flow and explicit cleanup[cite: 1].
- Support concurrent sessions once a single session is reliable[cite: 1].

### Phase 4: Git polish and editor integration

- Staging and unstaging, commits, and branch switching, including hunk-level staging where practical[cite: 1].
- Open files in an external editor, including within a TIDE pane[cite: 1].
- Defer a built-in editor; revisit only if external-editor workflows prove insufficient[cite: 1].

### Phase 5: Hardening

- Automated tests: state and layout snapshots with Ratatui's test backend, Git operations against temporary repositories, and PTY and process-management integration tests[cite: 1].
- Error handling, shutdown behavior with running sessions, documentation, and packaging[cite: 1].
- Measure startup time, idle usage, and behavior with multiple active sessions against the targets above[cite: 1].
- Verify Linux and macOS behavior before declaring cross-platform support[cite: 1].

## Success Criteria

The MVP is successful when a developer can[cite: 1]:

- Launch TIDE in a local repository on Windows 11[cite: 1].
- Open and switch between multiple shell sessions[cite: 1].
- Launch a supported coding agent in an isolated, bootstrapped worktree and see its status and working directory[cite: 1].
- Stop and restart the agent without leaving orphaned child processes or corrupting the repository[cite: 1].
- Review the agent's diff and integrate or discard it deliberately[cite: 1].
- Stage changes and create a commit[cite: 1].
- Open a file in an external editor from TIDE[cite: 1].
- Identify the active project, branch, terminal, and agent session at a glance[cite: 1].
- Recover from common errors without losing work[cite: 1].

## Risks and Open Questions

- **Windows ConPTY quirks** may force emulator workarounds or reduce fidelity[cite: 1]. Mitigation: Phase 0 spike[cite: 1].
- **Orphaned child processes locking worktree files on Windows:** agents spawn compilers and servers that hold file locks. Mitigation: Windows Job Objects per session validated in Phase 0.
- **Cold worktrees lacking untracked dependencies/configs:** fresh worktrees lack `.env` or build caches. Mitigation: configurable worktree bootstrap hooks in Phase 3.
- **Agent CLI churn:** agent interfaces change frequently[cite: 1]. Mitigation: adapter boundary and capability detection[cite: 1].
- **Status unreliability:** mitigated by confidence levels and an honest Unknown state[cite: 1].
- **Persistence:** deferring a daemon split risks costly retrofits[cite: 1]. Mitigation: module boundaries and an early recorded decision[cite: 1].
- **Scope creep toward a full IDE:** mitigated by deferring the built-in editor[cite: 1].
- **Name collision:** verify before publishing[cite: 1].

## Summary

TIDE is a proposed native terminal workspace for running and reviewing parallel coding agents[cite: 1]. It uses a real embedded terminal, per-session Git worktrees, honest status reporting, and explicit safety rules[cite: 1]. Success depends on early validation of terminal and agent behavior on Windows 11, not on the number of panels on screen[cite: 1].