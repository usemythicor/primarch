<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getVersion } from '@tauri-apps/api/app';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  FolderIcon,
  Cog6ToothIcon,
  CommandLineIcon,
  CodeBracketIcon,
  CloudArrowUpIcon,
  CloudArrowDownIcon,
  ArrowDownTrayIcon,
  MinusIcon,
  StopIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline';
import PaneContainer from './components/layout/PaneContainer.vue';
import TabBar from './components/layout/TabBar.vue';
import ShellSelector from './components/terminal/ShellSelector.vue';
import LayoutPresetPicker from './components/layout/LayoutPresetPicker.vue';
import WorkspaceManager from './components/workspace/WorkspaceManager.vue';
import SettingsPanel from './components/settings/SettingsPanel.vue';
import CommandPalette from './components/palette/CommandPalette.vue';
import GitSidebar from './components/git/GitSidebar.vue';
import DiffViewer from './components/git/DiffViewer.vue';
import MarkdownViewer from './components/viewer/MarkdownViewer.vue';
import { useLayoutStore } from './stores/layout';
import { createTerminalNode } from './components/layout/LayoutTree';
import { useSettingsStore } from './stores/settings';
import { useGitStore } from './stores/git';
import { useUpdater } from './composables/useUpdater';

// Window controls
const appWindow = getCurrentWindow();
const isMaximized = ref(false);
const isMacOS = navigator.platform.startsWith('Mac');

function minimizeWindow() {
  appWindow.minimize();
}

function toggleMaximize() {
  appWindow.toggleMaximize();
}

function closeWindow() {
  appWindow.close();
}

// Start dragging the window
function startDrag() {
  appWindow.startDragging();
}

// Track maximize state
async function updateMaximizedState() {
  isMaximized.value = await appWindow.isMaximized();
}

// Window state persistence
const WINDOW_STATE_KEY = 'primarch-window-state';
let saveWindowStateTimer: ReturnType<typeof setTimeout> | null = null;

async function saveWindowState() {
  try {
    const maximized = await appWindow.isMaximized();
    if (!maximized) {
      const pos = await appWindow.outerPosition();
      const size = await appWindow.outerSize();
      localStorage.setItem(WINDOW_STATE_KEY, JSON.stringify({
        x: pos.x, y: pos.y,
        width: size.width, height: size.height,
        maximized: false,
      }));
    } else {
      // Only update the maximized flag, keep last known position/size
      const prev = JSON.parse(localStorage.getItem(WINDOW_STATE_KEY) || '{}');
      localStorage.setItem(WINDOW_STATE_KEY, JSON.stringify({ ...prev, maximized: true }));
    }
  } catch { /* ignore */ }
}

function debouncedSaveWindowState() {
  if (saveWindowStateTimer) clearTimeout(saveWindowStateTimer);
  saveWindowStateTimer = setTimeout(saveWindowState, 500);
}

async function restoreWindowState() {
  try {
    const raw = localStorage.getItem(WINDOW_STATE_KEY);
    if (!raw) return;
    const state = JSON.parse(raw);
    if (state.width && state.height) {
      const { LogicalSize, LogicalPosition } = await import('@tauri-apps/api/dpi');
      await appWindow.setSize(new LogicalSize(state.width, state.height));
      if (state.x != null && state.y != null) {
        await appWindow.setPosition(new LogicalPosition(state.x, state.y));
      }
    }
    if (state.maximized) {
      await appWindow.maximize();
    }
  } catch { /* ignore — use defaults */ }
}

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
const { updateAvailable, updateInfo, isDownloading, requiresManualUpdate, checkForUpdates, downloadAndInstall, openReleasesPage } = useUpdater();
const showWorkspaceManager = ref(false);
const showSettings = ref(false);
const showCommandPalette = ref(false);
const showMarkdownViewer = ref(false);
const markdownSource = ref<string>('');
const appVersion = ref('0.0.0');
let globalShortcutUnlisten: UnlistenFn | null = null;

const terminalCount = computed(() => layoutStore.terminalCount);
let openDirUnlisten: UnlistenFn | null = null;

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

// Status bar notification
const statusNotification = ref<string | null>(null);
let notificationTimer: ReturnType<typeof setTimeout> | null = null;

function showStatusNotification(message: string, duration = 5000) {
  if (notificationTimer) clearTimeout(notificationTimer);
  statusNotification.value = message;
  notificationTimer = setTimeout(() => {
    statusNotification.value = null;
  }, duration);
}

