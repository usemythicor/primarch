<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { CheckIcon, SparklesIcon, ChevronDownIcon } from '@heroicons/vue/24/outline';
import { useGitStore } from '../../stores/git';
import { useSettingsStore } from '../../stores/settings';

const gitStore = useGitStore();
const settingsStore = useSettingsStore();

// Commit mode
const COMMIT_MODE_KEY = 'primarch-commit-mode';
type CommitMode = 'commit' | 'commit-push' | 'commit-sync' | 'amend';

const commitMode = ref<CommitMode>('commit');
const showModeDropdown = ref(false);

const commitModes: { mode: CommitMode; label: string; description: string }[] = [
  { mode: 'commit', label: 'Commit', description: 'Commit staged changes' },
  { mode: 'commit-push', label: 'Commit & Push', description: 'Commit and push to remote' },
  { mode: 'commit-sync', label: 'Commit & Sync', description: 'Commit, pull, then push' },
  { mode: 'amend', label: 'Amend', description: 'Replace the last commit' },
];

onMounted(() => {
  const saved = localStorage.getItem(COMMIT_MODE_KEY);
  if (commitModes.some(m => m.mode === saved)) {
    commitMode.value = saved as CommitMode;
  }
});

function setCommitMode(mode: CommitMode) {
  commitMode.value = mode;
  localStorage.setItem(COMMIT_MODE_KEY, mode);
  showModeDropdown.value = false;
}

const commitButtonLabel = computed(() => {
  if (isCommitting.value) {
    if (commitMode.value === 'amend') return 'AMENDING...';
    return 'COMMITTING...';
  }
  const found = commitModes.find(m => m.mode === commitMode.value);
  return found ? found.label.toUpperCase() : 'COMMIT';
});

// Resizable panel logic
const STORAGE_KEY = 'primarch-commit-panel-height';
const MIN_HEIGHT = 150;
const MAX_HEIGHT = 400;
const DEFAULT_HEIGHT = 150;

const panelHeight = ref(DEFAULT_HEIGHT);
const isResizing = ref(false);
const startY = ref(0);
const startHeight = ref(0);

onMounted(() => {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    const parsed = parseInt(saved, 10);
    if (!isNaN(parsed) && parsed >= MIN_HEIGHT && parsed <= MAX_HEIGHT) {
      panelHeight.value = parsed;
    }
  }
});

function startResize(e: MouseEvent) {
  isResizing.value = true;
  startY.value = e.clientY;
  startHeight.value = panelHeight.value;
  document.addEventListener('mousemove', onResize);
  document.addEventListener('mouseup', stopResize);
  document.body.style.cursor = 'ns-resize';
  document.body.style.userSelect = 'none';
}

function onResize(e: MouseEvent) {
  if (!isResizing.value) return;
  const delta = startY.value - e.clientY;
  const newHeight = Math.min(MAX_HEIGHT, Math.max(MIN_HEIGHT, startHeight.value + delta));
  panelHeight.value = newHeight;
}

function stopResize() {
  if (!isResizing.value) return;
  isResizing.value = false;
  document.removeEventListener('mousemove', onResize);
  document.removeEventListener('mouseup', stopResize);
  document.body.style.cursor = '';
  document.body.style.userSelect = '';
  localStorage.setItem(STORAGE_KEY, panelHeight.value.toString());
}

onUnmounted(() => {
  document.removeEventListener('mousemove', onResize);
  document.removeEventListener('mouseup', stopResize);
});

const isGenerating = ref(false);
const aiProvider = computed(() => settingsStore.aiProvider);

const canGenerate = computed(() => {
  if (gitStore.stagedFiles.length === 0 || isGenerating.value) return false;
  if (aiProvider.value === 'none') return false;
  if (aiProvider.value === 'api') return !!settingsStore.anthropicApiKey;
  // CLI providers (claude, codex) are always available if selected
  return true;
});

const generateButtonTitle = computed(() => {
  if (aiProvider.value === 'none') return 'Enable AI in Settings';
  if (aiProvider.value === 'api' && !settingsStore.anthropicApiKey) return 'Set Anthropic API key in Settings';
  if (gitStore.stagedFiles.length === 0) return 'Stage files first';
  return 'Generate commit message with AI';
});

async function generateCommitMessage() {
  if (!canGenerate.value || !gitStore.repoId) return;
  isGenerating.value = true;
  try {
    let message: string;

    if (aiProvider.value === 'api') {
      // Use Anthropic API
      message = await invoke<string>('generate_commit_message', {
        repoId: gitStore.repoId,
        apiKey: settingsStore.anthropicApiKey,
      });
    } else {
      // Use CLI (claude or codex)
      message = await invoke<string>('generate_commit_message_cli', {
        repoId: gitStore.repoId,
        cli: aiProvider.value,
      });
    }

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

const canCommit = computed(() => {
  if (commitMode.value === 'amend') {
    return !!gitStore.commitMessage.trim() && !gitStore.isCommitting;
  }
  return gitStore.canCommit;
});
const isCommitting = computed(() => gitStore.isCommitting);
const stagedCount = computed(() => gitStore.stagedFiles.length);

async function handleCommit() {
  if (commitMode.value === 'amend') {
    if (!gitStore.commitMessage.trim()) return;
    await gitStore.amend();
    return;
  }

  if (!canCommit.value) return;
  await gitStore.commit();
  if (gitStore.error) return;

  if (commitMode.value === 'commit-push') {
    await gitStore.push();
  } else if (commitMode.value === 'commit-sync') {
    await gitStore.sync();
  }
}

function handleKeydown(e: KeyboardEvent) {
  // Ctrl+Enter to commit
  if (e.ctrlKey && e.key === 'Enter' && canCommit.value) {
    e.preventDefault();
    handleCommit();
  }
}

// Close dropdown on click outside
function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest('.commit-dropdown-area')) {
    showModeDropdown.value = false;
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
});
</script>

