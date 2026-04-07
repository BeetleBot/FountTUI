# Modularization Plan (Detailed)

Our current files are way too big to manage. We're going to split them up into smaller, logical pieces so we can actually find things when we need to. Here is the exact plan for what goes where.

## Phase 1: Cleaning up the Logic (`src/app/methods/`)

We're moving almost everything out of the 2,200+ line `mod.rs` into specialized files. Each file will just be an `impl App { ... }` block.

### editing.rs
This covers everything that actually changes the text or manages history.
- `save_state` (History management)
- `undo` / `redo`
- `insert_char` 
- `insert_newline`
- `backspace` / `delete_forward`
- `delete_word_back` / `delete_word_forward`
- `delete_selection`
- `cut_line` / `paste_line`
- `handle_tab` (The element switcher)
- `clear_selection` / `selection_range` / `selected_text`
- `copy_to_clipboard` / `cut_to_clipboard` / `paste_from_clipboard`

### navigation.rs
Everything related to moving around the script.
- `move_up` / `move_down` / `move_left` / `move_right`
- `move_word_left` / `move_word_right`
- `move_home` / `move_end`
- `execute_search` / `update_search_regex`
- `jump_to_scene` / `jump_to_line`
- `report_cursor_position`
- `update_layout` (Ensures the view stays synced with the cursor)

### io.rs
Handles file system operations and exporting.
- `save` / `save_as`
- `load_file`
- `export_pdf`
- `export_fountain` / `export_scene_csv` / `export_character_csv`
- `emergency_save` (The panic handler)
- `set_status` / `clear_status` / `set_error`

### analysis.rs
Methods that calculate things about the script but don't change it.
- `parse_document` (The core parser runner)
- `total_word_count` / `total_page_count`
- `current_page_number`
- `open_scene_navigator` / `open_character_sidebar`
- `refresh_ensemble_list`
- `auto_number_locked_scenes` (Production lock logic)

---

## Phase 2: Input and UI

Once the core logic is moved, we'll tackle the 1,100-line `input.rs` and `ui.rs` files.

### input/ (src/app/input/)
We'll split the massive `handle_event` function into handlers for each mode:
- `normal.rs`: The main writing keys.
- `command.rs`: Logic for what happens when you type `/`.
- `navigation.rs`: Handlers for the Scene and Character sidebars.
- `panes.rs`: Handling interactions for Settings and Export panels.

### ui/ (src/app/ui/)
`ui.rs` will be broken down by visual component:
- `editor.rs`: Draws the actual screenplay lines.
- `sidebar.rs`: Draws the Scene/Character navigators.
- `footer.rs`: Draws the mode bar and status messages.
- `panes/`: A folder for Settings, Export, and Shortcuts UI.

---

## Phase 3: The Tests

Finally, we'll split the 3,200-line `tests.rs` so tests live near the logic they're testing.
- `editing.rs`: Validates text changes and undo/redo.
- `navigation.rs`: Tests movement and search logic.
- `ui.rs`: Layout and focus mode tests.
- `ux.rs`: Boundary cases, line joining, and UTF-8 safety.

---

## How we'll execute
We'll do this in small, safe steps. After each file move, we'll run `cargo check` to make sure we haven't broken any imports. Once a phase is done, we'll run the full `cargo test` suite to verify everything still works exactly as it did before.
