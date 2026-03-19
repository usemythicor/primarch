<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import {
  FolderIcon,
  Cog6ToothIcon,
  CommandLineIcon,
  CodeBracketIcon,
  ArrowPathRoundedSquareIcon,
} from '@heroicons/vue/24/outline';
import PaneContainer from './components/layout/PaneContainer.vue';
import ShellSelector from './components/terminal/ShellSelector.vue';
import WorkspaceManager from './components/workspace/WorkspaceManager.vue';
import SettingsPanel from './components/settings/SettingsPanel.vue';
import GitSidebar from './components/git/GitSidebar.vue';
import DiffViewer from './components/git/DiffViewer.vue';
import { useLayoutStore } from './stores/layout';
import { useSettingsStore } from './stores/settings';
import { useGitStore } from './stores/git';

interface ShellInfo {
  id: string;
  name: string;
  command: string;
  args: string[];
  shell_type: string;
}

const layoutStore = useLayoutStore();
const settingsStore = useSettingsStore();
const gitStore = useGitStore();
const showWorkspaceManager = ref(false);
const showSettings = ref(false);

const terminalCount = computed(() => layoutStore.terminalCount);

// Delayed tooltip
const tooltipText = ref('');
const tooltipVisible = ref(false);
const tooltipX = ref(0);
const tooltipY = ref(0);
let tooltipTimer: ReturnType<typeof setTimeout> | null = null;

function showTooltip(e: MouseEvent, text: string) {
  const target = (e.currentTarget as HTMLElement).getBoundingClientRect();
  tooltipX.value = target.left + target.width / 2;
  tooltipY.value = target.bottom + 6;
  tooltipText.value = text;
  tooltipTimer = setTimeout(() => {
    tooltipVisible.value = true;
  }, 800);
}

function hideTooltip() {
  if (tooltipTimer) clearTimeout(tooltipTimer);
  tooltipTimer = null;
  tooltipVisible.value = false;
}
const terminalBg = computed(() => settingsStore.currentTheme.background);
const showGitSidebar = computed(() => gitStore.sidebarVisible);
const gitChangeCount = computed(() => gitStore.changeCount);
const showDiffViewer = computed(() => gitStore.diffVisible);
const gitBranchName = computed(() => gitStore.branchName);
const gitAhead = computed(() => gitStore.ahead);
const gitBehind = computed(() => gitStore.behind);
const gitHasRepo = computed(() => gitStore.hasRepo);
const gitIsRemoteOperating = computed(() => gitStore.isRemoteOperating);

function handleShellSelect(shell: ShellInfo) {
  layoutStore.splitVertical(undefined, {
    shell: shell.command,
  });
}

function closeModals() {
  showWorkspaceManager.value = false;
  showSettings.value = false;
}

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  // Ctrl+Shift+D: Split vertical
  if (e.ctrlKey && e.shiftKey && e.key === 'D') {
    e.preventDefault();
    layoutStore.splitVertical();
  }
  // Ctrl+Shift+E: Split horizontal
  if (e.ctrlKey && e.shiftKey && e.key === 'E') {
    e.preventDefault();
    layoutStore.splitHorizontal();
  }
  // Ctrl+Shift+W: Close pane
  if (e.ctrlKey && e.shiftKey && e.key === 'W') {
    e.preventDefault();
    if (layoutStore.activePane) {
      layoutStore.closePane(layoutStore.activePane);
    }
  }
  // Ctrl+Tab: Next pane
  if (e.ctrlKey && e.key === 'Tab' && !e.shiftKey) {
    e.preventDefault();
    layoutStore.focusNextPane();
  }
  // Ctrl+Shift+Tab: Previous pane
  if (e.ctrlKey && e.shiftKey && e.key === 'Tab') {
    e.preventDefault();
    layoutStore.focusPreviousPane();
  }
  // Ctrl+Shift+S: Toggle workspace manager
  if (e.ctrlKey && e.shiftKey && e.key === 'S') {
    e.preventDefault();
    showSettings.value = false;
    showWorkspaceManager.value = !showWorkspaceManager.value;
  }
  // Ctrl+,: Toggle settings
  if (e.ctrlKey && e.key === ',') {
    e.preventDefault();
    showWorkspaceManager.value = false;
    showSettings.value = !showSettings.value;
  }
  // Ctrl+Shift+G: Toggle git sidebar
  if (e.ctrlKey && e.shiftKey && e.key === 'G') {
    e.preventDefault();
    gitStore.toggleSidebar();
  }
  // Escape: Close modals and diff viewer
  if (e.key === 'Escape') {
    if (gitStore.diffVisible) {
      gitStore.closeDiff();
    } else {
      closeModals();
      gitStore.hideSidebar();
    }
  }
}

