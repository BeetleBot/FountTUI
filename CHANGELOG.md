# Changelog


## [0.8.1] - 2026-04-18

### Changed
- **Windows Distribution**: Removed pre-built Windows binaries and MSI installer from releases due to Smart App Control blocking unsigned apps. Windows users should install via `cargo install fount`.

### Added
- **Zen Studio Polish**:
    - **Rotating Quotes**: The home screen now features inspiring quotes from legendary screenwriters.
    - **Saved Indicator**: A subtle visual confirmation (✓ Saved) now appears in the status bar after successful saves.

## [0.7.8] - 2026-04-18

### Added
- **Windows MSI Installer**: Professional installer for Windows with Start Menu integration and desktop shortcuts.
- **Embedded App Logo**: The executable now features the Fount branding in File Explorer and the Taskbar.

### Fixed
- **Windows Icon Rendering**: Improved terminal icon compatibility with automatic fallback for standard Windows terminals (Nerd Font vs. Unicode).

## [0.7.6] - 2026-04-18

### Fixed
- **Windows Startup**: Resolved a startup crash on legacy Windows terminals and improved release naming conventions.

## [0.7.5] - 2026-04-18

### Added
- **X-Ray Script Analysis**: A comprehensive new diagnostic dashboard (`/xray`) with triple-tabbed pages for Dialogue Balance (bar charts), Pacing Heatmaps (action vs dialogue density), and Scene Length tracking with "TOO LONG" warnings.
- **Enhanced Character Analytics**: Implemented robust mapping to merge character variants (e.g. `(V.O.)`, `(CONT'D)`) into single entities across all sidebars and reports.
- **Focus Mode Header Hiding**: Focus Mode now automatically collapses the top header and tab bar along with the status bar for a zero-distraction writing environment.
- **Platform-Specific READMEs**: Individual `PORTABLE.txt` files for Windows, Linux, and macOS now bundle with each release in `docs/readme/`.

### Changed
- **Home Screen UI Overhaul**: Completely redesigned the dashboard with a premium glassmorphism aesthetic, featuring curated "Quick Start" shortcuts for tutorials and recent scripts.
- **Widened Analysis Modal**: Increased the X-Ray modal width to 100 columns and added Dialogue Line counts (`L`) for more granular prominence tracking.

### Fixed
- **Name Normalization**: Fixed a long-standing bug where parenthetical character extensions caused duplicated entries in the Ensemble Sidebar (`Ctrl+L`).

## [0.7.0] - 2026-04-17

### Added
- **Refined Buffer Tabs**: Implementation of a minimalist, adaptive tab bar that appears only when multiple scripts are open. Features high-contrast dynamic foreground detection and dirty state indicators.
- **Buffer Switch Shortcuts**: New navigation keys for multi-buffer workflows: `Ctrl+PageUp` (Previous) and `Ctrl+PageDown` (Next).
- **Interactive Save Prompt**: `/w` command now dynamically triggers the file picker when saving unnamed buffers, providing a seamless "Save As" experience.

### Changed
- **Streamlined Distribution**: Removed Windows MSI installers in favor of standardized, portable tarballs across all platforms.
- **UI Aesthetic Polish**: Refined the tab bar with ` | ` separators, vibrant primary-colored delimiters, and subtle horizontal padding for a premium feel.
- **Workflow Automation**: Simplified GitHub Actions release cycle by removing redundant platform-specific README documents.

### Fixed
- **Active Tab Contrast**: Resolved visibility issues in the "Adaptive" theme by using unified theme selection colors for the active tab.

## [0.6.0] - 2026-04-08

### Added
- **Sprint Tracking**: Professional timed writing goals with real-time status bar progress, persistent history, and CSV export functionality.
- **Advanced Session Snapshots**: Redesigned session recovery with a table-based UI and dual restoration modes (Replace current or Open in new buffer).
- **Theme Management System**: High-performance theme engine with dynamic swapping and persistent user themes support.
- **Production Mode Scene Locking**: Industry-standard scene numbering system with auto-incrementing suffixes and production-safe locks.
- **Enhanced Command Help**: Completely redesigned command pane with standardized `/` prefixes, clear categorization, and "one command per line" layout.

### Changed
- **Command Prefix Standardization**: Unified all Ex-style commands under the `/` prefix for consistent interaction.
- **UI Architecture**: Refined the modal rendering stack to ensure consistent visibility across all application modes.

### Fixed
- **Snapshot State Handling**: Resolved type mismatch issue when navigating the upgraded snapshot table.
- **Sprint Export Workflow**: Replaced legacy menu-based export with an interactive TUI file picker for custom save locations.

## [0.5.0] - 2026-04-06

### Added
- **TUI File Picker**: Native, high-performance file selection with zero GUI dependencies, featuring scrolling support and home directory defaults.
- **Structural Thread Navigation**: Completely refined scene navigator with new **Structural Thread** support, integrating Sections, Scenes, and Synopses for advanced script organization.
- **Character Reports Pane**: Dedicated new pane for comprehensive character analysis and reporting.
- **Interactive Tutorial**: Dedicated tutorial mode with rewritten documentation using the new engine features.

### Changed
- **Command-First UI**: Transitioned the interface to be primarily command-based. Basic functions and pane-opening shortcuts are preserved, while most secondary actions have migrated to the command bar.
- **Robust Error Handling**: Significantly improved application stability by replacing unsafe `.unwrap()` calls with proper error propagation and recovery logic.
- **Home Screen Refinement**: Refined navigation, buffer management, and active UI prompt handling.
- **Build Optimization**: Smaller binary size and faster compilation through `rfd` removal and refined release profiles.

### Fixed
- **Buffer Protection**: Enhanced protection mechanisms to prevent accidental document loss during complex edits.
- Removed unused `NavigatorItem` and other redundant imports.
- Fixed all unit and doc-tests for the updated engine.

## [0.4.3] - 2026-04-05

### Fixed
- **Windows MSI Packaging**: Fixed GitHub Actions release workflow failing to build the MSI installer due to hardcoded executable paths.
- **Dynamic MSI Naming**: The resulting Windows installer now includes the release version in its filename (e.g., `Fount_Windows_v0.4.3.msi`).

## [0.4.2] - 2026-04-05

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
