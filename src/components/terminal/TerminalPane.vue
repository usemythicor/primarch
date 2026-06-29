<script lang="ts">
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { SearchAddon } from '@xterm/addon-search';
import { MarkdownRenderer } from '../../utils/markdownRenderer';

// Module-level cache: shared across ALL TerminalPane instances
// Preserves xterm instances across splits so scrollback and display are not lost
const xtermCache = new Map<string, {
  terminal: Terminal;
  fitAddon: FitAddon;
  searchAddon: SearchAddon;
  element: HTMLDivElement;
  markdownRenderer: MarkdownRenderer | null;
  onDataDisposable: { dispose: () => void } | null;
}>();
</script>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { invoke } from '@tauri-apps/api/core';
import { readText, readImage, writeText } from '@tauri-apps/plugin-clipboard-manager';
import { openUrl, openPath } from '@tauri-apps/plugin-opener';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { XMarkIcon, ChevronDownIcon } from '@heroicons/vue/24/outline';
import { useTerminal } from '../../composables/useTerminal';
import { useSettingsStore } from '../../stores/settings';
import { useLayoutStore } from '../../stores/layout';
import { useGitStore } from '../../stores/git';
import { getAliases } from '../../utils/aliases';
import SearchBar from './SearchBar.vue';
import '@xterm/xterm/css/xterm.css';

const props = defineProps<{
  nodeId?: string;
  existingSessionId?: string;
  shell?: string;
  cwd?: string;
  startupCommand?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'title-change', title: string): void;
}>();

const settingsStore = useSettingsStore();
const layoutStore = useLayoutStore();
const gitStore = useGitStore();
const terminalRef = ref<HTMLDivElement>();
const sessionId = ref<string>();
const isConnected = ref(false);

// Floating preview shown after pasting an image (the file path is still written
// to the PTY; this just gives a visual confirmation of what was pasted).
const pastePreviewUrl = ref<string | null>(null);
let pastePreviewPath: string | null = null;
let pastePreviewTimer: ReturnType<typeof setTimeout> | null = null;

let terminal: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let searchAddon: SearchAddon | null = null;
let xtermElement: HTMLDivElement | null = null; // Direct ref to the xterm wrapper div
let inputBuffer = ''; // Track current line input for alias expansion
let markdownRenderer: MarkdownRenderer | null = null;
let onDataDisposable: { dispose: () => void } | null = null; // Track onData listener for cleanup on reattach

// Search state
const showSearch = ref(false);
let lastSearchQuery = '';
let lastSearchOptions: { caseSensitive: boolean; regex: boolean } = { caseSensitive: false, regex: false };

function toggleSearch() {
  showSearch.value = !showSearch.value;
  if (!showSearch.value && searchAddon) {
    searchAddon.clearDecorations();
  }
}

const searchDecorations = {
  matchBackground: '#3a3520',
  matchBorder: '#8a7a30',
  matchOverviewRuler: '#ffd93d',
  activeMatchBackground: '#5a4a00',
  activeMatchBorder: '#ffd93d',
  activeMatchColorOverviewRuler: '#ffd93d',
};

const searchResultCount = ref(0);
const searchResultIndex = ref(-1);

function setupSearchResultListener() {
  if (!searchAddon) return;
  searchAddon.onDidChangeResults((e) => {
    searchResultCount.value = e.resultCount;
    searchResultIndex.value = e.resultIndex;
  });
}

function handleSearch(query: string, options: { caseSensitive: boolean; regex: boolean }) {
  if (!searchAddon) return;
  lastSearchQuery = query;
  lastSearchOptions = options;
  if (!query) {
    searchAddon.clearDecorations();
    searchResultCount.value = 0;
    searchResultIndex.value = -1;
    return;
  }
  searchAddon.findNext(query, {
    caseSensitive: options.caseSensitive,
    regex: options.regex,
    decorations: searchDecorations,
    incremental: true,
  });
}

function handleSearchNext() {
  if (!searchAddon || !lastSearchQuery) return;
  searchAddon.findNext(lastSearchQuery, {
    ...lastSearchOptions,
    decorations: searchDecorations,
  });
}

function handleSearchPrevious() {
  if (!searchAddon || !lastSearchQuery) return;
  searchAddon.findPrevious(lastSearchQuery, {
    ...lastSearchOptions,
    decorations: searchDecorations,
  });
}

function closeSearch() {
  showSearch.value = false;
  searchAddon?.clearDecorations();
  terminal?.focus();
}

// Watch for search toggle signal from layout store (only respond if this is the active pane)
watch(() => layoutStore.searchToggleSignal, () => {
  if (props.nodeId && layoutStore.activePane === props.nodeId) {
    toggleSearch();
  }
});

