import type { FileStatus, Commit, DiffResult, Branch } from '$lib/git/types';
import { gitStatus, gitLog, gitBranches, gitDiff, startWatching, stopWatching } from '$lib/git/commands';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { load, type Store } from '@tauri-apps/plugin-store';

const STORE_FILE = 'app-settings.json';
const RECENT_REPOS_KEY = 'recentRepos';
const PINNED_REPOS_KEY = 'pinnedRepos';
const MAX_RECENT_REPOS = 10;
const MAX_COMMITS_PER_TAB = 200;

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(STORE_FILE);
  }
  return storeInstance;
}

// ── Types ──────────────────────────────────────────────

export interface Toast {
  id: number;
  message: string;
  type: 'success' | 'error' | 'info';
  autoDismiss: boolean;
}

export interface RepoState {
  stagedFiles: FileStatus[];
  unstagedFiles: FileStatus[];
  commits: Commit[];
  branches: Branch[];
  currentBranch: string;
  selectedFile: string | null;
  // currentDiff intentionally excluded — cleared on tab switch to save memory
}

export interface RepoTab {
  id: string;
  path: string;
  name: string;
  state: RepoState;
}

// ── Helpers ────────────────────────────────────────────

function repoNameFromPath(path: string): string {
  const parts = path.replace(/\\/g, '/').split('/');
  return parts[parts.length - 1] || '';
}

function createEmptyState(): RepoState {
  return {
    stagedFiles: [],
    unstagedFiles: [],
    commits: [],
    branches: [],
    currentBranch: '',
    selectedFile: null,
  };
}

// ── AppState ───────────────────────────────────────────
//
//  Multi-tab state management:
//
//  tabs[]  ◄── all open repo tabs
//    │
//    ├── activeTabId ── which tab is shown
//    │       │
//    │       └── activeTab (computed) ── convenience accessor
//    │
//    └── each tab holds its own RepoState
//
//  On tab switch:
//    1. Save current state into tab
//    2. Restore target tab state into activeTab
//    3. Restart fs watcher for new repo
//    4. Background refresh gitStatus
//    5. currentDiff is NOT saved (memory optimization)
//

let toastCounter = 0;

class AppState {
  // ── Tab management ──
  tabs = $state<RepoTab[]>([]);
  activeTabId = $state<string | null>(null);

  // ── currentDiff lives outside tabs (not persisted across tab switches) ──
  currentDiff = $state<DiffResult | null>(null);

  // ── Repo lists ──
  recentRepos = $state<string[]>([]);
  pinnedRepos = $state<string[]>([]);

  // ── UI state ──
  loading = $state(false);
  toasts = $state<Toast[]>([]);

  // ── Watcher ──
  private unlistenGitChanged: UnlistenFn | null = null;
  private switchGuard = false;

  // ── Computed ──

  get activeTab(): RepoTab | null {
    if (!this.activeTabId) return null;
    return this.tabs.find((t) => t.id === this.activeTabId) ?? null;
  }

  get hasRepo(): boolean {
    return this.tabs.length > 0 && this.activeTabId !== null;
  }

  get repoPath(): string | null {
    return this.activeTab?.path ?? null;
  }

  get repoName(): string {
    return this.activeTab?.name ?? '';
  }

  get stagedFiles(): FileStatus[] {
    return this.activeTab?.state.stagedFiles ?? [];
  }

  get unstagedFiles(): FileStatus[] {
    return this.activeTab?.state.unstagedFiles ?? [];
  }

  get commits(): Commit[] {
    return this.activeTab?.state.commits ?? [];
  }

  get branches(): Branch[] {
    return this.activeTab?.state.branches ?? [];
  }

  get currentBranch(): string {
    return this.activeTab?.state.currentBranch ?? '';
  }

  get selectedFile(): string | null {
    return this.activeTab?.state.selectedFile ?? null;
  }

  // ── Setters that write to activeTab ──

  set currentBranch(value: string) {
    const tab = this.activeTab;
    if (tab) tab.state.currentBranch = value;
  }

  set selectedFile(value: string | null) {
    const tab = this.activeTab;
    if (tab) tab.state.selectedFile = value;
  }

  set stagedFiles(value: FileStatus[]) {
    const tab = this.activeTab;
    if (tab) tab.state.stagedFiles = value;
  }

  set unstagedFiles(value: FileStatus[]) {
    const tab = this.activeTab;
    if (tab) tab.state.unstagedFiles = value;
  }

  set commits(value: Commit[]) {
    const tab = this.activeTab;
    if (tab) tab.state.commits = value;
  }

  set branches(value: Branch[]) {
    const tab = this.activeTab;
    if (tab) tab.state.branches = value;
  }

  // ── Tab CRUD ──

