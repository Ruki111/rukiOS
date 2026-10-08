# rukiOS — Project Requirements

## What We Are Building

rukiOS is an **Arch Linux–based distribution** focused on a **terminal-first operating system experience**.

We are not building a new Linux kernel or replacing existing applications. We are building a custom system experience and tooling on top of Arch Linux.

## Core Philosophy

- **Keyboard First:** Every system-management workflow must be usable from the keyboard. Keyboard navigation is the default and must not be a second-class alternative to mouse use.
- **Mouse Optional:** Mouse input may make pointer-oriented tasks more convenient, but no essential action may require a mouse. Use it where it adds value, such as selecting charts, resizable panels, or graphical applications.
- **Terminal First:** System management should primarily happen through the terminal, with polished TUIs for interactive and information-dense tasks.
- **Use GUIs Where They Fit:** Graphical applications such as browsers, editors, and creative tools remain welcome. The system should not force a terminal workflow where a GUI is a better fit.
- **Use Existing Linux Software:** Reuse mature Linux tools instead of reinventing them.
- **AI Integrated:** AI should be deeply integrated into system management and automation.
- **Simple and Developer Friendly:** The system should be fast, minimal, keyboard-driven, and comfortable for developers.

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
- Support mouse input as an optional convenience where it improves interaction; keyboard users must be able to complete the same workflows.
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

`ruki` should provide a consistent interface for managing the system.

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
- Existing GUI applications

## Graphical Environment

A Wayland compositor such as **Sway or Hyprland** may be included to support graphical applications and modern desktop workflows.

It is **not the core of rukiOS**.

The terminal and ruki tooling remain the primary interface.

## Technology Direction

- **Rust:** ruki CLI/TUI and system-level tooling.
- **Bash:** Installation and small system scripts where appropriate.
- **Arch Linux:** Base distribution.
- **Wayland:** Graphical protocol when graphical support is required.
- **Sway/Hyprland:** Optional/replaceable compositor layer.
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
- Not a GUI-free requirement for every application.

## Long-Term Goal

Make rukiOS feel like:

> **Linux where the terminal is the operating system's primary interface, and AI is the intelligent layer that helps you control it.**