// Refit terminal when its tab becomes visible after a tab switch
watch(() => layoutStore.tabSwitchSignal, () => {
  if (!props.nodeId || !sessionId.value) return;

  // Check if this pane belongs to the now-active tab by checking session registry
  const activeTab = layoutStore.tabs.find(t => t.id === layoutStore.activeTabId);
  if (!activeTab || !activeTab.sessionRegistry.has(props.nodeId)) return;

  // Refit after the DOM has updated visibility
  requestAnimationFrame(() => {
    if (fitAddon && terminal && sessionId.value && terminalRef.value) {
      const { clientWidth, clientHeight } = terminalRef.value;
      if (clientWidth === 0 || clientHeight === 0) return;
      try {
        fitAddon.fit();
        terminal.refresh(0, terminal.rows - 1);
        const dimensions = fitAddon.proposeDimensions();
        if (dimensions && dimensions.cols > 0 && dimensions.rows > 0) {
          resize(sessionId.value, dimensions.cols, dimensions.rows);
        }
      } catch {
        // Ignore fit errors during transition
      }
    }
  });
});

// Clear bell blink when this pane becomes active
watch(() => layoutStore.activePane, (newActive) => {
  if (newActive === props.nodeId && bellFlash.value) {
    bellFlash.value = false;
  }
});

const { createSession, startReading, reattachReading, write, resize, kill, cleanup } = useTerminal();

// Check if input matches an alias and return expanded command, or null if no match
function expandAlias(input: string): string | null {
  if (!input.startsWith('!')) return null;

  const aliasName = input.slice(1).trim();
  if (!aliasName) return null;

  const aliases = getAliases();
  const alias = aliases.find(a => a.name === aliasName);

  return alias ? alias.command : null;
}

const bgColor = computed(() => settingsStore.currentTheme.background);
const bellFlash = ref(false);

// Process completion detection — fires ONCE when a long-running command finishes.
// Tracks sustained output over time. When output stops, fires once and arms only
// after the user sends new input (Enter key = new command).
let idleTimer: ReturnType<typeof setTimeout> | null = null;
let outputBytesInBurst = 0;
let outputStartTime = 0;
let commandRunning = false; // Armed when user presses Enter, disarmed after notification
let paneReady = false; // Suppress bells during shell startup
const STARTUP_GRACE_MS = 5000;
const IDLE_THRESHOLD_MS = 2000;
const MIN_OUTPUT_FOR_NOTIFICATION = 1; // Any output after Enter counts
const MIN_DURATION_MS = 10000; // Only notify for commands that run 10+ seconds

function isPaneActive(): boolean {
  return document.hasFocus() && layoutStore.activePane === props.nodeId;
}

function onTerminalOutput(dataLength: number) {
  if (!commandRunning || !paneReady) return; // Not armed or still starting up

  if (outputBytesInBurst === 0) {
    outputStartTime = Date.now();
  }
  outputBytesInBurst += dataLength;

  // Reset idle timer on every output chunk
  if (idleTimer) clearTimeout(idleTimer);
  idleTimer = setTimeout(() => {
    const duration = Date.now() - outputStartTime;
    // Only fire if: enough output, ran long enough, and pane not focused
    if (
      outputBytesInBurst >= MIN_OUTPUT_FOR_NOTIFICATION &&
      duration >= MIN_DURATION_MS &&
      !isPaneActive()
    ) {
      handleBell();
      commandRunning = false; // Disarm — won't fire again until next Enter
    }
    outputBytesInBurst = 0;
  }, IDLE_THRESHOLD_MS);
}

function onUserInput(data: string) {
  // Enter key arms the detector for the next command
  if (data === '\r') {
    commandRunning = true;
    outputBytesInBurst = 0;
    if (idleTimer) {
      clearTimeout(idleTimer);
      idleTimer = null;
    }
  }
}

// AudioContext singleton for bell sound
let audioCtx: AudioContext | null = null;

function playBellSound() {
  if (!audioCtx) {
    audioCtx = new AudioContext();
  }
  const osc = audioCtx.createOscillator();
  const gain = audioCtx.createGain();
  osc.connect(gain);
  gain.connect(audioCtx.destination);
  osc.frequency.value = 800;
  osc.type = 'sine';
  gain.gain.setValueAtTime(0.15, audioCtx.currentTime);
  gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.15);
  osc.start();
  osc.stop(audioCtx.currentTime + 0.15);
}

async function handleBell() {
  const style = settingsStore.bellStyle;
  if (style === 'none' || !paneReady) return;

  // Visual blink — stays blinking until pane is focused
  if (style === 'visual' || style === 'both') {
    bellFlash.value = true;
  }

  // Audio beep
  if (style === 'sound' || style === 'both') {
    playBellSound();
  }

  // Notify layout store for tab indicator
  if (props.nodeId) {
    layoutStore.notifyBellForPane(props.nodeId);
  }

  // OS-level notification when window is not focused
  if (!document.hasFocus()) {
    try {
      let permGranted = await isPermissionGranted();
      if (!permGranted) {
        const perm = await requestPermission();
        permGranted = perm === 'granted';
      }
      if (permGranted) {
        sendNotification({
          title: 'Primarch',
          body: 'A terminal needs your attention',
        });
      }
    } catch {
      // Notification not available
    }
  }
}

