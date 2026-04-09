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
      .filter((b) => !searchQuery || b.name.toLowerCase().includes(searchQuery.toLowerCase()))
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
    app.tags.filter((t) => !searchQuery || t.name.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  const filteredStashes = $derived(
    stashes.filter((s) => !searchQuery || s.message.toLowerCase().includes(searchQuery.toLowerCase()))
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
      if (selectedStashIndex === index) { app.stashDiff = null; selectedStashIndex = null; }
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
      if (selectedStashIndex === index) { app.stashDiff = null; selectedStashIndex = null; }
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
      case 'pop': await handleStashPop(index); break;
      case 'apply': await handleStashApply(index); break;
      case 'drop': await handleStashDrop(index); break;
    }
  }
</script>

<div class="branch-sidebar">
  <!-- Search -->
  <div class="search-bar">
    <input
      type="text"
      class="search-input"
      placeholder="過濾分支 / Tag / Stash..."
      bind:value={searchQuery}
    />
    {#if searchQuery}
      <button class="search-clear" onclick={() => searchQuery = ''} aria-label="清除搜尋">×</button>
    {/if}
  </div>

  <!-- LOCAL -->
  <button class="section-toggle" onclick={() => toggleSection('local')}>
    <span class="toggle-icon">{isCollapsed('local') ? '▸' : '▾'}</span>
    <span class="section-label">LOCAL</span>
    <span class="section-count">{localBranches.length}</span>
  </button>
  {#if !isCollapsed('local')}
    <div class="branch-list">
      {#each localBranches as branch (branch.name)}
        <button
          class="branch-item"
          class:current={branch.is_current}
          class:filtered={app.logFilter.type === 'Branch' && app.logFilter.value === branch.name}
          onclick={() => handleLocalBranchClick(branch.name)}
          oncontextmenu={(e) => handleLocalBranchContextMenu(e, branch.name)}
          title="{branch.name}（單擊過濾 / 雙擊 checkout）"
        >
          <span class="branch-icon">⎇</span>
          <span class="branch-name">{branch.name}</span>
          {#if branch.ahead || branch.behind}
            <span class="sync-status">
              {#if branch.ahead}<span class="ahead">↑{branch.ahead}</span>{/if}
              {#if branch.behind}<span class="behind">↓{branch.behind}</span>{/if}
            </span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}

  <!-- REMOTE -->
  {#each Object.entries(remoteGroups) as [remote, branches] (remote)}
    <button class="section-toggle" onclick={() => toggleSection(`remote-${remote}`)}>
      <span class="toggle-icon">{isCollapsed(`remote-${remote}`) ? '▸' : '▾'}</span>
      <span class="section-label">REMOTE</span>
      <span class="remote-name">{remote}</span>
      <span class="section-count">{branches.length}</span>
    </button>
    {#if !isCollapsed(`remote-${remote}`)}
      <div class="branch-list">
        {#each branches as branch (branch.name)}
          <button
            class="branch-item"
            class:filtered={app.logFilter.type === 'Branch' && app.logFilter.value === branch.name}
            onclick={() => handleRemoteBranchClick(branch.name)}
            oncontextmenu={(e) => handleRemoteBranchContextMenu(e, branch.name)}
            title={branch.name}
          >
            <span class="branch-icon remote-icon">☁</span>
            <span class="branch-name">{shortRemoteBranch(branch.name)}</span>
          </button>
        {/each}
      </div>
    {/if}
  {/each}

  <!-- TAGS -->
  <div class="section-header-row">
    <button class="section-toggle tag-toggle" onclick={() => toggleSection('tags')}>
      <span class="toggle-icon">{isCollapsed('tags') ? '▸' : '▾'}</span>
      <span class="section-label">TAGS</span>
      <span class="section-count">{filteredTags.length}</span>
    </button>
    <div class="section-actions">
      {#if app.tags.length > 0}
        <button class="section-action-btn" title="推送所有 Tags" onclick={handlePushAllTags}>↑</button>
      {/if}
      <button class="section-action-btn" title="建立 Tag" onclick={handleCreateTag}>+</button>
    </div>
  </div>
  {#if !isCollapsed('tags')}
    <div class="branch-list">
      {#each filteredTags as tag (tag.name)}
        <button
          class="branch-item tag-item"
          title="{tag.name} ({tag.commit_id}){tag.message ? '\n' + tag.message : ''}"
          oncontextmenu={(e) => handleTagContextMenu(e, tag.name)}
        >
          <span class="branch-icon tag-icon">🏷</span>
          <span class="branch-name">{tag.name}</span>
          <span class="tag-commit">{tag.commit_id}</span>
        </button>
      {/each}
      {#if filteredTags.length === 0}
        <div class="empty-hint">{searchQuery ? '無符合結果' : '尚無 Tags'}</div>
      {/if}
    </div>
  {/if}

  <!-- STASHES -->
  <div class="section-header-row">
    <button class="section-toggle stash-toggle" onclick={() => toggleSection('stashes')}>
      <span class="toggle-icon">{isCollapsed('stashes') ? '▸' : '▾'}</span>
      <span class="section-label">STASHES</span>
      <span class="section-count">{filteredStashes.length}</span>
    </button>
    <div class="section-actions">
      <button class="section-action-btn" title="Stash 所有變更" onclick={() => showPushForm = !showPushForm}>+</button>
    </div>
  </div>
  {#if !isCollapsed('stashes')}
    <div class="branch-list">
      {#if showPushForm}
        <div class="push-form">
          <input
            type="text"
            placeholder="Stash 描述（選填）..."
            bind:value={pushMessage}
            class="push-input"
            autofocus
            onkeydown={(e) => {
              if (e.key === 'Enter') handleStashPush();
              if (e.key === 'Escape') { showPushForm = false; pushMessage = ''; }
            }}
          />
          <div class="push-actions">
            <button class="btn-create" onclick={handleStashPush} disabled={pushing}>
              {#if pushing}<span class="spinner"></span>{:else}Stash{/if}
            </button>
            <button class="btn-text" onclick={() => { showPushForm = false; pushMessage = ''; }}>取消</button>
          </div>
        </div>
      {/if}
      {#if loadingStashes}
        <div class="empty-hint"><span class="spinner"></span></div>
      {:else if filteredStashes.length === 0}
        <div class="empty-hint">{searchQuery ? '無符合結果' : '尚無 Stash'}</div>
      {:else}
        {#each filteredStashes as stash (stash.index)}
          <button
            class="branch-item stash-item"
            class:stash-selected={selectedStashIndex === stash.index}
            onclick={() => handleStashClick(stash.index)}
            oncontextmenu={(e) => handleStashContextMenu(e, stash.index)}
            title="stash@{'{' + stash.index + '}'}: {stash.message}（點擊預覽 diff）"
          >
            <span class="branch-icon stash-icon">📦</span>
            <div class="stash-info">
              <span class="stash-ref">stash@{'{' + stash.index + '}'}</span>
              <span class="branch-name stash-msg">{stash.message}</span>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<!-- Context menus -->
{#if tagContextMenu}
  <ContextMenu
    x={tagContextMenu.x}
    y={tagContextMenu.y}
    items={tagContextMenuItems}
    onSelect={handleTagContextSelect}
    onClose={() => tagContextMenu = null}
  />
{/if}

{#if localBranchMenu}
  <ContextMenu
    x={localBranchMenu.x}
    y={localBranchMenu.y}
    items={localBranchMenuItems}
    onSelect={handleLocalBranchMenuSelect}
    onClose={() => localBranchMenu = null}
  />
{/if}

{#if remoteBranchMenu}
  <ContextMenu
    x={remoteBranchMenu.x}
    y={remoteBranchMenu.y}
    items={remoteBranchMenuItems}
    onSelect={handleRemoteBranchMenuSelect}
    onClose={() => remoteBranchMenu = null}
  />
{/if}

{#if stashMenu}
  <ContextMenu
    x={stashMenu.x}
    y={stashMenu.y}
    items={stashMenuItems}
    onSelect={handleStashMenuSelect}
    onClose={() => stashMenu = null}
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
  .search-input:focus { border-color: var(--accent); }
  .search-input::placeholder { color: var(--text-muted); }

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
  .search-clear:hover { color: var(--text-primary); }

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
  .section-toggle:hover { background: var(--bg-hover); }

  .toggle-icon {
    font-size: 10px;
    width: 12px;
    flex-shrink: 0;
  }

  .section-label { flex-shrink: 0; }

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
  .branch-item:hover { background: var(--bg-hover); }
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
  .remote-icon { color: var(--text-muted); }
  .tag-icon { color: var(--warning); }
  .stash-icon { font-size: 11px; color: var(--text-muted); }

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
  .ahead { color: var(--success); }
  .behind { color: var(--warning); }

  /* ── Tags ── */

  .section-header-row {
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border);
  }
  .section-header-row:hover { background: var(--bg-hover); }

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

  .tag-item { align-items: flex-start; }

  .tag-commit {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    flex-shrink: 0;
  }

  /* ── Stash items ── */

  .stash-item { align-items: flex-start; gap: var(--space-xs); }

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
  .push-input:focus { border-color: var(--accent); }

  .push-actions { display: flex; gap: var(--space-xs); align-items: center; }

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
  .btn-create:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-text {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }
  .btn-text:hover { color: var(--text-primary); }

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
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
