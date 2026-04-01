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
import { openUrl } from '@tauri-apps/plugin-opener';
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

// Handle clipboard paste
async function handlePaste() {
  if (!sessionId.value) return;

  try {
    // Try to read text first
    const text = await readText();
    if (text) {
      await write(sessionId.value, text);
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
    }
  } catch (e) {
    // No image or failed to save, ignore
    console.error('Failed to paste image:', e);
  }
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
  const onPtyData = (data: string) => {
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
      sessionId.value = await createSession(props.shell, props.cwd);
      await startReading(sessionId.value, onPtyData, onPtyClose, onPtyError);
    }

    isConnected.value = true;

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
      emit('title-change', title);
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
defineExpose({ focus, toggleSearch });
</script>

<template>
  <div
    class="terminal-pane"
    :class="{ searching: showSearch }"
    :style="{ backgroundColor: bgColor }"
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
    <div ref="terminalRef" class="terminal-container"></div>
  </div>
</template>

<style scoped>
.terminal-pane {
  width: 100%;
  height: 100%;
  overflow: hidden;
  padding: 4px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
}

.terminal-container {
  width: 100%;
  flex: 1;
  min-height: 0;
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
