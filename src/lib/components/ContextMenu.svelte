<script lang="ts">
  export interface MenuItem {
    id: string;
    label: string;
    shortcut?: string;
    disabled?: boolean;
    separator?: boolean;
  }

  interface Props {
    x: number;
    y: number;
    items: MenuItem[];
    onSelect: (id: string) => void;
    onClose: () => void;
  }

  let { x, y, items, onSelect, onClose }: Props = $props();

  let menuEl: HTMLDivElement | undefined = $state();

  // Boundary correction: if menu overflows viewport bottom, flip up
  let adjustedY = $derived.by(() => {
    if (!menuEl) return y;
    const menuHeight = menuEl.offsetHeight;
    const viewportHeight = window.innerHeight;
    if (y + menuHeight > viewportHeight - 8) {
      return Math.max(8, y - menuHeight);
    }
    return y;
  });

  let adjustedX = $derived.by(() => {
    if (!menuEl) return x;
    const menuWidth = menuEl.offsetWidth;
    const viewportWidth = window.innerWidth;
    if (x + menuWidth > viewportWidth - 8) {
      return Math.max(8, viewportWidth - menuWidth - 8);
    }
    return x;
  });

  function handleItemClick(item: MenuItem) {
    if (item.disabled) return;
    onSelect(item.id);
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="context-overlay"
  onclick={onClose}
  oncontextmenu={(e) => { e.preventDefault(); onClose(); }}
></div>

<div
  class="context-menu"
  bind:this={menuEl}
  style="left:{adjustedX}px;top:{adjustedY}px"
  role="menu"
>
  {#each items as item (item.id)}
    {#if item.separator}
      <div class="context-divider"></div>
    {:else}
      <button
        class="context-item"
        class:disabled={item.disabled}
        role="menuitem"
        aria-disabled={item.disabled}
        onclick={() => handleItemClick(item)}
      >
        <span>{item.label}</span>
        {#if item.shortcut}
          <span class="shortcut">{item.shortcut}</span>
        {/if}
      </button>
    {/if}
  {/each}
</div>

<style>
  .context-overlay {
    position: fixed;
    inset: 0;
    z-index: 99;
  }
  .context-menu {
    position: fixed;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    min-width: 180px;
    max-height: 60vh;
    overflow-y: auto;
    z-index: 100;
    padding: var(--space-xs) 0;
  }
  .context-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 6px var(--space-md);
    font-size: 12px;
    font-family: var(--font-ui);
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
  }
  .context-item:hover:not(.disabled) { background: var(--bg-hover); }
  .context-item.disabled {
    color: var(--text-muted);
    cursor: default;
  }
  .shortcut {
    color: var(--text-muted);
    font-size: 11px;
    font-family: var(--font-mono);
  }
  .context-divider {
    height: 1px;
    background: var(--border);
    margin: var(--space-xs) 0;
  }
</style>
