<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitStage, gitUnstage, gitCommit, gitIgnore, gitCheckoutFile, openInEditor, gitStashPushFiles, generateCommitMessage } from '$lib/git/commands';
  import type { FileStatus } from '$lib/git/types';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';

  let commitTitle = $state('');
  let commitBody = $state('');
  let committing = $state(false);
  let generating = $state(false);
  let commitType = $state('auto');

  const COMMIT_TYPES = ['auto', 'feat', 'fix', 'refactor', 'docs', 'test', 'chore', 'perf', 'ci'] as const;
  const TYPE_PREFIX_RE = /^(feat|fix|refactor|docs|test|chore|perf|ci):\s*/;
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

  function buildCommitMessage(): string {
    const title = commitTitle.trim();
    const body = commitBody.trim();
    if (!body) return title;
    return `${title}\n\n${body}`;
  }

  async function handleCommit() {
    if (!app.repoPath || !commitTitle.trim() || committing) return;
    committing = true;
    try {
      const hash = await gitCommit(app.repoPath, buildCommitMessage());
      app.addToast(`Committed ${hash.substring(0, 7)}`, 'success');
      commitTitle = '';
      commitBody = '';
      commitType = 'auto';
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      committing = false;
    }
  }

  async function handleAiGenerate() {
    if (!app.repoPath || app.stagedFiles.length === 0 || generating) return;

    // Validate API key for non-Ollama providers
    if (app.aiProvider !== 'ollama' && !app.aiApiKey.trim()) {
      app.addToast('請先在設定中填入 API Key', 'error');
      return;
    }

    generating = true;
    try {
      const message = await generateCommitMessage({
        path: app.repoPath,
        provider: app.aiProvider,
        apiKey: app.aiApiKey,
        model: app.aiModel,
        language: app.aiLanguage,
        customPrompt: app.aiCustomPrompt || undefined,
        ollamaEndpoint: app.aiProvider === 'ollama' ? app.aiOllamaEndpoint : undefined,
        commitType: commitType !== 'auto' ? commitType : undefined,
      });
      // Split AI response into title + body at first blank line
      const blankIdx = message.indexOf('\n\n');
      if (blankIdx !== -1) {
        commitTitle = message.substring(0, blankIdx).trim();
        commitBody = message.substring(blankIdx + 2).trim();
      } else {
        commitTitle = message.trim();
        commitBody = '';
      }
      app.addToast('AI 已生成 commit message', 'success');
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      generating = false;
    }
  }

  function handleCommitTypeChange(newType: string) {
    commitType = newType;

    // If there's already a title, swap the prefix
    if (commitTitle.trim() && newType !== 'auto') {
      const match = commitTitle.match(TYPE_PREFIX_RE);
      if (match) {
        commitTitle = commitTitle.replace(TYPE_PREFIX_RE, `${newType}: `);
      } else {
        commitTitle = `${newType}: ${commitTitle}`;
      }
    }
    // If switching back to auto and title has a prefix, leave it as-is
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

    // Stash options
    items.push({ id: 'stash-file', label: 'Stash 這個檔案' });

    items.push({ id: '_sep3', label: '', separator: true });

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
        case 'stash-file':
          if (app.repoPath) {
            await gitStashPushFiles(app.repoPath, [file.path], `Stash file: ${file.path}`);
            app.addToast(`已 Stash ${file.path}`, 'success');
            await app.refreshAll();
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
  <!-- File changes summary -->
  {#if app.stagedFiles.length + app.unstagedFiles.length > 0}
    <div class="changes-badge">
      <span class="changes-count">{app.stagedFiles.length + app.unstagedFiles.length}</span> file change{app.stagedFiles.length + app.unstagedFiles.length !== 1 ? 's' : ''} on <span class="changes-branch">{app.currentBranch}</span>
    </div>
  {/if}

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
    <div class="commit-title-row">
      <select
        class="commit-type-select"
        value={commitType}
        onchange={(e) => handleCommitTypeChange((e.target as HTMLSelectElement).value)}
      >
        {#each COMMIT_TYPES as t}
          <option value={t}>{t === 'auto' ? 'auto' : `${t}:`}</option>
        {/each}
      </select>
      <input
        type="text"
        class="commit-title-input"
        bind:value={commitTitle}
        placeholder="簡述變更"
        onkeydown={handleKeydown}
      />
    </div>
    <textarea
      class="commit-body-input"
      bind:value={commitBody}
      placeholder="詳細說明（選填）"
      onkeydown={handleKeydown}
    ></textarea>
    <div class="commit-actions">
      <button
        class="ai-gen-btn"
        onclick={handleAiGenerate}
        disabled={app.stagedFiles.length === 0 || generating || committing}
      >
        {#if generating}
          <span class="spinner"></span> 生成中...
        {:else}
          AI 生成
        {/if}
      </button>
      <button
        class="commit-btn"
        onclick={handleCommit}
        disabled={!commitTitle.trim() || committing || app.stagedFiles.length === 0}
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
  .changes-badge {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    font-size: 12px;
    font-family: var(--font-ui);
    color: var(--text-secondary);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .changes-count {
    font-weight: 700;
    color: var(--accent);
  }
  .changes-branch {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--text-primary);
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
  .commit-title-row {
    display: flex;
    align-items: stretch;
  }
  .commit-type-select {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm) 0 0 0;
    border-right: none;
    color: var(--text-secondary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    padding: var(--space-xs) var(--space-xs);
    outline: none;
    cursor: pointer;
    flex-shrink: 0;
    border-bottom: none;
  }
  .commit-type-select:focus { border-color: var(--accent); }
  .commit-title-input {
    width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0 var(--radius-sm) 0 0;
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    font-weight: 600;
    padding: var(--space-sm);
    outline: none;
    border-bottom: none;
  }
  .commit-title-input:focus { border-color: var(--accent); }
  .commit-title-row:focus-within + .commit-body-input { border-color: var(--accent); }
  .commit-title-input::placeholder { color: var(--text-muted); font-weight: 400; }
  .commit-body-input {
    width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0 0 var(--radius-sm) var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    padding: var(--space-sm);
    resize: vertical;
    min-height: 48px;
    max-height: 160px;
    outline: none;
  }
  .commit-body-input:focus { border-color: var(--accent); }
  .commit-body-input::placeholder { color: var(--text-muted); }
  .commit-actions {
    margin-top: var(--space-sm);
    display: flex;
    gap: var(--space-xs);
  }
  .ai-gen-btn {
    background: var(--bg-surface);
    color: var(--accent);
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    padding: var(--space-sm) var(--space-md);
    font-size: var(--font-size-sm);
    font-weight: 600;
    cursor: pointer;
    font-family: var(--font-ui);
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
  .ai-gen-btn:hover:not(:disabled) { background: var(--bg-hover); }
  .ai-gen-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .commit-btn {
    flex: 1;
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
