<script lang="ts">
  import { cn } from '$lib/utils/cn';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  type Variant = 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  type Size = 'default' | 'sm' | 'lg' | 'icon';

  interface Props extends HTMLButtonAttributes {
    variant?: Variant;
    size?: Size;
    loading?: boolean;
  }

  let {
    variant = 'default',
    size = 'default',
    loading = false,
    class: className = '',
    children,
    disabled,
    ...rest
  }: Props = $props();

  const variants: Record<Variant, string> = {
    default: 'bg-[var(--accent)] text-[var(--bg-primary)] hover:brightness-110',
    destructive: 'bg-[var(--error)] text-white hover:brightness-110',
    outline: 'border border-[var(--border)] bg-transparent text-[var(--text-primary)] hover:bg-[var(--bg-hover)]',
    secondary: 'bg-[var(--bg-surface)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] border border-[var(--border)]',
    ghost: 'bg-transparent text-[var(--text-primary)] hover:bg-[var(--bg-hover)]',
    link: 'text-[var(--accent)] underline-offset-4 hover:underline bg-transparent',
  };

  const sizes: Record<Size, string> = {
    default: 'h-8 px-4 py-1.5 text-sm',
    sm: 'h-7 px-3 text-xs',
    lg: 'h-10 px-6 text-sm',
    icon: 'h-8 w-8',
  };
</script>

<button
  class={cn(
    'inline-flex items-center justify-center gap-2 rounded-[var(--radius-sm)] font-medium transition-all',
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]',
    'disabled:opacity-50 disabled:cursor-not-allowed',
    'cursor-pointer font-[var(--font-ui)]',
    variants[variant],
    sizes[size],
    className
  )}
  disabled={disabled || loading}
  {...rest}
>
  {#if loading}
    <span class="inline-block h-3 w-3 rounded-full border-2 border-current border-t-transparent animate-spin"></span>
  {/if}
  {@render children?.()}
</button>
