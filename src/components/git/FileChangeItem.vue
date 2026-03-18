<script setup lang="ts">
import { computed } from 'vue';
import {
  DocumentIcon,
  PlusIcon,
  MinusIcon,
} from '@heroicons/vue/24/outline';
import type { FileStatus } from '../../types';

const props = defineProps<{
  file: FileStatus;
  staged: boolean;
}>();

const emit = defineEmits<{
  (e: 'stage', path: string): void;
  (e: 'unstage', path: string): void;
  (e: 'view-diff'): void;
}>();

const fileName = computed(() => {
  const parts = props.file.path.split('/');
  return parts[parts.length - 1];
});

const directory = computed(() => {
  const parts = props.file.path.split('/');
  if (parts.length > 1) {
    return parts.slice(0, -1).join('/') + '/';
  }
  return '';
});

const statusBadge = computed(() => {
  switch (props.file.status) {
    case 'Modified':
      return { text: 'M', color: 'var(--accent-orange)' };
    case 'Added':
      return { text: 'A', color: 'var(--accent-green)' };
    case 'Deleted':
      return { text: 'D', color: 'var(--accent-red)' };
    case 'Renamed':
      return { text: 'R', color: 'var(--accent-cyan)' };
    case 'Copied':
      return { text: 'C', color: 'var(--accent-cyan)' };
    case 'TypeChanged':
      return { text: 'T', color: 'var(--accent-yellow)' };
    default:
      return { text: '?', color: 'var(--text-muted)' };
  }
});

function handleAction() {
  if (props.staged) {
    emit('unstage', props.file.path);
  } else {
    emit('stage', props.file.path);
  }
}
</script>

<template>
  <div
    class="file-change-item flex items-center gap-2 px-3 py-1 cursor-pointer transition-colors group"
    @click="emit('view-diff')"
  >
    <DocumentIcon class="w-3.5 h-3.5 flex-shrink-0" style="color: var(--text-muted);" />

    <div class="flex-1 min-w-0 flex items-baseline gap-1">
      <span class="text-label truncate" style="color: var(--text-primary);">
        {{ fileName }}
      </span>
      <span v-if="directory" class="text-label truncate" style="color: var(--text-muted); font-size: 0.55rem;">
        {{ directory }}
      </span>
    </div>

    <!-- Action button (stage/unstage) -->
    <button
      @click.stop="handleAction"
      class="action-btn p-0.5 transition-colors opacity-0 group-hover:opacity-100"
      :title="staged ? 'Unstage' : 'Stage'"
    >
      <MinusIcon v-if="staged" class="w-3.5 h-3.5" style="color: var(--accent-red);" />
      <PlusIcon v-else class="w-3.5 h-3.5" style="color: var(--accent-green);" />
    </button>

    <!-- Status badge -->
    <span
      class="text-label font-bold"
      :style="{ color: statusBadge.color }"
    >
      {{ statusBadge.text }}
    </span>
  </div>
</template>

<style scoped>
.file-change-item:hover {
  background: var(--bg-hover);
}

.action-btn:hover {
  background: var(--bg-elevated);
  border-radius: 2px;
}
</style>
