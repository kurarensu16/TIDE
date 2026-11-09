# tide-cli

**Terminal Integrated Development Environment for parallel AI coding agents.**

This is the official npm wrapper distribution for [TIDE](https://github.com/kurarensu16/TIDE). It automatically installs and launches the pre-compiled native binary for your operating system (Windows, macOS, and Linux).

---

## Quick Start

### Run Instantly via `npx` (No installation needed)
```bash
npx tide-cli
```

### Install Globally via `npm`
```bash
npm install -g tide-cli
```

Once installed globally, you can launch TIDE from any terminal using `tide` or `tide-cli`:
```bash
# Open current working directory
tide .

# Open a specific workspace
tide "C:\path\to\project"
```

---

## Features
* **Zero Compilation**: Downloads the high-performance native Rust binary directly.
* **Full Terminal Capability**: Directly inherits stdio with complete ConPTY / VT100 interactive support.
* **Windows Job Object & Git Worktree Isolation**: Orchestrate multiple coding-agent CLIs in parallel.

---

## Documentation & Repository
* Website & Docs: [https://kurarensu16.github.io/TIDE/](https://kurarensu16.github.io/TIDE/)
* GitHub: [https://github.com/kurarensu16/TIDE](https://github.com/kurarensu16/TIDE)

