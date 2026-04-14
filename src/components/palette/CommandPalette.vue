<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  MagnifyingGlassIcon,
  FolderIcon,
  SwatchIcon,
  Squares2X2Icon,
  BookmarkIcon,
  ArrowRightIcon,
  Cog6ToothIcon,
  XMarkIcon,
  CodeBracketIcon,
  ArrowsPointingOutIcon,
} from '@heroicons/vue/24/outline';
import { useLayoutStore } from '../../stores/layout';
import { useSettingsStore } from '../../stores/settings';
import { useWorkspaceStore } from '../../stores/workspace';
import { layoutPresets } from '../layout/presets';
import { themes } from '../../themes/presets';
import { fuzzyMatch } from '../../utils/fuzzyMatch';
import { getRecentDirectories, addRecentDirectory } from '../../utils/recentDirectories';
import { getAliases, saveAlias, deleteAlias, type CommandAlias } from '../../utils/aliases';
import { CommandLineIcon, PlusIcon, TrashIcon, DocumentTextIcon, ArrowDownTrayIcon, ViewfinderCircleIcon } from '@heroicons/vue/24/outline';

interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

interface PaletteItem {
  id: string;
  label: string;
  description?: string;
  category?: string;
  icon?: any;
  action: () => void;
  score: number;
}

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'showSettings'): void;
  (e: 'showWorkspaces'): void;
  (e: 'toggleGit'): void;
  (e: 'openMarkdown', source: string): void;
}>();

const layoutStore = useLayoutStore();
const settingsStore = useSettingsStore();
const workspaceStore = useWorkspaceStore();

const inputRef = ref<HTMLInputElement | null>(null);
const query = ref('');
const selectedIndex = ref(0);
const mode = ref<'commands' | 'directories' | 'themes' | 'layouts' | 'workspaces' | 'create-alias' | 'markdown'>('commands');
const directoryEntries = ref<DirEntry[]>([]);
const searchResults = ref<DirEntry[]>([]);
const browsePath = ref('');
const isLoadingDirs = ref(false);
const searchDebounceTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const listRef = ref<HTMLDivElement | null>(null);
const aliases = ref<CommandAlias[]>(getAliases());
const aliasName = ref('');
const aliasCommand = ref('');
const aliasStep = ref<'name' | 'command'>('name');
const markdownFiles = ref<string[]>([]);

// Detect platform for shortcut display
const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
const mod = isMac ? 'Cmd' : 'Ctrl';

onMounted(() => {
  nextTick(() => inputRef.value?.focus());
  workspaceStore.loadWorkspaces();
});

