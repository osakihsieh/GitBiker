<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitRemoteList, gitRemoteAdd, gitRemoteRemove, gitRemoteRename } from '$lib/git/commands';
  import type { RemoteInfo } from '$lib/git/types';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  // Remote management state
  let remotes = $state<RemoteInfo[]>([]);
  let loadingRemotes = $state(false);
  let showAddForm = $state(false);
  let newRemoteName = $state('');
  let newRemoteUrl = $state('');
  let renamingRemote = $state<string | null>(null);
  let renameValue = $state('');

  async function loadRemotes() {
    if (!app.repoPath) return;
    loadingRemotes = true;
    try {
      remotes = await gitRemoteList(app.repoPath);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      loadingRemotes = false;
    }
  }

  async function handleAddRemote() {
    if (!app.repoPath || !newRemoteName.trim() || !newRemoteUrl.trim()) return;
    try {
      await gitRemoteAdd(app.repoPath, newRemoteName.trim(), newRemoteUrl.trim());
      app.addToast(`已新增 remote: ${newRemoteName.trim()}`, 'success');
      newRemoteName = '';
      newRemoteUrl = '';
      showAddForm = false;
      await loadRemotes();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleRemoveRemote(name: string) {
    if (!app.repoPath) return;
    try {
      await gitRemoteRemove(app.repoPath, name);
      app.addToast(`已移除 remote: ${name}`, 'success');
      await loadRemotes();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  function startRename(name: string) {
    renamingRemote = name;
    renameValue = name;
  }

  async function handleRenameRemote() {
    if (!app.repoPath || !renamingRemote || !renameValue.trim()) return;
    try {
      await gitRemoteRename(app.repoPath, renamingRemote, renameValue.trim());
      app.addToast(`已重新命名 remote: ${renamingRemote} → ${renameValue.trim()}`, 'success');
      renamingRemote = null;
      await loadRemotes();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  // Load remotes when settings opens and repo is available
  $effect(() => {
    if (app.repoPath) loadRemotes();
  });

  const shortcuts = [
    { keys: 'Ctrl+Enter', action: 'Commit' },
    { keys: 'Ctrl+Shift+P', action: '命令面板（Coming soon）' },
    { keys: 'Ctrl+1', action: '聚焦 File Tree' },
    { keys: 'Ctrl+2', action: '聚焦 Diff Viewer' },
    { keys: 'Ctrl+3', action: '聚焦 Commit History' },
    { keys: 'Escape', action: '關閉 Dialog / Settings' },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings">
  <div class="settings-header">
    <button class="back-btn" onclick={onClose}>← Back</button>
    <span class="settings-title">Settings</span>
  </div>

  <div class="settings-body">
    <div class="section">
      <div class="section-title">Appearance</div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Theme</span>
          <span class="setting-desc">
            {#if app.theme === 'system'}
              跟隨系統主題（目前：{app.resolvedTheme === 'dark' ? '深色' : '淺色'}）
            {:else if app.theme === 'dark'}
              手動深色主題
            {:else}
              手動淺色主題
            {/if}
          </span>
        </div>
        <div class="theme-segmented" role="radiogroup" aria-label="Theme">
          {#each [
            { value: 'system' as const, label: '⚙ System' },
            { value: 'dark' as const, label: '☽ Dark' },
            { value: 'light' as const, label: '☀ Light' },
          ] as option}
            <button
              class="theme-option"
              class:active={app.theme === option.value}
              role="radio"
              aria-checked={app.theme === option.value}
              onclick={() => app.setTheme(option.value)}
            >
              {option.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <div class="section">
      <div class="section-title">Keyboard Shortcuts</div>
      <div class="shortcuts-list">
        {#each shortcuts as shortcut}
          <div class="shortcut-row">
            <kbd class="shortcut-keys">{shortcut.keys}</kbd>
            <span class="shortcut-action">{shortcut.action}</span>
          </div>
        {/each}
      </div>
    </div>

    {#if app.hasRepo}
    <div class="section">
      <div class="section-title">Repository — Remotes</div>
      {#if loadingRemotes}
        <div class="remote-loading">Loading...</div>
      {:else}
        <div class="remote-list">
          {#each remotes as remote (remote.name)}
            <div class="remote-item">
              {#if renamingRemote === remote.name}
                <div class="remote-rename-form">
                  <input
                    type="text"
                    class="remote-input"
                    bind:value={renameValue}
                    onkeydown={(e) => e.key === 'Enter' && handleRenameRemote()}
                  />
                  <button class="remote-action-btn" onclick={handleRenameRemote}>Save</button>
                  <button class="remote-action-btn" onclick={() => renamingRemote = null}>Cancel</button>
                </div>
              {:else}
                <div class="remote-info">
                  <span class="remote-name">{remote.name}</span>
                  <span class="remote-url">{remote.url}</span>
                </div>
                <div class="remote-actions">
                  <button class="remote-action-btn" onclick={() => startRename(remote.name)}>Rename</button>
                  <button class="remote-action-btn danger" onclick={() => handleRemoveRemote(remote.name)}>Remove</button>
                </div>
              {/if}
            </div>
          {:else}
            <div class="remote-empty">No remotes configured. Add one to push and pull.</div>
          {/each}
        </div>

        {#if showAddForm}
          <div class="add-remote-form">
            <input type="text" class="remote-input" placeholder="Name (e.g. origin)" bind:value={newRemoteName} />
            <input type="text" class="remote-input" placeholder="URL (https:// or git@...)" bind:value={newRemoteUrl} />
            <div class="add-remote-actions">
              <button class="remote-action-btn" onclick={() => { showAddForm = false; newRemoteName = ''; newRemoteUrl = ''; }}>Cancel</button>
              <button class="remote-action-btn primary" onclick={handleAddRemote} disabled={!newRemoteName.trim() || !newRemoteUrl.trim()}>Add</button>
            </div>
          </div>
        {:else}
          <button class="add-remote-btn" onclick={() => showAddForm = true}>+ Add Remote</button>
        {/if}
      {/if}
    </div>
    {/if}

    <div class="section">
      <div class="section-title">About</div>
      <div class="about-info">
        <div class="about-row">
          <span class="about-label">Version</span>
          <span class="about-value">0.1.0</span>
        </div>
        <div class="about-row">
          <span class="about-label">Framework</span>
          <span class="about-value">Tauri 2.x + Svelte 5</span>
        </div>
        <div class="about-row">
          <span class="about-label">License</span>
          <span class="about-value">MIT</span>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }
  .settings-header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .back-btn {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--font-size-md);
  }
  .back-btn:hover { text-decoration: underline; }
  .settings-title {
    font-size: var(--font-size-lg);
    font-weight: 600;
  }
  .settings-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
    max-width: 600px;
    margin: 0 auto;
    width: 100%;
  }
  .section {
    margin-bottom: 32px;
  }
  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    margin-bottom: var(--space-md);
    padding-bottom: var(--space-xs);
    border-bottom: 1px solid var(--border);
  }
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) 0;
  }
  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .setting-label { font-size: var(--font-size-md); }
  .setting-desc { font-size: 11px; color: var(--text-muted); }
  .theme-segmented {
    display: flex;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .theme-option {
    flex: 1;
    padding: 6px 12px;
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    white-space: nowrap;
  }
  .theme-option:hover:not(.active) {
    background: var(--bg-hover);
  }
  .theme-option.active {
    background: var(--accent);
    color: var(--bg-primary);
    font-weight: 600;
  }
  .theme-option:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) 0;
  }
  .shortcut-keys {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
  }
  .shortcut-action {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  .about-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }
  .about-row {
    display: flex;
    justify-content: space-between;
    font-size: var(--font-size-sm);
  }
  .about-label { color: var(--text-secondary); }
  .about-value { color: var(--text-primary); font-family: var(--font-mono); }

  /* Remote Management */
  .remote-loading {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    padding: var(--space-sm) 0;
  }
  .remote-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .remote-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .remote-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }
  .remote-name {
    font-size: var(--font-size-md);
    font-weight: 600;
    color: var(--text-primary);
  }
  .remote-url {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .remote-actions {
    display: flex;
    gap: var(--space-xs);
    flex-shrink: 0;
  }
  .remote-action-btn {
    font-size: 11px;
    color: var(--accent);
    background: none;
    border: none;
    cursor: pointer;
    font-family: var(--font-ui);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }
  .remote-action-btn:hover { background: var(--bg-hover); }
  .remote-action-btn.danger { color: var(--error); }
  .remote-action-btn.danger:hover { background: rgba(255, 107, 107, 0.1); }
  .remote-action-btn.primary {
    background: var(--accent);
    color: var(--bg-primary);
  }
  .remote-action-btn.primary:hover { filter: brightness(1.1); }
  .remote-action-btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .remote-rename-form {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
  }
  .remote-input {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    padding: var(--space-xs) var(--space-sm);
    outline: none;
  }
  .remote-input:focus { border-color: var(--accent); }
  .remote-input::placeholder { color: var(--text-muted); }
  .remote-empty {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    padding: var(--space-md) 0;
    text-align: center;
  }
  .add-remote-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    margin-top: var(--space-sm);
    padding: var(--space-sm);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .add-remote-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-xs);
  }
  .add-remote-btn {
    margin-top: var(--space-sm);
    font-size: var(--font-size-sm);
    color: var(--accent);
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    cursor: pointer;
    font-family: var(--font-ui);
  }
  .add-remote-btn:hover { border-color: var(--accent); background: var(--bg-hover); }
</style>
