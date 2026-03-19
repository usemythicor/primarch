# Mythicor Terminal

A modern, workspace-aware terminal emulator for Windows built with [Tauri 2](https://tauri.app/) and [Vue 3](https://vuejs.org/).

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.x-orange.svg)

## Features

- **Split Panes** - Divide your terminal into multiple panes (horizontal/vertical splits)
- **Workspace Management** - Save and restore complex terminal layouts with working directories and startup commands
- **Git Integration** - Built-in source control with staging, commits, diffs, branch switching, and history
- **Multiple Shells** - Support for PowerShell, CMD, WSL, and Git Bash
- **Theme Support** - 8 built-in color schemes (Dracula, One Dark, Monokai, Nord, and more)
- **Customizable** - Configurable fonts, font sizes, and terminal themes

## Screenshots

*Coming soon*

## Installation

### Pre-built Binaries

Download the latest release from the [Releases](https://github.com/mythicor/terminal/releases) page.

Available formats:
- **MSI** - Windows Installer package
- **NSIS** - Executable installer

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

#### Steps

```bash
# Clone the repository
git clone https://github.com/mythicor/terminal.git
cd terminal

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
| `Ctrl+Shift+D` | Split pane vertically |
| `Ctrl+Shift+E` | Split pane horizontally |
| `Ctrl+Shift+W` | Close current pane |
| `Ctrl+Tab` | Focus next pane |
| `Ctrl+Shift+Tab` | Focus previous pane |
| `Ctrl+Shift+S` | Toggle workspace manager |
| `Ctrl+,` | Toggle settings |

### Git Integration

The git sidebar automatically detects repositories based on your terminal's current directory. Features include:

- **Changes View** - Stage, unstage, and discard file changes
- **Commit** - Create commits with a message
- **Branch Switching** - Create, checkout, and delete branches
- **History** - Browse commit history with diff viewing
- **Remote Operations** - Fetch, pull, and push

### Workspaces

Save your current terminal layout (pane arrangement, working directories, shells) as a workspace. Restore it later to pick up exactly where you left off.

## Tech Stack

- **Backend**: Tauri 2.x (Rust) with [portable-pty](https://crates.io/crates/portable-pty) for PTY management
- **Frontend**: Vue 3 + TypeScript + Vite
- **Terminal**: [@xterm/xterm](https://xtermjs.org/) v6
- **State**: [Pinia](https://pinia.vuejs.org/)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/) v4
- **Git**: [git2](https://crates.io/crates/git2) (libgit2 bindings)

## Project Structure

```
mythicor-terminal/
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
- Terminal theme selection
- Font family and size
- Default shell

Workspaces are saved as JSON files in:
- **Windows**: `%APPDATA%/com.mythicor.terminal/workspaces/`

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - For the amazing cross-platform framework
- [xterm.js](https://xtermjs.org/) - For the terminal emulator component
- [git2-rs](https://github.com/rust-lang/git2-rs) - For git integration