// Commands available in default mode
const staticCommands: PaletteItem[] = [
  {
    id: 'goto-dir',
    label: 'Go to Directory',
    description: 'cd into a directory',
    icon: FolderIcon,
    action: () => enterMode('directories'),
    score: 0,
  },
  {
    id: 'preview-markdown',
    label: 'Preview Markdown',
    description: 'Open a .md file or URL',
    icon: DocumentTextIcon,
    action: () => enterMode('markdown'),
    score: 0,
  },
  {
    id: 'change-theme',
    label: 'Change Theme',
    description: 'Switch color scheme',
    icon: SwatchIcon,
    action: () => enterMode('themes'),
    score: 0,
  },
  {
    id: 'apply-layout',
    label: 'Apply Layout Preset',
    description: 'Multi-pane layouts',
    icon: Squares2X2Icon,
    action: () => enterMode('layouts'),
    score: 0,
  },
  {
    id: 'load-workspace',
    label: 'Load Workspace',
    description: 'Restore saved workspace',
    icon: BookmarkIcon,
    action: () => enterMode('workspaces'),
    score: 0,
  },
  {
    id: 'split-right',
    label: 'Split Right',
    description: `${mod}+Shift+D`,
    icon: ArrowsPointingOutIcon,
    action: () => { layoutStore.splitHorizontal(); close(); },
    score: 0,
  },
  {
    id: 'split-down',
    label: 'Split Down',
    description: `${mod}+Shift+E`,
    icon: ArrowsPointingOutIcon,
    action: () => { layoutStore.splitVertical(); close(); },
    score: 0,
  },
  {
    id: 'close-pane',
    label: 'Close Pane',
    description: `${mod}+Shift+W`,
    icon: XMarkIcon,
    action: () => {
      if (layoutStore.activePane) layoutStore.closePane(layoutStore.activePane);
      close();
    },
    score: 0,
  },
  {
    id: 'open-settings',
    label: 'Open Settings',
    description: `${mod}+,`,
    icon: Cog6ToothIcon,
    action: () => { emit('showSettings'); close(); },
    score: 0,
  },
  {
    id: 'toggle-git',
    label: 'Toggle Git Sidebar',
    description: `${mod}+Shift+G`,
    icon: CodeBracketIcon,
    action: () => { emit('toggleGit'); close(); },
    score: 0,
  },
  {
    id: 'focus-next',
    label: 'Focus Next Pane',
    description: `${mod}+Tab`,
    icon: ArrowRightIcon,
    action: () => { layoutStore.focusNextPane(); close(); },
    score: 0,
  },
  {
    id: 'new-tab',
    label: 'New Tab',
    description: `${mod}+T`,
    icon: PlusIcon,
    action: () => { layoutStore.addTab(); close(); },
    score: 0,
  },
  {
    id: 'close-tab',
    label: 'Close Tab',
    description: `${mod}+W`,
    icon: XMarkIcon,
    action: () => { layoutStore.closeTab(layoutStore.activeTabId); close(); },
    score: 0,
  },
  {
    id: 'zoom-pane',
    label: 'Toggle Zoom Pane',
    description: `${mod}+Shift+Z`,
    icon: ViewfinderCircleIcon,
    action: () => { layoutStore.toggleZoom(); close(); },
    score: 0,
  },
  {
    id: 'save-output',
    label: 'Save Terminal Output',
    description: 'Export scrollback to file',
    icon: ArrowDownTrayIcon,
    action: async () => {
      close();
      // Find the active pane's TerminalPane component via DOM and call getBufferText
      const activeNodeId = layoutStore.activePane;
      if (!activeNodeId) return;
      const sessionId = layoutStore.getSessionId(activeNodeId);
      if (!sessionId) return;
      // Dispatch a custom event that PaneContainer can handle
      window.dispatchEvent(new CustomEvent('primarch-export-output', { detail: { nodeId: activeNodeId } }));
    },
    score: 0,
  },
  {
    id: 'create-alias',
    label: 'Create Alias',
    description: 'Save a command shortcut',
    icon: PlusIcon,
    action: () => enterCreateAlias(),
    score: 0,
  },
];

