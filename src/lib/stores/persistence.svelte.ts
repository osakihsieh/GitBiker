import { load, type Store } from '@tauri-apps/plugin-store';

const STORE_FILE = 'app-settings.json';
const RECENT_REPOS_KEY = 'recentRepos';
const PINNED_REPOS_KEY = 'pinnedRepos';
const PREFERRED_EDITOR_KEY = 'preferredEditor';
const AI_PROVIDER_KEY = 'aiProvider';
const AI_API_KEY_KEY = 'aiApiKey';
const AI_MODEL_KEY = 'aiModel';
const AI_CUSTOM_PROMPT_KEY = 'aiCustomPrompt';
const AI_LANGUAGE_KEY = 'aiLanguage';
const AI_OLLAMA_ENDPOINT_KEY = 'aiOllamaEndpoint';
const DISABLE_AUTO_CRLF_KEY = 'disableAutoCrlf';
const IGNORE_EOL_KEY = 'ignoreEol';
const TERMINAL_SHELL_KEY = 'terminalShell';
const SCAN_PATHS_KEY = 'multiRepoScanPaths';
const CACHED_REPO_INFOS_KEY = 'multiRepoCachedInfos';
const MULTI_REPO_REFRESH_INTERVAL_KEY = 'multiRepoRefreshInterval';
const USE_SYSTEM_NOTIFICATION_KEY = 'useSystemNotification';
const MAX_RECENT_REPOS = 10;

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(STORE_FILE);
  }
  return storeInstance;
}

// ── Types ──────────────────────────────────────────────

export type AiProviderType = 'gemini' | 'openai' | 'ollama';
export type AiLanguage = 'zh-TW' | 'en' | 'auto';

/** 任何有 recentRepos + pinnedRepos 的物件（避免循環 import AppState） */
export interface PersistableState {
  recentRepos: string[];
  pinnedRepos: string[];
  preferredEditor: string | null;
  aiProvider: AiProviderType;
  aiApiKey: string;
  aiModel: string;
  aiCustomPrompt: string;
  aiLanguage: AiLanguage;
  aiOllamaEndpoint: string;
  disableAutoCrlf: boolean;
  ignoreEol: boolean;
  terminalShell: string | null;
  useSystemNotification: boolean;
}

// ── Recent Repos ──────────────────────────────────────

export async function loadAppSettings(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    const [savedRecent, savedPinned, savedEditor, savedAiProvider, savedAiKey, savedAiModel, savedAiPrompt, savedAiLang, savedAiEndpoint, savedDisableAutoCrlf, savedIgnoreEol, savedTerminalShell, savedUseSystemNotification] = await Promise.all([
      store.get<string[]>(RECENT_REPOS_KEY),
      store.get<string[]>(PINNED_REPOS_KEY),
      store.get<string | null>(PREFERRED_EDITOR_KEY),
      store.get<AiProviderType>(AI_PROVIDER_KEY),
      store.get<string>(AI_API_KEY_KEY),
      store.get<string>(AI_MODEL_KEY),
      store.get<string>(AI_CUSTOM_PROMPT_KEY),
      store.get<AiLanguage>(AI_LANGUAGE_KEY),
      store.get<string>(AI_OLLAMA_ENDPOINT_KEY),
      store.get<boolean>(DISABLE_AUTO_CRLF_KEY),
      store.get<boolean>(IGNORE_EOL_KEY),
      store.get<string | null>(TERMINAL_SHELL_KEY),
      store.get<boolean>(USE_SYSTEM_NOTIFICATION_KEY),
    ]);
    if (Array.isArray(savedRecent)) {
      state.recentRepos = savedRecent.slice(0, MAX_RECENT_REPOS);
    }
    if (Array.isArray(savedPinned)) {
      state.pinnedRepos = savedPinned;
    }
    if (typeof savedEditor === 'string') {
      state.preferredEditor = savedEditor;
    }
    if (savedAiProvider) state.aiProvider = savedAiProvider;
    if (typeof savedAiKey === 'string') state.aiApiKey = savedAiKey;
    if (typeof savedAiModel === 'string') state.aiModel = savedAiModel;
    if (typeof savedAiPrompt === 'string') state.aiCustomPrompt = savedAiPrompt;
    if (savedAiLang) state.aiLanguage = savedAiLang;
    if (typeof savedAiEndpoint === 'string') state.aiOllamaEndpoint = savedAiEndpoint;
    if (typeof savedDisableAutoCrlf === 'boolean') state.disableAutoCrlf = savedDisableAutoCrlf;
    if (typeof savedIgnoreEol === 'boolean') state.ignoreEol = savedIgnoreEol;
    if (typeof savedTerminalShell === 'string') state.terminalShell = savedTerminalShell;
    if (typeof savedUseSystemNotification === 'boolean') state.useSystemNotification = savedUseSystemNotification;
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

export async function reorderPinnedRepos(state: PersistableState, newOrder: string[]): Promise<void> {
  state.pinnedRepos = newOrder;
  await savePinnedRepos(state);
}

async function savePinnedRepos(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    await store.set(PINNED_REPOS_KEY, state.pinnedRepos);
  } catch {}
}

