<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
</script>

{#if app.toasts.length > 0}
  <div class="toast-container">
    {#each app.toasts as toast (toast.id)}
      <div class="toast toast-{toast.type}">
        <span class="toast-icon">
          {#if toast.type === 'success'}✓{:else if toast.type === 'error'}✕{:else}ℹ{/if}
        </span>
        <span class="toast-message">{toast.message}</span>
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
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    animation: fadeIn 0.2s ease;
    min-width: 200px;
    max-width: 400px;
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
  .toast-icon { flex-shrink: 0; }
  .toast-message { flex: 1; }
  .toast-close {
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