const filteredItems = computed<PaletteItem[]>(() => {
  const q = query.value.trim();

  if (mode.value === 'commands') {
    const aliasItems: PaletteItem[] = aliases.value.map((a) => ({
      id: `alias-${a.id}`,
      label: a.name,
      description: a.command,
      category: 'Alias',
      icon: CommandLineIcon,
      action: () => runAlias(a),
      score: 0,
    }));
    const allCommands = [...staticCommands, ...aliasItems];
    if (!q) return allCommands;
    return allCommands
      .map((cmd) => {
        const matchLabel = fuzzyMatch(q, cmd.label);
        const matchDesc = cmd.description ? fuzzyMatch(q, cmd.description) : null;
        const best = matchLabel && matchDesc
          ? (matchLabel.score >= matchDesc.score ? matchLabel : matchDesc)
          : matchLabel || matchDesc;
        return best ? { ...cmd, score: best.score } : null;
      })
      .filter((x): x is PaletteItem => x !== null)
      .sort((a, b) => b.score - a.score);
  }

  if (mode.value === 'create-alias') {
    return [];
  }

  if (mode.value === 'directories') {
    const items: PaletteItem[] = [];

    // Recent directories (filtered by query)
    const recent = getRecentDirectories();
    for (const dir of recent) {
      const dirName = dir.split('/').pop() || dir;
      if (q) {
        const match = fuzzyMatch(q, dirName) || fuzzyMatch(q, dir);
        if (!match) continue;
        items.push({
          id: `recent-${dir}`,
          label: dirName,
          description: dir,
          category: 'Recent',
          icon: FolderIcon,
          action: () => cdToDirectory(dir),
          score: match.score + 100, // Boost recent dirs
        });
      } else {
        items.push({
          id: `recent-${dir}`,
          label: dirName,
          description: dir,
          category: 'Recent',
          icon: FolderIcon,
          action: () => cdToDirectory(dir),
          score: 100,
        });
      }
    }

    // Browse entries (shown when no query)
    if (!q) {
      for (const entry of directoryEntries.value) {
        items.push({
          id: `dir-${entry.path}`,
          label: entry.name,
          description: entry.path,
          category: 'Browse',
          icon: FolderIcon,
          action: () => cdToDirectory(entry.path),
          score: 0,
        });
      }
    }

    // Search results (shown when query is typed)
    if (q) {
      const seen = new Set(items.map((i) => i.description));
      for (const entry of searchResults.value) {
        if (seen.has(entry.path)) continue;
        const match = fuzzyMatch(q, entry.name);
        items.push({
          id: `search-${entry.path}`,
          label: entry.name,
          description: entry.path,
          category: 'Search',
          icon: FolderIcon,
          action: () => cdToDirectory(entry.path),
          score: match?.score ?? 1,
        });
      }
    }

    return items.sort((a, b) => b.score - a.score);
  }

  if (mode.value === 'themes') {
    const items: PaletteItem[] = [];
    for (const theme of themes) {
      if (q) {
        const match = fuzzyMatch(q, theme.name);
        if (!match) continue;
        items.push({
          id: `theme-${theme.id}`,
          label: theme.name,
          description: theme.light ? 'Light' : 'Dark',
          icon: SwatchIcon,
          action: () => { settingsStore.setTheme(theme.id); close(); },
          score: match.score,
        });
      } else {
        items.push({
          id: `theme-${theme.id}`,
          label: theme.name,
          description: theme.light ? 'Light' : 'Dark',
          icon: SwatchIcon,
          action: () => { settingsStore.setTheme(theme.id); close(); },
          score: theme.id === settingsStore.themeId ? 100 : 0,
        });
      }
    }
    return items.sort((a, b) => b.score - a.score);
  }

  if (mode.value === 'layouts') {
    const items: PaletteItem[] = [];
    for (const preset of layoutPresets) {
      if (q) {
        const match = fuzzyMatch(q, preset.name);
        if (!match) continue;
        items.push({
          id: `layout-${preset.id}`,
          label: preset.name,
          description: `${preset.terminalCount} terminals`,
          icon: Squares2X2Icon,
          action: () => { layoutStore.applyPreset(preset); close(); },
          score: match.score,
        });
      } else {
        items.push({
          id: `layout-${preset.id}`,
          label: preset.name,
          description: `${preset.terminalCount} terminals`,
          icon: Squares2X2Icon,
          action: () => { layoutStore.applyPreset(preset); close(); },
          score: 0,
        });
      }
    }
    return items.sort((a, b) => b.score - a.score);
  }

  if (mode.value === 'workspaces') {
    const items: PaletteItem[] = [];
    for (const ws of workspaceStore.workspaces) {
      if (q) {
        const match = fuzzyMatch(q, ws.name);
        if (!match) continue;
        items.push({
          id: `ws-${ws.id}`,
          label: ws.name,
          description: new Date(ws.createdAt).toLocaleDateString(),
          icon: BookmarkIcon,
          action: () => { workspaceStore.loadWorkspace(ws.id); close(); },
          score: match.score,
        });
      } else {
        items.push({
          id: `ws-${ws.id}`,
          label: ws.name,
          description: new Date(ws.createdAt).toLocaleDateString(),
          icon: BookmarkIcon,
          action: () => { workspaceStore.loadWorkspace(ws.id); close(); },
          score: 0,
        });
      }
    }
    return items.sort((a, b) => b.score - a.score);
  }

  if (mode.value === 'markdown') {
    const items: PaletteItem[] = [];

    // If query looks like a URL, offer to open it
    if (q && (q.startsWith('http://') || q.startsWith('https://'))) {
      items.push({
        id: 'md-url',
        label: 'Open URL',
        description: q,
        icon: DocumentTextIcon,
        action: () => { emit('openMarkdown', q); close(); },
        score: 1000,
      });
    }

    // List discovered .md files
    for (const file of markdownFiles.value) {
      const name = file.replace(/\\/g, '/').split('/').pop() || file;
      if (q && !q.startsWith('http')) {
        const match = fuzzyMatch(q, name) || fuzzyMatch(q, file);
        if (!match) continue;
        items.push({
          id: `md-${file}`,
          label: name,
          description: file,
          icon: DocumentTextIcon,
          action: () => { emit('openMarkdown', file); close(); },
          score: match.score,
        });
      } else if (!q) {
        items.push({
          id: `md-${file}`,
          label: name,
          description: file,
          icon: DocumentTextIcon,
          action: () => { emit('openMarkdown', file); close(); },
          score: 0,
        });
      }
    }

    return items.sort((a, b) => b.score - a.score);
  }

  return [];
});

