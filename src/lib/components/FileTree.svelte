<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitStage, gitUnstage, gitCommit } from '$lib/git/commands';
  import type { FileStatus } from '$lib/git/types';

  let commitMessage = $state('');
  let committing = $state(false);

  function statusLabel(kind: FileStatus['kind']): string {
    const map: Record<string, string> = {
      Modified: 'M', Added: 'A', Deleted: 'D', Renamed: 'R',
      Copied: 'C', Untracked: 'U', Conflicted: '!', Unknown: '?'
    };
    return map[kind] || '?';
  }

  function statusClass(kind: FileStatus['kind']): string {
    const map: Record<string, string> = {
      Modified: 'status-m', Added: 'status-a', Deleted: 'status-d',
      Untracked: 'status-u', Conflicted: 'status-c'
    };
    return map[kind] || '';
  }

  async function handleToggleStage(file: FileStatus) {
    if (!app.repoPath) return;
    try {
      if (file.staging === 'Staged') {
        await gitUnstage(app.repoPath, [file.path]);
      } else {
        await gitStage(app.repoPath, [file.path]);
      }
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleStageAll() {
    if (!app.repoPath) return;
    const paths = app.unstagedFiles.map((f) => f.path);
    if (paths.length === 0) return;
    try {
      await gitStage(app.repoPath, paths);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleUnstageAll() {
    if (!app.repoPath) return;
    const paths = app.stagedFiles.map((f) => f.path);
    if (paths.length === 0) return;
    try {
      await gitUnstage(app.repoPath, paths);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleCommit() {
    if (!app.repoPath || !commitMessage.trim() || committing) return;
    committing = true;
    try {
      const hash = await gitCommit(app.repoPath, commitMessage);
      app.addToast(`Committed ${hash.substring(0, 7)}`, 'success');
      commitMessage = '';
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      committing = false;
    }
  }

  function selectFile(path: string) {
    app.selectedFile = path;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'Enter') {
      e.preventDefault();
      handleCommit();
    }
  }

  function fileName(path: string): string {
    return path.replace(/\\/g, '/').split('/').pop() || path;
  }
</script>

<div class="file-tree">
  <!-- Staged -->
  <div class="section-header">
    <span>Staged <span class="count">{app.stagedFiles.length}</span></span>
    {#if app.stagedFiles.length > 0}
      <button class="section-action" onclick={handleUnstageAll}>Unstage All</button>
    {/if}
  </div>
  <div class="file-list">
    {#each app.stagedFiles as file (file.path + file.staging)}
      <button
        class="file-item"
        class:active={app.selectedFile === file.path}
        onclick={() => selectFile(file.path)}
      >
        <span
          class="checkbox checked"
          role="checkbox"
          aria-checked="true"
          tabindex="-1"
          onclick={(e: MouseEvent) => { e.stopPropagation(); handleToggleStage(file); }}
          onkeydown={(e) => e.key === 'Enter' && handleToggleStage(file)}
        >✓</span>
        <span class="status {statusClass(file.kind)}">{statusLabel(file.kind)}</span>
        <span class="filename" title={file.path}>{fileName(file.path)}</span>
      </button>
    {:else}
      <div class="empty-section">No staged changes</div>
    {/each}
  </div>

  <!-- Unstaged -->
  <div class="section-header">
    <span>Unstaged <span class="count">{app.unstagedFiles.length}</span></span>
    {#if app.unstagedFiles.length > 0}
      <button class="section-action" onclick={handleStageAll}>Stage All</button>
    {/if}
  </div>
  <div class="file-list file-list-grow">
    {#each app.unstagedFiles as file (file.path + file.staging)}
      <button
        class="file-item"
        class:active={app.selectedFile === file.path}
        onclick={() => selectFile(file.path)}
      >
        <span
          class="checkbox"
          role="checkbox"
          aria-checked="false"
          tabindex="-1"
          onclick={(e: MouseEvent) => { e.stopPropagation(); handleToggleStage(file); }}
          onkeydown={(e) => e.key === 'Enter' && handleToggleStage(file)}
        ></span>
        <span class="status {statusClass(file.kind)}">{statusLabel(file.kind)}</span>
        <span class="filename" title={file.path}>{fileName(file.path)}</span>
      </button>
    {:else}
      <div class="empty-section">No unstaged changes</div>
    {/each}
  </div>

  <!-- Commit Form -->
  <div class="commit-form">
    <textarea
      bind:value={commitMessage}
      placeholder="Commit message (Ctrl+Enter to commit)"
      onkeydown={handleKeydown}
    ></textarea>
    <div class="commit-actions">
      <button
        class="commit-btn"
        onclick={handleCommit}
        disabled={!commitMessage.trim() || committing || app.stagedFiles.length === 0}
      >
        {#if committing}
          <span class="spinner"></span>
        {/if}
        Commit ({app.stagedFiles.length} files)
      </button>
    </div>
    <div class="shortcut-hint">Ctrl+Enter</div>
  </div>
</div>

<style>
  .file-tree {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border);
    user-select: none;
    flex-shrink: 0;
  }
  .count {
    background: var(--bg-surface);
    padding: 1px 6px;
    border-radius: 8px;
    font-size: 10px;
    color: var(--text-muted);
  }
  .section-action {
    font-size: 10px;
    color: var(--accent);
    cursor: pointer;
    background: none;
    border: none;
    font-family: var(--font-ui);
  }
  .section-action:hover { text-decoration: underline; }
  .file-list {
    overflow-y: auto;
    max-height: 140px;
    flex-shrink: 0;
  }
  .file-list-grow {
    flex: 1;
    max-height: none;
    min-height: 60px;
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
  .checkbox {
    width: 14px;
    height: 14px;
    border: 1px solid var(--text-muted);
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    font-size: 10px;
    cursor: pointer;
  }
  .checkbox.checked {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--bg-primary);
  }
  .status { font-size: 11px; flex-shrink: 0; width: 12px; text-align: center; }
  .status-m { color: var(--warning); }
  .status-a { color: var(--success); }
  .status-d { color: var(--error); }
  .status-u { color: var(--text-secondary); }
  .status-c { color: var(--error); font-weight: bold; }
  .filename {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  .empty-section {
    padding: var(--space-md);
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    text-align: center;
  }
  .commit-form {
    border-top: 1px solid var(--border);
    padding: var(--space-sm) var(--space-md);
    flex-shrink: 0;
  }
  .commit-form textarea {
    width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    padding: var(--space-sm);
    resize: vertical;
    min-height: 60px;
    max-height: 120px;
    outline: none;
  }
  .commit-form textarea:focus { border-color: var(--accent); }
  .commit-form textarea::placeholder { color: var(--text-muted); }
  .commit-actions {
    margin-top: var(--space-sm);
  }
  .commit-btn {
    width: 100%;
    background: var(--accent);
    color: var(--bg-primary);
    border: none;
    border-radius: var(--radius-sm);
    padding: var(--space-sm);
    font-weight: 600;
    font-size: var(--font-size-sm);
    cursor: pointer;
    font-family: var(--font-ui);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-xs);
  }
  .commit-btn:hover:not(:disabled) { filter: brightness(1.1); }
  .commit-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .shortcut-hint {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: var(--space-xs);
    text-align: right;
  }
  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid rgba(0,0,0,0.3);
    border-top-color: var(--bg-primary);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
