<script setup lang="ts">
import { computed } from 'vue';
import { ref } from 'vue';
import {
  CodeBracketIcon,
  ArrowPathIcon,
  XMarkIcon,
  PlusIcon,
  DocumentIcon,
  FolderIcon,
  ChevronDownIcon,
  ArrowUturnLeftIcon,
  TrashIcon,
  CloudArrowDownIcon,
  CloudArrowUpIcon,
} from '@heroicons/vue/24/outline';
import { useGitStore } from '../../stores/git';
import FileChangeItem from './FileChangeItem.vue';
import CommitPanel from './CommitPanel.vue';
import CommitHistory from './CommitHistory.vue';
import BranchSelector from './BranchSelector.vue';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const gitStore = useGitStore();

const stagedFiles = computed(() => gitStore.stagedFiles);
const unstagedFiles = computed(() => gitStore.unstagedFiles);
const untrackedFiles = computed(() => gitStore.untrackedFiles);
const hasChanges = computed(() => gitStore.hasChanges);
const hasRepo = computed(() => gitStore.hasRepo);
const error = computed(() => gitStore.error);
const remoteMessage = computed(() => gitStore.remoteMessage);
const activeTab = computed(() => gitStore.activeTab);
const branchName = computed(() => gitStore.branchName);
const branchSelectorVisible = computed(() => gitStore.branchSelectorVisible);
const isDiscarding = computed(() => gitStore.isDiscarding);
const ahead = computed(() => gitStore.ahead);
const behind = computed(() => gitStore.behind);
const isRemoteOperating = computed(() => gitStore.isRemoteOperating);
const needsPublish = computed(() => gitStore.needsPublish);

const showActionsMenu = ref(false);

function toggleActionsMenu() {
  showActionsMenu.value = !showActionsMenu.value;
}

function closeActionsMenu() {
  showActionsMenu.value = false;
}

async function handlePull() {
  closeActionsMenu();
  await gitStore.pull();
}

async function handlePush() {
  closeActionsMenu();
  if (needsPublish.value) {
    await gitStore.publish();
  } else {
    await gitStore.push();
  }
}

async function handleSync() {
  closeActionsMenu();
  await gitStore.sync();
}

function handleRefresh() {
  closeActionsMenu();
  refresh();
}

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

function handleDiscard(path: string) {
  if (confirm(`Discard changes to "${path}"?`)) {
    gitStore.discardFile(path);
  }
}

async function handleDiscardAll() {
  if (confirm('Discard all unstaged changes? This cannot be undone.')) {
    await gitStore.discardAll();
  }
}

async function handleCleanUntracked() {
  if (confirm('Delete all untracked files? This cannot be undone.')) {
    await gitStore.cleanUntracked();
  }
}

function showBranchSelector() {
  gitStore.showBranchSelector();
}