// Reset selected index when results change
watch(filteredItems, () => {
  selectedIndex.value = 0;
});

// Debounced directory search when typing in directory mode
watch(query, (q) => {
  if (mode.value !== 'directories') return;
  if (searchDebounceTimer.value) clearTimeout(searchDebounceTimer.value);
  const trimmed = q.trim();
  if (trimmed.length < 2) {
    searchResults.value = [];
    return;
  }
  isLoadingDirs.value = true;
  searchDebounceTimer.value = setTimeout(async () => {
    try {
      searchResults.value = await invoke<DirEntry[]>('search_directories', { query: trimmed });
    } catch {
      searchResults.value = [];
    } finally {
      isLoadingDirs.value = false;
    }
  }, 150);
});

// Load directory entries when entering directory mode
async function loadDirectoryEntries(path: string) {
  isLoadingDirs.value = true;
  try {
    directoryEntries.value = await invoke<DirEntry[]>('list_directory', { path });
    browsePath.value = path;
  } catch {
    directoryEntries.value = [];
  } finally {
    isLoadingDirs.value = false;
  }
}

function enterMode(newMode: typeof mode.value) {
  mode.value = newMode;
  query.value = '';
  selectedIndex.value = 0;

  if (newMode === 'directories') {
    // Load the active terminal's CWD for browsing
    const activeId = layoutStore.activePane;
    if (activeId) {
      const sessionId = layoutStore.getSessionId(activeId);
      if (sessionId) {
        invoke<string>('get_terminal_cwd', { sessionId })
          .then((cwd) => loadDirectoryEntries(cwd))
          .catch(() => loadDirectoryEntries('~'));
      } else {
        loadDirectoryEntries('~');
      }
    } else {
      loadDirectoryEntries('~');
    }
  }

  if (newMode === 'markdown') {
    // Load .md files from current working directory
    markdownFiles.value = [];
    const activeId = layoutStore.activePane;
    if (activeId) {
      const sessionId = layoutStore.getSessionId(activeId);
      if (sessionId) {
        invoke<string>('get_terminal_cwd', { sessionId })
          .then((cwd) => invoke<string[]>('list_markdown_files', { path: cwd }))
          .then((files) => { markdownFiles.value = files; })
          .catch(() => {});
      }
    }
  }
}

async function cdToDirectory(path: string) {
  const activeId = layoutStore.activePane;
  if (!activeId) return;

  const sessionId = layoutStore.getSessionId(activeId);
  if (!sessionId) return;

  // Write cd command to the active terminal
  const cdCommand = `cd ${path.includes(' ') ? `"${path}"` : path}\r`;
  await invoke('write_terminal', { sessionId, data: cdCommand });
  addRecentDirectory(path);
  close();
}

async function runAlias(alias: CommandAlias) {
  const activeId = layoutStore.activePane;
  if (!activeId) return;
  const sessionId = layoutStore.getSessionId(activeId);
  if (!sessionId) return;
  await invoke('write_terminal', { sessionId, data: alias.command + '\r' });
  close();
}

function enterCreateAlias() {
  mode.value = 'create-alias';
  aliasStep.value = 'name';
  aliasName.value = '';
  aliasCommand.value = '';
  query.value = '';
}

function handleCreateAliasSubmit() {
  if (aliasStep.value === 'name') {
    const name = query.value.trim();
    if (!name) return;
    aliasName.value = name;
    aliasStep.value = 'command';
    query.value = '';
  } else {
    const command = query.value.trim();
    if (!command) return;
    aliasCommand.value = command;
    const newAlias: CommandAlias = {
      id: crypto.randomUUID(),
      name: aliasName.value,
      command: aliasCommand.value,
    };
    saveAlias(newAlias);
    aliases.value = getAliases();
    mode.value = 'commands';
    query.value = '';
  }
}

