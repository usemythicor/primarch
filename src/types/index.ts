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
  old_path?: string;
  hunks: DiffHunk[];
  is_binary: boolean;
  additions: number;
  deletions: number;
}

export interface DiffHunk {
  header: string;
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  lines: DiffLine[];
}

export interface DiffLine {
  origin: string;
  content: string;
  old_lineno?: number;
  new_lineno?: number;
}

export interface CommitInfo {
  oid: string;
  short_id: string;
  message: string;
  summary: string;
  author_name: string;
  author_email: string;
  timestamp: number;
  parent_ids: string[];
  refs: RefInfo[];
}

export interface RefInfo {
  name: string;
  ref_type: 'Branch' | 'Tag' | 'RemoteBranch';
  is_head: boolean;
}
