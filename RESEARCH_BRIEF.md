# Primarch Terminal - Research Brief

## Mission Statement

Primarch Terminal is a workspace-aware terminal emulator designed to be the optimal environment for AI-assisted software development. Our goal is to create a terminal that understands context, preserves workflow state, and enhances the experience of working with AI coding assistants like Claude Code, GitHub Copilot CLI, and similar tools.

## What We're Building

### Core Product
A modern terminal emulator for Windows (with macOS support) built with:
- **Tauri 2.x** (Rust backend) for native performance and small binary size
- **Vue 3 + TypeScript** frontend for reactive UI
- **xterm.js** for terminal emulation
- **Portable-pty** for cross-platform PTY management

### Key Differentiators

1. **Workspace Persistence** - Save and restore complex multi-pane terminal layouts with:
   - Working directories per pane
   - Startup commands
   - Split configurations (horizontal/vertical with custom ratios)
   - Shell selection per pane

2. **AI-Native Features**
   - Inline markdown-to-ANSI rendering (like Glow) for AI CLI output
   - Auto-detection of AI tools (Claude, Codex, etc.)
   - Optimized for streaming AI responses

3. **Developer Experience**
   - 20+ built-in color themes (Dracula, Nord, Tokyo Night, etc.)
   - Customizable accent colors
   - Command palette (Ctrl+P)
   - Alias system with `!aliasname` expansion
   - Directory browser integration

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Tauri Window                            │
├─────────────────────────────────────────────────────────────┤
│  Vue 3 Frontend                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  App.vue (Keyboard shortcuts, modals, layout)           ││
│  │  ├── CommandPalette.vue                                 ││
│  │  ├── WorkspaceManager.vue                               ││
│  │  ├── SettingsPanel.vue                                  ││
│  │  └── PaneContainer.vue (recursive layout)               ││
│  │      └── TerminalPane.vue (xterm.js + markdown renderer)││
│  └─────────────────────────────────────────────────────────┘│
│  Pinia Stores: layout.ts, settings.ts, workspace.ts, git.ts │
├─────────────────────────────────────────────────────────────┤
│  Tauri IPC Bridge (@tauri-apps/api)                         │
├─────────────────────────────────────────────────────────────┤
│  Rust Backend (src-tauri/)                                   │
│  ├── PTY Manager (portable-pty)                             │
│  │   └── Terminal sessions with read/write/resize           │
│  ├── Workspace Storage (JSON persistence)                   │
│  ├── Shell Detection (PowerShell, CMD, WSL, Git Bash)       │
│  └── System Integration (clipboard images, file dialogs)    │
└─────────────────────────────────────────────────────────────┘
```

## How It Works

### Terminal Sessions
1. User opens a new terminal pane
2. Rust backend spawns a PTY process with detected/selected shell
3. PTY output is streamed to frontend via Tauri events
4. xterm.js renders the output with theme colors
5. Markdown renderer processes AI tool output for enhanced display

### Layout System
- Binary tree structure where each node is either:
  - **Split node**: direction + ratio + two children
  - **Terminal node**: shell + cwd + startup command
- Operations: split, close, resize, focus navigation
- Serializable for workspace persistence

### Markdown Rendering Pipeline
```
PTY Output → Chunk Detection → Buffer Management → Markdown Parser → ANSI Renderer → xterm.js
                   │
                   ├── Real-time updates (spinners) → Pass through unchanged
                   ├── Cursor control sequences → Pass through unchanged
                   └── Markdown content → Parse and render with theme colors
```

## Current Feature Set

### Implemented
- [x] Multi-pane split terminals (horizontal/vertical)
- [x] Workspace save/load with full state restoration
- [x] 20+ color themes with live preview
- [x] Customizable accent colors
- [x] Shell auto-detection (PowerShell, CMD, WSL, Git Bash)
- [x] Command palette (Ctrl+P)
- [x] Keyboard shortcuts for all operations
- [x] Alias expansion (`!aliasname`)
- [x] Inline markdown-to-ANSI rendering
- [x] Clipboard support (text + images)
- [x] Clickable URLs
- [x] Auto-updates via Tauri updater

### In Progress / Planned
- [ ] Git integration sidebar
- [ ] Session history/search
- [ ] SSH connection management
- [ ] Plugin system
- [ ] Cloud workspace sync
- [ ] Collaboration features

## Target Users

1. **AI-Assisted Developers** - Primary target
   - Heavy users of Claude Code, Copilot CLI, aider
   - Need better rendering of AI markdown output
   - Want to preserve context across sessions

2. **Power Users**
   - Multiple projects with different shell configurations
   - Need workspace persistence
   - Value keyboard-driven workflows

3. **Windows Developers**
   - Need WSL + PowerShell + Git Bash in one app
   - Want modern terminal features Windows Terminal lacks

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Framework | Tauri 2.x | Small binary (~10MB), native performance, Rust safety |
| Frontend | Vue 3 + Vite | Fast dev experience, composition API, TypeScript |
| Terminal | xterm.js | Industry standard, extensive addon ecosystem |
| State | Pinia | Vue-native, TypeScript support, devtools |
| Styling | Tailwind CSS v4 | Rapid UI development, consistent design |
| PTY | portable-pty | Cross-platform, mature, ConPTY support |

## Development Workflow

```bash
# Install dependencies
npm install

# Development mode (hot reload)
npm run tauri dev

# Production build (creates installers)
npm run tauri build

# Type checking
npx vue-tsc --noEmit
```

## File Structure

```
terminal/
├── src/                    # Vue frontend
│   ├── components/
│   │   ├── terminal/       # TerminalPane, ShellSelector
│   │   ├── layout/         # SplitPane, PaneContainer, LayoutTree
│   │   ├── workspace/      # WorkspaceManager
│   │   └── settings/       # SettingsPanel
│   ├── stores/             # Pinia stores
│   ├── utils/
│   │   └── markdownRenderer/  # AI output rendering
│   ├── themes/             # Color scheme presets
│   └── types/              # TypeScript interfaces
├── src-tauri/              # Rust backend
│   └── src/
│       ├── pty/            # PTY management
│       └── workspace/      # Persistence
└── CLAUDE.md               # AI assistant instructions
```

## Metrics & Goals

### Performance Targets
- Cold start: < 500ms
- PTY latency: < 10ms
- Memory per terminal: < 50MB
- Binary size: < 15MB

### User Experience Goals
- Zero-config for common workflows
- Keyboard-navigable for everything
- AI output should "just look good"
- Workspaces restore in < 1 second

## Competitive Landscape

| Feature | Primarch | Windows Terminal | Warp | iTerm2 |
|---------|----------|------------------|------|--------|
| Workspace persistence | ✅ Full | ❌ | ✅ | ✅ Partial |
| AI output rendering | ✅ | ❌ | ✅ | ❌ |
| Cross-platform | ✅ Win/Mac | ❌ Win only | ❌ Mac only | ❌ Mac only |
| Multi-shell support | ✅ | ✅ | ✅ | ✅ |
| Open source | ✅ | ✅ | ❌ | ❌ |
| Binary size | ~10MB | ~50MB | ~100MB | ~30MB |

## Contact & Resources

- **Repository**: github.com/usemythicor/primarch
- **Built with**: Tauri, Vue, xterm.js, Rust
- **License**: TBD

---

*This document is intended for research and development planning purposes.*
