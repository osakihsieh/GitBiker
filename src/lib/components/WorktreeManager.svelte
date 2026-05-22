<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { agentStore } from '$lib/stores/agentStore.svelte';
  import { extractErrorMessage } from '$lib/utils/error';
  import { gitGetWorktrees, gitAddWorktree, gitRemoveWorktree, openInFolder } from '$lib/git/commands';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';
  import type { WorktreeInfo } from '$lib/git/types';
  import { HardDrive, Plus, RefreshCw, Lock, Trash2, FolderOpen, ExternalLink, Activity } from 'lucide-react';

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

  function getAgentForWorktree(path: string) {
    return agentStore.statuses.find(a => a.worktree === path || a.worktree?.startsWith(path));
  }

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

<div class="worktree-section flex flex-col">
  <div class="px-4 py-2 flex items-center justify-between group/header">
    <button 
      class="flex items-center gap-1.5 text-[10.5px] font-bold text-ink-35 uppercase tracking-[0.5px] hover:text-ink transition-colors"
      onclick={() => (collapsed = !collapsed)}
    >
      <span class="w-3 text-center transition-transform duration-200" class:rotate-[-90deg]={collapsed}>▾</span>
      WORKTREES
      <span class="ml-1 px-1.5 py-0.5 rounded-full bg-ink-05 text-ink-35 text-[9px]">{app.worktrees.length}</span>
    </button>
    <div class="flex items-center gap-1 opacity-0 group-hover/header:opacity-100 transition-opacity">
      <button class="p-1 rounded hover:bg-ink-05 text-ink-35 hover:text-ink transition-colors" onclick={refreshWorktrees} disabled={loading}>
        <RefreshCw size={12} strokeWidth={2} class={loading ? 'animate-spin' : ''} />
      </button>
      <button class="p-1 rounded hover:bg-ink-05 text-ink-35 hover:text-ink transition-colors" onclick={handleAdd}>
        <Plus size={12} strokeWidth={2} />
      </button>
    </div>
  </div>

  {#if !collapsed}
    <div class="flex flex-col pb-4">
      {#if app.worktrees.length === 0}
        <div class="px-8 py-3 text-[12px] text-ink-35 italic">No worktrees found</div>
      {:else}
        {#each filteredWorktrees as wt (wt.path)}
          {@const agent = getAgentForWorktree(wt.path)}
          <button
            class="flex flex-col px-8 py-2.5 hover:bg-ink-05 transition-colors text-left relative group border-l-2 border-transparent hover:border-accent/30"
            oncontextmenu={(e) => handleContextMenu(e, wt)}
            onclick={() => app.openRepo(wt.path)}
          >
            <div class="flex items-center justify-between gap-2">
              <div class="flex items-center gap-2 overflow-hidden">
                <HardDrive size={13} strokeWidth={1.5} class="text-ink-50 shrink-0" />
                <span class="text-[13.5px] font-medium text-ink truncate tracking-tight">{wt.name}</span>
              </div>
              {#if agent}
                <div class="flex items-center gap-1 px-1.5 py-0.5 rounded-full bg-accent-bg text-accent text-[9px] font-bold animate-pulse">
                  <Activity size={8} strokeWidth={3} />
                  AGENT
                </div>
              {/if}
            </div>
            
            <div class="flex items-center gap-1.5 mt-1">
              <span class="text-[11px] font-mono text-ink-50 flex items-center gap-1">
                <span class="text-[10px] opacity-40">⑇</span> {wt.branch || '(no branch)'}
              </span>
              {#if wt.is_locked}
                <Lock size={10} class="text-warn opacity-60" />
              {/if}
            </div>

            {#if agent}
              <div class="mt-2 p-2 rounded-lg bg-bg-deep/50 border border-accent/10 flex flex-col gap-1">
                <span class="text-[9px] font-bold text-accent uppercase tracking-wider">{agent.profile}</span>
                <span class="text-[10px] text-ink-50 line-clamp-1 italic font-mono">{agent.last_action}</span>
              </div>
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
    onSelect={handleMenuSelect}
    onClose={() => (contextMenu = null)}
  />
{/if}
