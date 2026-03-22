import type { BufferResult, MarkdownSessionState } from './types';

const MAX_BUFFER_SIZE = 65536; // 64KB max buffer

/**
 * Extract complete lines from buffer, leaving incomplete line in remaining
 */
export function extractCompleteLines(buffer: string): BufferResult {
  const lastNewline = buffer.lastIndexOf('\n');
  if (lastNewline === -1) {
    return { complete: '', remaining: buffer };
  }
  return {
    complete: buffer.slice(0, lastNewline + 1),
    remaining: buffer.slice(lastNewline + 1),
  };
}

/**
 * Check if a line is a code fence (``` or ~~~)
 */
export function isCodeFence(line: string): { isFence: boolean; fence: string; lang: string } {
  const trimmed = line.trim();
  const match = trimmed.match(/^(`{3,}|~{3,})(\w*)\s*$/);
  if (match) {
    return { isFence: true, fence: match[1], lang: match[2] || '' };
  }
  return { isFence: false, fence: '', lang: '' };
}

/**
 * Check if a line closes the current code block
 */
export function isCodeFenceClose(line: string, openFence: string): boolean {
  const trimmed = line.trim();
  const fenceChar = openFence[0];
  const fenceLength = openFence.length;
  // Closing fence must use same char and be at least as long
  const regex = new RegExp(`^${fenceChar}{${fenceLength},}\\s*$`);
  return regex.test(trimmed);
}

/**
 * Process buffer and handle code block state
 */
export function processBuffer(
  chunk: string,
  state: MarkdownSessionState
): { output: string; state: MarkdownSessionState } {
  // Add chunk to buffer
  let buffer = state.buffer + chunk;

  // Enforce max buffer size
  if (buffer.length > MAX_BUFFER_SIZE) {
    // Flush everything if buffer too large
    const output = buffer;
    return {
      output,
      state: { ...state, buffer: '', inCodeBlock: false, codeBlockFence: '', codeBlockLang: '' },
    };
  }

  // Extract complete lines
  const { complete, remaining } = extractCompleteLines(buffer);

  // Process complete lines
  const lines = complete.split('\n');
  const processedLines: string[] = [];
  let currentState = { ...state };

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Skip empty line at end (from split)
    if (i === lines.length - 1 && line === '') {
      continue;
    }

    if (currentState.inCodeBlock) {
      // Check for closing fence
      if (isCodeFenceClose(line, currentState.codeBlockFence)) {
        currentState.inCodeBlock = false;
        currentState.codeBlockFence = '';
        currentState.codeBlockLang = '';
      }
      processedLines.push(line);
    } else {
      // Check for opening fence
      const { isFence, fence, lang } = isCodeFence(line);
      if (isFence) {
        currentState.inCodeBlock = true;
        currentState.codeBlockFence = fence;
        currentState.codeBlockLang = lang;
      }
      processedLines.push(line);
    }
  }

  // Rejoin with newlines (add trailing newline since we split on it)
  const output = processedLines.length > 0 ? processedLines.join('\n') + '\n' : '';

  return {
    output,
    state: { ...currentState, buffer: remaining },
  };
}

/**
 * Create initial session state
 */
export function createSessionState(enabled: boolean = false): MarkdownSessionState {
  return {
    enabled,
    buffer: '',
    inCodeBlock: false,
    codeBlockFence: '',
    codeBlockLang: '',
  };
}

/**
 * Flush any remaining buffer content
 */
export function flushBuffer(state: MarkdownSessionState): { output: string; state: MarkdownSessionState } {
  const output = state.buffer;
  return {
    output,
    state: { ...state, buffer: '' },
  };
}
