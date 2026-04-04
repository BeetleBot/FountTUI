# Fount - Development To-Do List

## Pending Tasks
- **Main Home Page**:
    - Implement a landing page shown when no file is provided at startup.
    - Options: New .fountain, Open .fountain, Tutorial, Exit.
- **Internal Text Selection**: Restore/Implement the ability to select text within the editor while mouse capture is active.
- **Mouse Selection Jump**: Enable clicking on a specific line in the Scene Navigator to jump there.
- **Settings Persistence**: Ensure settings changed in the pane are saved back to the config file.


## Completed Tasks
- **App Renaming**: Completely renamed to **Fount**. Terminal command is now `fount`.
- **Independent Scene Navigator (Alt+S)**
    - Sidebar implementation (left side, 45 columns).
    - Lists all scenes with Scene Number and Heading (in CAPS).
    - Includes word-wrapped Synopses (`=`) under each scene.
    - Inherits marker colors (e.g., `[[red]]`) from headings or subsequent lines.
    - Clear segregation between scenes.
    - Jump to scene on `Enter`.
    - Auto-closes when the Settings Pane opens.
    - Header updates to show "SCENE NAVIGATOR" when active.
    - Fixed scrolling issue using stateful list management.
- **Auto-Save Feature**
    - Automatically saves modified buffers.
    - Default interval: 30 seconds.
    - Configurable in `fount.conf`.
- **Settings Pane (Alt+P)**
    - Sidebar implementation (right side).
    - Toggle on-the-fly settings.
    - Help system: `[?]` icons with mouse-click support and keyboard shortcut (`?` or `h`).
    - Cleaned up options: Removed basic Typewriter, Match Parentheses, and Close Elements (moved to permanent features).
    - Header updates to show "SETTINGS" when active.
- **Configuration Defaults**
    - `Strict Typewriter Mode` is now enabled by default.
    - `Match Parentheses` and `Close Elements` are permanently enabled.
- **Editor Improvements**
    - Removed `Highlight Action` feature.
    - Enhanced `Hide Markup` to respect Fountain sigils (forced headings, lyrics, etc.).
    - Multi-line Paste support via Bracketed Paste mode.

