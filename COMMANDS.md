# rukiOS Commands

This file lists the commands currently implemented in the `ruki` CLI and the separate `ruki-monitor` app. Run Cargo examples from the repository root. After installing the applications, replace `cargo run --` with `ruki` (or use `ruki-monitor` for the standalone monitor).

## Main `ruki` application

| Command | What it does |
| --- | --- |
| `cargo run` or `ruki` | Opens the interactive System Console TUI. |
| `cargo run -- system` or `ruki system` | Prints operating system, kernel, uptime, and memory information. |
| `cargo run -- disk` or `ruki disk` | Lists local mounted filesystem capacity, including size, used space, available space, and mount point. |
| `cargo run -- memory` or `ruki memory` | Prints current memory usage. |
| `cargo run -- network` or `ruki network` | Lists active network interfaces and their IP addresses. |
| `cargo run -- processes [number]` or `ruki processes [number]` | Lists running processes sorted by CPU use. The optional positive `number` sets how many to show; default is 10. |
| `cargo run -- services` or `ruki services` | Lists currently running systemd services. |
| `cargo run -- health` or `ruki health` | Summarizes memory, root disk, and network status with `OK`, `WARN`, or `UNKNOWN` labels. |
| `cargo run -- health --json` or `ruki health --json` | Prints the health summary as JSON for scripts and other tools. |
| `cargo run -- logs [number]` or `ruki logs [number]` | Shows recent system journal entries. The optional positive `number` sets how many to show; default is 40. |
| `cargo run -- help`, `ruki help`, `ruki --help`, `ruki -h` | Prints CLI usage and the command list. |

Examples:

```bash
cargo run -- processes 20
cargo run -- logs 100
cargo run -- health --json
```

## System Console keyboard controls

Open the System Console by running `cargo run` or `ruki` without a command.

| Keys | Action |
| --- | --- |
| `j` / `k`, `↓` / `↑` | Move between sections. |
| `h` / `l`, `←` / `→` | Move to the previous or next section. |
| `1`–`9`, `0` | Jump to sections 1–10. |
| `g` / `G` | Jump to the first or last section. |
| `Ctrl-u` / `Ctrl-d`, `Page Up` / `Page Down` | Scroll section content. |
| `r` | Refresh the displayed system snapshot. |
| `q` / `Esc` | Quit the System Console. |

The sections are Overview, Monitor, System, Disk, Memory, Network, Processes, Services, Health, and Logs.

## Standalone live monitor

Run `cargo run --bin ruki-monitor` from the repository root, or run `ruki-monitor` after installation. This opens the live system monitor in its own terminal screen.

| Keys / input | Action |
| --- | --- |
| `Tab`, `h` / `l`, `←` / `→` | Move between dashboard panels. |
| `1`–`6` or mouse click | Select Overview, CPU, Memory, Disks, Network, or Processes. |
| `j` / `k`, `↓` / `↑` | Scroll the process list when Processes is selected. |
| `Page Up` / `Page Down` | Move one process-list page. |
| `g` / `G`, `Home` / `End` | Jump to the first or last process-list page. |
| `r` | Refresh readings immediately. |
| `q` / `Esc` | Quit the monitor. |

Metrics refresh automatically about once per second. Some hardware readings depend on what Linux and the installed drivers expose.

## External tools and limits

Some read-only commands call standard Linux tools: `df` for disk space, `ip` for network addresses, `ps` for processes, `systemctl` for services, and `journalctl` for logs. Journal visibility depends on the current user's permissions. The commands documented here are the implemented commands; package management, file management, and natural-language requests are planned, not available yet.
