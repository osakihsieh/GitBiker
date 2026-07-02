<script lang="ts">
  import { app, type GitHubItem } from '$lib/stores/app.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';

  let activeTab = $state<'prs' | 'issues'>('prs');

  const items = $derived(activeTab === 'prs' ? app.prs : app.issues);

  async function handleItemClick(item: GitHubItem) {
    if (item.url) {
      await openUrl(item.url);
    }
  }

  function formatTime(dateStr?: string) {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return (
      date.toLocaleDateString() +
      ' ' +
      date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    );
  }
</script>

<div class="github-dashboard">
  <!-- Header Tabs -->
  <div class="dashboard-tabs">
    <button
      class="tab-btn"
      class:tab-active={activeTab === 'prs'}
      onclick={() => (activeTab = 'prs')}
    >
      Pull Requests ({app.prs.length})
    </button>
    <button
      class="tab-btn"
      class:tab-active={activeTab === 'issues'}
      onclick={() => (activeTab = 'issues')}
    >
      Issues ({app.issues.length})
    </button>

    {#if app.isLoadingRemote}
      <div class="loading-badge">
        <span class="loading-dot"></span>
        同步中...
      </div>
    {/if}
  </div>

  <!-- List Content -->
  <div class="dashboard-list">
    {#each items as item}
      <button class="list-item" onclick={() => handleItemClick(item)}>
        <div class="item-header">
          <span class="item-title">
            <span class="item-number">#{item.number}</span>
            {item.title}
          </span>
          <span
            class="item-state"
            class:state-open={item.state === 'OPEN'}
            class:state-closed={item.state !== 'OPEN'}
          >
            {item.state}
          </span>
        </div>
        <div class="item-meta">
          <span>👤 {item.author?.login || 'unknown'}</span>
          <span>🕒 {formatTime(item.updatedAt)}</span>
        </div>
      </button>
    {/each}

    {#if items.length === 0 && !app.isLoadingRemote}
      <div class="empty-state">
        <span class="empty-icon">📭</span>
        <p>目前沒有開放的 {activeTab === 'prs' ? 'Pull Request' : 'Issue'}</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .github-dashboard {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    overflow: hidden;
  }
  .dashboard-tabs {
    display: flex;
    align-items: center;
    height: 40px;
    padding: 0 var(--space-md);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    gap: var(--space-md);
  }
  .tab-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: var(--text-sm);
    font-weight: 500;
    cursor: pointer;
    padding: 0;
    height: 100%;
    border-bottom: 2px solid transparent;
    transition:
      color 0.15s,
      border-color 0.15s;
  }
  .tab-btn:hover {
    color: var(--text-primary);
  }
  .tab-active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }
  .loading-badge {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    font-size: 10px;
    color: var(--text-muted);
    font-style: italic;
  }
  .loading-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 1.5s infinite;
  }
  .dashboard-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-sm);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .list-item {
    display: flex;
    flex-direction: column;
    padding: var(--space-sm);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    width: 100%;
    background: none;
    border: none;
    transition: background 0.15s;
  }
  .list-item:hover {
    background: var(--bg-hover);
  }
  .item-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-sm);
  }
  .item-title {
    font-size: var(--text-sm);
    color: var(--text-primary);
    font-weight: 500;
  }
  .list-item:hover .item-title {
    color: var(--accent);
  }
  .item-number {
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
  .item-state {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 20px;
    font-weight: 600;
    flex-shrink: 0;
  }
  .state-open {
    color: var(--success);
    background: var(--success-light);
  }
  .state-closed {
    color: var(--text-muted);
    background: var(--bg-hover);
  }
  .item-meta {
    margin-top: 4px;
    display: flex;
    align-items: center;
    gap: var(--space-md);
    font-size: 10px;
    color: var(--text-muted);
  }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 0;
    color: var(--text-muted);
  }
  .empty-icon {
    font-size: 32px;
    margin-bottom: var(--space-sm);
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }
</style>
