<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitShowFiles, gitShowFileDiff } from '$lib/git/commands';
  import type { FileStatus } from '$lib/git/types';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';

  let files = $state<FileStatus[]>([]);
  let loading = $state(false);
  let selectedFile = $state<string | null>(null);

  // Load files when selectedCommit changes
  $effect(() => {
    const commit = app.selectedCommit;
    const repoPath = app.repoPath;
    if (!commit || !repoPath) {
      files = [];
      return;
    }

    loading = true;
    selectedFile = null;
    gitShowFiles(repoPath, commit.id)
      .then((result) => { files = result; })
      .catch((e) => {
        app.addToast(String(e), 'error');
        files = [];
      })
      .finally(() => { loading = false; });
  });

  async function handleFileClick(file: FileStatus) {
    if (!app.repoPath || !app.selectedCommit) return;
    selectedFile = file.path;
    try {
      app.currentDiff = await gitShowFileDiff(
        app.repoPath,
        app.selectedCommit.id,
        file.path,
      );
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  function statusLabel(kind: FileStatus['kind']): string {
    const map: Record<string, string> = {
      Modified: 'M', Added: 'A', Deleted: 'D', Renamed: 'R',
      Copied: 'C', Unknown: '?'
    };
    return map[kind] || '?';
  }

  function statusClass(kind: FileStatus['kind']): string {
    const map: Record<string, string> = {
      Modified: 'status-m', Added: 'status-a', Deleted: 'status-d',
    };
    return map[kind] || '';
  }

  function fileName(path: string): string {
    return path.replace(/\\/g, '/').split('/').pop() || path;
  }

  function firstLine(msg: string): string {
    return msg.split('\n')[0] || '';
  }

  function shortHash(id: string): string {
    return id.substring(0, 7);
  }

  let fileContextMenu = $state<{ file: FileStatus; x: number; y: number } | null>(null);

  const fileContextMenuItems: MenuItem[] = [
    { id: 'file-history', label: '查看檔案歷史' },
    { id: '_sep', label: '', separator: true },
    { id: 'copy-path', label: '複製路徑' },
  ];

  function handleFileContextMenu(e: MouseEvent, file: FileStatus) {
    e.preventDefault();
    fileContextMenu = { file, x: e.clientX, y: e.clientY };
  }

  async function handleFileContextSelect(actionId: string) {
    if (!fileContextMenu) return;
    const { file } = fileContextMenu;
    switch (actionId) {
      case 'file-history':
        app.showFileHistory(file.path);
        break;
      case 'copy-path':
        await navigator.clipboard.writeText(file.path);
        app.addToast('已複製路徑', 'success');
        break;
    }
  }

  function timeAgo(timestamp: number): string {
    const seconds = Math.floor(Date.now() / 1000 - timestamp);
    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  }
</script>

<div class="commit-detail">
  <!-- Back button -->
  <button class="back-btn" onclick={() => app.backToWorktree()}>
    <span class="back-arrow">←</span> Changes
  </button>

  <!-- Commit info -->
  {#if app.selectedCommit}
    <div class="commit-info-block">
      <div class="commit-hash">{shortHash(app.selectedCommit.id)}</div>
      <div class="commit-msg" title={app.selectedCommit.message}>
        {firstLine(app.selectedCommit.message)}
      </div>
      <div class="commit-meta">
        {app.selectedCommit.author} · {timeAgo(app.selectedCommit.timestamp)}
      </div>
    </div>
  {/if}

  <div class="separator"></div>

  <!-- File list header -->
  <div class="section-header">
    <span>Files <span class="count">{files.length}</span></span>
  </div>

  <!-- File list -->
  <div class="file-list">
    {#if loading}
      <div class="loading-state">
        <span class="spinner"></span> Loading...
      </div>
    {:else}
      {#each files as file (file.path)}
        <button
          class="file-item"
          class:active={selectedFile === file.path}
          onclick={() => handleFileClick(file)}
          oncontextmenu={(e) => handleFileContextMenu(e, file)}
        >
          <span class="status {statusClass(file.kind)}">{statusLabel(file.kind)}</span>
          <span class="filename" title={file.path}>{fileName(file.path)}</span>
        </button>
      {:else}
        <div class="empty-state">No files in this commit</div>
      {/each}
    {/if}
  </div>
</div>

{#if fileContextMenu}
  <ContextMenu
    x={fileContextMenu.x}
    y={fileContextMenu.y}
    items={fileContextMenuItems}
    onSelect={handleFileContextSelect}
    onClose={() => fileContextMenu = null}
  />
{/if}

<style>
  .commit-detail {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .back-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--accent);
    font-size: 12px;
    font-family: var(--font-ui);
    cursor: pointer;
    text-align: left;
    flex-shrink: 0;
  }
  .back-btn:hover { text-decoration: underline; }
  .back-arrow { font-size: 14px; }
  .commit-info-block {
    padding: var(--space-sm) var(--space-md);
    flex-shrink: 0;
  }
  .commit-hash {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }
  .commit-msg {
    font-size: 12px;
    color: var(--text-primary);
    margin-top: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .commit-meta {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: 2px;
  }
  .separator {
    height: 1px;
    background: var(--border);
    flex-shrink: 0;
  }
  .section-header {
    padding: var(--space-sm) var(--space-md);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .count {
    background: var(--bg-surface);
    padding: 1px 6px;
    border-radius: 8px;
    font-size: 10px;
    color: var(--text-muted);
  }
  .file-list {
    flex: 1;
    overflow-y: auto;
  }
  .file-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-md);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    border: none;
    border-left: 2px solid transparent;
    background: none;
    color: var(--text-primary);
    width: 100%;
    text-align: left;
  }
  .file-item:hover { background: var(--bg-hover); }
  .file-item.active {
    background: var(--bg-surface);
    border-left-color: var(--accent);
  }
  .status { font-size: 11px; flex-shrink: 0; width: 12px; text-align: center; }
  .status-m { color: var(--warning); }
  .status-a { color: var(--success); }
  .status-d { color: var(--error); }
  .filename {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  .empty-state {
    padding: var(--space-md);
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    text-align: center;
  }
  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: var(--space-lg);
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }
  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid var(--text-muted);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