// Watch for AI CLI detection
watch(() => settingsStore.availableAiClis, (clis) => {
  if (clis.length > 0) {
    const names = clis.map(c => c === 'claude' ? 'Claude Max' : c === 'codex' ? 'Codex' : c).join(', ');
    showStatusNotification(`${names} detected - AI commit messages enabled`);
  }
}, { immediate: true });
const gitChangeCount = computed(() => gitStore.changeCount);
const showDiffViewer = computed(() => gitStore.diffVisible);
const gitBranchName = computed(() => gitStore.branchName);
const gitAhead = computed(() => gitStore.ahead);
const gitBehind = computed(() => gitStore.behind);
const gitHasRepo = computed(() => gitStore.hasRepo);
const gitIsRemoteOperating = computed(() => gitStore.isRemoteOperating);
const gitNeedsPublish = computed(() => gitStore.needsPublish);
async function handleStatusPull() {
  if (gitIsRemoteOperating.value) return;
  await gitStore.pull();
}

async function handleStatusPush() {
  if (gitIsRemoteOperating.value) return;
  if (gitNeedsPublish.value) {
    await gitStore.publish();
  } else {
    await gitStore.push();
  }
}

function openMarkdownViewer(source: string) {
  markdownSource.value = source;
  showMarkdownViewer.value = true;
}

function handleShellSelect(shell: ShellInfo) {
  layoutStore.splitVertical(undefined, {
    shell: shell.command,
  });
}

function closeModals() {
  showWorkspaceManager.value = false;
  showSettings.value = false;
  showCommandPalette.value = false;
}

// Keyboard shortcuts (capture phase — fires once before any pane handlers)
function handleKeydown(e: KeyboardEvent) {
  let handled = false;
  // Treat Cmd (macOS) and Ctrl (Windows/Linux) as the same modifier so the
  // documented shortcut table works natively on both platforms.
  const mod = e.ctrlKey || e.metaKey;

  // Cmd/Ctrl+P: Toggle command palette (backup for when global shortcut doesn't fire)
  if (mod && !e.shiftKey && e.code === 'KeyP') {
    if (!showCommandPalette.value) {
      closeModals();
      showCommandPalette.value = true;
    } else {
      showCommandPalette.value = false;
    }
    handled = true;
  }
  // Cmd/Ctrl+T: New tab
  else if (mod && !e.shiftKey && e.code === 'KeyT') {
    layoutStore.addTab();
    handled = true;
  }
  // Cmd/Ctrl+W: Close tab (when only one pane in tab) or close pane
  else if (mod && !e.shiftKey && e.code === 'KeyW') {
    if (layoutStore.activeTabTerminalCount <= 1) {
      layoutStore.closeTab(layoutStore.activeTabId);
    } else if (layoutStore.activePane) {
      layoutStore.closePane(layoutStore.activePane);
    }
    handled = true;
  }
  // Cmd/Ctrl+PageDown: Next tab
  else if (mod && e.code === 'PageDown') {
    layoutStore.nextTab();
    handled = true;
  }
  // Cmd/Ctrl+PageUp: Previous tab
  else if (mod && e.code === 'PageUp') {
    layoutStore.previousTab();
    handled = true;
  }
  // Cmd/Ctrl+1-9: Switch to tab by index
  else if (mod && !e.shiftKey && e.code.match(/^Digit[1-9]$/)) {
    const index = parseInt(e.code.replace('Digit', '')) - 1;
    layoutStore.switchToTab(index);
    handled = true;
  }
  // Cmd/Ctrl+Shift+E: Split down (vertical split)
  else if (mod && e.shiftKey && e.code === 'KeyE') {
    layoutStore.splitVertical();
    handled = true;
  }
  // Cmd/Ctrl+Shift+D: Split right (horizontal split)
  else if (mod && e.shiftKey && e.code === 'KeyD') {
    layoutStore.splitHorizontal();
    handled = true;
  }
  // Cmd/Ctrl+Shift+W: Close pane
  else if (mod && e.shiftKey && e.code === 'KeyW') {
    if (layoutStore.activePane) {
      layoutStore.closePane(layoutStore.activePane);
    }
    handled = true;
  }
  // Cmd/Ctrl+Tab: Next pane
  else if (mod && e.code === 'Tab' && !e.shiftKey) {
    layoutStore.focusNextPane();
    handled = true;
  }
  // Cmd/Ctrl+Shift+Tab: Previous pane
  else if (mod && e.shiftKey && e.code === 'Tab') {
    layoutStore.focusPreviousPane();
    handled = true;
  }
  // Cmd/Ctrl+Shift+S: Toggle workspace manager
  else if (mod && e.shiftKey && e.code === 'KeyS') {
    showSettings.value = false;
    showWorkspaceManager.value = !showWorkspaceManager.value;
    handled = true;
  }
  // Cmd/Ctrl+,: Toggle settings
  else if (mod && e.code === 'Comma') {
    showWorkspaceManager.value = false;
    showSettings.value = !showSettings.value;
    handled = true;
  }
  // Cmd/Ctrl+Shift+F: Toggle terminal search
  else if (mod && e.shiftKey && e.code === 'KeyF') {
    layoutStore.triggerSearchToggle();
    handled = true;
  }
  // Cmd/Ctrl+Shift+Z: Toggle pane zoom
  else if (mod && e.shiftKey && e.code === 'KeyZ') {
    layoutStore.toggleZoom();
    handled = true;
  }
  // Cmd/Ctrl+Shift+G: Toggle git sidebar
  else if (mod && e.shiftKey && e.code === 'KeyG') {
    gitStore.toggleSidebar();
    handled = true;
  }
  // Escape: Close modals/diff viewer AND always forward to terminal PTY
  else if (e.code === 'Escape') {
    if (gitStore.diffVisible) {
      gitStore.closeDiff();
    } else if (showWorkspaceManager.value || showSettings.value || showCommandPalette.value || gitStore.sidebarVisible) {
      closeModals();
      gitStore.hideSidebar();
    }
    // Always send ESC to the active terminal so programs like Claude, vim, etc. receive it
    const activeNodeId = layoutStore.activePane;
    if (activeNodeId) {
      const ptySessionId = layoutStore.getSessionId(activeNodeId);
      if (ptySessionId) {
        invoke('write_terminal', { sessionId: ptySessionId, data: '\x1b' }).catch(() => {});
      }
    }
    handled = true;
  }

  if (handled) {
    e.preventDefault();
    e.stopImmediatePropagation();
  }
}

