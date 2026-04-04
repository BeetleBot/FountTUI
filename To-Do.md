# Fount: To-Do

### In the Works
- [ ] **Release Automation (GitHub Actions)**: Get `.msi`/`.exe` and `.dmg` installers ready for future releases. Tentative timing. 
- [ ] **Home Page**: A simple landing area when opening without a file. (New, Open, Tutorial, Exit).
- [ ] **PDF Export**: The big one—implementing a clean PDF generator.
- [ ] **Refactoring app.rs**: It's getting huge (~197KB). Break it down into logic and UI modules for better maintenance.
- [ ] **Status Bar Overhaul**: 
    - Move shortcut legend to the side bar.
    - Add real-time stats (Word count, Page count, etc.) to the bar.
- [ ] **Structure & Navigation**: 
    - Way to implement structures into the screenplay.
    - Making those structures visible in the Scene Navigator.
- [ ] **Visual Selection**: Restore text selection within the editor when mouse capture is active.
- [ ] **Navigator Clicking**: Enable clicking on a line in the Scene Navigator to jump there.
- [ ] **Settings Fix**: Ensure settings changed in the pane actually save back to the config file.
- [ ] **Dynamic Contrast**: Implement an "Adaptive" approach where we use terminal default colors (like `Reset`) more intelligently to blend across themes naturally.


### Already Done
- [x] **App Identity**: Completely renamed to **Fount**. Terminal command is now `fount`.
- [x] **Scene Navigator (Alt+S)**:
    - Sidebar with scene numbers, headings, and synopses.
    - Respects marker colors (like `[[red]]`).
    - Smart scroll management and "Enter" to jump.
    - Auto-hides when Settings opens to keep things clean.
- [x] **Auto-Save**: Modified buffers now save every 30s by default (configurable).
- [x] **Settings Pane (Alt+P)**:
    - Right-side sidebar for on-the-fly toggles.
    - Clickable help `[?]` icons for quick info.
    - Cleaned up the list; moved essentials like `Match Parentheses` to permanent features.
- [x] **Better Defaults**: `Strict Typewriter Mode` is now on by default along with core editor helpers.
- [x] **Editor Polish**:
    - `Hide Markup` now respects Fountain sigils (headings, lyrics, etc.).
    - Proper support for pasting multiple lines of text.
    - Removed redundant 'Highlight Action' feature.
