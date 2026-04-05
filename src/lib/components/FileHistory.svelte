<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitFileLog, gitShowFileDiff } from '$lib/git/commands';
  import type { Commit, DiffResult } from '$lib/git/types';

  let commits = $state<Commit[]>([]);
  let loading = $state(false);
  let selectedCommit = $state<Commit | null>(null);

  const filePath = $derived(app.fileHistoryTarget);

  function fileName(path: string): string {
    return path.replace(/\\/g, '/').split('/').pop() || path;
  }

  function timeAgo(timestamp: number): string {
    const seconds = Math.floor(Date.now() / 1000 - timestamp);
    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  function shortHash(id: string): string {
    return id.substring(0, 7);
  }

  async function loadHistory() {
    if (!app.repoPath || !filePath) return;
    loading = true;
    try {
      commits = await gitFileLog(app.repoPath, filePath);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
      commits = [];
    } finally {
      loading = false;
    }
  }

  async function handleCommitClick(commit: Commit) {
    if (!app.repoPath || !filePath) return;
    selectedCommit = commit;
    try {
      const diff: DiffResult = await gitShowFileDiff(app.repoPath, commit.id, filePath);
      app.currentDiff = diff;
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  function handleBack() {
    app.backToWorktree();
  }

  // Load history when file path changes
  $effect(() => {
    if (filePath) {
      loadHistory();
    }
  });
</script>

<div class="file-history">
  <div class="header">
    <button class="back-btn" onclick={handleBack} title="返回">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M19 12H5M12 19l-7-7 7-7"/>
      </svg>
    </button>
    <div class="title">
      <span class="label">File History</span>
      <span class="file-name" title={filePath}>{fileName(filePath ?? '')}</span>
    </div>
  </div>

  {#if loading}
    <div class="loading">載入中...</div>
  {:else if commits.length === 0}
    <div class="empty">此檔案沒有 commit 紀錄</div>
  {:else}
    <div class="commit-list">
      {#each commits as commit (commit.id)}
        <button
          class="commit-item"
          class:selected={selectedCommit?.id === commit.id}
          onclick={() => handleCommitClick(commit)}
        >
          <div class="commit-graph">
            <div class="commit-dot"></div>
          </div>
          <div class="commit-info">
            <div class="commit-msg" title={commit.message}>
              {commit.message.split('\n')[0]}
            </div>
            <div class="commit-meta">
              <span class="commit-hash">{shortHash(commit.id)}</span>
              <span>{commit.author}</span>
              <span>{timeAgo(commit.timestamp)}</span>
            </div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .file-history {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .back-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
  }
  .back-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .title {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }
  .file-name {
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    color: var(--accent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .loading, .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-lg);
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    flex: 1;
  }
  .commit-list {
    overflow-y: auto;
    flex: 1;
  }
  .commit-item {
    display: flex;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
    background: none;
    border-left: 2px solid transparent;
    border-right: none;
    border-top: none;
    width: 100%;
    text-align: left;
    cursor: pointer;
    color: var(--text-primary);
    font-family: var(--font-ui);
  }
  .commit-item:hover { background: var(--bg-hover); }
  .commit-item.selected {
    background: var(--bg-surface);
    border-left-color: var(--accent);
  }
  .commit-graph {
    display: flex;
    align-items: flex-start;
    width: 20px;
    flex-shrink: 0;
    padding-top: 4px;
  }
  .commit-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
  }
  .commit-info { flex: 1; min-width: 0; }
  .commit-msg {
    font-size: var(--font-size-sm);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.4;
  }
  .commit-meta {
    font-size: 11px;
    color: var(--text-muted);
    display: flex;
    gap: var(--space-sm);
    margin-top: 2px;
  }
  .commit-hash { font-family: var(--font-mono); font-size: 10px; }
</style>
