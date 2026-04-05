<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitStage, gitUnstage, gitCommit, gitIgnore, gitCheckoutFile, openInEditor } from '$lib/git/commands';
  import type { FileStatus } from '$lib/git/types';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';

  let commitMessage = $state('');
  let committing = $state(false);
  let contextMenu = $state<{ file: FileStatus; x: number; y: number } | null>(null);

  function statusLabel(kind: FileStatus['kind']): string {
    const map: Record<string, string> = {
      Modified: 'M', Added: 'A', Deleted: 'D', Renamed: 'R',
      Copied: 'C', Untracked: 'U', Conflicted: '⚠', Unknown: '?'
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
      await app.refreshStatus();
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
      await app.refreshStatus();
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
      await app.refreshStatus();
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
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      committing = false;
    }
  }

  function selectFile(path: string) {
    // Check if this is a conflicted file — enter conflict mode
    const allFiles = [...app.stagedFiles, ...app.unstagedFiles];
    const file = allFiles.find(f => f.path === path);
    if (file?.kind === 'Conflicted') {
      app.enterConflictMode().then(() => app.selectConflictFile(path));
      return;
    }
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

  function fileExtension(path: string): string | null {
    const name = fileName(path);
    const dot = name.lastIndexOf('.');
    return dot > 0 ? name.substring(dot) : null;
  }

  function parentFolder(path: string): string | null {
    const normalized = path.replace(/\\/g, '/');
    const slash = normalized.lastIndexOf('/');
    return slash > 0 ? normalized.substring(0, slash) + '/' : null;
  }

  function handleFileContextMenu(e: MouseEvent, file: FileStatus) {
    e.preventDefault();
    contextMenu = { file, x: e.clientX, y: e.clientY };
  }

  function buildContextMenuItems(file: FileStatus): MenuItem[] {
    const items: MenuItem[] = [];
    const ext = fileExtension(file.path);
    const folder = parentFolder(file.path);

    // Revert / Unstage
    if (file.staging === 'Unstaged' && file.kind !== 'Untracked') {
      items.push({ id: 'revert', label: '還原變更' });
    } else if (file.staging === 'Staged') {
      items.push({
        id: 'revert',
        label: file.kind === 'Added' ? '取消追蹤' : '取消暫存',
      });
    }

    items.push({ id: '_sep1', label: '', separator: true });

    // Ignore options
    items.push({ id: 'ignore-file', label: `Ignore ${fileName(file.path)}` });
    if (ext) {
      items.push({ id: 'ignore-ext', label: `Ignore *${ext}` });
    }
    if (folder) {
      items.push({ id: 'ignore-folder', label: `Ignore ${folder}` });
    }

    items.push({ id: '_sep2', label: '', separator: true });

    // Utilities
    items.push({ id: 'copy-path', label: '複製路徑' });
    items.push({ id: 'open-editor', label: '在編輯器開啟' });

    // File history (only for tracked files)
    if (file.kind !== 'Untracked') {
      items.push({ id: '_sep3', label: '', separator: true });
      items.push({ id: 'file-history', label: '查看檔案歷史' });
    }

    return items;
  }

  async function handleContextSelect(actionId: string) {
    if (!contextMenu || !app.repoPath) return;
    const { file } = contextMenu;

    try {
      switch (actionId) {
        case 'revert':
          await gitCheckoutFile(app.repoPath, file.path, file.staging, file.kind);
          await app.refreshStatus();
          break;
        case 'ignore-file':
          await gitIgnore(app.repoPath, file.path);
          await app.refreshStatus();
          break;
        case 'ignore-ext': {
          const ext = fileExtension(file.path);
          if (ext) {
            await gitIgnore(app.repoPath, `*${ext}`);
            await app.refreshStatus();
          }
          break;
        }
        case 'ignore-folder': {
          const folder = parentFolder(file.path);
          if (folder) {
            await gitIgnore(app.repoPath, folder);
            await app.refreshStatus();
          }
          break;
        }
        case 'copy-path':
          await navigator.clipboard.writeText(file.path);
          app.addToast('已複製路徑', 'success');
          break;
        case 'open-editor':
          if (app.repoPath) {
            await openInEditor(app.repoPath + '/' + file.path, app.preferredEditor ?? undefined);
          }
          break;
        case 'file-history':
          app.showFileHistory(file.path);
          break;
      }
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
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
        oncontextmenu={(e) => handleFileContextMenu(e, file)}
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
        oncontextmenu={(e) => handleFileContextMenu(e, file)}
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

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={buildContextMenuItems(contextMenu.file)}
    onSelect={handleContextSelect}
    onClose={() => contextMenu = null}
  />
{/if}

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
