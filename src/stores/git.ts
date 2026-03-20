import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { GitStatus, FileDiff, CommitInfo, BranchInfo } from '../types';
import { useLayoutStore } from './layout';

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

  // Branch state
  const branches = ref<BranchInfo[]>([]);
  const branchesLoading = ref(false);
  const branchSelectorVisible = ref(false);
  const isCheckingOut = ref(false);
  const isCreatingBranch = ref(false);

  // Discard state
  const isDiscarding = ref(false);

  // Watcher state
  let watcherUnlisten: UnlistenFn | null = null;

  // CWD watcher state
  let cwdWatcherInterval: ReturnType<typeof setInterval> | null = null;
  let lastCheckedCwd: string | null = null;

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

  const hasUpstream = computed(() => status.value?.upstream !== null);

  const needsPublish = computed(() =>
    hasRepo.value && branchName.value !== null && !hasUpstream.value
  );

  const canPush = computed(() =>
    hasRepo.value && (ahead.value > 0 || needsPublish.value) && !isRemoteOperating.value
  );

  const canPull = computed(() =>
    hasRepo.value && behind.value > 0 && !isRemoteOperating.value
  );

  // === Actions ===

  // Repository management
  async function openRepository(path: string) {
    isLoading.value = true;
    error.value = null;

    try {
      // Discover .git directory from path
      const discoveredPath = await invoke<string>('git_discover_repo', { path });

      repoId.value = await invoke<string>('git_open_repo', { path: discoveredPath });
      repoPath.value = discoveredPath;

      // Load initial status
      await refreshStatus();

      // Start file watcher
      await startWatcher();
    } catch (e) {
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
      } catch {
        // Ignore close errors - repo may already be closed
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
    } catch {
      // Watcher is optional - continue without it
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
      } catch {
        // Ignore stop errors - watcher may not exist
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

  async function push(setUpstream: boolean = false) {
    if (!repoId.value || isPushing.value) return;

    isPushing.value = true;
    error.value = null;
    remoteMessage.value = null;

    try {
      await invoke('git_push', { repoId: repoId.value, setUpstream });
      remoteMessage.value = setUpstream ? 'Branch published' : 'Push complete';
      await refreshStatus();
    } catch (e) {
      error.value = `Push failed: ${e}`;
    } finally {
      isPushing.value = false;
    }
  }

  async function publish() {
    await push(true);
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

  // ============ Branch Operations ============

  async function loadBranches() {
    if (!repoId.value) return;

    branchesLoading.value = true;

    try {
      branches.value = await invoke<BranchInfo[]>('git_list_branches', {
        repoId: repoId.value,
      });
    } catch (e) {
      error.value = `Failed to load branches: ${e}`;
    } finally {
      branchesLoading.value = false;
    }
  }

  async function checkoutBranch(branchName: string) {
    if (!repoId.value || isCheckingOut.value) return;

    isCheckingOut.value = true;
    error.value = null;

    try {
      await invoke('git_checkout_branch', {
        repoId: repoId.value,
        branchName,
      });
      await refreshStatus();
      await loadBranches();
      branchSelectorVisible.value = false;
    } catch (e) {
      error.value = `Failed to checkout branch: ${e}`;
    } finally {
      isCheckingOut.value = false;
    }
  }

  async function createBranch(branchName: string, checkout: boolean = true) {
    if (!repoId.value || isCreatingBranch.value) return;

    isCreatingBranch.value = true;
    error.value = null;

    try {
      await invoke('git_create_branch', {
        repoId: repoId.value,
        branchName,
        checkout,
      });
      await refreshStatus();
      await loadBranches();
      branchSelectorVisible.value = false;
    } catch (e) {
      error.value = `Failed to create branch: ${e}`;
    } finally {
      isCreatingBranch.value = false;
    }
  }

  async function deleteBranch(branchName: string) {
    if (!repoId.value) return;

    error.value = null;

    try {
      await invoke('git_delete_branch', {
        repoId: repoId.value,
        branchName,
      });
      await loadBranches();
    } catch (e) {
      error.value = `Failed to delete branch: ${e}`;
    }
  }

  function showBranchSelector() {
    branchSelectorVisible.value = true;
    loadBranches();
  }

  function hideBranchSelector() {
    branchSelectorVisible.value = false;
  }

  // ============ Discard Operations ============

  async function discardFile(path: string) {
    if (!repoId.value || isDiscarding.value) return;

    isDiscarding.value = true;
    error.value = null;

    try {
      await invoke('git_discard_file', {
        repoId: repoId.value,
        path,
      });
      await refreshStatus();
    } catch (e) {
      error.value = `Failed to discard changes: ${e}`;
    } finally {
      isDiscarding.value = false;
    }
  }

  async function discardAll() {
    if (!repoId.value || isDiscarding.value) return;

    isDiscarding.value = true;
    error.value = null;

    try {
      await invoke('git_discard_all', {
        repoId: repoId.value,
      });
      await refreshStatus();
    } catch (e) {
      error.value = `Failed to discard all changes: ${e}`;
    } finally {
      isDiscarding.value = false;
    }
  }

  async function cleanUntracked(paths?: string[]) {
    if (!repoId.value || isDiscarding.value) return;

    isDiscarding.value = true;
    error.value = null;

    try {
      const removed = await invoke<number>('git_clean_untracked', {
        repoId: repoId.value,
        paths: paths ?? null,
      });
      remoteMessage.value = `Removed ${removed} untracked file${removed !== 1 ? 's' : ''}`;
      await refreshStatus();
    } catch (e) {
      error.value = `Failed to clean untracked files: ${e}`;
    } finally {
      isDiscarding.value = false;
    }
  }

  // CWD watcher - monitors active terminal directory for git repo changes
  async function checkActiveCwd() {
    const layoutStore = useLayoutStore();
    const activePane = layoutStore.activePane;
    if (!activePane) return;

    const sessionId = layoutStore.getSessionId(activePane);
    if (!sessionId) return;

    try {
      const cwd = await invoke<string>('get_terminal_cwd', { sessionId });

      // Skip if same as last check
      if (cwd === lastCheckedCwd) return;
      lastCheckedCwd = cwd;

      // Try to discover git repo from new CWD
      try {
        const discoveredPath = await invoke<string>('git_discover_repo', { path: cwd });

        // If different repo than current, switch to it
        if (discoveredPath !== repoPath.value) {
          await closeRepository();
          await openRepository(cwd);
        }
      } catch {
        // Not a git repo - close current repo if any
        if (hasRepo.value) {
          await closeRepository();
        }
      }
    } catch {
      // Failed to get CWD, ignore
    }
  }

  function startCwdWatcher() {
    if (cwdWatcherInterval) return;

    // Check every 1 second
    cwdWatcherInterval = setInterval(checkActiveCwd, 1000);

    // Also check immediately
    checkActiveCwd();
  }

  function stopCwdWatcher() {
    if (cwdWatcherInterval) {
      clearInterval(cwdWatcherInterval);
      cwdWatcherInterval = null;
    }
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
    branches,
    branchesLoading,
    branchSelectorVisible,
    isCheckingOut,
    isCreatingBranch,
    isDiscarding,

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
    hasUpstream,
    needsPublish,
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
    publish,
    sync,
    clearRemoteMessage,
    loadHistory,
    selectCommit,
    clearSelectedCommit,
    clearError,

    // Branch operations
    loadBranches,
    checkoutBranch,
    createBranch,
    deleteBranch,
    showBranchSelector,
    hideBranchSelector,

    // Discard operations
    discardFile,
    discardAll,
    cleanUntracked,

    // CWD watcher
    startCwdWatcher,
    stopCwdWatcher,
  };
});
