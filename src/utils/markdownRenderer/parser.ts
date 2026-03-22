import type { ParsedLine } from './types';

// Patterns for markdown detection
const PATTERNS = {
  header: /^(#{1,6})\s+(.+)$/,
  codeFence: /^(`{3,}|~{3,})(\w*)\s*$/,
  bulletList: /^(\s*)([-*+])\s+(.*)$/,
  numberedList: /^(\s*)(\d+)\.\s+(.*)$/,
  blockquote: /^(>+)\s*(.*)$/,
  horizontalRule: /^[-*_]{3,}\s*$/,
  empty: /^\s*$/,
};

// Inline patterns
const INLINE_PATTERNS = {
  boldItalic: /\*\*\*(.+?)\*\*\*/g,
  bold: /\*\*(.+?)\*\*|__(.+?)__/g,
  italic: /(?<!\*)\*([^*]+?)\*(?!\*)|(?<!_)_([^_]+?)_(?!_)/g,
  code: /`([^`]+?)`/g,
  strikethrough: /~~(.+?)~~/g,
  link: /\[([^\]]+)\]\(([^)]+)\)/g,
};

/**
 * Parse a single line and determine its type
 */
export function parseLine(line: string, inCodeBlock: boolean): ParsedLine {
  // If in code block, everything is code block content
  if (inCodeBlock) {
    // Check for closing fence
    const fenceMatch = line.match(PATTERNS.codeFence);
    if (fenceMatch) {
      return {
        type: 'codeBlockEnd',
        content: line,
        fence: fenceMatch[1],
      };
    }
    return {
      type: 'codeBlockContent',
      content: line,
    };
  }

  // Empty line
  if (PATTERNS.empty.test(line)) {
    return { type: 'empty', content: line };
  }

  // Code fence start
  const fenceMatch = line.match(PATTERNS.codeFence);
  if (fenceMatch) {
    return {
      type: 'codeBlockStart',
      content: line,
      fence: fenceMatch[1],
      lang: fenceMatch[2] || undefined,
    };
  }

  // Header
  const headerMatch = line.match(PATTERNS.header);
  if (headerMatch) {
    return {
      type: 'header',
      content: headerMatch[2],
      level: headerMatch[1].length,
    };
  }

  // Horizontal rule
  if (PATTERNS.horizontalRule.test(line)) {
    return { type: 'horizontalRule', content: line };
  }

  // Blockquote
  const blockquoteMatch = line.match(PATTERNS.blockquote);
  if (blockquoteMatch) {
    return {
      type: 'blockquote',
      content: blockquoteMatch[2],
      level: blockquoteMatch[1].length,
    };
  }

  // Bullet list
  const bulletMatch = line.match(PATTERNS.bulletList);
  if (bulletMatch) {
    return {
      type: 'list',
      content: bulletMatch[3],
      level: Math.floor(bulletMatch[1].length / 2),
      ordered: false,
    };
  }

  // Numbered list
  const numberedMatch = line.match(PATTERNS.numberedList);
  if (numberedMatch) {
    return {
      type: 'list',
      content: numberedMatch[3],
      level: Math.floor(numberedMatch[1].length / 2),
      ordered: true,
      number: parseInt(numberedMatch[2], 10),
    };
  }

  // Default to paragraph
  return { type: 'paragraph', content: line };
}

/**
 * Check if text contains any markdown patterns
 */
export function containsMarkdown(text: string): boolean {
  // Quick check for common markdown characters
  if (!/[#*_`\[\]~>\-]/.test(text)) {
    return false;
  }

  // Check for actual patterns
  return (
    PATTERNS.header.test(text) ||
    PATTERNS.codeFence.test(text) ||
    PATTERNS.bulletList.test(text) ||
    PATTERNS.numberedList.test(text) ||
    PATTERNS.blockquote.test(text) ||
    PATTERNS.horizontalRule.test(text) ||
    INLINE_PATTERNS.bold.test(text) ||
    INLINE_PATTERNS.italic.test(text) ||
    INLINE_PATTERNS.code.test(text) ||
    INLINE_PATTERNS.link.test(text)
  );
}

/**
 * Extract inline elements and their positions
 */
export interface InlineElement {
  type: 'bold' | 'italic' | 'boldItalic' | 'code' | 'strikethrough' | 'link';
  start: number;
  end: number;
  content: string;
  url?: string; // For links
}

export function extractInlineElements(text: string): InlineElement[] {
  const elements: InlineElement[] = [];

  // Bold italic (must check first)
  for (const match of text.matchAll(/\*\*\*(.+?)\*\*\*/g)) {
    elements.push({
      type: 'boldItalic',
      start: match.index!,
      end: match.index! + match[0].length,
      content: match[1],
    });
  }

  // Bold
  for (const match of text.matchAll(/\*\*(.+?)\*\*|__(.+?)__/g)) {
    const content = match[1] || match[2];
    // Skip if overlaps with boldItalic
    const overlaps = elements.some(
      (e) => e.type === 'boldItalic' && match.index! >= e.start && match.index! < e.end
    );
    if (!overlaps) {
      elements.push({
        type: 'bold',
        start: match.index!,
        end: match.index! + match[0].length,
        content,
      });
    }
  }

  // Italic (careful not to match inside bold)
  for (const match of text.matchAll(/(?<!\*)\*([^*]+?)\*(?!\*)|(?<!_)_([^_]+?)_(?!_)/g)) {
    const content = match[1] || match[2];
    const overlaps = elements.some(
      (e) => match.index! >= e.start && match.index! < e.end
    );
    if (!overlaps) {
      elements.push({
        type: 'italic',
        start: match.index!,
        end: match.index! + match[0].length,
        content,
      });
    }
  }

  // Code
  for (const match of text.matchAll(/`([^`]+?)`/g)) {
    elements.push({
      type: 'code',
      start: match.index!,
      end: match.index! + match[0].length,
      content: match[1],
    });
  }

  // Strikethrough
  for (const match of text.matchAll(/~~(.+?)~~/g)) {
    elements.push({
      type: 'strikethrough',
      start: match.index!,
      end: match.index! + match[0].length,
      content: match[1],
    });
  }

  // Links
  for (const match of text.matchAll(/\[([^\]]+)\]\(([^)]+)\)/g)) {
    elements.push({
      type: 'link',
      start: match.index!,
      end: match.index! + match[0].length,
      content: match[1],
      url: match[2],
    });
  }

  // Sort by position
  elements.sort((a, b) => a.start - b.start);

  return elements;
}
