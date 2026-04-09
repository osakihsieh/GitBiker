<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { cn } from '$lib/utils/cn';

  let expanded = $state<Set<number>>(new Set());

  function toggleExpand(id: number) {
    const next = new Set(expanded);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expanded = next;
  }

  function isLong(message: string): boolean {
    return message.length > 120 || message.includes('\n');
  }

  function summary(message: string): string {
    const firstLine = message.split('\n')[0];
    return firstLine.length > 120 ? firstLine.slice(0, 117) + '…' : firstLine;
  }

  const typeStyles: Record<string, string> = {
    success: 'border-[var(--success)]/40 bg-[var(--success)]/10 text-[var(--success)]',
    error: 'border-[var(--error)]/40 bg-[var(--error)]/10 text-[var(--error)]',
    info: 'border-[var(--accent)]/40 bg-[var(--accent)]/10 text-[var(--accent)]',
  };

  const typeIcons: Record<string, string> = {
    success: '✓',
    error: '✕',
    info: 'ℹ',
  };
</script>

{#if app.toasts.length > 0}
  <div class="fixed bottom-4 right-4 z-[300] flex flex-col gap-2" role="region" aria-label="Notifications">
    {#each app.toasts as toast (toast.id)}
      <div
        class={cn(
          'flex min-w-[220px] max-w-[480px] items-start gap-2.5',
          'rounded-md border px-3 py-2.5 text-xs shadow-lg',
          'animate-in slide-in-from-right-2 fade-in-0',
          typeStyles[toast.type] ?? typeStyles.info
        )}
        role="alert"
        aria-live="polite"
      >
        <!-- Icon -->
        <span class="mt-0.5 shrink-0 font-semibold">{typeIcons[toast.type] ?? 'ℹ'}</span>

        <!-- Body -->
        <span class="flex min-w-0 flex-1 flex-col gap-1">
          {#if isLong(toast.message) && !expanded.has(toast.id)}
            <span class="break-words leading-relaxed">{summary(toast.message)}</span>
            <button
              class="self-start text-[10px] opacity-70 underline hover:opacity-100 bg-transparent border-none cursor-pointer p-0 text-inherit"
              onclick={() => toggleExpand(toast.id)}
            >展開</button>
          {:else if isLong(toast.message) && expanded.has(toast.id)}
            <span class="block max-h-60 overflow-y-auto break-words whitespace-pre-wrap leading-relaxed">{toast.message}</span>
            <button
              class="self-start text-[10px] opacity-70 underline hover:opacity-100 bg-transparent border-none cursor-pointer p-0 text-inherit"
              onclick={() => toggleExpand(toast.id)}
            >收起</button>
          {:else}
            <span class="break-words leading-relaxed">{toast.message}</span>
          {/if}
        </span>

        <!-- Close (only for non-auto-dismiss) -->
        {#if !toast.autoDismiss}
          <button
            class="shrink-0 rounded p-0.5 opacity-60 hover:opacity-100 transition-opacity bg-transparent border-none cursor-pointer text-inherit"
            onclick={() => app.removeToast(toast.id)}
            aria-label="關閉通知"
          >
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        {/if}
      </div>
    {/each}
  </div>
{/if}
