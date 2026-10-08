# rukiOS

rukiOS is an Arch Linux–based distribution built around a terminal-first way to manage and use a computer. It aims to make the terminal the primary system interface, with TUIs for complex tasks and graphical applications available when they are useful.

rukiOS reuses the Linux kernel, Arch's package ecosystem, and mature tools such as `pacman`, `systemd`, and NetworkManager. It is a custom system experience and tooling layer, not a new kernel or an operating system built from scratch.

## Project status

rukiOS is in early development. The Rust-based `ruki` program runs on an existing Linux installation and now has an initial full-screen TUI alongside its CLI commands. Building a custom ISO comes later.

## Vision

- Manage the system from a consistent `ruki` command-line interface.
- Offer terminal user interfaces for areas such as services, packages, networking, processes, storage, and logs.
- Integrate AI to translate natural-language requests into understandable system actions.
- Make automation safe by showing planned actions, asking for confirmation when appropriate, recording command and output history, detecting dangerous operations, and supporting rollback where practical.
- Keep the system minimal, keyboard-driven, and comfortable for developers.
- Support graphical applications and, optionally, a replaceable Wayland compositor such as Sway or Hyprland.

## Interface

Running `ruki` without a command opens the interactive **System Console**. The standalone **ruki-monitor** is a separate full-screen app that you can run in its own terminal window. Its live metrics refresh once per second. Use Tab or `h`/`l` to move between panels, `1`–`6` or mouse clicks to select Overview, CPU, Memory, Disks, Network, or Processes, `j`/`k` or the arrow keys to select and move through processes, `/` to filter, `s` to cycle sorting by CPU/memory/PID, `t` to toggle the process tree, and `x` to request a graceful stop with confirmation. Page Up/Down moves a page, Home/End or `g`/`G` jumps to the first/last process, `r` refreshes, and `q` or Escape quits. Compact terminals show a one-page Overview with key metrics; select a panel to inspect its details.

Start it from the project directory with:

```bash
cargo run --bin ruki
# In another terminal window:
cargo run --bin ruki-monitor
```

The monitor dashboard shows CPU load and per-core usage, recent CPU history, available GPU and temperature readings, memory and swap, root disk capacity and disk I/O, network throughput, and processes sorted by CPU. Hardware sensor support depends on what Linux exposes on the machine; unavailable values are shown as unavailable. The System Console sections cover overview, system information, disk, memory, network, processes, services, health, and recent logs. The command-line commands are still available for scripts and quick checks:

```bash
cargo run -- system
cargo run -- disk
cargo run -- memory
cargo run -- network
cargo run -- logs          # show the latest 40 journal entries
cargo run -- logs 100      # choose how many entries to show
cargo run -- processes       # show the top 10 by CPU use
cargo run -- processes 20    # choose how many processes to show
cargo run -- services       # list active systemd services
cargo run -- health
cargo run -- health --json
cargo run -- help
```

Run these commands from the project directory. Once installed, use `ruki` in place of `cargo run --`.

`health` summarizes memory, root disk usage, and active network interfaces. It reports `WARN` when disk use reaches 85% or memory use reaches 90%, and `UNKNOWN` when a reading is unavailable. The JSON form provides the same checks in a machine-readable format.

The interface uses Ratatui and Crossterm. System details rely on standard Linux utilities: `df`, `ip`, `ps`, `systemctl`, `journalctl`, and `uname`. Some sections require the corresponding utility or systemd to be available. Access to system logs depends on the journal permissions of the current user.

The repository workspace and planned package boundaries are described in [docs/architecture/PROJECT_STRUCTURE.md](docs/architecture/PROJECT_STRUCTURE.md).

Commands for packages, files, and natural-language requests are planned for later.

See [COMMANDS.md](COMMANDS.md) for the complete command reference and keyboard shortcuts for both TUIs.

Natural-language requests are a longer-term goal, for example:

```text
ruki "install Docker and start it automatically"
```

## Development direction

1. Expand the Rust CLI and TUI for use on existing Arch installations.
2. Improve system-management capabilities and usability.
3. Integrate AI with clear plans and safe execution controls.
4. Package the rukiOS experience and eventually build a custom ISO.

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
