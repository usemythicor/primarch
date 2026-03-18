<script setup lang="ts">
import { computed } from 'vue';
import type { LayoutNode } from '../../types';
import SplitPane from './SplitPane.vue';
import TerminalPane from '../terminal/TerminalPane.vue';
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
    :style="{
      background: terminalBg,
      boxShadow: layoutStore.activePane === node.id ? 'inset 0 0 0 1px var(--accent-cyan)' : 'none',
    }"
    @click="handleFocus"
    @focusin="handleFocus"
  >
    <TerminalPane
      :node-id="node.id"
      :shell="node.shell"
      :cwd="node.cwd"
      :startup-command="node.startupCommand"
      @title-change="handleTitleChange"
      @close="handleClose"
    />
  </div>
</template>

<style scoped>
.terminal-wrapper {
  outline: none;
}
</style>
