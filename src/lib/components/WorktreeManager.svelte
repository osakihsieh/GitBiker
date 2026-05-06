<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { extractErrorMessage } from '$lib/utils/error';
  import { gitGetWorktrees, gitAddWorktree, gitRemoveWorktree, openInFolder } from '$lib/git/commands';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';
  import type { WorktreeInfo } from '$lib/git/types';

  // ── State ──────────────────────────────────────────────
  let searchQuery = $state('');
  let collapsed = $state(false);
  let loading = $state(false);
  let contextMenu = $state<{ worktree: WorktreeInfo; x: number; y: number } | null>(null);

  // ── Derived ────────────────────────────────────────────
  const filteredWorktrees = $derived(
    app.worktrees.filter((w) => 
      !searchQuery || 
      w.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      w.branch?.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  // ── Operations ──────────────────────────────────────────
  async function refreshWorktrees() {
    if (!app.repoPath) return;
    loading = true;
    try {
      app.worktrees = await gitGetWorktrees(app.repoPath);
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      loading = false;
    }
  }

  async function handleAdd() {
    if (!app.repoPath) return;
    const branch = prompt('分支名稱 (現有或新分支):');
    if (!branch) return;
    const path = prompt('本地路徑 (全路徑或相對於主倉庫):');
    if (!path) return;

    loading = true;
    try {
      await gitAddWorktree(app.repoPath, path, branch);
      app.addToast(`已新增 worktree: ${path}`, 'success');
      await refreshWorktrees();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      loading = false;
    }
  }

  async function handleRemove(worktree: WorktreeInfo, force = false) {
    if (!app.repoPath) return;
    if (!force && !confirm(`確定要移除 worktree「${worktree.name}」？`)) return;

    loading = true;
    try {
      await gitRemoveWorktree(app.repoPath, worktree.name, force);
      app.addToast(`已移除 worktree: ${worktree.name}`, 'success');
      await refreshWorktrees();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      loading = false;
    }
  }

  // ── Context Menu ───────────────────────────────────────
  function handleContextMenu(e: MouseEvent, worktree: WorktreeInfo) {
    e.preventDefault();
    e.stopPropagation();
    contextMenu = { worktree, x: e.clientX, y: e.clientY };
  }

  const menuItems: MenuItem[] = [
    { id: 'open_folder', label: '在檔案總管開啟' },
    { id: 'open_repo', label: '在 GitBiker 開啟' },
    { id: '_sep1', label: '', separator: true },
    { id: 'remove', label: '移除 (Remove)' },
    { id: 'force_remove', label: '強制移除 (Force Remove)' },
  ];

  async function handleMenuSelect(actionId: string) {
    if (!contextMenu || !app.repoPath) return;
    const { worktree } = contextMenu;
    contextMenu = null;

    switch (actionId) {
      case 'open_folder':
        await openInFolder(worktree.path);
        break;
      case 'open_repo':
        app.openRepo(worktree.path);
        break;
      case 'remove':
        await handleRemove(worktree, false);
        break;
      case 'force_remove':
        await handleRemove(worktree, true);
        break;
    }
  }
</script>

<div class="worktree-manager">
  <div class="section-header-row">
    <button class="section-toggle" onclick={() => (collapsed = !collapsed)}>
      <span class="toggle-icon">{collapsed ? '▸' : '▾'}</span>
      <span class="section-label">WORKTREES</span>
      <span class="section-count">{app.worktrees.length}</span>
    </button>
    <div class="section-actions">
      <button class="section-action-btn" title="重新整理" onclick={refreshWorktrees} disabled={loading}>
        <span class:spinning={loading}>↻</span>
      </button>
      <button class="section-action-btn" title="新增 Worktree" onclick={handleAdd} disabled={loading}>+</button>
    </div>
  </div>

  {#if !collapsed}
    <div class="worktree-list">
      {#if app.worktrees.length === 0}
        <div class="empty-state">尚無 Worktrees</div>
      {:else}
        {#each filteredWorktrees as wt (wt.path)}
          <button
            class="worktree-item"
            oncontextmenu={(e) => handleContextMenu(e, wt)}
            title="{wt.name}\nBranch: {wt.branch}\nPath: {wt.path}"
          >
            <span class="wt-icon">🌿</span>
            <div class="wt-info">
              <span class="wt-name">{wt.name}</span>
              <span class="wt-branch">{wt.branch || '(no branch)'}</span>
            </div>
            {#if wt.is_locked}
              <span class="wt-lock" title={wt.lock_reason || 'Locked'}>🔒</span>
            {/if}
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={menuItems}
    onselect={handleMenuSelect}
    onclose={() => (contextMenu = null)}
  />
{/if}

<style>
  .worktree-manager {
    display: flex;
    flex-direction: column;
  }

  .section-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-right: 8px;
  }

  .section-toggle {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    background: none;
    border: none;
    color: var(--text-muted, #aaa);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    cursor: pointer;
    text-align: left;
  }

  .section-toggle:hover {
    color: var(--text-color, #eee);
  }

  .toggle-icon {
    width: 10px;
    font-size: 10px;
  }

  .section-count {
    background: rgba(255, 255, 255, 0.1);
    padding: 1px 6px;
    border-radius: 10px;
    font-size: 10px;
  }

  .section-actions {
    display: flex;
    gap: 4px;
  }

  .section-action-btn {
    background: none;
    border: none;
    color: var(--text-muted, #aaa);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    font-size: var(--text-lg);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
  }

  .section-action-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-color, #eee);
  }

  .section-action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .worktree-list {
    display: flex;
    flex-direction: column;
    padding: 0 4px 8px 4px;
  }

  .worktree-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px 6px 28px;
    background: none;
    border: none;
    color: var(--text-color, #ccc);
    font-size: var(--text-md);
    cursor: pointer;
    text-align: left;
    border-radius: 4px;
    width: 100%;
  }

  .worktree-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .wt-icon {
    font-size: var(--text-sm);
    opacity: 0.7;
  }

  .wt-info {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .wt-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .wt-branch {
    font-size: 10px;
    color: var(--text-muted, #888);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .wt-lock {
    font-size: 10px;
    opacity: 0.6;
  }

  .empty-state {
    padding: 8px 32px;
    color: var(--text-muted, #666);
    font-size: var(--text-sm);
    font-style: italic;
  }

  .spinning {
    display: inline-block;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
