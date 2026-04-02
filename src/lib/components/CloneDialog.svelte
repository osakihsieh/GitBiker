<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { app } from '$lib/stores/app.svelte';

  interface Props {
    onClose: () => void;
    onCloned: (path: string) => void;
  }

  let { onClose, onCloned }: Props = $props();

  let url = $state('');
  let destPath = $state('');
  let cloning = $state(false);
  let progress = $state('');
  let error = $state('');

  async function handleClone() {
    if (!url.trim() || !destPath.trim() || cloning) return;
    error = '';
    cloning = true;
    progress = 'Cloning...';

    try {
      // Use subprocess git clone via a new Tauri command
      await invoke('git_clone', { url: url.trim(), dest: destPath.trim() });
      progress = 'Done!';
      app.addToast('Clone 完成', 'success');
      onCloned(destPath.trim());
    } catch (e: unknown) {
      const msg = String(e);
      if (msg.includes('Authentication') || msg.includes('could not read Username')) {
        error = '認證失敗 — 請確認 URL 和 credential helper 設定';
      } else if (msg.includes('not found') || msg.includes('Repository not found')) {
        error = '找不到 repository — 請確認 URL 是否正確';
      } else {
        error = msg;
      }
    } finally {
      cloning = false;
    }
  }

  function guessDestName(): string {
    if (!url) return '';
    const match = url.match(/\/([^/]+?)(?:\.git)?$/);
    return match ? match[1] : '';
  }

  function handleUrlChange() {
    if (!destPath) {
      const name = guessDestName();
      if (name) {
        // Default to user's home + project name
        destPath = `C:\\Users\\${name}`;
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
    if (e.ctrlKey && e.key === 'Enter') handleClone();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <div class="dialog-header">
      <span class="dialog-title">Clone Repository</span>
      <button class="close-btn" onclick={onClose}>✕</button>
    </div>

    <div class="dialog-body">
      <label class="field">
        <span class="field-label">Repository URL</span>
        <input
          type="text"
          bind:value={url}
          oninput={handleUrlChange}
          placeholder="https://github.com/user/repo.git"
          disabled={cloning}
          autofocus
        />
      </label>

      <label class="field">
        <span class="field-label">Clone to</span>
        <input
          type="text"
          bind:value={destPath}
          placeholder="C:\Users\you\projects\repo"
          disabled={cloning}
        />
      </label>

      {#if error}
        <div class="error-msg">{error}</div>
      {/if}

      {#if cloning}
        <div class="progress">
          <div class="progress-bar">
            <div class="progress-fill"></div>
          </div>
          <span class="progress-text">{progress}</span>
        </div>
      {/if}
    </div>

    <div class="dialog-footer">
      <button class="btn-secondary" onclick={onClose} disabled={cloning}>Cancel</button>
      <button
        class="btn-primary"
        onclick={handleClone}
        disabled={!url.trim() || !destPath.trim() || cloning}
      >
        {#if cloning}Cloning...{:else}Clone{/if}
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    width: 480px;
    max-width: 90vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
  }
  .dialog-title {
    font-size: var(--font-size-lg);
    font-weight: 600;
  }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 16px;
    padding: var(--space-xs);
  }
  .close-btn:hover { color: var(--text-primary); }
  .dialog-body {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .field-label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    font-weight: 500;
  }
  .field input {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    padding: var(--space-sm);
    outline: none;
  }
  .field input:focus { border-color: var(--accent); }
  .field input::placeholder { color: var(--text-muted); }
  .field input:disabled { opacity: 0.5; }
  .error-msg {
    color: var(--error);
    font-size: var(--font-size-sm);
    padding: var(--space-sm);
    background: rgba(255, 107, 107, 0.1);
    border-radius: var(--radius-sm);
  }
  .progress {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .progress-bar {
    height: 4px;
    background: var(--bg-surface);
    border-radius: 2px;
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    animation: indeterminate 1.5s ease-in-out infinite;
    width: 40%;
  }
  @keyframes indeterminate {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(350%); }
  }
  .progress-text {
    font-size: 11px;
    color: var(--text-muted);
  }
  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
    padding: var(--space-md) var(--space-lg);
    border-top: 1px solid var(--border);
  }
  .btn-secondary {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: var(--space-sm) var(--space-lg);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
  }
  .btn-secondary:hover { background: var(--bg-hover); color: var(--text-primary); }
  .btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-primary {
    background: var(--accent);
    border: none;
    color: var(--bg-primary);
    padding: var(--space-sm) var(--space-lg);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    font-weight: 600;
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
