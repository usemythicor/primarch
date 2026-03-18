<script setup lang="ts">
import { computed } from 'vue';
import {
  CodeBracketIcon,
  ArrowPathIcon,
  XMarkIcon,
  PlusIcon,
  DocumentIcon,
  FolderIcon,
} from '@heroicons/vue/24/outline';
import { useGitStore } from '../../stores/git';
import FileChangeItem from './FileChangeItem.vue';
import CommitPanel from './CommitPanel.vue';
import CommitHistory from './CommitHistory.vue';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const gitStore = useGitStore();

const isLoading = computed(() => gitStore.isLoading);
const stagedFiles = computed(() => gitStore.stagedFiles);
const unstagedFiles = computed(() => gitStore.unstagedFiles);
const untrackedFiles = computed(() => gitStore.untrackedFiles);
const hasChanges = computed(() => gitStore.hasChanges);
const hasRepo = computed(() => gitStore.hasRepo);
const error = computed(() => gitStore.error);
const remoteMessage = computed(() => gitStore.remoteMessage);
const activeTab = computed(() => gitStore.activeTab);

function setTab(tab: 'changes' | 'history') {
  gitStore.setActiveTab(tab);
}

async function refresh() {
  await gitStore.refreshStatus();
}

async function stageAll() {
  await gitStore.stageAll();
}

function handleStage(path: string) {
  gitStore.stageFile(path);
}

function handleUnstage(path: string) {
  gitStore.unstageFile(path);
}

function handleViewDiff(path: string, staged: boolean) {
  gitStore.viewFileDiff(path, staged);
}
</script>

