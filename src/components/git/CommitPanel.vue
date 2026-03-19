<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { CheckIcon, SparklesIcon } from '@heroicons/vue/24/outline';
import { useGitStore } from '../../stores/git';
import { useSettingsStore } from '../../stores/settings';

const gitStore = useGitStore();
const settingsStore = useSettingsStore();

const isGenerating = ref(false);
const canGenerate = computed(
  () => settingsStore.anthropicApiKey && gitStore.stagedFiles.length > 0 && !isGenerating.value
);

async function generateCommitMessage() {
  if (!canGenerate.value || !gitStore.repoId) return;
  isGenerating.value = true;
  try {
    const message = await invoke<string>('generate_commit_message', {
      repoId: gitStore.repoId,
      apiKey: settingsStore.anthropicApiKey,
    });
    gitStore.commitMessage = message;
  } catch (e) {
    gitStore.commitMessage = `Error: ${e}`;
  } finally {
    isGenerating.value = false;
  }
}

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
    <!-- Header with AI generate -->
    <div class="flex items-center justify-between mb-1">
      <span class="text-label" style="color: var(--text-muted);">Message</span>
      <button
        @click="generateCommitMessage"
        :disabled="!canGenerate"
        class="generate-btn flex items-center gap-1 px-1.5 py-0.5"
        :class="{ 'disabled': !canGenerate, 'loading': isGenerating }"
        :title="settingsStore.anthropicApiKey ? 'Generate commit message with AI' : 'Set Anthropic API key in Settings'"
      >
        <SparklesIcon class="w-3 h-3" :class="{ 'animate-pulse': isGenerating }" />
        <span class="text-label" style="letter-spacing: 0.05em;">{{ isGenerating ? 'GENERATING...' : 'AI' }}</span>
      </button>
    </div>

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
  background: rgba(var(--accent-rgb), 0.1);
  border: 1px solid var(--accent-cyan);
  color: var(--accent-cyan);
  font-weight: 600;
  letter-spacing: 0.05em;
  transition: all 0.15s ease;
  cursor: pointer;
}

.commit-btn:hover:not(.disabled) {
  background: rgba(var(--accent-rgb), 0.2);
  box-shadow: 0 0 10px rgba(var(--accent-rgb), 0.2);
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

.generate-btn {
  background: transparent;
  border: 1px solid var(--border-subtle);
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.generate-btn:hover:not(.disabled) {
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.generate-btn.disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.generate-btn.loading {
  opacity: 0.7;
  cursor: wait;
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}
</style>