// Sync body background with terminal theme
watch(terminalBg, (bg) => {
  document.documentElement.style.background = bg;
  document.body.style.background = bg;
}, { immediate: true });

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown, true);
  // Restore saved window size/position before anything renders
  await restoreWindowState();
  // Set initial background
  document.documentElement.style.background = terminalBg.value;
  document.body.style.background = terminalBg.value;
  // On macOS, hide window title (traffic lights don't overlap terminal panes)
  if (isMacOS) {
    appWindow.setTitle('');
  }
  // Expose markdown viewer opener globally
  (window as any).__openMarkdownViewer = openMarkdownViewer;
  // Start watching for CWD changes to update git
  gitStore.startCwdWatcher();
  // Track window maximize state and persist window geometry
  isMaximized.value = await appWindow.isMaximized();
  appWindow.onResized(() => { updateMaximizedState(); debouncedSaveWindowState(); });
  appWindow.onMoved(debouncedSaveWindowState);
  // Get app version
  appVersion.value = await getVersion();
  // Check for updates silently on startup
  checkForUpdates(true);

  // Listen for "open-directory" events from second-instance launches
  // (Windows/Linux via single-instance plugin, macOS via Unix socket IPC).
  openDirUnlisten = await listen<string>('open-directory', (event) => {
    const cwd = event.payload;
    if (cwd) {
      layoutStore.addTab(undefined, createTerminalNode({ cwd }));
    }
  });

  // Register global shortcut for command palette.
  // CmdOrCtrl resolves to Cmd on macOS and Ctrl on Windows/Linux.
  // This bypasses WebView2 and works reliably on Windows.
  try {
    await register('CmdOrCtrl+P', () => {});
  } catch (e) {
    console.warn('Failed to register global shortcut CmdOrCtrl+P:', e);
  }

  // Listen for global shortcut events from Rust
  globalShortcutUnlisten = await listen<string>('global-shortcut', (event) => {
    if (event.payload.includes('P') && !event.payload.includes('Shift')) {
      if (!showCommandPalette.value) {
        closeModals();
        showCommandPalette.value = true;
      } else {
        showCommandPalette.value = false;
      }
    }
  });
});

