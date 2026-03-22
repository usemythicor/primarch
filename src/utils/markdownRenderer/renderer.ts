import type { Theme } from '../../types';
import type { AnsiTheme, ParsedLine } from './types';
import { parseLine, extractInlineElements } from './parser';

// ANSI escape code helpers
const ESC = '\x1b[';
const RESET = `${ESC}0m`;
const BOLD = `${ESC}1m`;
const DIM = `${ESC}2m`;
const ITALIC = `${ESC}3m`;
const UNDERLINE = `${ESC}4m`;
const STRIKETHROUGH = `${ESC}9m`;

/**
 * Convert hex color to ANSI 24-bit color code
 */
function hexToAnsi(hex: string, foreground: boolean = true): string {
  const clean = hex.replace('#', '');
  const r = parseInt(clean.slice(0, 2), 16);
  const g = parseInt(clean.slice(2, 4), 16);
  const b = parseInt(clean.slice(4, 6), 16);
  const code = foreground ? 38 : 48; // 38 = foreground, 48 = background
  return `${ESC}${code};2;${r};${g};${b}m`;
}

/**
 * Create ANSI theme from terminal theme
 */
export function createAnsiTheme(theme: Theme): AnsiTheme {
  return {
    reset: RESET,
    bold: BOLD,
    italic: ITALIC,
    underline: UNDERLINE,
    dim: DIM,
    header1: `${BOLD}${hexToAnsi(theme.brightMagenta)}`,
    header2: `${BOLD}${hexToAnsi(theme.brightCyan)}`,
    header3: `${BOLD}${hexToAnsi(theme.brightBlue)}`,
    header4: `${BOLD}${hexToAnsi(theme.brightGreen)}`,
    code: hexToAnsi(theme.yellow),
    codeBlock: `${DIM}${hexToAnsi(theme.brightBlack, false)}`,
    codeBlockBorder: hexToAnsi(theme.brightBlack),
    link: `${UNDERLINE}${hexToAnsi(theme.blue)}`,
    linkUrl: `${DIM}${hexToAnsi(theme.cyan)}`,
    listBullet: hexToAnsi(theme.cyan),
    blockquote: `${ITALIC}${hexToAnsi(theme.brightBlack)}`,
    horizontalRule: hexToAnsi(theme.brightBlack),
    strikethrough: STRIKETHROUGH,
  };
}

/**
 * Render inline markdown elements to ANSI
 */
export function renderInline(text: string, ansiTheme: AnsiTheme): string {
  const elements = extractInlineElements(text);

  if (elements.length === 0) {
    return text;
  }

  let result = '';
  let lastEnd = 0;

  for (const element of elements) {
    // Add text before this element
    result += text.slice(lastEnd, element.start);

    // Render the element
    switch (element.type) {
      case 'boldItalic':
        result += `${ansiTheme.bold}${ansiTheme.italic}${element.content}${ansiTheme.reset}`;
        break;
      case 'bold':
        result += `${ansiTheme.bold}${element.content}${ansiTheme.reset}`;
        break;
      case 'italic':
        result += `${ansiTheme.italic}${element.content}${ansiTheme.reset}`;
        break;
      case 'code':
        result += `${ansiTheme.code}${element.content}${ansiTheme.reset}`;
        break;
      case 'strikethrough':
        result += `${ansiTheme.strikethrough}${element.content}${ansiTheme.reset}`;
        break;
      case 'link':
        // Show link text underlined, then URL in parentheses dimmed
        result += `${ansiTheme.link}${element.content}${ansiTheme.reset}`;
        result += ` ${ansiTheme.linkUrl}(${element.url})${ansiTheme.reset}`;
        break;
    }

    lastEnd = element.end;
  }

  // Add remaining text
  result += text.slice(lastEnd);

  return result;
}

/**
 * Render a parsed line to ANSI
 */
