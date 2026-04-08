<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitSwitchBranch } from '$lib/git/commands';

  let collapsedSections = $state<Record<string, boolean>>({});

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
      // Switch to local branch
      try {
        await gitSwitchBranch(app.repoPath, branchName);
        await app.refreshAll();
      } catch (e: unknown) {
        app.addToast(String(e), 'error');
      }
    } else {
      // Filter commit log by this remote branch
      app.setLogFilter({ type: 'Branch', value: branchName });
    }
  }

  function shortRemoteBranch(name: string): string {
    const slash = name.indexOf('/');
    return slash > 0 ? name.substring(slash + 1) : name;
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
</div>

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
</style>
