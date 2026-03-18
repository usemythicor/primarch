import type { LayoutNode } from '../../types';

/**
 * Generate a unique ID for layout nodes
 */
export function generateId(): string {
  return Math.random().toString(36).substring(2, 11);
}

/**
 * Create a new terminal node
 */
export function createTerminalNode(options?: {
  shell?: string;
  cwd?: string;
  startupCommand?: string;
  title?: string;
}): LayoutNode {
  return {
    id: generateId(),
    type: 'terminal',
    shell: options?.shell,
    cwd: options?.cwd,
    startupCommand: options?.startupCommand,
    title: options?.title,
  };
}

/**
 * Create a new split node
 */
export function createSplitNode(
  direction: 'horizontal' | 'vertical',
  first: LayoutNode,
  second: LayoutNode,
  ratio: number = 0.5
): LayoutNode {
  return {
    id: generateId(),
    type: 'split',
    direction,
    ratio,
    children: [first, second],
  };
}

/**
 * Find a node by ID in the layout tree
 */
export function findNode(root: LayoutNode, id: string): LayoutNode | null {
  if (root.id === id) {
    return root;
  }
  if (root.type === 'split' && root.children) {
    for (const child of root.children) {
      const found = findNode(child, id);
      if (found) return found;
    }
  }
  return null;
}

/**
 * Find the parent of a node by ID
 */
export function findParent(root: LayoutNode, id: string): LayoutNode | null {
  if (root.type === 'split' && root.children) {
    for (const child of root.children) {
      if (child.id === id) {
        return root;
      }
      const found = findParent(child, id);
      if (found) return found;
    }
  }
  return null;
}

/**
 * Split a terminal node into two panes
 */
export function splitNode(
  root: LayoutNode,
  targetId: string,
  direction: 'horizontal' | 'vertical',
  newNodeOptions?: {
    shell?: string;
    cwd?: string;
    startupCommand?: string;
  }
): LayoutNode {
  const target = findNode(root, targetId);
  if (!target || target.type !== 'terminal') {
    return root;
  }

  const parent = findParent(root, targetId);
  const newTerminal = createTerminalNode(newNodeOptions);
  const newSplit = createSplitNode(direction, { ...target }, newTerminal);

  if (!parent) {
    // Target is the root node
    return newSplit;
  }

  // Replace the target with the new split node in the parent
  if (parent.children) {
    const index = parent.children.findIndex((c) => c.id === targetId);
    if (index !== -1) {
      parent.children[index] = newSplit;
    }
  }

  return root;
}

/**
 * Close a terminal node and clean up the tree
 */
export function closeNode(root: LayoutNode, targetId: string): LayoutNode | null {
  if (root.id === targetId) {
    // Closing the root node - return null to indicate no more nodes
    return null;
  }

  const parent = findParent(root, targetId);
  if (!parent || !parent.children) {
    return root;
  }

  // Find the sibling node
  const targetIndex = parent.children.findIndex((c) => c.id === targetId);
  const siblingIndex = targetIndex === 0 ? 1 : 0;
  const sibling = parent.children[siblingIndex];

  // Find the grandparent to replace parent with sibling
  const grandparent = findParent(root, parent.id!);

  if (!grandparent) {
    // Parent is the root, so sibling becomes the new root
    return sibling;
  }

  // Replace parent with sibling in grandparent
  if (grandparent.children) {
    const parentIndex = grandparent.children.findIndex((c) => c.id === parent.id);
    if (parentIndex !== -1) {
      grandparent.children[parentIndex] = sibling;
    }
  }

  return root;
}

/**
 * Update the ratio of a split node
 */
export function updateRatio(root: LayoutNode, splitId: string, ratio: number): LayoutNode {
  const node = findNode(root, splitId);
  if (node && node.type === 'split') {
    node.ratio = Math.max(0.1, Math.min(0.9, ratio));
  }
  return root;
}

/**
 * Get all terminal nodes in the tree
 */
export function getAllTerminals(root: LayoutNode): LayoutNode[] {
  const terminals: LayoutNode[] = [];

  function traverse(node: LayoutNode) {
    if (node.type === 'terminal') {
      terminals.push(node);
    } else if (node.type === 'split' && node.children) {
      node.children.forEach(traverse);
    }
  }

  traverse(root);
  return terminals;
}

/**
 * Count total terminal nodes
 */
export function countTerminals(root: LayoutNode): number {
  return getAllTerminals(root).length;
}

/**
 * Deep clone a layout tree
 */
export function cloneLayout(node: LayoutNode): LayoutNode {
  if (node.type === 'terminal') {
    return { ...node, id: generateId() };
  }

  return {
    ...node,
    id: generateId(),
    children: node.children?.map(cloneLayout) as [LayoutNode, LayoutNode],
  };
}

/**
 * Serialize layout to JSON (for saving workspaces)
 */
export function serializeLayout(root: LayoutNode): string {
  return JSON.stringify(root);
}

/**
 * Deserialize layout from JSON
 */
export function deserializeLayout(json: string): LayoutNode {
  return JSON.parse(json);
}
