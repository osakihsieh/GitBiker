<script lang="ts">
  import { extractErrorMessage } from '$lib/utils/error';
  import { app } from '$lib/stores/app.svelte';
  import { multiRepo, type RepoInfo } from '$lib/stores/multiRepoStore.svelte';
  import { openInEditor } from '$lib/git/commands';

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();
  let searchQuery = $state('');
  let searchInput = $state<HTMLInputElement | null>(null);
  let selectedIndex = $state(-1);
  let bulkDropdownOpen = $state(false);

  // Auto-focus search on open
  $effect(() => {
    if (open) {
      searchQuery = '';
      selectedIndex = -1;
      setTimeout(() => searchInput?.focus(), 50);
    }
  });

  const filteredRepos = $derived(multiRepo.filterRepos(searchQuery));

  // Smart grouping
  const displayGroups = $derived.by(() => {
    const repos = filteredRepos;
    if (!multiRepo.useGrouping) {
      return [{ scanPath: '', repos }];
    }
    const groups = new Map<string, RepoInfo[]>();
    for (const repo of repos) {
      const group = groups.get(repo.scanPath) ?? [];
      group.push(repo);
      groups.set(repo.scanPath, group);
    }
    return Array.from(groups.entries()).map(([scanPath, repos]) => ({ scanPath, repos }));
  });

  function dirName(path: string): string {
    const parts = path.replace(/\\/g, '/').split('/');
    return parts.slice(-2).join('/') || path;
  }

  function handleRepoClick(repo: RepoInfo) {
    app.openRepo(repo.path);
    onClose();
  }

  async function handleBulkFetch() {
    const result = await multiRepo.bulkFetch();
    app.addToast(
      `已 fetch ${result.ok} 個 repos${result.fail > 0 ? `，${result.fail} 失敗` : ''}`,
      result.fail > 0 ? 'error' : 'success',
    );
    bulkDropdownOpen = false;
  }

  async function handleBulkPull() {
    const result = await multiRepo.bulkPull();
    app.addToast(
      `Pull 完成：${result.ok} 成功${result.fail > 0 ? `，${result.fail} 失敗` : ''}`,
      result.fail > 0 ? 'error' : 'success',
    );
    bulkDropdownOpen = false;
  }

  async function handleBulkPush() {
    const pushable = multiRepo.pushableCount;
    if (pushable === 0) {
      app.addToast('沒有需要 push 的 repo', 'info');
      bulkDropdownOpen = false;
      return;
    }
    const result = await multiRepo.bulkPush();
    app.addToast(
      `Push 完成：${result.ok} 成功${result.fail > 0 ? `，${result.fail} 失敗` : ''}`,
      result.fail > 0 ? 'error' : 'success',
    );
    bulkDropdownOpen = false;
  }

  async function handleRepoFetch(e: Event, repo: RepoInfo) {
    e.stopPropagation();
    await multiRepo.repoFetch(repo.path);
  }

  async function handleRepoPull(e: Event, repo: RepoInfo) {
    e.stopPropagation();
    await multiRepo.repoPull(repo.path);
  }

  async function handleRepoPush(e: Event, repo: RepoInfo) {
    e.stopPropagation();
    await multiRepo.repoPush(repo.path);
  }

  async function handleRepoEditor(e: Event, repo: RepoInfo) {
    e.stopPropagation();
    try {
      await openInEditor(repo.path, app.preferredEditor ?? undefined);
    } catch (err: unknown) {
      app.addToast(String(err), 'error');
    }
  }

  async function handleAddDirectory() {
    try {
      const { open: openDialog } = await import('@tauri-apps/plugin-dialog');
      const selected = await openDialog({
        directory: true,
        title: '選擇包含 Git Repos 的資料夾',
      });
      if (selected) {
        await multiRepo.addScanPath(selected, app.repoPath);
        if (multiRepo.repos.filter((r) => r.scanPath === selected).length === 0) {
          app.addToast('此目錄中沒有找到 Git repositories', 'info');
        }
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  function handleRemoveScanPath(e: Event, scanPath: string) {
    e.stopPropagation();
    multiRepo.removeScanPath(scanPath);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;
    const allRepos = filteredRepos;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, allRepos.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, -1);
    } else if (e.key === 'Enter' && selectedIndex >= 0 && selectedIndex < allRepos.length) {
      e.preventDefault();
      handleRepoClick(allRepos[selectedIndex]);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="popover-overlay" onclick={onClose} onkeydown={handleKeydown}></div>
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="multi-repo-popover" role="dialog" aria-label="Multi-repo manager" onkeydown={handleKeydown}>
    <div class="popover-header">
      <input
        bind:this={searchInput}
        bind:value={searchQuery}
        class="search-input"
        type="text"
        placeholder="搜尋 repos..."
        aria-label="Search repositories"
      />
      <div class="bulk-wrapper">
        <button
          class="bulk-btn"
          onclick={() => (bulkDropdownOpen = !bulkDropdownOpen)}
          disabled={multiRepo.bulkRunning || multiRepo.repos.length === 0}
        >
          {#if multiRepo.bulkRunning}
            <span class="spinner"></span> {multiRepo.bulkAction}
          {:else}
            批量操作 ▾
          {/if}
        </button>
        {#if bulkDropdownOpen}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="bulk-overlay" onclick={() => (bulkDropdownOpen = false)}></div>
          <div class="bulk-dropdown">
            <button class="bulk-item" onclick={handleBulkFetch} disabled={multiRepo.bulkRunning}>
              ↻ Fetch All
            </button>
            <button class="bulk-item" onclick={handleBulkPull} disabled={multiRepo.bulkRunning}>
              ↓ Pull All
            </button>
            <button class="bulk-item" onclick={handleBulkPush} disabled={multiRepo.bulkRunning}>
              ↑ Push All
            </button>
          </div>
        {/if}
      </div>
    </div>

    <div class="repo-list" role="listbox" aria-label="Repositories">
      {#if multiRepo.loading && multiRepo.repos.length === 0}
        {#each Array(3) as _}
          <div class="skeleton-row">
            <div class="skeleton-dot"></div>
            <div class="skeleton-text"></div>
          </div>
        {/each}
      {:else if !multiRepo.hasScanPaths}
        <div class="empty-state">
          <div class="empty-icon">📂</div>
          <div class="empty-text">新增一個目錄來開始管理多個 repos</div>
        </div>
      {:else if filteredRepos.length === 0 && searchQuery}
        <div class="empty-state">
          <div class="empty-text">找不到符合 "{searchQuery}" 的 repo</div>
        </div>
      {:else}
        {#each displayGroups as group}
          {#if multiRepo.useGrouping && group.scanPath}
            <div class="group-header">
              <span class="group-path">{dirName(group.scanPath)} ({group.repos.length})</span>
              <button class="group-remove" onclick={(e) => handleRemoveScanPath(e, group.scanPath)} title="移除此目錄">
                ×
              </button>
            </div>
          {/if}
          {#each group.repos as repo, i}
            {@const flatIdx = filteredRepos.indexOf(repo)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <div
              class="repo-row"
              class:selected={flatIdx === selectedIndex}
              class:has-error={!!repo.error}
              onclick={() => handleRepoClick(repo)}
              role="option"
              aria-selected={flatIdx === selectedIndex}
            >
              <span class="dirty-dot" class:dirty={repo.dirty > 0} class:error={!!repo.error}></span>
              <span class="repo-name">{repo.name}</span>
              <span class="repo-branch">({repo.branch})</span>
              <span class="repo-stats">
                {#if repo.error}
                  <span class="stat-error">?</span>
                {:else if repo.dirty > 0}
                  <span class="stat-dirty">{repo.dirty}▲</span>
                {:else}
                  <span class="stat-clean">clean</span>
                {/if}
                {#if repo.ahead > 0}
                  <span class="stat-ahead">{repo.ahead}↑</span>
                {/if}
                {#if repo.behind > 0}
                  <span class="stat-behind">{repo.behind}↓</span>
                {/if}
              </span>
              <span class="repo-actions">
                {#if repo.loading}
                  <span class="spinner-sm"></span>
                {:else}
                  <button class="action-icon" onclick={(e) => handleRepoFetch(e, repo)} title="Fetch">↻</button>
                  <button class="action-icon" onclick={(e) => handleRepoPull(e, repo)} title="Pull">↓</button>
                  <button class="action-icon" onclick={(e) => handleRepoPush(e, repo)} title="Push">↑</button>
                  <button class="action-icon" onclick={(e) => handleRepoEditor(e, repo)} title="Open in Editor">📝</button>
                {/if}
              </span>
            </div>
          {/each}
        {/each}
      {/if}
    </div>

    <div class="popover-footer">
      <button class="add-dir-btn" onclick={handleAddDirectory}>
        + 新增目錄
      </button>
    </div>
  </div>
{/if}

<style>
  .popover-overlay {
    position: fixed;
    inset: 0;
    z-index: 99;
  }
  .multi-repo-popover {
    position: fixed;
    top: 44px;
    left: 60px;
    min-width: 340px;
    max-width: 500px;
    width: auto;
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 100;
    animation: popover-in 0.15s ease-out;
  }
  @keyframes popover-in {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .popover-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--font-ui);
    outline: none;
    padding: var(--space-xs) 0;
  }
  .search-input::placeholder { color: var(--text-muted); }

  .bulk-wrapper { position: relative; }
  .bulk-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 11px;
    font-family: var(--font-ui);
    padding: 2px 8px;
    cursor: pointer;
    white-space: nowrap;
  }
  .bulk-btn:hover { border-color: var(--accent); color: var(--accent); }
  .bulk-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .bulk-overlay { position: fixed; inset: 0; z-index: 101; }
  .bulk-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 2px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 102;
    min-width: 120px;
  }
  .bulk-item {
    display: block;
    width: 100%;
    padding: 6px 12px;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-ui);
    cursor: pointer;
    text-align: left;
  }
  .bulk-item:hover { background: var(--bg-hover); }
  .bulk-item:disabled { opacity: 0.5; cursor: not-allowed; }

  .repo-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-xs) 0;
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-md);
    margin-top: var(--space-xs);
  }
  .group-path {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    flex: 1;
  }
  .group-remove {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 14px;
    padding: 0 4px;
    opacity: 0;
    transition: opacity 0.1s;
  }
  .group-header:hover .group-remove { opacity: 1; }
  .group-remove:hover { color: var(--error); }

  .repo-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-md);
    background: none;
    border: none;
    color: var(--text-primary);
    font-family: var(--font-ui);
    cursor: pointer;
    width: 100%;
    text-align: left;
    height: 32px;
  }
  .repo-row:hover { background: var(--bg-hover); }
  .repo-row.selected { background: var(--bg-hover); }
  .repo-row.has-error { opacity: 0.7; }

  .dirty-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
    background: transparent;
  }
  .dirty-dot.dirty { background: var(--warning); }
  .dirty-dot.error { background: var(--error); }

  .repo-name {
    font-size: 12px;
    font-family: var(--font-mono);
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 140px;
  }
  .repo-branch {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
  }
  .repo-stats {
    display: flex;
    gap: var(--space-xs);
    margin-left: auto;
    flex-shrink: 0;
    font-size: 10px;
    font-family: var(--font-mono);
  }
  .stat-dirty { color: var(--warning); }
  .stat-clean { color: var(--success); opacity: 0.6; }
  .stat-ahead { color: var(--diff-add-text); }
  .stat-behind { color: var(--diff-del-text); }
  .stat-error { color: var(--error); }

  .repo-actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.1s;
  }
  .repo-row:hover .repo-actions { opacity: 1; }
  .action-icon {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 12px;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
  }
  .action-icon:hover { color: var(--accent); background: var(--bg-hover); }

  .popover-footer {
    padding: var(--space-sm) var(--space-md);
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }
  .add-dir-btn {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: 12px;
    font-family: var(--font-ui);
    padding: var(--space-xs) 0;
  }
  .add-dir-btn:hover { text-decoration: underline; }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-lg) var(--space-md);
    color: var(--text-muted);
  }
  .empty-icon { font-size: 24px; }
  .empty-text { font-size: 12px; text-align: center; }

  .skeleton-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    height: 32px;
  }
  .skeleton-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--bg-hover);
    animation: pulse 1.2s ease-in-out infinite;
  }
  .skeleton-text {
    height: 10px;
    width: 60%;
    border-radius: 2px;
    background: var(--bg-hover);
    animation: pulse 1.2s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 0.8; }
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

  @media (max-width: 900px) {
    .multi-repo-popover {
      left: 12px;
      right: 12px;
      max-width: none;
      width: auto;
    }
  }
</style>
