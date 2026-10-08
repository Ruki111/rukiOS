# rukiOS

rukiOS is an Arch Linux–based distribution built around a terminal-first way to manage and use a computer. It aims to make the terminal the primary system interface, with TUIs for complex tasks and graphical applications available when they are useful.

rukiOS reuses the Linux kernel, Arch's package ecosystem, and mature tools such as `pacman`, `systemd`, and NetworkManager. It is a custom system experience and tooling layer, not a new kernel or an operating system built from scratch.

## Project status

rukiOS is in early development. Its first milestone is a Rust-based `ruki` CLI that runs on an existing Linux installation. Building a custom ISO comes later.

## Vision

- Manage the system from a consistent `ruki` command-line interface.
- Offer terminal user interfaces for areas such as services, packages, networking, processes, storage, and logs.
- Integrate AI to translate natural-language requests into understandable system actions.
- Make automation safe by showing planned actions, asking for confirmation when appropriate, recording command and output history, detecting dangerous operations, and supporting rollback where practical.
- Keep the system minimal, keyboard-driven, and comfortable for developers.
- Support graphical applications and, optionally, a replaceable Wayland compositor such as Sway or Hyprland.

## CLI

The current CLI provides these system information commands:

```bash
cargo run -- system
cargo run -- disk
cargo run -- memory
cargo run -- network
cargo run -- processes       # show the top 10 by CPU use
cargo run -- processes 20    # choose how many processes to show
cargo run -- services       # list active systemd services
cargo run -- health
cargo run -- health --json
cargo run -- help
```

Run these commands from the project directory. Once installed, use `ruki` in place of `cargo run --`.

`health` summarizes memory, root disk usage, and active network interfaces. It reports `WARN` when disk use reaches 85% or memory use reaches 90%, and `UNKNOWN` when a reading is unavailable. The JSON form provides the same checks in a machine-readable format.

The CLI currently relies on standard Linux utilities: `df`, `ip`, `ps`, `systemctl`, and `uname`. Some commands require the corresponding utility or systemd to be available.

Commands for packages, files, and natural-language requests are planned for later.

Natural-language requests are a longer-term goal, for example:

```text
ruki "install Docker and start it automatically"
```

## Development direction

1. Build the Rust CLI for use on existing Arch installations.
2. Add a TUI for system management.
3. Expand system-management capabilities.
4. Integrate AI with clear plans and safe execution controls.
5. Package the rukiOS experience and eventually build a custom ISO.

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
