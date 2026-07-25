# Linux Release Build — Fount

Builds a `.tar.gz` tarball for Linux distribution.

## Prerequisites

- Rust toolchain (the script installs it if missing)

## Quick Start

```bash
chmod +x Release/linux/build-tarball.sh
./Release/linux/build-tarball.sh
```

This will:
1. Build the Rust binary with `cargo build --release`
2. Package a `.tar.gz` with binary, icon, desktop file, and install scripts
3. Output → `Release/artifacts/Fount-Linux-x64-<version>.tar.gz`

## Output

```
Release/artifacts/
  Fount-Linux-x64-<version>.tar.gz
```

Inside the tarball:
```
usr/share/fount/fount                → Main binary
usr/share/applications/fount.desktop  → Desktop entry
usr/share/icons/hicolor/256x256/apps/fount.png  → App icon
install.sh                            → Installs to /usr/share
uninstall.sh                          → Removes from /usr/share
```

## User Installation

```bash
tar -xzf Fount-Linux-x64-<version>.tar.gz
sudo ./install.sh
```

## Uninstallation

```bash
sudo ./uninstall.sh
```
