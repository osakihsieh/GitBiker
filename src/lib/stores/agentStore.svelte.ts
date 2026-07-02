import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface AgentStatus {
  profile: string;
  pid?: number;
  last_action: string;
  status: 'thinking' | 'searching' | 'patching' | 'idle' | 'working' | 'orchestrating';
  last_update: number;
  history: string[];
  worktree?: string;
}

class AgentStore {
  statuses = $state<AgentStatus[]>([]);
  initialized = false;

  async init() {
    if (this.initialized) return;
    
    // Initial fetch
    try {
      this.statuses = await invoke<AgentStatus[]>('get_agent_statuses');
    } catch (e) {
      console.error('Failed to fetch agent statuses:', e);
    }

    // Listen for updates
    await listen<AgentStatus[]>('agent_statuses_update', (event) => {
      this.statuses = event.payload;
    });

    this.initialized = true;
  }
}

export const agentStore = new AgentStore();
