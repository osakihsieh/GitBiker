import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mockGitCommands, mockUnlisten } from '../../tests/mocks/tauri';
import '../../tests/mocks/tauri';

import {
  refreshStatus,
  refreshAll,
  loadDiff,
  loadRepoData,
  setupWatcher,
  teardownWatcher,
  _resetWatcherForTest,
  type GitActionableState,
} from './git-actions.svelte';

function createMockState(overrides?: Partial<GitActionableState>): GitActionableState {
  return {
    activeTab: {
      path: '/test/repo',
      state: {
        stagedFiles: [],
        unstagedFiles: [],
        commits: [],
        branches: [],
        currentBranch: 'main',
      },
    },
    tabs: [],
    currentDiff: null,
    repoPath: '/test/repo',
    loading: false,
    addToast: vi.fn(),
    ...overrides,
  };
}

describe('git-actions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    _resetWatcherForTest();
  });

  describe('refreshStatus', () => {
    it('呼叫 gitStatus 並更新 staged/unstaged', async () => {
      const state = createMockState();
      mockGitCommands.gitStatus.mockResolvedValueOnce([
        { path: 'a.ts', kind: 'Modified', staging: 'Staged' },
        { path: 'b.ts', kind: 'Modified', staging: 'Unstaged' },
      ]);

      await refreshStatus(state);
      expect(state.activeTab!.state.stagedFiles).toHaveLength(1);
      expect(state.activeTab!.state.unstagedFiles).toHaveLength(1);
    });

    it('無 activeTab 時不執行', async () => {
      const state = createMockState({ activeTab: null });
      await refreshStatus(state);
      expect(mockGitCommands.gitStatus).not.toHaveBeenCalled();
    });

    it('拋錯時觸發 toast', async () => {
      const state = createMockState();
      mockGitCommands.gitStatus.mockRejectedValueOnce(new Error('git error'));

      await refreshStatus(state);
      expect(state.addToast).toHaveBeenCalledWith('Error: git error', 'error');
    });
  });

  describe('refreshAll', () => {
    it('呼叫 gitStatus + gitLog + gitBranches', async () => {
      const state = createMockState();
      mockGitCommands.gitStatus.mockResolvedValueOnce([]);
      mockGitCommands.gitLog.mockResolvedValueOnce([{ id: '123', message: 'test', author: 'A', email: '', timestamp: 0, parents: [] }]);
      mockGitCommands.gitBranches.mockResolvedValueOnce([{ name: 'main', is_current: true, is_remote: false, upstream: null, commit_id: '123' }]);

      await refreshAll(state);
      expect(mockGitCommands.gitStatus).toHaveBeenCalled();
      expect(mockGitCommands.gitLog).toHaveBeenCalled();
      expect(mockGitCommands.gitBranches).toHaveBeenCalled();
      expect(state.activeTab!.state.commits).toHaveLength(1);
      expect(state.activeTab!.state.currentBranch).toBe('main');
    });

    it('無 activeTab 時不執行', async () => {
      const state = createMockState({ activeTab: null });
      await refreshAll(state);
      expect(mockGitCommands.gitStatus).not.toHaveBeenCalled();
    });
  });

  describe('loadDiff', () => {
    it('呼叫 gitDiff 並設定 currentDiff', async () => {
      const state = createMockState();
      const fakeDiff = { file_path: 'a.ts', hunks: [], stats: { additions: 1, deletions: 0 }, is_binary: false, is_truncated: false };
      mockGitCommands.gitDiff.mockResolvedValueOnce(fakeDiff);

      await loadDiff(state, 'a.ts');
      expect(state.currentDiff).toEqual(fakeDiff);
    });

    it('無 activeTab 時不執行', async () => {
      const state = createMockState({ activeTab: null });
      await loadDiff(state, 'a.ts');
      expect(mockGitCommands.gitDiff).not.toHaveBeenCalled();
    });
  });

  describe('loadRepoData', () => {
    it('正確分類 staged/unstaged 並填入 tab', async () => {
      const state = createMockState();
      state.tabs = [{ id: 'tab-1', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: '' } }];
      mockGitCommands.gitStatus.mockResolvedValueOnce([
        { path: 'staged.ts', kind: 'Added', staging: 'Staged' },
        { path: 'unstaged.ts', kind: 'Modified', staging: 'Unstaged' },
      ]);
      mockGitCommands.gitLog.mockResolvedValueOnce([]);
      mockGitCommands.gitBranches.mockResolvedValueOnce([{ name: 'dev', is_current: true, is_remote: false, upstream: null, commit_id: null }]);

      await loadRepoData(state, 'tab-1', '/test/repo');
      expect(state.tabs[0].state.stagedFiles).toHaveLength(1);
      expect(state.tabs[0].state.unstagedFiles).toHaveLength(1);
      expect(state.tabs[0].state.currentBranch).toBe('dev');
    });
  });

  describe('watcher', () => {
    it('setupWatcher 呼叫 startWatching + listen', async () => {
      const state = createMockState();
      await setupWatcher(state, '/test/repo');
      expect(mockGitCommands.startWatching).toHaveBeenCalledWith('/test/repo');
    });

    it('teardownWatcher 呼叫 unlisten + stopWatching', async () => {
      const state = createMockState();
      // First setup to establish the listener
      await setupWatcher(state, '/test/repo');

      teardownWatcher();
      expect(mockUnlisten).toHaveBeenCalled();
      expect(mockGitCommands.stopWatching).toHaveBeenCalled();
    });
  });
});