// Handle clipboard paste
async function handlePaste() {
  if (!sessionId.value) return;

  try {
    // Try to read text first
    const text = await readText();
    if (text) {
      // Guard against accidentally running multiple commands at once: a paste
      // containing newlines (other than a single trailing one) is confirmed
      // first instead of being executed immediately.
      const lineCount = text.replace(/\r?\n$/, '').split(/\r?\n/).length;
      if (lineCount > 1) {
        pendingPasteText.value = text;
        pendingPasteLines.value = lineCount;
      } else {
        await write(sessionId.value, text);
      }
      return;
    }
  } catch {
    // No text in clipboard, try image
  }

  try {
    // Try to read image from clipboard
    const image = await readImage();
    if (image) {
      const size = await image.size();
      const rgbaData = await image.rgba();

      // Save image to temp file and get the path
      const filePath = await invoke<string>('save_clipboard_image', {
        rgbaData: Array.from(rgbaData),
        width: size.width,
        height: size.height,
      });

      // Paste the file path (with quotes in case of spaces)
      await write(sessionId.value, `"${filePath}"`);

      // Show a floating thumbnail so the user can see what they pasted.
      showImagePreview(rgbaData, size.width, size.height, filePath);
    }
  } catch (e) {
    // No image or failed to save, ignore
    console.error('Failed to paste image:', e);
  }
}

// Build a small thumbnail from the pasted RGBA pixels and show it as a
// floating overlay in the pane corner. Auto-dismisses after a few seconds.
function showImagePreview(
  rgba: Uint8Array,
  width: number,
  height: number,
  path: string,
) {
  try {
    const source = document.createElement('canvas');
    source.width = width;
    source.height = height;
    const sctx = source.getContext('2d');
    if (!sctx) return;
    sctx.putImageData(new ImageData(new Uint8ClampedArray(rgba), width, height), 0, 0);

    // Downscale to a thumbnail bounded by maxDim on its longest side.
    const maxDim = 180;
    const scale = Math.min(1, maxDim / Math.max(width, height));
    const tw = Math.max(1, Math.round(width * scale));
    const th = Math.max(1, Math.round(height * scale));
    const thumb = document.createElement('canvas');
    thumb.width = tw;
    thumb.height = th;
    const tctx = thumb.getContext('2d');
    if (!tctx) return;
    tctx.drawImage(source, 0, 0, tw, th);

    pastePreviewUrl.value = thumb.toDataURL('image/png');
    pastePreviewPath = path;

    if (pastePreviewTimer) clearTimeout(pastePreviewTimer);
    pastePreviewTimer = setTimeout(dismissImagePreview, 6000);
  } catch (e) {
    console.error('Failed to build image preview:', e);
  }
}

function dismissImagePreview() {
  pastePreviewUrl.value = null;
  pastePreviewPath = null;
  if (pastePreviewTimer) {
    clearTimeout(pastePreviewTimer);
    pastePreviewTimer = null;
  }
}

// Open the pasted image full-size with the OS default viewer.
async function openPastedImage() {
  const path = pastePreviewPath;
  dismissImagePreview();
  if (path) {
    try {
      await openPath(path);
    } catch (e) {
      console.error('Failed to open pasted image:', e);
    }
  }
}

