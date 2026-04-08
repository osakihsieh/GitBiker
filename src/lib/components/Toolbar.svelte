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

  // Derive repo display name from path
  let repoName = $derived(
    app.repoPath
      ? app.repoPath.split(/[/\\]/).filter(Boolean).pop() ?? app.repoPath
      : 'No repo'
  );

  async function handleOpenFolder() {
    if (!app.repoPath) return;
    try { await openInFolder(app.repoPath); }
    catch (e: unknown) { app.addToast(String(e), 'error'); }
  }

  async function handleOpenEditor() {
    if (!app.repoPath) return;
    try { await openInEditor(app.repoPath, app.preferredEditor ?? undefined); }
    catch (e: unknown) { app.addToast(String(e), 'error'); }
  }

  async function handleOpenTerminal() {
    if (!app.repoPath) return;
    try { await openInTerminal(app.repoPath); }
    catch (e: unknown) { app.addToast(String(e), 'error'); }
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
      gitBranches(app.repoPath).then((b) => { app.branches = b; });
    }
  }

  function closeBranchDropdown() { branchDropdownOpen = false; }

  function openBranchManager() {
    branchDropdownOpen = false;
    branchManagerOpen = true;
    if (app.repoPath) {
      gitBranches(app.repoPath).then((b) => { app.branches = b; });
    }
  }

  function closeBranchManager() { branchManagerOpen = false; }

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
  <!-- Left: repo + branch info -->
  <div class="left-section">
    <button class="multi-repo-btn" onclick={onOpenMultiRepo} aria-label="Multi-repo manager (Ctrl+M)" title="多倉庫管理 (Ctrl+M)">
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

    <div class="toolbar-sep"></div>

    <button class="repo-btn" onclick={onOpenPopover} aria-label="Open repo switcher (Ctrl+T)" title="切換倉庫 (Ctrl+T)">
      <span class="info-label">repository</span>
      <span class="info-value">
        <span class="info-name">{repoName}</span>
        <span class="chevron">▾</span>
      </span>
    </button>

    <div class="toolbar-sep"></div>

    <div class="branch-wrapper">
      <button class="branch-btn" onclick={toggleBranchDropdown} aria-label="Switch branch">
        <span class="info-label">branch</span>
        <span class="info-value">
          <span class="branch-icon">⑇</span>
          <span class="info-name">{app.currentBranch || 'main'}</span>
          <span class="chevron">▾</span>
        </span>
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
          <button class="manage-link" onclick={openBranchManager}>管理分支...</button>
        </div>
      {/if}

      {#if branchManagerOpen}
        <BranchManager open={branchManagerOpen} onClose={closeBranchManager} />
      {/if}
    </div>

    {#if app.conflictFiles.length > 0}
      <button
        class="conflict-badge"
        onclick={() => app.isInConflictMode ? app.exitConflictMode() : app.enterConflictMode()}
        title="Ctrl+Shift+M"
      >
        ⚠ {app.conflictFiles.length}
      </button>
    {/if}
  </div>

  <div class="toolbar-sep tall"></div>

  <!-- Middle: external tools + git actions -->
  <div class="mid-section">
    <div class="tool-group">
      <button class="tool-btn" onclick={handleOpenFolder} title="在檔案總管開啟 (Alt+O)" aria-label="Open in file explorer">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
      </button>
      <button class="tool-btn" onclick={handleOpenEditor} title="在編輯器開啟 (Alt+E)" aria-label="Open in editor">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
      </button>
      <button class="tool-btn" onclick={handleOpenTerminal} title="開啟外部終端機 (Alt+T)" aria-label="Open external terminal">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
      </button>
      <button class="tool-btn" class:active={app.showTerminal} onclick={() => app.toggleTerminal()} title="切換內建終端機 (Ctrl+`)" aria-label="Toggle inline terminal">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><path d="M6 18l2-2-2-2"/><line x1="12" y1="18" x2="16" y2="18"/></svg>
      </button>
    </div>

    <div class="toolbar-sep"></div>

    <div class="actions">
      <button class="action-btn" onclick={handlePull} disabled={pulling} title="Git Pull">
        <span class="action-icon">{#if pulling}<span class="spinner"></span>{:else}↓{/if}</span>
        <span class="action-label">Pull</span>
      </button>
      <button class="action-btn" onclick={handlePush} disabled={pushing} title="Git Push">
        <span class="action-icon">{#if pushing}<span class="spinner"></span>{:else}↑{/if}</span>
        <span class="action-label">Push</span>
      </button>
      <button class="action-btn" onclick={handlePushTags} disabled={pushingTags} title="推送所有 Tags">
        <span class="action-icon">{#if pushingTags}<span class="spinner"></span>{:else}🏷{/if}</span>
        <span class="action-label">Tags</span>
      </button>
      <button class="action-btn" onclick={handleFetch} disabled={fetching} title="Git Fetch">
        <span class="action-icon">{#if fetching}<span class="spinner"></span>{:else}↻{/if}</span>
        <span class="action-label">Fetch</span>
      </button>
    </div>
  </div>

  <div class="drag-spacer" data-tauri-drag-region></div>

  <button class="settings-btn" onclick={onOpenSettings} title="設定" aria-label="Settings">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
    </svg>
  </button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0;
    padding: 0 var(--space-sm);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    height: 48px;
    flex-shrink: 0;
    user-select: none;
  }

  /* ── Left section: repo + branch ── */
  .left-section {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    flex-shrink: 0;
  }

  .repo-btn,
  .branch-btn {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1px;
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    padding: 3px var(--space-sm);
    cursor: pointer;
    min-width: 80px;
  }
  .repo-btn:hover,
  .branch-btn:hover {
    background: var(--bg-hover);
    border-color: var(--border);
  }

  .info-label {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-ui);
    line-height: 1;
    text-transform: lowercase;
    letter-spacing: 0.02em;
  }
  .info-value {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 13px;
    font-family: var(--font-ui);
    color: var(--text-primary);
    line-height: 1.2;
  }
  .info-name {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .branch-icon {
    color: var(--accent);
    font-size: 13px;
  }
  .chevron {
    color: var(--text-muted);
    font-size: 9px;
  }

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

  .multi-repo-btn {
    position: relative;
    display: flex;
    align-items: center;
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    color: var(--text-muted);
    cursor: pointer;
  }
  .multi-repo-btn:hover { border-color: var(--border); color: var(--accent); background: var(--bg-hover); }
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

  /* ── Branch dropdown ── */
  .branch-wrapper { position: relative; }
  .dropdown-overlay {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
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
  .dropdown-divider { height: 1px; background: var(--border); margin: var(--space-xs) 0; }
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

  /* ── Separators ── */
  .toolbar-sep {
    width: 1px;
    height: 20px;
    background: var(--border);
    margin: 0 var(--space-xs);
    flex-shrink: 0;
  }
  .toolbar-sep.tall { height: 32px; }

  /* ── Mid section ── */
  .mid-section {
    display: flex;
    align-items: center;
    gap: 0;
    flex-shrink: 0;
  }

  /* ── Tool buttons (icon only) ── */
  .tool-group {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 0 var(--space-xs);
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
  .tool-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .tool-btn.active { background: var(--accent-subtle); color: var(--accent); }

  /* ── Action buttons (icon + label) ── */
  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 0 var(--space-xs);
  }
  .action-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    padding: 3px 10px;
    cursor: pointer;
    min-width: 44px;
  }
  .action-btn:hover {
    background: var(--bg-hover);
    border-color: var(--border);
  }
  .action-btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .action-icon {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 16px;
  }
  .action-label {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-ui);
    line-height: 1;
    white-space: nowrap;
  }
  .action-btn:hover .action-icon { color: var(--text-primary); }
  .action-btn:hover .action-label { color: var(--text-secondary); }

  /* ── Drag spacer ── */
  .drag-spacer { flex: 1; height: 100%; min-width: 0; }

  /* ── Settings button ── */
  .settings-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
    flex-shrink: 0;
  }
  .settings-btn:hover { background: var(--bg-hover); color: var(--text-primary); }

  /* ── Spinner ── */
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
