import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mockStore } from '../../tests/mocks/tauri';

// 必須在 import app 之前設定 mock
import '../../tests/mocks/tauri';

// Mock document.documentElement for applyTheme
Object.defineProperty(document.documentElement, 'setAttribute', {
  value: vi.fn(),
  writable: true,
});

const { app } = await import('./app.svelte');

describe('AppState', () => {
  beforeEach(() => {
    vi.clearAllMocks();
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
        },
      }];
      app.activeTabId = 'test-1';
      expect(app.repoName).toBe('my-repo');
    });
  });

  describe('tab management', () => {
    it('closeTab 移除 tab', () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null } },
        { id: 'b', path: '/b', name: 'b', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'dev', selectedFile: null } },
      ];
      app.activeTabId = 'a';

      app.closeTab('a');
      expect(app.tabs.length).toBe(1);
      expect(app.tabs[0].id).toBe('b');
    });

    it('closeTab 最後一個 tab 回到 Welcome', () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null } },
      ];
      app.activeTabId = 'a';

      app.closeTab('a');
      expect(app.tabs.length).toBe(0);
      expect(app.activeTabId).toBe(null);
      expect(app.hasRepo).toBe(false);
    });

    it('closeOtherTabs 只保留指定 tab', () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null } },
        { id: 'b', path: '/b', name: 'b', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'dev', selectedFile: null } },
        { id: 'c', path: '/c', name: 'c', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'feat', selectedFile: null } },
      ];
      app.activeTabId = 'a';

      app.closeOtherTabs('b');
      expect(app.tabs.length).toBe(1);
      expect(app.tabs[0].id).toBe('b');
    });

    it('closeAllTabs 清空所有 tabs', () => {
      app.tabs = [
        { id: 'a', path: '/a', name: 'a', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: 'main', selectedFile: null } },
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
        },
      }];
      expect(app.dirtyCount('a')).toBe(3);
    });

    it('displayName 處理同名 tabs', () => {
      app.tabs = [
        { id: 'a', path: '/projects/alpha/main', name: 'main', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: '', selectedFile: null } },
        { id: 'b', path: '/projects/beta/main', name: 'main', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: '', selectedFile: null } },
      ];

      expect(app.displayName(app.tabs[0])).toBe('alpha/main');
      expect(app.displayName(app.tabs[1])).toBe('beta/main');
    });

    it('displayName 無重複時只顯示名稱', () => {
      app.tabs = [
        { id: 'a', path: '/projects/alpha', name: 'alpha', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: '', selectedFile: null } },
        { id: 'b', path: '/projects/beta', name: 'beta', state: { stagedFiles: [], unstagedFiles: [], commits: [], branches: [], currentBranch: '', selectedFile: null } },
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
    it('loadRecentRepos 從 store 載入', async () => {
      const repos = ['/repo/a', '/repo/b'];
      mockStore.get.mockResolvedValueOnce(repos);
      mockStore.get.mockResolvedValueOnce(null); // pinnedRepos

      await app.loadRecentRepos();
      expect(mockStore.get).toHaveBeenCalledWith('recentRepos');
      expect(app.recentRepos).toEqual(repos);
    });

    it('loadRecentRepos 在 store 為空時保持空陣列', async () => {
      mockStore.get.mockResolvedValueOnce(null);
      mockStore.get.mockResolvedValueOnce(null);
      app.recentRepos = [];

      await app.loadRecentRepos();
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
});
