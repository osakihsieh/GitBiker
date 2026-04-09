<script lang="ts">
  import { cn } from '$lib/utils/cn';
  import type { HTMLInputAttributes } from 'svelte/elements';

  interface Props extends Omit<HTMLInputAttributes, 'value'> {
    value?: string | number;
    error?: boolean;
  }

  let { class: className = '', error = false, value = $bindable(''), ...rest }: Props = $props();
</script>

<input
  bind:value
  class={cn(
    'flex h-8 w-full rounded-[var(--radius-sm)] border border-[var(--border)]',
    'bg-[var(--bg-surface)] px-3 py-1.5 text-sm text-[var(--text-primary)]',
    'font-[var(--font-mono)] placeholder:text-[var(--text-muted)]',
    'transition-colors outline-none',
    'focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]',
    'disabled:opacity-50 disabled:cursor-not-allowed',
    error && 'border-[var(--error)]',
    className
  )}
  {...rest}
/>
