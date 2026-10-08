# Project structure

## Current workspace

The repository is a Cargo workspace. The root manifest owns shared workspace settings and `Cargo.lock`; the runnable application lives in `apps/ruki`. The workspace defaults to that app, so `cargo run` from the repository root continues to work.

```text
rukiOS/
├── Cargo.toml                 # workspace settings and shared dependency versions
├── Cargo.lock                 # one lockfile for the workspace
├── apps/
│   └── ruki/                  # user-facing executable and composition root
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs        # entry point and command dispatch
│           └── tui.rs         # current TUI implementation
├── crates/                    # shared Rust packages as they become necessary
│   └── README.md
├── docs/
│   └── architecture/
│       └── PROJECT_STRUCTURE.md
├── packaging/arch/            # Arch PKGBUILD for the current user-space apps
├── scripts/                   # planned development and release helpers
└── assets/                    # planned themes, icons, and other bundled resources
```

## Target structure as capabilities grow

Add workspace packages around stable responsibilities, not around every source file. Keep member packages directly under `crates/`; Cargo can include them with a glob so adding a package does not require maintaining a long manifest list.

```text
apps/
└── ruki/                       # binary, CLI parsing, application wiring

crates/
├── ruki-core/                  # platform-neutral domain types and use cases
├── ruki-monitor/               # metric models, sampling rules, history/aggregation
├── ruki-platform-linux/        # procfs/sysfs, systemd, and Linux-specific adapters
├── ruki-tui/                   # Ratatui views, keyboard input, optional mouse input
├── ruki-cli/                   # non-interactive text/JSON commands
├── ruki-config/                # configuration model and loading
├── ruki-safety/                # plans, policies, confirmation, and audit records
└── ruki-ai/                    # natural-language planning integration, added later

docs/
├── architecture/               # architecture notes and decision records
└── user/                       # user guides and keyboard shortcut references

packaging/
└── arch/                       # PKGBUILD and Arch-specific packaging docs

assets/                         # themes, icons, and other static resources
scripts/                        # reproducible developer/release automation
```

This is a target map, not a request to create empty crates for every future idea. Add a package when it has a real implementation, a clear public API, and an independent reason to build, reuse, or maintain it.

## Dependency direction

```text
apps/ruki
├── ruki-cli ───────┐
├── ruki-tui ───────┼──> ruki-core / ruki-monitor
└── platform wiring ┘              ▲
                                    │
                       ruki-platform-linux
```

- `ruki-core` owns platform-neutral system-management concepts and use cases. It must not depend on Ratatui, terminal event handling, `/proc`, `/sys`, or systemd commands.
- `ruki-monitor` owns normalized metrics and sampling/history behavior. It defines what the UI consumes without deciding how Linux obtains the values.
- `ruki-platform-linux` implements data-source interfaces using Linux facilities. Other operating-system adapters can be added without changing UI code if portability becomes a goal.
- `ruki-tui` renders state and translates keyboard/mouse input into application actions. It does not read kernel files or execute system commands directly.
- `ruki-cli` handles command-line presentation and JSON/text output, using the same application operations as the TUI.
- `apps/ruki` is the composition root: it constructs adapters and starts the requested interface. Avoid putting feature logic in `main.rs`.

Dependencies point inward toward stable domain/application APIs. UI crates must not depend on Linux implementation details, and platform adapters must not depend on UI crates. Avoid cyclic package dependencies.

## Scaling rules

- Begin with focused modules inside a package; extract a workspace crate when a capability develops a stable boundary or needs separate reuse, ownership, or platform dependencies.
- Keep each crate organized by feature/domain, with small files for focused responsibilities rather than one giant `main.rs` or a flat directory of unrelated modules.
- Put unit tests beside implementation where useful and integration tests in each package's `tests/` directory. Keep shared fixtures under `tests/fixtures/` when multiple packages use them.
- Add architecture decision records under `docs/architecture/decisions/` when a choice affects package boundaries or public interfaces.
- Keep platform-specific and optional integrations behind interfaces; unavailable sensors or permissions should produce explicit unavailable values, not fabricated zeroes.
- Keep workspace dependencies, package metadata, and lint policy in the root manifest. Add shared linting and CI checks as the workspace gains packages.

## Research basis

- The Rust Book recommends splitting growing programs into modules/files first, extracting packages when appropriate, and using workspaces for related packages that evolve together.
- The Cargo Book recommends a flat `crates/` member directory and documents shared lockfiles, output directories, package selection, and inherited metadata/dependencies.
- The Rust system monitor `bottom` separates source code, docs, assets, configuration/schema, scripts, demos, and tests while keeping related Rust packages in one repository. Its feature areas (CPU, memory, network, disks, temperature, and process views) are a useful reference for the monitor's scope.
