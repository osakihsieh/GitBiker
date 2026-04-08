import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mockStore } from '../../tests/mocks/tauri';
import '../../tests/mocks/tauri';

import {
  loadAppSettings,
  addRecentRepo,
  removeRecentRepo,
  isPinned,
  pinRepo,
  unpinRepo,
  togglePin,
  type PersistableState,
  _resetStoreForTest,
} from './persistence.svelte';

function createMockState(): PersistableState {
  return { recentRepos: [], pinnedRepos: [], preferredEditor: null };
}

describe('persistence', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    _resetStoreForTest();
  });

  describe('loadAppSettings', () => {
    it('從 store 載入 recentRepos 和 pinnedRepos', async () => {
      const state = createMockState();
      mockStore.get.mockResolvedValueOnce(['/repo/a', '/repo/b']);
      mockStore.get.mockResolvedValueOnce(['/pinned/a']);

      await loadAppSettings(state);
      expect(state.recentRepos).toEqual(['/repo/a', '/repo/b']);
      expect(state.pinnedRepos).toEqual(['/pinned/a']);
    });

    it('store 為空時保持空陣列', async () => {
      const state = createMockState();
      mockStore.get.mockResolvedValueOnce(null);
      mockStore.get.mockResolvedValueOnce(null);

      await loadAppSettings(state);
      expect(state.recentRepos).toEqual([]);
      expect(state.pinnedRepos).toEqual([]);
    });

    it('store 拋錯時不 crash', async () => {
      const state = createMockState();
      mockStore.get.mockRejectedValueOnce(new Error('store error'));

      await expect(loadAppSettings(state)).resolves.toBeUndefined();
      expect(state.recentRepos).toEqual([]);
    });
  });

  describe('addRecentRepo', () => {
    it('新增路徑並寫入 store', async () => {
      const state = createMockState();
      mockStore.set.mockResolvedValueOnce(undefined);

      await addRecentRepo(state, '/new/repo');
      expect(state.recentRepos[0]).toBe('/new/repo');
      expect(mockStore.set).toHaveBeenCalledWith('recentRepos', state.recentRepos);
    });

    it('重複路徑移到最前面', async () => {
      const state = createMockState();
      state.recentRepos = ['/repo/a', '/repo/b', '/repo/c'];
      mockStore.set.mockResolvedValueOnce(undefined);

      await addRecentRepo(state, '/repo/c');
      expect(state.recentRepos).toEqual(['/repo/c', '/repo/a', '/repo/b']);
    });

    it('最多保留 10 筆', async () => {
      const state = createMockState();
      state.recentRepos = Array.from({ length: 10 }, (_, i) => `/repo/${i}`);
      mockStore.set.mockResolvedValueOnce(undefined);

      await addRecentRepo(state, '/repo/new');
      expect(state.recentRepos.length).toBe(10);
      expect(state.recentRepos[0]).toBe('/repo/new');
    });
  });

  describe('removeRecentRepo', () => {
    it('移除並寫入 store', async () => {
      const state = createMockState();
      state.recentRepos = ['/repo/a', '/repo/b'];
      mockStore.set.mockResolvedValueOnce(undefined);

      await removeRecentRepo(state, '/repo/a');
      expect(state.recentRepos).toEqual(['/repo/b']);
      expect(mockStore.set).toHaveBeenCalled();
    });
  });

  describe('pin management', () => {
    it('isPinned 正確判斷', () => {
      const state = createMockState();
      state.pinnedRepos = ['/repo/a'];
      expect(isPinned(state, '/repo/a')).toBe(true);
      expect(isPinned(state, '/repo/b')).toBe(false);
    });

    it('pinRepo 新增到 pinnedRepos', async () => {
      const state = createMockState();
      mockStore.set.mockResolvedValueOnce(undefined);

      await pinRepo(state, '/repo/a');
      expect(state.pinnedRepos).toContain('/repo/a');
    });

    it('pinRepo 已 pin 時不重複', async () => {
      const state = createMockState();
      state.pinnedRepos = ['/repo/a'];

      await pinRepo(state, '/repo/a');
      expect(state.pinnedRepos).toEqual(['/repo/a']);
    });

    it('unpinRepo 從 pinnedRepos 移除', async () => {
      const state = createMockState();
      state.pinnedRepos = ['/repo/a', '/repo/b'];
      mockStore.set.mockResolvedValueOnce(undefined);

      await unpinRepo(state, '/repo/a');
      expect(state.pinnedRepos).not.toContain('/repo/a');
      expect(state.pinnedRepos).toContain('/repo/b');
    });

    it('togglePin 切換 pin 狀態', async () => {
      const state = createMockState();
      mockStore.set.mockResolvedValue(undefined);

      await togglePin(state, '/repo/a');
      expect(isPinned(state, '/repo/a')).toBe(true);

      await togglePin(state, '/repo/a');
      expect(isPinned(state, '/repo/a')).toBe(false);
    });
  });
});
