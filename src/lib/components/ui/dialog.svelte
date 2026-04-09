<script lang="ts">
  import { Dialog } from 'bits-ui';
  import { cn } from '$lib/utils/cn';

  interface Props {
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
    title: string;
    description?: string;
    class?: string;
    children?: import('svelte').Snippet;
    footer?: import('svelte').Snippet;
  }

  let {
    open = $bindable(false),
    onOpenChange,
    title,
    description,
    class: className = '',
    children,
    footer,
  }: Props = $props();
</script>

<Dialog.Root bind:open {onOpenChange}>
  <Dialog.Portal>
    <Dialog.Overlay
      class="fixed inset-0 z-[200] bg-black/60 backdrop-blur-[1px] animate-in fade-in-0"
    />

    <Dialog.Content
      class={cn(
        'fixed left-1/2 top-1/2 z-[201] w-full max-w-[480px] -translate-x-1/2 -translate-y-1/2',
        'rounded-md border border-[var(--border)] bg-[var(--bg-secondary)] shadow-2xl',
        'focus:outline-none',
        'animate-in fade-in-0 zoom-in-95',
        className
      )}
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-[var(--border)] px-4 py-3">
        <Dialog.Title class="text-sm font-semibold text-[var(--text-primary)]">
          {title}
        </Dialog.Title>
        {#if description}
          <Dialog.Description class="sr-only">{description}</Dialog.Description>
        {/if}
        <Dialog.Close
          class="flex h-6 w-6 items-center justify-center rounded text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </Dialog.Close>
      </div>

      <!-- Body -->
      <div class="p-4">
        {@render children?.()}
      </div>

      <!-- Footer -->
      {#if footer}
        <div class="flex items-center justify-end gap-2 border-t border-[var(--border)] px-4 py-3">
          {@render footer?.()}
        </div>
      {/if}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
