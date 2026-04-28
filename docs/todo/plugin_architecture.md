# Plugin System Architecture

This document outlines the strategy for introducing a scripting engine to FountCLI. The primary goal is to allow users to extend the editor's functionality without requiring modifications to the core Rust codebase.

## Objective

We want to transform FountCLI into a platform for screenwriters who need custom tools—be it specialized export formats, advanced scene analysis, or automated workflow triggers. By integrating a scripting language, we can provide a safe sandbox for these extensions.

## Language Selection: Lua 5.4

We have chosen Lua as the scripting language for several reasons:
1. It is extremely lightweight and fast, adding minimal overhead to the TUI.
2. It has a proven track record in the TUI world (Neovim, WezTerm).
3. The mlua crate provides excellent, safe bindings between Rust and Lua.

## Core Components

### 1. The Plugin Manager
A central struct within the App that manages the Lua VM. It will be responsible for initializing the environment, loading scripts from the user's configuration directory, and managing global state.

### 2. The Bridge API
We will expose a global `fount` object to the Lua environment. This object will contain methods to interact with the editor.

Proposed API Surface:
- `fount.status(message)`: Display a message in the status bar.
- `fount.insert(text)`: Insert text at the current cursor position.
- `fount.get_cursor()`: Returns the current line and column.
- `fount.get_line(index)`: Returns the text of a specific line.
- `fount.get_line_type(index)`: Returns the screenplay element type for a line (e.g., "SceneHeading", "Action", "Character").
- `fount.get_buffer()`: Returns the entire current buffer as a table of strings.
- `fount.get_elements()`: Returns an array of objects containing the text and type for every line in the script.
- `fount.command(name, callback)`: Register a new command that can be called via the slash palette.

### 3. Event System
Plugins need to react to editor state changes. We will implement a hook system for:
- `on_init`: Called when the plugin is first loaded.
- `on_save`: Called after a successful file save.
- `on_change`: Called whenever the text buffer is modified.
- `on_key`: A low-level hook for intercepting specific key combinations.

## Implementation Roadmap

### Phase 1: Infrastructure
The initial step involves adding the mlua dependency and setting up the PluginManager. We will focus on the loading logic, ensuring that Fount looks for scripts in `~/.config/fount/plugins/` (or the equivalent XDG directory) during startup.

### Phase 2: State Exposure
We will begin mapping internal App methods to Lua. This involves creating "userdata" or wrapper functions that safely access the active BufferState. This phase is complete when a Lua script can successfully print "Hello World" to the Fount status bar.

### Phase 3: Command Integration
We will modify the command palette logic to check the PluginManager for registered commands. This allows plugins to feel like native parts of the editor.

### Phase 4: Lifecycle and Documentation
The final phase involves hardening the system against script errors and providing a comprehensive guide for users to write their own plugins. We will include a few "standard" plugins as examples, such as a word-count tracker or a simple "Scene Title" auto-formatter.

## Security and Performance

While Lua is safe, we must ensure that scripts cannot hang the editor. We will implement basic timeouts for script execution and use Lua's internal error handling to catch crashes without taking down the entire TUI. For performance, plugins will only be executed on the main thread during specific event triggers to avoid slowing down the render loop.
