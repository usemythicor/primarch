<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import {
  Squares2X2Icon,
  FolderIcon,
  Cog6ToothIcon,
  CommandLineIcon,
} from '@heroicons/vue/24/outline';
import PaneContainer from './components/layout/PaneContainer.vue';
import ShellSelector from './components/terminal/ShellSelector.vue';
import WorkspaceManager from './components/workspace/WorkspaceManager.vue';
import SettingsPanel from './components/settings/SettingsPanel.vue';
import { useLayoutStore } from './stores/layout';
import { useSettingsStore } from './stores/settings';

interface ShellInfo {
  id: string;
  name: string;
  command: string;
  args: string[];
  shell_type: string;
}

const layoutStore = useLayoutStore();
const settingsStore = useSettingsStore();
const showWorkspaceManager = ref(false);
const showSettings = ref(false);

const terminalCount = computed(() => layoutStore.terminalCount);
const currentThemeName = computed(() => settingsStore.currentTheme.name);
const terminalBg = computed(() => settingsStore.currentTheme.background);

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
  // Escape: Close modals
  if (e.key === 'Escape') {
    closeModals();
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
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
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
          class="flex items-center gap-2 px-3 py-1.5 transition-all duration-150"
          :class="showWorkspaceManager ? 'text-cyan-400' : ''"
          :style="{
            background: showWorkspaceManager ? 'rgba(0, 212, 255, 0.1)' : 'transparent',
            border: showWorkspaceManager ? '1px solid var(--accent-cyan)' : '1px solid var(--border-default)',
            color: showWorkspaceManager ? 'var(--accent-cyan)' : 'var(--text-secondary)',
          }"
          title="Workspaces (Ctrl+Shift+S)"
        >
          <FolderIcon class="w-3.5 h-3.5" />
          <span class="text-label">Workspaces</span>
        </button>

        <ShellSelector @select="handleShellSelect" />

        <button
          @click="showSettings = !showSettings; showWorkspaceManager = false"
          class="p-1.5 transition-all duration-150"
          :style="{
            background: showSettings ? 'rgba(0, 212, 255, 0.1)' : 'transparent',
            border: showSettings ? '1px solid var(--accent-cyan)' : '1px solid transparent',
            color: showSettings ? 'var(--accent-cyan)' : 'var(--text-muted)',
          }"
          title="Settings (Ctrl+,)"
        >
          <Cog6ToothIcon class="w-4 h-4" />
        </button>

        <div class="w-px h-4" style="background: var(--border-default);"></div>

        <span class="text-label" style="color: var(--text-muted);">v0.1.0</span>
      </div>
    </div>

    <!-- Layout area -->
    <div class="flex-1 overflow-hidden relative min-h-0" :style="{ background: terminalBg }">
      <PaneContainer :node="layoutStore.rootLayout" />

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
      </Teleport>
    </div>

    <!-- Status bar -->
    <div
      class="flex items-center justify-between px-4 h-6 select-none"
      style="background: var(--bg-primary);"
    >
      <div class="flex items-center gap-6">
        <div class="flex items-center gap-2">
          <Squares2X2Icon class="w-3 h-3" style="color: var(--text-muted);" />
          <span class="text-label" style="color: var(--text-muted);">Ctrl+Shift+D Split</span>
        </div>
        <span class="text-label" style="color: var(--text-muted);">Ctrl+Shift+W Close</span>
      </div>
      <div class="flex items-center gap-6">
        <div class="flex items-center gap-2">
          <div class="w-1.5 h-1.5 rounded-full" style="background: var(--accent-green);"></div>
          <span class="text-label" style="color: var(--text-secondary);">{{ currentThemeName }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
