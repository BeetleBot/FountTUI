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

- **Adaptive Interior**: Automatically detects your terminal's background. No more squinting at light themes or wrestling with dark modes—Fount respects your environment natively.
- **Unified Vision**: A single-line footer displays real-time **Word Counts**, **Page Numbers**, and **Status Messages** without cluttering your screen.
- **Story-Map Navigation**: Toggle the **Scene Navigator (`Ctrl+H`)** to see your screenplay's structure at a glance, complete with synopses and interactive focus indicators.
- **Contextual Shortcuts**: Keep your mind on the page. Use **`F1`** to reveal a clean sidebar of keybindings only when you need them.

## ✨ Features

- **Strict Typewriter Mode**: Keep your focus centered on the active line, just like a physical typewriter.
- **Smart Formatting**: Automatic `(CONT'D)` cues, Markdown support for visual emphasis, and "Hide Markup" logic for a clean preview of your prose.
- **Multi-Buffer Workflow**: Open, edit, and switch between multiple scripts in a single session.
- **Safe-by-Design**: Integrated **Auto-Save** and **Emergency Recovery** logic to ensure your draft is never lost to a crash or power failure.
- **Portable Linux Binary**: Statically linked for universal compatibility across any distribution.

---

## 🚀 Quick Start

### Installation

**Windows (Recommended):**
Download and run the latest **[Fount Windows Installer (.msi)](https://github.com/BeetleBot/Fount/releases/latest)**. This will automatically add `fount` to your system PATH.

**Linux & macOS (Single Command):**
```bash
curl -sSfL https://raw.githubusercontent.com/BeetleBot/Fount/main/scripts/install.sh | sh
```

**Via Cargo (Recommended for Developers):**
```bash
cargo install fount
```

**From Source:**
```bash
git clone https://github.com/BeetleBot/Fount.git
cd Fount
cargo install --path .
```

### Usage
```bash
fount [script.fountain]
```

---

## ⌨️ Essential Keybinds

| Key | Action |
| :--- | :--- |
| **`Ctrl + H`** | Open Scene Navigator |
| **`Ctrl + P`** | Open Settings Pane |
| **`F1`** | Toggle Shortcuts Legend |
| **`Ctrl + S`** | Save Current Script |
| **`Ctrl + X`** | Close Buffer / Exit |
| **`Ctrl + W`** | Search (Regex Support) |
| **`Ctrl + K`** | Cut Line |
| **`Ctrl + U`** | Paste Line |

---

## 🎨 Configuration

Tailor Fount to your process by editing `~/.config/fount/fount.conf`. 

> [!TIP]
> See `fount.conf.example` in the repository for a complete list of "Zen" options, including `Strict Typewriter Mode` and `Auto-Save` intervals.

## 🏛️ Credits

Fount is a labor of love, built on the shoulders of giants in the terminal and screenwriting communities.

- **[Lottie](https://github.com/coignard/lottie)**: The original project by [Thibault Coignard](https://github.com/coignard), which provided the solid foundation for Fount.
- **[Ratatui](https://ratatui.rs/)**: The powerful library that powers our terminal interface.
- **[Fountain](https://fountain.io/)**: The simple, universal screenplay format created by **John August** and **Ninian Lowe**.
- **[Rust](https://www.rust-lang.org/)**: For the performance and safety that keeps your drafts secure.

## ⚖️ Heritage & License

Fount is built on top of [Lottie](https://github.com/coignard/lottie). It is proudly open-source and licensed under the **GPL-3.0 License**.

---

*Crafted with <3 for the screenwriting community.*
