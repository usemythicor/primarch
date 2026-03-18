<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { useTerminal } from '../../composables/useTerminal';
import { useSettingsStore } from '../../stores/settings';
import { useLayoutStore } from '../../stores/layout';
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
const terminalRef = ref<HTMLDivElement>();
const sessionId = ref<string>();
const isConnected = ref(false);

let terminal: Terminal | null = null;
let fitAddon: FitAddon | null = null;
const { createSession, startReading, write, resize, kill } = useTerminal();

const bgColor = computed(() => settingsStore.currentTheme.background);

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
  terminal.loadAddon(new WebLinksAddon());

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
      (error) => {
        console.error('Terminal error:', error);
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

    // Run startup command if provided
    if (props.startupCommand) {
      await write(sessionId.value, props.startupCommand + '\r');
    }
  } catch (error) {
    console.error('Failed to create terminal session:', error);
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
