<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitPush, gitPull, gitBranches, gitSwitchBranch } from '$lib/git/commands';

  let branchDropdownOpen = $state(false);
  let pushing = $state(false);
  let pulling = $state(false);
  let fetching = $state(false);

  async function handlePush() {
    if (!app.repoPath || pushing) return;
    pushing = true;
    try {
      const result = await gitPush(app.repoPath);
      if (result.success) {
        app.addToast(`Pushed to ${result.remote}/${result.branch}`, 'success');
      } else {
        app.addToast(result.message, 'error', false);
      }
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error', false);
    } finally {
      pushing = false;
    }
  }

  async function handlePull() {
    if (!app.repoPath || pulling) return;
    pulling = true;
    try {
      const result = await gitPull(app.repoPath);
      if (result.success) {
        app.addToast('Pull 完成', 'success');
      } else {
        app.addToast(result.message, 'error', false);
      }
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error', false);
    } finally {
      pulling = false;
    }
  }

  async function handleSwitchBranch(name: string) {
    if (!app.repoPath) return;
    try {
      await gitSwitchBranch(app.repoPath, name);
      app.currentBranch = name;
      branchDropdownOpen = false;
      app.addToast(`已切換到 ${name}`, 'success');
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  function toggleBranchDropdown() {
    branchDropdownOpen = !branchDropdownOpen;
    if (branchDropdownOpen && app.repoPath) {
      gitBranches(app.repoPath).then((b) => {
        app.branches = b;
      });
    }
  }

  interface Props {
    onOpenSettings?: () => void;
  }
  let { onOpenSettings }: Props = $props();
</script>

<div class="toolbar">
  <span class="repo-name">{app.repoName}</span>

  <div class="branch-wrapper">
    <button class="branch-selector" onclick={toggleBranchDropdown}>
      <span class="branch-icon">⑇</span>
      <span>{app.currentBranch || 'main'}</span>
      <span class="chevron">▾</span>
    </button>
    {#if branchDropdownOpen}
      <div class="branch-dropdown">
        {#each app.branches.filter(b => !b.is_remote) as branch}
          <button
            class="branch-item"
            class:active={branch.is_current}
            onclick={() => handleSwitchBranch(branch.name)}
          >
            {branch.name}
            {#if branch.is_current}<span class="current-marker">●</span>{/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <div class="actions">
    <button class="btn" onclick={handlePull} disabled={pulling}>
      {#if pulling}<span class="spinner"></span>{:else}↓{/if} Pull
    </button>
    <button class="btn" onclick={handlePush} disabled={pushing}>
      {#if pushing}<span class="spinner"></span>{:else}↑{/if} Push
    </button>
  </div>

  <button class="settings-btn" onclick={onOpenSettings}>⚙</button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-md);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    height: 40px;
    flex-shrink: 0;
    user-select: none;
  }
  .repo-name {
    font-weight: 600;
    font-size: var(--font-size-lg);
    margin-right: var(--space-sm);
  }
  .branch-wrapper { position: relative; }
  .branch-selector {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    color: var(--accent);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
  }
  .branch-selector:hover { border-color: var(--accent); }
  .branch-icon { font-size: var(--font-size-lg); }
  .chevron { color: var(--text-muted); font-size: 10px; }
  .branch-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: var(--space-xs);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    min-width: 200px;
    max-height: 300px;
    overflow-y: auto;
    z-index: 50;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
  .branch-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-sm) var(--space-md);
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    cursor: pointer;
    text-align: left;
  }
  .branch-item:hover { background: var(--bg-hover); }
  .branch-item.active { color: var(--accent); }
  .current-marker { color: var(--accent); font-size: 8px; }
  .actions {
    display: flex;
    gap: var(--space-xs);
    margin-left: auto;
  }
  .btn {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
  .btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .settings-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 16px;
    padding: var(--space-xs);
  }
  .settings-btn:hover { color: var(--text-primary); }
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
