<script setup lang="ts">
import { computed } from 'vue';
import { CheckIcon } from '@heroicons/vue/24/outline';
import { useGitStore } from '../../stores/git';

const gitStore = useGitStore();

const commitMessage = computed({
  get: () => gitStore.commitMessage,
  set: (value: string) => { gitStore.commitMessage = value; }
});

const canCommit = computed(() => gitStore.canCommit);
const isCommitting = computed(() => gitStore.isCommitting);
const stagedCount = computed(() => gitStore.stagedFiles.length);

async function handleCommit() {
  if (canCommit.value) {
    await gitStore.commit();
  }
}

function handleKeydown(e: KeyboardEvent) {
  // Ctrl+Enter to commit
  if (e.ctrlKey && e.key === 'Enter' && canCommit.value) {
    e.preventDefault();
    handleCommit();
  }
}
</script>

<template>
  <div
    class="commit-panel px-3 py-3"
    style="border-top: 1px solid var(--border-subtle); background: var(--bg-tertiary);"
  >
    <!-- Commit message input -->
    <textarea
      v-model="commitMessage"
      @keydown="handleKeydown"
      placeholder="Commit message (Ctrl+Enter to commit)"
      class="commit-input w-full px-2 py-1.5 resize-none"
      rows="3"
      :disabled="isCommitting"
    ></textarea>

    <!-- Commit button -->
    <div class="flex items-center justify-between mt-2">
      <span class="text-label" style="color: var(--text-muted);">
        {{ stagedCount }} staged
      </span>
      <button
        @click="handleCommit"
        :disabled="!canCommit"
        class="commit-btn flex items-center gap-1.5 px-3 py-1"
        :class="{ 'disabled': !canCommit, 'loading': isCommitting }"
      >
        <CheckIcon class="w-3.5 h-3.5" />
        <span class="text-label">{{ isCommitting ? 'COMMITTING...' : 'COMMIT' }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.commit-input {
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 0.7rem;
  border-radius: 2px;
  outline: none;
}

.commit-input:focus {
  border-color: var(--accent-cyan);
}

.commit-input::placeholder {
  color: var(--text-muted);
}

.commit-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.commit-btn {
  background: rgba(0, 212, 255, 0.1);
  border: 1px solid var(--accent-cyan);
  color: var(--accent-cyan);
  font-weight: 600;
  letter-spacing: 0.05em;
  transition: all 0.15s ease;
  cursor: pointer;
}

.commit-btn:hover:not(.disabled) {
  background: rgba(0, 212, 255, 0.2);
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.2);
}

.commit-btn.disabled {
  opacity: 0.4;
  cursor: not-allowed;
  border-color: var(--border-default);
  color: var(--text-muted);
  background: transparent;
}

.commit-btn.loading {
  opacity: 0.7;
  cursor: wait;
}
</style>
