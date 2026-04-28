# Fount: Engineering Roadmap & Todo

## 🏛️ Architectural Improvements

### [ ] Modularize the `App` State (`src/app/mod.rs`)
The `App` struct is currently a monolithic state container. Refactor related fields into focused sub-structs:
- `EditorBuffer`: Core text, cursor, and undo/redo logic.
- `NavigatorState`: Scene and character navigation data.
- `SessionStats`: Sprints, goals, and time tracking.

### [ ] Refactor the Layout Engine (`src/layout.rs`)
> [!IMPORTANT]
> The `build_layout` function is currently a monolithic loop exceeding 400 lines. This complexity makes it difficult to debug and prevents easy extension.

**Goal**: Decompose `build_layout` into specialized sub-processors:
- **PaginationProcessor**: Manage page breaks and numbering.
- **DialogueProcessor**: Handle "Continued" extensions and dual-dialogue logic.
- **SceneProcessor**: Handle scene numbering and mirroring.
- **MarkupProcessor**: Mapping Fountain markers to visual formatting.

### [ ] Implement Incremental Parsing & Layout
Currently, Fount re-processes the entire document on most changes. This will cause lag in scripts over 100 pages.
- **Task**: Implement a "Dirty Region" tracking system to only re-parse and re-layout changed scenes or pages.

---

## ⚡ Performance & Code Quality

### [ ] Optimize Parser Throughput (`src/parser.rs`)
- **Issue**: Every line is converted to a `Vec<char>` for Unicode handling, which is memory-heavy for large files.
- **Refactor**: Use `unicode-segmentation` for grapheme-aware iteration instead of redundant vector allocations.

### [ ] Strengthen Error Handling
- **Issue**: Widespread use of `.unwrap()` or unchecked `Option` values in command execution and file I/O.
- **Task**: Introduce a custom `FountError` enum and migrate `src/app/methods/` to use the `Result` pattern systematically.

---

## 🎨 UX & Polish

- [ ] **Ghost Formatting Markers**: Instead of hiding markers like `*` or `_` completely, render them in a very low-contrast "dim" color to provide structural hints without distraction.
- [ ] **Pacing Heatmap (X-Ray)**: Add a visual "Dialogue vs. Action" heatmap to the X-Ray view to help writers visualize script rhythm.

---

## ⌨️ Workflow & Navigation

- [ ] **Multi-file Project Support**: Support for a `.fount` project file that aggregates multiple Fountain files (e.g., acts or episodes) into a single unified workspace.


---

## ✅ Completed
- [x] **Match Count**: Show `[X/Y]` in the status bar during search navigation.
- [x] **Navigation Shortcuts**: `Alt+Up` / `Alt+Down` for jumping between search matches.
- [x] **Buffer Tabs**: Minimal adaptive tab bar for multi-buffer workflows.
- [x] **Save Prompt**: Updated `/w` to prompt for filenames on unnamed buffers.
- [x] **Dirty Indicator**: Visual `*` in status bar when a buffer has unsaved changes.
- [x] **Sticky Headings**: Pin the current scene name to the top of the viewport or display it prominently in the footer during scrolling.
- [x] **Live Navigator Preview**: Scroll the editor background dynamically as the user moves through the Scene Navigator (`Ctrl+H`).