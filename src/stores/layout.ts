import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { ref, computed } from 'vue';
import type { LayoutNode } from '../types';
import {
  createTerminalNode,
  splitNode,
  closeNode,
  updateRatio,
  findNode,
  getAllTerminals,
  countTerminals,
} from '../components/layout/LayoutTree';

export const useLayoutStore = defineStore('layout', () => {
  // State
  const rootLayout = ref<LayoutNode>(createTerminalNode());
  const activePane = ref<string | undefined>(rootLayout.value.id);
  // Maps terminal node IDs to their PTY session IDs
  const sessionRegistry = ref<Map<string, string>>(new Map());

  // Getters
  const terminalCount = computed(() => countTerminals(rootLayout.value));
  const allTerminals = computed(() => getAllTerminals(rootLayout.value));

  // Actions
  async function splitPane(
    direction: 'horizontal' | 'vertical',
    targetId?: string,
    options?: {
      shell?: string;
      cwd?: string;
      startupCommand?: string;
    }
  ) {
    const target = targetId || activePane.value;
    if (!target) return;

    // If no cwd provided, try to get it from the active terminal
    let finalOptions = options || {};
    if (!finalOptions.cwd && target) {
      const existingNode = findNode(rootLayout.value, target);
      const sessionId = sessionRegistry.value.get(target);

      if (sessionId) {
        try {
          const cwd = await invoke<string>('get_terminal_cwd', { sessionId });
          if (cwd) {
            finalOptions = { ...finalOptions, cwd };
          }
        } catch {
          // CWD not available from PTY tracking - fall back to node's stored cwd
          if (existingNode?.type === 'terminal' && existingNode.cwd) {
            finalOptions = { ...finalOptions, cwd: existingNode.cwd };
          }
        }
      } else if (existingNode?.type === 'terminal' && existingNode.cwd) {
        // No session yet - use node's existing cwd
        finalOptions = { ...finalOptions, cwd: existingNode.cwd };
      }
    }

    // Update the existing node's cwd before splitting.
    // When Vue re-renders, both the copy of the original terminal and the
    // new terminal need the current CWD (the old session gets destroyed).
    if (finalOptions.cwd) {
      const existingNode = findNode(rootLayout.value, target);
      if (existingNode && existingNode.type === 'terminal') {
        existingNode.cwd = finalOptions.cwd;
      }
    }

    rootLayout.value = splitNode(rootLayout.value, target, direction, finalOptions);

    // Set focus to the new pane
    const terminals = getAllTerminals(rootLayout.value);
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
    const result = closeNode(rootLayout.value, targetId);

    if (result === null) {
      // All panes closed - create a new terminal
      rootLayout.value = createTerminalNode();
      activePane.value = rootLayout.value.id;
    } else {
      rootLayout.value = result;

      // If the closed pane was active, set active to first terminal
      if (activePane.value === targetId) {
        const terminals = getAllTerminals(rootLayout.value);
        activePane.value = terminals[0]?.id;
      }
    }
  }

  function setActivePane(id: string) {
    activePane.value = id;
  }

  function updateRatioAction(splitId: string, ratio: number) {
    rootLayout.value = updateRatio(rootLayout.value, splitId, ratio);
  }

  function updateTerminalTitle(terminalId: string, title: string) {
    const node = findNode(rootLayout.value, terminalId);
    if (node && node.type === 'terminal') {
      node.title = title;
    }
  }

  function setLayout(layout: LayoutNode) {
    rootLayout.value = layout;
    const terminals = getAllTerminals(rootLayout.value);
    activePane.value = terminals[0]?.id;
  }

  function getLayout(): LayoutNode {
    return rootLayout.value;
  }

  // Session registry management
  function registerSession(nodeId: string, sessionId: string) {
    sessionRegistry.value.set(nodeId, sessionId);
  }

  function unregisterSession(nodeId: string) {
    sessionRegistry.value.delete(nodeId);
  }

  function getSessionId(nodeId: string): string | undefined {
    return sessionRegistry.value.get(nodeId);
  }

  function getAllSessionMappings(): Map<string, string> {
    return sessionRegistry.value;
  }

  // Navigate between panes
  function focusNextPane() {
    const terminals = getAllTerminals(rootLayout.value);
    if (terminals.length <= 1) return;

    const currentIndex = terminals.findIndex((t) => t.id === activePane.value);
    const nextIndex = (currentIndex + 1) % terminals.length;
    activePane.value = terminals[nextIndex]?.id;
  }

  function focusPreviousPane() {
    const terminals = getAllTerminals(rootLayout.value);
    if (terminals.length <= 1) return;

    const currentIndex = terminals.findIndex((t) => t.id === activePane.value);
    const prevIndex = currentIndex <= 0 ? terminals.length - 1 : currentIndex - 1;
    activePane.value = terminals[prevIndex]?.id;
  }

  return {
    // State
    rootLayout,
    activePane,

    // Getters
    terminalCount,
    allTerminals,

    // Actions
    splitPane,
    splitHorizontal,
    splitVertical,
    closePane,
    setActivePane,
    updateRatio: updateRatioAction,
    updateTerminalTitle,
    setLayout,
    getLayout,
    focusNextPane,
    focusPreviousPane,

    // Session registry
    registerSession,
    unregisterSession,
    getSessionId,
    getAllSessionMappings,
  };
});
