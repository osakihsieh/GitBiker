<script lang="ts">
  import { agentStore } from '$lib/stores/agentStore.svelte';
  import { onMount } from 'svelte';
  import { Badge } from '$lib/components/ui';

  onMount(() => {
    agentStore.init();
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'thinking': return 'bg-yellow-400';
      case 'patching': return 'bg-green-500';
      case 'searching': return 'bg-blue-400';
      case 'orchestrating': return 'bg-purple-400';
      case 'working': return 'bg-orange-400';
      default: return 'bg-gray-400';
    }
  };

  const getStatusEmoji = (status: string) => {
    switch (status) {
      case 'thinking': return '🧠';
      case 'patching': return '🛠️';
      case 'searching': return '🔍';
      case 'orchestrating': return '🏗️';
      case 'working': return '⚙️';
      default: return '💤';
    }
  };
</script>

<div class="p-6 border-b-8 border-black bg-white overflow-hidden">
  <div class="flex justify-between items-center mb-6">
    <div class="flex items-center gap-3">
        <span class="text-4xl">📡</span>
        <div class="flex flex-col">
            <h2 class="text-3xl font-black uppercase tracking-tighter leading-none">Agent Radar</h2>
            <span class="text-[10px] font-black text-gray-400 tracking-[0.2em]">PASSIVE MONITORING SYSTEM V1.0</span>
        </div>
    </div>
    <div class="flex gap-2">
      <div class="px-3 py-1 border-4 border-black font-black text-sm uppercase bg-black text-white shadow-[4px_4px_0px_0px_rgba(0,0,0,0.3)]">
        Online: {agentStore.statuses.filter(s => s.pid).length}
      </div>
    </div>
  </div>
  
  {#if agentStore.statuses.length === 0}
    <div class="p-12 border-8 border-black border-dashed text-center">
      <p class="text-2xl font-black text-gray-300 uppercase italic tracking-widest animate-pulse">
        Waiting for agent footprints...
      </p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
      {#each agentStore.statuses as agent}
        <div class="group border-8 border-black p-5 bg-white shadow-[12px_12px_0px_0px_rgba(0,0,0,1)] flex flex-col gap-4 relative transition-transform hover:-translate-y-1">
          {#if !agent.pid}
             <div class="absolute inset-0 bg-white/80 backdrop-blur-[2px] z-10 flex items-center justify-center">
                <span class="bg-red-600 text-white px-4 py-2 font-black text-xl uppercase -rotate-6 border-4 border-black shadow-lg">Disconnected</span>
             </div>
          {/if}

          <!-- Header -->
          <div class="flex justify-between items-start border-b-4 border-black pb-3">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 border-4 border-black flex items-center justify-center font-black text-2xl uppercase bg-black text-white group-hover:bg-yellow-400 group-hover:text-black transition-colors">
                {agent.profile.slice(0, 1)}
              </div>
              <div class="flex flex-col">
                <span class="font-black uppercase text-lg leading-none tracking-tighter">{agent.profile}</span>
                <span class="text-[10px] font-black text-gray-500 uppercase tracking-widest">Profile Identity</span>
              </div>
            </div>
            {#if agent.pid}
              <div class="font-mono text-[10px] font-black bg-gray-100 px-2 py-0.5 border-2 border-black">
                PID:{agent.pid}
              </div>
            {/if}
          </div>
          
          <!-- Status Line -->
          <div class="flex items-center gap-3">
            <div class="w-6 h-6 border-4 border-black {getStatusColor(agent.status)} {agent.pid ? 'animate-pulse' : ''}"></div>
            <div class="flex flex-col">
                <span class="font-black text-xs uppercase tracking-[0.1em] text-gray-400">Current Vector</span>
                <span class="font-black text-sm uppercase leading-none flex items-center gap-2">
                    {agent.status} {getStatusEmoji(agent.status)}
                </span>
            </div>
          </div>

          <!-- Active Action -->
          <div class="bg-black text-white p-4 border-4 border-black font-bold font-mono text-xs break-words leading-relaxed min-h-[80px] relative overflow-hidden">
            <div class="absolute top-0 right-0 p-1 opacity-20 text-[8px] font-black">LOG_STREAM</div>
            <span class="text-green-400 mr-2">>>></span>{agent.last_action}
            <span class="inline-block w-2 h-4 bg-green-400 ml-1 animate-bounce"></span>
          </div>

          <!-- Passive Kanban -->
          <div class="flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <div class="h-1 flex-1 bg-black"></div>
                <span class="text-[10px] font-black uppercase tracking-widest bg-white px-2">Task History</span>
                <div class="h-1 flex-1 bg-black"></div>
            </div>
            <div class="flex flex-col gap-2">
              {#each agent.history.slice(-3).reverse() as hist}
                <div class="text-[10px] font-bold font-mono text-gray-600 border-l-4 border-gray-200 pl-2 py-1 bg-gray-50 flex items-center gap-2 italic">
                  <span class="text-gray-300">#</span> {hist}
                </div>
              {/each}
              {#if agent.history.length === 0}
                <div class="text-[10px] font-bold text-gray-300 uppercase text-center py-2 border-2 border-dashed border-gray-200">
                    No prior data recorded
                </div>
              {/if}
            </div>
          </div>

          <!-- Footer -->
          <div class="flex justify-between items-center mt-auto pt-2 border-t-2 border-black border-dotted">
            <span class="text-[9px] font-black text-gray-400">SYS_ACCEL_M4</span>
            <span class="text-[9px] text-black font-black uppercase">
              {new Date(agent.last_update * 1000).toLocaleTimeString()}
            </span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* Custom scrollbar logic for the dashboard container if needed */
  ::-webkit-scrollbar {
    width: 16px;
  }
  ::-webkit-scrollbar-track {
    background: #fff;
    border-left: 8px solid #000;
  }
  ::-webkit-scrollbar-thumb {
    background: #000;
    border: 4px solid #fff;
  }
  ::-webkit-scrollbar-thumb:hover {
    background: #444;
  }
</style>