  async openRepo(path: string, background = false): Promise<void> {
    // If already open, just switch to it
    const existing = this.tabs.find((t) => t.path === path);
    if (existing) {
      if (!background) await this.switchTab(existing.id);
      return;
    }

    const tabId = crypto.randomUUID();
    const tab: RepoTab = {
      id: tabId,
      path,
      name: repoNameFromPath(path),
      state: createEmptyState(),
    };

    this.tabs = [...this.tabs, tab];

    if (!background) {
      this.activeTabId = tabId;
      this.currentDiff = null;
      this.loading = true;

      try {
        const [status, commits, branches] = await Promise.all([
          gitStatus(path),
          gitLog(path, MAX_COMMITS_PER_TAB),
          gitBranches(path),
        ]);

        // Use reactive proxy from this.tabs, not the original local variable
        const reactiveTab = this.tabs.find((t) => t.id === tabId);
        if (reactiveTab) {
          reactiveTab.state.stagedFiles = status.filter((f) => f.staging === 'Staged');
          reactiveTab.state.unstagedFiles = status.filter((f) => f.staging === 'Unstaged');
          reactiveTab.state.commits = commits;
          reactiveTab.state.branches = branches;
          reactiveTab.state.currentBranch = branches.find((b) => b.is_current)?.name || 'main';
        }

        await this.addRecentRepo(path);
        await this.setupWatcher(path);
      } catch (e: unknown) {
        // Remove the tab if loading failed
        this.tabs = this.tabs.filter((t) => t.id !== tabId);
        if (this.activeTabId === tabId) {
          this.activeTabId = this.tabs.length > 0 ? this.tabs[this.tabs.length - 1].id : null;
        }
        this.addToast(String(e), 'error');
      } finally {
        this.loading = false;
      }
    }
  }

  async switchTab(id: string): Promise<void> {
    if (this.activeTabId === id || this.switchGuard) return;

    const target = this.tabs.find((t) => t.id === id);
    if (!target) return;

    this.switchGuard = true;
    try {
      this.activeTabId = id;
      this.currentDiff = null;

      await this.setupWatcher(target.path);

      // Background refresh
      this.refreshStatus().catch(() => {});
    } finally {
      this.switchGuard = false;
    }
  }

  closeTab(id: string): void {
    const idx = this.tabs.findIndex((t) => t.id === id);
    if (idx === -1) return;

    const wasActive = this.activeTabId === id;
    this.tabs = this.tabs.filter((t) => t.id !== id);

    if (wasActive) {
      if (this.tabs.length === 0) {
        this.activeTabId = null;
        this.currentDiff = null;
        this.teardownWatcher();
      } else {
        // Activate adjacent tab
        const newIdx = Math.min(idx, this.tabs.length - 1);
        this.switchTab(this.tabs[newIdx].id);
      }
    }
  }

  closeOtherTabs(keepId: string): void {
    this.tabs = this.tabs.filter((t) => t.id === keepId);
    if (this.activeTabId !== keepId) {
      this.switchTab(keepId);
    }
  }

  closeAllTabs(): void {
    this.tabs = [];
    this.activeTabId = null;
    this.currentDiff = null;
    this.teardownWatcher();
  }

  // ── Tab info helpers ──

  dirtyCount(tabId: string): number {
    const tab = this.tabs.find((t) => t.id === tabId);
    if (!tab) return 0;
    return tab.state.stagedFiles.length + tab.state.unstagedFiles.length;
  }

  tabBranch(tabId: string): string {
    const tab = this.tabs.find((t) => t.id === tabId);
    return tab?.state.currentBranch ?? '';
  }

  /** Disambiguate tab names when duplicates exist */
  displayName(tab: RepoTab): string {
    const dupes = this.tabs.filter((t) => t.name === tab.name);
    if (dupes.length <= 1) return tab.name;
    // Show parent folder
    const parts = tab.path.replace(/\\/g, '/').split('/');
    const parent = parts.length >= 2 ? parts[parts.length - 2] : '';
    return parent ? `${parent}/${tab.name}` : tab.name;
  }

  // ── Pin management ──

  isPinned(path: string): boolean {
    return this.pinnedRepos.includes(path);
  }

  async pinRepo(path: string): Promise<void> {
    if (this.isPinned(path)) return;
    this.pinnedRepos = [...this.pinnedRepos, path];
    await this.savePinnedRepos();
  }

  async unpinRepo(path: string): Promise<void> {
    this.pinnedRepos = this.pinnedRepos.filter((r) => r !== path);
    await this.savePinnedRepos();
  }

  async togglePin(path: string): Promise<void> {
    if (this.isPinned(path)) {
      await this.unpinRepo(path);
    } else {
      await this.pinRepo(path);
    }
  }

  // ── Persistence ──

  async loadRecentRepos(): Promise<void> {
    try {
      const store = await getStore();
      const [savedRecent, savedPinned] = await Promise.all([
        store.get<string[]>(RECENT_REPOS_KEY),
        store.get<string[]>(PINNED_REPOS_KEY),
      ]);
      if (Array.isArray(savedRecent)) {
        this.recentRepos = savedRecent.slice(0, MAX_RECENT_REPOS);
      }
      if (Array.isArray(savedPinned)) {
        this.pinnedRepos = savedPinned;
      }
    } catch {
      // 首次啟動 store 檔案不存在，忽略
    }
  }

