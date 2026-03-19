<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue';
import { useSettingsStore } from '../../stores/settings';

const settingsStore = useSettingsStore();
const terminalBg = computed(() => settingsStore.currentTheme.background);

const props = defineProps<{
  direction: 'horizontal' | 'vertical';
  ratio: number;
  minSize?: number;
}>();

const emit = defineEmits<{
  (e: 'update:ratio', ratio: number): void;
}>();

const containerRef = ref<HTMLDivElement>();
const isDragging = ref(false);
const isHovered = ref(false);
const minSizePercent = props.minSize ?? 10;

const firstPaneStyle = computed(() => {
  const size = props.ratio * 100;
  return props.direction === 'horizontal'
    ? { width: `${size}%` }
    : { height: `${size}%` };
});

const secondPaneStyle = computed(() => {
  const size = (1 - props.ratio) * 100;
  return props.direction === 'horizontal'
    ? { width: `${size}%` }
    : { height: `${size}%` };
});

const dividerStyle = computed(() => {
  const isActive = isDragging.value || isHovered.value;
  return {
    background: isActive ? 'var(--accent-cyan)' : 'var(--border-subtle)',
    boxShadow: isActive ? '0 0 10px rgba(var(--accent-rgb), 0.3)' : 'none',
  };
});

function startDrag(e: MouseEvent) {
  e.preventDefault();
  isDragging.value = true;
  document.addEventListener('mousemove', onDrag);
  document.addEventListener('mouseup', stopDrag);
  document.body.style.cursor =
    props.direction === 'horizontal' ? 'col-resize' : 'row-resize';
  document.body.style.userSelect = 'none';
}

function onDrag(e: MouseEvent) {
  if (!isDragging.value || !containerRef.value) return;

  const rect = containerRef.value.getBoundingClientRect();
  let newRatio: number;

  if (props.direction === 'horizontal') {
    newRatio = (e.clientX - rect.left) / rect.width;
  } else {
    newRatio = (e.clientY - rect.top) / rect.height;
  }

  // Clamp ratio to min/max
  const minRatio = minSizePercent / 100;
  const maxRatio = 1 - minRatio;
  newRatio = Math.max(minRatio, Math.min(maxRatio, newRatio));

  emit('update:ratio', newRatio);
}

function stopDrag() {
  isDragging.value = false;
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
  document.body.style.cursor = '';
  document.body.style.userSelect = '';
}

onUnmounted(() => {
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
});
</script>

<template>
  <div
    ref="containerRef"
    class="split-pane h-full w-full flex"
    :class="direction === 'horizontal' ? 'flex-row' : 'flex-col'"
    :style="{ background: terminalBg }"
  >
    <!-- First pane -->
    <div class="pane overflow-hidden" :style="{ ...firstPaneStyle, background: terminalBg }">
      <slot name="first"></slot>
    </div>

    <!-- Divider -->
    <div
      class="divider flex-shrink-0 transition-all duration-150"
      :class="direction === 'horizontal' ? 'w-px cursor-col-resize' : 'h-px cursor-row-resize'"
      :style="dividerStyle"
      @mousedown="startDrag"
      @mouseenter="isHovered = true"
      @mouseleave="isHovered = false"
    >
      <!-- Hover hit area (invisible but larger) -->
      <div
        class="absolute"
        :class="direction === 'horizontal' ? 'inset-y-0 -left-1 -right-1 w-3' : 'inset-x-0 -top-1 -bottom-1 h-3'"
      ></div>
    </div>

    <!-- Second pane -->
    <div class="pane overflow-hidden" :style="{ ...secondPaneStyle, background: terminalBg }">
      <slot name="second"></slot>
    </div>
  </div>
</template>

<style scoped>
.split-pane {
  position: relative;
}

.pane {
  position: relative;
}

.divider {
  z-index: 10;
  position: relative;
}
</style>
