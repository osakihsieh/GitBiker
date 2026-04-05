import type { FileStatus, Commit, DiffResult, Branch, ConflictFile, ConflictContent, LogFilter, BranchCompareResult } from '$lib/git/types';
import { gitGetConflictFiles, gitGetConflictContent, gitBranchCompare } from '$lib/git/commands';
import {
  loadRecentRepos as _loadRecentRepos,
  addRecentRepo as _addRecentRepo,
  removeRecentRepo as _removeRecentRepo,
  isPinned as _isPinned,
  pinRepo as _pinRepo,
  unpinRepo as _unpinRepo,
  togglePin as _togglePin,
  savePreferredEditor as _savePreferredEditor,
  reorderPinnedRepos as _reorderPinnedRepos,
} from './persistence.svelte';
import {
  refreshStatus as _refreshStatus,
  refreshAll as _refreshAll,
  loadDiff as _loadDiff,
  loadRepoData,
  setupWatcher,
  teardownWatcher,
} from './git-actions.svelte';

// ── Types ──────────────────────────────────────────────

export interface Toast {
  id: number;
  message: string;
  type: 'success' | 'error' | 'info';
  autoDismiss: boolean;
}

export type ViewMode = 'worktree' | 'commit-detail' | 'conflict-resolution' | 'file-history' | 'branch-compare';

export interface RepoState {
  stagedFiles: FileStatus[];
  unstagedFiles: FileStatus[];
  commits: Commit[];
  branches: Branch[];
  currentBranch: string;
  selectedFile: string | null;
  viewMode: ViewMode;
  // Conflict resolution state
  conflictFiles: ConflictFile[];
  activeConflictFile: string | null;
  conflictContent: ConflictContent | null;
  hunkChoices: Record<number, 'Ours' | 'Theirs' | 'Both'>;
  // File history
  fileHistoryTarget: string | null;
  // Log filter
  logFilter: LogFilter;
  // Branch compare
  branchCompareResult: BranchCompareResult | null;
}
  export interface RepoTab {
  id: string;
  path: string;
  name: string;
  state: RepoState;
  }

  // ── Helpers ────────────────────────────────────────────

  export function repoNameFromPath(path: string): string {
  const parts = path.replace(/\\/g, '/').split('/');
  return parts[parts.length - 1] || '';
  }

