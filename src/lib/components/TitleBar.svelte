<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import type { Snippet } from 'svelte';

  interface Props {
    children?: Snippet;
  }
  let { children }: Props = $props();

  let maximized = $state(false);

  const appWindow = getCurrentWindow();

  $effect(() => {
    let cleanup: (() => void) | undefined;

    appWindow.isMaximized().then((v) => { maximized = v; });
    appWindow.onResized(() => {
      appWindow.isMaximized().then((v) => { maximized = v; });
    }).then((unlisten) => { cleanup = unlisten; });

    return () => { cleanup?.(); };
  });

  function minimize() { appWindow.minimize(); }
  function toggleMaximize() { appWindow.toggleMaximize(); }
  function close() { appWindow.close(); }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="titlebar">
  <div class="titlebar-left" data-tauri-drag-region>
    <img src="/GitBiker.png" alt="GitBiker" class="app-icon" />
    <span class="app-name">GitBiker</span>
  </div>

  <div class="titlebar-tabs" data-tauri-drag-region ondblclick={toggleMaximize}>
    {@render children?.()}
  </div>

  <div class="titlebar-controls">
    <button class="control-btn" onclick={minimize} aria-label="Minimize">
      <svg width="12" height="12" viewBox="0 0 12 12"><rect x="2" y="6" width="8" height="1" fill="currentColor"/></svg>
    </button>
    <button class="control-btn" onclick={toggleMaximize} aria-label={maximized ? 'Restore' : 'Maximize'}>
      {#if maximized}
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect x="3" y="1" width="8" height="8" rx="0.5" fill="none" stroke="currentColor" stroke-width="1"/>
          <rect x="1" y="3" width="8" height="8" rx="0.5" fill="var(--bg-secondary)" stroke="currentColor" stroke-width="1"/>
        </svg>
      {:else}
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect x="1.5" y="1.5" width="9" height="9" rx="0.5" fill="none" stroke="currentColor" stroke-width="1"/>
        </svg>
      {/if}
    </button>
    <button class="control-btn close-btn" onclick={close} aria-label="Close">
      <svg width="12" height="12" viewBox="0 0 12 12">
        <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.2"/>
        <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.2"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .titlebar {
    display: flex;
    align-items: stretch;
    height: 32px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
  }
  .titlebar-left {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 0 var(--space-md) 0 var(--space-sm);
    flex-shrink: 0;
    cursor: default;
  }
  .app-icon {
    width: 16px;
    height: 16px;
    object-fit: contain;
    flex-shrink: 0;
  }
  .app-name {
    font-size: 12px;
    color: var(--text-muted);
    font-family: var(--font-ui);
    white-space: nowrap;
  }
  .titlebar-tabs {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: stretch;
    overflow: hidden;
  }
  .titlebar-controls {
    display: flex;
    flex-shrink: 0;
  }
  .control-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 32px;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
  }
  .control-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .close-btn:hover {
    background: var(--error);
    color: white;
  }
</style>
