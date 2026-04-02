<script lang="ts">
  import { app } from '$lib/stores/app.svelte';

  interface Props {
    onOpenRepo: (path: string) => void;
    onClone: () => void;
  }

  let { onOpenRepo, onClone }: Props = $props();

  async function handleOpenLocal() {
    try {
      // Use Tauri dialog to pick a folder
      const { invoke } = await import('@tauri-apps/api/core');
      // For now, prompt user for path via a simple approach
      // TODO: Replace with tauri-plugin-dialog folder picker
      const path = prompt('Enter repo path:');
      if (path) {
        onOpenRepo(path);
      }
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }
</script>

<div class="welcome">
  <div class="logo">
    <svg class="logo-icon" viewBox="0 0 64 64" fill="none">
      <circle cx="16" cy="44" r="10" stroke="var(--accent)" stroke-width="2" fill="none"/>
      <circle cx="48" cy="44" r="10" stroke="var(--accent)" stroke-width="2" fill="none"/>
      <path d="M16 44 L28 24 L40 44 L48 44" stroke="var(--accent)" stroke-width="2" fill="none" stroke-linejoin="round"/>
      <path d="M28 24 L40 24" stroke="var(--accent)" stroke-width="2"/>
      <circle cx="28" cy="24" r="2" fill="var(--accent)"/>
      <path d="M38 12 L44 12" stroke="var(--text-primary)" stroke-width="1.5" opacity="0.4"/>
      <circle cx="36" cy="12" r="2" fill="var(--text-primary)" opacity="0.4"/>
      <circle cx="46" cy="12" r="2" fill="var(--text-primary)" opacity="0.4"/>
    </svg>
    <div class="logo-text">GitBiker</div>
    <div class="logo-sub">fast. minimal. yours.</div>
  </div>

  <div class="actions">
    <button class="action-btn" onclick={onClone}>
      <span class="icon">⇣</span>
      <span class="label">Clone a Repo</span>
      <span class="hint">from URL</span>
    </button>
    <button class="action-btn" onclick={handleOpenLocal}>
      <span class="icon">📂</span>
      <span class="label">Open Local Repo</span>
      <span class="hint">from disk</span>
    </button>
  </div>

  <div class="recent">
    <div class="recent-header">Recent Repos</div>
    {#if app.recentRepos.length > 0}
      <div class="recent-list">
        {#each app.recentRepos as repoPath}
          <button class="recent-item" onclick={() => onOpenRepo(repoPath)}>
            <span class="repo-icon">📁</span>
            <div class="repo-info">
              <div class="repo-name">{repoPath.replace(/\\/g, '/').split('/').pop()}</div>
              <div class="repo-path">{repoPath}</div>
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <div class="recent-empty">
        No repos yet. Clone or open one to get started.
      </div>
    {/if}
  </div>

  <div class="version">v0.1.0</div>
</div>

<style>
  .welcome {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 32px;
    padding: 48px;
    height: 100%;
  }
  .logo {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .logo-icon { width: 64px; height: 64px; }
  .logo-text {
    font-size: 24px;
    font-weight: 300;
    letter-spacing: 2px;
  }
  .logo-sub {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    letter-spacing: 1px;
  }
  .actions { display: flex; gap: 16px; }
  .action-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 20px 32px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    cursor: pointer;
    font-family: var(--font-ui);
    min-width: 160px;
    transition: border-color 0.15s, background 0.15s;
  }
  .action-btn:hover {
    border-color: var(--accent);
    background: var(--bg-hover);
  }
  .icon { font-size: 24px; color: var(--accent); }
  .label { font-size: var(--font-size-lg); font-weight: 500; }
  .hint { font-size: 11px; color: var(--text-muted); }
  .recent { width: 100%; max-width: 480px; }
  .recent-header {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    margin-bottom: var(--space-sm);
    padding-bottom: var(--space-xs);
    border-bottom: 1px solid var(--border);
  }
  .recent-empty {
    text-align: center;
    padding: 24px;
    color: var(--text-muted);
    font-size: var(--font-size-md);
  }
  .recent-list { display: flex; flex-direction: column; }
  .recent-item {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    cursor: pointer;
    border-radius: var(--radius-sm);
    background: none;
    border: none;
    color: var(--text-primary);
    font-family: var(--font-ui);
    text-align: left;
    width: 100%;
  }
  .recent-item:hover { background: var(--bg-hover); }
  .repo-icon { font-size: 16px; flex-shrink: 0; }
  .repo-info { flex: 1; min-width: 0; }
  .repo-name { font-size: var(--font-size-md); font-weight: 500; }
  .repo-path {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .version {
    position: fixed;
    bottom: 12px;
    font-size: 10px;
    color: var(--text-muted);
    opacity: 0.4;
  }
</style>
