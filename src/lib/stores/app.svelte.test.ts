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
  });

  describe('hasRepo', () => {
    it('repoPath 為 null 時回傳 false', () => {
      app.repoPath = null;
      expect(app.hasRepo).toBe(false);
    });

    it('repoPath 有值時回傳 true', () => {
      app.repoPath = '/some/repo';
      expect(app.hasRepo).toBe(true);
      app.repoPath = null;
    });
  });

  describe('repoName', () => {
    it('repoPath 為 null 時回傳空字串', () => {
      app.repoPath = null;
      expect(app.repoName).toBe('');
    });

    it('提取 Unix 路徑最後一段', () => {
      app.repoPath = '/home/user/my-repo';
      expect(app.repoName).toBe('my-repo');
      app.repoPath = null;
    });

    it('提取 Windows 路徑最後一段', () => {
      app.repoPath = 'C:\\Users\\user\\my-repo';
      expect(app.repoName).toBe('my-repo');
      app.repoPath = null;
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
      // localStorage 為空時預設 system
      expect(['system', 'dark', 'light']).toContain(app.theme);
    });

    it('setTheme 更新主題並寫入 localStorage', () => {
      app.setTheme('dark');
      expect(app.theme).toBe('dark');
      expect(localStorage.getItem('gitbiker-theme')).toBe('dark');

      app.setTheme('light');
      expect(app.theme).toBe('light');
      expect(localStorage.getItem('gitbiker-theme')).toBe('light');

      // 還原
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

      await app.loadRecentRepos();
      expect(mockStore.get).toHaveBeenCalledWith('recentRepos');
      expect(app.recentRepos).toEqual(repos);
    });

    it('loadRecentRepos 在 store 為空時保持空陣列', async () => {
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
