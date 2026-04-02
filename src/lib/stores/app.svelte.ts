import type { FileStatus, Commit, DiffResult, Branch } from '$lib/git/types';
import { gitStatus, gitLog, gitBranches } from '$lib/git/commands';
import { load, type Store } from '@tauri-apps/plugin-store';

const STORE_FILE = 'app-settings.json';
const RECENT_REPOS_KEY = 'recentRepos';
const MAX_RECENT_REPOS = 10;

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(STORE_FILE);
  }
  return storeInstance;
}

export interface Toast {
  id: number;
  message: string;
  type: 'success' | 'error' | 'info';
  autoDismiss: boolean;
}

let toastCounter = 0;

class AppState {
  repoPath = $state<string | null>(null);
  recentRepos = $state<string[]>([]);
  stagedFiles = $state<FileStatus[]>([]);
  unstagedFiles = $state<FileStatus[]>([]);
  commits = $state<Commit[]>([]);
  branches = $state<Branch[]>([]);
  currentBranch = $state<string>('');
  selectedFile = $state<string | null>(null);
  currentDiff = $state<DiffResult | null>(null);
  loading = $state(false);
  toasts = $state<Toast[]>([]);

  get hasRepo(): boolean {
    return this.repoPath !== null;
  }

  async loadRecentRepos(): Promise<void> {
    try {
      const store = await getStore();
      const saved = await store.get<string[]>(RECENT_REPOS_KEY);
      if (Array.isArray(saved)) {
        this.recentRepos = saved.slice(0, MAX_RECENT_REPOS);
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

  get repoName(): string {
    if (!this.repoPath) return '';
    const parts = this.repoPath.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1] || '';
  }

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

  async refreshStatus() {
    if (!this.repoPath) return;
    try {
      const status = await gitStatus(this.repoPath);
      this.stagedFiles = status.filter((f) => f.staging === 'Staged');
      this.unstagedFiles = status.filter((f) => f.staging === 'Unstaged');
    } catch (e: unknown) {
      this.addToast(String(e), 'error');
    }
  }

  async refreshAll() {
    if (!this.repoPath) return;
    try {
      const [status, commits, branches] = await Promise.all([
        gitStatus(this.repoPath),
        gitLog(this.repoPath),
        gitBranches(this.repoPath),
      ]);
      this.stagedFiles = status.filter((f) => f.staging === 'Staged');
      this.unstagedFiles = status.filter((f) => f.staging === 'Unstaged');
      this.commits = commits;
      this.branches = branches;
      this.currentBranch = branches.find((b) => b.is_current)?.name || this.currentBranch;
    } catch (e: unknown) {
      this.addToast(String(e), 'error');
    }
  }

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
