<script setup lang="ts">
import { ref, nextTick } from 'vue';
import { PlusIcon, XMarkIcon } from '@heroicons/vue/24/outline';
import { useLayoutStore } from '../../stores/layout';

const layoutStore = useLayoutStore();

const editingTabId = ref<string | null>(null);
const editInput = ref<HTMLInputElement>();
const editValue = ref('');

// Drag state
const dragTabId = ref<string | null>(null);
const dragOverTabId = ref<string | null>(null);

function startRename(tabId: string, currentName: string) {
  editingTabId.value = tabId;
  editValue.value = currentName;
  nextTick(() => {
    editInput.value?.focus();
    editInput.value?.select();
  });
}

function commitRename() {
  if (editingTabId.value && editValue.value.trim()) {
    layoutStore.renameTab(editingTabId.value, editValue.value.trim());
  }
  editingTabId.value = null;
}

function cancelRename() {
  editingTabId.value = null;
}

function handleMiddleClick(e: MouseEvent, tabId: string) {
  if (e.button === 1) {
    e.preventDefault();
    layoutStore.closeTab(tabId);
  }
}

function handleDragStart(e: DragEvent, tabId: string) {
  dragTabId.value = tabId;
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move';
  }
}

function handleDragOver(e: DragEvent, tabId: string) {
  e.preventDefault();
  dragOverTabId.value = tabId;
}

function handleDrop(_e: DragEvent, tabId: string) {
  if (dragTabId.value && dragTabId.value !== tabId) {
    const fromIndex = layoutStore.tabs.findIndex((t) => t.id === dragTabId.value);
    const toIndex = layoutStore.tabs.findIndex((t) => t.id === tabId);
    layoutStore.moveTab(fromIndex, toIndex);
  }
  dragTabId.value = null;
  dragOverTabId.value = null;
}

function handleDragEnd() {
  dragTabId.value = null;
  dragOverTabId.value = null;
}
</script>

<template>
  <div class="tab-bar">
    <div class="tab-list">
      <div
        v-for="tab in layoutStore.tabs"
        :key="tab.id"
        class="tab"
        :class="{
          'tab-active': tab.id === layoutStore.activeTabId,
          'tab-drag-over': tab.id === dragOverTabId && tab.id !== dragTabId,
        }"
        draggable="true"
        @click="layoutStore.setActiveTab(tab.id)"
        @mousedown="handleMiddleClick($event, tab.id)"
        @dblclick.stop="startRename(tab.id, tab.name)"
        @dragstart="handleDragStart($event, tab.id)"
        @dragover="handleDragOver($event, tab.id)"
        @drop="handleDrop($event, tab.id)"
        @dragend="handleDragEnd"
      >
        <input
          v-if="editingTabId === tab.id"
          ref="editInput"
          v-model="editValue"
          class="tab-rename-input"
          @blur="commitRename"
          @keydown.enter="commitRename"
          @keydown.escape="cancelRename"
          @click.stop
        />
        <span v-else class="tab-name">{{ tab.name }}</span>
        <button
          class="tab-close"
          @click.stop="layoutStore.closeTab(tab.id)"
          title="Close tab"
        >
          <XMarkIcon class="w-3 h-3" />
        </button>
      </div>
    </div>
    <button
      class="tab-add"
      @click="layoutStore.addTab()"
      title="New tab (Ctrl+T)"
    >
      <PlusIcon class="w-3.5 h-3.5" />
    </button>
  </div>
</template>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  height: 32px;
  min-height: 32px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-subtle);
  padding: 0 4px;
  gap: 2px;
  overflow: hidden;
}

.tab-list {
  display: flex;
  align-items: center;
  gap: 2px;
  overflow-x: auto;
  flex: 1;
  min-width: 0;
}

.tab-list::-webkit-scrollbar {
  display: none;
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 8px 0 12px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 0;
  cursor: pointer;
  transition: all 0.1s ease;
  flex-shrink: 0;
  max-width: 180px;
  min-width: 0;
}

.tab:hover {
  background: var(--bg-hover);
}

.tab-active {
  background: var(--bg-tertiary);
  border-color: var(--border-default);
  border-bottom-color: var(--bg-tertiary);
}

.tab-active .tab-name {
  color: var(--text-primary);
}

.tab-drag-over {
  border-left: 2px solid var(--accent-cyan);
}

.tab-name {
  font-size: 0.65rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  user-select: none;
}

.tab-rename-input {
  font-size: 0.65rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-primary);
  background: var(--bg-secondary);
  border: 1px solid var(--accent-cyan);
  outline: none;
  padding: 1px 4px;
  width: 100px;
  font-family: inherit;
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2px;
  color: var(--text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  opacity: 0;
  transition: all 0.1s ease;
  flex-shrink: 0;
}

.tab:hover .tab-close,
.tab-active .tab-close {
  opacity: 1;
}

.tab-close:hover {
  color: var(--accent-red);
}

.tab-add {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.1s ease;
  flex-shrink: 0;
}

.tab-add:hover {
  color: var(--accent-cyan);
  border-color: var(--border-default);
  background: var(--bg-hover);
}
</style>
