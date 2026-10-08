# Arch package: `ruki`

This directory contains an Arch `PKGBUILD` for the current rukiOS applications. It builds and installs both `ruki` (the System Console and CLI) and `ruki-monitor` (the standalone live monitor). It tracks the GitHub `main` branch and uses the committed `Cargo.lock` for locked Rust dependency resolution.

## Build and install

From a clone of the repository, after the packaging changes are available on GitHub:

```bash
cd packaging/arch
makepkg -si
```

`makepkg` will fetch Rust dependencies, build both binaries, run the workspace tests, and create an Arch package. `-i` asks pacman to install the package after a successful build. To only create the package, run `makepkg` without `-i`.

The package depends on `systemd`, `iproute2`, `procps-ng`, and `coreutils` for the Linux tools used by system inspection and management features. Starting, stopping, or restarting services still depends on the permissions of the logged-in user. No service is automatically enabled or modified during installation.

## Scope

This packages the current user-space applications; it is not an installer, bootable ISO, kernel, or complete Linux distribution. The package recipe should be updated to use immutable release sources and checksums when rukiOS starts publishing tagged releases.
