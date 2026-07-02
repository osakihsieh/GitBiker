import { extractErrorMessage } from '$lib/utils/error';
import type {
  FileStatus,
  Commit,
  DiffResult,
  Branch,
  LogFilter,
  TagInfo,
  GitLfsStatus,
  SubmoduleInfo,
  WorktreeInfo,
} from '$lib/git/types';
import {
  gitStatus,
  gitLog,
  gitBranches,
  gitDiff,
  gitTags,
  gitLfsStatus,
  gitGetSubmodules,
  gitGetWorktrees,
  startWatching,
  stopWatching,
} from '$lib/git/commands';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

const MAX_COMMITS_PER_TAB = 200;

// ── Types ──────────────────────────────────────────────

/** AppState 的最小介面，避免循環 import */
export interface GitActionableState {
  activeTab: {
    path: string;
    state: {
      stagedFiles: FileStatus[];
      unstagedFiles: FileStatus[];
      commits: Commit[];
      branches: Branch[];
      tags: TagInfo[];
      currentBranch: string;
      logFilter: LogFilter;
      lfsStatus: GitLfsStatus | null;
      submodules: SubmoduleInfo[];
      worktrees: WorktreeInfo[];
    };
  } | null;
  tabs: Array<{
    id: string;
    state: {
      stagedFiles: FileStatus[];
      unstagedFiles: FileStatus[];
      commits: Commit[];
      branches: Branch[];
      tags: TagInfo[];
      currentBranch: string;
      logFilter: LogFilter;
      lfsStatus: GitLfsStatus | null;
      submodules: SubmoduleInfo[];
      worktrees: WorktreeInfo[];
    };
  }>;
  currentDiff: DiffResult | null;
  repoPath: string | null;
  loading: boolean;
  addToast: (message: string, type: 'success' | 'error' | 'info', autoDismiss?: boolean) => void;
}

// ── Watcher ───────────────────────────────────────────

let unlistenGitChanged: UnlistenFn | null = null;

export async function setupWatcher(state: GitActionableState, path: string): Promise<void> {
  teardownWatcher();

  try {
    await startWatching(path);
    unlistenGitChanged = await listen<string>('git-changed', (event) => {
      // Only process events for the active tab's repo
      if (state.repoPath !== path) return;

      if (event.payload === 'index') {
        refreshStatus(state);
      } else {
        refreshAll(state);
      }
    });
  } catch (e: unknown) {
    console.warn('fs watcher 啟動失敗:', e);
  }
}

export function teardownWatcher(): void {
  if (unlistenGitChanged) {
    unlistenGitChanged();
    unlistenGitChanged = null;
  }
  stopWatching().catch(() => {});
}

// ── Refresh ───────────────────────────────────────────

export async function refreshStatus(state: GitActionableState): Promise<void> {
  const tab = state.activeTab;
  if (!tab) return;
  try {
    const status = await gitStatus(tab.path);
    tab.state.stagedFiles = status.filter((f) => f.staging === 'Staged');
    tab.state.unstagedFiles = status.filter((f) => f.staging === 'Unstaged');
  } catch (e: unknown) {
    state.addToast(extractErrorMessage(e), 'error');
  }
}

export async function refreshAll(state: GitActionableState): Promise<void> {
  const tab = state.activeTab;
  if (!tab) return;
  const t0 = performance.now();
  try {
    const results = await Promise.all([
      gitStatus(tab.path).then(
        (r) => (console.log(`[perf] gitStatus: ${(performance.now() - t0).toFixed(0)}ms`), r),
      ),
      gitLog(tab.path, MAX_COMMITS_PER_TAB, tab.state.logFilter).then(
        (r) => (console.log(`[perf] gitLog: ${(performance.now() - t0).toFixed(0)}ms`), r),
      ),
      gitBranches(tab.path).then(
        (r) => (console.log(`[perf] gitBranches: ${(performance.now() - t0).toFixed(0)}ms`), r),
      ),
      gitTags(tab.path).then(
        (r) => (console.log(`[perf] gitTags: ${(performance.now() - t0).toFixed(0)}ms`), r),
      ),
      gitLfsStatus(tab.path)
        .catch(() => null)
        .then((r) => (console.log(`[perf] gitLfs: ${(performance.now() - t0).toFixed(0)}ms`), r)),
      gitGetSubmodules(tab.path)
        .catch(() => [])
        .then(
          (r) => (console.log(`[perf] submodules: ${(performance.now() - t0).toFixed(0)}ms`), r),
        ),
      gitGetWorktrees(tab.path)
        .catch(() => [])
        .then(
          (r) => (console.log(`[perf] worktrees: ${(performance.now() - t0).toFixed(0)}ms`), r),
        ),
    ]);
    const [status, commits, branches, tags, lfs, submodules, worktrees] = results;
    tab.state.stagedFiles = status.filter((f) => f.staging === 'Staged');
    tab.state.unstagedFiles = status.filter((f) => f.staging === 'Unstaged');
    tab.state.commits = commits;
    tab.state.branches = branches;
    tab.state.tags = tags;
    tab.state.lfsStatus = lfs;
    tab.state.submodules = submodules;
    tab.state.worktrees = worktrees;
    tab.state.currentBranch = branches.find((b) => b.is_current)?.name || tab.state.currentBranch;
    console.log(`[perf] refreshAll total: ${(performance.now() - t0).toFixed(0)}ms`);
  } catch (e: unknown) {
    state.addToast(extractErrorMessage(e), 'error');
  }
}

// ── Diff ──────────────────────────────────────────────

export async function loadDiff(state: GitActionableState, filePath: string): Promise<void> {
  const tab = state.activeTab;
  if (!tab) return;
  try {
    state.currentDiff = await gitDiff(tab.path, filePath);
  } catch (e: unknown) {
    state.addToast(extractErrorMessage(e), 'error');
  }
}

// ── Repo Data Loading ─────────────────────────────────

export async function loadRepoData(
  state: GitActionableState,
  tabId: string,
  path: string,
): Promise<void> {
  const reactiveTab = state.tabs.find((t) => t.id === tabId);
  if (!reactiveTab) return;

  const [status, commits, branches, tags, lfs, submodules] = await Promise.all([
    gitStatus(path),
    gitLog(path, MAX_COMMITS_PER_TAB, reactiveTab.state.logFilter),
    gitBranches(path),
    gitTags(path),
    gitLfsStatus(path).catch(() => null),
    gitGetSubmodules(path).catch(() => []),
  ]);

  reactiveTab.state.stagedFiles = status.filter((f) => f.staging === 'Staged');
  reactiveTab.state.unstagedFiles = status.filter((f) => f.staging === 'Unstaged');
  reactiveTab.state.commits = commits;
  reactiveTab.state.branches = branches;
  reactiveTab.state.tags = tags;
  reactiveTab.state.lfsStatus = lfs;
  reactiveTab.state.submodules = submodules;
  reactiveTab.state.currentBranch = branches.find((b) => b.is_current)?.name || 'main';
}

// ── Test Helpers ──────────────────────────────────────

export function _resetWatcherForTest(): void {
  unlistenGitChanged = null;
}