  async addRecentRepo(path: string): Promise<void> {
    const filtered = this.recentRepos.filter((r) => r !== path);
    this.recentRepos = [path, ...filtered].slice(0, MAX_RECENT_REPOS);
    try {
      const store = await getStore();
      await store.set(RECENT_REPOS_KEY, this.recentRepos);
    } catch {
      // 寫入失敗不影響功能
    }
  }

  async removeRecentRepo(path: string): Promise<void> {
    this.recentRepos = this.recentRepos.filter((r) => r !== path);
    try {
      const store = await getStore();
      await store.set(RECENT_REPOS_KEY, this.recentRepos);
    } catch {}
  }

  private async savePinnedRepos(): Promise<void> {
    try {
      const store = await getStore();
      await store.set(PINNED_REPOS_KEY, this.pinnedRepos);
    } catch {}
  }

  // ── Refresh ──

  async refreshStatus(): Promise<void> {
    const tab = this.activeTab;
    if (!tab) return;
    try {
      const status = await gitStatus(tab.path);
      tab.state.stagedFiles = status.filter((f) => f.staging === 'Staged');
      tab.state.unstagedFiles = status.filter((f) => f.staging === 'Unstaged');
    } catch (e: unknown) {
      this.addToast(String(e), 'error');
    }
  }

  async refreshAll(): Promise<void> {
    const tab = this.activeTab;
    if (!tab) return;
    try {
      const [status, commits, branches] = await Promise.all([
        gitStatus(tab.path),
        gitLog(tab.path, MAX_COMMITS_PER_TAB),
        gitBranches(tab.path),
      ]);
      tab.state.stagedFiles = status.filter((f) => f.staging === 'Staged');
      tab.state.unstagedFiles = status.filter((f) => f.staging === 'Unstaged');
      tab.state.commits = commits;
      tab.state.branches = branches;
      tab.state.currentBranch = branches.find((b) => b.is_current)?.name || tab.state.currentBranch;
    } catch (e: unknown) {
      this.addToast(String(e), 'error');
    }
  }

  // ── Diff ──

  async loadDiff(filePath: string): Promise<void> {
    const tab = this.activeTab;
    if (!tab) return;
    try {
      this.currentDiff = await gitDiff(tab.path, filePath);
    } catch (e: unknown) {
      this.addToast(String(e), 'error');
    }
  }

  // ── Watcher ──

  private async setupWatcher(path: string): Promise<void> {
    this.teardownWatcher();

    try {
      await startWatching(path);
      this.unlistenGitChanged = await listen<string>('git-changed', (event) => {
        // Only process events for the active tab's repo
        if (this.repoPath !== path) return;

        if (event.payload === 'index') {
          this.refreshStatus();
        } else {
          this.refreshAll();
        }
      });
    } catch (e: unknown) {
      console.warn('fs watcher 啟動失敗:', e);
    }
  }

  private teardownWatcher(): void {
    if (this.unlistenGitChanged) {
      this.unlistenGitChanged();
      this.unlistenGitChanged = null;
    }
    stopWatching().catch(() => {});
  }

  // ── Toast ──

  addToast(message: string, type: Toast['type'], autoDismiss = true) {
    const id = ++toastCounter;
    this.toasts = [...this.toasts, { id, message, type, autoDismiss }];

    if (autoDismiss) {
      const delay = type === 'error' ? 5000 : 2000;
      setTimeout(() => this.removeToast(id), delay);
    }
  }

  removeToast(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  // ── Theme ──

  theme = $state<'system' | 'dark' | 'light'>(
    (typeof localStorage !== 'undefined'
      ? (localStorage.getItem('gitbiker-theme') as 'system' | 'dark' | 'light')
      : null) || 'system'
  );

  systemPrefersDark = $state(
    typeof window !== 'undefined'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches
      : true
  );

  get resolvedTheme(): 'dark' | 'light' {
    if (this.theme === 'system') {
      return this.systemPrefersDark ? 'dark' : 'light';
    }
    return this.theme;
  }

  constructor() {
    if (typeof window !== 'undefined') {
      const mq = window.matchMedia('(prefers-color-scheme: dark)');
      mq.addEventListener('change', (e) => {
        this.systemPrefersDark = e.matches;
        if (this.theme === 'system') this.applyTheme();
      });
      this.applyTheme();
    }
  }

  setTheme(value: 'system' | 'dark' | 'light') {
    this.theme = value;
    try { localStorage.setItem('gitbiker-theme', value); } catch {}
    this.applyTheme();
  }

  private applyTheme() {
    document.documentElement.setAttribute('data-theme', this.resolvedTheme);
  }
}

export const app = new AppState();
