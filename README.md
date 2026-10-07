# Rookie OS

Rookie OS is an Arch Linux–based distribution built around a terminal-first way to manage and use a computer. It aims to make the terminal the primary system interface, with TUIs for complex tasks and graphical applications available when they are useful.

Rookie OS reuses the Linux kernel, Arch's package ecosystem, and mature tools such as `pacman`, `systemd`, and NetworkManager. It is a custom system experience and tooling layer, not a new kernel or an operating system built from scratch.

## Project status

Rookie OS is at the early planning and development stage. The first milestone is a Rust-based `rookie` CLI that works on an existing Arch Linux installation. Building a custom ISO comes later.

## Vision

- Manage the system from a consistent `rookie` command-line interface.
- Offer terminal user interfaces for areas such as services, packages, networking, processes, storage, and logs.
- Integrate AI to translate natural-language requests into understandable system actions.
- Make automation safe by showing planned actions, asking for confirmation when appropriate, recording command and output history, detecting dangerous operations, and supporting rollback where practical.
- Keep the system minimal, keyboard-driven, and comfortable for developers.
- Support graphical applications and, optionally, a replaceable Wayland compositor such as Sway or Hyprland.

## Planned CLI

The initial CLI milestone focuses on:

```bash
rookie system
rookie services
```

The broader interface may grow to include commands such as:

```bash
rookie network
rookie packages
rookie files
```

Natural-language requests are a longer-term goal, for example:

```text
rookie "install Docker and start it automatically"
```

## Development direction

1. Build the Rust CLI for use on existing Arch installations.
2. Add a TUI for system management.
3. Expand system-management capabilities.
4. Integrate AI with clear plans and safe execution controls.
5. Package the Rookie OS experience and eventually build a custom ISO.

## Technology

- **Base:** Arch Linux
- **CLI, TUI, and system tooling:** Rust
- **Installation and small system scripts:** Bash where appropriate
- **System services:** systemd
- **Networking:** NetworkManager
- **Optional graphical layer:** Wayland with a compositor such as Sway or Hyprland

## Contributing

The project is in its early stages. Before contributing, review [REQUIREMENTS.md](REQUIREMENTS.md) for the project goals, principles, and initial development scope.

## Requirements

See [REQUIREMENTS.md](REQUIREMENTS.md) for the full project requirements and long-term vision.