onUnmounted(async () => {
  window.removeEventListener('keydown', handleKeydown, true);
  gitStore.stopCwdWatcher();
  // Unregister event listeners
  if (openDirUnlisten) {
    openDirUnlisten();
  }
  if (globalShortcutUnlisten) {
    globalShortcutUnlisten();
  }
  try {
    await unregister('CmdOrCtrl+P');
  } catch (e) {
    // Ignore errors during cleanup
  }
});
</script>

<template>
  <div class="h-screen w-screen flex flex-col" :style="{ background: terminalBg }">
    <!-- Title bar -->
    <div
      class="flex items-center justify-between h-10 select-none"
      style="background: var(--bg-primary);"
    >
      <!-- Left side - Logo, title and actions (draggable area) -->
      <div
        class="flex items-center gap-4 flex-1 h-full"
        :class="isMacOS ? 'pl-20 pr-4' : 'px-4'"
        data-tauri-drag-region
        @mousedown="startDrag"
        @dblclick="toggleMaximize"
      >
        <div class="flex items-center gap-2">
          <div class="w-1 h-4 rounded-sm" style="background: var(--accent-cyan);"></div>
          <span class="text-header" style="color: var(--text-primary);">PRIMARCH</span>
        </div>
        <div class="flex items-center gap-2" style="color: var(--text-muted);">
          <CommandLineIcon class="w-3.5 h-3.5" />
          <span class="text-label">{{ terminalCount }} ACTIVE</span>
        </div>

        <!-- Actions - stop propagation to prevent drag -->
        <div class="flex items-center gap-2 ml-auto" @mousedown.stop>
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

          <LayoutPresetPicker />

          <ShellSelector @select="handleShellSelect" />
        </div>
      </div>

      <!-- Window controls (Windows only - macOS uses native traffic lights) -->
      <div v-if="!isMacOS" class="flex items-center h-full">
        <button
          @click="minimizeWindow"
          class="window-control h-full px-4 flex items-center justify-center transition-colors"
          title="Minimize"
        >
          <MinusIcon class="w-4 h-4" />
        </button>
        <button
          @click="toggleMaximize"
          class="window-control h-full px-4 flex items-center justify-center transition-colors"
          :title="isMaximized ? 'Restore' : 'Maximize'"
        >
          <StopIcon class="w-3.5 h-3.5" />
        </button>
        <button
          @click="closeWindow"
          class="window-control window-control-close h-full px-4 flex items-center justify-center transition-colors"
          title="Close"
        >
          <XMarkIcon class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Tab bar (only show when multiple tabs) -->
    <TabBar v-if="layoutStore.tabs.length > 1" />

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

      <!-- Terminal Panes — render ALL tabs, show only active (keeps PTY sessions alive) -->
      <div class="flex-1 min-w-0 relative">
        <div
          v-for="tab in layoutStore.tabs"
          :key="tab.id"
          class="absolute inset-0"
          :class="{ 'pointer-events-none': tab.id !== layoutStore.activeTabId }"
          :style="{ visibility: tab.id === layoutStore.activeTabId ? 'visible' : 'hidden' }"
        >
          <PaneContainer :node="tab.layout" />
        </div>
      </div>

      <!-- Markdown Viewer (right side) -->
      <Transition
        enter-active-class="transition-all duration-200 ease-out"
        enter-from-class="w-0 opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition-all duration-150 ease-in"
        leave-from-class="opacity-100"
        leave-to-class="w-0 opacity-0"
      >
        <MarkdownViewer
          v-if="showMarkdownViewer"
          :source="markdownSource"
          @close="showMarkdownViewer = false"
        />
      </Transition>

      <!-- Modals -->
      <Teleport to="body">
        <!-- Command Palette -->
        <Transition
          enter-active-class="transition duration-100 ease-out"
          enter-from-class="opacity-0 -translate-y-2"
          enter-to-class="opacity-100 translate-y-0"
          leave-active-class="transition duration-75 ease-in"
          leave-from-class="opacity-100 translate-y-0"
          leave-to-class="opacity-0 -translate-y-2"
        >
          <CommandPalette
            v-if="showCommandPalette"
            @close="showCommandPalette = false"
            @show-settings="showSettings = true"
            @show-workspaces="showWorkspaceManager = true"
            @toggle-git="gitStore.toggleSidebar()"
            @open-markdown="(source: string) => openMarkdownViewer(source)"
          />
        </Transition>

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
      class="flex items-center justify-between px-4 h-6 select-none relative"
      style="background: var(--bg-primary); border-top: 1px solid var(--border-subtle);"
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
            <span
              v-if="gitChangeCount > 0"
              class="px-1 rounded text-label"
              style="background: var(--accent-cyan); color: var(--bg-primary); font-size: 0.55rem;"
            >
              {{ gitChangeCount }}
            </span>
          </template>
          <span v-else class="text-label" style="color: var(--text-muted);">No Repository</span>
        </div>

        <!-- Pull / Push — grouped tightly -->
        <div v-if="gitHasRepo" class="flex items-center">
          <button
            @click.stop="handleStatusPull"
            :disabled="gitIsRemoteOperating"
            class="flex items-center gap-0.5 px-1 py-0.5 rounded transition-colors hover:bg-[var(--bg-hover)]"
            :style="{ color: gitBehind > 0 ? 'var(--accent-orange)' : 'var(--text-muted)' }"
            :title="gitBehind > 0 ? `Pull ${gitBehind} commit${gitBehind === 1 ? '' : 's'} from remote` : 'Pull'"
          >
            <CloudArrowDownIcon class="w-3 h-3" />
            <span v-if="gitBehind > 0" class="text-label">{{ gitBehind }}</span>
          </button>
          <button
            @click.stop="handleStatusPush"
            :disabled="gitIsRemoteOperating"
            class="flex items-center gap-0.5 px-1 py-0.5 rounded transition-colors hover:bg-[var(--bg-hover)]"
            :style="{ color: (gitAhead > 0 || gitNeedsPublish) ? 'var(--accent-green)' : 'var(--text-muted)' }"
            :title="gitNeedsPublish ? 'Publish branch to remote' : gitAhead > 0 ? `Push ${gitAhead} commit${gitAhead === 1 ? '' : 's'} to remote` : 'Push'"
          >
            <CloudArrowUpIcon class="w-3 h-3" />
            <span v-if="gitAhead > 0 || gitNeedsPublish" class="text-label">{{ gitNeedsPublish ? 'publish' : gitAhead }}</span>
          </button>
        </div>
      </div>

      <!-- Status notification (center) -->
      <Transition
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition duration-500 ease-in"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <span
          v-if="statusNotification"
          class="absolute left-1/2 -translate-x-1/2 text-label"
          style="color: var(--text-muted);"
        >
          {{ statusNotification }}
        </span>
      </Transition>

      <!-- Right side - Settings and version -->
      <div class="flex items-center gap-3">
        <button
          @click="showSettings = !showSettings; showWorkspaceManager = false"
          class="flex items-center gap-1.5 transition-colors hover:opacity-80"
          :class="{ 'text-accent': showSettings }"
          :style="{ color: showSettings ? 'var(--accent-cyan)' : 'var(--text-muted)' }"
          title="Settings (Ctrl+,)"
        >
          <Cog6ToothIcon class="w-3 h-3" />
          <span class="text-label">Settings</span>
        </button>
        <!-- Update available button -->
        <button
          v-if="updateAvailable && !requiresManualUpdate"
          @click="downloadAndInstall"
          :disabled="isDownloading"
          class="flex items-center gap-1.5 px-2 py-0.5 rounded transition-opacity cursor-pointer hover:opacity-80"
          style="color: var(--accent-cyan);"
          :title="`Update to v${updateInfo?.version}`"
        >
          <ArrowDownTrayIcon class="w-3 h-3" style="color: inherit;" />
          <span class="text-label" style="color: inherit;">{{ isDownloading ? 'Installing...' : `Update v${updateInfo?.version}` }}</span>
        </button>
        <!-- Manual download button (macOS or auto-update failed) -->
        <button
          v-else-if="updateAvailable && requiresManualUpdate"
          @click="openReleasesPage"
          class="flex items-center gap-1.5 px-2 py-0.5 rounded transition-opacity cursor-pointer hover:opacity-80"
          style="color: var(--accent-cyan);"
          :title="`Download v${updateInfo?.version} from GitHub`"
        >
          <ArrowDownTrayIcon class="w-3 h-3" style="color: inherit;" />
          <span class="text-label" style="color: inherit;">Download v{{ updateInfo?.version }}</span>
        </button>
        <span class="text-label" style="color: var(--text-muted);">v{{ appVersion }}</span>
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
