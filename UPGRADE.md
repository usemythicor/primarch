# Upgrade Roadmap

Feature gap analysis vs popular terminal emulators (Windows Terminal, iTerm2, Warp, WezTerm, Kitty, Tabby).

## Completed

### Tabs
Multiple tab support alongside the existing split-pane system. Each tab contains its own split layout tree.
- [x] Tab bar UI with add/close/reorder (auto-hides when single tab)
- [x] Each tab owns an independent layout tree and session registry
- [x] Keyboard shortcuts (Ctrl+T new, Ctrl+W close, Ctrl+PageDown/PageUp cycle, Ctrl+1-9 switch)
- [x] Double-click to rename, middle-click to close
- [x] Drag to reorder tabs
- [x] Command palette integration (New Tab / Close Tab)
- [ ] Tab context menu (rename, duplicate, close others)

### Search in Terminal
Find text in terminal scrollback buffer.
- [x] Integrate `@xterm/addon-search`
- [x] Search bar UI (Ctrl+Shift+F)
- [x] Next/previous match navigation (Enter / Shift+Enter)
- [x] Match highlighting
- [x] Case sensitivity toggle
- [x] Regex toggle
- [x] Match count display (e.g. "2 of 5")

## Planned — High Impact

### Auto Session Restore
Automatically restore open terminals on relaunch (not just manual workspace save/load).
- [ ] Persist active layout + CWDs on close
- [ ] Restore layout and reconnect shells on startup
- [ ] Option to disable in settings

### Profiles
Named shell profiles combining shell, theme, font, and startup command.
- [ ] Profile creation/editing UI
- [ ] Assign profile to new tab or pane
- [ ] `ShellProfile` type already exists — wire it up
- [ ] Default profile setting

### Quake / Dropdown Mode
Global hotkey to toggle terminal as a dropdown overlay.
- [ ] Global shortcut registration (already have plugin)
- [ ] Animate window in/out from top of screen
- [ ] Configurable hotkey and screen edge

## Planned — Medium Impact

### Window Opacity / Blur
Acrylic or transparent terminal backgrounds.
- [ ] Opacity slider in settings
- [ ] Acrylic/blur material on Windows
- [ ] Vibrancy on macOS

### Notifications
Alert when long-running commands finish.
- [ ] Bell character handling
- [ ] System notification on process completion (when terminal not focused)
- [ ] Configurable in settings

### Broadcast Input
Type in one pane, send to all panes simultaneously.
- [ ] Toggle broadcast mode per tab
- [ ] Visual indicator on broadcasting panes

### Drag and Drop Files
Drop files onto terminal to paste their path.
- [ ] Listen for file drop events
- [ ] Insert quoted file path at cursor

### Shell Integration Marks
Command boundaries and exit status decorations.
- [ ] Detect command start/end via OSC sequences
- [ ] Gutter decorations for success/failure
- [ ] Click to scroll between commands

## Backlog — Lower Priority

### Image Rendering
Inline images via sixel, kitty, or iTerm2 image protocol. Limited by xterm.js capabilities.

### SSH Manager
Saved SSH connections with profiles and one-click connect.

### Ligature Support
Requires xterm.js canvas/WebGL addon work — limited by upstream.

### Environment Variable Management
Per-session or per-profile env var editor.
