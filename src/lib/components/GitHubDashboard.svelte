<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { GitPullRequest, CircleDot, User, Clock, ExternalLink, RefreshCw, Inbox } from 'lucide-react';

  let activeTab = $state<'prs' | 'issues'>('prs');

  const items = $derived(activeTab === 'prs' ? app.prs : app.issues);

  async function handleItemClick(url: string) {
    if (url) {
      await openUrl(url);
    }
  }

  function formatTime(dateStr?: string) {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return date.toLocaleDateString();
  }
</script>

<div class="h-full flex flex-col bg-bg overflow-hidden animate-fade-up">
  <!-- Header -->
  <header class="px-8 py-6 flex justify-between items-end border-b border-ink-10 bg-bg">
    <div class="flex flex-col gap-1">
      <div class="flex items-center gap-2">
        <div class="w-7 h-7 rounded-lg bg-ink flex items-center justify-center text-bg font-bold text-lg">
          <GitPullRequest size={16} strokeWidth={2.5} />
        </div>
        <h1 class="text-[26px] font-semibold tracking-[-0.6px] leading-tight">GitHub Pulse</h1>
      </div>
      <p class="text-[10.5px] font-medium text-ink-35 uppercase tracking-[0.5px]">REMOTE COLLABORATION HUB</p>
    </div>

    <div class="flex items-center gap-1 bg-ink-05 p-1 rounded-xl">
      <button 
        class="px-4 py-1.5 rounded-lg text-[13px] font-semibold transition-all"
        class:bg-bg={activeTab === 'prs'}
        class:text-ink={activeTab === 'prs'}
        class:shadow-card={activeTab === 'prs'}
        class:text-ink-35={activeTab !== 'prs'}
        onclick={() => activeTab = 'prs'}
      >
        Pull Requests
      </button>
      <button 
        class="px-4 py-1.5 rounded-lg text-[13px] font-semibold transition-all"
        class:bg-bg={activeTab === 'issues'}
        class:text-ink={activeTab === 'issues'}
        class:shadow-card={activeTab === 'issues'}
        class:text-ink-35={activeTab !== 'issues'}
        onclick={() => activeTab = 'issues'}
      >
        Issues
      </button>
    </div>

    <button 
      class="p-2 rounded-lg hover:bg-ink-05 text-ink-50 transition-colors"
      onclick={() => app.loadGitHubData()}
      disabled={app.isLoadingRemote}
    >
      <RefreshCw size={16} strokeWidth={1.5} class={app.isLoadingRemote ? 'animate-spin' : ''} />
    </button>
  </header>

  <!-- List Content -->
  <div class="flex-1 overflow-y-auto p-8 custom-scrollbar bg-bg-deep/20">
    <div class="max-w-5xl mx-auto space-y-4">
      {#each items as item}
        <button 
          class="w-full bg-card p-5 rounded-[20px] shadow-card hover:shadow-float transition-all duration-300 border border-ink-05 group text-left flex flex-col gap-3"
          onclick={() => handleItemClick(item.url)}
        >
          <div class="flex items-start justify-between gap-4">
            <div class="flex flex-col gap-1">
              <div class="flex items-center gap-2">
                <span class="text-[12px] font-mono text-ink-35 font-bold">#{item.number}</span>
                <h3 class="text-[16px] font-semibold text-ink group-hover:text-accent transition-colors leading-snug">
                  {item.title}
                </h3>
              </div>
            </div>
            
            <div 
              class="px-2.5 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider flex items-center gap-1.5 shrink-0"
              class:bg-accent-bg={item.state === 'OPEN'}
              class:text-accent={item.state === 'OPEN'}
              class:bg-ink-10={item.state !== 'OPEN'}
              class:text-ink-50={item.state !== 'OPEN'}
            >
              <CircleDot size={10} strokeWidth={3} />
              {item.state}
            </div>
          </div>

          <div class="flex items-center gap-4 text-[11px] font-medium text-ink-35">
            <div class="flex items-center gap-1.5">
              <User size={12} strokeWidth={1.5} />
              <span>{item.author?.login || 'unknown'}</span>
            </div>
            <div class="flex items-center gap-1.5">
              <Clock size={12} strokeWidth={1.5} />
              <span>{formatTime(item.updatedAt)}</span>
            </div>
            <div class="ml-auto opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-1 text-accent">
              <span class="text-[10px] uppercase font-bold tracking-widest">View on GitHub</span>
              <ExternalLink size={12} strokeWidth={2} />
            </div>
          </div>
        </button>
      {/each}

      {#if items.length === 0 && !app.isLoadingRemote}
        <div class="h-64 flex flex-col items-center justify-center border-2 border-dashed border-ink-10 rounded-[32px] bg-bg-deep/30 gap-4">
          <div class="w-12 h-12 rounded-full bg-ink-05 flex items-center justify-center text-ink-35">
            <Inbox size={24} strokeWidth={1.5} />
          </div>
          <div class="text-center">
            <p class="text-[15px] text-ink font-semibold tracking-tight">No {activeTab === 'prs' ? 'Pull Requests' : 'Issues'} found</p>
            <p class="text-[11px] text-ink-35 font-medium uppercase tracking-wider mt-1">Remote workspace is clean</p>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  /* Custom scrollbar matching Ginga design */
  ::-webkit-scrollbar {
    width: 6px;
  }
  ::-webkit-scrollbar-track {
    background: transparent;
  }
  ::-webkit-scrollbar-thumb {
    background: var(--color-ink-10);
    border-radius: 10px;
  }
  ::-webkit-scrollbar-thumb:hover {
    background: var(--color-ink-20);
  }
</style>
