<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useGitStore } from '../../stores/git';

const gitStore = useGitStore();

const commits = computed(() => gitStore.commits);
const isLoading = computed(() => gitStore.historyLoading);
const hasMore = computed(() => gitStore.historyHasMore);
const selectedCommit = computed(() => gitStore.selectedCommit);

const listRef = ref<HTMLElement | null>(null);

onMounted(() => {
  gitStore.loadHistory(true);
});

function handleScroll(e: Event) {
  const target = e.target as HTMLElement;
  const threshold = 100;
  const nearBottom = target.scrollHeight - target.scrollTop - target.clientHeight < threshold;

  if (nearBottom && hasMore.value && !isLoading.value) {
    gitStore.loadHistory();
  }
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    return 'Today';
  } else if (diffDays === 1) {
    return 'Yesterday';
  } else if (diffDays < 7) {
    return `${diffDays} days ago`;
  } else if (diffDays < 30) {
    const weeks = Math.floor(diffDays / 7);
    return `${weeks} week${weeks > 1 ? 's' : ''} ago`;
  } else {
    return date.toLocaleDateString();
  }
}

function getRefColor(refType: string): string {
  switch (refType) {
    case 'Branch':
      return 'var(--accent-green)';
    case 'RemoteBranch':
      return 'var(--accent-cyan)';
    case 'Tag':
      return 'var(--accent-yellow)';
    default:
      return 'var(--text-muted)';
  }
}
</script>

<template>
  <div class="commit-history h-full flex flex-col">
    <!-- Commit list -->
    <div
      ref="listRef"
      class="flex-1 overflow-y-auto"
      @scroll="handleScroll"
    >
      <div
        v-for="commit in commits"
        :key="commit.oid"
        class="commit-item px-3 py-2 cursor-pointer transition-colors"
        :class="{ 'selected': selectedCommit?.oid === commit.oid }"
        @click="gitStore.selectCommit(commit.oid)"
      >
        <!-- Refs (branches, tags) -->
        <div v-if="commit.refs.length > 0" class="flex flex-wrap gap-1 mb-1">
          <span
            v-for="ref in commit.refs"
            :key="ref.name"
            class="ref-badge px-1.5 py-0.5 text-label"
            :style="{
              background: `${getRefColor(ref.refType)}20`,
              color: getRefColor(ref.refType),
              border: ref.isHead ? `1px solid ${getRefColor(ref.refType)}` : 'none',
            }"
          >
            {{ ref.name }}
          </span>
        </div>

        <!-- Commit info -->
        <div class="flex items-start gap-2">
          <!-- Graph line indicator -->
          <div class="graph-line mt-1.5">
            <div class="graph-dot"></div>
          </div>

          <div class="flex-1 min-w-0">
            <!-- Summary -->
            <div class="text-label truncate" style="color: var(--text-primary);">
              {{ commit.summary }}
            </div>

            <!-- Meta -->
            <div class="flex items-center gap-2 mt-0.5">
              <span class="text-label" style="color: var(--accent-cyan); font-family: var(--font-mono);">
                {{ commit.shortId }}
              </span>
              <span class="text-label" style="color: var(--text-muted);">
                {{ commit.authorName }}
              </span>
              <span class="text-label" style="color: var(--text-muted);">
                {{ formatDate(commit.timestamp) }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Loading indicator -->
      <div
        v-if="isLoading"
        class="flex items-center justify-center py-4"
      >
        <span class="text-label" style="color: var(--text-muted);">Loading...</span>
      </div>

      <!-- End of history -->
      <div
        v-if="!hasMore && commits.length > 0"
        class="flex items-center justify-center py-4"
      >
        <span class="text-label" style="color: var(--text-muted);">End of history</span>
      </div>

      <!-- No commits -->
      <div
        v-if="!isLoading && commits.length === 0"
        class="flex items-center justify-center py-8"
      >
        <span class="text-label" style="color: var(--text-muted);">No commits</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.commit-item {
  border-bottom: 1px solid var(--border-subtle);
}

.commit-item:hover {
  background: var(--bg-hover);
}

.commit-item.selected {
  background: rgba(0, 212, 255, 0.1);
  border-left: 2px solid var(--accent-cyan);
}

.ref-badge {
  font-size: 0.55rem;
  font-weight: 600;
  border-radius: 2px;
}

.graph-line {
  width: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
}

.graph-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-cyan);
  border: 2px solid var(--bg-secondary);
  position: relative;
  z-index: 1;
}

.commit-item:not(:last-child) .graph-line::after {
  content: '';
  position: absolute;
  top: 10px;
  left: 50%;
  transform: translateX(-50%);
  width: 2px;
  height: calc(100% + 8px);
  background: var(--border-default);
}
</style>
