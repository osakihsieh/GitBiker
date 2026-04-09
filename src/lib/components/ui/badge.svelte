<script lang="ts">
  import { cn } from '$lib/utils/cn';

  type Variant = 'default' | 'success' | 'warning' | 'error' | 'muted';

  interface Props {
    variant?: Variant;
    class?: string;
    children?: import('svelte').Snippet;
  }

  let { variant = 'default', class: className = '', children }: Props = $props();

  const variants: Record<Variant, string> = {
    default: 'bg-[var(--accent)]/20 text-[var(--accent)] border-[var(--accent)]/30',
    success: 'bg-[var(--success)]/20 text-[var(--success)] border-[var(--success)]/30',
    warning: 'bg-[var(--warning)]/20 text-[var(--warning)] border-[var(--warning)]/30',
    error: 'bg-[var(--error)]/20 text-[var(--error)] border-[var(--error)]/30',
    muted: 'bg-[var(--bg-surface)] text-[var(--text-muted)] border-[var(--border)]',
  };
</script>

<span
  class={cn(
    'inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-semibold',
    variants[variant],
    className
  )}
>
  {@render children?.()}
</span>
