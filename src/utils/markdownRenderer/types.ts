import type { Theme } from '../../types';

export interface MarkdownRendererOptions {
  theme: Theme;
  enabled?: boolean;
  triggerCommands?: string[];
}

export interface MarkdownSessionState {
  enabled: boolean;
  buffer: string;
  inCodeBlock: boolean;
  codeBlockFence: string;
  codeBlockLang: string;
}

export interface AnsiTheme {
  reset: string;
  bold: string;
  italic: string;
  underline: string;
  dim: string;
  header1: string;
  header2: string;
  header3: string;
  header4: string;
  code: string;
  codeBlock: string;
  codeBlockBorder: string;
  link: string;
  linkUrl: string;
  listBullet: string;
  blockquote: string;
  horizontalRule: string;
  strikethrough: string;
}

export interface ParsedLine {
  type: 'header' | 'code' | 'codeBlockStart' | 'codeBlockEnd' | 'codeBlockContent' |
        'list' | 'blockquote' | 'horizontalRule' | 'paragraph' | 'empty';
  content: string;
  level?: number;        // For headers (1-6) and lists (indent level)
  lang?: string;         // For code blocks
  fence?: string;        // For code block fence (``` or ~~~)
  ordered?: boolean;     // For lists
  number?: number;       // For ordered lists
}

export interface BufferResult {
  complete: string;
  remaining: string;
}