function hideBranchSelector() {
  gitStore.hideBranchSelector();
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
        <!-- Actions dropdown -->
        <div class="relative">
          <button
            @click="toggleActionsMenu"
            class="btn-icon"
            :class="{ 'btn-toolbar-active': showActionsMenu }"
          >
            <ChevronDownIcon class="w-4 h-4" />
          </button>

          <Transition
            enter-active-class="transition duration-100 ease-out"
            enter-from-class="opacity-0 -translate-y-1"
            enter-to-class="opacity-100 translate-y-0"
            leave-active-class="transition duration-75 ease-in"
            leave-from-class="opacity-100 translate-y-0"
            leave-to-class="opacity-0 -translate-y-1"
          >
            <div
              v-if="showActionsMenu"
              class="absolute right-0 top-full mt-1 w-48 z-50 py-1"
              style="background: var(--bg-secondary); border: 1px solid var(--border-default);"
            >
              <button
                @click="handlePull"
                :disabled="isRemoteOperating"
                class="git-menu-item w-full flex items-center gap-3 px-3 py-2 text-left transition-colors"
              >
                <CloudArrowDownIcon class="w-4 h-4 flex-shrink-0" :style="{ color: behind > 0 ? 'var(--accent-orange)' : 'var(--text-muted)' }" />
                <span class="text-label flex-1" style="color: var(--text-secondary);">Pull</span>
                <span v-if="behind > 0" class="text-label" style="color: var(--accent-orange);">{{ behind }}</span>
              </button>
              <button
                @click="handlePush"
                :disabled="isRemoteOperating"
                class="git-menu-item w-full flex items-center gap-3 px-3 py-2 text-left transition-colors"
              >
                <CloudArrowUpIcon class="w-4 h-4 flex-shrink-0" :style="{ color: (ahead > 0 || needsPublish) ? 'var(--accent-green)' : 'var(--text-muted)' }" />
                <span class="text-label flex-1" style="color: var(--text-secondary);">{{ needsPublish ? 'Publish Branch' : 'Push' }}</span>
                <span v-if="ahead > 0 && !needsPublish" class="text-label" style="color: var(--accent-green);">{{ ahead }}</span>
              </button>
              <div class="my-1" style="border-top: 1px solid var(--border-subtle);"></div>
              <button
                @click="handleSync"
                :disabled="isRemoteOperating"
                class="git-menu-item w-full flex items-center gap-3 px-3 py-2 text-left transition-colors"
              >
                <ArrowPathIcon class="w-4 h-4 flex-shrink-0" style="color: var(--text-muted);" />
                <span class="text-label" style="color: var(--text-secondary);">Sync</span>
              </button>
              <button
                @click="handleRefresh"
                class="git-menu-item w-full flex items-center gap-3 px-3 py-2 text-left transition-colors"
              >
                <ArrowPathIcon class="w-4 h-4 flex-shrink-0" style="color: var(--text-muted);" />
                <span class="text-label" style="color: var(--text-secondary);">Refresh</span>
              </button>
            </div>
          </Transition>

          <!-- Backdrop -->
          <div
            v-if="showActionsMenu"
            class="fixed inset-0 z-40"
            @click="closeActionsMenu"
          ></div>
        </div>

        <button
          @click="emit('close')"
          class="btn-icon btn-icon-danger"
          title="Close"
        >
          <XMarkIcon class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Branch Selector Button -->
    <button
      v-if="hasRepo && branchName"
      @click="showBranchSelector"
      class="flex items-center gap-1 px-3 py-1.5 w-full text-left transition-colors hover:bg-[var(--bg-hover)]"
      style="border-bottom: 1px solid var(--border-subtle);"
    >
      <CodeBracketIcon class="w-3.5 h-3.5" style="color: var(--accent-green);" />
      <span class="text-sm flex-1 truncate" style="color: var(--text-primary);">
        {{ branchName }}
      </span>
      <ChevronDownIcon class="w-3.5 h-3.5" style="color: var(--text-muted);" />
    </button>

    <!-- Remote message -->
    <div
      v-if="remoteMessage"
      class="px-3 py-1.5 flex items-center justify-between"
      style="background: rgba(var(--accent-rgb), 0.1); color: var(--accent-cyan);"
    >
      <span class="text-label">{{ remoteMessage }}</span>
      <button
        @click="gitStore.clearRemoteMessage()"
        class="btn-icon btn-icon-danger p-0.5"
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
          background: activeTab === 'changes' ? 'rgba(var(--accent-rgb), 0.05)' : 'transparent',
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
          background: activeTab === 'history' ? 'rgba(var(--accent-rgb), 0.05)' : 'transparent',
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
          <div class="flex items-center gap-1">
            <button
              @click="handleDiscardAll"
              :disabled="isDiscarding"
              class="btn-icon btn-icon-danger p-0.5"
              title="Discard All Changes"
            >
              <ArrowUturnLeftIcon class="w-3.5 h-3.5" />
            </button>
            <button
              @click="stageAll"
              class="btn-icon p-0.5"
              title="Stage All"
            >
              <PlusIcon class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
        <FileChangeItem
          v-for="file in unstagedFiles"
          :key="file.path"
          :file="file"
          :staged="false"
          @stage="handleStage"
          @discard="handleDiscard"
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
          <div class="flex items-center gap-1">
            <button
              @click="handleCleanUntracked"
              :disabled="isDiscarding"
              class="btn-icon btn-icon-danger p-0.5"
              title="Delete All Untracked"
            >
              <TrashIcon class="w-3.5 h-3.5" />
            </button>
            <button
              @click="untrackedFiles.forEach(p => handleStage(p))"
              class="btn-icon p-0.5"
              title="Stage All Untracked"
            >
              <PlusIcon class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
        <div
          v-for="path in untrackedFiles"
          :key="path"
          class="untracked-item flex items-center gap-2 px-3 py-1 cursor-pointer transition-colors group"
          style="color: var(--text-secondary);"
        >
          <DocumentIcon class="w-3.5 h-3.5 flex-shrink-0" />
          <span class="text-label truncate flex-1" @click="handleStage(path)">{{ path }}</span>
          <button
            @click.stop="handleDiscard(path)"
            class="p-0.5 transition-opacity opacity-0 group-hover:opacity-100"
            style="color: var(--accent-red);"
            title="Delete File"
          >
            <TrashIcon class="w-3 h-3" />
          </button>
          <button
            @click.stop="handleStage(path)"
            class="p-0.5 transition-opacity opacity-0 group-hover:opacity-100"
            style="color: var(--accent-green);"
            title="Stage"
          >
            <PlusIcon class="w-3 h-3" />
          </button>
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

    <!-- Branch Selector Modal -->
    <BranchSelector
      v-if="branchSelectorVisible"
      @close="hideBranchSelector"
    />

  </div>
</template>

<style scoped>
.git-sidebar {
  width: 280px;
  min-width: 200px;
  max-width: 400px;
}

.untracked-item:hover {
  background: var(--bg-hover);
}

.git-sidebar button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.git-menu-item:hover:not(:disabled) {
  background: var(--bg-hover);
}

.git-menu-item:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
