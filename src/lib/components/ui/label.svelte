<script lang="ts">
  import { cn } from '$lib/utils/cn';
  import type { HTMLLabelAttributes } from 'svelte/elements';

  interface Props extends HTMLLabelAttributes {}
  let { class: className = '', children, ...rest }: Props = $props();
</script>

<label
  class={cn(
    'text-xs font-medium text-[var(--text-secondary)]',
    className
  )}
  {...rest}
>
  {@render children?.()}
</label>
