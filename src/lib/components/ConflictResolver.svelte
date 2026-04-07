<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import {
    gitResolveConflictContent,
    gitResolveConflictChoice,
    gitCompleteMerge,
    gitMergeAbort,
    gitStage,
    openInEditor,
  } from '$lib/git/commands';
  import type { ConflictSegment, ConflictHunk } from '$lib/git/types';

  // ── State ──
  let saving = $state(false);
  let completing = $state(false);
  let aborting = $state(false);
  /** Custom edited content per hunk index */
  let customEdits = $state<Record<number, string>>({});

  // ── Derived ──
  const files = $derived(app.conflictFiles);
  const activeFile = $derived(app.activeConflictFile);

  // Reset custom edits when switching files
  $effect(() => {
    if (activeFile) {
      customEdits = {};
    }
  });
  const content = $derived(app.conflictContent);
  const choices = $derived(app.hunkChoices);

  const hunks = $derived(() => {
    if (!content?.segments) return [];
    return content.segments.filter(
      (s): s is { type: 'Hunk'; value: ConflictHunk } => s.type === 'Hunk',
    );
  });

  const allHunksChosen = $derived(() => {
    const h = hunks();
    if (h.length === 0) return false;
    return h.every((s) => choices[s.value.index] !== undefined);
  });

  const activeFileInfo = $derived(() => {
    if (!activeFile) return null;
    return files.find((f) => f.path === activeFile) ?? null;
  });

  const resolvedCount = $derived(() => {
    // Files that are no longer in the conflict list are considered resolved
    // For simplicity, we track by checking how many files have been processed
    return 0; // Will be updated when files list shrinks after resolve + refresh
  });

  // ── Helpers ──
  function fileName(path: string): string {
    return path.split('/').pop() ?? path;
  }

  function assembleResolvedContent(): string {
    if (!content) return '';
    return content.segments
      .map((seg) => {
        if (seg.type === 'Context') return seg.value;
        const hunk = seg.value as ConflictHunk;
        const choice = choices[hunk.index];
        if (choice === 'Custom') return customEdits[hunk.index] ?? '';
        if (choice === 'Ours') return hunk.ours;
        if (choice === 'Theirs') return hunk.theirs;
        if (choice === 'Both') return hunk.ours + '\n' + hunk.theirs;
        return ''; // Should not happen if allHunksChosen
      })
      .join('\n');
  }

  function startCustomEdit(hunkIndex: number, ours: string, theirs: string) {
    // Pre-fill with both versions for user to edit/reorder
    customEdits = { ...customEdits, [hunkIndex]: ours + '\n' + theirs };
    app.setHunkChoice(hunkIndex, 'Custom');
  }

  function updateCustomEdit(hunkIndex: number, value: string) {
    customEdits = { ...customEdits, [hunkIndex]: value };
  }

  // ── Handlers ──
  async function handleSaveFile() {
    const path = app.repoPath;
    const file = activeFile;
    if (!path || !file || !content || saving) return;

    saving = true;
    try {
      const info = activeFileInfo();
      if (info?.is_binary || info?.conflict_type === 'DeleteModify' || info?.conflict_type === 'AddAdd') {
        // Binary / delete-modify: use choice command
        const choice = choices[0]; // Only one "hunk" for these types
        if (choice === 'Ours' || choice === 'Theirs') {
          await gitResolveConflictChoice(path, file, choice);
        }
      } else if (content.parse_error) {
        // Can't resolve malformed markers in-app
        return;
      } else {
        // Content conflict: assemble and write
        const resolved = assembleResolvedContent();
        await gitResolveConflictContent(path, file, resolved, content.content_hash);
      }

      // Stage the resolved file
      await gitStage(path, [file]);

      // Refresh conflict files list
      await app.refreshConflictFiles();

      // Auto-jump to next unresolved file
      const remaining = app.conflictFiles;
      if (remaining.length > 0) {
        await app.selectConflictFile(remaining[0].path);
      }

      app.addToast(`已解決 ${fileName(file)}`, 'success');
    } catch (e: unknown) {
      const msg = String(e);
      if (msg.includes('外部修改')) {
        // Reload content
        await app.selectConflictFile(file);
      }
      app.addToast(msg, 'error');
    } finally {
      saving = false;
    }
  }

  async function handleCompleteMerge() {
    const path = app.repoPath;
    if (!path || completing) return;

    completing = true;
    try {
      const result = await gitCompleteMerge(path);
      app.addToast(`Merge 完成 (${result.commit_hash.slice(0, 7)})`, 'success');
      app.exitConflictMode();
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      completing = false;
    }
  }

  async function handleAbort() {
    const path = app.repoPath;
    if (!path || aborting) return;

    aborting = true;
    try {
      await gitMergeAbort(path);
      app.addToast('已取消 merge', 'info');
      app.exitConflictMode();
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      aborting = false;
    }
  }

  async function handleOpenInEditor() {
    const path = app.repoPath;
    if (!path) return;
    try {
      await openInEditor(path, app.preferredEditor ?? undefined);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }
</script>

<div class="conflict-resolver">
  <!-- Left: Conflict file list -->
  <div class="conflict-sidebar">
    <div class="sidebar-header">
      <span class="section-label">CONFLICTS</span>
    </div>

    <!-- Progress bar -->
    {#if files.length > 0}
      <div class="progress-section">
        <div class="progress-bar">
          <div
            class="progress-fill"
            style="width: {files.length === 0 ? 0 : Math.max(0, 100 - (files.length / (files.length + 1)) * 100)}%"
          ></div>
        </div>
        <span class="progress-text">{files.length} 個待解決</span>
      </div>
    {/if}

    <!-- File list -->
    <div class="file-list">
      {#each files as file}
        <button
          class="file-item"
          class:active={activeFile === file.path}
          onclick={() => app.selectConflictFile(file.path)}
        >
          <span class="file-icon">⚠</span>
          <span class="file-path">{file.path}</span>
          {#if file.is_binary}
            <span class="file-tag">binary</span>
          {:else if file.conflict_type === 'DeleteModify'}
            <span class="file-tag">delete</span>
          {:else if file.conflict_type === 'AddAdd'}
            <span class="file-tag">add/add</span>
          {/if}
        </button>
      {/each}

      {#if files.length === 0}
        <div class="empty-state">
          <span class="empty-icon">✓</span>
          <span>全部衝突已解決</span>
        </div>
      {/if}
    </div>

    <!-- Bottom actions -->
    <div class="sidebar-actions">
      <button
        class="btn-complete"
        onclick={handleCompleteMerge}
        disabled={files.length > 0 || completing}
      >
        {#if completing}<span class="spinner"></span>{/if}
        完成 Merge
      </button>
      <button
        class="btn-abort"
        onclick={handleAbort}
        disabled={aborting}
      >
        取消 Merge
      </button>
    </div>
  </div>

  <!-- Center: Conflict content -->
  <div class="conflict-content">
    {#if !activeFile}
      <div class="content-empty">選擇一個衝突檔案</div>
    {:else if !content}
      <div class="content-loading"><span class="spinner"></span></div>
    {:else if content.parse_error}
      <!-- Malformed markers fallback -->
      <div class="content-header">
        <span class="content-path">{content.path}</span>
        <span class="content-error">無法解析衝突</span>
        <button class="btn-ghost" onclick={handleOpenInEditor}>在編輯器中開啟</button>
      </div>
      <div class="content-raw">
        <pre>{content.segments.length > 0 ? '' : '（檔案內容無法顯示）'}</pre>
      </div>
    {:else}
      <!-- Normal conflict content -->
      <div class="content-header">
        <span class="content-path">{content.path}</span>
        <span class="content-counter">
          {hunks().length} 個衝突
        </span>
        <div class="content-actions">
          <button class="btn-ghost" onclick={handleOpenInEditor}>在編輯器中開啟</button>
          <button
            class="btn-save"
            onclick={handleSaveFile}
            disabled={!allHunksChosen() || saving}
          >
            {#if saving}<span class="spinner-sm"></span>{/if}
            儲存
          </button>
        </div>
      </div>

      <div class="content-body">
        {#if activeFileInfo()?.is_binary}
          <!-- Binary conflict -->
          <div class="binary-conflict">
            <p>Binary 檔案衝突 — 請選擇要保留的版本</p>
            <div class="binary-actions">
              <button
                class="btn-ours"
                class:selected={choices[0] === 'Ours'}
                onclick={() => app.setHunkChoice(0, 'Ours')}
              >Accept Ours (HEAD)</button>
              <button
                class="btn-theirs"
                class:selected={choices[0] === 'Theirs'}
                onclick={() => app.setHunkChoice(0, 'Theirs')}
              >Accept Theirs</button>
            </div>
          </div>
        {:else if activeFileInfo()?.conflict_type === 'DeleteModify'}
          <!-- Delete-modify conflict -->
          <div class="binary-conflict">
            <p>一方刪除了此檔案，另一方修改了它</p>
            <div class="binary-actions">
              <button
                class="btn-ours"
                class:selected={choices[0] === 'Ours'}
                onclick={() => app.setHunkChoice(0, 'Ours')}
              >Accept Ours (HEAD)</button>
              <button
                class="btn-theirs"
                class:selected={choices[0] === 'Theirs'}
                onclick={() => app.setHunkChoice(0, 'Theirs')}
              >Accept Theirs</button>
            </div>
          </div>
        {:else}
          <!-- Content conflict with hunks -->
          {#each content.segments as segment, idx}
            {#if segment.type === 'Context'}
              <div class="context-block">
                <pre class="code-block">{segment.value}</pre>
              </div>
            {:else}
              {@const hunk = segment.value}
              {@const choice = choices[hunk.index]}
              <div class="hunk-block" role="group" aria-label="Conflict {hunk.index + 1}">
                <div class="hunk-label">Conflict {hunk.index + 1}</div>

                <!-- Ours section -->
                <div class="hunk-section ours" class:selected={choice === 'Ours'} class:dimmed={choice !== undefined && choice !== 'Ours'}>
                  <div class="section-header-row">
                    <span class="section-tag ours-tag">HEAD</span>
                    <button
                      class="btn-accept ours-btn"
                      onclick={() => app.setHunkChoice(hunk.index, 'Ours')}
                    >Accept Ours</button>
                  </div>
                  <pre class="code-block">{hunk.ours || '(empty)'}</pre>
                </div>

                <!-- Accept Both / Custom Edit buttons -->
                <div class="hunk-divider">
                  <button
                    class="btn-accept-both"
                    class:selected={choice === 'Both'}
                    onclick={() => app.setHunkChoice(hunk.index, 'Both')}
                  >Accept Both</button>
                  <button
                    class="btn-accept-both"
                    class:selected={choice === 'Custom'}
                    onclick={() => startCustomEdit(hunk.index, hunk.ours, hunk.theirs)}
                  >Custom Edit</button>
                </div>

                <!-- Theirs section -->
                <div class="hunk-section theirs" class:selected={choice === 'Theirs'} class:dimmed={choice !== undefined && choice !== 'Theirs'}>
                  <div class="section-header-row">
                    <span class="section-tag theirs-tag">THEIRS</span>
                    <button
                      class="btn-accept theirs-btn"
                      onclick={() => app.setHunkChoice(hunk.index, 'Theirs')}
                    >Accept Theirs</button>
                  </div>
                  <pre class="code-block">{hunk.theirs || '(empty)'}</pre>
                </div>

                <!-- Custom edit textarea -->
                {#if choice === 'Custom'}
                  <div class="custom-edit-section">
                    <div class="section-header-row">
                      <span class="section-tag custom-tag">CUSTOM</span>
                      <span class="custom-hint">自由編輯、重新排列行的順序</span>
                    </div>
                    <textarea
                      class="custom-textarea"
                      value={customEdits[hunk.index] ?? ''}
                      oninput={(e) => updateCustomEdit(hunk.index, e.currentTarget.value)}
                      rows={Math.max(4, (customEdits[hunk.index] ?? '').split('\n').length + 1)}
                    ></textarea>
                  </div>
                {/if}

                <!-- diff3 base (collapsible) -->
                {#if hunk.base !== null}
                  <details class="base-section">
                    <summary class="base-toggle">Show base</summary>
                    <div class="hunk-section base">
                      <span class="section-tag base-tag">BASE</span>
                      <pre class="code-block">{hunk.base}</pre>
                    </div>
                  </details>
                {/if}
              </div>
            {/if}
          {/each}
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .conflict-resolver {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  /* ── Sidebar ── */
  .conflict-sidebar {
    width: 240px;
    min-width: 180px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 0;
  }

  .sidebar-header {
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }

  .section-label {
    font-size: 10px;
    font-family: var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
  }

  .progress-section {
    padding: var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--border);
  }

  .progress-bar {
    height: 4px;
    background: var(--bg-hover);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    display: block;
    margin-top: var(--space-xs);
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
  }

  .file-item {
    display: flex;
    align-items: center;
    padding: 4px var(--space-md);
    gap: var(--space-xs);
    min-height: 28px;
    width: 100%;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
  }

  .file-item:hover { background: var(--bg-hover); }
  .file-item.active {
    background: var(--bg-surface);
    border-left: 2px solid var(--accent);
  }

  .file-icon { color: var(--warning); font-size: 12px; flex-shrink: 0; }

  .file-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-tag {
    font-size: 10px;
    color: var(--text-muted);
    padding: 0 4px;
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .empty-state {
    padding: var(--space-lg) var(--space-md);
    text-align: center;
    color: var(--success);
    font-size: var(--font-size-sm);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
  }
  .empty-icon { font-size: 24px; }

  .sidebar-actions {
    padding: var(--space-sm) var(--space-md);
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .btn-complete {
    background: var(--accent);
    color: var(--bg-primary);
    border: none;
    border-radius: var(--radius-sm);
    padding: var(--space-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-xs);
  }
  .btn-complete:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-complete:hover:not(:disabled) { filter: brightness(1.1); }

  .btn-abort {
    background: none;
    color: var(--error);
    border: 1px solid var(--error);
    border-radius: var(--radius-sm);
    padding: var(--space-xs);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    cursor: pointer;
    text-align: center;
  }
  .btn-abort:hover { background: rgba(255, 107, 107, 0.1); }
  .btn-abort:disabled { opacity: 0.5; cursor: not-allowed; }

  /* ── Content ── */
  .conflict-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 300px;
  }

  .content-empty, .content-loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: var(--font-size-md);
  }

  .content-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-md);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    min-height: 28px;
    flex-shrink: 0;
  }

  .content-path {
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    color: var(--text-primary);
  }

  .content-counter {
    font-size: 11px;
    color: var(--text-muted);
  }

  .content-error {
    font-size: 11px;
    color: var(--error);
  }

  .content-actions {
    margin-left: auto;
    display: flex;
    gap: var(--space-xs);
  }

  .btn-ghost {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 12px;
    font-family: var(--font-ui);
    cursor: pointer;
    padding: var(--space-xs) var(--space-sm);
  }
  .btn-ghost:hover { color: var(--text-primary); }

  .btn-save {
    background: var(--accent);
    color: var(--bg-primary);
    border: none;
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
  .btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-save:hover:not(:disabled) { filter: brightness(1.1); }

  .content-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-sm);
  }

  .content-raw {
    flex: 1;
    overflow: auto;
    padding: var(--space-md);
  }

  /* ── Code blocks ── */
  .code-block {
    margin: 0;
    padding: var(--space-sm);
    font-family: var(--font-mono);
    font-size: var(--font-size-md);
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .context-block .code-block {
    color: var(--text-secondary);
    background: var(--bg-primary);
  }

  /* ── Hunk blocks ── */
  .hunk-block {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    margin: var(--space-sm) 0;
    overflow: hidden;
  }

  .hunk-label {
    font-size: 10px;
    font-family: var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    padding: var(--space-xs) var(--space-sm);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
  }

  .hunk-section { transition: opacity 0.15s ease; }
  .hunk-section.dimmed { opacity: 0.4; }
  .hunk-section.selected { border-left: 2px solid var(--accent); }

  .hunk-section.ours .code-block { background: var(--diff-add-bg); }
  .hunk-section.theirs .code-block { background: var(--diff-del-bg); }
  .hunk-section.base .code-block { background: var(--bg-hover); }

  .section-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-xs) var(--space-sm);
  }

  .section-tag {
    font-size: 10px;
    font-family: var(--font-mono);
    text-transform: uppercase;
    padding: 1px 6px;
    border-radius: var(--radius-sm);
  }

  .ours-tag { color: var(--diff-add-text); background: rgba(46, 160, 67, 0.2); }
  .theirs-tag { color: var(--diff-del-text); background: rgba(248, 81, 73, 0.2); }
  .base-tag { color: var(--text-muted); background: var(--bg-hover); }

  .btn-accept {
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 2px var(--space-sm);
    font-size: 11px;
    font-family: var(--font-ui);
    cursor: pointer;
    color: var(--text-secondary);
  }
  .btn-accept:hover { background: var(--bg-hover); color: var(--text-primary); }
  .ours-btn:hover { color: var(--diff-add-text); border-color: var(--diff-add-text); }
  .theirs-btn:hover { color: var(--diff-del-text); border-color: var(--diff-del-text); }

  .hunk-divider {
    display: flex;
    justify-content: center;
    padding: var(--space-xs);
    border-top: 1px dashed var(--border);
    border-bottom: 1px dashed var(--border);
  }

  .btn-accept-both {
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 2px var(--space-sm);
    font-size: 11px;
    font-family: var(--font-ui);
    cursor: pointer;
    color: var(--text-muted);
  }
  .btn-accept-both:hover { color: var(--text-primary); background: var(--bg-hover); }
  .btn-accept-both.selected { color: var(--accent); border-color: var(--accent); }

  .custom-edit-section {
    border-top: 1px solid var(--border);
  }
  .custom-tag {
    color: var(--accent);
    background: rgba(79, 193, 255, 0.15);
  }
  .custom-hint {
    font-size: 10px;
    color: var(--text-muted);
    font-style: italic;
  }
  .custom-textarea {
    width: 100%;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: none;
    border-top: 1px solid var(--border);
    padding: var(--space-sm);
    font-family: var(--font-mono);
    font-size: var(--font-size-md);
    line-height: 1.6;
    resize: vertical;
    outline: none;
    min-height: 80px;
  }
  .custom-textarea:focus {
    box-shadow: inset 0 0 0 1px var(--accent);
  }

  .base-section { margin: 0; }
  .base-toggle {
    font-size: 11px;
    font-family: var(--font-ui);
    color: var(--text-muted);
    padding: var(--space-xs) var(--space-sm);
    cursor: pointer;
    user-select: none;
  }
  .base-toggle:hover { color: var(--text-secondary); }

  .binary-conflict {
    padding: var(--space-lg);
    text-align: center;
    color: var(--text-secondary);
  }
  .binary-conflict p { margin-bottom: var(--space-md); }
  .binary-actions { display: flex; gap: var(--space-sm); justify-content: center; }

  .btn-ours, .btn-theirs {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-sm) var(--space-md);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    cursor: pointer;
    color: var(--text-primary);
  }
  .btn-ours:hover { border-color: var(--diff-add-text); color: var(--diff-add-text); }
  .btn-theirs:hover { border-color: var(--diff-del-text); color: var(--diff-del-text); }
  .btn-ours.selected { background: var(--diff-add-bg); border-color: var(--diff-add-text); color: var(--diff-add-text); }
  .btn-theirs.selected { background: var(--diff-del-bg); border-color: var(--diff-del-text); color: var(--diff-del-text); }

  .spinner, .spinner-sm {
    display: inline-block;
    border: 2px solid var(--text-muted);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  .spinner { width: 12px; height: 12px; }
  .spinner-sm { width: 10px; height: 10px; border-width: 1.5px; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
