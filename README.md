# Primarch.sh

A modern, workspace-aware terminal emulator built with [Tauri 2](https://tauri.app/) and [Vue 3](https://vuejs.org/).

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.x-orange.svg)

## Features

- **Split Panes** — Divide your terminal into multiple panes (horizontal/vertical splits) with directory inheritance
- **Workspace Management** — Save and restore complex terminal layouts with working directories and startup commands
- **Git Integration** — Built-in source control with staging, commits, diffs, branch switching, history, pull/push
- **AI Commit Messages** — Generate commit messages from staged diffs using Claude (Anthropic API)
- **Multiple Shells** — Support for PowerShell, CMD, WSL, Git Bash, zsh, and bash
- **Theme Support** — 20 built-in color schemes including light and dark themes
- **Accent Colors** — 8 accent color presets (Cyan, Blue, Purple, Pink, Red, Orange, Gold, Green)
- **Shell Integration** — Automatic CWD tracking via OSC 7 sequences for zsh, bash, and PowerShell

## Screenshots

<img width="1526" height="899" alt="image" src="https://github.com/user-attachments/assets/691e4337-b69e-4ea1-aa7e-3f224758608b" />

<img width="1526" height="899" alt="image" src="https://github.com/user-attachments/assets/bde9b1ba-1344-4ae8-87fd-a98e5c141c57" />

<img width="1526" height="899" alt="image" src="https://github.com/user-attachments/assets/077653ea-a041-4cd0-a004-28ca7c9c367b" />


## Installation

### Pre-built Binaries

Download the latest release from the [Releases](https://github.com/mythicor/primarch/releases) page.

| Platform | File | Notes |
|----------|------|-------|
| **Windows** | `.msi` or `.exe` | MSI for enterprise, EXE for personal |
| **macOS** | `.dmg` | Apple Silicon (M1/M2/M3/M4) |
| **Linux** | `.AppImage` or `.deb` | AppImage runs on most distros |

#### macOS Installation

The app is not signed with an Apple Developer certificate, so macOS will block it by default. To run it:

1. Download and open the `.dmg`
2. Drag **Primarch** to Applications
3. **First launch**: Right-click the app → **Open** → **Open** (bypasses Gatekeeper)

Alternatively: System Settings → Privacy & Security → scroll down and click **Open Anyway**.

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

#### Steps

```bash
# Clone the repository
git clone https://github.com/mythicor/primarch.git
cd primarch

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

Build outputs are located in `src-tauri/target/release/bundle/`.

## Usage

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+E` | Split pane horizontally (new pane below) |
| `Ctrl+Shift+D` | Split pane vertically (new pane to the right) |
| `Ctrl+Shift+W` | Close current pane |
| `Ctrl+Tab` | Focus next pane |
| `Ctrl+Shift+Tab` | Focus previous pane |
| `Ctrl+Shift+S` | Toggle workspace manager |
| `Ctrl+Shift+G` | Toggle git sidebar |
| `Ctrl+,` | Toggle settings |

### Git Integration

The git sidebar automatically detects repositories based on your terminal's current directory. Features include:

- **Changes View** — Stage, unstage, and discard file changes
- **AI Commit Messages** — Generate commit messages from your staged diff with one click
- **Commit** — Create commits with a message (or let AI write it)
- **Branch Switching** — Create, checkout, and delete branches
- **History** — Browse commit history with diff viewing
- **Pull / Push** — Separate pull and push with commit count indicators

### AI Commit Messages

1. Open **Settings** (`Ctrl+,`)
2. Paste your Anthropic API key in the **AI** section ([Get a key](https://console.anthropic.com/settings/keys))
3. Stage some changes in the git sidebar
4. Click the **AI** button above the commit message input

Uses Claude Haiku — costs fractions of a cent per message.

### Workspaces

Save your current terminal layout (pane arrangement, working directories, shells) as a workspace. Restore it later to pick up exactly where you left off.

## Tech Stack

- **Backend**: Tauri 2.x (Rust) with [portable-pty](https://crates.io/crates/portable-pty) for PTY management
- **Frontend**: Vue 3 + TypeScript + Vite
- **Terminal**: [@xterm/xterm](https://xtermjs.org/) v6
- **State**: [Pinia](https://pinia.vuejs.org/)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/) v4
- **Git**: [git2](https://crates.io/crates/git2) (libgit2 bindings)
- **AI**: [Anthropic API](https://docs.anthropic.com/) (Claude Haiku)

## Project Structure

```
primarch/
├── src/                    # Vue frontend
│   ├── components/         # Vue components
│   │   ├── terminal/       # Terminal pane components
│   │   ├── layout/         # Split pane layout system
│   │   ├── git/            # Git sidebar components
│   │   ├── workspace/      # Workspace management
│   │   └── settings/       # Settings panel
│   ├── stores/             # Pinia stores
│   ├── themes/             # Terminal color schemes
│   └── types/              # TypeScript type definitions
├── src-tauri/              # Rust backend
│   └── src/
│       ├── pty/            # PTY session management
│       ├── git/            # Git operations
│       └── workspace/      # Workspace persistence
└── ...
```

## Configuration

Settings are stored in localStorage and include:
- Terminal theme and accent color
- Font family and size
- Cursor style and blink
- Anthropic API key (for AI commit messages)

Workspaces are saved as JSON files in:
- **Windows**: `%APPDATA%/primarch/workspaces/`
- **macOS**: `~/Library/Application Support/primarch/workspaces/`

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) — Cross-platform desktop framework
- [xterm.js](https://xtermjs.org/) — Terminal emulator component
- [git2-rs](https://github.com/rust-lang/git2-rs) — Git integration
- [Anthropic](https://anthropic.com/) — AI commit message generation