<template>
  <div class="git-sidebar h-full flex flex-col" style="background: var(--bg-secondary);">
    <!-- Header -->
    <div
      class="flex items-center justify-between px-3 py-2"
      style="border-bottom: 1px solid var(--border-subtle);"
    >
      <div class="flex items-center gap-2">
        <CodeBracketIcon class="w-4 h-4" style="color: var(--accent-cyan);" />
        <span class="text-header">SOURCE CONTROL</span>
      </div>
      <div class="flex items-center gap-1">
        <button
          @click="refresh"
          class="p-1 transition-colors"
          :class="isLoading ? 'animate-spin' : ''"
          style="color: var(--text-muted);"
          title="Refresh"
        >
          <ArrowPathIcon class="w-4 h-4" />
        </button>
        <button
          @click="emit('close')"
          class="p-1 transition-colors hover:text-white"
          style="color: var(--text-muted);"
          title="Close"
        >
          <XMarkIcon class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Remote message -->
    <div
      v-if="remoteMessage"
      class="px-3 py-1.5 flex items-center justify-between"
      style="background: rgba(0, 212, 255, 0.1); color: var(--accent-cyan);"
    >
      <span class="text-label">{{ remoteMessage }}</span>
      <button
        @click="gitStore.clearRemoteMessage()"
        class="p-0.5 hover:text-white"
      >
        <XMarkIcon class="w-3 h-3" />
      </button>
    </div>

    <!-- Tabs -->
    <div
      v-if="hasRepo"
      class="flex"
      style="border-bottom: 1px solid var(--border-subtle);"
    >
      <button
        @click="setTab('changes')"
        class="flex-1 px-3 py-1.5 text-label transition-colors"
        :style="{
          color: activeTab === 'changes' ? 'var(--accent-cyan)' : 'var(--text-muted)',
          borderBottom: activeTab === 'changes' ? '2px solid var(--accent-cyan)' : '2px solid transparent',
          background: activeTab === 'changes' ? 'rgba(0, 212, 255, 0.05)' : 'transparent',
        }"
      >
        CHANGES
      </button>
      <button
        @click="setTab('history')"
        class="flex-1 px-3 py-1.5 text-label transition-colors"
        :style="{
          color: activeTab === 'history' ? 'var(--accent-cyan)' : 'var(--text-muted)',
          borderBottom: activeTab === 'history' ? '2px solid var(--accent-cyan)' : '2px solid transparent',
          background: activeTab === 'history' ? 'rgba(0, 212, 255, 0.05)' : 'transparent',
        }"
      >
        HISTORY
      </button>
    </div>

    <!-- Error display -->
    <div
      v-if="error"
      class="px-3 py-2"
      style="background: rgba(255, 71, 87, 0.1); color: var(--accent-red);"
    >
      <span class="text-label">{{ error }}</span>
    </div>

    <!-- No repo message -->
    <div
      v-if="!hasRepo"
      class="flex-1 flex items-center justify-center px-4"
    >
      <div class="text-center">
        <FolderIcon class="w-12 h-12 mx-auto mb-3" style="color: var(--text-muted);" />
        <p class="text-label" style="color: var(--text-muted);">
          No Git repository detected
        </p>
        <p class="text-label mt-1" style="color: var(--text-muted); font-size: 0.6rem;">
          Open a terminal in a Git repository
        </p>
      </div>
    </div>

    <!-- Changes Tab -->
    <div v-if="hasRepo && activeTab === 'changes'" class="flex-1 overflow-y-auto">
      <!-- Staged Changes -->
      <div v-if="stagedFiles.length > 0" class="py-2">
        <div class="flex items-center justify-between px-3 py-1">
          <span class="text-label" style="color: var(--text-muted);">
            STAGED CHANGES
            <span style="color: var(--accent-green);">({{ stagedFiles.length }})</span>
          </span>
        </div>
        <FileChangeItem
          v-for="file in stagedFiles"
          :key="file.path"
          :file="file"
          :staged="true"
          @unstage="handleUnstage"
          @view-diff="handleViewDiff(file.path, true)"
        />
      </div>

      <!-- Unstaged Changes -->
      <div v-if="unstagedFiles.length > 0" class="py-2">
        <div class="flex items-center justify-between px-3 py-1">
          <span class="text-label" style="color: var(--text-muted);">
            CHANGES
            <span style="color: var(--accent-orange);">({{ unstagedFiles.length }})</span>
          </span>
          <button
            @click="stageAll"
            class="p-0.5 transition-colors hover:text-white"
            style="color: var(--text-muted);"
            title="Stage All"
          >
            <PlusIcon class="w-3.5 h-3.5" />
          </button>
        </div>
        <FileChangeItem
          v-for="file in unstagedFiles"
          :key="file.path"
          :file="file"
          :staged="false"
          @stage="handleStage"
          @view-diff="handleViewDiff(file.path, false)"
        />
      </div>

      <!-- Untracked Files -->
      <div v-if="untrackedFiles.length > 0" class="py-2">
        <div class="flex items-center justify-between px-3 py-1">
          <span class="text-label" style="color: var(--text-muted);">
            UNTRACKED
            <span style="color: var(--text-secondary);">({{ untrackedFiles.length }})</span>
          </span>
        </div>
        <div
          v-for="path in untrackedFiles"
          :key="path"
          class="flex items-center gap-2 px-3 py-1 cursor-pointer transition-colors"
          style="color: var(--text-secondary);"
          @click="handleStage(path)"
        >
          <DocumentIcon class="w-3.5 h-3.5 flex-shrink-0" />
          <span class="text-label truncate flex-1">{{ path }}</span>
          <span class="text-label" style="color: var(--accent-green);">U</span>
        </div>
      </div>

      <!-- No changes -->
      <div
        v-if="!hasChanges"
        class="flex items-center justify-center py-8"
      >
        <p class="text-label" style="color: var(--text-muted);">
          No changes
        </p>
      </div>
    </div>

    <!-- History Tab -->
    <CommitHistory v-else-if="hasRepo && activeTab === 'history'" class="flex-1" />

    <!-- Commit Panel (only on changes tab) -->
    <CommitPanel v-if="hasRepo && activeTab === 'changes'" />
  </div>
</template>

<style scoped>
.git-sidebar {
  width: 280px;
  min-width: 200px;
  max-width: 400px;
}
</style>