// ── Preferred Editor ─────────────────────────────────

export async function savePreferredEditor(state: PersistableState, editor: string | null): Promise<void> {
  state.preferredEditor = editor;
  try {
    const store = await getStore();
    await store.set(PREFERRED_EDITOR_KEY, editor);
  } catch {}
}

// ── AI Settings ─────────────────────────────────────

export async function saveAiSettings(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    await Promise.all([
      store.set(AI_PROVIDER_KEY, state.aiProvider),
      store.set(AI_API_KEY_KEY, state.aiApiKey),
      store.set(AI_MODEL_KEY, state.aiModel),
      store.set(AI_CUSTOM_PROMPT_KEY, state.aiCustomPrompt),
      store.set(AI_LANGUAGE_KEY, state.aiLanguage),
      store.set(AI_OLLAMA_ENDPOINT_KEY, state.aiOllamaEndpoint),
    ]);
  } catch {}
}

// ── Git Settings ───────────────────────────────────────

export async function saveDisableAutoCrlf(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    await store.set(DISABLE_AUTO_CRLF_KEY, state.disableAutoCrlf);
  } catch {}
}

export async function saveIgnoreEol(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    await store.set(IGNORE_EOL_KEY, state.ignoreEol);
  } catch {}
}

// ── Notification Mode ─────────────────────────────────

export async function saveUseSystemNotification(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    await store.set(USE_SYSTEM_NOTIFICATION_KEY, state.useSystemNotification);
  } catch {}
}

// ── Terminal Shell ────────────────────────────────────

export async function saveTerminalShell(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    await store.set(TERMINAL_SHELL_KEY, state.terminalShell);
  } catch {}
}

// ── Multi-Repo Scan Paths ─────────────────────────────

export interface CachedRepoInfo {
  path: string;
  name: string;
  branch: string;
  dirty: number;
  ahead: number;
  behind: number;
  scanPath: string;
}

export async function loadScanPaths(): Promise<string[]> {
  try {
    const store = await getStore();
    const saved = await store.get<string[]>(SCAN_PATHS_KEY);
    return Array.isArray(saved) ? saved : [];
  } catch {
    return [];
  }
}

export async function saveScanPaths(paths: string[]): Promise<void> {
  try {
    const store = await getStore();
    await store.set(SCAN_PATHS_KEY, paths);
  } catch (e: unknown) {
    console.warn('Failed to save scan paths:', e);
  }
}

export async function loadCachedRepoInfos(): Promise<CachedRepoInfo[]> {
  try {
    const store = await getStore();
    const saved = await store.get<CachedRepoInfo[]>(CACHED_REPO_INFOS_KEY);
    return Array.isArray(saved) ? saved : [];
  } catch {
    return [];
  }
}

export async function saveCachedRepoInfos(infos: CachedRepoInfo[]): Promise<void> {
  try {
    const store = await getStore();
    await store.set(CACHED_REPO_INFOS_KEY, infos);
  } catch (e: unknown) {
    console.warn('Failed to save cached repo infos:', e);
  }
}

export async function loadMultiRepoRefreshInterval(): Promise<number> {
  try {
    const store = await getStore();
    const saved = await store.get<number>(MULTI_REPO_REFRESH_INTERVAL_KEY);
    return typeof saved === 'number' && saved > 0 ? saved : 60;
  } catch {
    return 60;
  }
}

export async function saveMultiRepoRefreshInterval(seconds: number): Promise<void> {
  try {
    const store = await getStore();
    await store.set(MULTI_REPO_REFRESH_INTERVAL_KEY, seconds);
  } catch {}
}

// ── Test Helpers ──────────────────────────────────────

export function _resetStoreForTest(): void {
  storeInstance = null;
}