export function createEmptyState(): RepoState {
  return {
    stagedFiles: [],
    unstagedFiles: [],
    commits: [],
    branches: [],
    currentBranch: '',
    selectedFile: null,
    viewMode: 'worktree',
    conflictFiles: [],
    activeConflictFile: null,
    conflictContent: null,
    hunkChoices: {},
    fileHistoryTarget: null,
    logFilter: { type: 'Head' },
    branchCompareResult: null,
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
//  Sidebar modes:
//    viewMode = 'worktree'       ← staged/unstaged + commit form
//    viewMode = 'commit-detail'  ← selected commit's file list
//
//  On tab switch:
//    1. Restore target tab state
//    2. Restart fs watcher for new repo
//    3. Background refresh gitStatus
//    4. Reset viewMode to 'worktree'
//    5. currentDiff is NOT saved (memory optimization)
//

let toastCounter = 0;

class AppState {
  // ── Tab management ──
  tabs = $state<RepoTab[]>([]);
  activeTabId = $state<string | null>(null);

  // ── currentDiff lives outside tabs (not persisted across tab switches) ──
  currentDiff = $state<DiffResult | null>(null);

  // ── View mode (proxied to active tab's RepoState) ──
  selectedCommit = $state<Commit | null>(null);

  get viewMode(): ViewMode {
    return this.activeTab?.state.viewMode ?? 'worktree';
  }

  set viewMode(value: ViewMode) {
    const tab = this.activeTab;
    if (tab) tab.state.viewMode = value;
  }

  // ── Repo lists ──
  recentRepos = $state<string[]>([]);
  pinnedRepos = $state<string[]>([]);

  // ── Editor preference ──
  preferredEditor = $state<string | null>(null);

  // ── UI state ──
  loading = $state(false);
  toasts = $state<Toast[]>([]);

  // ── Switch guard ──
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

  // ── Conflict State (proxied to active tab) ──

  get conflictFiles(): ConflictFile[] {
    return this.activeTab?.state.conflictFiles ?? [];
  }

  get activeConflictFile(): string | null {
    return this.activeTab?.state.activeConflictFile ?? null;
  }

  get conflictContent(): ConflictContent | null {
    return this.activeTab?.state.conflictContent ?? null;
  }

  get hunkChoices(): Record<number, 'Ours' | 'Theirs' | 'Both'> {
    return this.activeTab?.state.hunkChoices ?? {};
  }

  get isInConflictMode(): boolean {
    return this.viewMode === 'conflict-resolution';
  }

  get conflictResolvedCount(): number {
    const files = this.conflictFiles;
    const active = this.activeConflictFile;
    // A file is "resolved" if it's no longer in the conflict list after refresh
    // For now, count is managed via the conflict files list length vs initial
    return 0; // Will be computed from actual state
  }

  // ── View Mode ──

  selectCommit(commit: Commit): void {
    this.selectedCommit = commit;
    this.viewMode = 'commit-detail';
  }

  backToWorktree(): void {
    this.selectedCommit = null;
    this.viewMode = 'worktree';
    const tab = this.activeTab;
    if (tab) tab.state.fileHistoryTarget = null;
  }

  showFileHistory(filePath: string): void {
    this.selectedCommit = null;
    this.viewMode = 'file-history';
    const tab = this.activeTab;
    if (tab) tab.state.fileHistoryTarget = filePath;
  }

  setLogFilter(filter: LogFilter): void {
    const tab = this.activeTab;
    if (tab) {
      tab.state.logFilter = filter;
      _refreshAll(this);
    }
  }

  get logFilter(): LogFilter {
    return this.activeTab?.state.logFilter ?? { type: 'Head' };
  }

  get fileHistoryTarget(): string | null {
    return this.activeTab?.state.fileHistoryTarget ?? null;
  }

  async compareBranches(base: string, compare: string): Promise<void> {
    if (!this.repoPath) return;
    this.viewMode = 'branch-compare';
    const tab = this.activeTab;
    if (tab) {
      try {
        tab.state.branchCompareResult = await gitBranchCompare(this.repoPath, base, compare);
      } catch (e: unknown) {
        this.addToast(String(e), 'error');
      }
    }
  }

  get branchCompareResult(): BranchCompareResult | null {
    return this.activeTab?.state.branchCompareResult ?? null;
  }

  // ── Conflict Resolution Methods ──

  async enterConflictMode(): Promise<void> {
    const path = this.repoPath;
    if (!path) return;

    try {
      const files = await gitGetConflictFiles(path);
      const tab = this.activeTab;
      if (!tab) return;
      tab.state.conflictFiles = files;
      tab.state.activeConflictFile = files.length > 0 ? files[0].path : null;
      tab.state.conflictContent = null;
      tab.state.hunkChoices = {};
      tab.state.viewMode = 'conflict-resolution';

      if (files.length > 0) {
        await this.selectConflictFile(files[0].path);
      }
    } catch (e: unknown) {
      this.addToast(String(e), 'error');
    }
  }

  exitConflictMode(): void {
    const tab = this.activeTab;
    if (!tab) return;
    tab.state.conflictFiles = [];
    tab.state.activeConflictFile = null;
    tab.state.conflictContent = null;
    tab.state.hunkChoices = {};
    tab.state.viewMode = 'worktree';
    this.selectedCommit = null;
  }

  async selectConflictFile(filePath: string): Promise<void> {
    const path = this.repoPath;
    const tab = this.activeTab;
    if (!path || !tab) return;

    tab.state.activeConflictFile = filePath;
    tab.state.hunkChoices = {};

    try {
      const content = await gitGetConflictContent(path, filePath);
      tab.state.conflictContent = content;
    } catch (e: unknown) {
      tab.state.conflictContent = null;
      this.addToast(String(e), 'error');
    }
  }

  setHunkChoice(hunkIndex: number, choice: 'Ours' | 'Theirs' | 'Both'): void {
    const tab = this.activeTab;
    if (!tab) return;
    tab.state.hunkChoices = { ...tab.state.hunkChoices, [hunkIndex]: choice };
  }

  async refreshConflictFiles(): Promise<void> {
    const path = this.repoPath;
    const tab = this.activeTab;
    if (!path || !tab) return;

    try {
      const files = await gitGetConflictFiles(path);
      tab.state.conflictFiles = files;
    } catch {
      // If not in merge state anymore, exit conflict mode
      this.exitConflictMode();
    }
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
      // viewMode defaults to 'worktree' via createEmptyState()
      this.selectedCommit = null;
      this.loading = true;

      try {
        await loadRepoData(this, tabId, path);
        await _addRecentRepo(this, path);
        await setupWatcher(this, path);
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
      // viewMode is now per-tab in RepoState — no reset needed
      this.selectedCommit = null;

      await setupWatcher(this, target.path);

      // Background refresh
      _refreshStatus(this).catch(() => {});
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
        this.selectedCommit = null;
        teardownWatcher();
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
    this.selectedCommit = null;
    teardownWatcher();
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

  // ── Persistence wrappers (maintain existing API) ──

  async loadRecentRepos() { return _loadRecentRepos(this); }
  async addRecentRepo(path: string) { return _addRecentRepo(this, path); }
  async removeRecentRepo(path: string) { return _removeRecentRepo(this, path); }
  isPinned(path: string) { return _isPinned(this, path); }
  async pinRepo(path: string) { return _pinRepo(this, path); }
  async unpinRepo(path: string) { return _unpinRepo(this, path); }
  async togglePin(path: string) { return _togglePin(this, path); }
  async savePreferredEditor(editor: string | null) { return _savePreferredEditor(this, editor); }
  async reorderPinnedRepos(newOrder: string[]) { return _reorderPinnedRepos(this, newOrder); }

  // ── Git action wrappers (maintain existing API) ──

  async refreshStatus() { return _refreshStatus(this); }
  async refreshAll() { return _refreshAll(this); }
  async loadDiff(filePath: string) { return _loadDiff(this, filePath); }

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