<template>
  <div
    class="commit-panel px-3 py-3"
    :style="{ height: panelHeight + 'px', background: 'var(--bg-tertiary)' }"
  >
    <!-- Resize handle -->
    <div
      class="resize-handle"
      @mousedown.prevent="startResize"
    ></div>
    <!-- Header with AI generate -->
    <div class="flex items-center justify-between mb-1">
      <span class="text-label" style="color: var(--text-muted);">Message</span>
      <button
        @click="generateCommitMessage"
        :disabled="!canGenerate"
        class="generate-btn flex items-center gap-1 px-1.5 py-0.5"
        :class="{ 'disabled': !canGenerate, 'loading': isGenerating }"
        :title="generateButtonTitle"
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
      class="commit-input w-full px-2 py-1.5 resize-none flex-1"
      :disabled="isCommitting"
    ></textarea>

    <!-- Commit button row -->
    <div class="flex items-center justify-between mt-2">
      <span class="text-label" style="color: var(--text-muted);">
        {{ stagedCount }} staged
      </span>

      <!-- Split button: main action + dropdown toggle -->
      <div class="commit-dropdown-area relative">
        <div class="commit-split-btn flex items-center" :class="{ 'disabled': !canCommit, 'loading': isCommitting }">
          <button
            @click="handleCommit"
            :disabled="!canCommit"
            class="commit-btn-main flex items-center gap-1.5 px-3 py-1"
          >
            <CheckIcon class="w-3.5 h-3.5" />
            <span class="text-label">{{ commitButtonLabel }}</span>
          </button>
          <button
            @click="showModeDropdown = !showModeDropdown"
            :disabled="!canCommit && !showModeDropdown"
            class="commit-btn-dropdown flex items-center px-1.5 py-1"
          >
            <ChevronDownIcon class="w-3 h-3" />
          </button>
        </div>

        <!-- Dropdown menu -->
        <Transition
          enter-active-class="transition duration-100 ease-out"
          enter-from-class="opacity-0 -translate-y-1"
          enter-to-class="opacity-100 translate-y-0"
          leave-active-class="transition duration-75 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <div v-if="showModeDropdown" class="commit-dropdown">
            <button
              v-for="m in commitModes"
              :key="m.mode"
              class="dropdown-item"
              :class="{ active: commitMode === m.mode }"
              :title="m.description"
              @click="setCommitMode(m.mode)"
            >
              <span class="text-label">{{ m.label }}</span>
              <CheckIcon v-if="commitMode === m.mode" class="w-3 h-3" style="color: var(--accent-cyan);" />
            </button>
          </div>
        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.commit-panel {
  position: relative;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  border-top: 1px solid var(--border-subtle);
}

.resize-handle {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  cursor: ns-resize;
  background: transparent;
  transition: background 0.15s ease;
}

.resize-handle:hover,
.resize-handle:active {
  background: var(--accent-cyan);
}

.commit-input {
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 0.7rem;
  border-radius: 2px;
  outline: none;
  min-height: 40px;
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

/* Split button container */
.commit-split-btn {
  border: 1px solid var(--accent-cyan);
  background: rgba(var(--accent-rgb), 0.1);
  transition: all 0.15s ease;
}

.commit-split-btn.disabled {
  opacity: 0.4;
  border-color: var(--border-default);
  background: transparent;
}

.commit-split-btn.loading {
  opacity: 0.7;
  cursor: wait;
}

/* Main commit button (left side) */
.commit-btn-main {
  background: transparent;
  border: none;
  color: var(--accent-cyan);
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease;
}

.commit-btn-main:hover:not(:disabled) {
  background: rgba(var(--accent-rgb), 0.1);
}

.commit-btn-main:disabled {
  cursor: not-allowed;
  color: var(--text-muted);
}

/* Dropdown toggle (right side) */
.commit-btn-dropdown {
  background: transparent;
  border: none;
  border-left: 1px solid rgba(var(--accent-rgb), 0.3);
  color: var(--accent-cyan);
  cursor: pointer;
  transition: background 0.15s ease;
}

.commit-btn-dropdown:hover:not(:disabled) {
  background: rgba(var(--accent-rgb), 0.15);
}

.commit-btn-dropdown:disabled {
  cursor: not-allowed;
  color: var(--text-muted);
}

.commit-split-btn.disabled .commit-btn-dropdown {
  border-left-color: var(--border-default);
}

.commit-split-btn:not(.disabled):hover {
  box-shadow: 0 0 10px rgba(var(--accent-rgb), 0.2);
}

/* Dropdown menu */
.commit-dropdown {
  position: absolute;
  bottom: calc(100% + 4px);
  right: 0;
  min-width: 160px;
  padding: 4px 0;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  box-shadow: 0 -4px 16px rgba(0, 0, 0, 0.4);
  z-index: 50;
}

.dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 6px 12px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  cursor: pointer;
  transition: all 0.1s ease;
  text-align: left;
}

.dropdown-item:hover {
  background: rgba(var(--accent-rgb), 0.1);
  color: var(--text-primary);
}

.dropdown-item.active {
  color: var(--accent-cyan);
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
