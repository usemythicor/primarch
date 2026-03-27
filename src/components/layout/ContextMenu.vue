<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';

export interface MenuItem {
  label: string;
  shortcut?: string;
  icon?: string;
  action: () => void;
  separator?: false;
  disabled?: boolean;
  danger?: boolean;
}

export interface SeparatorItem {
  separator: true;
}

export type ContextMenuItem = MenuItem | SeparatorItem;

const props = defineProps<{
  items: ContextMenuItem[];
  x: number;
  y: number;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const menuRef = ref<HTMLDivElement>();
const adjustedX = ref(props.x);
const adjustedY = ref(props.y);

function handleClick(item: ContextMenuItem) {
  if ('separator' in item && item.separator) return;
  if ('disabled' in item && item.disabled) return;
  (item as MenuItem).action();
  emit('close');
}

function handleClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    emit('close');
  }
}

function handleEscape(e: KeyboardEvent) {
  if (e.code === 'Escape') {
    emit('close');
  }
}

onMounted(async () => {
  await nextTick();
  // Adjust position if menu would overflow viewport
  if (menuRef.value) {
    const rect = menuRef.value.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;

    if (props.x + rect.width > vw) {
      adjustedX.value = vw - rect.width - 4;
    }
    if (props.y + rect.height > vh) {
      adjustedY.value = vh - rect.height - 4;
    }
  }

  document.addEventListener('mousedown', handleClickOutside);
  document.addEventListener('keydown', handleEscape);
  // Close on scroll or window blur
  window.addEventListener('blur', () => emit('close'));
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  document.removeEventListener('keydown', handleEscape);
});
</script>

<template>
  <Teleport to="body">
    <div
      ref="menuRef"
      class="context-menu"
      :style="{ left: adjustedX + 'px', top: adjustedY + 'px' }"
    >
      <template v-for="(item, i) in items" :key="i">
        <div v-if="'separator' in item && item.separator" class="context-separator"></div>
        <button
          v-else
          class="context-item"
          :class="{
            'context-item-disabled': 'disabled' in item && item.disabled,
            'context-item-danger': 'danger' in item && item.danger,
          }"
          @click="handleClick(item)"
        >
          <span class="context-label">{{ (item as MenuItem).label }}</span>
          <span v-if="(item as MenuItem).shortcut" class="context-shortcut">
            {{ (item as MenuItem).shortcut }}
          </span>
        </button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 1000;
  min-width: 180px;
  padding: 4px 0;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  font-family: var(--font-mono);
}

.context-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 6px 12px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.7rem;
  font-family: var(--font-mono);
  cursor: pointer;
  transition: all 0.1s ease;
  text-align: left;
}

.context-item:hover {
  background: rgba(var(--accent-rgb), 0.1);
  color: var(--text-primary);
}

.context-item-danger:hover {
  background: rgba(255, 71, 87, 0.1);
  color: var(--accent-red);
}

.context-item-disabled {
  opacity: 0.4;
  pointer-events: none;
}

.context-label {
  flex: 1;
}

.context-shortcut {
  margin-left: 24px;
  color: var(--text-muted);
  font-size: 0.6rem;
  white-space: nowrap;
}

.context-separator {
  height: 1px;
  margin: 4px 8px;
  background: var(--border-subtle);
}
</style>
