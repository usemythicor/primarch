export interface TerminalSession {
  id: string;
  shell?: string;
  cwd?: string;
  title?: string;
}

export interface LayoutNode {
  id?: string;
  type: 'split' | 'terminal';
  // For split nodes
  direction?: 'horizontal' | 'vertical';
  ratio?: number;
  children?: [LayoutNode, LayoutNode];
  // For terminal nodes
  sessionId?: string;
  shell?: string;
  cwd?: string;
  startupCommand?: string;
  title?: string;
}

export interface Workspace {
  id: string;
  name: string;
  createdAt: string;
  layout: LayoutNode;
}

export interface ShellProfile {
  id: string;
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  icon?: string;
}

export interface Theme {
  id: string;
  name: string;
  background: string;
  foreground: string;
  cursor: string;
  selection: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  brightBlack: string;
  brightRed: string;
  brightGreen: string;
  brightYellow: string;
  brightBlue: string;
  brightMagenta: string;
  brightCyan: string;
  brightWhite: string;
}
