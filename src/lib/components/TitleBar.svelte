<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { app } from '$lib/stores/app.svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    children?: Snippet;
  }
  let { children }: Props = $props();

  let maximized = $state(false);

  const appWindow = getCurrentWindow();

  $effect(() => {
    let cleanup: (() => void) | undefined;

    appWindow.isMaximized().then((v) => {
      maximized = v;
    });
    appWindow
      .onResized(() => {
        appWindow.isMaximized().then((v) => {
          maximized = v;
        });
      })
      .then((unlisten) => {
        cleanup = unlisten;
      });

    return () => {
      cleanup?.();
    };
  });

  function minimize() {
    appWindow.minimize();
  }
  function toggleMaximize() {
    appWindow.toggleMaximize();
  }
  function close() {
    appWindow.close();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="titlebar" class:is-mac={app.isMac}>
  {#if app.isMac}
    <div class="titlebar-controls mac-controls">
      <button class="control-btn mac-close" onclick={close} aria-label="Close"></button>
      <button class="control-btn mac-minimize" onclick={minimize} aria-label="Minimize"></button>
      <button class="control-btn mac-maximize" onclick={toggleMaximize} aria-label="Maximize"></button>
    </div>
  {/if}

  {#if !app.isMac}
    <div class="titlebar-left" data-tauri-drag-region>
      <img src="/GitBiker.png" alt="GitBiker" class="app-icon" />
      <span class="app-name">GitBiker</span>
    </div>
  {/if}

  <div class="titlebar-tabs" data-tauri-drag-region ondblclick={toggleMaximize}>
    {@render children?.()}
  </div>

  {#if !app.isMac}
    <div class="titlebar-controls">
      <button class="control-btn" onclick={minimize} aria-label="Minimize">
        <svg width="12" height="12" viewBox="0 0 12 12"
          ><rect x="2" y="6" width="8" height="1" fill="currentColor" /></svg
        >
      </button>
      <button
        class="control-btn"
        onclick={toggleMaximize}
        aria-label={maximized ? 'Restore' : 'Maximize'}
      >
        {#if maximized}
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect
              x="3"
              y="1"
              width="8"
              height="8"
              rx="0.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1"
            />
            <rect
              x="1"
              y="3"
              width="8"
              height="8"
              rx="0.5"
              fill="var(--bg-secondary)"
              stroke="currentColor"
              stroke-width="1"
            />
          </svg>
        {:else}
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect
              x="1.5"
              y="1.5"
              width="9"
              height="9"
              rx="0.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1"
            />
          </svg>
        {/if}
      </button>
      <button class="control-btn close-btn" onclick={close} aria-label="Close">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.2" />
          <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
    </div>
  {/if}
</div>

<style>
  .titlebar {
    display: flex;
    align-items: stretch;
    height: 38px;
    background: var(--bg);
    border-bottom: 1px solid var(--color-ink-10);
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
    padding: 0 12px;
  }
  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-right: 16px;
    flex-shrink: 0;
    cursor: default;
  }
  .app-icon {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    object-fit: contain;
  }
  .app-name {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--ink);
    letter-spacing: -0.2px;
  }
  .titlebar-tabs {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .titlebar-controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .mac-controls {
    gap: 8px;
  }
  .mac-controls .control-btn {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: none;
    padding: 0;
    transition: opacity 0.2s;
  }
  .mac-controls .control-btn:hover { opacity: 0.8; }
  .mac-close { background: #ff5f56; }
  .mac-minimize { background: #ffbd2e; }
  .mac-maximize { background: #27c93f; }

  .control-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: none;
    border: none;
    color: var(--color-ink-50);
    cursor: pointer;
    border-radius: 6px;
    transition: background 0.2s, color 0.2s;
  }
  .control-btn:hover {
    background: var(--color-ink-05);
    color: var(--ink);
  }
  .close-btn:hover {
    background: var(--color-warn-bg);
    color: var(--color-warn);
  }
</style>
