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
}

// ── Recent Repos ──────────────────────────────────────

export async function loadRecentRepos(state: PersistableState): Promise<void> {
  try {
    const store = await getStore();
    const [savedRecent, savedPinned, savedEditor, savedAiProvider, savedAiKey, savedAiModel, savedAiPrompt, savedAiLang, savedAiEndpoint] = await Promise.all([
      store.get<string[]>(RECENT_REPOS_KEY),
      store.get<string[]>(PINNED_REPOS_KEY),
      store.get<string | null>(PREFERRED_EDITOR_KEY),
      store.get<AiProviderType>(AI_PROVIDER_KEY),
      store.get<string>(AI_API_KEY_KEY),
      store.get<string>(AI_MODEL_KEY),
      store.get<string>(AI_CUSTOM_PROMPT_KEY),
      store.get<AiLanguage>(AI_LANGUAGE_KEY),
      store.get<string>(AI_OLLAMA_ENDPOINT_KEY),
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

// ── Test Helpers ──────────────────────────────────────

export function _resetStoreForTest(): void {
  storeInstance = null;
}
