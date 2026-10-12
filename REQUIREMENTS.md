# rukiOS — Project Requirements

## What We Are Building

rukiOS is an **Arch Linux–based distribution** focused on a **terminal-first operating system experience**.

We are not building a new Linux kernel or replacing existing applications. We are building a custom system experience and tooling on top of Arch Linux.

## Core Philosophy

- **Keyboard First:** The keyboard is the primary control method throughout the distribution. Essential workflows must be complete without a mouse.
- **Vim-Inspired Navigation:** Use consistent Vim-style movement and command patterns, alongside familiar arrow, page, and conventional shortcuts. Show shortcuts and make them discoverable.
- **TUI and CLI First:** System setup, management, and daily system workflows are provided through a polished TUI or scriptable CLI. A graphical settings layer is outside the core product direction.
- **Filesystem-Accessible:** System state and configuration should be available through clear, documented filesystem paths and formats where practical. Users and agents should be able to inspect changes and understand where system data lives; filesystem access complements the CLI/TUI rather than hiding behavior.
- **Arch Linux Base:** Build on Arch Linux and its package ecosystem, reusing the kernel and mature system components instead of replacing them without a clear need.
- **Agents Are First-Class:** Integrated AI agents use documented ruki interfaces and bounded system tools. They should understand the distribution's operations and state, propose changes clearly, and request approval for consequential actions.
- **Simple and Developer Friendly:** Keep the system fast, understandable, maintainable, and comfortable for developers.
- **Graphical Applications Are Out of Scope for the Core Experience:** Do not make GUI applications, mouse-oriented workflows, or a desktop environment a product requirement. Any later optional graphical support must not displace the keyboard/TUI/CLI experience.

## ruki System Monitor

The first substantial ruki application is a system monitor and task manager for Linux, inspired by tools such as Windows Task Manager, btop, and bottom. It is a rukiOS application that runs on an existing Linux installation; it is not an operating system kernel or a replacement for the Linux kernel.

### Monitoring requirements

- Show total and per-core CPU usage, load averages, and CPU temperature when supported.
- Show GPU identity and utilization, memory, and temperature when supported by available hardware interfaces and permissions.
- Show memory and swap totals, usage, and availability.
- Show mounted filesystem capacity and disk read/write activity where available.
- Show processes with useful columns, sorting, search/filtering, and process-tree views. Any process-changing action must be explicit and confirmed.
- Show network interfaces, per-interface receive/transmit rates, and cumulative data totals; distinguish physical interfaces from virtual/container interfaces where possible.
- Refresh live metrics on a predictable sampling interval. Moving focus or navigating panels must not itself reset samples or cause readings to jump; non-monitor sections may keep a snapshot until the user refreshes them.
- Clearly mark metrics that are unavailable instead of showing misleading zero values. Hardware-specific readings must degrade gracefully without requiring elevated privileges for ordinary read-only monitoring.

### Interaction requirements

- Provide keyboard access to every panel, selection, sort, filter, and available action.
- Show discoverable shortcut hints in the interface and provide a help view.
- Support Vim-style movement and standard arrow/page keys, with consistent behavior across screens.
- Make keyboard operation the complete, primary interaction model; mouse support is not a requirement for the core TUI.
- Adapt layouts to terminal size, preserve readable labels, and bound scrolling to available content.
- Use clear status colors and text labels so meaning is not communicated by color alone.

### Growth and maintenance requirements

- Keep presentation, application behavior, domain types, and Linux data collection separate.
- Use Cargo workspaces and multiple packages when components become coherent reusable capabilities; avoid a monolithic crate and avoid creating empty speculative crates prematurely.
- Isolate Linux-specific `/proc`, `/sys`, systemd, and networking integrations behind platform adapters so they do not leak into UI rendering.
- Keep the executable entry point thin; route commands and initialize application services there, but keep feature implementation in focused modules/crates.
- Document crate responsibilities and allowed dependency direction before adding major subsystems.

## Main Selling Points

### 1. ruki command and system monitor

The `ruki` executable is the main user-facing system-management application. Running `ruki` without a subcommand opens the keyboard-first system-monitor TUI; CLI subcommands remain available for quick checks and automation.

`ruki` should provide a consistent interface for managing the system through CLI commands, its keyboard-first TUI, and documented filesystem interfaces where appropriate.

Examples:

