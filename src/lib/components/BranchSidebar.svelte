<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitSwitchBranch, gitTagCreate, gitTagDelete, gitTagDeleteRemote, gitPushTag, gitPushTags } from '$lib/git/commands';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';

  let collapsedSections = $state<Record<string, boolean>>({});
  let tagContextMenu = $state<{ tagName: string; x: number; y: number } | null>(null);

  /** Group remote branches by remote name (e.g., "origin") */
  const remoteGroups = $derived.by(() => {
    const groups: Record<string, typeof app.branches> = {};
    for (const b of app.branches.filter((b) => b.is_remote)) {
      const slash = b.name.indexOf('/');
      const remote = slash > 0 ? b.name.substring(0, slash) : 'origin';
      if (!groups[remote]) groups[remote] = [];
      groups[remote].push(b);
    }
    return groups;
  });

  const localBranches = $derived(app.branches.filter((b) => !b.is_remote));

  function toggleSection(key: string) {
    collapsedSections = { ...collapsedSections, [key]: !collapsedSections[key] };
  }

  function isCollapsed(key: string): boolean {
    return !!collapsedSections[key];
  }

  async function handleBranchClick(branchName: string, isRemote: boolean) {
    if (!app.repoPath) return;
    if (!isRemote) {
      try {
        await gitSwitchBranch(app.repoPath, branchName);
        await app.refreshAll();
      } catch (e: unknown) {
        app.addToast(String(e), 'error');
      }
    } else {
      app.setLogFilter({ type: 'Branch', value: branchName });
    }
  }

  function shortRemoteBranch(name: string): string {
    const slash = name.indexOf('/');
    return slash > 0 ? name.substring(slash + 1) : name;
  }

  // ── Tag context menu ──

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
      app.addToast(String(e), 'error');
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
      app.addToast(String(e), 'error');
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
      app.addToast(String(e), 'error');
    }
  }
</script>

<div class="branch-sidebar">
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
          onclick={() => handleBranchClick(branch.name, false)}
          title={branch.name}
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
            onclick={() => handleBranchClick(branch.name, true)}
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
      <span class="section-count">{app.tags.length}</span>
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
      {#each app.tags as tag (tag.name)}
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
      {#if app.tags.length === 0}
        <div class="empty-hint">尚無 Tags</div>
      {/if}
    </div>
  {/if}
</div>

{#if tagContextMenu}
  <ContextMenu
    x={tagContextMenu.x}
    y={tagContextMenu.y}
    items={tagContextMenuItems}
    onSelect={handleTagContextSelect}
    onClose={() => tagContextMenu = null}
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

  .branch-icon {
    font-size: 12px;
    color: var(--accent);
    flex-shrink: 0;
  }
  .remote-icon { color: var(--text-muted); }

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

  /* ── Tags section ── */

  .section-header-row {
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border);
  }
  .section-header-row:hover { background: var(--bg-hover); }

  .tag-toggle {
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

  .tag-icon { color: var(--warning); }

  .tag-commit {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    flex-shrink: 0;
  }

  .empty-hint {
    padding: var(--space-sm) var(--space-md) var(--space-sm) calc(var(--space-md) + 12px);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-style: italic;
  }
</style>
