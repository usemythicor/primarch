<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { invoke } from '@tauri-apps/api/core';
import { readText, readImage, writeText } from '@tauri-apps/plugin-clipboard-manager';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useTerminal } from '../../composables/useTerminal';
import { useSettingsStore } from '../../stores/settings';
import { useLayoutStore } from '../../stores/layout';
import { useGitStore } from '../../stores/git';
import '@xterm/xterm/css/xterm.css';

const props = defineProps<{
  nodeId?: string;
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
const { createSession, startReading, write, resize, kill } = useTerminal();

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

  const options = settingsStore.terminalOptions;

  // Create xterm instance
  terminal = new Terminal({
    cursorBlink: options.cursorBlink,
    cursorStyle: options.cursorStyle,
    fontSize: options.fontSize,
    fontFamily: options.fontFamily,
    theme: options.theme,
  });

  // Add addons
  fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);
  terminal.loadAddon(new WebLinksAddon((_event, uri) => {
    // Open links in default browser using Tauri's opener plugin
    openUrl(uri).catch((err) => {
      console.error('Failed to open URL:', err);
    });
  }));

  // Mount to DOM
  terminal.open(terminalRef.value);
  fitAddon.fit();

  // Create PTY session
  try {
    sessionId.value = await createSession(props.shell, props.cwd);
    isConnected.value = true;

    // Register session with layout store for cwd tracking
    if (props.nodeId) {
      layoutStore.registerSession(props.nodeId, sessionId.value);
    }

    // Try to detect git repository from CWD
    if (!gitStore.hasRepo) {
      // Use provided cwd or try to get it from the terminal session
      const tryDetectGitRepo = async () => {
        let cwd = props.cwd;

        // If no cwd provided, try to get it from the terminal session
        if (!cwd && sessionId.value) {
          try {
            cwd = await invoke<string>('get_terminal_cwd', { sessionId: sessionId.value });
          } catch {
            // CWD not available yet
          }
        }

        if (cwd) {
          gitStore.openRepository(cwd).catch(() => {
            // Not a git repo or failed to open - this is expected for non-git directories
          });
        }
      };

      // Try immediately and again after a short delay (terminal might need to initialize)
      tryDetectGitRepo();
      setTimeout(tryDetectGitRepo, 1000);
    }

    // Start reading from PTY
    await startReading(
      sessionId.value,
      (data) => {
        terminal?.write(data);
      },
      () => {
        isConnected.value = false;
        emit('close');
      },
      () => {
        // Terminal read error - session may have ended
      }
    );

    // Send terminal size to PTY
    const dimensions = fitAddon.proposeDimensions();
    if (dimensions) {
      await resize(sessionId.value, dimensions.cols, dimensions.rows);
    }

    // Handle user input
    terminal.onData(async (data) => {
      if (sessionId.value) {
        await write(sessionId.value, data);
      }
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
      if (event.code === 'Escape') shouldRedispatch = true;

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

    // Handle clipboard copy (Ctrl+C with selection)
    terminal.attachCustomKeyEventHandler((event) => {
      if (event.type === 'keydown') {
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
          if (['KeyD', 'KeyE', 'KeyW', 'KeyS', 'KeyG', 'Tab'].includes(event.code)) {
            return false;
          }
        }
        // Ctrl+P for command palette
        if (isCtrlOrCmd && !event.shiftKey && event.code === 'KeyP') return false;
        if (isCtrlOrCmd && event.code === 'Comma') return false;
        if (event.ctrlKey && event.code === 'Tab') return false;
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

  // Handle resize
  const resizeObserver = new ResizeObserver(() => {
    requestAnimationFrame(() => {
      if (fitAddon && terminal && sessionId.value) {
        fitAddon.fit();
        // Force terminal to repaint and fill the space
        terminal.refresh(0, terminal.rows - 1);
        const dimensions = fitAddon.proposeDimensions();
        if (dimensions) {
          resize(sessionId.value, dimensions.cols, dimensions.rows);
        }
      }
    });
  });

  resizeObserver.observe(terminalRef.value);

  onUnmounted(() => {
    resizeObserver.disconnect();
  });
});

// Cleanup on unmount
onUnmounted(async () => {
  // Unregister session from layout store
  if (props.nodeId) {
    layoutStore.unregisterSession(props.nodeId);
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

// Expose focus method
defineExpose({ focus });
</script>

<template>
  <div
    class="terminal-pane"
    :style="{ backgroundColor: bgColor }"
  >
    <div ref="terminalRef" class="terminal-container"></div>
  </div>
</template>

<style scoped>
.terminal-pane {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.terminal-container {
  width: 100%;
  height: 100%;
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

.terminal-pane :deep(.xterm-rows) {
  padding: 4px;
}
</style>
