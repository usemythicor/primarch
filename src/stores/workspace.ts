import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { LayoutNode, Workspace } from '../types';
import { useLayoutStore } from './layout';
import { generateId, getAllTerminals } from '../components/layout/LayoutTree';

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<Workspace[]>([]);
  const currentWorkspace = ref<Workspace | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  /**
   * Load all saved workspaces
   */
  async function loadWorkspaces() {
    isLoading.value = true;
    error.value = null;

    try {
      workspaces.value = await invoke<Workspace[]>('list_workspaces_cmd');
    } catch (e) {
      error.value = `Failed to load workspaces: ${e}`;
      console.error(error.value);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Update layout nodes with current working directories from active sessions
   */
  async function updateLayoutWithCurrentCwds(layout: LayoutNode): Promise<LayoutNode> {
    const layoutStore = useLayoutStore();
    const terminals = getAllTerminals(layout);

    // Create a deep copy of the layout
    const updatedLayout = JSON.parse(JSON.stringify(layout)) as LayoutNode;
    const updatedTerminals = getAllTerminals(updatedLayout);

    // Query each terminal's current working directory
    for (let i = 0; i < terminals.length; i++) {
      const terminal = terminals[i];
      const updatedTerminal = updatedTerminals[i];

      if (terminal.id) {
        const sessionId = layoutStore.getSessionId(terminal.id);
        if (sessionId) {
          try {
            const cwd = await invoke<string>('get_terminal_cwd', { sessionId });
            updatedTerminal.cwd = cwd;
          } catch (e) {
            console.warn(`Failed to get cwd for session ${sessionId}:`, e);
            // Keep the original cwd if we can't get the current one
          }
        }
      }
    }

    return updatedLayout;
  }

  /**
   * Save the current layout as a workspace
   */
  async function saveCurrentLayout(name: string) {
    const layoutStore = useLayoutStore();
    isLoading.value = true;
    error.value = null;

    try {
      // Get layout with updated cwds
      const layoutWithCwds = await updateLayoutWithCurrentCwds(layoutStore.rootLayout);

      const workspace: Workspace = {
        id: generateId(),
        name,
        createdAt: new Date().toISOString(),
        layout: layoutWithCwds,
      };

      await invoke('save_workspace_cmd', { workspace: convertLayoutForRust(workspace) });
      await loadWorkspaces();

      return workspace;
    } catch (e) {
      error.value = `Failed to save workspace: ${e}`;
      console.error(error.value);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Load a workspace and apply it to the layout
   */
  async function loadWorkspace(id: string) {
    const layoutStore = useLayoutStore();
    isLoading.value = true;
    error.value = null;

    try {
      const workspace = await invoke<Workspace>('load_workspace_cmd', { id });
      const converted = convertLayoutFromRust(workspace);
      layoutStore.setLayout(converted.layout);
      currentWorkspace.value = converted;

      return converted;
    } catch (e) {
      error.value = `Failed to load workspace: ${e}`;
      console.error(error.value);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Delete a workspace
   */
  async function deleteWorkspace(id: string) {
    isLoading.value = true;
    error.value = null;

    try {
      await invoke('delete_workspace_cmd', { id });
      await loadWorkspaces();

      if (currentWorkspace.value?.id === id) {
        currentWorkspace.value = null;
      }
    } catch (e) {
      error.value = `Failed to delete workspace: ${e}`;
      console.error(error.value);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Update an existing workspace with the current layout
   */
  async function updateWorkspace(id: string) {
    const layoutStore = useLayoutStore();
    const existing = workspaces.value.find((w) => w.id === id);

    if (!existing) {
      throw new Error(`Workspace ${id} not found`);
    }

    isLoading.value = true;
    error.value = null;

    try {
      // Get layout with updated cwds
      const layoutWithCwds = await updateLayoutWithCurrentCwds(layoutStore.rootLayout);

      const workspace: Workspace = {
        ...existing,
        layout: layoutWithCwds,
      };

      await invoke('save_workspace_cmd', { workspace: convertLayoutForRust(workspace) });
      await loadWorkspaces();

      return workspace;
    } catch (e) {
      error.value = `Failed to update workspace: ${e}`;
      console.error(error.value);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    workspaces,
    currentWorkspace,
    isLoading,
    error,
    loadWorkspaces,
    saveCurrentLayout,
    loadWorkspace,
    deleteWorkspace,
    updateWorkspace,
  };
});

/**
 * Convert frontend layout format to Rust format
 */
function convertLayoutForRust(workspace: Workspace): any {
  return {
    id: workspace.id,
    name: workspace.name,
    created_at: workspace.createdAt,
    updated_at: new Date().toISOString(),
    layout: convertNodeForRust(workspace.layout),
  };
}

function convertNodeForRust(node: LayoutNode): any {
  if (node.type === 'split') {
    return {
      type: 'split',
      id: node.id || generateId(),
      direction: node.direction,
      ratio: node.ratio || 0.5,
      children: node.children ? [
        convertNodeForRust(node.children[0]),
        convertNodeForRust(node.children[1]),
      ] : [],
    };
  }

  return {
    type: 'terminal',
    id: node.id || generateId(),
    shell: node.shell || null,
    cwd: node.cwd || null,
    startup_command: node.startupCommand || null,
    title: node.title || null,
  };
}

/**
 * Convert Rust layout format to frontend format
 */
function convertLayoutFromRust(workspace: any): Workspace {
  return {
    id: workspace.id,
    name: workspace.name,
    createdAt: workspace.created_at,
    layout: convertNodeFromRust(workspace.layout),
  };
}

function convertNodeFromRust(node: any): LayoutNode {
  if (node.type === 'split') {
    return {
      id: node.id,
      type: 'split',
      direction: node.direction,
      ratio: node.ratio,
      children: node.children ? [
        convertNodeFromRust(node.children[0]),
        convertNodeFromRust(node.children[1]),
      ] : undefined,
    };
  }

  return {
    id: node.id,
    type: 'terminal',
    shell: node.shell,
    cwd: node.cwd,
    startupCommand: node.startup_command,
    title: node.title,
  };
}
