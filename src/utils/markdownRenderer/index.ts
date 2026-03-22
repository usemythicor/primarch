import type { Theme } from '../../types';
import type { MarkdownRendererOptions, MarkdownSessionState, AnsiTheme } from './types';
import { createSessionState, processBuffer, flushBuffer } from './buffer';
import { containsMarkdown } from './parser';
import { createAnsiTheme, renderMarkdown, preserveAnsi, restoreAnsi } from './renderer';

const DEFAULT_TRIGGER_COMMANDS = ['claude', 'glow', 'mdcat', 'bat', 'rich'];

export class MarkdownRenderer {
  private ansiTheme: AnsiTheme;
  private enabled: boolean;
  private triggerCommands: string[];
  private state: MarkdownSessionState;
  private flushTimeout: ReturnType<typeof setTimeout> | null = null;
  private onFlush: ((data: string) => void) | null = null;

  constructor(options: MarkdownRendererOptions) {
    this.ansiTheme = createAnsiTheme(options.theme);
    this.enabled = options.enabled ?? true;
    this.triggerCommands = options.triggerCommands ?? DEFAULT_TRIGGER_COMMANDS;
    this.state = createSessionState(false);
  }

  /**
   * Update the theme (e.g., when user changes settings)
   */
  updateTheme(theme: Theme): void {
    this.ansiTheme = createAnsiTheme(theme);
  }

  /**
   * Enable or disable markdown rendering
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
    if (!enabled) {
      this.state.enabled = false;
    }
  }

  /**
   * Check if a command should trigger markdown mode
   */
  shouldTrigger(command: string): boolean {
    const cmd = command.trim().split(/\s+/)[0].toLowerCase();
    return this.triggerCommands.some((trigger) => cmd === trigger || cmd.endsWith('/' + trigger));
  }

  /**
   * Notify renderer that a command was entered
   */
  onCommand(command: string): void {
    if (this.enabled && this.shouldTrigger(command)) {
      this.state.enabled = true;
    }
  }

  /**
   * Notify renderer that command output has ended (shell prompt returned)
   */
  onPrompt(): void {
    this.state.enabled = false;
    this.state.inCodeBlock = false;
    this.state.codeBlockFence = '';
    this.state.codeBlockLang = '';
  }

  /**
   * Set callback for flushed buffer content
   */
  setFlushCallback(callback: (data: string) => void): void {
    this.onFlush = callback;
  }

  /**
   * Process a chunk of terminal output
   */
  process(chunk: string): string {
    // Clear any pending flush timeout
    if (this.flushTimeout) {
      clearTimeout(this.flushTimeout);
      this.flushTimeout = null;
    }

    // Pass through real-time status updates (carriage returns without newlines)
    // These are used by CLIs like claude for spinner animations and progress updates
    if (chunk.includes('\r') && !chunk.includes('\n')) {
      return chunk;
    }

    // Pass through chunks with cursor control sequences (used for in-place updates)
    // Matches: cursor movement, clear line, save/restore cursor
    if (/\x1b\[[\d;]*[ABCDEFGJKST]|\x1b\[[\d;]*[su]|\x1b\[\?25[hl]/.test(chunk)) {
      return chunk;
    }

    // If markdown rendering is disabled or not active, pass through
    if (!this.enabled || !this.state.enabled) {
      // Still check for markdown patterns to auto-enable
      if (this.enabled && containsMarkdown(chunk)) {
        // Heuristic: if output contains markdown, enable rendering
        this.state.enabled = true;
      } else {
        return chunk;
      }
    }

    // Preserve any existing ANSI sequences from the PTY
    const { preserved, stripped } = preserveAnsi(chunk);

    // Process buffer with the stripped content
    const { output, state: newState } = processBuffer(stripped, this.state);
    this.state = newState;

    // If there's remaining buffer, schedule a flush
    if (this.state.buffer.length > 0) {
      this.flushTimeout = setTimeout(() => {
        const { output: flushed, state: flushedState } = flushBuffer(this.state);
        this.state = flushedState;
        if (flushed && this.onFlush) {
          const rendered = this.renderChunk(flushed);
          this.onFlush(rendered);
        }
      }, 100);
    }

    // Render the complete output
    if (!output) {
      return '';
    }

    const rendered = this.renderChunk(output);

    // Restore preserved ANSI sequences
    return restoreAnsi(rendered, preserved);
  }

  /**
   * Render a chunk of text to ANSI
   */
  private renderChunk(text: string): string {
    const { rendered, inCodeBlock } = renderMarkdown(
      text,
      this.ansiTheme,
      this.state.inCodeBlock
    );
    this.state.inCodeBlock = inCodeBlock;
    return rendered;
  }

  /**
   * Force flush any buffered content
   */
  flush(): string {
    if (this.flushTimeout) {
      clearTimeout(this.flushTimeout);
      this.flushTimeout = null;
    }

    const { output, state } = flushBuffer(this.state);
    this.state = state;

    if (!output) {
      return '';
    }

    return this.renderChunk(output);
  }

  /**
   * Reset renderer state
   */
  reset(): void {
    this.state = createSessionState(false);
    if (this.flushTimeout) {
      clearTimeout(this.flushTimeout);
      this.flushTimeout = null;
    }
  }

  /**
   * Check if markdown mode is currently active
   */
  isActive(): boolean {
    return this.state.enabled;
  }
}

// Re-export types
export type { MarkdownRendererOptions, MarkdownSessionState, AnsiTheme } from './types';
