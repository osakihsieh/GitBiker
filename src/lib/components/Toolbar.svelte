<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { multiRepo } from '$lib/stores/multiRepoStore.svelte';
  import { gitPush, gitPushTags, gitPull, gitFetch, gitBranches, gitSwitchBranch, openInFolder, openInEditor, openInTerminal } from '$lib/git/commands';
  import BranchManager from './BranchManager.svelte';

  let branchDropdownOpen = $state(false);
  let branchManagerOpen = $state(false);
  let pushing = $state(false);
  let pushingTags = $state(false);
  let pulling = $state(false);
  let fetching = $state(false);

  async function handleOpenFolder() {
    if (!app.repoPath) return;
    try {
      await openInFolder(app.repoPath);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleOpenEditor() {
    if (!app.repoPath) return;
    try {
      await openInEditor(app.repoPath, app.preferredEditor ?? undefined);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleOpenTerminal() {
    if (!app.repoPath) return;
    try {
      await openInTerminal(app.repoPath);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

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

  async function handlePushTags() {
    if (!app.repoPath || pushingTags) return;
    pushingTags = true;
    try {
      const result = await gitPushTags(app.repoPath);
      if (result.success) {
        app.addToast(`已推送所有 Tags 到 ${result.remote}`, 'success');
      } else {
        app.addToast(result.message, 'error', false);
      }
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error', false);
    } finally {
      pushingTags = false;
    }
  }

  async function handlePull() {
    if (!app.repoPath || pulling) return;
    pulling = true;
    try {
      const result = await gitPull(app.repoPath);
      if (result.success) {
        app.addToast('Pull 完成', 'success');
      } else if (result.conflicts.length > 0) {
        app.addToast(`Pull 衝突：${result.conflicts.length} 個檔案需要解決`, 'error', false);
        await app.refreshAll();
        await app.enterConflictMode();
        return;
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

  async function handleFetch() {
    if (!app.repoPath || fetching) return;
    fetching = true;
    try {
      await gitFetch(app.repoPath);
      await app.refreshAll();
      app.addToast('Fetch 完成', 'success');
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      fetching = false;
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

  function closeBranchDropdown() {
    branchDropdownOpen = false;
  }

  function openBranchManager() {
    branchDropdownOpen = false;
    branchManagerOpen = true;
    if (app.repoPath) {
      gitBranches(app.repoPath).then((b) => {
        app.branches = b;
      });
    }
  }

  function closeBranchManager() {
    branchManagerOpen = false;
  }

  function handleBranchKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && branchDropdownOpen) {
      e.stopPropagation();
      closeBranchDropdown();
    }
  }

  interface Props {
    onOpenSettings?: () => void;
    onOpenPopover?: () => void;
    onOpenMultiRepo?: () => void;
  }
  let { onOpenSettings, onOpenPopover, onOpenMultiRepo }: Props = $props();
</script>

<svelte:window onkeydown={handleBranchKeydown} />

<div class="toolbar">
  <button class="folder-btn" onclick={onOpenPopover} aria-label="Open repo switcher">
    <span class="folder-icon">📁</span><span class="chevron">▾</span>
  </button>

  <button class="multi-repo-btn" onclick={onOpenMultiRepo} aria-label="Multi-repo manager (Ctrl+M)">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
      <rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
    </svg>
    {#if multiRepo.dirtyCount > 0}
      <span class="multi-repo-badge" aria-label="{multiRepo.dirtyCount} repositories with changes">
        {multiRepo.dirtyCount > 9 ? '9+' : multiRepo.dirtyCount}
      </span>
    {/if}
  </button>

  <div class="branch-wrapper">
    <button class="branch-selector" onclick={toggleBranchDropdown}>
      <span class="branch-icon">⑇</span>
      <span>{app.currentBranch || 'main'}</span>
      <span class="chevron">▾</span>
    </button>
    {#if branchDropdownOpen}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="dropdown-overlay" onclick={closeBranchDropdown}></div>
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
        <div class="dropdown-divider"></div>
        <button class="manage-link" onclick={openBranchManager}>
          管理分支...
        </button>
      </div>
    {/if}
    {#if app.conflictFiles.length > 0}
      <button class="conflict-badge" onclick={() => app.isInConflictMode ? app.exitConflictMode() : app.enterConflictMode()} title="Ctrl+Shift+M">
        ⚠ {app.conflictFiles.length}
      </button>
    {/if}

    {#if branchManagerOpen}
      <BranchManager open={branchManagerOpen} onClose={closeBranchManager} />
    {/if}
  </div>

  <div class="external-tools">
    <button class="tool-btn" onclick={handleOpenFolder} title="在檔案總管開啟 (Alt+O)" aria-label="Open in file explorer">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
    </button>
    <button class="tool-btn" onclick={handleOpenEditor} title="在編輯器開啟 (Alt+E)" aria-label="Open in editor">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
    </button>
    <button class="tool-btn" onclick={handleOpenTerminal} title="開啟外部終端機 (Alt+T)" aria-label="Open terminal">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
    </button>
    <button class="tool-btn" class:active={app.showTerminal} onclick={() => app.toggleTerminal()} title="切換內建終端機 (Ctrl+`)" aria-label="Toggle inline terminal">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><path d="M6 18l2-2-2-2"/><line x1="12" y1="18" x2="16" y2="18"/></svg>
    </button>
  </div>

  <div class="drag-spacer" data-tauri-drag-region></div>

  <div class="actions">
    <button class="btn" onclick={handlePull} disabled={pulling}>
      {#if pulling}<span class="spinner"></span>{:else}↓{/if} Pull
    </button>
    <button class="btn" onclick={handlePush} disabled={pushing}>
      {#if pushing}<span class="spinner"></span>{:else}↑{/if} Push
    </button>
    <button class="btn" onclick={handlePushTags} disabled={pushingTags} title="推送所有 Tags">
      {#if pushingTags}<span class="spinner"></span>{:else}🏷{/if} Push Tags
    </button>
    <button class="btn" onclick={handleFetch} disabled={fetching}>
      {#if fetching}<span class="spinner"></span>{:else}↻{/if} Fetch
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
  .folder-btn {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    color: var(--accent);
    cursor: pointer;
    font-size: 14px;
  }
  .folder-btn:hover { border-color: var(--accent); }
  .folder-icon { font-size: 14px; }
  .multi-repo-btn {
    position: relative;
    display: flex;
    align-items: center;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    color: var(--text-secondary);
    cursor: pointer;
  }
  .multi-repo-btn:hover { border-color: var(--accent); color: var(--accent); }
  .multi-repo-badge {
    position: absolute;
    top: -4px;
    right: -4px;
    min-width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--warning);
    color: var(--bg-primary);
    font-size: 10px;
    font-weight: 700;
    font-family: var(--font-ui);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 3px;
    line-height: 1;
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

  .conflict-badge {
    background: none;
    border: none;
    color: var(--warning);
    font-size: 11px;
    font-family: var(--font-ui);
    font-weight: 600;
    cursor: pointer;
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
  }
  .conflict-badge:hover { background: var(--bg-hover); }
  .branch-icon { font-size: var(--font-size-lg); }
  .chevron { color: var(--text-muted); font-size: 10px; }
  .dropdown-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 49;
  }
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
  .dropdown-divider {
    height: 1px;
    background: var(--border);
    margin: var(--space-xs) 0;
  }
  .manage-link {
    display: block;
    width: 100%;
    padding: var(--space-sm) var(--space-md);
    background: none;
    border: none;
    color: var(--accent);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    cursor: pointer;
    text-align: left;
  }
  .manage-link:hover { background: var(--bg-hover); }
  .external-tools {
    display: flex;
    gap: 2px;
    margin-left: var(--space-sm);
  }
  .tool-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
  }
  .tool-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .tool-btn.active {
    background: var(--accent-subtle);
    color: var(--accent);
  }
  .drag-spacer {
    flex: 1;
    height: 100%;
    min-width: 0;
  }
  .actions {
    display: flex;
    gap: var(--space-xs);
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
