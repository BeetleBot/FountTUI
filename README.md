# Fount

A simple yet powerful terminal-based **Fountain screenplay editor** built with Rust.

**Fount** is designed to be a lightweight and efficient tool for screenwriters who prefer the terminal environment. It handles the [Fountain](https://fountain.io) markup language natively, providing a seamless writing experience with real-time formatting.

![Fount Screenshot](https://raw.githubusercontent.com/BeetleBot/Fount/main/assets/logo.svg)

## Features

- **Native Fountain Parsing**: Fully supports the Fountain spec, including title pages, scene headings, character cues, dialogue, parentheticals, and notes.
- **TUI Interface**: A clean, stateful terminal user interface built with `ratatui`.
- **Intelligent Formatting**: 
    - Automatic (CONT'D) appending for consecutive speakers.
    - Markdown support for **bold**, *italic*, and _underline_.
    - Hides markup syntax when the cursor is off-line for a distraction-free view.
- **Screenplay Navigation**: 
    - **Independent Scene Navigator (Alt+S)**: A dedicated sidebar to jump between scenes and view synopses.
    - **Settings Pane (Alt+P)**: Toggle features on the fly.
- **Export Options**: Export your scripts to plain text or ANSI formats for previewing.
- **Auto-Save**: Never lose progress with customizable auto-save intervals.
- **Search & Highlighting**: Incremental search with regex support.

## Installation

### Via Cargo (Recommended)

Ensure you have [Rust and Cargo](https://rustup.rs/) installed, then run:

```bash
cargo install fount
```

### From Source

```bash
git clone https://github.com/BeetleBot/Fount.git
cd Fount
cargo install --path .
```

## Usage

Start the editor by running:

```bash
fount [path/to/script.fountain]
```

### Keybinds

- **Alt+S**: Open Scene Navigator
- **Alt+P**: Open Settings Pane
- **Ctrl+S**: Save current buffer
- **Ctrl+Q**: Quit or close buffer
- **Alt+N / Alt+H**: Next/Previous buffer
- **Ctrl+K / Ctrl+U**: Cut/Paste whole lines

## Configuration

Place your `fount.conf` in `~/.config/fount/` to customize your experience. See the `fount.conf.example` file for all available options.

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.
