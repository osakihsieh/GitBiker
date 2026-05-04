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
      // AI & Acceleration
      {
        id: 'ai-review',
        label: 'AI: 智慧代碼審查 (Review)',
        shortcut: 'Shift+Cmd+R',
        category: 'AI',
        action: () => {
           app.addToast('AI 審查啟動中...', 'success');
           // Trigger logic here
        },
      },
      {
        id: 'ai-fusion',
        label: 'AI: 智慧衝突融合 (Fusion)',
        shortcut: '',
        category: 'AI',
        action: () => {
           app.enterConflictMode();
        },
      },
      // Git
      {
        id: 'push',
        label: 'Git: Push (推送到遠端)',
        shortcut: '',
        category: 'Git',
        action: async () => {
          if (!app.repoPath) return;
          const result = await gitPush(app.repoPath);
          app.addToast(
            result.success ? `Pushed to ${result.remote}/${result.branch}` : result.message,
            result.success ? 'success' : 'error',
          );
          await app.refreshAll();
        },
      },
      {
        id: 'pull',
        label: 'Git: Pull (從遠端拉取)',
        shortcut: '',
        category: 'Git',
        action: async () => {
          if (!app.repoPath) return;
          const result = await gitPull(app.repoPath);
          app.addToast(
            result.success ? 'Pull 完成' : result.message,
            result.success ? 'success' : 'error',
          );
          await app.refreshAll();
        },
      },
      {
        id: 'fetch',
        label: 'Git: Fetch',
        shortcut: '',
        category: 'Git',
        action: async () => {
          if (!app.repoPath) return;
          await gitFetch(app.repoPath);
          await app.refreshAll();
          app.addToast('Fetch 完成', 'success');
        },
      },
      // Navigation
      {
        id: 'settings',
        label: '開啟設定 (Settings)',
        shortcut: app.isMac ? 'Cmd+,' : 'Ctrl+,',
        category: 'App',
        action: () => {
          onOpenSettings();
        },
      },
      {
        id: 'folder',
        label: '在檔案瀏覽器開啟 (Explorer)',
        shortcut: 'Alt+O',
        category: 'App',
        action: () => {
          if (app.repoPath) openInFolder(app.repoPath);
        },
      },
      {
        id: 'editor',
        label: '在編輯器開啟 (Editor)',
        shortcut: 'Alt+E',
        category: 'App',
        action: () => {
          if (app.repoPath) openInEditor(app.repoPath, app.preferredEditor ?? undefined);
        },
      },
      {
        id: 'terminal',
        label: '開啟終端機 (Terminal)',
        shortcut: 'Alt+T',
        category: 'App',
        action: () => {
          if (app.repoPath) openInTerminal(app.repoPath);
        },
      },
    ];

    // Add branch switching (limit to top 5 for palette performance)
    for (const branch of app.branches.filter((b) => !b.is_remote && !b.is_current).slice(0, 5)) {
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

    return cmds;
  });

  const filtered = $derived(() => {
    const q = query.toLowerCase().trim();
    if (!q) return commands;
    return commands.filter(
      (c) => c.label.toLowerCase().includes(q) || c.category.toLowerCase().includes(q),
    );
  });

  $effect(() => {
    if (open) {
      query = '';
      selectedIndex = 0;
      requestAnimationFrame(() => inputEl?.focus());
    }
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
  <div 
    class="fixed inset-0 bg-black/60 z-[100] flex justify-center pt-[15vh] backdrop-blur-sm"
    onclick={onClose} 
    onkeydown={handleKeydown}
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="w-[600px] max-w-[90vw] max-h-[50vh] bg-bg-secondary border border-bg-tertiary rounded-large shadow-2xl flex flex-col overflow-hidden self-start"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-label="Command Palette"
    >
      <div class="px-md py-sm border-b border-bg-tertiary flex items-center gap-sm">
        <span class="text-monokai-blue">⌘</span>
        <input
          type="text"
          class="flex-1 bg-transparent border-none text-text-bright text-md outline-none placeholder:text-text-dimmed"
          placeholder="搜尋指令或功能..."
          bind:value={query}
          bind:this={inputEl}
          onkeydown={handleKeydown}
        />
      </div>
      
      <div class="overflow-y-auto flex-1 min-h-0 py-xs" role="listbox">
        {#each filtered() as cmd, i}
          <button
            class="flex items-center justify-between w-full px-md py-sm text-left transition-colors"
            class:bg-bg-tertiary={i === selectedIndex}
            class:text-monokai-yellow={i === selectedIndex}
            role="option"
            aria-selected={i === selectedIndex}
            onclick={() => handleSelect(cmd)}
            onmouseenter={() => (selectedIndex = i)}
          >
            <div class="flex items-center gap-md">
                <span class="text-xs uppercase tracking-widest text-text-dimmed w-12">{cmd.category}</span>
                <span class="text-sm font-medium">{cmd.label}</span>
            </div>
            {#if cmd.shortcut}
              <span class="text-[10px] font-mono opacity-50 bg-bg-primary px-1.5 py-0.5 rounded border border-bg-tertiary">
                {cmd.shortcut}
              </span>
            {/if}
          </button>
        {/each}
        {#if filtered().length === 0}
          <div class="p-lg text-center text-text-dimmed text-sm">找不到匹配的指令</div>
        {/if}
      </div>
      
      <div class="px-md py-xs bg-bg-tertiary/50 border-t border-bg-tertiary flex justify-between items-center">
         <span class="text-[10px] text-text-dimmed">使用方向鍵選擇，Enter 執行</span>
         <div class="flex gap-sm">
            <span class="text-[10px] text-text-dimmed px-1 border border-bg-tertiary rounded">ESC</span>
            <span class="text-[10px] text-text-dimmed">關閉</span>
         </div>
      </div>
    </div>
  </div>
{/if}
