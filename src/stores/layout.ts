import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { ref, computed } from 'vue';
import type { LayoutNode, Tab } from '../types';
import {
  createTerminalNode,
  splitNode,
  closeNode,
  updateRatio,
  findNode,
  getAllTerminals,
  countTerminals,
} from '../components/layout/LayoutTree';
import type { LayoutPreset } from '../components/layout/presets';

function createTab(name?: string, layout?: LayoutNode): Tab {
  const node = layout || createTerminalNode();
  return {
    id: crypto.randomUUID(),
    name: name || 'Terminal',
    layout: node,
    sessionRegistry: new Map(),
    pendingReattach: new Set(),
  };
}

export const useLayoutStore = defineStore('layout', () => {
  // State
  const tabs = ref<Tab[]>([createTab()]);
  const activeTabId = ref<string>(tabs.value[0].id);
  const activePane = ref<string | undefined>(tabs.value[0].layout.id);

  // Tab counter for naming
  let tabCounter = 1;

  // Helpers
  function getActiveTab(): Tab | undefined {
    return tabs.value.find((t) => t.id === activeTabId.value);
  }

  // Getters
  const rootLayout = computed(() => {
    const tab = getActiveTab();
    return tab ? tab.layout : createTerminalNode();
  });

  const terminalCount = computed(() => {
    let count = 0;
    for (const tab of tabs.value) {
      count += countTerminals(tab.layout);
    }
    return count;
  });

  const activeTabTerminalCount = computed(() => {
    const tab = getActiveTab();
    return tab ? countTerminals(tab.layout) : 0;
  });

  const allTerminals = computed(() => {
    const tab = getActiveTab();
    return tab ? getAllTerminals(tab.layout) : [];
  });

  // Tab actions
  function addTab(name?: string, layout?: LayoutNode) {
    tabCounter++;
    const tab = createTab(name || `Terminal ${tabCounter}`, layout);
    tabs.value.push(tab);
    activeTabId.value = tab.id;
    activePane.value = getAllTerminals(tab.layout)[0]?.id;
    return tab.id;
  }

  function closeTab(tabId: string) {
    const index = tabs.value.findIndex((t) => t.id === tabId);
    if (index === -1) return;

    // Kill all PTY sessions in this tab
    const tab = tabs.value[index];
    for (const sessionId of tab.sessionRegistry.values()) {
      invoke('kill_terminal', { sessionId }).catch(() => {});
    }

    tabs.value.splice(index, 1);

    // If we closed the last tab, create a new one
    if (tabs.value.length === 0) {
      tabCounter++;
      const newTab = createTab(`Terminal ${tabCounter}`);
      tabs.value.push(newTab);
      activeTabId.value = newTab.id;
      activePane.value = newTab.layout.id;
      return;
    }

    // If we closed the active tab, switch to adjacent
    if (activeTabId.value === tabId) {
      const newIndex = Math.min(index, tabs.value.length - 1);
      activeTabId.value = tabs.value[newIndex].id;
      const terminals = getAllTerminals(tabs.value[newIndex].layout);
      activePane.value = terminals[0]?.id;
    }
  }

  function setActiveTab(tabId: string) {
    const tab = tabs.value.find((t) => t.id === tabId);
    if (!tab) return;
    activeTabId.value = tabId;
    const terminals = getAllTerminals(tab.layout);
    // Restore active pane for this tab, or default to first
    if (!activePane.value || !terminals.find((t) => t.id === activePane.value)) {
      activePane.value = terminals[0]?.id;
    }
  }

  function renameTab(tabId: string, name: string) {
    const tab = tabs.value.find((t) => t.id === tabId);
    if (tab) tab.name = name;
  }

  function moveTab(fromIndex: number, toIndex: number) {
    if (fromIndex < 0 || fromIndex >= tabs.value.length) return;
    if (toIndex < 0 || toIndex >= tabs.value.length) return;
    const [tab] = tabs.value.splice(fromIndex, 1);
    tabs.value.splice(toIndex, 0, tab);
  }

  function duplicateTab(tabId: string) {
    const tab = tabs.value.find((t) => t.id === tabId);
    if (!tab) return;
    // Create a new tab — can't clone running PTY sessions, so just open a fresh terminal
    tabCounter++;
    addTab(`${tab.name} (copy)`);
  }

  // Pane actions (operate on active tab)
  async function splitPane(
    direction: 'horizontal' | 'vertical',
    targetId?: string,
    options?: {
      shell?: string;
      cwd?: string;
      startupCommand?: string;
    }
  ) {
    const tab = getActiveTab();
    if (!tab) return;

    const target = targetId || activePane.value;
    if (!target) return;

    let finalOptions = options || {};
    if (!finalOptions.cwd && target) {
      const existingNode = findNode(tab.layout, target);
      const sessionId = tab.sessionRegistry.get(target);

      if (sessionId) {
        try {
          const cwd = await invoke<string>('get_terminal_cwd', { sessionId });
          if (cwd) {
            finalOptions = { ...finalOptions, cwd };
          }
        } catch {
          if (existingNode?.type === 'terminal' && existingNode.cwd) {
            finalOptions = { ...finalOptions, cwd: existingNode.cwd };
          }
        }
      } else if (existingNode?.type === 'terminal' && existingNode.cwd) {
        finalOptions = { ...finalOptions, cwd: existingNode.cwd };
      }
    }

    if (finalOptions.cwd) {
      const existingNode = findNode(tab.layout, target);
      if (existingNode && existingNode.type === 'terminal') {
        existingNode.cwd = finalOptions.cwd;
      }
    }

    const existingSessionId = tab.sessionRegistry.get(target);
    if (existingSessionId) {
      tab.pendingReattach.add(existingSessionId);
      setTimeout(() => tab.pendingReattach.delete(existingSessionId), 5000);
    }

    tab.layout = splitNode(tab.layout, target, direction, finalOptions);

    if (existingSessionId) {
      const copiedNode = findNode(tab.layout, target);
      if (copiedNode && copiedNode.type === 'terminal') {
        copiedNode.sessionId = existingSessionId;
      }
    }

    const terminals = getAllTerminals(tab.layout);
    const newTerminal = terminals[terminals.length - 1];
    if (newTerminal?.id) {
      activePane.value = newTerminal.id;
    }
  }

  async function splitHorizontal(
    targetId?: string,
    options?: { shell?: string; cwd?: string; startupCommand?: string }
  ) {
    await splitPane('horizontal', targetId, options);
  }

  async function splitVertical(
    targetId?: string,
    options?: { shell?: string; cwd?: string; startupCommand?: string }
  ) {
    await splitPane('vertical', targetId, options);
  }

  function closePane(targetId: string) {
    const tab = getActiveTab();
    if (!tab) return;

    const allTerms = getAllTerminals(tab.layout);
    for (const t of allTerms) {
      if (t.id && t.id !== targetId) {
        const ptySessionId = tab.sessionRegistry.get(t.id);
        if (ptySessionId) {
          tab.pendingReattach.add(ptySessionId);
          t.sessionId = ptySessionId;
          setTimeout(() => tab.pendingReattach.delete(ptySessionId), 5000);
        }
      }
    }

    const result = closeNode(tab.layout, targetId);

    if (result === null) {
      tab.layout = createTerminalNode();
      activePane.value = tab.layout.id;
    } else {
      tab.layout = result;

      if (activePane.value === targetId) {
        const terminals = getAllTerminals(tab.layout);
        activePane.value = terminals[0]?.id;
      }
    }
  }

  function setActivePane(id: string) {
    activePane.value = id;
  }

  function updateRatioAction(splitId: string, ratio: number) {
    const tab = getActiveTab();
    if (!tab) return;
    tab.layout = updateRatio(tab.layout, splitId, ratio);
  }

  function updateTerminalTitle(terminalId: string, title: string) {
    const tab = getActiveTab();
    if (!tab) return;
    const node = findNode(tab.layout, terminalId);
    if (node && node.type === 'terminal') {
      node.title = title;
    }
  }

  async function applyPreset(preset: LayoutPreset) {
    const tab = getActiveTab();
    if (!tab) return;

    let cwd: string | undefined;
    if (activePane.value) {
      const sessionId = tab.sessionRegistry.get(activePane.value);
      if (sessionId) {
        try {
          cwd = await invoke<string>('get_terminal_cwd', { sessionId });
        } catch {
          // Fall back to no cwd
        }
      }
    }

    tab.sessionRegistry.clear();
    setLayout(preset.build(cwd));
  }

  function setLayout(layout: LayoutNode) {
    const tab = getActiveTab();
    if (!tab) return;
    tab.layout = layout;
    const terminals = getAllTerminals(tab.layout);
    activePane.value = terminals[0]?.id;
  }

  function getLayout(): LayoutNode {
    const tab = getActiveTab();
    return tab ? tab.layout : createTerminalNode();
  }

  // Session registry management (scoped to active tab)
  function registerSession(nodeId: string, sessionId: string) {
    const tab = getActiveTab();
    if (tab) tab.sessionRegistry.set(nodeId, sessionId);
  }

  function unregisterSession(nodeId: string) {
    const tab = getActiveTab();
    if (tab) tab.sessionRegistry.delete(nodeId);
  }

  function getSessionId(nodeId: string): string | undefined {
    const tab = getActiveTab();
    return tab?.sessionRegistry.get(nodeId);
  }

  function getAllSessionMappings(): Map<string, string> {
    const tab = getActiveTab();
    return tab?.sessionRegistry || new Map();
  }

  function isPendingReattach(ptySessionId: string): boolean {
    const tab = getActiveTab();
    return tab?.pendingReattach.has(ptySessionId) || false;
  }

  function clearPendingReattach(ptySessionId: string) {
    const tab = getActiveTab();
    if (tab) tab.pendingReattach.delete(ptySessionId);
  }

  // Navigate between panes
  function focusNextPane() {
    const terminals = allTerminals.value;
    if (terminals.length <= 1) return;

    const currentIndex = terminals.findIndex((t) => t.id === activePane.value);
    const nextIndex = (currentIndex + 1) % terminals.length;
    activePane.value = terminals[nextIndex]?.id;
  }

  function focusPreviousPane() {
    const terminals = allTerminals.value;
    if (terminals.length <= 1) return;

    const currentIndex = terminals.findIndex((t) => t.id === activePane.value);
    const prevIndex = currentIndex <= 0 ? terminals.length - 1 : currentIndex - 1;
    activePane.value = terminals[prevIndex]?.id;
  }

  // Navigate between tabs
  function nextTab() {
    const index = tabs.value.findIndex((t) => t.id === activeTabId.value);
    const nextIndex = (index + 1) % tabs.value.length;
    setActiveTab(tabs.value[nextIndex].id);
  }

  function previousTab() {
    const index = tabs.value.findIndex((t) => t.id === activeTabId.value);
    const prevIndex = index <= 0 ? tabs.value.length - 1 : index - 1;
    setActiveTab(tabs.value[prevIndex].id);
  }

  function switchToTab(index: number) {
    if (index >= 0 && index < tabs.value.length) {
      setActiveTab(tabs.value[index].id);
    }
  }

  // Search toggle signal — incremented to trigger watchers in active TerminalPane
  const searchToggleSignal = ref(0);
  function triggerSearchToggle() {
    searchToggleSignal.value++;
  }

  return {
    // State
    tabs,
    activeTabId,
    rootLayout,
    activePane,

    // Getters
    terminalCount,
    activeTabTerminalCount,
    allTerminals,

    // Tab actions
    addTab,
    closeTab,
    setActiveTab,
    renameTab,
    moveTab,
    duplicateTab,
    nextTab,
    previousTab,
    switchToTab,

    // Pane actions
    splitPane,
    splitHorizontal,
    splitVertical,
    applyPreset,
    closePane,
    setActivePane,
    updateRatio: updateRatioAction,
    updateTerminalTitle,
    setLayout,
    getLayout,
    focusNextPane,
    focusPreviousPane,

    // Search
    searchToggleSignal,
    triggerSearchToggle,

    // Session registry
    registerSession,
    unregisterSession,
    getSessionId,
    getAllSessionMappings,
    isPendingReattach,
    clearPendingReattach,
  };
});
