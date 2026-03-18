# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Mythicor Terminal is a workspace-aware terminal emulator for Windows built with Tauri 2.x and Vue 3. It allows users to save and restore complex split-pane terminal layouts with working directories and startup commands.

## Tech Stack

- **Backend**: Tauri 2.x (Rust) with portable-pty for PTY management
- **Frontend**: Vue 3 + TypeScript + Vite
- **Terminal**: @xterm/xterm v6 for terminal rendering
- **State**: Pinia
- **Styling**: Tailwind CSS v4
- **Icons**: @heroicons/vue

## Development Commands

```bash
# Install dependencies
npm install

# Run in development mode (starts both Vite and Tauri)
npm run tauri dev

# Build for production (creates MSI and NSIS installers)
npm run tauri build

# Build only the frontend
npm run build

# Type check
npx vue-tsc --noEmit
```

## Project Architecture

### Rust Backend (`src-tauri/`)

```
src-tauri/src/
├── main.rs           # Entry point
├── lib.rs            # Tauri commands and app setup
├── pty/
│   ├── mod.rs        # PtyManager - manages all terminal sessions
│   ├── session.rs    # TerminalSession - individual PTY instance
│   └── shells.rs     # Shell detection (PowerShell, CMD, WSL, Git Bash)
└── workspace/
    ├── mod.rs        # Workspace module exports
    ├── config.rs     # Workspace and LayoutNode types
    └── storage.rs    # JSON file persistence
```

**Terminal Tauri Commands:**
- `create_terminal(shell?, cwd?)` - Creates a new PTY session, returns session ID
- `write_terminal(session_id, data)` - Write to terminal stdin
- `resize_terminal(session_id, cols, rows)` - Resize terminal
- `kill_terminal(session_id)` - Kill terminal session
- `start_terminal_reader(session_id)` - Start reading from PTY and emit events
- `list_terminals()` - List all active terminal sessions
- `get_available_shells()` - Detect installed shells

**Workspace Tauri Commands:**
- `save_workspace_cmd(workspace)` - Save workspace to JSON file
- `load_workspace_cmd(id)` - Load workspace by ID
- `delete_workspace_cmd(id)` - Delete workspace
- `list_workspaces_cmd()` - List all saved workspaces

**Events emitted:**
- `terminal-data-{session_id}` - Terminal output
- `terminal-closed-{session_id}` - Terminal closed
- `terminal-error-{session_id}` - Terminal error

### Vue Frontend (`src/`)

```
src/
├── components/
│   ├── terminal/
│   │   ├── TerminalPane.vue    # xterm.js wrapper with theme support
│   │   └── ShellSelector.vue   # Shell selection dropdown
│   ├── layout/
│   │   ├── LayoutTree.ts       # Binary tree operations for split panes
│   │   ├── SplitPane.vue       # Resizable split container
│   │   └── PaneContainer.vue   # Recursive layout renderer
│   ├── workspace/
│   │   └── WorkspaceManager.vue # Save/load workspace UI
│   └── settings/
│       └── SettingsPanel.vue   # Theme and font settings
├── composables/
│   └── useTerminal.ts          # Terminal IPC composable
├── stores/
│   ├── layout.ts               # Layout tree state (Pinia)
│   ├── workspace.ts            # Workspace management (Pinia)
│   └── settings.ts             # Settings with localStorage (Pinia)
├── themes/
│   └── presets.ts              # 8 color scheme presets
├── types/
│   └── index.ts                # TypeScript interfaces
├── assets/
│   └── main.css                # Tailwind imports
├── App.vue                     # Main app with modals
└── main.ts
```

**Key Interfaces (`src/types/index.ts`):**
- `TerminalSession` - Terminal session data
- `LayoutNode` - Binary tree node (split or terminal)
- `Workspace` - Saved workspace configuration
- `ShellProfile` - Shell configuration (command, args, env)
- `Theme` - Terminal color scheme (16 ANSI colors + special)

**Keyboard Shortcuts:**
- `Ctrl+Shift+D` - Split vertical
- `Ctrl+Shift+E` - Split horizontal
- `Ctrl+Shift+W` - Close pane
- `Ctrl+Tab` - Focus next pane
- `Ctrl+Shift+Tab` - Focus previous pane
- `Ctrl+Shift+S` - Toggle workspace manager
- `Ctrl+,` - Toggle settings

**Available Themes:**
- Dracula (default)
- One Dark
- Monokai
- Nord
- Solarized Dark
- GitHub Dark
- Gruvbox Dark
- Tokyo Night

## Layout System

The layout uses a binary tree structure where each node is either:
- **Split node**: Has direction (horizontal/vertical), ratio (0-1), and two children
- **Terminal node**: Has shell, cwd, startupCommand, and terminal reference

Layout operations in `LayoutTree.ts`:
- `splitNode()` - Split a terminal into two panes
- `removeNode()` - Remove a terminal and collapse parent
- `findNode()` - Find node by ID
- `getAllTerminalIds()` - Get all terminal node IDs

## Data Storage

Workspaces are saved as JSON files in the user's config directory:
- Windows: `%APPDATA%/com.mythicor.terminal/workspaces/`

Settings are stored in localStorage under the key `mythicor-terminal-settings`.

## Build Output

Production builds create installers at:
- MSI: `src-tauri/target/release/bundle/msi/`
- NSIS: `src-tauri/target/release/bundle/nsis/`

## Key Files to Modify

When implementing new features:

- **New Tauri commands**: `src-tauri/src/lib.rs`
- **PTY functionality**: `src-tauri/src/pty/`
- **Workspace persistence**: `src-tauri/src/workspace/`
- **Terminal UI**: `src/components/terminal/`
- **Layout system**: `src/components/layout/`
- **Workspace UI**: `src/components/workspace/`
- **Settings UI**: `src/components/settings/`
- **State management**: `src/stores/`
- **Themes**: `src/themes/presets.ts`
- **Type definitions**: `src/types/index.ts`
