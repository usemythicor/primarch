import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { ref, onUnmounted } from 'vue';
import type { TerminalSession } from '../types';

export function useTerminal() {
  const sessions = ref<Map<string, TerminalSession>>(new Map());
  const listeners = ref<Map<string, UnlistenFn[]>>(new Map());

  /**
   * Create a new terminal session
   */
  async function createSession(shell?: string, cwd?: string): Promise<string> {
    const sessionId = await invoke<string>('create_terminal', { shell, cwd });

    sessions.value.set(sessionId, {
      id: sessionId,
      shell,
      cwd,
    });

    return sessionId;
  }

  /**
   * Start listening for terminal output
   */
  async function startReading(
    sessionId: string,
    onData: (data: string) => void,
    onClose?: () => void,
    onError?: (error: string) => void
  ): Promise<void> {
    // Start the reader on the Rust side
    await invoke('start_terminal_reader', { sessionId });

    // Listen for data events
    const unlistenData = await listen<string>(`terminal-data-${sessionId}`, (event) => {
      onData(event.payload);
    });

    // Listen for close events
    const unlistenClose = await listen(`terminal-closed-${sessionId}`, () => {
      onClose?.();
      cleanup(sessionId);
    });

    // Listen for error events
    const unlistenError = await listen<string>(`terminal-error-${sessionId}`, (event) => {
      onError?.(event.payload);
    });

    // Store listeners for cleanup
    listeners.value.set(sessionId, [unlistenData, unlistenClose, unlistenError]);
  }

  /**
   * Reattach to an existing PTY session (listen for events without starting a new reader thread)
   */
  async function reattachReading(
    sessionId: string,
    onData: (data: string) => void,
    onClose?: () => void,
    onError?: (error: string) => void
  ): Promise<void> {
    const unlistenData = await listen<string>(`terminal-data-${sessionId}`, (event) => {
      onData(event.payload);
    });

    const unlistenClose = await listen(`terminal-closed-${sessionId}`, () => {
      onClose?.();
      cleanup(sessionId);
    });

    const unlistenError = await listen<string>(`terminal-error-${sessionId}`, (event) => {
      onError?.(event.payload);
    });

    listeners.value.set(sessionId, [unlistenData, unlistenClose, unlistenError]);
  }

  /**
   * Write data to a terminal session
   */
  async function write(sessionId: string, data: string): Promise<void> {
    await invoke('write_terminal', { sessionId, data });
  }

  /**
   * Resize a terminal session
   */
  async function resize(sessionId: string, cols: number, rows: number): Promise<void> {
    await invoke('resize_terminal', { sessionId, cols, rows });
  }

  /**
   * Kill a terminal session
   */
  async function kill(sessionId: string): Promise<void> {
    await invoke('kill_terminal', { sessionId });
    cleanup(sessionId);
  }

  /**
   * Get the current working directory of a terminal session
   */
  async function getCwd(sessionId: string): Promise<string> {
    return await invoke<string>('get_terminal_cwd', { sessionId });
  }

  /**
   * Clean up listeners for a session
   */
  function cleanup(sessionId: string): void {
    const sessionListeners = listeners.value.get(sessionId);
    if (sessionListeners) {
      sessionListeners.forEach((unlisten) => unlisten());
      listeners.value.delete(sessionId);
    }
    sessions.value.delete(sessionId);
  }

  /**
   * Clean up all sessions on unmount
   */
  onUnmounted(() => {
    listeners.value.forEach((sessionListeners) => {
      sessionListeners.forEach((unlisten) => unlisten());
    });
    listeners.value.clear();
  });

  return {
    sessions,
    createSession,
    startReading,
    reattachReading,
    write,
    resize,
    kill,
    cleanup,
    getCwd,
  };
}
