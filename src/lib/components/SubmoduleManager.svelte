<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { extractErrorMessage } from '$lib/utils/error';
  import { gitUpdateSubmodule, gitAddSubmodule, gitGetSubmodules } from '$lib/git/commands';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';
  import type { SubmoduleInfo } from '$lib/git/types';

  // ── State ──────────────────────────────────────────────
  let searchQuery = $state('');
  let collapsed = $state(false);
  let loading = $state(false);
  let contextMenu = $state<{ submodule: SubmoduleInfo; x: number; y: number } | null>(null);

  // ── Derived ────────────────────────────────────────────
  const filteredSubmodules = $derived(
    app.submodules.filter((s) => 
      !searchQuery || 
      s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      s.path.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  // ── Operations ──────────────────────────────────────────
  async function refreshSubmodules() {
    if (!app.repoPath) return;
    loading = true;
    try {
      app.submodules = await gitGetSubmodules(app.repoPath);
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      loading = false;
    }
  }

  async function handleUpdate(submodule: SubmoduleInfo, init = true, recursive = true) {
    if (!app.repoPath) return;
    loading = true;
    try {
      await gitUpdateSubmodule(app.repoPath, submodule.name, init, recursive);
      app.addToast(`已更新 submodule: ${submodule.name}`, 'success');
      await refreshSubmodules();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      loading = false;
    }
  }

  async function handleAdd() {
    if (!app.repoPath) return;
    const url = prompt('Submodule 遠端 URL:');
    if (!url) return;
    const path = prompt('本地路徑 (相對路徑):');
    if (!path) return;

    loading = true;
    try {
      await gitAddSubmodule(app.repoPath, url, path);
      app.addToast(`已新增 submodule: ${path}`, 'success');
      await refreshSubmodules();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      loading = false;
    }
  }

  // ── Context Menu ───────────────────────────────────────
  function handleContextMenu(e: MouseEvent, submodule: SubmoduleInfo) {
    e.preventDefault();
    e.stopPropagation();
    contextMenu = { submodule, x: e.clientX, y: e.clientY };
  }

  const menuItems: MenuItem[] = [
    { id: 'update', label: '更新 (Update)' },
    { id: 'init_update', label: '初始化並更新 (Init + Update)' },
    { id: 'recursive_update', label: '遞迴更新 (Recursive)' },
    { id: '_sep1', label: '', separator: true },
    { id: 'copy_url', label: '複製 URL' },
    { id: 'open_repo', label: '開啟此倉庫' },
  ];

  async function handleMenuSelect(actionId: string) {
    if (!contextMenu || !app.repoPath) return;
    const { submodule } = contextMenu;
    contextMenu = null;

    switch (actionId) {
      case 'update':
        await handleUpdate(submodule, false, false);
        break;
      case 'init_update':
        await handleUpdate(submodule, true, false);
        break;
      case 'recursive_update':
        await handleUpdate(submodule, true, true);
        break;
      case 'copy_url':
        await navigator.clipboard.writeText(submodule.url);
        app.addToast('已複製 URL', 'success');
        break;
      case 'open_repo':
        app.openRepo(`${app.repoPath}/${submodule.path}`);
        break;
    }
  }

  function getStatusColor(status: string) {
    switch (status) {
      case 'UpToDate': return 'var(--success-color, #4caf50)';
      case 'Outdated': return 'var(--warning-color, #ff9800)';
      case 'Uninitialized': return 'var(--text-muted, #888)';
      default: return 'inherit';
    }
  }
</script>

<div class="submodule-manager">
  <div class="section-header-row">
    <button class="section-toggle" onclick={() => (collapsed = !collapsed)}>
      <span class="toggle-icon">{collapsed ? '▸' : '▾'}</span>
      <span class="section-label">SUBMODULES</span>
      <span class="section-count">{app.submodules.length}</span>
    </button>
    <div class="section-actions">
      <button class="section-action-btn" title="重新整理" onclick={refreshSubmodules} disabled={loading}>
        <span class:spinning={loading}>↻</span>
      </button>
      <button class="section-action-btn" title="新增 Submodule" onclick={handleAdd} disabled={loading}>+</button>
    </div>
  </div>

  {#if !collapsed}
    <div class="submodule-list">
      {#if app.submodules.length === 0}
        <div class="empty-state">尚無 Submodules</div>
      {:else}
        {#each filteredSubmodules as sub (sub.path)}
          <button
            class="submodule-item"
            oncontextmenu={(e) => handleContextMenu(e, sub)}
            title="{sub.name}\nURL: {sub.url}\nStatus: {sub.status}"
          >
            <span class="sub-icon">📦</span>
            <span class="sub-name">{sub.name}</span>
            <span class="sub-status" style:color={getStatusColor(sub.status)}>●</span>
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
  .submodule-manager {
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

  .submodule-list {
    display: flex;
    flex-direction: column;
    padding: 0 4px 8px 4px;
  }

  .submodule-item {
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

  .submodule-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .sub-icon {
    font-size: var(--text-sm);
    opacity: 0.7;
  }

  .sub-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sub-status {
    font-size: 10px;
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
