
export interface GitHubItem {
  number: number;
  title: string;
  state: string;
  url: string;
  author?: { login: string };
  updatedAt?: string;
}
import {
  getGithubPrs,
  getGithubIssues,
  extractErrorMessage,
} from '$lib/utils/error';
import type {
  FileStatus,
  Commit,
  DiffResult,
  Branch,
  ConflictFile,
  ConflictContent,
  LogFilter,
  BranchCompareResult,
  TagInfo,
  RebaseResult,
  RebaseCommit,
} from '$lib/git/types';
import { gitBranchCompare } from '$lib/git/commands';
import {
  loadAppSettings as _loadAppSettings,
  addRecentRepo as _addRecentRepo,
  removeRecentRepo as _removeRecentRepo,
  isPinned as _isPinned,
  pinRepo as _pinRepo,
  unpinRepo as _unpinRepo,
  togglePin as _togglePin,
  savePreferredEditor as _savePreferredEditor,
  reorderPinnedRepos as _reorderPinnedRepos,
  saveAiSettings as _saveAiSettings,
  saveAiReviewEnabled as _saveAiReviewEnabled,
  saveDisableAutoCrlf as _saveDisableAutoCrlf,
  saveIgnoreEol as _saveIgnoreEol,
  saveTerminalShell as _saveTerminalShell,
  saveUseSystemNotification as _saveUseSystemNotification,
} from './persistence.svelte';
import type { AiProviderType, AiLanguage } from './persistence.svelte';
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

