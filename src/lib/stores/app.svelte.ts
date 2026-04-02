import type { FileStatus, Commit, DiffResult, Branch } from '$lib/git/types';

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
}

export const app = new AppState();