function removeAlias(id: string) {
  deleteAlias(id);
  aliases.value = getAliases();
}

function close() {
  emit('close');
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, filteredItems.value.length - 1);
    scrollToSelected();
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
    scrollToSelected();
  } else if (e.key === 'Enter') {
    e.preventDefault();
    if (mode.value === 'create-alias') {
      handleCreateAliasSubmit();
    } else if (mode.value === 'markdown') {
      const source = query.value.trim();
      if (source) {
        emit('openMarkdown', source);
        close();
      }
    } else {
      const item = filteredItems.value[selectedIndex.value];
      if (item) item.action();
    }
  } else if (e.key === 'Escape') {
    e.preventDefault();
    if (mode.value !== 'commands') {
      mode.value = 'commands';
      query.value = '';
    } else {
      close();
    }
  } else if (e.key === 'Backspace' && query.value === '' && mode.value !== 'commands') {
    mode.value = 'commands';
  }
}

function scrollToSelected() {
  nextTick(() => {
    const list = listRef.value;
    if (!list) return;
    const item = list.children[selectedIndex.value] as HTMLElement;
    if (item) {
      item.scrollIntoView({ block: 'nearest' });
    }
  });
}

const modeLabel = computed(() => {
  switch (mode.value) {
    case 'directories': return 'Go to Directory';
    case 'themes': return 'Change Theme';
    case 'layouts': return 'Apply Layout';
    case 'workspaces': return 'Load Workspace';
    case 'create-alias': return aliasStep.value === 'name' ? 'New Alias' : `Alias: ${aliasName.value}`;
    case 'markdown': return 'Preview Markdown';
    default: return '';
  }
});

const placeholderText = computed(() => {
  switch (mode.value) {
    case 'directories': return 'Search directories...';
    case 'themes': return 'Search themes...';
    case 'layouts': return 'Search layouts...';
    case 'workspaces': return 'Search workspaces...';
    case 'create-alias': return aliasStep.value === 'name' ? 'Enter alias name...' : 'Enter command to run...';
    case 'markdown': return 'Enter file path or URL...';
    default: return 'Type a command...';
  }
});
</script>

