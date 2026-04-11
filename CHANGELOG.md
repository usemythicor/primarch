# Changelog

## [0.1.6] - 2026-04-11

### Fixes

- **PTY UTF-8 corruption on macOS** — multi-byte characters and ANSI escape sequences split across 4096-byte PTY read boundaries are now reassembled in a pending buffer instead of being replaced with U+FFFD; fixes garbled text, broken powerline glyphs, and mangled escape codes on macOS
- **macOS keyboard shortcuts** — all 14 documented shortcuts (command palette, new tab, split, close, tab switching, settings, git sidebar, terminal search, etc.) now accept Cmd on macOS, not just Ctrl
- **Global command palette shortcut on macOS** — registered as `CmdOrCtrl+P` so it resolves to Cmd on macOS and Ctrl on Windows/Linux
- **PTY working directory fallback** — uses `/` on Unix-likes instead of hardcoded `C:\`, so terminals spawn correctly on macOS even if both home and current directory lookups fail
- **Terminal font stack on macOS** — added SF Mono, Menlo, and Consolas as fallbacks so macOS users get a proper monospace font instead of generic `monospace`

### UI

- **Separate pull and push buttons in the status bar** — replaces the always-visible sync icon; a down-arrow pull button appears only when there are commits to pull, an up-arrow push button appears only when there are commits to push (or when the branch needs publishing), each with its count

## [0.1.5] - 2026-04-03

### Fixes

- **Tab switching no longer resets terminals** — all tabs now stay mounted with live PTY sessions preserved across tab switches, fixing the bug where switching tabs killed and recreated terminal sessions
- **Session registry works across all tabs** — session management functions now correctly find sessions in any tab, not just the active one
- **Terminal refit on tab switch** — terminals properly resize when their tab becomes visible again

## [0.1.4] - 2026-04-02

### Features

- **Smart bell notifications** — detect when commands finish in unfocused panes with visual border blink, audio chirp, tab dot indicator, and Windows toast notifications
- **Bell style settings** — configurable alert modes: none, visual, sound, or both
- **Process completion detection** — intelligent idle detection that arms on Enter and fires once when output stops, no false positives
- **Tabs and terminal search** — draggable tabs with full-text scrollback search and regex support
- **Markdown viewer** — resizable drawer with file browser for viewing markdown files

### Fixes

- **Bell spam prevention** — notifications only fire for unfocused panes and disarm after firing until next command

## [0.1.3] - 2026-03-27

### Features

- **Git amend and commit mode dropdown** — switch between commit and amend modes directly from the git UI
- **Terminal context menu** — right-click menu with copy, paste, clear, and split actions
- **Native macOS window management** — overlay title bar with native traffic light controls
- **Inline markdown-to-ANSI rendering** — terminal output renders markdown formatting natively
- **Alias expansion** — `!alias` shorthand expansion in terminal input
- **Global command palette shortcut** — `Ctrl+P` opens the command palette from anywhere
- **Manual update fallback** — fallback update flow when auto-update is unavailable

### Fixes

- **TUI rendering for Claude Code** — improved rendering compatibility and enabled updater capability
- **Terminal padding** — moved padding from xterm-rows to pane wrapper for consistent spacing
- **macOS terminal padding** — removed excessive left padding on macOS
- **Double-paste on Windows** — prevented clipboard content from pasting twice
- **Pane session preservation** — terminal sessions and display are preserved when splitting or closing panes
- **Layout presets and split shortcuts** — corrected preset layouts, fixed split keyboard shortcuts, added active pane indicator
- **Markdown renderer status updates** — preserved real-time status updates during rendering

### Chores

- Removed CI workflow
- Fixed repository URLs and code quality issues
- Updated app icons

## [0.1.2] - 2025-12-01

Initial tracked release with command palette, directory browser, and PATH fixes.

## [0.1.1] - 2025-11-15

Windows console window flash fix.