// Detects absolute (C:\…, /usr/…), home (~/…), dot-relative (./…, ../…), and
// word/word.ext relative file paths, with an optional :line:col suffix.
const FILE_PATH_RE = /(?<=^|\s)((?:[A-Za-z]:[\\/]|\.\.?[\\/]|~[\\/]|[\\/])[^\s:*?"<>|]*|(?:[\w.\-]+[\\/])+[\w.\-]+\.[A-Za-z0-9]+)(?::(\d+)(?::(\d+))?)?/g;

// Build an xterm link provider that turns file paths in the buffer into
// clickable links.
function createFilePathLinkProvider(term: Terminal) {
  return {
    provideLinks(y: number, callback: (links: any[] | undefined) => void) {
      const line = term.buffer.active.getLine(y - 1);
      if (!line) {
        callback(undefined);
        return;
      }
      const text = line.translateToString(true);
      const links: any[] = [];
      const re = new RegExp(FILE_PATH_RE.source, 'g');
      let m: RegExpExecArray | null;
      while ((m = re.exec(text)) !== null) {
        const full = m[0];
        const path = m[1];
        const lineNo = m[2] ? parseInt(m[2], 10) : undefined;
        links.push({
          text: full,
          range: {
            start: { x: m.index + 1, y },
            end: { x: m.index + full.length, y },
          },
          activate: () => { openFilePathLink(path, lineNo); },
        });
      }
      callback(links.length ? links : undefined);
    },
  };
}

async function currentCwd(): Promise<string | null> {
  if (!sessionId.value) return null;
  try {
    return await invoke<string>('get_terminal_cwd', { sessionId: sessionId.value });
  } catch {
    return null;
  }
}

async function openFilePathLink(rawPath: string, _lineNo?: number) {
  try {
    // Strip trailing punctuation that often abuts a path in prose.
    let p = rawPath.replace(/[)\]}.,;'"]+$/, '');
    const isAbsolute = /^[A-Za-z]:[\\/]/.test(p) || /^[\\/]/.test(p) || p.startsWith('~');
    if (!isAbsolute) {
      const cwd = await currentCwd();
      if (!cwd) return;
      const sep = cwd.includes('\\') ? '\\' : '/';
      p = cwd.replace(/[\\/]$/, '') + sep + p.replace(/^\.[\\/]/, '');
    }
    if (/\.md$/i.test(p) && (window as any).__openMarkdownViewer) {
      (window as any).__openMarkdownViewer(p);
      return;
    }
    await openPath(p);
  } catch (e) {
    console.error('Failed to open path link:', e);
  }
}

// Optional per-pane header showing directory + running process.
const headerCwd = ref('');
const headerTitle = ref('');
const headerDir = computed(() => {
  const p = headerCwd.value;
  if (!p) return '';
  const trimmed = p.replace(/[\\/]+$/, '');
  const parts = trimmed.split(/[\\/]/);
  return parts[parts.length - 1] || trimmed;
});
let headerInterval: ReturnType<typeof setInterval> | null = null;

async function updatePaneHeader() {
  if (!settingsStore.showPaneHeader || !sessionId.value) return;
  try {
    const cwd = await invoke<string>('get_terminal_cwd', { sessionId: sessionId.value });
    if (cwd) headerCwd.value = cwd;
  } catch { /* ignore */ }
}

// True when the viewport is scrolled up into the scrollback (not at live bottom).
const isScrolledUp = ref(false);

function scrollToLiveBottom() {
  terminal?.scrollToBottom();
  isScrolledUp.value = false;
  terminal?.focus();
}

// Multi-line paste confirmation state
const pendingPasteText = ref<string | null>(null);
const pendingPasteLines = ref(0);

async function confirmPaste() {
  const text = pendingPasteText.value;
  pendingPasteText.value = null;
  if (text && sessionId.value) {
    await write(sessionId.value, text);
  }
  terminal?.focus();
}

function cancelPaste() {
  pendingPasteText.value = null;
  terminal?.focus();
}

// Handle clipboard copy
async function handleCopy(text: string) {
  try {
    await writeText(text);
  } catch (e) {
    console.error('Failed to copy to clipboard:', e);
  }
}

// Watch for theme/settings changes
watch(
  () => settingsStore.terminalOptions,
  (options) => {
    if (terminal) {
      terminal.options.theme = options.theme;
      terminal.options.fontSize = options.fontSize;
      terminal.options.fontFamily = options.fontFamily;
      terminal.options.cursorBlink = options.cursorBlink;
      terminal.options.cursorStyle = options.cursorStyle;

      // Refit after font size change
      if (fitAddon) {
        fitAddon.fit();
      }
    }
  },
  { deep: true }
);

// Initialize terminal
onMounted(async () => {
  if (!terminalRef.value) return;

  // Poll the directory for the optional pane header (cheap, gated by setting).
  headerInterval = setInterval(updatePaneHeader, 2000);

  const reattachId = props.existingSessionId;
  const isReattach = reattachId && layoutStore.isPendingReattach(reattachId);
  const cached = isReattach ? xtermCache.get(reattachId) : null;

  if (cached) {
    // Reuse existing xterm instance — preserves scrollback and display
    terminal = cached.terminal;
    fitAddon = cached.fitAddon;
    searchAddon = cached.searchAddon;
    setupSearchResultListener();
    markdownRenderer = cached.markdownRenderer;
    // Dispose old onData handler to prevent duplicate input after split
    if (cached.onDataDisposable) {
      cached.onDataDisposable.dispose();
    }
    // Move the xterm DOM element to our new container
    xtermElement = cached.element;
    terminalRef.value.appendChild(xtermElement);
    xtermCache.delete(reattachId!);
    fitAddon.fit();
  } else {
    const options = settingsStore.terminalOptions;

    // Create xterm instance
    terminal = new Terminal({
      cursorBlink: options.cursorBlink,
      cursorStyle: options.cursorStyle,
      fontSize: options.fontSize,
      fontFamily: options.fontFamily,
      theme: options.theme,
      allowProposedApi: true,
    });

    // Add addons
    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.loadAddon(new WebLinksAddon((_event, uri) => {
      openUrl(uri).catch((err) => {
        console.error('Failed to open URL:', err);
      });
    }));

    searchAddon = new SearchAddon();
    terminal.loadAddon(searchAddon);
    setupSearchResultListener();

    // Make file paths in terminal output clickable (opens in the OS default app,
    // or the in-app markdown viewer for .md files).
    terminal.registerLinkProvider(createFilePathLinkProvider(terminal));

    // Create a wrapper div for the terminal so we can re-parent it later
    xtermElement = document.createElement('div');
    xtermElement.style.width = '100%';
    xtermElement.style.height = '100%';
    terminalRef.value.appendChild(xtermElement);
    terminal.open(xtermElement);
    fitAddon.fit();

    // Initialize markdown renderer
    markdownRenderer = new MarkdownRenderer({
      theme: settingsStore.currentTheme,
      enabled: settingsStore.markdownRendering !== 'never',
    });

    markdownRenderer.setFlushCallback((data) => {
      terminal?.write(data);
    });
  }

  // Data handler for PTY output
  const oscRegex = /\x1b\]7777;([^;]+);([^\x07\x1b]*)\x07/g;

  const onPtyData = (data: string) => {
    // Track output for process completion detection
    onTerminalOutput(data.length);

    // Check for custom OSC sequences (e.g. open-md)
    const match = oscRegex.exec(data);
    if (match) {
      const [fullMatch, command, arg] = match;
      if (command === 'open-md' && arg) {
        (window as any).__openMarkdownViewer?.(arg);
      }
      // Strip the OSC sequence from output so it doesn't render as garbage
      data = data.replace(fullMatch, '');
      oscRegex.lastIndex = 0;
      if (!data) return;
    }
    oscRegex.lastIndex = 0;

    if (markdownRenderer && settingsStore.markdownRendering !== 'never') {
      const processed = markdownRenderer.process(data);
      if (processed) {
        terminal?.write(processed);
      }
    } else {
      terminal?.write(data);
    }
  };
  const onPtyClose = () => {
    isConnected.value = false;
    emit('close');
  };
  const onPtyError = () => {
    // Terminal read error - session may have ended
  };

  // Create or reattach PTY session
  try {
    if (cached) {
      // Reattach to existing PTY session (preserves running processes + display)
      sessionId.value = reattachId!;
      layoutStore.clearPendingReattach(reattachId!);
      // Re-subscribe to PTY events
      await reattachReading(sessionId.value, onPtyData, onPtyClose, onPtyError);
      // Resize PTY to new dimensions
      const dims = fitAddon.proposeDimensions();
      if (dims) {
        await resize(sessionId.value, dims.cols, dims.rows);
      }
    } else {
      // Create a fresh PTY session
      sessionId.value = await createSession(props.shell, props.cwd, settingsStore.timestampPrompt);
      await startReading(sessionId.value, onPtyData, onPtyClose, onPtyError);
    }

    isConnected.value = true;

    // Allow bells after startup grace period (shell prompts often emit BEL)
    setTimeout(() => { paneReady = true; }, STARTUP_GRACE_MS);

    // Register session with layout store for cwd tracking
    if (props.nodeId) {
      layoutStore.registerSession(props.nodeId, sessionId.value);
    }

    // Try to detect git repository from CWD
    if (!gitStore.hasRepo) {
      const tryDetectGitRepo = async () => {
        let cwd = props.cwd;
        if (!cwd && sessionId.value) {
          try {
            cwd = await invoke<string>('get_terminal_cwd', { sessionId: sessionId.value });
          } catch {
            // CWD not available yet
          }
        }
        if (cwd) {
          gitStore.openRepository(cwd).catch(() => {});
        }
      };
      tryDetectGitRepo();
      setTimeout(tryDetectGitRepo, 1000);
    }

    // Send terminal size to PTY
    const dimensions = fitAddon.proposeDimensions();
    if (dimensions) {
      await resize(sessionId.value, dimensions.cols, dimensions.rows);
    }

    // Handle user input with alias expansion support
    onDataDisposable = terminal.onData(async (data) => {
      if (!sessionId.value) return;

      // Track input for process completion detection (arms on Enter)
      onUserInput(data);

      // Check for Enter key (carriage return)
      if (data === '\r') {
        const expanded = expandAlias(inputBuffer.trim());

        if (expanded) {
          // Clear the typed alias from the terminal line
          // Send backspaces to erase what was typed
          const backspaces = '\b \b'.repeat(inputBuffer.length);
          await write(sessionId.value, backspaces);

          // Notify markdown renderer about the command
          markdownRenderer?.onCommand(expanded);

          // Send the expanded command + Enter
          await write(sessionId.value, expanded + '\r');
        } else {
          // Notify markdown renderer about the command
          markdownRenderer?.onCommand(inputBuffer);

          // No alias match, send Enter normally
          await write(sessionId.value, data);
        }

        // Reset buffer after Enter
        inputBuffer = '';
        return;
      }

      // Handle backspace (ASCII 127 or \b)
      if (data === '\x7f' || data === '\b') {
        if (inputBuffer.length > 0) {
          inputBuffer = inputBuffer.slice(0, -1);
        }
        await write(sessionId.value, data);
        return;
      }

      // Handle Ctrl+C, Ctrl+D, etc. - reset buffer
      if (data.charCodeAt(0) < 32 && data !== '\t') {
        inputBuffer = '';
        await write(sessionId.value, data);
        return;
      }

      // Regular character - add to buffer and send
      inputBuffer += data;
      await write(sessionId.value, data);
    });

    // Handle title changes
    terminal.onTitleChange((title) => {
      headerTitle.value = title;
      emit('title-change', title);
    });

    // Handle bell (BEL character \x07)
    terminal.onBell(() => {
      handleBell();
    });

    // Track whether the viewport is scrolled up from the live bottom so we can
    // show a "jump to bottom" affordance.
    terminal.onScroll((position) => {
      isScrolledUp.value = position < (terminal?.buffer.active.baseY ?? 0);
    });

    // Intercept paste events at capture phase before xterm handles them
    // This is the ONLY place we handle paste - prevents double-paste
    terminalRef.value.addEventListener('paste', (event) => {
      event.preventDefault();
      event.stopPropagation();
      handlePaste();
    }, { capture: true });

    // Intercept app-level keyboard shortcuts at capture phase before xterm handles them
    // This ensures shortcuts work on Windows where xterm may consume events
    // We re-dispatch the event to window to ensure App.vue handler receives it
    terminalRef.value.addEventListener('keydown', (event) => {
      const isCtrlOrCmd = event.ctrlKey || event.metaKey;
      let shouldRedispatch = false;

      // App-level shortcuts that should bypass xterm
      if (isCtrlOrCmd && event.shiftKey) {
        if (['KeyD', 'KeyE', 'KeyW', 'KeyS', 'KeyG', 'Tab'].includes(event.code)) {
          shouldRedispatch = true;
        }
      }
      // Ctrl+P for command palette
      if (isCtrlOrCmd && !event.shiftKey && event.code === 'KeyP') shouldRedispatch = true;
      if (isCtrlOrCmd && event.code === 'Comma') shouldRedispatch = true;
      if (event.ctrlKey && event.code === 'Tab') shouldRedispatch = true;
      // Tab shortcuts: Ctrl+T, Ctrl+W, Ctrl+PageDown/Up, Ctrl+1-9
      if (isCtrlOrCmd && !event.shiftKey && event.code === 'KeyT') shouldRedispatch = true;
      if (isCtrlOrCmd && !event.shiftKey && event.code === 'KeyW') shouldRedispatch = true;
      if (isCtrlOrCmd && (event.code === 'PageDown' || event.code === 'PageUp')) shouldRedispatch = true;
      if (isCtrlOrCmd && !event.shiftKey && event.code.match(/^Digit[1-9]$/)) shouldRedispatch = true;
      // Ctrl+Shift+F for terminal search
      if (isCtrlOrCmd && event.shiftKey && event.code === 'KeyF') shouldRedispatch = true;

      if (shouldRedispatch) {
        event.preventDefault();
        event.stopPropagation();
        // Dispatch a new keyboard event to window so App.vue handler can process it
        window.dispatchEvent(new KeyboardEvent('keydown', {
          code: event.code,
          key: event.key,
          ctrlKey: event.ctrlKey,
          shiftKey: event.shiftKey,
          altKey: event.altKey,
          metaKey: event.metaKey,
          bubbles: true,
        }));
      }
    }, { capture: true });

    // Handle clipboard copy and let app-level shortcuts bubble up
    terminal.attachCustomKeyEventHandler((event) => {
      if (event.type === 'keydown') {
        // Scrollback navigation (handled locally, not forwarded to the PTY)
        if (event.shiftKey && event.code === 'PageUp') {
          terminal?.scrollPages(-1);
          return false;
        }
        if (event.shiftKey && event.code === 'PageDown') {
          terminal?.scrollPages(1);
          return false;
        }
        if (event.ctrlKey && event.code === 'Home') {
          terminal?.scrollToTop();
          return false;
        }
        if (event.ctrlKey && event.code === 'End') {
          terminal?.scrollToBottom();
          return false;
        }
        // Ctrl+V: let the paste event listener handle it (prevents double-paste)
        if (event.ctrlKey && event.code === 'KeyV') {
          return false;
        }
        // Ctrl+C with selection: copy instead of SIGINT
        if (event.ctrlKey && event.code === 'KeyC') {
          const selection = terminal?.getSelection();
          if (selection) {
            handleCopy(selection);
            return false;
          }
        }
        // Let app-level shortcuts bubble up to window handler (use event.code for consistency)
        const isCtrlOrCmd = event.ctrlKey || event.metaKey;
        if (isCtrlOrCmd && event.shiftKey) {
          if (['KeyD', 'KeyE', 'KeyW', 'KeyS', 'KeyG', 'KeyF', 'Tab'].includes(event.code)) {
            return false;
          }
        }
        // Ctrl+P for command palette
        if (isCtrlOrCmd && !event.shiftKey && event.code === 'KeyP') return false;
        if (isCtrlOrCmd && event.code === 'Comma') return false;
        if (event.ctrlKey && event.code === 'Tab') return false;
        // Tab shortcuts
        if (isCtrlOrCmd && !event.shiftKey && event.code === 'KeyT') return false;
        if (isCtrlOrCmd && !event.shiftKey && event.code === 'KeyW') return false;
        if (isCtrlOrCmd && (event.code === 'PageDown' || event.code === 'PageUp')) return false;
        if (isCtrlOrCmd && !event.shiftKey && event.code.match(/^Digit[1-9]$/)) return false;
      }
      return true;
    });

    // Run startup command if provided
    if (props.startupCommand) {
      await write(sessionId.value, props.startupCommand + '\r');
    }
  } catch (error) {
    // Display error in the terminal itself so user can see it
    terminal.writeln(`\x1b[31mFailed to create terminal session: ${error}\x1b[0m`);
  }

  // Handle resize - guard against zero-size containers during maximize/restore animations
  const resizeObserver = new ResizeObserver(() => {
    requestAnimationFrame(() => {
      if (fitAddon && terminal && sessionId.value && terminalRef.value) {
        const { clientWidth, clientHeight } = terminalRef.value;
        if (clientWidth === 0 || clientHeight === 0) return;

        try {
          fitAddon.fit();
          terminal.refresh(0, terminal.rows - 1);
          const dimensions = fitAddon.proposeDimensions();
          if (dimensions && dimensions.cols > 0 && dimensions.rows > 0) {
            resize(sessionId.value, dimensions.cols, dimensions.rows);
          }
        } catch {
          // fit() can throw during animations - will recover on next resize
        }
      }
    });
  });

  resizeObserver.observe(terminalRef.value);

  // Fallback: re-fit after window resize settles (catches maximize/restore animations)
  let resizeTimeout: ReturnType<typeof setTimeout> | null = null;
  function handleWindowResize() {
    if (resizeTimeout) clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
      if (fitAddon && terminal && sessionId.value && terminalRef.value) {
        const { clientWidth, clientHeight } = terminalRef.value;
        if (clientWidth === 0 || clientHeight === 0) return;
        try {
          fitAddon.fit();
          terminal.refresh(0, terminal.rows - 1);
          const dimensions = fitAddon.proposeDimensions();
          if (dimensions && dimensions.cols > 0 && dimensions.rows > 0) {
            resize(sessionId.value, dimensions.cols, dimensions.rows);
          }
        } catch {
          // Ignore fit errors
        }
      }
    }, 150);
  }
  window.addEventListener('resize', handleWindowResize);

  onUnmounted(() => {
    resizeObserver.disconnect();
    window.removeEventListener('resize', handleWindowResize);
    if (resizeTimeout) clearTimeout(resizeTimeout);
  });
});