<template>
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50"
    style="background: rgba(0, 0, 0, 0.4); backdrop-filter: blur(2px);"
    @click="close"
  >
    <!-- Palette container -->
    <div
      class="mx-auto mt-[12vh]"
      style="
        width: min(600px, 85vw);
        background: var(--bg-secondary);
        border: 1px solid var(--border-default);
        box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
      "
      @click.stop
    >
      <!-- Input area -->
      <div
        class="flex items-center gap-3 px-4 py-3"
        style="border-bottom: 1px solid var(--border-subtle);"
      >
        <MagnifyingGlassIcon class="w-4 h-4 flex-shrink-0" style="color: var(--text-muted);" />

        <!-- Mode badge -->
        <div
          v-if="mode !== 'commands'"
          class="flex items-center gap-1.5 px-2 py-0.5 flex-shrink-0"
          style="background: rgba(var(--accent-rgb), 0.1); border: 1px solid var(--accent-cyan); font-size: 0.65rem; color: var(--accent-cyan);"
        >
          <span class="text-label" style="color: var(--accent-cyan);">{{ modeLabel }}</span>
          <button
            @click="mode = 'commands'; query = ''"
            class="hover:opacity-80"
          >
            <XMarkIcon class="w-3 h-3" />
          </button>
        </div>

        <input
          ref="inputRef"
          v-model="query"
          :placeholder="placeholderText"
          class="flex-1 bg-transparent outline-none"
          style="color: var(--text-primary); font-size: 0.85rem; font-family: inherit;"
          @keydown="handleKeydown"
        />

        <div
          class="flex-shrink-0 px-1.5 py-0.5"
          style="font-size: 0.6rem; color: var(--text-muted); border: 1px solid var(--border-subtle);"
        >
          ESC
        </div>
      </div>

      <!-- Results list -->
      <div
        ref="listRef"
        class="overflow-y-auto"
        style="max-height: 400px;"
      >
        <div
          v-if="mode === 'create-alias'"
          class="px-4 py-4"
        >
          <div class="flex items-center gap-3 mb-2">
            <div
              class="w-6 h-6 flex items-center justify-center text-xs font-bold"
              :style="{
                background: aliasStep === 'name' ? 'var(--accent-cyan)' : 'var(--bg-tertiary)',
                color: aliasStep === 'name' ? 'var(--bg-primary)' : 'var(--text-muted)',
                border: '1px solid var(--border-subtle)',
              }"
            >1</div>
            <span style="font-size: 0.75rem; color: var(--text-secondary);">Name</span>
            <div style="flex: 0 0 24px; height: 1px; background: var(--border-subtle);"></div>
            <div
              class="w-6 h-6 flex items-center justify-center text-xs font-bold"
              :style="{
                background: aliasStep === 'command' ? 'var(--accent-cyan)' : 'var(--bg-tertiary)',
                color: aliasStep === 'command' ? 'var(--bg-primary)' : 'var(--text-muted)',
                border: '1px solid var(--border-subtle)',
              }"
            >2</div>
            <span style="font-size: 0.75rem; color: var(--text-secondary);">Command</span>
          </div>
          <div style="font-size: 0.7rem; color: var(--text-muted);">
            {{ aliasStep === 'name' ? 'Type a name for your alias, then press Enter' : 'Type the command to execute, then press Enter' }}
          </div>
        </div>

        <div
          v-else-if="isLoadingDirs && mode === 'directories'"
          class="px-4 py-6 text-center"
        >
          <span style="font-size: 0.75rem; color: var(--text-muted);">Loading...</span>
        </div>

        <div
          v-else-if="filteredItems.length === 0"
          class="px-4 py-6 text-center"
        >
          <span style="font-size: 0.75rem; color: var(--text-muted);">No results found</span>
        </div>

        <button
          v-for="(item, index) in filteredItems"
          :key="item.id"
          @click="item.action()"
          @mouseenter="selectedIndex = index"
          class="w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors duration-75 group"
          :style="{
            background: index === selectedIndex ? 'var(--bg-elevated)' : 'transparent',
            borderLeft: index === selectedIndex ? '2px solid var(--accent-cyan)' : '2px solid transparent',
          }"
        >
          <component
            :is="item.icon"
            v-if="item.icon"
            class="w-4 h-4 flex-shrink-0"
            :style="{ color: index === selectedIndex ? 'var(--accent-cyan)' : 'var(--text-muted)' }"
          />

          <div class="flex-1 min-w-0">
            <div
              class="truncate"
              style="font-size: 0.8rem; color: var(--text-primary);"
            >
              {{ item.label }}
            </div>
            <div
              v-if="item.description"
              class="truncate"
              style="font-size: 0.65rem; color: var(--text-muted);"
            >
              {{ item.description }}
            </div>
          </div>

          <span
            v-if="item.category"
            style="font-size: 0.6rem; color: var(--text-muted); opacity: 0.6;"
          >
            {{ item.category }}
          </span>

          <button
            v-if="item.id.startsWith('alias-')"
            class="w-5 h-5 flex items-center justify-center opacity-0 group-hover:opacity-100 hover:!opacity-100 flex-shrink-0"
            style="color: var(--text-muted);"
            @click.stop="removeAlias(item.id.replace('alias-', ''))"
            title="Delete alias"
          >
            <TrashIcon class="w-3 h-3 hover:text-red-400" />
          </button>

          <ArrowRightIcon
            v-if="mode === 'commands' && ['goto-dir', 'change-theme', 'apply-layout', 'load-workspace'].includes(item.id)"
            class="w-3 h-3 flex-shrink-0"
            style="color: var(--text-muted);"
          />
        </button>
      </div>

      <!-- Footer hint -->
      <div
        class="flex items-center gap-4 px-4 py-2"
        style="border-top: 1px solid var(--border-subtle); font-size: 0.6rem; color: var(--text-muted);"
      >
        <span><kbd style="border: 1px solid var(--border-subtle); padding: 0 3px;">↑↓</kbd> navigate</span>
        <span><kbd style="border: 1px solid var(--border-subtle); padding: 0 3px;">↵</kbd> select</span>
        <span><kbd style="border: 1px solid var(--border-subtle); padding: 0 3px;">esc</kbd> {{ mode !== 'commands' ? 'back' : 'close' }}</span>
      </div>
    </div>
  </div>
</template>