export type ViewMode =
  | 'worktree'
  | 'commit-detail'
  | 'conflict-resolution'
  | 'file-history'
  | 'branch-compare'
  | 'rebase-editor'
  | 'ai-branch-manager'
  | 'github';

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
  hunkChoices: Record<number, 'Ours' | 'Theirs' | 'Both' | 'Custom'>;
  // File history
  fileHistoryTarget: string | null;
  // Tags
  tags: TagInfo[];
  // Log filter
  logFilter: LogFilter;
  // Branch compare
  branchCompareResult: BranchCompareResult | null;
  // Rebase
  rebaseBase: string | null;
  rebaseCommits: RebaseCommit[];
  // LFS
  lfsStatus: GitLfsStatus | null;
  // Submodules
  submodules: SubmoduleInfo[];
  // Worktrees
  worktrees: WorktreeInfo[];
  // GitHub
  prs: GitHubItem[];
  issues: GitHubItem[];
  isLoadingRemote: boolean;
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
    tags: [],
    conflictFiles: [],
    activeConflictFile: null,
    conflictContent: null,
    hunkChoices: {},
    fileHistoryTarget: null,
    logFilter: { type: 'Head' },
    branchCompareResult: null,
    rebaseBase: null,
    rebaseCommits: [],
    lfsStatus: null,
    submodules: [],
    worktrees: [],
    prs: [],
    issues: [],
    isLoadingRemote: false,
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

  // ── stashDiff: raw patch text from git stash show -p (not persisted) ──
  stashDiff = $state<string | null>(null);

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

  // ── AI Settings ──
  aiProvider = $state<AiProviderType>('gemini');
  aiApiKey = $state('');
  aiModel = $state('');
  aiCustomPrompt = $state('');
  aiLanguage = $state<AiLanguage>('zh-TW');
  aiOllamaEndpoint = $state('http://localhost:11434');
  aiReviewEnabled = $state(true);

  // ── Git Settings ──
  disableAutoCrlf = $state(true); // 預設開啟：禁止自動轉換換行符
  ignoreEol = $state(false); // 忽略換行符差異（LF/CRLF）

  // ── Terminal Settings ──
  terminalShell = $state<string | null>(null); // null = git-only 模式

  // ── Auto Fetch ──
  autoFetchEnabled = $state(false);
  autoFetchInterval = $state(5); // minutes
  private autoFetchTimer: ReturnType<typeof setInterval> | null = null;

  // ── UI state ──
  loading = $state(false);
  toasts = $state<Toast[]>([]);
  useSystemNotification = $state(false);
  showTerminal = $state(false);
  showAgentDashboard = $state(false);
  isMac = $state(false);

  toggleTerminal(): void {
    this.showTerminal = !this.showTerminal;
  }

  toggleAgentDashboard(): void {
    this.showAgentDashboard = !this.showAgentDashboard;
  }

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

  get tags(): TagInfo[] {
    return this.activeTab?.state.tags ?? [];
  }

  get submodules(): SubmoduleInfo[] {
    return this.activeTab?.state.submodules ?? [];
  }

  get worktrees(): WorktreeInfo[] {
    return this.activeTab?.state.worktrees ?? [];
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

  set tags(value: TagInfo[]) {
    const tab = this.activeTab;
    if (tab) tab.state.tags = value;
  }

  set submodules(value: SubmoduleInfo[]) {
    const tab = this.activeTab;
    if (tab) tab.state.submodules = value;
  }

  set worktrees(value: WorktreeInfo[]) {
    const tab = this.activeTab;
    if (tab) tab.state.worktrees = value;
  }

  get prs(): GitHubItem[] {
    return this.activeTab?.state.prs ?? [];
  }

  set prs(value: GitHubItem[]) {
    const tab = this.activeTab;
    if (tab) tab.state.prs = value;
  }

  get issues(): GitHubItem[] {
    return this.activeTab?.state.issues ?? [];
  }

  set issues(value: GitHubItem[]) {
    const tab = this.activeTab;
    if (tab) tab.state.issues = value;
  }

  get isLoadingRemote(): boolean {
    return this.activeTab?.state.isLoadingRemote ?? false;
  }

  set isLoadingRemote(value: boolean) {
    const tab = this.activeTab;
    if (tab) tab.state.isLoadingRemote = value;
  }

  // ── Tab CRUD ──

  selectCommit(commit: Commit): void {
    this.selectedCommit = commit;
    this.selectedFile = null;
    this.currentDiff = null;
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

  get isInConflictMode(): boolean {
    return this.viewMode === 'conflict-resolution';
  }

  async enterConflictMode(): Promise<void> {
    if (!this.repoPath) return;
    this.viewMode = 'conflict-resolution';
    await _refreshStatus(this);
  }

  exitConflictMode(): void {
    this.viewMode = 'worktree';
  }

  selectConflictFile(path: string): void {
    const tab = this.activeTab;
    if (tab) {
      tab.state.activeConflictFile = path;
    }
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
        this.addToast(extractErrorMessage(e), 'error');
      }
    }
  }

  async loadGitHubData(): Promise<void> {
    const path = this.repoPath;
    const tab = this.activeTab;
    if (!path || !tab) return;

    tab.state.isLoadingRemote = true;
    try {
      const [prs, issues] = await Promise.all([
        getGithubPrs(path),
        getGithubIssues(path)
      ]);
      tab.state.prs = prs;
      tab.state.issues = issues;
    } catch (e: unknown) {
      this.addToast('GitHub data failed to load. Check "gh" CLI auth.', 'error');
    } finally {
      tab.state.isLoadingRemote = false;
    }
  }

  showGitHub(): void {
    this.viewMode = 'github';
    this.loadGitHubData();
  }

  get branchCompareResult(): BranchCompareResult | null {
    return this.activeTab?.state.branchCompareResult ?? null;
  }

  async openRebaseEditor(onto: string): Promise<void> {
    if (!this.repoPath) return;
    const tab = this.activeTab;
    if (!tab) return;

    try {
      this.loading = true;
      // Get commits between current HEAD and onto
      const compare = await gitBranchCompare(this.repoPath, onto, 'HEAD');
      tab.state.rebaseBase = onto;
      tab.state.rebaseCommits = [...compare.commits].reverse().map((c) => ({
        action: 'Pick',
        id: c.id,
        message: c.message,
      }));
      this.viewMode = 'rebase-editor';
    } catch (e: unknown) {
      this.addToast(extractErrorMessage(e), 'error');
    } finally {
      this.loading = false;
    }
  }

  get rebaseCommits(): RebaseCommit[] {
    return this.activeTab?.state.rebaseCommits ?? [];
  }

  get rebaseBase(): string | null {
    return this.activeTab?.state.rebaseBase ?? null;
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
        this.addToast(extractErrorMessage(e), 'error');
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

  async loadAppSettings() {
    await _loadAppSettings(this);
    // 載入持久化設定後，同步 CRLF 設定到 Rust 後端
    this.syncDisableAutoCrlf();
  }
  async addRecentRepo(path: string) {
    return _addRecentRepo(this, path);
  }
  async removeRecentRepo(path: string) {
    return _removeRecentRepo(this, path);
  }
  isPinned(path: string) {
    return _isPinned(this, path);
  }
  async pinRepo(path: string) {
    return _pinRepo(this, path);
  }
  async unpinRepo(path: string) {
    return _unpinRepo(this, path);
  }
  async togglePin(path: string) {
    return _togglePin(this, path);
  }

  // ── LFS ──

  async lfsTrack(pattern: string): Promise<void> {
    if (!this.repoPath) return;
    try {
      const { gitLfsTrack } = await import('$lib/git/commands');
      await gitLfsTrack(this.repoPath, pattern);
      this.addToast(`已開始追蹤 LFS: ${pattern}`, 'success');
      await _refreshAll(this);
    } catch (e: unknown) {
      this.addToast(extractErrorMessage(e), 'error');
    }
  }

  async lfsUntrack(pattern: string): Promise<void> {
    if (!this.repoPath) return;
    try {
      const { gitLfsUntrack } = await import('$lib/git/commands');
      await gitLfsUntrack(this.repoPath, pattern);
      this.addToast(`已停止追蹤 LFS: ${pattern}`, 'success');
      await _refreshAll(this);
    } catch (e: unknown) {
      this.addToast(extractErrorMessage(e), 'error');
    }
  }

  async savePreferredEditor(editor: string | null) {
    return _savePreferredEditor(this, editor);
  }
  async reorderPinnedRepos(newOrder: string[]) {
    return _reorderPinnedRepos(this, newOrder);
  }
  async saveAiSettings() {
    return _saveAiSettings(this);
  }
  async saveAiReviewEnabled() {
    return _saveAiReviewEnabled(this);
  }
  async saveDisableAutoCrlf() {
    return _saveDisableAutoCrlf(this);
  }
  async saveIgnoreEol() {
    return _saveIgnoreEol(this);
  }
  async saveTerminalShell() {
    return _saveTerminalShell(this);
  }
  async saveUseSystemNotification() {
    return _saveUseSystemNotification(this);
  }

  // ── Auto Fetch ──

  startAutoFetch(): void {
    this.stopAutoFetch();
    if (!this.autoFetchEnabled) return;

    const intervalMs = this.autoFetchInterval * 60 * 1000;
    this.autoFetchTimer = setInterval(async () => {
      const path = this.repoPath;
      if (!path) return;
      try {
        const { gitFetch } = await import('$lib/git/commands');
        await gitFetch(path);
        // Silently refresh commits and branches
        await _refreshAll(this);
      } catch {
        // Silent fail — auto fetch should not disturb the user
      }
    }, intervalMs);
  }

  stopAutoFetch(): void {
    if (this.autoFetchTimer) {
      clearInterval(this.autoFetchTimer);
      this.autoFetchTimer = null;
    }
  }

  setAutoFetch(enabled: boolean, interval?: number): void {
    this.autoFetchEnabled = enabled;
    if (interval !== undefined) this.autoFetchInterval = interval;
    try {
      localStorage.setItem(
        'gitbiker-auto-fetch',
        JSON.stringify({
          enabled: this.autoFetchEnabled,
          interval: this.autoFetchInterval,
        }),
      );
    } catch {}
    if (enabled) {
      this.startAutoFetch();
    } else {
      this.stopAutoFetch();
    }
  }

  loadAutoFetchSettings(): void {
    try {
      const raw = localStorage.getItem('gitbiker-auto-fetch');
      if (raw) {
        const { enabled, interval } = JSON.parse(raw);
        this.autoFetchEnabled = !!enabled;
        this.autoFetchInterval = typeof interval === 'number' ? interval : 5;
      }
    } catch {}
    if (this.autoFetchEnabled) {
      this.startAutoFetch();
    }
  }

  // ── Git action wrappers (maintain existing API) ──

  async refreshStatus() {
    return _refreshStatus(this);
  }
  async refreshAll() {
    return _refreshAll(this);
  }
  async loadDiff(filePath: string) {
    return _loadDiff(this, filePath);
  }

  // ── Toast ──

  addToast(message: string, type: Toast['type'], autoDismiss = true) {
    // 若啟用系統通知，且非需手動關閉的錯誤 → 用 OS 通知
    if (this.useSystemNotification && autoDismiss) {
      import('@tauri-apps/plugin-notification').then(({ sendNotification }) => {
        sendNotification({ title: 'GitBiker', body: message });
      });
      return;
    }

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
      : null) || 'system',
  );

  systemPrefersDark = $state(
    typeof window !== 'undefined'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches
      : true,
  );

  get resolvedTheme(): 'dark' | 'light' {
    if (this.theme === 'system') {
      return this.systemPrefersDark ? 'dark' : 'light';
    }
    return this.theme;
  }

  constructor() {
    if (typeof window !== 'undefined') {
      this.isMac = window.navigator.userAgent.includes('Mac');
      const mq = window.matchMedia('(prefers-color-scheme: dark)');
      mq.addEventListener('change', (e) => {
        this.systemPrefersDark = e.matches;
        if (this.theme === 'system') this.applyTheme();
      });
      this.applyTheme();
      this.loadAutoFetchSettings();
      this.syncDisableAutoCrlf();
      this.syncIgnoreEol();
    }
  }

  /** 將 disableAutoCrlf 設定同步到 Rust 後端 */
  async syncDisableAutoCrlf(): Promise<void> {
    try {
      const { setGitDisableAutoCrlf } = await import('$lib/git/commands');
      await setGitDisableAutoCrlf(this.disableAutoCrlf);
    } catch {}
  }

  /** 將 ignoreEol 設定同步到 Rust 後端 */
  async syncIgnoreEol(): Promise<void> {
    try {
      const { setGitIgnoreEol } = await import('$lib/git/commands');
      await setGitIgnoreEol(this.ignoreEol);
    } catch {}
  }

  setTheme(value: 'system' | 'dark' | 'light') {
    this.theme = value;
    try {
      localStorage.setItem('gitbiker-theme', value);
    } catch {}
    this.applyTheme();
  }

  private applyTheme() {
    document.documentElement.setAttribute('data-theme', this.resolvedTheme);
  }
}

export const app = new AppState();
