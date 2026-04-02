<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';

  interface Props {
    onOpenPopover?: () => void;
  }

  let { onOpenPopover }: Props = $props();

  let contextMenu = $state<{ tabId: string; x: number; y: number } | null>(null);
  let scrollContainer: HTMLDivElement | undefined = $state();
  let showLeftArrow = $state(false);
  let showRightArrow = $state(false);

  const contextMenuItems: MenuItem[] = [
    { id: 'close', label: 'Close', shortcut: 'Ctrl+W' },
    { id: 'closeOthers', label: 'Close Others' },
    { id: 'closeAll', label: 'Close All' },
    { id: '_sep', label: '', separator: true },
    { id: 'copyPath', label: 'Copy Path' },
  ];

  function handleClick(tabId: string) {
    app.switchTab(tabId);
  }

  function handleMiddleClick(e: MouseEvent, tabId: string) {
    if (e.button === 1) {
      e.preventDefault();
      app.closeTab(tabId);
    }
  }

  function handleClose(e: MouseEvent, tabId: string) {
    e.stopPropagation();
    app.closeTab(tabId);
  }

  function handleContextMenu(e: MouseEvent, tabId: string) {
    e.preventDefault();
    contextMenu = { tabId, x: e.clientX, y: e.clientY };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function handleContextSelect(actionId: string) {
    if (!contextMenu) return;
    const { tabId } = contextMenu;
    switch (actionId) {
      case 'close':
        app.closeTab(tabId);
        break;
      case 'closeOthers':
        app.closeOtherTabs(tabId);
        break;
      case 'closeAll':
        app.closeAllTabs();
        break;
      case 'copyPath': {
        const tab = app.tabs.find((t) => t.id === tabId);
        if (tab) navigator.clipboard.writeText(tab.path);
        break;
      }
    }
  }

  function checkOverflow() {
    if (!scrollContainer) return;
    const { scrollLeft, scrollWidth, clientWidth } = scrollContainer;
    showLeftArrow = scrollLeft > 0;
    showRightArrow = scrollLeft + clientWidth < scrollWidth - 1;
  }

  function scrollBy(delta: number) {
    scrollContainer?.scrollBy({ left: delta, behavior: 'smooth' });
  }

  $effect(() => {
    if (!scrollContainer) return;

    const el = scrollContainer;
    let debounceTimer: ReturnType<typeof setTimeout>;

    const observer = new ResizeObserver(() => {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(checkOverflow, 50);
    });
    observer.observe(el);
    el.addEventListener('scroll', checkOverflow);
    checkOverflow();

    return () => {
      observer.disconnect();
      el.removeEventListener('scroll', checkOverflow);
      clearTimeout(debounceTimer);
    };
  });

  // Auto-scroll active tab into view
  $effect(() => {
    const activeId = app.activeTabId;
    if (!activeId || !scrollContainer) return;
    const activeEl = scrollContainer.querySelector(`[data-tab-id="${activeId}"]`) as HTMLElement | null;
    activeEl?.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
  });
</script>

{#if app.tabs.length > 0}
  <div class="tab-bar" role="tablist" aria-label="Open repositories">
    {#if showLeftArrow}
      <button class="scroll-arrow left" onclick={() => scrollBy(-120)} aria-label="Scroll tabs left">◀</button>
    {/if}

    <div class="tabs-scroll" bind:this={scrollContainer}>
      {#each app.tabs as tab (tab.id)}
        {@const dirty = app.dirtyCount(tab.id) > 0}
        <button
          class="tab"
          class:active={app.activeTabId === tab.id}
          role="tab"
          aria-selected={app.activeTabId === tab.id}
          data-tab-id={tab.id}
          onclick={() => handleClick(tab.id)}
          onauxclick={(e) => handleMiddleClick(e, tab.id)}
          oncontextmenu={(e) => handleContextMenu(e, tab.id)}
        >
          {#if dirty}
            <span class="dot dirty"></span>
          {/if}
          <span class="tab-name">{app.displayName(tab)}</span>
          <span class="branch-hint">({app.tabBranch(tab.id) || '...'})</span>
          <span
            class="close"
            role="button"
            aria-label="Close {tab.name}"
            onclick={(e) => handleClose(e, tab.id)}
          >✕</span>
        </button>
      {/each}
    </div>

    {#if showRightArrow}
      <button class="scroll-arrow right" onclick={() => scrollBy(120)} aria-label="Scroll tabs right">▶</button>
    {/if}

    <button class="tab-add" onclick={onOpenPopover} aria-label="Open new repository">+</button>
  </div>
{/if}

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={contextMenuItems}
    onSelect={handleContextSelect}
    onClose={closeContextMenu}
  />
{/if}

<style>
  .tab-bar {
    display: flex;
    align-items: center;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    height: 28px;
    flex-shrink: 0;
    user-select: none;
    position: relative;
  }
  .tabs-scroll {
    display: flex;
    overflow-x: auto;
    scrollbar-width: none;
    flex: 1;
    min-width: 0;
  }
  .tabs-scroll::-webkit-scrollbar { display: none; }
  .tab {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 0 var(--space-md);
    height: 28px;
    font-size: 12px;
    font-family: var(--font-ui);
    color: var(--text-secondary);
    border: none;
    border-bottom: 2px solid transparent;
    background: none;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    min-width: 0;
  }
  .tab:hover { color: var(--text-primary); background: var(--bg-hover); }
  .tab.active {
    color: var(--text-primary);
    border-bottom-color: var(--accent);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .dot.dirty { background: var(--warning); }
  .tab-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 120px;
  }
  .branch-hint {
    color: var(--text-muted);
    font-size: 11px;
    flex-shrink: 0;
  }
  .close {
    font-size: 10px;
    color: var(--text-muted);
    margin-left: 2px;
    opacity: 0;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    min-width: 20px;
    min-height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .tab:hover .close { opacity: 1; }
  .close:hover { background: var(--bg-hover); color: var(--text-primary); }
  .scroll-arrow {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 28px;
    background: var(--bg-secondary);
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 10px;
    flex-shrink: 0;
  }
  .scroll-arrow:hover { color: var(--text-primary); background: var(--bg-hover); }
  .tab-add {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    margin: 0 var(--space-xs);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
    background: none;
    border: none;
  }
  .tab-add:hover { background: var(--bg-hover); color: var(--text-primary); }
</style>
