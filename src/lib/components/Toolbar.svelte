<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitPush, gitPull, gitBranches, gitSwitchBranch, openInFolder, openInEditor, openInTerminal } from '$lib/git/commands';

  let branchDropdownOpen = $state(false);
  let pushing = $state(false);
  let pulling = $state(false);

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
    onOpenPopover?: () => void;
  }
  let { onOpenSettings, onOpenPopover }: Props = $props();
</script>

<div class="toolbar">
  <button class="folder-btn" onclick={onOpenPopover} aria-label="Open repo switcher">
    <span class="folder-icon">📁</span><span class="chevron">▾</span>
  </button>

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

  <div class="external-tools">
    <button class="tool-btn" onclick={handleOpenFolder} title="在檔案總管開啟" aria-label="Open in file explorer">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
    </button>
    <button class="tool-btn" onclick={handleOpenEditor} title="在編輯器開啟" aria-label="Open in editor">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
    </button>
    <button class="tool-btn" onclick={handleOpenTerminal} title="開啟終端機" aria-label="Open terminal">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
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
