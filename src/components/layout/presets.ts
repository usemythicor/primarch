import type { LayoutNode } from '../../types';
import { createTerminalNode, createSplitNode } from './LayoutTree';

export interface LayoutPreset {
  id: string;
  name: string;
  terminalCount: number;
  build: (cwd?: string) => LayoutNode;
}

function t(cwd?: string) {
  return createTerminalNode({ cwd });
}

export const layoutPresets: LayoutPreset[] = [
  {
    id: '2-col',
    name: '2 Columns',
    terminalCount: 2,
    build: (cwd) => createSplitNode('horizontal', t(cwd), t(cwd)),
  },
  {
    id: '3-col',
    name: '3 Columns',
    terminalCount: 3,
    build: (cwd) =>
      createSplitNode(
        'horizontal',
        t(cwd),
        createSplitNode('horizontal', t(cwd), t(cwd)),
        0.333,
      ),
  },
  {
    id: '2x2',
    name: '2x2 Grid',
    terminalCount: 4,
    build: (cwd) =>
      createSplitNode(
        'vertical',
        createSplitNode('horizontal', t(cwd), t(cwd)),
        createSplitNode('horizontal', t(cwd), t(cwd)),
      ),
  },
  {
    id: '2x2-bottom',
    name: '2x2 + Bottom',
    terminalCount: 5,
    build: (cwd) =>
      createSplitNode(
        'vertical',
        createSplitNode(
          'vertical',
          createSplitNode('horizontal', t(cwd), t(cwd)),
          createSplitNode('horizontal', t(cwd), t(cwd)),
        ),
        t(cwd),
        0.667,
      ),
  },
  {
    id: '3x2',
    name: '3x2 Grid',
    terminalCount: 6,
    build: (cwd) =>
      createSplitNode(
        'vertical',
        createSplitNode(
          'horizontal',
          t(cwd),
          createSplitNode('horizontal', t(cwd), t(cwd)),
          0.333,
        ),
        createSplitNode(
          'horizontal',
          t(cwd),
          createSplitNode('horizontal', t(cwd), t(cwd)),
          0.333,
        ),
      ),
  },
];
