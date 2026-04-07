<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitLogSearch, gitTagCreate, gitRevert, gitResetSoft, gitResetHard, gitCherryPick } from '$lib/git/commands';
  import type { Commit } from '$lib/git/types';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';

  let searchQuery = $state('');
  let searchType = $state<'message' | 'author' | 'code'>('message');
  let searchResults = $state<Commit[] | null>(null);
  let searching = $state(false);
  let authorFilter = $state('');
  let contextMenu = $state<{ commit: Commit; x: number; y: number } | null>(null);
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  /** Unique authors from current commit list */
  const uniqueAuthors = $derived.by(() => {
    const commits = searchResults ?? app.commits;
    const authors = new Set(commits.map((c) => c.author));
    return [...authors].sort();
  });

  const displayCommits = $derived.by(() => {
    const base = searchResults ?? app.commits;
    if (!authorFilter) return base;
    return base.filter((c) => c.author === authorFilter);
  });

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

  function handleSearchInput() {
    clearTimeout(debounceTimer);
    if (!searchQuery.trim()) {
      searchResults = null;
      return;
    }
    debounceTimer = setTimeout(executeSearch, 300);
  }

  async function executeSearch() {
    if (!app.repoPath || !searchQuery.trim()) return;
    searching = true;
    try {
      searchResults = await gitLogSearch(app.repoPath, searchQuery.trim(), searchType, 200);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
      searchResults = null;
    } finally {
      searching = false;
    }
  }

  function clearSearch() {
    searchQuery = '';
    searchResults = null;
  }

  function handleCommitClick(commit: Commit) {
    // 如果點擊同一個 commit，取消選取回到 worktree
    if (app.selectedCommit?.id === commit.id) {
      app.backToWorktree();
    } else {
      app.selectCommit(commit);
    }
  }

  function handleContextMenu(e: MouseEvent, commit: Commit) {
    e.preventDefault();
    contextMenu = { commit, x: e.clientX, y: e.clientY };
  }

  function getContextMenuItems(commit: Commit): MenuItem[] {
    const items: MenuItem[] = [
      { id: 'copyHash', label: '複製 Hash' },
      { id: '_sep1', label: '', separator: true },
      { id: 'createTag', label: '建立 Tag...' },
      { id: '_sep2', label: '', separator: true },
      { id: 'revert', label: '撤回此 Commit (Revert)' },
      { id: 'cherryPick', label: 'Cherry-pick 此 Commit' },
    ];

    // 判斷是否為未推送的 commit：在 ahead 範圍內
    const currentBranch = app.branches.find((b) => b.is_current);
    const aheadCount = currentBranch?.ahead ?? 0;
    const commitIndex = displayCommits.findIndex((c) => c.id === commit.id);
    const isUnpushed = commitIndex >= 0 && commitIndex < aheadCount;

    if (isUnpushed) {
      items.push(
        { id: 'undoSoft', label: '撤銷到此 Commit (保留變更)' },
        { id: 'undoHard', label: '撤銷到此 Commit (丟棄變更)' },
      );
    }

    return items;
  }

  const contextMenuItems = $derived(
    contextMenu ? getContextMenuItems(contextMenu.commit) : [],
  );

  async function handleContextSelect(actionId: string) {
    if (!contextMenu || !app.repoPath) return;
    const { commit } = contextMenu;
    try {
      switch (actionId) {
        case 'copyHash':
          await navigator.clipboard.writeText(commit.id);
          app.addToast('已複製 commit hash', 'success');
          break;
        case 'createTag': {
          const tagName = prompt('Tag 名稱:');
          if (tagName?.trim()) {
            await gitTagCreate(app.repoPath, tagName.trim(), commit.id);
            app.addToast(`已建立 tag: ${tagName.trim()}`, 'success');
            await app.refreshAll();
          }
          break;
        }
        case 'revert': {
          const isMerge = commit.parents.length > 1;
          const label = isMerge ? '（Merge commit，將使用 -m 1）' : '';
          if (confirm(`確定要 Revert commit ${commit.id.substring(0, 7)}？${label}\n\n${commit.message}`)) {
            await gitRevert(app.repoPath, commit.id, isMerge);
            app.addToast('Revert 成功', 'success');
            await app.refreshAll();
          }
          break;
        }
        case 'undoSoft': {
          if (confirm(`確定要撤銷到 ${commit.id.substring(0, 7)}？\n變更將保留在 staged 區域。`)) {
            await gitResetSoft(app.repoPath, commit.id);
            app.addToast('已撤銷 commit（變更已保留）', 'success');
            await app.refreshAll();
          }
          break;
        }
        case 'undoHard': {
          if (confirm(`⚠️ 確定要撤銷到 ${commit.id.substring(0, 7)}？\n\n此操作不可撤銷，所有變更將被丟棄！`)) {
            await gitResetHard(app.repoPath, commit.id);
            app.addToast('已撤銷 commit（變更已丟棄）', 'success');
            await app.refreshAll();
          }
          break;
        }
        case 'cherryPick': {
          if (confirm(`確定要 Cherry-pick commit ${commit.id.substring(0, 7)}？\n\n${commit.message}`)) {
            const result = await gitCherryPick(app.repoPath, commit.id);
            if (result.success) {
              app.addToast('Cherry-pick 成功', 'success');
            } else if (result.conflicts.length > 0) {
              app.addToast(`Cherry-pick 衝突：需要解決衝突`, 'error', false);
              await app.enterConflictMode();
            }
            await app.refreshAll();
          }
          break;
        }
      }
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      clearSearch();
    }
  }

  function handleFilterChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    const val = target.value;
    if (val === 'head') {
      app.setLogFilter({ type: 'Head' });
    } else if (val === 'all') {
      app.setLogFilter({ type: 'All' });
    } else {
      app.setLogFilter({ type: 'Branch', value: val });
    }
  }

  const currentFilterValue = $derived.by(() => {
    const f = app.logFilter;
    if (f.type === 'Head') return 'head';
    if (f.type === 'All') return 'all';
    return f.value;
  });
