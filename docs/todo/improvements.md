# Fount Polish & Improvements
- [ ] **Search "Match Count"**: When using `/search`, show `Match X of Y` in the footer status bar to provide better spatial awareness during searches.

- [ ] **Smooth Home Screen**: Enhance the Home screen with more professional dashboard elements:
    - [ ] Add "last edited" timestamps to recent files.
    - [ ] Show total word counts for each recent file.
    - [ ] Add a "Quick Start" section for tutorials or recent snapshots.

- [ ] **Live Preview in Navigator**: As the user scrolls through the Scene Navigator (`Ctrl+H`), the editor should dynamically scroll to those scenes in the background.
- [ ] **Sticky Scene Headings**: Display the current scene name in the footer or "stick" the heading to the top of the editor during scrolling.

## Completed
- [x] **Buffer Tabs**: Minimal adaptive tab bar for multi-buffer workflows. Correctly handles adaptive themes and contrast.
- [x] **Save Prompt for New Buffers**: Updates `/w` to prompt for a filename using the file picker if the current buffer is unnamed.
- [x] **Dirty Status Indicator**: Add a subtle `*` or icon next to the filename in the status bar when the buffer has unsaved changes.