<script lang="ts">
  import { extractErrorMessage } from '$lib/utils/error';
  import { app } from '$lib/stores/app.svelte';
  import {
    gitSwitchBranch,
    gitCheckoutRemoteBranch,
    gitDeleteBranch,
    gitTagCreate,
    gitTagDelete,
    gitTagDeleteRemote,
    gitPushTag,
    gitPushTags,
    gitStashList,
    gitStashPush,
    gitStashPop,
    gitStashApply,
    gitStashDrop,
    gitStashShow,
  } from '$lib/git/commands';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';
  import SubmoduleManager from './SubmoduleManager.svelte';
  import WorktreeManager from './WorktreeManager.svelte';
  import type { StashEntry } from '$lib/git/types';

  // ── State ──────────────────────────────────────────────
  let collapsedSections = $state<Record<string, boolean>>({});
  let searchQuery = $state('');

  // Stash state
  let stashes = $state<StashEntry[]>([]);
  let loadingStashes = $state(false);
  let showPushForm = $state(false);
  let pushMessage = $state('');
  let pushing = $state(false);
  let selectedStashIndex = $state<number | null>(null);

  // Double-click tracking for local branches
  let lastClickBranch = $state('');
  let lastClickTime = $state(0);
  const DBLCLICK_MS = 400;

  // Context menus
  let localBranchMenu = $state<{ name: string; x: number; y: number } | null>(null);
  let remoteBranchMenu = $state<{ name: string; x: number; y: number } | null>(null);
  let stashMenu = $state<{ index: number; x: number; y: number } | null>(null);
  let tagContextMenu = $state<{ tagName: string; x: number; y: number } | null>(null);

  // ── Derived ────────────────────────────────────────────

  const localBranches = $derived(
    app.branches
      .filter((b) => !b.is_remote)
      .filter((b) => !searchQuery || b.name.toLowerCase().includes(searchQuery.toLowerCase())),
  );

  const remoteGroups = $derived.by(() => {
    const groups: Record<string, typeof app.branches> = {};
    for (const b of app.branches.filter((b) => b.is_remote)) {
      if (searchQuery && !b.name.toLowerCase().includes(searchQuery.toLowerCase())) continue;
      const slash = b.name.indexOf('/');
      const remote = slash > 0 ? b.name.substring(0, slash) : 'origin';
      if (!groups[remote]) groups[remote] = [];
      groups[remote].push(b);
    }
    return groups;
  });

  const filteredTags = $derived(
    app.tags.filter(
      (t) => !searchQuery || t.name.toLowerCase().includes(searchQuery.toLowerCase()),
    ),
  );

  const filteredStashes = $derived(
    stashes.filter(
      (s) => !searchQuery || s.message.toLowerCase().includes(searchQuery.toLowerCase()),
    ),
  );

  // ── Section helpers ────────────────────────────────────

  function toggleSection(key: string) {
    collapsedSections = { ...collapsedSections, [key]: !collapsedSections[key] };
  }

  function isCollapsed(key: string): boolean {
    return !!collapsedSections[key];
  }

  // ── Stash loading ──────────────────────────────────────

  async function loadStashes() {
    if (!app.repoPath) return;
    loadingStashes = true;
    try {
      stashes = await gitStashList(app.repoPath);
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      loadingStashes = false;
    }
  }

  $effect(() => {
    // Reload stashes whenever the active repo changes
    const _path = app.repoPath;
    loadStashes();
  });

  // ── Local branch: single click = filter log, double click = checkout ──

  async function handleLocalBranchClick(branchName: string) {
    const now = Date.now();
    if (lastClickBranch === branchName && now - lastClickTime < DBLCLICK_MS) {
      // Double click → checkout
      lastClickBranch = '';
      lastClickTime = 0;
      await checkoutLocalBranch(branchName);
    } else {
      // Single click → filter log
      lastClickBranch = branchName;
      lastClickTime = now;
      app.setLogFilter({ type: 'Branch', value: branchName });
    }
  }

  async function checkoutLocalBranch(branchName: string) {
    if (!app.repoPath) return;
    try {
      await gitSwitchBranch(app.repoPath, branchName);
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  // ── Remote branch ─────────────────────────────────────

  function handleRemoteBranchClick(branchName: string) {
    app.setLogFilter({ type: 'Branch', value: branchName });
  }

  function shortRemoteBranch(name: string): string {
    const slash = name.indexOf('/');
    return slash > 0 ? name.substring(slash + 1) : name;
  }

  // ── Stash operations ───────────────────────────────────

  async function handleStashClick(index: number) {
    if (!app.repoPath) return;
    selectedStashIndex = index;
    try {
      const raw = await gitStashShow(app.repoPath, index);
      app.stashDiff = raw;
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleStashPush() {
    if (!app.repoPath) return;
    pushing = true;
    try {
      const msg = pushMessage.trim() || undefined;
      await gitStashPush(app.repoPath, msg);
      app.addToast('已 stash 變更', 'success');
      pushMessage = '';
      showPushForm = false;
      await Promise.all([loadStashes(), app.refreshStatus()]);
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      pushing = false;
    }
  }

  async function handleStashPop(index: number) {
    if (!app.repoPath) return;
    try {
      await gitStashPop(app.repoPath, index);
      app.addToast('已 pop stash', 'success');
      if (selectedStashIndex === index) {
        app.stashDiff = null;
        selectedStashIndex = null;
      }
      await Promise.all([loadStashes(), app.refreshAll()]);
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleStashApply(index: number) {
    if (!app.repoPath) return;
    try {
      await gitStashApply(app.repoPath, index);
      app.addToast('已 apply stash', 'success');
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleStashDrop(index: number) {
    if (!app.repoPath) return;
    try {
      await gitStashDrop(app.repoPath, index);
      app.addToast('已刪除 stash', 'success');
      if (selectedStashIndex === index) {
        app.stashDiff = null;
        selectedStashIndex = null;
      }
      await loadStashes();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  // ── Tag operations ─────────────────────────────────────

  function handleTagContextMenu(e: MouseEvent, tagName: string) {
    e.preventDefault();
    e.stopPropagation();
    tagContextMenu = { tagName, x: e.clientX, y: e.clientY };
  }

  const tagContextMenuItems: MenuItem[] = [
    { id: 'pushTag', label: '推送此 Tag' },
    { id: '_sep1', label: '', separator: true },
    { id: 'deleteTag', label: '刪除本地 Tag' },
    { id: 'deleteRemoteTag', label: '刪除遠端 Tag' },
  ];

  async function handleTagContextSelect(actionId: string) {
    if (!tagContextMenu || !app.repoPath) return;
    const { tagName } = tagContextMenu;
    tagContextMenu = null;
    try {
      switch (actionId) {
        case 'pushTag': {
          const result = await gitPushTag(app.repoPath, tagName);
          if (result.success) {
            app.addToast(`已推送 tag: ${tagName}`, 'success');
          } else {
            app.addToast(result.message, 'error', false);
          }
          break;
        }
        case 'deleteTag': {
          if (confirm(`確定要刪除本地 tag「${tagName}」？`)) {
            await gitTagDelete(app.repoPath, tagName);
            app.addToast(`已刪除 tag: ${tagName}`, 'success');
            await app.refreshAll();
          }
          break;
        }
        case 'deleteRemoteTag': {
          if (confirm(`確定要刪除遠端 tag「${tagName}」？\n\n此操作會從 origin 移除該 tag。`)) {
            const result = await gitTagDeleteRemote(app.repoPath, tagName);
            if (result.success) {
              app.addToast(`已刪除遠端 tag: ${tagName}`, 'success');
            } else {
              app.addToast(result.message, 'error', false);
            }
          }
          break;
        }
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleCreateTag() {
    if (!app.repoPath) return;
    const tagName = prompt('Tag 名稱:');
    if (!tagName?.trim()) return;
    try {
      await gitTagCreate(app.repoPath, tagName.trim());
      app.addToast(`已建立 tag: ${tagName.trim()}`, 'success');
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handlePushAllTags() {
    if (!app.repoPath) return;
    try {
      const result = await gitPushTags(app.repoPath);
      if (result.success) {
        app.addToast('已推送所有 tags', 'success');
      } else {
        app.addToast(result.message, 'error', false);
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  // ── Local branch context menu ──────────────────────────

  function handleLocalBranchContextMenu(e: MouseEvent, name: string) {
    e.preventDefault();
    e.stopPropagation();
    localBranchMenu = { name, x: e.clientX, y: e.clientY };
  }

  const localBranchMenuItems: MenuItem[] = [
    { id: 'checkout', label: 'Checkout' },
    { id: 'filter', label: '過濾 Commit Log' },
    { id: '_sep1', label: '', separator: true },
    { id: 'delete', label: '刪除分支' },
  ];

  async function handleLocalBranchMenuSelect(actionId: string) {
    if (!localBranchMenu || !app.repoPath) return;
    const { name } = localBranchMenu;
    localBranchMenu = null;
    try {
      switch (actionId) {
        case 'checkout':
          await checkoutLocalBranch(name);
          break;
        case 'filter':
          app.setLogFilter({ type: 'Branch', value: name });
          break;
        case 'delete': {
          if (confirm(`確定要刪除本地分支「${name}」？`)) {
            await gitDeleteBranch(app.repoPath, name);
            app.addToast(`已刪除分支: ${name}`, 'success');
            await app.refreshAll();
          }
          break;
        }
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  // ── Remote branch context menu ─────────────────────────

  function handleRemoteBranchContextMenu(e: MouseEvent, name: string) {
    e.preventDefault();
    e.stopPropagation();
    remoteBranchMenu = { name, x: e.clientX, y: e.clientY };
  }

  const remoteBranchMenuItems: MenuItem[] = [
    { id: 'filter', label: '過濾 Commit Log' },
    { id: 'checkout', label: 'Checkout 為本地分支' },
  ];

  async function handleRemoteBranchMenuSelect(actionId: string) {
    if (!remoteBranchMenu || !app.repoPath) return;
    const { name } = remoteBranchMenu;
    remoteBranchMenu = null;
    try {
      switch (actionId) {
        case 'filter':
          app.setLogFilter({ type: 'Branch', value: name });
          break;
        case 'checkout': {
          const localName = await gitCheckoutRemoteBranch(app.repoPath, name);
          app.addToast(`已建立本地分支: ${localName}`, 'success');
          await app.refreshAll();
          break;
        }
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  // ── Stash context menu ─────────────────────────────────

  function handleStashContextMenu(e: MouseEvent, index: number) {
    e.preventDefault();
    e.stopPropagation();
    stashMenu = { index, x: e.clientX, y: e.clientY };
  }

  const stashMenuItems: MenuItem[] = [
    { id: 'pop', label: 'Pop（apply + drop）' },
    { id: 'apply', label: 'Apply（保留 stash）' },
    { id: '_sep1', label: '', separator: true },
    { id: 'drop', label: '刪除 stash' },
  ];

  async function handleStashMenuSelect(actionId: string) {
    if (!stashMenu) return;
    const { index } = stashMenu;
    stashMenu = null;
    switch (actionId) {
      case 'pop':
        await handleStashPop(index);
        break;
      case 'apply':
        await handleStashApply(index);
        break;
      case 'drop':
        await handleStashDrop(index);
        break;
    }
  }
</script>

<div class="branch-sidebar flex flex-col h-full bg-bg border-r border-ink-10">
  <!-- Search -->
  <div class="px-4 py-3 border-b border-ink-10">
    <div class="relative group">
      <input
        type="text"
        class="w-full bg-ink-05 border border-transparent focus:border-accent/30 rounded-lg px-3 py-1.5 text-[12px] text-ink placeholder:text-ink-35 outline-none transition-all"
        placeholder="Filter everything..."
        bind:value={searchQuery}
      />
      {#if searchQuery}
        <button class="absolute right-2 top-1/2 -translate-y-1/2 text-ink-35 hover:text-ink" onclick={() => (searchQuery = '')}>×</button>
      {/if}
    </div>
  </div>

  <div class="flex-1 overflow-y-auto overflow-x-hidden custom-scrollbar">
    <!-- WORKTREES (TOP PRIORITY) -->
    <WorktreeManager />

    <div class="h-px bg-ink-10 mx-4 my-2"></div>

    <!-- LOCAL -->
    <div class="px-4 py-2 flex items-center justify-between group/header">
      <button 
        class="flex items-center gap-1.5 text-[10.5px] font-bold text-ink-35 uppercase tracking-[0.5px] hover:text-ink transition-colors"
        onclick={() => toggleSection('local')}
      >
        <span class="w-3 text-center transition-transform duration-200" class:rotate-[-90deg]={isCollapsed('local')}>▾</span>
        LOCAL
        <span class="ml-1 px-1.5 py-0.5 rounded-full bg-ink-05 text-ink-35 text-[9px]">{localBranches.length}</span>
      </button>
    </div>
    {#if !isCollapsed('local')}
      <div class="flex flex-col pb-4">
        {#each localBranches as branch (branch.name)}
          <button
            class="flex items-center gap-2 px-8 py-1.5 hover:bg-ink-05 transition-colors text-left relative group border-l-2 border-transparent"
            class:border-accent={branch.is_current}
            class:bg-accent-bg={branch.is_current}
            onclick={() => handleLocalBranchClick(branch.name)}
            oncontextmenu={(e) => handleLocalBranchContextMenu(e, branch.name)}
          >
            <span class="text-[10px] text-accent opacity-60">⑇</span>
            <span class="text-[13px] font-medium text-ink truncate shrink min-w-0" class:font-bold={branch.is_current}>{branch.name}</span>
            {#if branch.ahead || branch.behind}
              <div class="ml-auto flex items-center gap-1 text-[9px] font-bold">
                {#if branch.ahead}<span class="text-accent">↑{branch.ahead}</span>{/if}
                {#if branch.behind}<span class="text-warn">↓{branch.behind}</span>{/if}
              </div>
            {/if}
          </button>
        {/each}
      </div>
    {/if}

    <!-- REMOTE -->
    {#each Object.entries(remoteGroups) as [remote, branches] (remote)}
      <div class="px-4 py-2 flex items-center justify-between group/header">
        <button 
          class="flex items-center gap-1.5 text-[10.5px] font-bold text-ink-35 uppercase tracking-[0.5px] hover:text-ink transition-colors"
          onclick={() => toggleSection(`remote-${remote}`)}
        >
          <span class="w-3 text-center transition-transform duration-200" class:rotate-[-90deg]={isCollapsed(`remote-${remote}`)}>▾</span>
          REMOTE / {remote}
          <span class="ml-1 px-1.5 py-0.5 rounded-full bg-ink-05 text-ink-35 text-[9px]">{branches.length}</span>
        </button>
      </div>
      {#if !isCollapsed(`remote-${remote}`)}
        <div class="flex flex-col pb-4">
          {#each branches as branch (branch.name)}
            <button
              class="flex items-center gap-2 px-8 py-1.5 hover:bg-ink-05 transition-colors text-left border-l-2 border-transparent"
              onclick={() => handleRemoteBranchClick(branch.name)}
              oncontextmenu={(e) => handleRemoteBranchContextMenu(e, branch.name)}
            >
              <span class="text-[10px] text-ink-35">☁</span>
              <span class="text-[13px] font-medium text-ink-70 truncate shrink min-w-0">{shortRemoteBranch(branch.name)}</span>
            </button>
          {/each}
        </div>
      {/if}
    {/each}

    <!-- TAGS -->
    <div class="px-4 py-2 flex items-center justify-between group/header">
      <button 
        class="flex items-center gap-1.5 text-[10.5px] font-bold text-ink-35 uppercase tracking-[0.5px] hover:text-ink transition-colors"
        onclick={() => toggleSection('tags')}
      >
        <span class="w-3 text-center transition-transform duration-200" class:rotate-[-90deg]={isCollapsed('tags')}>▾</span>
        TAGS
        <span class="ml-1 px-1.5 py-0.5 rounded-full bg-ink-05 text-ink-35 text-[9px]">{filteredTags.length}</span>
      </button>
      <div class="flex items-center gap-1 opacity-0 group-hover/header:opacity-100 transition-opacity">
        <button class="p-1 rounded hover:bg-ink-05 text-ink-35 hover:text-ink" onclick={handleCreateTag}>+</button>
      </div>
    </div>
    {#if !isCollapsed('tags')}
      <div class="flex flex-col pb-4">
        {#each filteredTags as tag (tag.name)}
          <button
            class="flex items-center gap-2 px-8 py-1.5 hover:bg-ink-05 transition-colors text-left border-l-2 border-transparent"
            oncontextmenu={(e) => handleTagContextMenu(e, tag.name)}
          >
            <span class="text-[10px] text-ink-35">🏷</span>
            <span class="text-[13px] font-medium text-ink-70 truncate shrink min-w-0">{tag.name}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Context menus -->
{#if tagContextMenu}
  <ContextMenu
    x={tagContextMenu.x}
    y={tagContextMenu.y}
    items={tagContextMenuItems}
    onSelect={handleTagContextSelect}
    onClose={() => (tagContextMenu = null)}
  />
{/if}

{#if localBranchMenu}
  <ContextMenu
    x={localBranchMenu.x}
    y={localBranchMenu.y}
    items={localBranchMenuItems}
    onSelect={handleLocalBranchMenuSelect}
    onClose={() => (localBranchMenu = null)}
  />
{/if}

{#if remoteBranchMenu}
  <ContextMenu
    x={remoteBranchMenu.x}
    y={remoteBranchMenu.y}
    items={remoteBranchMenuItems}
    onSelect={handleRemoteBranchMenuSelect}
    onClose={() => (remoteBranchMenu = null)}
  />
{/if}

{#if stashMenu}
  <ContextMenu
    x={stashMenu.x}
    y={stashMenu.y}
    items={stashMenuItems}
    onSelect={handleStashMenuSelect}
    onClose={() => (stashMenu = null)}
  />
{/if}

<style>
  .branch-sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    user-select: none;
  }

  /* ── Search ── */

  .search-bar {
    position: relative;
    padding: var(--space-xs) var(--space-sm);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .search-input {
    width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    padding-right: 24px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-ui);
    outline: none;
    box-sizing: border-box;
  }
  .search-input:focus {
    border-color: var(--accent);
  }
  .search-input::placeholder {
    color: var(--text-muted);
  }

  .search-clear {
    position: absolute;
    right: calc(var(--space-sm) + 6px);
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 0 2px;
  }
  .search-clear:hover {
    color: var(--text-primary);
  }

  /* ── Section toggles ── */

  .section-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
    width: 100%;
    text-align: left;
    font-family: var(--font-ui);
  }
  .section-toggle:hover {
    background: var(--bg-hover);
  }

  .toggle-icon {
    font-size: 10px;
    width: 12px;
    flex-shrink: 0;
  }

  .section-label {
    flex-shrink: 0;
  }

  .remote-name {
    font-weight: 400;
    color: var(--text-muted);
    text-transform: none;
    letter-spacing: 0;
  }

  .section-count {
    margin-left: auto;
    background: var(--bg-surface);
    padding: 1px 6px;
    border-radius: 8px;
    font-size: 10px;
    color: var(--text-muted);
  }

  /* ── Branch items ── */

  .branch-list {
    display: flex;
    flex-direction: column;
  }

  .branch-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-md) var(--space-xs) calc(var(--space-md) + 12px);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    color: var(--text-primary);
    background: none;
    border: none;
    border-left: 2px solid transparent;
    cursor: pointer;
    width: 100%;
    text-align: left;
  }
  .branch-item:hover {
    background: var(--bg-hover);
  }
  .branch-item.current {
    border-left-color: var(--accent);
    background: var(--bg-surface);
  }
  .branch-item.filtered {
    border-left-color: var(--warning);
    background: var(--bg-surface);
  }
  .branch-item.current.filtered {
    border-left-color: var(--accent);
  }
  .branch-item.stash-selected {
    border-left-color: var(--accent);
    background: var(--bg-surface);
  }

  .branch-icon {
    font-size: 12px;
    color: var(--accent);
    flex-shrink: 0;
  }
  .remote-icon {
    color: var(--text-muted);
  }
  .tag-icon {
    color: var(--warning);
  }
  .stash-icon {
    font-size: 11px;
    color: var(--text-muted);
  }

  .branch-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .sync-status {
    display: flex;
    gap: 2px;
    font-size: 10px;
    flex-shrink: 0;
  }
  .ahead {
    color: var(--success);
  }
  .behind {
    color: var(--warning);
  }

  /* ── Tags ── */

  .section-header-row {
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border);
  }
  .section-header-row:hover {
    background: var(--bg-hover);
  }

  .tag-toggle,
  .stash-toggle {
    flex: 1;
    border-bottom: none;
  }

  .section-actions {
    display: flex;
    gap: 2px;
    padding-right: var(--space-sm);
    flex-shrink: 0;
  }

  .section-action-btn {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-muted);
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .section-action-btn:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
  }

  .tag-item {
    align-items: flex-start;
  }

  .tag-commit {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    flex-shrink: 0;
  }

  /* ── Stash items ── */

  .stash-item {
    align-items: flex-start;
    gap: var(--space-xs);
  }

  .stash-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex: 1;
    min-width: 0;
  }

  .stash-ref {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .stash-msg {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
  }

  /* ── Stash push form ── */

  .push-form {
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .push-input {
    width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-sm);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    outline: none;
    box-sizing: border-box;
  }
  .push-input:focus {
    border-color: var(--accent);
  }

  .push-actions {
    display: flex;
    gap: var(--space-xs);
    align-items: center;
  }

  .btn-create {
    background: var(--accent);
    color: var(--bg-primary);
    border: none;
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
  .btn-create:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-text {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }
  .btn-text:hover {
    color: var(--text-primary);
  }

  /* ── Misc ── */

  .empty-hint {
    padding: var(--space-sm) var(--space-md) var(--space-sm) calc(var(--space-md) + 12px);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-style: italic;
  }

  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid var(--text-muted);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
