<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { LayoutNode } from '../../types';
import SplitPane from './SplitPane.vue';
import TerminalPane from '../terminal/TerminalPane.vue';
import ContextMenu from './ContextMenu.vue';
import type { ContextMenuItem } from './ContextMenu.vue';
import { useLayoutStore } from '../../stores/layout';
import { useSettingsStore } from '../../stores/settings';

const props = defineProps<{
  node: LayoutNode;
}>();

const layoutStore = useLayoutStore();
const settingsStore = useSettingsStore();
const terminalBg = computed(() => settingsStore.currentTheme.background);

const isSplit = computed(() => props.node.type === 'split');
const isTerminal = computed(() => props.node.type === 'terminal');

// Context menu state
const showContextMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);

const terminalPaneRef = ref<InstanceType<typeof TerminalPane>>();

const contextMenuItems = computed<ContextMenuItem[]>(() => {
  const nodeId = props.node.id;
  if (!nodeId) return [];

  return [
    {
      label: 'Split Right',
      shortcut: 'Ctrl+Shift+D',
      action: () => layoutStore.splitHorizontal(nodeId),
    },
    {
      label: 'Split Down',
      shortcut: 'Ctrl+Shift+E',
      action: () => layoutStore.splitVertical(nodeId),
    },
    { separator: true as const },
    {
      label: 'Copy',
      shortcut: 'Ctrl+C',
      action: () => document.execCommand('copy'),
    },
    {
      label: 'Paste',
      shortcut: 'Ctrl+V',
      action: () => {
        // Trigger paste via the terminal's paste handler
        terminalPaneRef.value?.$el?.dispatchEvent(
          new Event('paste', { bubbles: true })
        );
      },
    },
    { separator: true as const },
    {
      label: 'Clear Terminal',
      action: () => {
        // Send clear command (Ctrl+L equivalent)
        const sessionId = layoutStore.getSessionId(nodeId);
        if (sessionId) {
          import('../../composables/useTerminal').then(({ useTerminal }) => {
            const { write } = useTerminal();
            write(sessionId, '\x0c');
          });
        }
      },
    },
    { separator: true as const },
    {
      label: 'Find',
      shortcut: 'Ctrl+Shift+F',
      action: () => terminalPaneRef.value?.toggleSearch(),
    },
    { separator: true as const },
    {
      label: 'New Tab',
      shortcut: 'Ctrl+T',
      action: () => layoutStore.addTab(),
    },
    {
      label: 'Close Tab',
      danger: layoutStore.tabs.length <= 1,
      action: () => layoutStore.closeTab(layoutStore.activeTabId),
    },
    { separator: true as const },
    {
      label: 'Command Palette',
      shortcut: 'Ctrl+P',
      action: () => window.dispatchEvent(new KeyboardEvent('keydown', {
        code: 'KeyP', key: 'p', ctrlKey: true, bubbles: true,
      })),
    },
    { separator: true as const },
    {
      label: 'Save Output',
      action: () => exportTerminalOutput(),
    },
    {
      label: layoutStore.zoomedPaneId === nodeId ? 'Unzoom Pane' : 'Zoom Pane',
      shortcut: 'Ctrl+Shift+Z',
      action: () => layoutStore.toggleZoom(nodeId),
    },
    {
      label: 'Close Pane',
      shortcut: 'Ctrl+Shift+W',
      danger: true,
      action: () => layoutStore.closePane(nodeId),
    },
  ];
});

async function exportTerminalOutput() {
  const text = terminalPaneRef.value?.getBufferText();
  if (!text) return;
  try {
    const filename = `terminal-${new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)}.log`;
    const path = await invoke<string>('export_terminal_output', { content: text, filename });
    // Open the folder so the user can see the file
    const { revealItemInDir } = await import('@tauri-apps/plugin-opener');
    await revealItemInDir(path);
  } catch { /* ignore */ }
}

function handleContextMenu(e: MouseEvent) {
  e.preventDefault();
  e.stopPropagation();
  // Focus this pane
  if (props.node.id) {
    layoutStore.setActivePane(props.node.id);
  }
  contextMenuX.value = e.clientX;
  contextMenuY.value = e.clientY;
  showContextMenu.value = true;
}

function updateRatio(newRatio: number) {
  if (props.node.id) {
    layoutStore.updateRatio(props.node.id, newRatio);
  }
}

function handleTitleChange(title: string) {
  if (props.node.id) {
    layoutStore.updateTerminalTitle(props.node.id, title);
  }
}

function handleClose() {
  if (props.node.id) {
    layoutStore.closePane(props.node.id);
  }
}

function handleFocus() {
  if (props.node.id) {
    layoutStore.setActivePane(props.node.id);
  }
}

// Listen for export-output events from the command palette
function onExportEvent(e: Event) {
  const detail = (e as CustomEvent).detail;
  if (detail?.nodeId === props.node.id) {
    exportTerminalOutput();
  }
}
onMounted(() => window.addEventListener('primarch-export-output', onExportEvent));
onUnmounted(() => window.removeEventListener('primarch-export-output', onExportEvent));
</script>

<template>
  <!-- Split node - render recursively -->
  <SplitPane
    v-if="isSplit && node.children"
    :direction="node.direction || 'horizontal'"
    :ratio="node.ratio || 0.5"
    @update:ratio="updateRatio"
  >
    <template #first>
      <PaneContainer :node="node.children[0]" />
    </template>
    <template #second>
      <PaneContainer :node="node.children[1]" />
    </template>
  </SplitPane>

  <!-- Terminal node - render terminal -->
  <div
    v-else-if="isTerminal"
    class="terminal-wrapper h-full w-full relative"
    :class="{
      'pane-active': layoutStore.activePane === node.id,
      'pane-zoomed': layoutStore.zoomedPaneId === node.id,
    }"
    :style="{ background: terminalBg }"
    @click="handleFocus"
    @focusin="handleFocus"
    @contextmenu="handleContextMenu"
  >
    <TerminalPane
      ref="terminalPaneRef"
      :key="node.id"
      :node-id="node.id"
      :existing-session-id="node.sessionId"
      :shell="node.shell"
      :cwd="node.cwd"
      :startup-command="node.startupCommand"
      @title-change="handleTitleChange"
      @close="handleClose"
    />

    <ContextMenu
      v-if="showContextMenu"
      :items="contextMenuItems"
      :x="contextMenuX"
      :y="contextMenuY"
      @close="showContextMenu = false"
    />
  </div>
</template>

<style scoped>
.terminal-wrapper {
  outline: none;
}

.terminal-wrapper.pane-active::after {
  content: '';
  position: absolute;
  inset: 0;
  border: 1px solid color-mix(in srgb, var(--accent-cyan) 20%, transparent);
  pointer-events: none;
  z-index: 10;
}

.terminal-wrapper.pane-zoomed {
  position: fixed !important;
  inset: 0;
  z-index: 45;
  width: auto !important;
  height: auto !important;
}
</style>
