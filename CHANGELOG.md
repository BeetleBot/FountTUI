# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2026-04-05

### Added
- **Home Screen**: New aesthetic main menu for quick access to recent scripts and help.
- **Production Reports**: Expanded Export UI with options for production-ready reports.
- **Shortcuts Mode**: Dedicated shortcuts status indicator in the UI.
- **Color-Coded Scene Navigator**: Scene headings now adopt the color of `[[color]]` markers placed anywhere in the scene.
- **Proper Installation Channels**: Added Windows MSI installer support and automated release workflows.

### Changed
- **Forced Physical Uppercase**: Scene headings, characters, and transitions are now permanently transformed to uppercase in the screenplay buffer for professional formatting.
- **Responsive UI Refactor**: Redesigned UI panels with mode-aware coloring and refreshed list layouts for better clarity and focus.
- **Improved Contrast**: Refined the theme detection and color mapping for a better experience across all terminals.
- **Streamlined Release Workflow**: Optimized CI/CD to focus on Windows MSI installers and Crates.io publication for higher reliability.

### Fixed
- **Robust Marker Detection**: Enhanced the scene parser to correctly identify marker colors even when separated by notes or empty lines.
- **Heading Cleaning**: Stripped metadata markers from scene navigator headings for a cleaner display.
- Resolved cursor misalignment in certain terminal environments.
- Fixed navigation issue where selection wouldn't update correctly when switching panes.

## [0.3.0] - 2026-04-04

### Added
- **Text Selection & Clipboard**: Implement text selection, system clipboard support (`Ctrl+C`, `Ctrl+X`, `Ctrl+V`).
- **Command Mode**: Implement a modernized command interface (`/`) with tab completion, migrating shortcut actions to command-based execution.
- **Format Pane**: Introduce FormatPane for document formatting and scene number management.
- **PDF Export**: Re-implement robust PDF screenplay export functionality.
- **Mouse Support**: Added 'Click with mouse' and 'Scroll with mouse' to the scene navigator.
- **Word Wrap**: Added word wrap for Scene headings.

### Changed
- Complete architecture refactoring of the source code (`app.rs` and inputs).
- **Zen Footer**: Modernized the status footer with a beautiful Powerline layout featuring colored edge blocks, transparent center blocks, soft `` separators, and live word/line counts.
- **Maximized Workspace**: Removed the redundant top title header to fully maximize vertical screen space for writing.
- **Light Mode Enhancements**: Removed hardcoded dark grey backgrounds in the footer so it natively adapts to light terminals, and softened the `Parenthetical` text styling using `Modifier::DIM` to prevent "washed out" colors.

## [0.2.0] - 2026-04-04

### Added
- **Zen Studio UI**: Complete aesthetic overhaul for a distraction-free, premium experience.
- **Unified Footer**: Consolidated status messages, real-time word/page counts, and shortcut hints into a single, clean bar.
- **Shortcuts Sidebar**: Interactive right-side pane (F1) for keybinding reference, keeping the main editor area uncluttered.
- **Vertical Pane Borders**: Added `│` separators for better visual pane isolation.
- **Selection Cursors**: Interactive `»` focus indicators in the Scene Navigator.
- **Adaptive Contrast**: Automated theme detection (Light/Dark) using `Modifier::DIM` and standard terminal colors (Color::Reset) instead of hardcoded white/black.

### Changed
- Refined typography and spacing across the TUI for a more "human" feel.
- Removed legacy `High Contrast` toggle in favor of the new adaptive system.

### Fixed
- Updated the test suite to align with the new consolidated layout and string labels.

## [0.1.3] - 2026-04-04

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