// Cleanup on unmount
onUnmounted(async () => {
  if (pastePreviewTimer) clearTimeout(pastePreviewTimer);
  if (headerInterval) clearInterval(headerInterval);

  // Unregister session from layout store
  if (props.nodeId) {
    layoutStore.unregisterSession(props.nodeId);
  }

  if (sessionId.value && layoutStore.isPendingReattach(sessionId.value)) {
    // Split in progress — cache the xterm instance and keep PTY alive
    cleanup(sessionId.value);
    if (terminal && fitAddon && searchAddon && xtermElement) {
      xtermCache.set(sessionId.value, {
        terminal,
        fitAddon,
        searchAddon,
        element: xtermElement,
        markdownRenderer,
        onDataDisposable,
      });
    }
    // Don't dispose terminal or kill session
    return;
  }

  if (sessionId.value) {
    await kill(sessionId.value);
  }
  terminal?.dispose();
});

// Focus the terminal
function focus() {
  terminal?.focus();
}

// Expose focus and search methods
/** Extract the full terminal scrollback buffer as plain text. */
function getBufferText(): string {
  if (!terminal) return '';
  const buf = terminal.buffer.active;
  const lines: string[] = [];
  for (let i = 0; i < buf.length; i++) {
    const line = buf.getLine(i);
    if (line) lines.push(line.translateToString(true));
  }
  // Trim trailing empty lines
  while (lines.length > 0 && lines[lines.length - 1].trim() === '') {
    lines.pop();
  }
  return lines.join('\n');
}

