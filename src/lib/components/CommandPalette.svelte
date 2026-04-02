<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import {
    gitSwitchBranch,
    gitFetch,
    gitPush,
    gitPull,
    gitStashPush,
    openInFolder,
    openInEditor,
    openInTerminal,
  } from '$lib/git/commands';

  interface Props {
    open: boolean;
    onClose: () => void;
    onOpenSettings: () => void;
  }

  let { open, onClose, onOpenSettings }: Props = $props();

  let query = $state('');
  let selectedIndex = $state(0);
  let inputEl = $state<HTMLInputElement | null>(null);

  interface Command {
    id: string;
    label: string;
    shortcut?: string;
    action: () => void | Promise<void>;
    category: string;
  }

  const commands: Command[] = $derived.by(() => {
    const cmds: Command[] = [
      // Git
      { id: 'push', label: 'Git: Push', shortcut: '', category: 'Git', action: async () => {
        if (!app.repoPath) return;
        const result = await gitPush(app.repoPath);
        app.addToast(result.success ? `Pushed to ${result.remote}/${result.branch}` : result.message, result.success ? 'success' : 'error');
        await app.refreshAll();
      }},
      { id: 'pull', label: 'Git: Pull', shortcut: '', category: 'Git', action: async () => {
        if (!app.repoPath) return;
        const result = await gitPull(app.repoPath);
        app.addToast(result.success ? 'Pull 完成' : result.message, result.success ? 'success' : 'error');
        await app.refreshAll();
      }},
      { id: 'fetch', label: 'Git: Fetch', shortcut: '', category: 'Git', action: async () => {
        if (!app.repoPath) return;
        await gitFetch(app.repoPath);
        await app.refreshAll();
        app.addToast('Fetch 完成', 'success');
      }},
      { id: 'stash', label: 'Git: Stash All', shortcut: '', category: 'Git', action: async () => {
        if (!app.repoPath) return;
        await gitStashPush(app.repoPath);
        app.addToast('已 stash 變更', 'success');
        await app.refreshStatus();
      }},
      // Navigation
      { id: 'settings', label: 'Open Settings', shortcut: 'Ctrl+,', category: 'App', action: () => { onOpenSettings(); }},
      { id: 'folder', label: 'Open in File Explorer', shortcut: 'Alt+O', category: 'App', action: () => {
        if (app.repoPath) openInFolder(app.repoPath);
      }},
      { id: 'editor', label: 'Open in Editor', shortcut: 'Alt+E', category: 'App', action: () => {
        if (app.repoPath) openInEditor(app.repoPath, app.preferredEditor ?? undefined);
      }},
      { id: 'terminal', label: 'Open Terminal', shortcut: 'Alt+T', category: 'App', action: () => {
        if (app.repoPath) openInTerminal(app.repoPath);
      }},
      // View
      { id: 'worktree', label: 'View: Worktree', shortcut: '', category: 'View', action: () => { app.backToWorktree(); }},
    ];

    // Add branch switching
    for (const branch of app.branches.filter((b) => !b.is_remote && !b.is_current)) {
      cmds.push({
        id: `branch-${branch.name}`,
        label: `Switch Branch: ${branch.name}`,
        shortcut: '',
        category: 'Branch',
        action: async () => {
          if (!app.repoPath) return;
          await gitSwitchBranch(app.repoPath, branch.name);
          app.currentBranch = branch.name;
          app.addToast(`已切換到 ${branch.name}`, 'success');
          await app.refreshAll();
        },
      });
    }

    // Add tab switching
    for (const tab of app.tabs.filter((t) => t.id !== app.activeTabId)) {
      cmds.push({
        id: `tab-${tab.id}`,
        label: `Switch to: ${tab.name}`,
        shortcut: '',
        category: 'Tab',
        action: () => { app.switchTab(tab.id); },
      });
    }

    return cmds;
  });

  const filtered = $derived(() => {
    const q = query.toLowerCase().trim();
    if (!q) return commands;
    return commands.filter((c) =>
      c.label.toLowerCase().includes(q) || c.category.toLowerCase().includes(q)
    );
  });

  $effect(() => {
    if (open) {
      query = '';
      selectedIndex = 0;
      // Focus input after render
      requestAnimationFrame(() => inputEl?.focus());
    }
  });

  // Reset selection when filter changes
  $effect(() => {
    filtered();
    selectedIndex = 0;
  });

  function handleKeydown(e: KeyboardEvent) {
    const items = filtered();
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % items.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + items.length) % items.length;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const cmd = items[selectedIndex];
      if (cmd) {
        onClose();
        cmd.action();
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }

  function handleSelect(cmd: Command) {
    onClose();
    cmd.action();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="palette-backdrop" onclick={onClose} onkeydown={handleKeydown}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="palette" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Command Palette">
      <input
        type="text"
        class="palette-input"
        placeholder="Type a command..."
        bind:value={query}
        bind:this={inputEl}
        onkeydown={handleKeydown}
      />
      <div class="palette-list" role="listbox">
        {#each filtered() as cmd, i}
          <button
            class="palette-item"
            class:selected={i === selectedIndex}
            role="option"
            aria-selected={i === selectedIndex}
            onclick={() => handleSelect(cmd)}
            onmouseenter={() => (selectedIndex = i)}
          >
            <span class="palette-label">{cmd.label}</span>
            {#if cmd.shortcut}
              <span class="palette-shortcut">{cmd.shortcut}</span>
            {/if}
          </button>
        {/each}
        {#if filtered().length === 0}
          <div class="palette-empty">No matching commands</div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 100;
    display: flex;
    justify-content: center;
    padding-top: 15vh;
  }
  .palette {
    width: 500px;
    max-width: 90vw;
    max-height: 50vh;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg, 8px);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    align-self: flex-start;
  }
  .palette-input {
    width: 100%;
    padding: var(--space-md);
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
    font-size: var(--font-size-md);
    font-family: var(--font-ui);
    outline: none;
  }
  .palette-input::placeholder { color: var(--text-muted); }
  .palette-list {
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }
  .palette-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-sm) var(--space-md);
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    cursor: pointer;
    text-align: left;
  }
  .palette-item:hover,
  .palette-item.selected {
    background: var(--bg-hover);
  }
  .palette-label { flex: 1; }
  .palette-shortcut {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    flex-shrink: 0;
    margin-left: var(--space-md);
  }
  .palette-empty {
    padding: var(--space-lg) var(--space-md);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }
</style>
