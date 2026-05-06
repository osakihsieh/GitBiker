<script lang="ts">
  import { app, type GitHubItem } from '$lib/stores/app.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';

  let activeTab = $state<'prs' | 'issues'>('prs');

  const items = $derived(activeTab === 'prs' ? app.prs : app.issues);

  async function handleItemClick(item: GitHubItem) {
    if (item.url) {
      await openUrl(item.url);
    }
  }

  function formatTime(dateStr?: string) {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
</script>

<div class="flex-1 flex flex-col bg-bg-primary overflow-hidden">
  <!-- Header Tabs -->
  <div class="h-10 flex items-center px-md bg-bg-secondary/50 border-b border-border gap-md">
    <button 
      class="text-sm font-medium transition-colors {activeTab === 'prs' ? 'text-monokai-blue border-b-2 border-monokai-blue' : 'text-text-dimmed hover:text-text-bright'}"
      onclick={() => activeTab = 'prs'}
    >
      Pull Requests ({app.prs.length})
    </button>
    <button 
      class="text-sm font-medium transition-colors {activeTab === 'issues' ? 'text-monokai-green border-b-2 border-monokai-green' : 'text-text-dimmed hover:text-text-bright'}"
      onclick={() => activeTab = 'issues'}
    >
      Issues ({app.issues.length})
    </button>
    
    {#if app.isLoadingRemote}
      <div class="ml-auto flex items-center gap-xs text-[10px] text-text-dimmed italic">
        <span class="animate-pulse">●</span> 同步中...
      </div>
    {/if}
  </div>

  <!-- List Content -->
  <div class="flex-1 overflow-y-auto p-sm">
    <div class="space-y-xs">
      {#each items as item}
        <button 
          class="w-full flex flex-col p-sm bg-bg-secondary/30 border border-white/5 rounded-large hover:bg-bg-hover/50 transition-colors group text-left"
          onclick={() => handleItemClick(item)}
        >
          <div class="flex items-start justify-between gap-sm">
            <span class="text-sm text-text-bright font-medium group-hover:text-monokai-blue transition-colors">
              <span class="text-text-dimmed font-mono">#{item.number}</span> {item.title}
            </span>
            <span class="text-[10px] px-1.5 py-0.5 rounded-full {item.state === 'OPEN' ? 'bg-monokai-green/20 text-monokai-green' : 'bg-text-dimmed/20 text-text-dimmed'}">
              {item.state}
            </span>
          </div>
          <div class="mt-xs flex items-center gap-md text-[10px] text-text-dimmed">
            <span>👤 {item.author?.login || 'unknown'}</span>
            <span>🕒 {formatTime(item.updatedAt)}</span>
          </div>
        </button>
      {/each}

      {#if items.length === 0 && !app.isLoadingRemote}
        <div class="flex flex-col items-center justify-center py-20 text-text-dimmed">
          <span class="text-3xl mb-sm">📭</span>
          <p>目前沒有開放的 {activeTab === 'prs' ? 'Pull Request' : 'Issue'}</p>
        </div>
      {/if}
    </div>
  </div>
</div>
