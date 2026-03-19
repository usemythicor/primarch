<script setup lang="ts">
import { computed } from 'vue';
import type { DiffHunk } from '../../types';

const props = defineProps<{
  hunk: DiffHunk;
  mode: 'split' | 'inline';
}>();

// For split mode, pair up old/new lines
const splitLines = computed(() => {
  if (props.mode !== 'split') return [];

  const pairs: Array<{
    oldLine: { lineno: number | null; content: string; type: 'deleted' | 'context' | null } | null;
    newLine: { lineno: number | null; content: string; type: 'added' | 'context' | null } | null;
  }> = [];

  const lines = props.hunk.lines;
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    if (line.origin === ' ') {
      // Context line - same on both sides
      pairs.push({
        oldLine: { lineno: line.oldLineno ?? null, content: line.content, type: 'context' },
        newLine: { lineno: line.newLineno ?? null, content: line.content, type: 'context' },
      });
      i++;
    } else if (line.origin === '-') {
      // Check if next line is an addition (potential change pair)
      const nextLine = lines[i + 1];
      if (nextLine && nextLine.origin === '+') {
        pairs.push({
          oldLine: { lineno: line.oldLineno ?? null, content: line.content, type: 'deleted' },
          newLine: { lineno: nextLine.newLineno ?? null, content: nextLine.content, type: 'added' },
        });
        i += 2;
      } else {
        // Just a deletion
        pairs.push({
          oldLine: { lineno: line.oldLineno ?? null, content: line.content, type: 'deleted' },
          newLine: null,
        });
        i++;
      }
    } else if (line.origin === '+') {
      // Addition without paired deletion
      pairs.push({
        oldLine: null,
        newLine: { lineno: line.newLineno ?? null, content: line.content, type: 'added' },
      });
      i++;
    } else {
      i++;
    }
  }

  return pairs;
});
</script>

<template>
  <div class="diff-hunk">
    <!-- Hunk header -->
    <div class="hunk-header px-4 py-1" style="background: rgba(var(--accent-rgb), 0.1); color: var(--accent-cyan);">
      {{ hunk.header }}
    </div>

    <!-- Inline mode -->
    <div v-if="mode === 'inline'" class="inline-diff">
      <div
        v-for="(line, index) in hunk.lines"
        :key="index"
        class="diff-line flex"
        :class="{
          'line-added': line.origin === '+',
          'line-deleted': line.origin === '-',
          'line-context': line.origin === ' ',
        }"
      >
        <span class="line-number old-lineno">{{ line.oldLineno ?? '' }}</span>
        <span class="line-number new-lineno">{{ line.newLineno ?? '' }}</span>
        <span class="line-origin">{{ line.origin }}</span>
        <span class="line-content">{{ line.content }}</span>
      </div>
    </div>

    <!-- Split mode -->
    <div v-else class="split-diff">
      <div
        v-for="(pair, index) in splitLines"
        :key="index"
        class="split-line flex"
      >
        <!-- Old side -->
        <div
          class="split-side flex flex-1"
          :class="{
            'line-deleted': pair.oldLine?.type === 'deleted',
            'line-context': pair.oldLine?.type === 'context',
            'line-empty': !pair.oldLine,
          }"
        >
          <span class="line-number">{{ pair.oldLine?.lineno ?? '' }}</span>
          <span class="line-origin">{{ pair.oldLine ? (pair.oldLine.type === 'deleted' ? '-' : ' ') : '' }}</span>
          <span class="line-content">{{ pair.oldLine?.content ?? '' }}</span>
        </div>

        <!-- Separator -->
        <div class="split-separator"></div>

        <!-- New side -->
        <div
          class="split-side flex flex-1"
          :class="{
            'line-added': pair.newLine?.type === 'added',
            'line-context': pair.newLine?.type === 'context',
            'line-empty': !pair.newLine,
          }"
        >
          <span class="line-number">{{ pair.newLine?.lineno ?? '' }}</span>
          <span class="line-origin">{{ pair.newLine ? (pair.newLine.type === 'added' ? '+' : ' ') : '' }}</span>
          <span class="line-content">{{ pair.newLine?.content ?? '' }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.diff-hunk {
  border-bottom: 1px solid var(--border-subtle);
}

.hunk-header {
  font-family: var(--font-mono);
  font-size: 0.65rem;
}

/* Line number styling */
.line-number {
  min-width: 40px;
  padding: 0 8px;
  text-align: right;
  color: var(--text-muted);
  user-select: none;
  background: var(--bg-tertiary);
}

.line-origin {
  width: 16px;
  text-align: center;
  user-select: none;
}

.line-content {
  flex: 1;
  padding-left: 4px;
  white-space: pre;
  overflow-x: auto;
}

/* Inline mode line styling */
.diff-line {
  min-height: 1.4em;
}

.inline-diff .old-lineno {
  border-right: 1px solid var(--border-subtle);
}

.inline-diff .new-lineno {
  border-right: 1px solid var(--border-subtle);
}

/* Split mode styling */
.split-line {
  min-height: 1.4em;
}

.split-side {
  overflow: hidden;
}

.split-separator {
  width: 1px;
  background: var(--border-default);
}

/* Line type colors */
.line-added {
  background: rgba(46, 160, 67, 0.15);
}

.line-added .line-origin {
  color: var(--accent-green);
}

.line-deleted {
  background: rgba(248, 81, 73, 0.15);
}

.line-deleted .line-origin {
  color: var(--accent-red);
}

.line-context {
  background: transparent;
}

.line-empty {
  background: var(--bg-tertiary);
}

/* Hover state */
.diff-line:hover,
.split-line:hover .split-side {
  filter: brightness(1.1);
}
</style>