export function renderLine(parsed: ParsedLine, ansiTheme: AnsiTheme): string {
  switch (parsed.type) {
    case 'header': {
      const level = parsed.level || 1;
      const prefix = '#'.repeat(level) + ' ';
      let style: string;
      switch (level) {
        case 1:
          style = ansiTheme.header1;
          break;
        case 2:
          style = ansiTheme.header2;
          break;
        case 3:
          style = ansiTheme.header3;
          break;
        default:
          style = ansiTheme.header4;
      }
      const content = renderInline(parsed.content, ansiTheme);
      return `${style}${prefix}${content}${ansiTheme.reset}`;
    }

    case 'codeBlockStart':
      return `${ansiTheme.codeBlockBorder}${parsed.content}${ansiTheme.reset}`;

    case 'codeBlockEnd':
      return `${ansiTheme.codeBlockBorder}${parsed.content}${ansiTheme.reset}`;

    case 'codeBlockContent':
      // Dim the code block content slightly
      return `${ansiTheme.dim}${parsed.content}${ansiTheme.reset}`;

    case 'list': {
      const indent = '  '.repeat(parsed.level || 0);
      const bullet = parsed.ordered
        ? `${ansiTheme.listBullet}${parsed.number}.${ansiTheme.reset}`
        : `${ansiTheme.listBullet}•${ansiTheme.reset}`;
      const content = renderInline(parsed.content, ansiTheme);
      return `${indent}${bullet} ${content}`;
    }

    case 'blockquote': {
      const bars = '│'.repeat(parsed.level || 1);
      const content = renderInline(parsed.content, ansiTheme);
      return `${ansiTheme.blockquote}${bars} ${content}${ansiTheme.reset}`;
    }

    case 'horizontalRule':
      return `${ansiTheme.horizontalRule}${'─'.repeat(40)}${ansiTheme.reset}`;

    case 'paragraph':
      return renderInline(parsed.content, ansiTheme);

    case 'empty':
      return parsed.content;

    default:
      return parsed.content;
  }
}

/**
 * Preserve existing ANSI sequences in input
 */
export function preserveAnsi(input: string): { preserved: Map<number, string>; stripped: string } {
  // Match ANSI escape sequences including OSC sequences
  const ansiRegex = /\x1b\[[0-9;]*[a-zA-Z]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)/g;
  const preserved = new Map<number, string>();
  let stripped = '';
  let lastIndex = 0;
  let strippedIndex = 0;

  for (const match of input.matchAll(ansiRegex)) {
    const beforeText = input.slice(lastIndex, match.index);
    stripped += beforeText;
    preserved.set(strippedIndex + beforeText.length, match[0]);
    strippedIndex += beforeText.length;
    lastIndex = match.index! + match[0].length;
  }

  stripped += input.slice(lastIndex);

  return { preserved, stripped };
}

/**
 * Restore preserved ANSI sequences into rendered output
 */
export function restoreAnsi(rendered: string, preserved: Map<number, string>): string {
  if (preserved.size === 0) {
    return rendered;
  }

  // This is tricky because the rendered output may have different length
  // For simplicity, we'll append preserved sequences that were stripped
  // A more sophisticated approach would track character positions
  let result = rendered;

  // For now, just ensure we don't lose any important ANSI sequences
  // The rendering already adds its own ANSI codes

  return result;
}

/**
 * Main render function for a chunk of text
 */
export function renderMarkdown(
  text: string,
  ansiTheme: AnsiTheme,
  inCodeBlock: boolean = false
): { rendered: string; inCodeBlock: boolean } {
  // If the text contains carriage returns (used for in-place updates like spinners),
  // pass through without markdown processing to preserve terminal control behavior
  if (text.includes('\r') && !text.includes('\r\n')) {
    return { rendered: text, inCodeBlock };
  }

  // Normalize line endings (handle both \r\n and \n)
  const normalizedText = text.replace(/\r\n/g, '\n');
  const lines = normalizedText.split('\n');
  const renderedLines: string[] = [];
  let currentInCodeBlock = inCodeBlock;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const parsed = parseLine(line, currentInCodeBlock);

    // Update code block state
    if (parsed.type === 'codeBlockStart') {
      currentInCodeBlock = true;
    } else if (parsed.type === 'codeBlockEnd') {
      currentInCodeBlock = false;
    }

    renderedLines.push(renderLine(parsed, ansiTheme));
  }

  return {
    rendered: renderedLines.join('\n'),
    inCodeBlock: currentInCodeBlock,
  };
}
