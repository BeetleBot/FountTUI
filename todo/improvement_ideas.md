# FountCLI Improvement Ideas

## Code Health

### 1. The `App` God Struct
`App` in `src/app/mod.rs` has **60+ public fields** and is 1344 lines long. Every feature bolts more state onto it. This makes it hard to reason about what state belongs to what feature.

**Suggestion**: Group related fields into sub-structs:
```rust
pub struct App {
    pub editor: EditorState,    // cursor, lines, scroll, undo/redo
    pub search: SearchState,    // query, regex, matches, highlight
    pub navigator: NavigatorState,
    pub xray: XRayState,
    pub sprint: SprintState,
    pub picker: Option<FilePickerState>,
    pub config: Config,
    pub theme: Theme,
    // ...
}
```
This doesn't change functionality — just groups fields logically. Touch one module at a time.

### 2. `lib.rs` Cleanup
`lib.rs` currently has 22 blank lines and 8 module declarations. It works, but it looks unfinished.

### 3. `swap_buffer` Fragility
The `swap_buffer` method does 17 individual `std::mem::swap` calls. If you ever add a field to `BufferState` and forget to add the corresponding swap, you get a silent bug. Consider deriving or implementing a trait that handles this automatically, or storing the active buffer state *inside* `BufferState` directly rather than mirroring it on `App`.

---

## Features

### 4. Homebrew Support (macOS)
You have AUR, Winget, Cargo, DEB, RPM — but no Homebrew tap. macOS screenwriters are a huge audience. A basic Homebrew formula is ~20 lines of Ruby and can be auto-updated via GitHub Actions.

### 5. CI Test Pipeline
Your `release.yml` builds and publishes, but there's no `ci.yml` that runs `cargo test` and `cargo clippy` on every PR/push. This is easy to add and catches regressions before they reach a release.

### 6. FinalDraft (.fdx) Import
Fountain is great, but many screenwriters receive notes and scripts in `.fdx` format. Even a basic read-only import would make Fount more practical for professional workflows. FDX is just XML under the hood.

### 7. Collaborative/Git-Aware Features
Since Fountain is plain text, it plays beautifully with Git. A `/diff` command that shows a side-by-side diff of the current file vs. the last commit would be a killer feature for revision tracking.

---

## Distribution & Packaging

### 8. Man Page
CLI tools should ship a man page. `clap` can auto-generate one via `clap_mangen`. Add it to your DEB/RPM assets.

### 9. Shell Completions
`clap` also supports generating shell completions for Bash, Zsh, Fish, and PowerShell via `clap_complete`. Ship these with your packages for a polished CLI experience.

### 10. Winget Automation
Manually submitting to Winget for each release is tedious. Tools like `wingetcreate` can be integrated into your GitHub Actions to auto-submit a PR to `microsoft/winget-pkgs` on every new release tag.

---

## Performance

### 11. `ui/mod.rs` is 1509 Lines
The main draw function is a single massive function. Extract the editor rendering, footer, header, and sidebar into separate functions. This improves compile times (smaller codegen units) and makes profiling easier.

### 12. Incremental Parsing
`parse_document()` is called on every text change and processes the entire document. For large scripts (120+ pages), consider incremental parsing — only re-parse the lines that changed and a small window around them.

---

## README & Discoverability

### 13. Windows Installation via Winget(Once u confirm that winget works)
Once your Winget PR merges, update the README to include:
```
winget install BeetleBot.Fount
```
This is the simplest install path for Windows users.

### 14. Screenshots / GIF Demo
Your README has no screenshots or demo GIF. A 10-second recording of the editor in action would dramatically increase interest. Tools like `vhs` (by Charm) can generate GIFs from a script.
