<script setup lang="ts">
import { ref, computed } from 'vue';
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
      label: 'Close Pane',
      shortcut: 'Ctrl+Shift+W',
      danger: true,
      action: () => layoutStore.closePane(nodeId),
    },
  ];
});

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
    :class="{ 'pane-active': layoutStore.activePane === node.id }"
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
</style>
