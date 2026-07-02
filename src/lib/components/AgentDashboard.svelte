<script lang="ts">
  import { agentStore } from '$lib/stores/agentStore.svelte';
  import { onMount } from 'svelte';
  import { Activity, Cpu, Search, Terminal, Brain, HardDrive } from 'lucide-react';

  onMount(() => {
    agentStore.init();
  });

  const statusColors: Record<string, string> = {
    thinking: 'var(--accent)',
    patching: 'var(--accent)',
    searching: '#3B82F6',
    orchestrating: '#8B5CF6',
    working: '#F59E0B',
  };

  const statusIcons: Record<string, typeof Brain> = {
    thinking: Brain,
    patching: Cpu,
    searching: Search,
    orchestrating: HardDrive,
    working: Activity,
  };

  function getStatusColor(status: string): string {
    return statusColors[status] || 'var(--text-muted)';
  }

  function getStatusIcon(status: string) {
    return statusIcons[status] || Terminal;
  }
</script>

<div class="agent-dashboard">
  <!-- Header -->
  <div class="dashboard-header">
    <div class="header-left">
      <h1 class="header-title">Agent Radar</h1>
      <p class="header-subtitle">PASSIVE MONITORING SYSTEM V2.0</p>
    </div>
    <div class="online-badge">
      <span class="online-dot"></span>
      <span>{agentStore.statuses.filter(s => s.pid).length} ONLINE</span>
    </div>
  </div>

  <!-- Content -->
  <div class="dashboard-content">
    {#if agentStore.statuses.length === 0}
      <div class="empty-state">
        <p class="empty-text">Waiting for agent footprints...</p>
      </div>
    {:else}
      <div class="agent-grid">
        {#each agentStore.statuses as agent}
          <div class="agent-card">
            {#if !agent.pid}
              <div class="disconnected-overlay">
                <span class="disconnected-badge">Disconnected</span>
              </div>
            {/if}

            <div class="card-header">
              <div class="card-header-left">
                <div class="icon-box" style="background-color: {getStatusColor(agent.status)}18; color: {getStatusColor(agent.status)};">
                  <svelte:component this={getStatusIcon(agent.status)} size={24} strokeWidth={1.5} />
                </div>
                <div class="agent-info">
                  <h3 class="agent-name">{agent.profile}</h3>
                  <div class="agent-meta">
                    <span class="meta-pid">PID: {agent.pid || 'N/A'}</span>
                    {#if agent.worktree}
                      <span class="meta-sep">|</span>
                      <span class="meta-worktree" title={agent.worktree}>{agent.worktree.split('/').pop()}</span>
                    {/if}
                  </div>
                </div>
              </div>
              <div class="timestamp">
                {new Date(agent.last_update * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
              </div>
            </div>

            <div class="card-body">
              <div class="action-row">
                <span class="action-label">Current Action</span>
                <div class="status-indicator">
                  <span class="status-dot" style="background-color: {getStatusColor(agent.status)}"></span>
                  <span class="status-text">{agent.status}</span>
                </div>
              </div>
              <div class="action-box">
                <p class="action-command">
                  <span class="prompt">$</span>{agent.last_action}
                </p>
              </div>
            </div>

            {#if agent.history.length > 0}
              <div class="card-footer">
                <span class="footer-label">Recent Chain</span>
                <div class="history-tags">
                  {#each agent.history.slice(-3).reverse() as hist}
                    <span class="history-tag">{hist}</span>
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
  .agent-dashboard {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    overflow: hidden;
  }
  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    padding: var(--space-lg) var(--space-lg) var(--space-sm);
    border-bottom: 1px solid var(--border);
  }
  .header-left {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .header-title {
    font-size: 22px;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.3px;
  }
  .header-subtitle {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .online-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 20px;
    background: var(--accent-light);
    color: var(--accent);
    font-size: 11px;
    font-weight: 600;
  }
  .online-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 2s infinite;
  }
  .dashboard-content {
    flex: 1;
    padding: var(--space-lg);
    overflow-y: auto;
  }
  .empty-state {
    height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px dashed var(--border);
    border-radius: 12px;
  }
  .empty-text {
    color: var(--text-muted);
    font-style: italic;
    font-size: 13px;
  }
  .agent-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
    gap: var(--space-lg);
  }
  .agent-card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    padding: var(--space-lg);
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--bg-surface);
    transition: box-shadow 0.2s;
  }
  .agent-card:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
  .disconnected-overlay {
    position: absolute;
    inset: 0;
    background: rgba(255, 255, 255, 0.6);
    backdrop-filter: blur(1px);
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 12px;
  }
  .disconnected-badge {
    background: var(--text-primary);
    color: var(--bg-primary);
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
  }
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }
  .card-header-left {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }
  .icon-box {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .agent-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .agent-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }
  .agent-meta {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .meta-pid {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
  }
  .meta-sep {
    color: var(--border);
    font-size: 10px;
  }
  .meta-worktree {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--accent);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 120px;
  }
  .timestamp {
    padding: 2px 8px;
    border-radius: 6px;
    background: var(--bg-hover);
    color: var(--text-muted);
    font-size: 9px;
    font-family: var(--font-mono);
  }
  .card-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }
  .action-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .action-label {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .status-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
  .status-text {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-primary);
  }
  .action-box {
    background: var(--bg-hover);
    padding: var(--space-md);
    border-radius: 8px;
    min-height: 60px;
  }
  .action-command {
    font-size: 13px;
    font-family: var(--font-mono);
    line-height: 1.5;
    color: var(--text-primary);
    word-break: break-all;
  }
  .prompt {
    color: var(--accent);
    opacity: 0.6;
    font-weight: 700;
    margin-right: 6px;
  }
  .card-footer {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .footer-label {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .history-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .history-tag {
    padding: 3px 10px;
    border-radius: 20px;
    background: var(--bg-hover);
    color: var(--text-secondary);
    font-size: 10px;
    font-weight: 500;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }
</style>