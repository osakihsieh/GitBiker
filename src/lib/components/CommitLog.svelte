<script lang="ts">
  import { app } from '$lib/stores/app.svelte';

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

  function firstLine(message: string): string {
    return message.split('\n')[0] || '';
  }
</script>

<div class="history-panel">
  <div class="history-header">Commit History</div>
  <div class="history-list">
    {#each app.commits as commit, i (commit.id)}
      <div class="commit-item">
        <div class="commit-graph">
          <div class="commit-dot"></div>
          {#if i < app.commits.length - 1}
            <div class="commit-line"></div>
          {/if}
        </div>
        <div class="commit-info">
          {#if i === 0 && app.currentBranch}
            <div class="commit-tags">
              <span class="branch-tag">{app.currentBranch}</span>
            </div>
          {/if}
          <div class="commit-msg" title={commit.message}>{firstLine(commit.message)}</div>
          <div class="commit-meta">
            <span class="commit-hash">{shortHash(commit.id)}</span>
            <span>{commit.author}</span>
            <span>{timeAgo(commit.timestamp)}</span>
          </div>
        </div>
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-icon">◯</div>
        <div>No commits yet</div>
      </div>
    {/each}
  </div>
</div>

<style>
  .history-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .history-header {
    padding: var(--space-sm) var(--space-md);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .history-list {
    overflow-y: auto;
    flex: 1;
  }
  .commit-item {
    display: flex;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }
  .commit-graph {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 20px;
    flex-shrink: 0;
  }
  .commit-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    flex-shrink: 0;
    margin-top: 4px;
    z-index: 1;
  }
  .commit-line {
    width: 2px;
    flex: 1;
    background: var(--accent);
    opacity: 0.3;
  }
  .commit-info { flex: 1; min-width: 0; }
  .commit-tags {
    display: flex;
    gap: var(--space-xs);
    margin-bottom: 2px;
  }
  .branch-tag {
    font-size: 10px;
    padding: 1px 4px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: var(--bg-primary);
    font-weight: 600;
  }
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
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: var(--space-lg);
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    height: 100%;
  }
  .empty-icon { font-size: 24px; opacity: 0.3; }
</style>
