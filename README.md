# FOUNT

```text

   ███████╗ ██████╗ ██╗   ██╗███╗   ██╗████████╗
   ██╔════╝██╔═══██╗██║   ██║████╗  ██║╚══██╔══╝
   █████╗  ██║   ██║██║   ██║██╔██╗ ██║   ██║   
   ██╔══╝  ██║   ██║██║   ██║██║╚██╗██║   ██║   
   ██║     ╚██████╔╝╚██████╔╝██║ ╚████║   ██║   
   ╚═╝      ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   

```

**Fount** is a minimal, distraction-free Fountain screenplay editor built for writers who live in the terminal. It blends the raw efficiency of Rust with a "Zen Studio" aesthetic, providing a writing experience that feels both professional and deeply personal.

---

## 🎞️ The Zen Experience

Fount is a dedicated writing environment designed to disappear while you work.

- **Adaptive Interior**: Automatically detects your terminal's background. Fount respects your environment natively.
- **Unified Vision**: A single-line footer displays real-time **Word Counts**, **Page Numbers**, and **Status Messages**.
- **Story-Map Navigation**: Toggle the **Scene Navigator (`Ctrl+H`)** to see your screenplay's structure at a glance.
- **Ensemble Management**: Use the **Character Sidebar (`Ctrl+L`)** to analyze character distribution and jump directly to appearances.
- **Multi-Buffer Workflow**: Open, edit, and switch between multiple scripts seamlessy.

---

## 🚀 Quick Start

### Installation

**Linux & macOS (Single Command):**
```bash
curl -sSfL https://raw.githubusercontent.com/BeetleBot/Fount/main/scripts/install.sh | sh
```

**Via Cargo:**
```bash
cargo install fount
```

### Usage
```bash
fount [script.fountain]
```

---

## ⌨️ Essential Keyboard Shortcuts

Fount prioritizes a command-driven workflow. Only core editing and viewing shortcuts remain mapped to keys.

| Shortcut | Action |
| :--- | :--- |
| **`/`** | **Command Mode** (Open footer command bar) |
| **`F1`** | **Internal Reference** (Toggle help panel) |
| **`Ctrl + H`** | Open Scene Navigator |
| **`Ctrl + L`** | Open Ensemble Sidebar |
| **`Ctrl + P`** | Open Settings Pane |
| **`Ctrl + E`** | Open Export Pane |
| **`Ctrl + A`** | Select All Text |
| **`Ctrl + C`** | Copy Selection |
| **`Ctrl + X`** | Cut Selection (or current line) |
| **`Ctrl + V`** | Paste from Clipboard |
| **`Shift + Arr`**   | Select Text |
| **`Tab`**            | Smart Indent / Autocomplete |
| **`Enter`**          | Insert Newline |
| **`Esc`**            | Clear Status / Dismiss Sidebars |

---

## ⚡ Shorthand Commands (/)

Type `/` followed by a command to execute operations. Most physical shortcuts have been moved here for a cleaner experience.

| Command | Action |
| :--- | :--- |
| **`/w`** | Save Current Script |
| **`/ww`**| Save As (opens file picker) |
| **`/o [path]`** | Open script (opens picker if no path) |
| **`/ud`** | Undo last change |
| **`/rd`** | Redo last change |
| **`/bn`** | Buffer Next (Switch script) |
| **`/bp`** | Buffer Previous (Switch script) |
| **`/q`** | Close Current Script (Return to Home) |
| **`/wq`** | Save and Close Script |
| **`/q!`** | Force Close Script (Discard Changes) |
| **`/ex`** | **Exit Fount Completely** |
| **`/new`** | Create a New Empty Buffer |
| **`/home`** | Return to the Home Screen |
| **`/search [q]`** | Global Script Search |
| **`/[number]`** | Jump to Line Number |
| **`/s[number]`** | Jump to Scene Number |
| **`/renum`** | Auto-Renumber all scenes |
| **`/injectnum`** | Tag current scene with next available number |
| **`/pos`** | Report detailed cursor position |

---

## 🎨 Global Configuration (/set)

Tailor your environment in real-time. Use `/` as the trigger.

```
Usage : /set [option] [on/off]
Example : /set focus
```

| Option | Description |
| :--- | :--- |
| **`focus`** | Zen Mode: Hides all UI elements for pure writing. |
| **`typewriter`** | Strict Typewriter: Cursor stays centered vertically. |
| **`markup`** | Hidden Sigils: Hides Fountain syntax markers (like `**`). |
| **`pagenums`** | Live Page Counting in the footer. |
| **`scenenums`** | Toggles Scene Numbers in the margin. |
| **`contd`** | Automatically appends `(CONT'D)` to dialogue. |
| **`autosave`** | Background saves your draft every 30 seconds. |

---

## 🛠️ Troubleshooting

**macOS Colors Look Wrong?**
The default macOS `Terminal.app` lacks Truecolor (24-bit) support, which Fount uses for its themes. To fix this:
1. Use a modern terminal emulator (iTerm2, Alacritty, Kitty, Ghostty).
2. Add `export COLORTERM=truecolor` to your `~/.zshrc` or `~/.bashrc`.

---

## 🏛️ Credits

- **[Ratatui](https://ratatui.rs/)**: Powering the terminal interface.
- **[Fountain](https://fountain.io/)**: The simple, universal screenplay format.
- **[Rust](https://www.rust-lang.org/)**: Performance and safety.

---

*Crafted for the screenwriting community.*
