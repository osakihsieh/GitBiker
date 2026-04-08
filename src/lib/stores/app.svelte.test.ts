import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mockStore, mockGitCommands } from '../../tests/mocks/tauri';

// 必須在 import app 之前設定 mock
import '../../tests/mocks/tauri';

// Mock document.documentElement for applyTheme
Object.defineProperty(document.documentElement, 'setAttribute', {
  value: vi.fn(),
  writable: true,
});

const { app } = await import('./app.svelte');

// Mock crypto.randomUUID
let uuidCounter = 0;
vi.stubGlobal('crypto', {
  randomUUID: () => `uuid-${++uuidCounter}`,
});

describe('AppState', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    uuidCounter = 0;
    // Reset tabs state
    app.closeAllTabs();
  });

  describe('hasRepo', () => {
    it('無 tabs 時回傳 false', () => {
      expect(app.hasRepo).toBe(false);
    });

    it('有 active tab 時回傳 true', () => {
      // Directly add a tab for testing
      app.tabs = [{
        id: 'test-1',
        path: '/some/repo',
        name: 'repo',
        state: {
          stagedFiles: [],
          unstagedFiles: [],
          commits: [],
          branches: [],
          currentBranch: 'main',
          selectedFile: null,
          viewMode: 'worktree' as const,
          conflictFiles: [],
          activeConflictFile: null,
          conflictContent: null,
          hunkChoices: {},
          fileHistoryTarget: null,
        },
      }];
      app.activeTabId = 'test-1';
      expect(app.hasRepo).toBe(true);
    });
  });

  describe('repoName', () => {
    it('無 active tab 時回傳空字串', () => {
      expect(app.repoName).toBe('');
    });

    it('回傳 active tab 的 name', () => {
      app.tabs = [{
        id: 'test-1',
        path: '/home/user/my-repo',
        name: 'my-repo',
        state: {
          stagedFiles: [],
          unstagedFiles: [],
          commits: [],
          branches: [],
          currentBranch: 'main',
          selectedFile: null,
          viewMode: 'worktree' as const,
          conflictFiles: [],
          activeConflictFile: null,
          conflictContent: null,
          hunkChoices: {},
          fileHistoryTarget: null,
        },
      }];
      app.activeTabId = 'test-1';
      expect(app.repoName).toBe('my-repo');
    });
  });

  describe('tab management', () => {
    it('closeTab 移除 tab', () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
        { id: 'b', path: '/b', name: 'b', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'dev', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];
      app.activeTabId = 'a';

      app.closeTab('a');
      expect(app.tabs.length).toBe(1);
      expect(app.tabs[0].id).toBe('b');
    });

    it('closeTab 最後一個 tab 回到 Welcome', () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];
      app.activeTabId = 'a';

      app.closeTab('a');
      expect(app.tabs.length).toBe(0);
      expect(app.activeTabId).toBe(null);
      expect(app.hasRepo).toBe(false);
    });

    it('closeOtherTabs 只保留指定 tab', () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
        { id: 'b', path: '/b', name: 'b', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'dev', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
        { id: 'c', path: '/c', name: 'c', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'feat', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];
      app.activeTabId = 'a';

      app.closeOtherTabs('b');
      expect(app.tabs.length).toBe(1);
      expect(app.tabs[0].id).toBe('b');
    });

    it('closeAllTabs 清空所有 tabs', () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];
      app.activeTabId = 'a';

      app.closeAllTabs();
      expect(app.tabs.length).toBe(0);
      expect(app.activeTabId).toBe(null);
    });

    it('dirtyCount 計算 staged + unstaged', () => {
      app.tabs = [{
        id: 'a', path: '/a', name: 'a',
        state: {
          stagedFiles: [{ path: 'f1', kind: 'Modified', staging: 'Staged' }],
          unstagedFiles: [{ path: 'f2', kind: 'Modified', staging: 'Unstaged' }, { path: 'f3', kind: 'Added', staging: 'Unstaged' }],
          commits: [], branches: [], currentBranch: 'main', selectedFile: null,
          viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {},
          fileHistoryTarget: null,
        },
      }];
      expect(app.dirtyCount('a')).toBe(3);
    });

    it('displayName 處理同名 tabs', () => {
      app.tabs = [
        { id: 'a', path: '/projects/alpha/main', name: 'main', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: '', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
        { id: 'b', path: '/projects/beta/main', name: 'main', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: '', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];

      expect(app.displayName(app.tabs[0])).toBe('alpha/main');
      expect(app.displayName(app.tabs[1])).toBe('beta/main');
    });

    it('displayName 無重複時只顯示名稱', () => {
      app.tabs = [
        { id: 'a', path: '/projects/alpha', name: 'alpha', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: '', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
        { id: 'b', path: '/projects/beta', name: 'beta', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: '', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];

      expect(app.displayName(app.tabs[0])).toBe('alpha');
    });
  });

  describe('pin management', () => {
    it('pinRepo 新增到 pinnedRepos', async () => {
      app.pinnedRepos = [];
      mockStore.set.mockResolvedValueOnce(undefined);

      await app.pinRepo('/repo/a');
      expect(app.pinnedRepos).toContain('/repo/a');
    });

    it('unpinRepo 從 pinnedRepos 移除', async () => {
      app.pinnedRepos = ['/repo/a', '/repo/b'];
      mockStore.set.mockResolvedValueOnce(undefined);

      await app.unpinRepo('/repo/a');
      expect(app.pinnedRepos).not.toContain('/repo/a');
    });

    it('togglePin 切換 pin 狀態', async () => {
      app.pinnedRepos = [];
      mockStore.set.mockResolvedValue(undefined);

      await app.togglePin('/repo/a');
      expect(app.isPinned('/repo/a')).toBe(true);

      await app.togglePin('/repo/a');
      expect(app.isPinned('/repo/a')).toBe(false);
    });
  });

  describe('toast', () => {
    it('addToast 新增一筆 toast', () => {
      const before = app.toasts.length;
      app.addToast('test message', 'info', false);
      expect(app.toasts.length).toBe(before + 1);
      const last = app.toasts[app.toasts.length - 1];
      expect(last.message).toBe('test message');
      expect(last.type).toBe('info');
      expect(last.autoDismiss).toBe(false);
    });

    it('removeToast 移除指定 id', () => {
      app.addToast('to remove', 'success', false);
      const toast = app.toasts[app.toasts.length - 1];
      app.removeToast(toast.id);
      expect(app.toasts.find((t) => t.id === toast.id)).toBeUndefined();
    });
  });

  describe('theme', () => {
    it('預設主題為 system', () => {
      expect(['system', 'dark', 'light']).toContain(app.theme);
    });

    it('setTheme 更新主題並寫入 localStorage', () => {
      app.setTheme('dark');
      expect(app.theme).toBe('dark');
      expect(localStorage.getItem('gitbiker-theme')).toBe('dark');

      app.setTheme('light');
      expect(app.theme).toBe('light');
      expect(localStorage.getItem('gitbiker-theme')).toBe('light');

      app.setTheme('system');
    });

    it('resolvedTheme 在 system 模式下根據系統偏好決定', () => {
      app.setTheme('system');
      app.systemPrefersDark = true;
      expect(app.resolvedTheme).toBe('dark');
      app.systemPrefersDark = false;
      expect(app.resolvedTheme).toBe('light');
    });

    it('resolvedTheme 在非 system 模式下直接回傳設定值', () => {
      app.setTheme('dark');
      expect(app.resolvedTheme).toBe('dark');
      app.setTheme('light');
      expect(app.resolvedTheme).toBe('light');
      app.setTheme('system');
    });
  });

  describe('recentRepos 持久化', () => {
    it('loadAppSettings 從 store 載入', async () => {
      const repos = ['/repo/a', '/repo/b'];
      mockStore.get.mockResolvedValueOnce(repos);
      mockStore.get.mockResolvedValueOnce(null); // pinnedRepos

      await app.loadAppSettings();
      expect(mockStore.get).toHaveBeenCalledWith('recentRepos');
      expect(app.recentRepos).toEqual(repos);
    });

    it('loadAppSettings 在 store 為空時保持空陣列', async () => {
      mockStore.get.mockResolvedValueOnce(null);
      mockStore.get.mockResolvedValueOnce(null);
      app.recentRepos = [];

      await app.loadAppSettings();
      expect(app.recentRepos).toEqual([]);
    });

    it('addRecentRepo 新增路徑並寫入 store', async () => {
      app.recentRepos = [];
      mockStore.set.mockResolvedValueOnce(undefined);

      await app.addRecentRepo('/new/repo');
      expect(app.recentRepos[0]).toBe('/new/repo');
      expect(mockStore.set).toHaveBeenCalledWith('recentRepos', app.recentRepos);
    });

    it('addRecentRepo 重複路徑移到最前面', async () => {
      app.recentRepos = ['/repo/a', '/repo/b', '/repo/c'];
      mockStore.set.mockResolvedValueOnce(undefined);

      await app.addRecentRepo('/repo/c');
      expect(app.recentRepos[0]).toBe('/repo/c');
      expect(app.recentRepos).toEqual(['/repo/c', '/repo/a', '/repo/b']);
    });

    it('addRecentRepo 最多保留 10 筆', async () => {
      app.recentRepos = Array.from({ length: 10 }, (_, i) => `/repo/${i}`);
      mockStore.set.mockResolvedValueOnce(undefined);

      await app.addRecentRepo('/repo/new');
      expect(app.recentRepos.length).toBe(10);
      expect(app.recentRepos[0]).toBe('/repo/new');
    });
  });

  describe('computed property setters', () => {
    const makeTab = () => ({
      id: 'tab-1',
      path: '/test/repo',
      name: 'repo',
      state: {
        stagedFiles: [] as any[],
        unstagedFiles: [] as any[],
        commits: [] as any[],
        branches: [] as any[],
        currentBranch: 'main',
        selectedFile: null as string | null,
        viewMode: 'worktree' as const,
        conflictFiles: [],
        activeConflictFile: null,
        conflictContent: null,
        hunkChoices: {},
        fileHistoryTarget: null,
      },
    });

    it('set currentBranch 寫入 activeTab', () => {
      app.tabs = [makeTab()];
      app.activeTabId = 'tab-1';
      app.currentBranch = 'develop';
      expect(app.activeTab!.state.currentBranch).toBe('develop');
    });

    it('set selectedFile 寫入 activeTab', () => {
      app.tabs = [makeTab()];
      app.activeTabId = 'tab-1';
      app.selectedFile = 'readme.md';
      expect(app.activeTab!.state.selectedFile).toBe('readme.md');
    });

    it('set stagedFiles 寫入 activeTab', () => {
      app.tabs = [makeTab()];
      app.activeTabId = 'tab-1';
      const files = [{ path: 'a.ts', kind: 'Modified', staging: 'Staged' }];
      app.stagedFiles = files as any;
      expect(app.activeTab!.state.stagedFiles).toEqual(files);
    });

    it('set unstagedFiles 寫入 activeTab', () => {
      app.tabs = [makeTab()];
      app.activeTabId = 'tab-1';
      const files = [{ path: 'b.ts', kind: 'Added', staging: 'Unstaged' }];
      app.unstagedFiles = files as any;
      expect(app.activeTab!.state.unstagedFiles).toEqual(files);
    });

    it('set commits 寫入 activeTab', () => {
      app.tabs = [makeTab()];
      app.activeTabId = 'tab-1';
      const commits = [{ id: 'abc', message: 'test', author: 'A', email: '', timestamp: 0, parents: [] }];
      app.commits = commits as any;
      expect(app.activeTab!.state.commits).toEqual(commits);
    });

    it('set branches 寫入 activeTab', () => {
      app.tabs = [makeTab()];
      app.activeTabId = 'tab-1';
      const branches = [{ name: 'main', is_current: true, is_remote: false, upstream: null, commit_id: '123' }];
      app.branches = branches as any;
      expect(app.activeTab!.state.branches).toEqual(branches);
    });

    it('無 activeTab 時 setter 不拋錯', () => {
      app.tabs = [];
      app.activeTabId = null;
      expect(() => { app.currentBranch = 'x'; }).not.toThrow();
      expect(() => { app.selectedFile = 'x'; }).not.toThrow();
      expect(() => { app.stagedFiles = []; }).not.toThrow();
      expect(() => { app.unstagedFiles = []; }).not.toThrow();
      expect(() => { app.commits = []; }).not.toThrow();
      expect(() => { app.branches = []; }).not.toThrow();
    });
  });

  describe('computed getters', () => {
    it('repoPath 回傳 activeTab 的 path', () => {
      app.tabs = [{
        id: 'a', path: '/my/repo', name: 'repo',
        state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null },
      }];
      app.activeTabId = 'a';
      expect(app.repoPath).toBe('/my/repo');
    });

    it('repoPath 無 activeTab 時回傳 null', () => {
      expect(app.repoPath).toBeNull();
    });

    it('stagedFiles/unstagedFiles/commits/branches/currentBranch/selectedFile 無 activeTab 時回傳預設值', () => {
      app.tabs = [];
      app.activeTabId = null;
      expect(app.stagedFiles).toEqual([]);
      expect(app.unstagedFiles).toEqual([]);
      expect(app.commits).toEqual([]);
      expect(app.branches).toEqual([]);
      expect(app.currentBranch).toBe('');
      expect(app.selectedFile).toBeNull();
    });
  });

  describe('tabBranch', () => {
    it('回傳指定 tab 的 currentBranch', () => {
      app.tabs = [{
        id: 'a', path: '/a', name: 'a',
        state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'feature-x', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null },
      }];
      expect(app.tabBranch('a')).toBe('feature-x');
    });

    it('找不到 tab 時回傳空字串', () => {
      expect(app.tabBranch('nonexistent')).toBe('');
    });
  });

  describe('dirtyCount edge cases', () => {
    it('找不到 tab 時回傳 0', () => {
      expect(app.dirtyCount('nonexistent')).toBe(0);
    });
  });

  describe('viewMode', () => {
    beforeEach(() => {
      // viewMode is now per-tab, need an active tab
      app.tabs = [{ id: 'vm-tab', path: '/vm', name: 'vm', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } }];
      app.activeTabId = 'vm-tab';
    });

    it('預設為 worktree', () => {
      expect(app.viewMode).toBe('worktree');
    });

    it('selectCommit 切換到 commit-detail', () => {
      const commit = { id: 'abc123', message: 'test', author: 'A', email: '', timestamp: 0, parents: [], refs: [] };
      app.selectCommit(commit);
      expect(app.viewMode).toBe('commit-detail');
      expect(app.selectedCommit).toEqual(commit);
    });

    it('backToWorktree 回到 worktree', () => {
      app.selectCommit({ id: 'abc', message: '', author: '', email: '', timestamp: 0, parents: [], refs: [] });
      app.backToWorktree();
      expect(app.viewMode).toBe('worktree');
      expect(app.selectedCommit).toBeNull();
    });

    it('closeAllTabs 重置 viewMode', () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];
      app.activeTabId = 'a';
      app.selectCommit({ id: 'x', message: '', author: '', email: '', timestamp: 0, parents: [], refs: [] });

      app.closeAllTabs();
      expect(app.viewMode).toBe('worktree');
      expect(app.selectedCommit).toBeNull();
    });
  });

  describe('openRepo', () => {
    it('開啟新 repo 建立 tab 並設為 active', async () => {
      mockGitCommands.gitStatus.mockResolvedValueOnce([]);
      mockGitCommands.gitLog.mockResolvedValueOnce([]);
      mockGitCommands.gitBranches.mockResolvedValueOnce([{ name: 'main', is_current: true, is_remote: false, upstream: null, commit_id: '123' }]);
      mockStore.set.mockResolvedValue(undefined);
      mockStore.get.mockResolvedValue(null);

      await app.openRepo('/new/repo');

      expect(app.tabs).toHaveLength(1);
      expect(app.activeTabId).toBe('uuid-1');
      expect(app.tabs[0].path).toBe('/new/repo');
      expect(app.tabs[0].name).toBe('repo');
    });

    it('已開啟的 repo 切換到該 tab', async () => {
      // Manually add a tab
      app.tabs = [{
        id: 'existing', path: '/existing/repo', name: 'repo',
        state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null },
      }];
      app.activeTabId = 'existing';

      mockGitCommands.gitStatus.mockResolvedValue([]);

      await app.openRepo('/existing/repo');

      // Should not create a new tab
      expect(app.tabs).toHaveLength(1);
      expect(app.activeTabId).toBe('existing');
    });

    it('background 模式不切換 activeTab', async () => {
      app.tabs = [{
        id: 'current', path: '/current', name: 'current',
        state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null },
      }];
      app.activeTabId = 'current';

      await app.openRepo('/background/repo', true);

      expect(app.tabs).toHaveLength(2);
      expect(app.activeTabId).toBe('current'); // didn't switch
    });

    it('載入失敗時移除 tab 並顯示 toast', async () => {
      mockGitCommands.gitStatus.mockRejectedValueOnce(new Error('repo 不存在'));

      await app.openRepo('/bad/repo');

      expect(app.tabs).toHaveLength(0);
      expect(app.toasts.some((t) => t.type === 'error')).toBe(true);
      expect(app.loading).toBe(false);
    });
  });

  describe('switchTab', () => {
    it('切換到目標 tab 並重設 viewMode', async () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
        { id: 'b', path: '/b', name: 'b', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'dev', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];
      app.activeTabId = 'a';
      app.selectCommit({ id: 'x', message: '', author: '', email: '', timestamp: 0, parents: [], refs: [] });

      mockGitCommands.gitStatus.mockResolvedValue([]);

      await app.switchTab('b');

      expect(app.activeTabId).toBe('b');
      expect(app.viewMode).toBe('worktree');
      expect(app.selectedCommit).toBeNull();
      expect(app.currentDiff).toBeNull();
    });

    it('切換到同一 tab 時不執行', async () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];
      app.activeTabId = 'a';

      await app.switchTab('a');
      expect(mockGitCommands.startWatching).not.toHaveBeenCalled();
    });

    it('目標 tab 不存在時不執行', async () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null, viewMode: 'worktree' as const, conflictFiles: [], activeConflictFile: null, conflictContent: null, hunkChoices: {}, fileHistoryTarget: null } },
      ];
      app.activeTabId = 'a';

      await app.switchTab('nonexistent');
      expect(app.activeTabId).toBe('a');
    });
  });

  describe('repoNameFromPath', () => {
    it('從 Unix 路徑提取名稱', async () => {
      const { repoNameFromPath } = await import('./app.svelte');
      expect(repoNameFromPath('/home/user/my-repo')).toBe('my-repo');
    });

    it('從 Windows 路徑提取名稱', async () => {
      const { repoNameFromPath } = await import('./app.svelte');
      expect(repoNameFromPath('C:\\Users\\user\\my-repo')).toBe('my-repo');
    });
  });
});