defineExpose({ focus, toggleSearch, getBufferText });
</script>

<template>
  <div
    class="terminal-pane"
    :class="{ searching: showSearch, 'bell-flash': bellFlash }"
    :style="{ backgroundColor: bgColor }"
    @mousedown="bellFlash = false"
  >
    <SearchBar
      :visible="showSearch"
      :result-count="searchResultCount"
      :result-index="searchResultIndex"
      @search="handleSearch"
      @next="handleSearchNext"
      @previous="handleSearchPrevious"
      @close="closeSearch"
    />
    <div v-if="settingsStore.showPaneHeader" class="pane-header">
      <span class="pane-header-dir">{{ headerDir || '—' }}</span>
      <span v-if="headerTitle" class="pane-header-proc">{{ headerTitle }}</span>
    </div>

    <div ref="terminalRef" class="terminal-container"></div>

    <!-- Scroll-locked indicator: jump back to the live bottom -->
    <Transition name="paste-preview">
      <button
        v-if="isScrolledUp"
        class="scroll-bottom-pill"
        title="Scrolled up — jump to bottom"
        @click="scrollToLiveBottom"
      >
        <ChevronDownIcon class="w-3 h-3" />
        <span>Jump to bottom</span>
      </button>
    </Transition>

    <!-- Multi-line paste confirmation -->
    <Transition name="paste-preview">
      <div v-if="pendingPasteText" class="paste-confirm-backdrop" @click.self="cancelPaste">
        <div class="paste-confirm">
          <div class="paste-confirm-title">
            Paste {{ pendingPasteLines }} lines?
          </div>
          <pre class="paste-confirm-body">{{ pendingPasteText }}</pre>
          <div class="paste-confirm-actions">
            <button class="paste-confirm-btn" @click="cancelPaste">Cancel</button>
            <button class="paste-confirm-btn paste-confirm-btn-primary" @click="confirmPaste">
              Paste
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Floating preview of the most recently pasted image -->
    <Transition name="paste-preview">
      <div
        v-if="pastePreviewUrl"
        class="paste-preview"
        title="Open pasted image"
        @click="openPastedImage"
      >
        <img :src="pastePreviewUrl" alt="Pasted image preview" />
        <button
          class="paste-preview-close"
          title="Dismiss"
          @click.stop="dismissImagePreview"
        >
          <XMarkIcon class="w-3 h-3" />
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.terminal-pane {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  padding: 4px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
}

