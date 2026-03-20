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
  light?: boolean;
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

// Git types
export interface GitStatus {
  branch: string | null;
  upstream: string | null;
  ahead: number;
  behind: number;
  staged: FileStatus[];
  unstaged: FileStatus[];
  untracked: string[];
  conflicted: string[];
}

export interface FileStatus {
  path: string;
  status: FileStatusType;
  oldPath?: string;
}

export type FileStatusType =
  | 'Modified'
  | 'Added'
  | 'Deleted'
  | 'Renamed'
  | 'Copied'
  | 'TypeChanged';

export interface BranchInfo {
  name: string;
  isHead: boolean;
  upstream: string | null;
  ahead: number;
  behind: number;
}

export interface FileDiff {
  path: string;
  oldPath?: string;
  hunks: DiffHunk[];
  isBinary: boolean;
  additions: number;
  deletions: number;
}

export interface DiffHunk {
  header: string;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
}

export interface DiffLine {
  origin: string;
  content: string;
  oldLineno?: number;
  newLineno?: number;
}

export interface CommitInfo {
  oid: string;
  shortId: string;
  message: string;
  summary: string;
  authorName: string;
  authorEmail: string;
  timestamp: number;
  parentIds: string[];
  refs: RefInfo[];
}

export interface RefInfo {
  name: string;
  refType: 'Branch' | 'Tag' | 'RemoteBranch';
  isHead: boolean;
}
