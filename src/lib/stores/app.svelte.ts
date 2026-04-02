import type { FileStatus, Commit, DiffResult, Branch } from '$lib/git/types';
import { gitStatus, gitLog, gitBranches } from '$lib/git/commands';

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

  theme = $state<'dark' | 'light'>('dark');

  toggleTheme() {
    this.theme = this.theme === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', this.theme);
  }
}

export const app = new AppState();