.paste-preview {
  position: absolute;
  right: 12px;
  bottom: 12px;
  z-index: 20;
  padding: 4px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 4px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.45);
  cursor: pointer;
  line-height: 0;
  transition: transform 0.1s ease, border-color 0.1s ease;
}

.paste-preview:hover {
  transform: translateY(-1px);
  border-color: var(--accent-cyan);
}

.paste-preview img {
  display: block;
  max-width: 180px;
  max-height: 180px;
  border-radius: 2px;
}

.paste-preview-close {
  position: absolute;
  top: -7px;
  right: -7px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  padding: 0;
  color: var(--text-primary);
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 50%;
  cursor: pointer;
  transition: color 0.1s ease, border-color 0.1s ease;
}

.paste-preview-close:hover {
  color: var(--accent-red);
  border-color: var(--accent-red);
}

.scroll-bottom-pill {
  position: absolute;
  left: 50%;
  bottom: 12px;
  transform: translateX(-50%);
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 12px;
  font-size: 0.65rem;
  font-weight: 600;
  letter-spacing: 0.03em;
  color: var(--bg-primary);
  background: var(--accent-cyan);
  border: none;
  border-radius: 999px;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4);
  cursor: pointer;
  transition: opacity 0.1s ease;
}

.scroll-bottom-pill:hover {
  opacity: 0.9;
}

