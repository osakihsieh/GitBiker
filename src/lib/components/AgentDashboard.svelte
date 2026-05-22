<script lang="ts">
  import { agentStore } from '$lib/stores/agentStore.svelte';
  import { onMount } from 'svelte';
  import { Activity, Cpu, Search, Terminal, Brain, HardDrive } from 'lucide-react';

  onMount(() => {
    agentStore.init();
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'thinking': return 'var(--color-accent)';
      case 'patching': return 'var(--color-accent)';
      case 'searching': return '#3d6a8c'; // Bank blue from Ginga
      case 'orchestrating': return '#7a4a8c'; // Wallet purple
      case 'working': return '#a16b3d'; // Card brown
      default: return 'var(--color-ink-35)';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'thinking': return Brain;
      case 'patching': return Cpu;
      case 'searching': return Search;
      case 'orchestrating': return HardDrive;
      case 'working': return Activity;
      default: return Terminal;
    }
  };
</script>

<div class="h-full flex flex-col bg-bg overflow-hidden animate-fade-up">
  <!-- Header -->
  <header class="px-8 py-6 flex justify-between items-end border-b border-ink-10">
    <div class="flex flex-col gap-1">
      <div class="flex items-center gap-2">
        <div class="w-7 h-7 rounded-lg bg-ink flex items-center justify-center text-bg font-bold text-lg">G</div>
        <h1 class="text-[26px] font-semibold tracking-[-0.6px] leading-tight">Agent Radar</h1>
      </div>
      <p class="text-[10.5px] font-medium text-ink-35 uppercase tracking-[0.5px]">PASSIVE MONITORING SYSTEM V2.0</p>
    </div>
    <div class="flex items-center gap-3">
      <div class="px-3 py-1.5 rounded-full bg-accent-bg text-accent text-[11px] font-semibold flex items-center gap-1.5">
        <span class="w-1.5 h-1.5 rounded-full bg-accent animate-pulse"></span>
        {agentStore.statuses.filter(s => s.pid).length} ONLINE
      </div>
    </div>
  </header>
  
  <div class="flex-1 p-8 overflow-auto">
    {#if agentStore.statuses.length === 0}
      <div class="h-64 flex flex-col items-center justify-center border-2 border-dashed border-ink-10 rounded-[20px] bg-bg-deep/30">
        <p class="text-[13.5px] text-ink-35 font-medium italic">Waiting for agent footprints...</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
        {#each agentStore.statuses as agent}
          <div class="group bg-card p-6 rounded-[20px] shadow-card hover:shadow-float transition-all duration-300 border border-ink-05 flex flex-col gap-5 relative overflow-hidden">
            {#if !agent.pid}
              <div class="absolute inset-0 bg-bg/60 backdrop-blur-[1px] z-10 flex items-center justify-center">
                <span class="bg-ink text-bg px-3 py-1 rounded-full font-bold text-[10px] uppercase tracking-wider">Disconnected</span>
              </div>
            {/if}

            <!-- Header -->
            <div class="flex justify-between items-start">
              <div class="flex items-center gap-4">
                <div 
                  class="w-12 h-12 rounded-xl flex items-center justify-center transition-colors"
                  style="background-color: {getStatusColor(agent.status)}15; color: {getStatusColor(agent.status)};"
                >
                  <svelte:component this={getStatusIcon(agent.status)} size={24} strokeWidth={1.5} />
                </div>
                <div class="flex flex-col">
                  <h3 class="text-[17px] font-semibold tracking-[-0.4px] leading-tight">{agent.profile}</h3>
                  <span class="text-[10.5px] font-medium text-ink-35 uppercase tracking-[0.3px]">PID: {agent.pid || 'N/A'}</span>
                </div>
              </div>
              <div class="px-2 py-0.5 rounded-md bg-ink-05 text-ink-50 text-[9px] font-mono">
                {new Date(agent.last_update * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
              </div>
            </div>
            
            <!-- Status Detail -->
            <div class="flex flex-col gap-3">
              <div class="flex items-center justify-between">
                <span class="text-[10px] font-medium text-ink-35 uppercase tracking-[0.5px]">Current Action</span>
                <div class="flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full" style="background-color: {getStatusColor(agent.status)}"></span>
                  <span class="text-[11px] font-semibold uppercase text-ink-70 tracking-tight">{agent.status}</span>
                </div>
              </div>
              <div class="bg-bg-deep/50 p-4 rounded-xl border border-ink-10 min-h-[70px]">
                <p class="text-[13.5px] font-mono leading-relaxed text-ink-70">
                  <span class="text-accent opacity-60 font-bold mr-2">$</span>{agent.last_action}
                </p>
              </div>
            </div>

            <!-- History -->
            {#if agent.history.length > 0}
              <div class="flex flex-col gap-2">
                <span class="text-[10px] font-medium text-ink-35 uppercase tracking-[0.5px]">Recent Chain</span>
                <div class="flex flex-wrap gap-1.5">
                  {#each agent.history.slice(-3).reverse() as hist}
                    <div class="px-2.5 py-1 rounded-full bg-ink-05 text-ink-50 text-[10.5px] font-medium border border-ink-05">
                      {hist}
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
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
