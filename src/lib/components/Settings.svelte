<script lang="ts">
  import { app } from '$lib/stores/app.svelte';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  const shortcuts = [
    { keys: 'Ctrl+Enter', action: 'Commit' },
    { keys: 'Ctrl+Shift+P', action: '命令面板（Coming soon）' },
    { keys: 'Ctrl+1', action: '聚焦 File Tree' },
    { keys: 'Ctrl+2', action: '聚焦 Diff Viewer' },
    { keys: 'Ctrl+3', action: '聚焦 Commit History' },
    { keys: 'Escape', action: '關閉 Dialog / Settings' },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings">
  <div class="settings-header">
    <button class="back-btn" onclick={onClose}>← Back</button>
    <span class="settings-title">Settings</span>
  </div>

  <div class="settings-body">
    <div class="section">
      <div class="section-title">Appearance</div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Theme</span>
          <span class="setting-desc">
            {#if app.theme === 'system'}
              跟隨系統主題（目前：{app.resolvedTheme === 'dark' ? '深色' : '淺色'}）
            {:else if app.theme === 'dark'}
              手動深色主題
            {:else}
              手動淺色主題
            {/if}
          </span>
        </div>
        <div class="theme-segmented" role="radiogroup" aria-label="Theme">
          {#each [
            { value: 'system' as const, label: '⚙ System' },
            { value: 'dark' as const, label: '☽ Dark' },
            { value: 'light' as const, label: '☀ Light' },
          ] as option}
            <button
              class="theme-option"
              class:active={app.theme === option.value}
              role="radio"
              aria-checked={app.theme === option.value}
              onclick={() => app.setTheme(option.value)}
            >
              {option.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <div class="section">
      <div class="section-title">Keyboard Shortcuts</div>
      <div class="shortcuts-list">
        {#each shortcuts as shortcut}
          <div class="shortcut-row">
            <kbd class="shortcut-keys">{shortcut.keys}</kbd>
            <span class="shortcut-action">{shortcut.action}</span>
          </div>
        {/each}
      </div>
    </div>

    <div class="section">
      <div class="section-title">About</div>
      <div class="about-info">
        <div class="about-row">
          <span class="about-label">Version</span>
          <span class="about-value">0.1.0</span>
        </div>
        <div class="about-row">
          <span class="about-label">Framework</span>
          <span class="about-value">Tauri 2.x + Svelte 5</span>
        </div>
        <div class="about-row">
          <span class="about-label">License</span>
          <span class="about-value">MIT</span>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }
  .settings-header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .back-btn {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--font-size-md);
  }
  .back-btn:hover { text-decoration: underline; }
  .settings-title {
    font-size: var(--font-size-lg);
    font-weight: 600;
  }
  .settings-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
    max-width: 600px;
    margin: 0 auto;
    width: 100%;
  }
  .section {
    margin-bottom: 32px;
  }
  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    margin-bottom: var(--space-md);
    padding-bottom: var(--space-xs);
    border-bottom: 1px solid var(--border);
  }
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) 0;
  }
  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .setting-label { font-size: var(--font-size-md); }
  .setting-desc { font-size: 11px; color: var(--text-muted); }
  .theme-segmented {
    display: flex;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .theme-option {
    flex: 1;
    padding: 6px 12px;
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    white-space: nowrap;
  }
  .theme-option:hover:not(.active) {
    background: var(--bg-hover);
  }
  .theme-option.active {
    background: var(--accent);
    color: var(--bg-primary);
    font-weight: 600;
  }
  .theme-option:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) 0;
  }
  .shortcut-keys {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
  }
  .shortcut-action {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  .about-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }
  .about-row {
    display: flex;
    justify-content: space-between;
    font-size: var(--font-size-sm);
  }
  .about-label { color: var(--text-secondary); }
  .about-value { color: var(--text-primary); font-family: var(--font-mono); }
</style>
