<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import {
    gitStashList,
    gitStashPush,
    gitStashPop,
    gitStashApply,
    gitStashDrop,
  } from '$lib/git/commands';
  import { clickOutside } from '$lib/actions/clickOutside';
  import type { StashEntry } from '$lib/git/types';

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  let stashes = $state<StashEntry[]>([]);
  let loading = $state(false);
  let pushMessage = $state('');
  let pushing = $state(false);
  let showPushForm = $state(false);

  async function loadStashes() {
    if (!app.repoPath) return;
    loading = true;
    try {
      stashes = await gitStashList(app.repoPath);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open) loadStashes();
  });

  async function handlePush() {
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
      app.addToast(String(e), 'error');
    } finally {
      pushing = false;
    }
  }

  async function handlePop(index: number) {
    if (!app.repoPath) return;
    try {
      await gitStashPop(app.repoPath, index);
      app.addToast('已 pop stash', 'success');
      await Promise.all([loadStashes(), app.refreshAll()]);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleApply(index: number) {
    if (!app.repoPath) return;
    try {
      await gitStashApply(app.repoPath, index);
      app.addToast('已 apply stash', 'success');
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleDrop(index: number) {
    if (!app.repoPath) return;
    try {
      await gitStashDrop(app.repoPath, index);
      app.addToast('已刪除 stash', 'success');
      await loadStashes();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }
</script>

{#if open}
  <div class="stash-manager" use:clickOutside={onClose} role="dialog" aria-label="Stash manager">
    <div class="stash-header">
      <span class="stash-title">Stash</span>
      {#if !showPushForm}
        <button class="btn-stash-push" onclick={() => (showPushForm = true)}>+ Stash All</button>
      {/if}
    </div>

    {#if showPushForm}
      <div class="push-form">
        <input
          type="text"
          placeholder="Stash message (optional)..."
          bind:value={pushMessage}
          class="push-input"
          autofocus
          onkeydown={(e) => {
            if (e.key === 'Enter') handlePush();
            if (e.key === 'Escape') { showPushForm = false; pushMessage = ''; }
          }}
        />
        <div class="push-actions">
          <button class="btn-create" onclick={handlePush} disabled={pushing}>
            {#if pushing}<span class="spinner"></span>{:else}Stash{/if}
          </button>
          <button class="btn-text" onclick={() => { showPushForm = false; pushMessage = ''; }}>取消</button>
        </div>
      </div>
    {/if}

    <div class="stash-list">
      {#if loading}
        <div class="empty-state"><span class="spinner"></span></div>
      {:else if stashes.length === 0}
        <div class="empty-state">No stashes</div>
      {:else}
        {#each stashes as stash}
          <div class="stash-item">
            <div class="stash-info">
              <span class="stash-index">stash@&#123;{stash.index}&#125;</span>
              <span class="stash-message">{stash.message}</span>
            </div>
            <div class="stash-actions">
              <button class="action-btn" title="Pop (apply + drop)" onclick={() => handlePop(stash.index)}>Pop</button>
              <button class="action-btn" title="Apply (keep stash)" onclick={() => handleApply(stash.index)}>Apply</button>
              <button class="action-btn delete-btn" title="Drop" onclick={() => handleDrop(stash.index)}>×</button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}

<style>
  .stash-manager {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: var(--space-xs);
    width: 340px;
    max-height: 50vh;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 60;
    display: flex;
    flex-direction: column;
    animation: popoverIn 0.15s ease-out;
  }

  @keyframes popoverIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .stash-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }
  .stash-title {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
  }
  .btn-stash-push {
    background: none;
    border: none;
    color: var(--accent);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    cursor: pointer;
  }
  .btn-stash-push:hover { text-decoration: underline; }

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

  .stash-list {
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .stash-item {
    display: flex;
    align-items: center;
    padding: var(--space-sm) var(--space-md);
    gap: var(--space-sm);
    border-bottom: 1px solid var(--border-subtle, rgba(255,255,255,0.05));
  }
  .stash-item:hover { background: var(--bg-hover); }
  .stash-item:hover .stash-actions { opacity: 1; }

  .stash-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .stash-index {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }
  .stash-message {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stash-actions {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.1s;
    flex-shrink: 0;
  }
  .action-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }
  .action-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .delete-btn:hover { color: var(--error); }

  .empty-state {
    padding: var(--space-lg) var(--space-md);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
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
