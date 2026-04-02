import { load, type Store } from '@tauri-apps/plugin-store';

const STORE_FILE = 'app-settings.json';
const RECENT_REPOS_KEY = 'recentRepos';
const PINNED_REPOS_KEY = 'pinnedRepos';
const MAX_RECENT_REPOS = 10;

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(STORE_FILE);
  }
  return storeInstance;
}

// ── Types ──────────────────────────────────────────────

/** 任何有 recentRepos + pinnedRepos 的物件（避免循環 import AppState） */
export interface PersistableState {
  recentRepos: string[];
  pinnedRepos: string[];
}

// ── Recent Repos ──────────────────────────────────────

export async function loadRecentRepos(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    const [savedRecent, savedPinned] = await Promise.all([
      store.get<string[]>(RECENT_REPOS_KEY),
      store.get<string[]>(PINNED_REPOS_KEY),
    ]);
    if (Array.isArray(savedRecent)) {
      state.recentRepos = savedRecent.slice(0, MAX_RECENT_REPOS);
    }
    if (Array.isArray(savedPinned)) {
      state.pinnedRepos = savedPinned;
    }
  } catch {
    // 首次啟動 store 檔案不存在，忽略
  }
}

export async function addRecentRepo(state: PersistableState, path: string): Promise<void> {
  const filtered = state.recentRepos.filter((r) => r !== path);
  state.recentRepos = [path, ...filtered].slice(0, MAX_RECENT_REPOS);
  try {
    const store = await getStore();
    await store.set(RECENT_REPOS_KEY, state.recentRepos);
  } catch {
    // 寫入失敗不影響功能
  }
}

export async function removeRecentRepo(state: PersistableState, path: string): Promise<void> {
  state.recentRepos = state.recentRepos.filter((r) => r !== path);
  try {
    const store = await getStore();
    await store.set(RECENT_REPOS_KEY, state.recentRepos);
  } catch {}
}

// ── Pin Management ────────────────────────────────────

export function isPinned(state: PersistableState, path: string): boolean {
  return state.pinnedRepos.includes(path);
}

export async function pinRepo(state: PersistableState, path: string): Promise<void> {
  if (isPinned(state, path)) return;
  state.pinnedRepos = [...state.pinnedRepos, path];
  await savePinnedRepos(state);
}

export async function unpinRepo(state: PersistableState, path: string): Promise<void> {
  state.pinnedRepos = state.pinnedRepos.filter((r) => r !== path);
  await savePinnedRepos(state);
}

export async function togglePin(state: PersistableState, path: string): Promise<void> {
  if (isPinned(state, path)) {
    await unpinRepo(state, path);
  } else {
    await pinRepo(state, path);
  }
}

async function savePinnedRepos(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    await store.set(PINNED_REPOS_KEY, state.pinnedRepos);
  } catch {}
}

// ── Test Helpers ──────────────────────────────────────

export function _resetStoreForTest(): void {
  storeInstance = null;
}