</script>

<div class="history-panel">
  <!-- Search Bar -->
  <div class="search-bar">
    <select class="search-type-select" bind:value={searchType} onchange={executeSearch}>
      <option value="message">Msg</option>
      <option value="author">User</option>
      <option value="code">Code</option>
    </select>
    <input
      type="text"
      class="search-input"
      placeholder="Search..."
      bind:value={searchQuery}
      oninput={handleSearchInput}
      onkeydown={handleSearchKeydown}
      aria-label="Search commits"
    />
    {#if searchQuery}
      <button class="search-clear" onclick={clearSearch} aria-label="Clear search">✕</button>
    {/if}
    {#if searching}
      <span class="search-spinner"></span>
    {/if}
  </div>

  <div class="history-header">
    <span class="header-title">Commits</span>
    <select class="filter-select" value={currentFilterValue} onchange={handleFilterChange}>
      <option value="head">Current Branch</option>
      <option value="all">All Branches</option>
      <optgroup label="Local Branches">
        {#each app.branches.filter(b => !b.is_remote) as b}
          <option value={b.name}>{b.name}</option>
        {/each}
      </optgroup>
      <optgroup label="Remote Branches">
        {#each app.branches.filter(b => b.is_remote) as b}
          <option value={b.name}>{b.name}</option>
        {/each}
      </optgroup>
    </select>
    {#if uniqueAuthors.length > 1}
      <select
        class="filter-select author-filter"
        value={authorFilter}
        onchange={(e) => authorFilter = e.currentTarget.value}
      >
        <option value="">All Authors</option>
        {#each uniqueAuthors as author}
          <option value={author}>{author}</option>
        {/each}
      </select>
    {/if}
  </div>

  <!-- Commit List -->
  <div class="history-list">
    {#if searchResults !== null && displayCommits.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🔍</div>
        <div>No commits matching "{searchQuery}"</div>
      </div>
    {:else}
      {#each displayCommits as commit, i (commit.id)}
        <button
          class="commit-item"
          class:selected={app.selectedCommit?.id === commit.id}
          onclick={() => handleCommitClick(commit)}
          oncontextmenu={(e) => handleContextMenu(e, commit)}
        >
          <div class="commit-graph">
            <div class="commit-dot"></div>
            {#if i < displayCommits.length - 1}
              <div class="commit-line"></div>
            {/if}
          </div>
          <div class="commit-info">
            {#if commit.refs && commit.refs.length > 0}
              <div class="commit-tags">
                {#each commit.refs.slice(0, 3) as ref}
                  <span
                    class="ref-tag"
                    class:ref-local={ref.kind === 'Local'}
                    class:ref-remote={ref.kind === 'Remote'}
                    class:ref-tag-badge={ref.kind === 'Tag'}
                  >{ref.name}</span>
                {/each}
                {#if commit.refs.length > 3}
                  <span class="ref-overflow" title={commit.refs.slice(3).map(r => r.name).join(', ')}>+{commit.refs.length - 3}</span>
                {/if}
              </div>
            {/if}
            <div class="commit-msg" title={commit.message}>{firstLine(commit.message)}</div>
            <div class="commit-meta">
              <span class="commit-hash">{shortHash(commit.id)}</span>
              <span>{commit.author}</span>
              <span>{timeAgo(commit.timestamp)}</span>
            </div>
          </div>
        </button>
      {:else}
        <div class="empty-state">
          <div class="empty-icon">◯</div>
          <div>No commits yet</div>
        </div>
      {/each}
    {/if}
  </div>
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={contextMenuItems}
    onSelect={handleContextSelect}
    onClose={() => contextMenu = null}
  />
{/if}

<style>
  .history-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Search */
  .search-bar {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .search-type-select {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-size: 10px;
    padding: 1px 2px;
    outline: none;
    cursor: pointer;
  }
  .search-type-select:hover { color: var(--text-primary); border-color: var(--accent); }
  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-ui);
    outline: none;
    min-width: 0;
  }
  .search-input::placeholder { color: var(--text-muted); }
  .search-clear {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 10px;
    padding: 2px 4px;
  }
  .search-clear:hover { color: var(--text-primary); }
  .search-spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid var(--text-muted);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  /* Header */
  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    gap: var(--space-sm);
  }
  .header-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    white-space: nowrap;
  }
  .filter-select {
    flex: 1;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 11px;
    padding: 2px 4px;
    outline: none;
    max-width: 150px;
  }
  .filter-select:hover { border-color: var(--accent); }
  .author-filter { max-width: 120px; }

  /* List */
  .history-list {
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

  /* Ref Tags */
  .commit-tags {
    display: flex;
    gap: var(--space-xs);
    margin-bottom: 2px;
    flex-wrap: wrap;
  }
  .ref-tag {
    font-size: 10px;
    padding: 1px 4px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    font-family: var(--font-mono);
    white-space: nowrap;
  }
  .ref-local {
    background: var(--accent);
    color: var(--bg-primary);
  }
  .ref-remote {
    background: var(--bg-surface);
    border: 1px solid var(--accent);
    color: var(--accent);
  }
  .ref-tag-badge {
    background: rgba(81, 207, 102, 0.2);
    color: var(--success);
  }
  .ref-overflow {
    font-size: 10px;
    color: var(--text-muted);
    padding: 1px 4px;
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

  @keyframes spin { to { transform: rotate(360deg); } }
</style>