.paste-confirm-backdrop {
  position: absolute;
  inset: 0;
  z-index: 30;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(2px);
}

.paste-confirm {
  width: min(440px, 90%);
  max-height: 80%;
  display: flex;
  flex-direction: column;
  padding: 14px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 6px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.paste-confirm-title {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.paste-confirm-body {
  flex: 1;
  min-height: 0;
  max-height: 180px;
  overflow: auto;
  margin: 0 0 12px;
  padding: 8px;
  font-size: 0.7rem;
  line-height: 1.4;
  color: var(--text-secondary);
  background: var(--bg-primary);
  border: 1px solid var(--border-subtle);
  border-radius: 4px;
  white-space: pre-wrap;
  word-break: break-word;
}

.paste-confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.paste-confirm-btn {
  padding: 5px 14px;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--text-secondary);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-default);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.1s ease;
}

.paste-confirm-btn:hover {
  color: var(--text-primary);
  border-color: var(--border-strong);
}

.paste-confirm-btn-primary {
  color: var(--bg-primary);
  background: var(--accent-cyan);
  border-color: var(--accent-cyan);
}

.paste-confirm-btn-primary:hover {
  color: var(--bg-primary);
  opacity: 0.9;
}

.paste-preview-enter-active,
.paste-preview-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.paste-preview-enter-from,
.paste-preview-leave-to {
  opacity: 0;
  transform: translateY(6px);
}

.terminal-container {
  width: 100%;
  flex: 1;
  min-height: 0;
}

.pane-header {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 20px;
  min-height: 20px;
  padding: 0 8px;
  margin-bottom: 2px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-subtle);
  overflow: hidden;
}

.pane-header-dir {
  font-size: 0.6rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-secondary);
  white-space: nowrap;
}

.pane-header-proc {
  font-size: 0.6rem;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.terminal-pane.bell-flash {
  animation: bell-blink 1.2s ease-in-out infinite;
}

@keyframes bell-blink {
  0%, 100% {
    box-shadow: inset 0 0 0 2px var(--accent-cyan), inset 0 0 12px rgba(var(--accent-rgb), 0.15);
  }
  50% {
    box-shadow: inset 0 0 0 2px transparent, inset 0 0 0 rgba(var(--accent-rgb), 0);
  }
}

.terminal-pane :deep(.xterm) {
  height: 100%;
}

.terminal-pane :deep(.xterm-viewport) {
  background-color: inherit !important;
}

.terminal-pane :deep(.xterm-screen) {
  background-color: inherit !important;
}

/* Ensure xterm decoration container renders above the text rows */
.terminal-pane :deep(.xterm-decoration-container) {
  z-index: 10 !important;
  pointer-events: none;
}

/* Search result decoration styling */
.terminal-pane :deep(.xterm-find-result-decoration) {
  background-color: rgba(255, 217, 61, 0.15) !important;
  outline: 1px solid rgba(138, 122, 48, 0.6) !important;
}

.terminal-pane :deep(.xterm-find-active-result-decoration) {
  background-color: rgba(255, 217, 61, 0.3) !important;
  outline: 1px solid #ffd93d !important;
}
</style>
