import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { GitStatus, FileDiff, CommitInfo } from '../types';

export const useGitStore = defineStore('git', () => {
  // === State ===
  const repoId = ref<string | null>(null);
  const repoPath = ref<string | null>(null);
  const status = ref<GitStatus | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Sidebar state
  const sidebarVisible = ref(false);
  const activeTab = ref<'changes' | 'history'>('changes');

  // Diff viewer state
  const currentDiff = ref<FileDiff | null>(null);
  const diffMode = ref<'split' | 'inline'>('split');
  const diffVisible = ref(false);
  const diffPath = ref<string | null>(null);
  const diffStaged = ref(false);
  const diffLoading = ref(false);

  // History state
  const commits = ref<CommitInfo[]>([]);
  const historyLoading = ref(false);
  const selectedCommit = ref<CommitInfo | null>(null);
  const historyHasMore = ref(true);

  // Commit state
  const commitMessage = ref('');
  const isCommitting = ref(false);

  // Remote operations state
  const isFetching = ref(false);
  const isPulling = ref(false);
  const isPushing = ref(false);
  const remoteMessage = ref<string | null>(null);

  // Watcher state
  let watcherUnlisten: UnlistenFn | null = null;

  // === Computed ===
  const hasRepo = computed(() => repoId.value !== null);
  const branchName = computed(() => status.value?.branch ?? null);
  const ahead = computed(() => status.value?.ahead ?? 0);
  const behind = computed(() => status.value?.behind ?? 0);
  const stagedFiles = computed(() => status.value?.staged ?? []);
  const unstagedFiles = computed(() => status.value?.unstaged ?? []);
  const untrackedFiles = computed(() => status.value?.untracked ?? []);
  const conflictedFiles = computed(() => status.value?.conflicted ?? []);

  const hasChanges = computed(() =>
    stagedFiles.value.length > 0 ||
    unstagedFiles.value.length > 0 ||
    untrackedFiles.value.length > 0
  );

  const canCommit = computed(() =>
    stagedFiles.value.length > 0 &&
    commitMessage.value.trim().length > 0 &&
    !isCommitting.value
  );

  const changeCount = computed(() =>
    stagedFiles.value.length +
    unstagedFiles.value.length +
    untrackedFiles.value.length
  );

  const isRemoteOperating = computed(() =>
    isFetching.value || isPulling.value || isPushing.value
  );

  const canPush = computed(() =>
    hasRepo.value && ahead.value > 0 && !isRemoteOperating.value
  );

  const canPull = computed(() =>
    hasRepo.value && behind.value > 0 && !isRemoteOperating.value
  );

  // === Actions ===

  // Repository management
  async function openRepository(path: string) {
    console.log('[GitStore] openRepository called with path:', path);
    isLoading.value = true;
    error.value = null;

    try {
      // Discover .git directory from path
      console.log('[GitStore] Calling git_discover_repo...');
      const discoveredPath = await invoke<string>('git_discover_repo', { path });
      console.log('[GitStore] Discovered repo path:', discoveredPath);

      repoId.value = await invoke<string>('git_open_repo', { path: discoveredPath });
      console.log('[GitStore] Opened repo with ID:', repoId.value);
      repoPath.value = discoveredPath;

      // Load initial status
      await refreshStatus();
      console.log('[GitStore] Status loaded, branch:', status.value?.branch);

      // Start file watcher
      await startWatcher();
      console.log('[GitStore] Watcher started');
    } catch (e) {
      console.error('[GitStore] Failed to open repository:', e);
      error.value = `Failed to open repository: ${e}`;
      repoId.value = null;
      repoPath.value = null;
      throw e; // Re-throw so caller knows it failed
    } finally {
      isLoading.value = false;
    }
  }

  async function closeRepository() {
    if (repoId.value) {
      // Stop watcher
      await stopWatcher();

      try {
        await invoke('git_close_repo', { repoId: repoId.value });
      } catch (e) {
        console.warn('Failed to close repo:', e);
      }
      repoId.value = null;
      repoPath.value = null;
      status.value = null;
    }
  }

  // Watcher management
  async function startWatcher() {
    if (!repoId.value || !repoPath.value) return;

    // Stop any existing watcher
    await stopWatcher();

    try {
      // Start backend watcher
      await invoke('git_start_watcher', {
        repoId: repoId.value,
        repoPath: repoPath.value,
      });

      // Listen for change events
      watcherUnlisten = await listen(`git-files-changed-${repoId.value}`, () => {
        // Refresh status when files change
        refreshStatus();
      });
    } catch (e) {
      console.warn('Failed to start watcher:', e);
    }
  }

  async function stopWatcher() {
    if (watcherUnlisten) {
      watcherUnlisten();
      watcherUnlisten = null;
    }

    if (repoId.value) {
      try {
        await invoke('git_stop_watcher', { repoId: repoId.value });
      } catch (e) {
        console.warn('Failed to stop watcher:', e);
      }
    }
  }

  // Status
  async function refreshStatus() {
    if (!repoId.value) return;

    try {
      status.value = await invoke<GitStatus>('git_status', { repoId: repoId.value });
    } catch (e) {
      error.value = `Failed to get status: ${e}`;
    }
  }

  // Staging
  async function stageFile(path: string) {
    if (!repoId.value) return;

    try {
      await invoke('git_stage_file', { repoId: repoId.value, path });
      await refreshStatus();
    } catch (e) {
      error.value = `Failed to stage file: ${e}`;
    }
  }

  async function unstageFile(path: string) {
    if (!repoId.value) return;

    try {
      await invoke('git_unstage_file', { repoId: repoId.value, path });
      await refreshStatus();
    } catch (e) {
      error.value = `Failed to unstage file: ${e}`;
    }
  }

  async function stageAll() {
    if (!repoId.value) return;

    try {
      await invoke('git_stage_all', { repoId: repoId.value });
      await refreshStatus();
    } catch (e) {
      error.value = `Failed to stage all: ${e}`;
    }
  }

  // Commit
  async function commit() {
    if (!repoId.value || !canCommit.value) return;

    isCommitting.value = true;
    error.value = null;

    try {
      await invoke('git_commit', {
        repoId: repoId.value,
        message: commitMessage.value.trim()
      });
      commitMessage.value = '';
      await refreshStatus();
    } catch (e) {
      error.value = `Failed to commit: ${e}`;
    } finally {
      isCommitting.value = false;
    }
  }

  // Sidebar
  function toggleSidebar() {
    sidebarVisible.value = !sidebarVisible.value;
  }

  function showSidebar() {
    sidebarVisible.value = true;
  }

  function hideSidebar() {
    sidebarVisible.value = false;
  }

  function setActiveTab(tab: 'changes' | 'history') {
    activeTab.value = tab;
  }

  // Diff viewer
  async function viewFileDiff(path: string, staged: boolean = false) {
    if (!repoId.value) return;

    diffPath.value = path;
    diffStaged.value = staged;
    diffVisible.value = true;
    diffLoading.value = true;

    try {
      currentDiff.value = await invoke<FileDiff>('git_diff_file', {
        repoId: repoId.value,
        path,
        staged,
      });
    } catch (e) {
      error.value = `Failed to load diff: ${e}`;
      currentDiff.value = null;
    } finally {
      diffLoading.value = false;
    }
  }

  function closeDiff() {
    diffVisible.value = false;
    diffPath.value = null;
    currentDiff.value = null;
  }

  function toggleDiffMode() {
    diffMode.value = diffMode.value === 'split' ? 'inline' : 'split';
  }

  // Remote operations
  async function fetch() {
    if (!repoId.value || isFetching.value) return;

    isFetching.value = true;
    error.value = null;
    remoteMessage.value = null;

    try {
      await invoke('git_fetch', { repoId: repoId.value });
      remoteMessage.value = 'Fetch complete';
      await refreshStatus();
    } catch (e) {
      error.value = `Fetch failed: ${e}`;
    } finally {
      isFetching.value = false;
    }
  }

  async function pull() {
    if (!repoId.value || isPulling.value) return;

    isPulling.value = true;
    error.value = null;
    remoteMessage.value = null;

    try {
      const message = await invoke<string>('git_pull', { repoId: repoId.value });
      remoteMessage.value = message;
      await refreshStatus();
    } catch (e) {
      error.value = `Pull failed: ${e}`;
    } finally {
      isPulling.value = false;
    }
  }

  async function push() {
    if (!repoId.value || isPushing.value) return;

    isPushing.value = true;
    error.value = null;
    remoteMessage.value = null;

    try {
      await invoke('git_push', { repoId: repoId.value });
      remoteMessage.value = 'Push complete';
      await refreshStatus();
    } catch (e) {
      error.value = `Push failed: ${e}`;
    } finally {
      isPushing.value = false;
    }
  }

  async function sync() {
    // Pull first, then push
    await pull();
    if (!error.value) {
      await push();
    }
  }

  function clearRemoteMessage() {
    remoteMessage.value = null;
  }

  // History
  async function loadHistory(reset: boolean = false) {
    if (!repoId.value) return;

    if (reset) {
      commits.value = [];
      historyHasMore.value = true;
    }

    if (!historyHasMore.value) return;

    historyLoading.value = true;

    try {
      const skip = commits.value.length;
      const limit = 50;
      const newCommits = await invoke<CommitInfo[]>('git_log', {
        repoId: repoId.value,
        limit,
        skip,
      });

      if (newCommits.length < limit) {
        historyHasMore.value = false;
      }

      commits.value = [...commits.value, ...newCommits];
    } catch (e) {
      error.value = `Failed to load history: ${e}`;
    } finally {
      historyLoading.value = false;
    }
  }

  async function selectCommit(oid: string) {
    if (!repoId.value) return;

    try {
      selectedCommit.value = await invoke<CommitInfo>('git_show_commit', {
        repoId: repoId.value,
        commitId: oid,
      });
    } catch (e) {
      error.value = `Failed to load commit: ${e}`;
    }
  }

  function clearSelectedCommit() {
    selectedCommit.value = null;
  }

  // Clear error
  function clearError() {
    error.value = null;
  }

  return {
    // State
    repoId,
    repoPath,
    status,
    isLoading,
    error,
    sidebarVisible,
    activeTab,
    currentDiff,
    diffMode,
    diffVisible,
    diffPath,
    diffStaged,
    diffLoading,
    commits,
    historyLoading,
    selectedCommit,
    historyHasMore,
    commitMessage,
    isCommitting,
    isFetching,
    isPulling,
    isPushing,
    remoteMessage,

    // Computed
    hasRepo,
    branchName,
    ahead,
    behind,
    stagedFiles,
    unstagedFiles,
    untrackedFiles,
    conflictedFiles,
    hasChanges,
    canCommit,
    changeCount,
    isRemoteOperating,
    canPush,
    canPull,

    // Actions
    openRepository,
    closeRepository,
    refreshStatus,
    stageFile,
    unstageFile,
    stageAll,
    commit,
    toggleSidebar,
    showSidebar,
    hideSidebar,
    setActiveTab,
    viewFileDiff,
    closeDiff,
    toggleDiffMode,
    fetch,
    pull,
    push,
    sync,
    clearRemoteMessage,
    loadHistory,
    selectCommit,
    clearSelectedCommit,
    clearError,
  };
});