// Sync body background with terminal theme
watch(terminalBg, (bg) => {
  document.documentElement.style.background = bg;
  document.body.style.background = bg;
}, { immediate: true });

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
  // Set initial background
  document.documentElement.style.background = terminalBg.value;
  document.body.style.background = terminalBg.value;
  // Start watching for CWD changes to update git
  gitStore.startCwdWatcher();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
  gitStore.stopCwdWatcher();
});
</script>

<template>
  <div class="h-screen w-screen flex flex-col" :style="{ background: terminalBg }">
    <!-- Title bar -->
    <div
      class="flex items-center justify-between px-4 h-10 select-none"
      style="background: var(--bg-primary);"
    >
      <!-- Left side - Logo and title -->
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <div class="w-1 h-4 rounded-sm" style="background: var(--accent-cyan);"></div>
          <span class="text-header" style="color: var(--text-primary);">MYTHICOR</span>
        </div>
        <div class="flex items-center gap-2" style="color: var(--text-muted);">
          <CommandLineIcon class="w-3.5 h-3.5" />
          <span class="text-label">{{ terminalCount }} ACTIVE</span>
        </div>
      </div>

      <!-- Right side - Actions -->
      <div class="flex items-center gap-2">
        <button
          @click="showWorkspaceManager = !showWorkspaceManager; showSettings = false"
          class="btn-toolbar px-3 py-1.5"
          :class="{ 'btn-toolbar-active': showWorkspaceManager }"
          title="Workspaces (Ctrl+Shift+S)"
        >
          <FolderIcon class="w-3.5 h-3.5" />
          <span class="text-label">Workspaces</span>
        </button>

        <div class="w-px h-4" style="background: var(--border-default);"></div>

        <!-- Split buttons -->
        <div class="tooltip-wrapper relative" @mouseenter="showTooltip($event, 'Split Down  Ctrl+Shift+E')" @mouseleave="hideTooltip">
          <button
            @click="layoutStore.splitVertical()"
            class="btn-toolbar"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="1.5" y="1.5" width="13" height="13" rx="1.5" /><line x1="1.5" y1="8" x2="14.5" y2="8" /></svg>
          </button>
        </div>

        <div class="tooltip-wrapper relative" @mouseenter="showTooltip($event, 'Split Right  Ctrl+Shift+D')" @mouseleave="hideTooltip">
          <button
            @click="layoutStore.splitHorizontal()"
            class="btn-toolbar"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="1.5" y="1.5" width="13" height="13" rx="1.5" /><line x1="8" y1="1.5" x2="8" y2="14.5" /></svg>
          </button>
        </div>

        <ShellSelector @select="handleShellSelect" />

        <button
          @click="showSettings = !showSettings; showWorkspaceManager = false"
          class="btn-icon"
          :class="{ 'btn-toolbar-active': showSettings }"
          title="Settings (Ctrl+,)"
        >
          <Cog6ToothIcon class="w-4 h-4" />
        </button>

        <div class="w-px h-4" style="background: var(--border-default);"></div>

        <span class="text-label" style="color: var(--text-muted);">v0.1.0</span>
      </div>
    </div>

    <!-- Layout area -->
    <div class="flex-1 overflow-hidden relative min-h-0 flex" :style="{ background: terminalBg }">
      <!-- Git Sidebar -->
      <Transition
        enter-active-class="transition-all duration-200 ease-out"
        enter-from-class="w-0 opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition-all duration-150 ease-in"
        leave-from-class="opacity-100"
        leave-to-class="w-0 opacity-0"
      >
        <GitSidebar
          v-if="showGitSidebar"
          @close="gitStore.hideSidebar()"
        />
      </Transition>

      <!-- Terminal Panes -->
      <div class="flex-1 min-w-0">
        <PaneContainer :node="layoutStore.rootLayout" />
      </div>

      <!-- Modals -->
      <Teleport to="body">
        <!-- Workspace Manager Modal -->
        <Transition
          enter-active-class="transition duration-150 ease-out"
          enter-from-class="opacity-0"
          enter-to-class="opacity-100"
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <div
            v-if="showWorkspaceManager"
            class="fixed inset-0 flex items-center justify-center z-50"
          >
            <div
              class="absolute inset-0"
              style="background: rgba(0, 0, 0, 0.8); backdrop-filter: blur(4px);"
              @click="showWorkspaceManager = false"
            ></div>
            <WorkspaceManager class="relative z-10" @close="showWorkspaceManager = false" />
          </div>
        </Transition>

        <!-- Settings Modal -->
        <Transition
          enter-active-class="transition duration-150 ease-out"
          enter-from-class="opacity-0"
          enter-to-class="opacity-100"
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <div
            v-if="showSettings"
            class="fixed inset-0 flex items-center justify-center z-50"
          >
            <div
              class="absolute inset-0"
              style="background: rgba(0, 0, 0, 0.8); backdrop-filter: blur(4px);"
              @click="showSettings = false"
            ></div>
            <SettingsPanel class="relative z-10" @close="showSettings = false" />
          </div>
        </Transition>

        <!-- Diff Viewer Modal -->
        <Transition
          enter-active-class="transition duration-150 ease-out"
          enter-from-class="opacity-0 scale-95"
          enter-to-class="opacity-100 scale-100"
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100 scale-100"
          leave-to-class="opacity-0 scale-95"
        >
          <div
            v-if="showDiffViewer"
            class="fixed inset-0 flex items-center justify-center z-50"
          >
            <div
              class="absolute inset-0"
              style="background: rgba(0, 0, 0, 0.8); backdrop-filter: blur(4px);"
              @click="gitStore.closeDiff()"
            ></div>
            <div class="relative z-10 w-4/5 h-4/5 max-w-6xl rounded overflow-hidden" style="border: 1px solid var(--border-default);">
              <DiffViewer />
            </div>
          </div>
        </Transition>
      </Teleport>
    </div>

    <!-- Status bar -->
    <div
      class="flex items-center justify-between px-4 h-6 select-none"
      style="background: var(--bg-primary);"
    >
      <div class="flex items-center gap-4">
        <!-- Git branch info -->
        <div
          class="flex items-center gap-2 cursor-pointer hover:opacity-80"
          @click="gitStore.toggleSidebar()"
          title="Source Control (Ctrl+Shift+G)"
        >
          <CodeBracketIcon class="w-3 h-3" style="color: var(--text-muted);" />
          <template v-if="gitHasRepo">
            <span class="text-label" style="color: var(--text-secondary);">{{ gitBranchName || 'No branch' }}</span>
            <span v-if="gitAhead > 0 || gitBehind > 0" class="flex items-center gap-1">
              <span v-if="gitBehind > 0" class="text-label" style="color: var(--accent-orange);">{{ gitBehind }}↓</span>
              <span v-if="gitAhead > 0" class="text-label" style="color: var(--accent-green);">{{ gitAhead }}↑</span>
            </span>
            <span
              v-if="gitChangeCount > 0"
              class="px-1 rounded text-label"
              style="background: var(--accent-cyan); color: var(--bg-primary); font-size: 0.55rem;"
            >
              {{ gitChangeCount }}
            </span>
            <button
              @click.stop="gitStore.sync()"
              :disabled="gitIsRemoteOperating"
              class="p-0.5 transition-colors hover:opacity-100"
              :class="{ 'animate-spin': gitIsRemoteOperating }"
              style="color: var(--text-muted);"
              title="Sync"
            >
              <ArrowPathRoundedSquareIcon class="w-3 h-3" />
            </button>
          </template>
          <span v-else class="text-label" style="color: var(--text-muted);">No Repository</span>
        </div>

      </div>
    </div>

    <!-- Delayed tooltip -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition duration-100 ease-out"
        enter-from-class="opacity-0 translate-y-[-2px]"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition duration-75 ease-in"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div
          v-if="tooltipVisible"
          class="fixed z-[100] px-2.5 py-1.5 pointer-events-none whitespace-nowrap text-label"
          :style="{
            left: tooltipX + 'px',
            top: tooltipY + 'px',
            transform: 'translateX(-50%)',
            background: 'var(--bg-elevated)',
            border: '1px solid var(--border-default)',
            color: 'var(--text-secondary)',
          }"
        >
          {{ tooltipText }}
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
