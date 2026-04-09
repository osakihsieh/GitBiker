<script lang="ts">
  import { app } from '$lib/stores/app.svelte';

  let expanded = $state<Set<number>>(new Set());

  function toggleExpand(id: number) {
    const next = new Set(expanded);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expanded = next;
  }

  function isLong(message: string): boolean {
    return message.length > 120 || message.includes('\n');
  }

  function summary(message: string): string {
    const firstLine = message.split('\n')[0];
    return firstLine.length > 120 ? firstLine.slice(0, 117) + '…' : firstLine;
  }
</script>

{#if app.toasts.length > 0}
  <div class="toast-container">
    {#each app.toasts as toast (toast.id)}
      <div class="toast toast-{toast.type}">
        <span class="toast-icon">
          {#if toast.type === 'success'}✓{:else if toast.type === 'error'}✕{:else}ℹ{/if}
        </span>
        <span class="toast-body">
          {#if isLong(toast.message) && !expanded.has(toast.id)}
            <span class="toast-message">{summary(toast.message)}</span>
            <button class="toast-expand" onclick={() => toggleExpand(toast.id)}>展開</button>
          {:else if isLong(toast.message) && expanded.has(toast.id)}
            <span class="toast-message toast-message--full">{toast.message}</span>
            <button class="toast-expand" onclick={() => toggleExpand(toast.id)}>收起</button>
          {:else}
            <span class="toast-message">{toast.message}</span>
          {/if}
        </span>
        {#if !toast.autoDismiss}
          <button class="toast-close" onclick={() => app.removeToast(toast.id)}>✕</button>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 16px;
    right: 16px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }
  .toast {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    animation: fadeIn 0.2s ease;
    min-width: 200px;
    max-width: 480px;
  }
  .toast-success {
    background: #1a3d2a;
    border: 1px solid var(--success);
    color: var(--success);
  }
  .toast-error {
    background: #3d1a1a;
    border: 1px solid var(--error);
    color: var(--error);
  }
  .toast-info {
    background: var(--bg-surface);
    border: 1px solid var(--accent);
    color: var(--accent);
  }
  .toast-icon { flex-shrink: 0; padding-top: 1px; }
  .toast-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .toast-message {
    word-break: break-word;
    white-space: pre-wrap;
    line-height: 1.4;
  }
  .toast-message--full {
    max-height: 240px;
    overflow-y: auto;
    display: block;
  }
  .toast-expand {
    align-self: flex-start;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0;
    font-size: 11px;
    opacity: 0.7;
    text-decoration: underline;
  }
  .toast-expand:hover { opacity: 1; }
  .toast-close {
    flex-shrink: 0;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 2px;
    font-size: 14px;
  }
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
