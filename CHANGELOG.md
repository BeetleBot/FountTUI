# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2.1] - 2026-04-04

### Added
- **Fount Portable (Linux)**: Integrated a fully static MUSL build for the Linux release, ensuring the application works on any Linux distribution (Arch, Ubuntu, Fedora, Alpine, etc.) without external dependencies.
- Added GitHub documentation for release procedures.

### Changed
- Streamlined release process to focus on portable Linux binaries and source code.
- Moved `clipboard-win` to Windows-only target dependencies to improve Linux build isolation.

### Removed
- Pre-built binaries for macOS and Windows (users on these platforms can still compile from source using `cargo`).

## [0.1.2] - 2026-04-04

### Added
- **Scene Navigator**: Quick navigation through scenes with `Ctrl+H`.
- **Settings Pane**: Interactive settings configuration with `Ctrl+P`.
- **Multi-Buffer Support**: Open and switch between multiple Fountain files.
- **Auto-Title Page**: Automatically generate title page metadata for new files.
- **Search**: Case-insensitive regex search support.
- **Undo/Redo**: Global history state management for all buffers.
- **Status Bar**: Detailed cursor position reporting (line, column, character percentage).
- **Safe Exit**: Automatic emergency saving if the application crashes.

### Changed
- Migrated primary keybindings to `Ctrl`-based modifiers for better macOS compatibility.
- Renamed project to **Fount**.

### Fixed
- Improved text rendering and indentation for across different terminal sizes.
