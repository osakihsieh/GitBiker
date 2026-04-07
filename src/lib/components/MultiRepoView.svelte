<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitStatus, gitBranches, gitFetch, gitPull, gitPush } from '$lib/git/commands';
  import type { FileStatus, Branch } from '$lib/git/types';

  interface Props {
    repoPaths: string[];
    onClose: () => void;
  }

  let { repoPaths, onClose }: Props = $props();

  interface RepoInfo {
    path: string;
    name: string;
    branch: string;
    dirty: number;
    ahead: number;
    behind: number;
    loading: boolean;
    error: string | null;
  }

  let repos = $state<RepoInfo[]>([]);
  let bulkRunning = $state(false);
  let bulkAction = $state('');

  function repoName(path: string): string {
    const parts = path.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1] || '';
  }

  async function loadRepoInfo(path: string): Promise<RepoInfo> {
    try {
      const [status, branches] = await Promise.all([
        gitStatus(path),
        gitBranches(path),
      ]);
      const current = branches.find((b: Branch) => b.is_current);
      const dirty = status.length;
      return {
        path,
        name: repoName(path),
        branch: current?.name ?? 'unknown',
        dirty,
        ahead: current?.ahead ?? 0,
        behind: current?.behind ?? 0,
        loading: false,
        error: null,
      };
    } catch (e: unknown) {
      return {
        path,
        name: repoName(path),
        branch: '?',
        dirty: 0,
        ahead: 0,
        behind: 0,
        loading: false,
        error: String(e),
      };
    }
  }

  async function loadAll() {
    repos = repoPaths.map((p) => ({
      path: p,
      name: repoName(p),
      branch: '...',
      dirty: 0,
      ahead: 0,
      behind: 0,
      loading: true,
      error: null,
    }));

    const results = await Promise.all(repoPaths.map(loadRepoInfo));
    repos = results;
  }

  $effect(() => {
    if (repoPaths.length > 0) loadAll();
  });

  async function handleBulkFetch() {
    bulkRunning = true;
    bulkAction = 'Fetching...';
    try {
      await Promise.all(repos.map((r) => gitFetch(r.path).catch(() => {})));
      app.addToast(`已 fetch ${repos.length} 個 repos`, 'success');
      await loadAll();
    } finally {
      bulkRunning = false;
      bulkAction = '';
    }
  }

  async function handleBulkPull() {
    bulkRunning = true;
    bulkAction = 'Pulling...';
    try {
      const results = await Promise.allSettled(repos.map((r) => gitPull(r.path)));
      const ok = results.filter((r) => r.status === 'fulfilled').length;
      const fail = results.filter((r) => r.status === 'rejected').length;
      app.addToast(`Pull 完成：${ok} 成功${fail > 0 ? `，${fail} 失敗` : ''}`, fail > 0 ? 'error' : 'success');
      await loadAll();
    } finally {
      bulkRunning = false;
      bulkAction = '';
    }
  }

  async function handleBulkPush() {
    bulkRunning = true;
    bulkAction = 'Pushing...';
    try {
      // Only push repos with commits ahead
      const pushable = repos.filter((r) => r.ahead > 0);
      if (pushable.length === 0) {
        app.addToast('沒有需要 push 的 repo', 'info');
      } else {
        const results = await Promise.allSettled(pushable.map((r) => gitPush(r.path)));
        const ok = results.filter((r) => r.status === 'fulfilled').length;
        const fail = results.filter((r) => r.status === 'rejected').length;
        app.addToast(`Push 完成：${ok} 成功${fail > 0 ? `，${fail} 失敗` : ''}`, fail > 0 ? 'error' : 'success');
        await loadAll();
      }
    } finally {
      bulkRunning = false;
      bulkAction = '';
    }
  }

  function handleOpenRepo(path: string) {
    app.openRepo(path);
    onClose();
  }
</script>

<div class="multi-repo">
  <div class="multi-header">
    <button class="back-btn" onclick={onClose}>← Back</button>
    <span class="multi-title">Multi-Repo ({repos.length})</span>
    <div class="bulk-actions">
      <button class="bulk-btn" onclick={handleBulkFetch} disabled={bulkRunning}>Fetch All</button>
      <button class="bulk-btn" onclick={handleBulkPull} disabled={bulkRunning}>Pull All</button>
      <button class="bulk-btn" onclick={handleBulkPush} disabled={bulkRunning}>Push All</button>
    </div>
  </div>

  {#if bulkAction}
    <div class="bulk-status">
      <span class="spinner"></span>
      <span>{bulkAction}</span>
    </div>
  {/if}

  <div class="repo-grid">
    {#each repos as repo}
      <button class="repo-card" onclick={() => handleOpenRepo(repo.path)}>
        <div class="repo-card-header">
          <span class="repo-card-name">{repo.name}</span>
          {#if repo.loading}
            <span class="spinner-sm"></span>
          {/if}
        </div>
        {#if repo.error}
          <div class="repo-card-error">{repo.error}</div>
        {:else}
          <div class="repo-card-branch">{repo.branch}</div>
          <div class="repo-card-stats">
            {#if repo.dirty > 0}
              <span class="stat dirty">{repo.dirty} changes</span>
            {/if}
            {#if repo.ahead > 0}
              <span class="stat ahead">{repo.ahead}↑</span>
            {/if}
            {#if repo.behind > 0}
              <span class="stat behind">{repo.behind}↓</span>
            {/if}
            {#if repo.dirty === 0 && repo.ahead === 0 && repo.behind === 0}
              <span class="stat clean">clean</span>
            {/if}
          </div>
        {/if}
      </button>
    {:else}
      <div class="empty-state">No git repositories found in this folder</div>
    {/each}
  </div>
</div>

<style>
  .multi-repo {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }
  .multi-header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .back-btn {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--font-size-md);
  }
  .back-btn:hover { text-decoration: underline; }
  .multi-title {
    font-size: var(--font-size-lg);
    font-weight: 600;
    flex: 1;
  }
  .bulk-actions {
    display: flex;
    gap: var(--space-xs);
  }
  .bulk-btn {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    padding: var(--space-xs) var(--space-sm);
    cursor: pointer;
  }
  .bulk-btn:hover { border-color: var(--accent); color: var(--accent); }
  .bulk-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .bulk-status {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-lg);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .repo-grid {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: var(--space-md);
    align-content: start;
  }
  .repo-card {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 4px);
    padding: var(--space-md);
    cursor: pointer;
    text-align: left;
    font-family: var(--font-ui);
    color: var(--text-primary);
    transition: border-color 0.15s;
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .repo-card:hover { border-color: var(--accent); }
  .repo-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .repo-card-name {
    font-weight: 600;
    font-size: var(--font-size-md);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .repo-card-branch {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--accent);
  }
  .repo-card-error {
    font-size: 11px;
    color: var(--error);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .repo-card-stats {
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }
  .stat {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
  }
  .stat.dirty { background: rgba(255, 179, 71, 0.2); color: var(--warning); }
  .stat.ahead { background: rgba(46, 160, 67, 0.2); color: var(--diff-add-text); }
  .stat.behind { background: rgba(248, 81, 73, 0.2); color: var(--diff-del-text); }
  .stat.clean { background: rgba(81, 207, 102, 0.15); color: var(--success); }

  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    color: var(--text-muted);
    padding: var(--space-lg);
  }

  .spinner, .spinner-sm {
    display: inline-block;
    border: 2px solid var(--text-muted);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  .spinner { width: 12px; height: 12px; }
  .spinner-sm { width: 10px; height: 10px; border-width: 1.5px; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
