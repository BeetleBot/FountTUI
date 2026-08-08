<div align="center">

  <img src="assets/icons/FountTUI_Logo.png" alt="Fount Logo" width="160">

  # FOUNT
  ### *Blockbusters in Terminal*

  <p align="center">
    <a href="https://fount.iyal.ink">
      <img src="https://img.shields.io/badge/Official%20Website-fount.iyal.ink-7aa2f7?style=for-the-badge&logo=googlechrome&logoColor=white" alt="Official Website" />
    </a>
    &nbsp;
    <a href="https://github.com/iyal-ink/FountTUI/releases">
      <img src="https://img.shields.io/badge/Download%20Fount-v0.11.3-7aa2f7?style=for-the-badge&logo=linux&logoColor=white" alt="Download Fount" />
    </a>
    &nbsp;
    <a href="https://actone.iyal.ink">
      <img src="https://img.shields.io/badge/Try%20ActOne%20GUI-actone.iyal.ink-bb9af7?style=for-the-badge&logo=windows&logoColor=white" alt="Try ActOne Screenplay" />
    </a>
  </p>

  <p align="center">
    <b>A minimal, distraction-free Fountain screenplay editor for screenwriters who live in the terminal.</b>
    <br>
    <i>Built with Rust & Ratatui • Engineered by <a href="https://iyal.ink">iyal.ink</a></i>
  </p>

</div>

---

## 🌟 Overview

**Fount** brings the pure focus of Zen-mode screenwriting into your terminal. Powered by the open plain-text [Fountain standard](https://fountain.io), Fount provides dynamic screenplay formatting, structural beat outlines, character stats, interactive scene trees, index cards, and Final Draft-standard PDF export—all without touching a mouse.

> 🎬 **Prefer a GUI Application?**  
> Check out **[ActOne Screenplay](https://actone.iyal.ink)**—our feature-rich desktop screenplay editor for Windows & Linux!

---

## ⚡ Quick Start & Installation

### 🐧 Linux (Recommended)
Download the latest pre-built package from our **[Releases](https://github.com/iyal-ink/FountTUI/releases)**:

```bash
tar -xzf Fount-Linux-x64-*.tar.gz
sudo ./install.sh
fount
```

### 💻 Cross-Platform via Cargo
```bash
cargo install fount
```

### 🪟 Windows (Microsoft Store)
<a href="https://apps.microsoft.com/detail/9nz3hv7n30s2?hl=en-US&gl=IN">
  <img src="https://get.microsoft.com/images/en-us%20dark.svg" width="200" alt="Get it from Microsoft Store"/>
</a>

### 🍎 macOS
```bash
cargo install fount
```
> *Tip: For optimal rendering and true-color themes, use modern terminals like Ghostty, iTerm2, or WezTerm.*

---

## ✨ Features at a Glance

| Feature | Description |
| :--- | :--- |
| ⚡ **Live Fountain Engine** | Auto-formats Scene Headings, Character Cues, Dialogue, Parentheticals & Transitions on the fly. |
| 🌲 **Scene Tree Sidebar** | Instant overview and keyboard navigation across sequences, scenes, and acts. |
| 🃏 **Index Cards View** | Grid-based story architect to plot synopses, beats, and structure visually. |
| 📊 **X-Ray Studio** | Real-time analytics for character interactions, scene lengths, and narrative pacing. |
| 📄 **Final Draft PDF Engine** | Industry-standard 100% exact pagination, dynamic right margins, and A4 / US Letter layout export. |
| ⏱️ **Session Auto-Snapshots** | Background auto-saving system preventing any data loss while you write. |
| 🎨 **Curated Themes** | Switch instantly between *Lilac*, *Catppuccin*, *Nord*, *Everforest*, and *Paper*. |

---

## 🎬 Visual Walkthrough

### 🏠 Homescreen Dashboard
<p align="center">
  <a href="https://asciinema.org/a/1076515">
    <img src="https://asciinema.org/a/1076515.svg" width="850" alt="Homescreen Dashboard">
  </a>
</p>

### 📝 Live Syntax & Markup
<p align="center">
  <a href="https://asciinema.org/a/1076522">
    <img src="https://asciinema.org/a/1076522.svg" width="850" alt="Syntax Markup">
  </a>
</p>

### 🌲 Scene Tree Navigator
<p align="center">
  <a href="https://asciinema.org/a/1076518">
    <img src="https://asciinema.org/a/1076518.svg" width="850" alt="Scene Tree Navigator">
  </a>
</p>

### 🃏 Index Cards & Story Architect
<p align="center">
  <a href="https://asciinema.org/a/1076517">
    <img src="https://asciinema.org/a/1076517.svg" width="850" alt="Index Cards">
  </a>
</p>

### 📊 X-Ray Analytics & Pacing
<p align="center">
  <a href="https://asciinema.org/a/1076524">
    <img src="https://asciinema.org/a/1076524.svg" width="850" alt="X-Ray Analytics">
  </a>
</p>

---

## ✍️ A Note from the Creator

> [!NOTE]
> **A Letter from the Creator**
> 
> As a credited Tamil/Indian screenwriter—writing predominantly in **English and Tanglish**—I found myself at a crossroads when I transitioned to Linux. I deeply missed **[Beat](https://github.com/lmparppei/Beat)**, my long-time companion for storytelling, and couldn't find a minimalist alternative that felt "right" in the terminal.
> 
> My search led me to **[Lottie](https://github.com/coignard/lottie)**, whose elegance immediately captivated me. I cloned the project and began shaping it into the tool I needed. While I possess a moderate grasp of Rust, this journey was significantly smoothed by the partnership of **AI Agents like Claude and Gemini**. They were instrumental in helping me overcome technical hurdles, complicated logic, and the often frustrating nuances of software release workflows. Fount is the result of my creative vision and writing background, the open-source code that inspired me, and the intelligence of the agents that helped me build it. It is a tool I use daily, and I hope it serves you just as well.

> [!IMPORTANT]
> **A Note on Pagination**
> Due to TUI constraints, page counts shown in the editor are approximate (+/- 2 pages). The final PDF export contains the true, definitive pagination matching Final Draft standards.

---

## 🏛️ Credits & Inspiration

Fount stands proudly on the shoulders of open-source projects:

1. **[Lottie](https://github.com/coignard/lottie)** — Foundational terminal editor that inspired Fount's codebase.
2. **[Beat](https://github.com/lmparppei/Beat)** — The benchmark for minimalist screenwriting software.
3. **[Fountain.io](https://fountain.io)** — The open plain-text format powering modern independent cinema.

---

<div align="center">

  <br>

  <b>FountTUI is engineered with care by <a href="https://iyal.ink">iyal.ink</a></b>  
  <i>Crafting free, open, and focused storytelling tools for independent screenwriters worldwide.</i>

  <br><br>

  🌐 <b><a href="https://iyal.ink">iyal.ink</a></b> &nbsp;•&nbsp; 🎬 <b><a href="https://actone.iyal.ink">ActOne Screenplay</a></b> &nbsp;•&nbsp; ⌨️ <b><a href="https://fount.iyal.ink">FountTUI</a></b>

</div>
