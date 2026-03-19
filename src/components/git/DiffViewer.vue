<script setup lang="ts">
import { computed } from 'vue';
import {
  XMarkIcon,
  ArrowsRightLeftIcon,
  Bars3BottomLeftIcon,
} from '@heroicons/vue/24/outline';
import { useGitStore } from '../../stores/git';
import DiffHunk from './DiffHunk.vue';

const gitStore = useGitStore();

const diff = computed(() => gitStore.currentDiff);
const isLoading = computed(() => gitStore.diffLoading);
const mode = computed(() => gitStore.diffMode);
const path = computed(() => gitStore.diffPath);
const isStaged = computed(() => gitStore.diffStaged);

const fileName = computed(() => {
  if (!path.value) return '';
  const parts = path.value.split('/');
  return parts[parts.length - 1];
});

const directory = computed(() => {
  if (!path.value) return '';
  const parts = path.value.split('/');
  if (parts.length > 1) {
    return parts.slice(0, -1).join('/') + '/';
  }
  return '';
});

function close() {
  gitStore.closeDiff();
}

function toggleMode() {
  gitStore.toggleDiffMode();
}
</script>

<template>
  <div class="diff-viewer flex flex-col h-full" style="background: var(--bg-secondary);">
    <!-- Header -->
    <div
      class="flex items-center justify-between px-4 py-2"
      style="border-bottom: 1px solid var(--border-subtle); background: var(--bg-primary);"
    >
      <div class="flex items-center gap-3">
        <span class="text-label" :style="{ color: isStaged ? 'var(--accent-green)' : 'var(--accent-orange)' }">
          {{ isStaged ? 'STAGED' : 'UNSTAGED' }}
        </span>
        <span class="text-label" style="color: var(--text-primary);">{{ fileName }}</span>
        <span v-if="directory" class="text-label" style="color: var(--text-muted); font-size: 0.6rem;">
          {{ directory }}
        </span>
      </div>

      <div class="flex items-center gap-2">
        <!-- Stats -->
        <div v-if="diff" class="flex items-center gap-2 mr-4">
          <span class="text-label" style="color: var(--accent-green);">
            +{{ diff.additions }}
          </span>
          <span class="text-label" style="color: var(--accent-red);">
            -{{ diff.deletions }}
          </span>
        </div>

        <!-- Mode toggle -->
        <button
          @click="toggleMode"
          class="btn-icon"
          :title="mode === 'split' ? 'Switch to inline mode' : 'Switch to split mode'"
        >
          <ArrowsRightLeftIcon v-if="mode === 'split'" class="w-4 h-4" />
          <Bars3BottomLeftIcon v-else class="w-4 h-4" />
        </button>

        <!-- Close button -->
        <button
          @click="close"
          class="btn-icon btn-icon-danger"
          title="Close"
        >
          <XMarkIcon class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-auto">
      <!-- Loading state -->
      <div v-if="isLoading" class="flex items-center justify-center h-full">
        <span class="text-label" style="color: var(--text-muted);">Loading diff...</span>
      </div>

      <!-- No diff -->
      <div v-else-if="!diff" class="flex items-center justify-center h-full">
        <span class="text-label" style="color: var(--text-muted);">No diff available</span>
      </div>

      <!-- Binary file -->
      <div v-else-if="diff.isBinary" class="flex items-center justify-center h-full">
        <span class="text-label" style="color: var(--text-muted);">Binary file - cannot display diff</span>
      </div>

      <!-- No changes -->
      <div v-else-if="diff.hunks.length === 0" class="flex items-center justify-center h-full">
        <span class="text-label" style="color: var(--text-muted);">No changes in this file</span>
      </div>

      <!-- Diff hunks -->
      <div v-else class="diff-content">
        <DiffHunk
          v-for="(hunk, index) in diff.hunks"
          :key="index"
          :hunk="hunk"
          :mode="mode"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.diff-viewer {
  min-width: 400px;
}

.diff-content {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  line-height: 1.5;
}
</style>
