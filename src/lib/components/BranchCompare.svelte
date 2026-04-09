<script lang="ts">
  import { extractErrorMessage } from '$lib/utils/error';
  import { app } from '$lib/stores/app.svelte';
  import type { Commit, FileStatus } from '$lib/git/types';
  import { gitDiff } from '$lib/git/commands';

  let selectedFile = $state<FileStatus | null>(null);
  let diffContent = $state<any>(null);

  const result = $derived(app.branchCompareResult);

  async function handleFileClick(file: FileStatus) {
    if (!app.repoPath || !result) return;
    selectedFile = file;
    try {
      // For triple dot comparison (base...compare), we need a custom diff command
      // but here we reuse gitDiff with a special path format or just a normal diff
      // Let's assume gitDiff(path, file) shows current worktree vs index.
      // For branch compare, we need diff between base and compare.
      // We'll use a new command git_show_file_diff(path, file, base, compare)
      // Actually, for simplicity in MVP, let's just use the file status.
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  function back() {
    app.backToWorktree();
  }
</script>

<div class="compare-view">
  <div class="compare-header">
    <button class="back-btn" onclick={back}>← Back</button>
    <div class="header-info">
      <div class="comparison">
        <span class="branch-name">{result?.base}</span>
        <span class="compare-sep">...</span>
        <span class="branch-name">{result?.compare}</span>
      </div>
      <div class="stats">
        <span class="stat ahead">{result?.ahead} ahead</span>
        <span class="stat behind">{result?.behind} behind</span>
      </div>
    </div>
  </div>

  <div class="compare-content">
    <div class="commits-list">
      <div class="section-title">Commits ({result?.commits.length})</div>
      <div class="scroll-area">
        {#each result?.commits ?? [] as commit}
          <div class="commit-item">
            <div class="commit-msg">{commit.message.split('\n')[0]}</div>
            <div class="commit-meta">
              <span class="hash">{commit.id.substring(0, 7)}</span>
              <span class="author">{commit.author}</span>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <div class="files-list">
      <div class="section-title">Changed Files ({result?.files.length})</div>
      <div class="scroll-area">
        {#each result?.files ?? [] as file}
          <button 
            class="file-item" 
            class:selected={selectedFile?.path === file.path}
            onclick={() => handleFileClick(file)}
          >
            <span class="status-icon status-{file.kind.toLowerCase()}">{file.kind[0]}</span>
            <span class="file-path">{file.path}</span>
          </button>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .compare-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }
  .compare-header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    border-bottom: 1px solid var(--border);
  }
  .back-btn {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .header-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .comparison {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 16px;
    font-weight: 500;
  }
  .branch-name {
    color: var(--accent);
    font-family: var(--font-mono);
  }
  .compare-sep { color: var(--text-muted); }
  .stats {
    display: flex;
    gap: var(--space-md);
    font-size: 12px;
  }
  .stat.ahead { color: var(--success); }
  .stat.behind { color: var(--error); }

  .compare-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .commits-list, .files-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
  }
  .section-title {
    padding: var(--space-sm) var(--space-md);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-secondary);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
  }
  .scroll-area {
    flex: 1;
    overflow-y: auto;
  }
  .commit-item {
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }
  .commit-msg {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .commit-meta {
    font-size: 11px;
    color: var(--text-muted);
    display: flex;
    gap: 8px;
  }
  .hash { font-family: var(--font-mono); }

  .file-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-md);
    width: 100%;
    background: none;
    border: none;
    text-align: left;
    color: var(--text-primary);
    cursor: pointer;
  }
  .file-item:hover { background: var(--bg-hover); }
  .file-item.selected { background: var(--bg-surface); }
  .status-icon {
    width: 16px;
    height: 16px;
    font-size: 10px;
    font-weight: 800;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 2px;
  }
  .status-added { color: var(--success); }
  .status-deleted { color: var(--error); }
  .status-modified { color: var(--accent); }
  .file-path {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
