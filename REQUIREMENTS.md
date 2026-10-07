# Rookie OS — Project Requirements

## What We Are Building

Rookie OS is an **Arch Linux–based distribution** focused on a **terminal-first operating system experience**.

We are not building a new Linux kernel or replacing existing applications. We are building a custom system experience and tooling on top of Arch Linux.

## Core Philosophy

- **Terminal First:** System management should primarily happen through the terminal.
- **TUI Over GUI:** When a terminal interface becomes complex, provide a TUI instead of a GUI.
- **GUI Only When Necessary:** Graphical applications such as Chrome, VS Code, browsers, editors, etc. can remain GUI applications.
- **Use Existing Linux Software:** Reuse mature Linux tools instead of reinventing them.
- **AI Integrated:** AI should be deeply integrated into system management and automation.
- **Simple and Developer Friendly:** The system should be fast, minimal, keyboard-driven, and comfortable for developers.

## Main Selling Points

### 1. Rookie CLI

The central component of Rookie OS.

`rookie` should provide a consistent interface for managing the system.

Examples:

```bash
rookie system
rookie services
rookie network
rookie packages
rookie files
```

Eventually:

```bash
rookie "connect me to my home Wi-Fi"
rookie "install Docker and start it automatically"
rookie "show me what is using my RAM"
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

AI will be integrated into Rookie so users can describe what they want in natural language.

The AI should translate the user's intent into safe system operations.

Example:

```text
User:
Install Docker and make it start on boot.

Rookie:
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

Rookie OS will use **Arch Linux** as its foundation.

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

It is **not the core of Rookie OS**.

The terminal and Rookie tooling remain the primary interface.

## Technology Direction

- **Rust:** Rookie CLI/TUI and system-level tooling.
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
Rookie CLI
    ↓
Rookie TUI
    ↓
System management
    ↓
AI integration
    ↓
Safety/rollback
    ↓
Rookie OS packaging
    ↓
Custom ISO
```

## Week 1 Goal

Build the first version of the **Rookie CLI in Rust**.

Initial capabilities:

```bash
rookie system
rookie services
```

The first version should be simple, reliable, and actually useful on an existing Arch installation.

## What Rookie OS Is NOT

- Not a new Linux kernel.
- Not a completely new operating system from scratch.
- Not a collection of unnecessary replacement applications.
- Not just an Arch theme.
- Not just Hyprland with dotfiles.
- Not a GUI-free requirement for every application.

## Long-Term Goal

Make Rookie OS feel like:

> **Linux where the terminal is the operating system's primary interface, and AI is the intelligent layer that helps you control it.**