```bash
ruki system
ruki services
ruki network
ruki packages
ruki files
```

Eventually:

```bash
ruki "connect me to my home Wi-Fi"
ruki "install Docker and start it automatically"
ruki "show me what is using my RAM"
```

The natural-language interface is backed by integrated agents that call documented ruki/system operations. It must not depend on an unrestricted, opaque shell session as its system integration contract.

### 2. TUI-Based System Management

System configuration should preferably use TUIs rather than traditional GUI settings applications.

Potential areas:

- Network
- Packages
- Services
- Processes
- Users
- Storage
- Displays
- System information
- Logs
- Configuration

### 3. AI System Interface

AI will be integrated into ruki so users can describe what they want in natural language.

The AI should translate the user's intent into safe system operations.

Example:

```text
User:
Install Docker and make it start on boot.

ruki:
I will:
1. Install Docker.
2. Enable docker.service.
3. Start docker.service.

Proceed? [y/N]
```

### 4. Safe Automation

AI must **not blindly execute commands**.

Requirements:

- Show planned actions.
- Ask for confirmation for potentially destructive operations.
- Provide command/output history.
- Detect dangerous operations.
- Support rollback where practical.

### 5. Arch Linux Base

rukiOS will use **Arch Linux** as its foundation.

We will reuse:

- Linux kernel
- Arch package ecosystem
- `pacman`
- `systemd`
- NetworkManager
- Existing Linux utilities

## Graphical Environment

A graphical desktop or compositor is not part of the required core experience or initial product scope. The distribution is designed to be usable through terminal, TUI, CLI, and documented filesystem interfaces. Optional graphical application support can be considered later only if it preserves this direction and does not introduce GUI-only system workflows.

## Technology Direction

- **Rust:** ruki CLI/TUI and system-level tooling.
- **Bash:** Installation and small system scripts where appropriate.
- **Arch Linux:** Base distribution.
- **Wayland/Sway/Hyprland:** Not required for the core product; reconsider only for explicitly optional graphical support.
- **Existing Linux tools:** Reuse wherever possible.

## Development Strategy

We will **not start by building an ISO**.

First:

```text
Arch Linux
    ↓
ruki CLI
    ↓
ruki TUI
    ↓
System management
    ↓
AI integration
    ↓
Safety/rollback
    ↓
rukiOS packaging
    ↓
Custom ISO
```

## First-Release User Outcomes

The first useful rukiOS release should let a person complete these four jobs from a keyboard, using the CLI/TUI and inspectable system data:

1. **Check system health:** Open a live overview of CPU, memory, storage, network, and health warnings, with unavailable readings clearly identified.
2. **Find and inspect a process:** Search or sort running processes, understand their resource use, and explicitly confirm before requesting a process stop.
3. **Manage a system service:** Find a service, inspect its state, and start, stop, or restart it with a clear confirmation before a state-changing action.
4. **Ask an integrated agent about system state:** Ask a plain-language question such as “What is using my memory?” and get an answer grounded in data returned by documented ruki tools. This first agent workflow is read-only.

These are the first-release outcomes, not a promise that every surrounding subsystem is already implemented. Defer an ISO/installer, GUI control panels, broad package/network configuration, multi-distro support, and agent-initiated system changes until these four workflows work end-to-end. Existing CLI, TUI, and filesystem interfaces should expose the same underlying system state where appropriate.

## Initial implementation milestone

The first milestone is a useful **ruki system monitor and management TUI** that runs on an existing Arch installation. It should establish the interaction, metric collection, and code-organization foundations before broader system-management and distribution work.

The existing starter implementation already includes:

```bash
ruki
ruki system
ruki services
```

The monitor roadmap includes live CPU/GPU, memory, storage, processes, and network details described above. Implement it in small, maintainable increments; do not treat the starter feature list as the final product scope.

## What rukiOS Is NOT

- Not a new Linux kernel.
- Not a completely new operating system from scratch.
- Not a collection of unnecessary replacement applications.
- Not just an Arch theme.
- Not just Hyprland with dotfiles.
- Not a mouse-first or GUI-centered desktop distribution.
- Not an agent that operates outside documented, bounded system interfaces.

## Long-Term Goal

Make rukiOS feel like:

> **Linux where the terminal is the operating system's primary interface, and AI is the intelligent layer that helps you control it.**
