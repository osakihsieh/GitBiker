<script lang="ts">
  import { extractErrorMessage } from '$lib/utils/error';
  import { app } from '$lib/stores/app.svelte';
  import { gitStageHunk, gitUnstageHunk, gitStashHunk } from '$lib/git/commands';
  import type { DiffHunk } from '$lib/git/types';

  function fileName(path: string): string {
    return path.replace(/\\/g, '/');
  }

  /** Check if the currently selected file is in the staged list */
  const isStaged = $derived(
    app.selectedFile != null &&
    app.stagedFiles.some((f) => f.path === app.selectedFile)
  );

  /** Check if we're in worktree mode (hunk staging only makes sense here) */
  const canHunkStage = $derived(app.viewMode === 'worktree' && app.currentDiff != null);

  /**
   * Build a unified diff patch for a single hunk.
   * This is what `git apply --cached` expects.
   */
  function buildHunkPatch(hunk: DiffHunk): string {
    const diff = app.currentDiff;
    if (!diff) return '';

    const filePath = fileName(diff.file_path);
    const lines: string[] = [
      `diff --git a/${filePath} b/${filePath}`,
      `--- a/${filePath}`,
      `+++ b/${filePath}`,
      hunk.header,
    ];

    for (const line of hunk.lines) {
      if (line.kind === 'Addition') {
        lines.push(`+${line.content}`);
      } else if (line.kind === 'Deletion') {
        lines.push(`-${line.content}`);
      } else if (line.kind === 'Context') {
        lines.push(` ${line.content}`);
      }
      // Skip 'Header' lines — they are part of the hunk header already
    }

    // Ensure trailing newline
    return lines.join('\n') + '\n';
  }

  async function handleStageHunk(hunk: DiffHunk) {
    if (!app.repoPath || !app.currentDiff) return;
    try {
      const patch = buildHunkPatch(hunk);
      await gitStageHunk(app.repoPath, patch);
      app.addToast('Hunk 已加入暫存區', 'success');
      await app.refreshStatus();
      // Reload diff to reflect updated state
      if (app.selectedFile) {
        await app.loadDiff(app.selectedFile);
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleStashHunk(hunk: DiffHunk) {
    if (!app.repoPath || !app.currentDiff) return;
    try {
      const patch = buildHunkPatch(hunk);
      await gitStashHunk(app.repoPath, patch);
      app.addToast('Hunk 已 stash', 'success');
      await app.refreshStatus();
      if (app.selectedFile) {
        await app.loadDiff(app.selectedFile);
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleUnstageHunk(hunk: DiffHunk) {
    if (!app.repoPath || !app.currentDiff) return;
    try {
      const patch = buildHunkPatch(hunk);
      await gitUnstageHunk(app.repoPath, patch);
      app.addToast('Hunk 已移出暫存區', 'success');
      await app.refreshStatus();
      if (app.selectedFile) {
        await app.loadDiff(app.selectedFile);
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }
</script>

<div class="diff-panel">
  {#if app.currentDiff}
    <div class="diff-header">
      <span class="filepath">{fileName(app.currentDiff.file_path)}</span>
      <div class="stats">
        <span class="add">+{app.currentDiff.stats.additions}</span>
        <span class="del">-{app.currentDiff.stats.deletions}</span>
      </div>
    </div>
    <div class="diff-content">
      {#if app.currentDiff.is_binary}
        <div class="diff-empty">Binary file — cannot display diff</div>
      {:else if app.currentDiff.is_truncated}
        <div class="diff-warning">File exceeds 10MB — diff truncated</div>
      {/if}
      {#each app.currentDiff.hunks as hunk}
        <div class="diff-line hunk">
          <span class="line-num"></span>
          <span class="line-num-new"></span>
          <span class="code">{hunk.header}</span>
          {#if canHunkStage}
            <span class="hunk-actions">
              {#if isStaged}
                <button
                  class="hunk-btn unstage"
                  onclick={() => handleUnstageHunk(hunk)}
                  title="Unstage 此 hunk"
                >−</button>
              {:else}
                <button
                  class="hunk-btn stage"
                  onclick={() => handleStageHunk(hunk)}
                  title="Stage 此 hunk"
                >+</button>
                <button
                  class="hunk-btn stash"
                  onclick={() => handleStashHunk(hunk)}
                  title="Stash 此 hunk"
                >⊟</button>
              {/if}
            </span>
          {/if}
        </div>
        {#each hunk.lines as line}
          <div
            class="diff-line"
            class:add={line.kind === 'Addition'}
            class:del={line.kind === 'Deletion'}
          >
            <span class="line-num">{line.old_lineno ?? ''}</span>
            <span class="line-num-new">{line.new_lineno ?? ''}</span>
            <span class="code">{line.content}</span>
          </div>
        {/each}
      {/each}
    </div>
  {:else}
    <div class="diff-empty-state">
      <div class="empty-icon">◇</div>
      <div class="empty-text">Select a file to view diff</div>
    </div>
  {/if}
</div>

<style>
  .diff-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    min-width: 300px;
  }
  .diff-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    font-size: var(--font-size-sm);
    flex-shrink: 0;
  }
  .filepath { font-family: var(--font-mono); }
  .stats {
    margin-left: auto;
    display: flex;
    gap: var(--space-sm);
    font-size: 11px;
    font-family: var(--font-mono);
  }
  .add { color: var(--diff-add-text); }
  .del { color: var(--diff-del-text); }
  .diff-content {
    flex: 1;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: var(--font-size-md);
    line-height: 1.6;
  }
  .diff-line {
    display: flex;
    white-space: pre;
    min-height: 21px;
  }
  .line-num, .line-num-new {
    width: 40px;
    min-width: 40px;
    text-align: right;
    padding-right: var(--space-sm);
    color: var(--text-muted);
    user-select: none;
    font-size: var(--font-size-sm);
    flex-shrink: 0;
  }
  .line-num-new {
    border-right: 1px solid var(--border);
    margin-right: var(--space-sm);
  }
  .code {
    flex: 1;
    padding-left: var(--space-sm);
  }
  .diff-line.add { background: var(--diff-add-bg); }
  .diff-line.add .code { color: var(--diff-add-text); }
  .diff-line.del { background: var(--diff-del-bg); }
  .diff-line.del .code { color: var(--diff-del-text); }
  .diff-line.hunk {
    background: var(--bg-surface);
    color: var(--accent);
    padding: var(--space-xs) 0;
    font-size: var(--font-size-sm);
  }
  .diff-line.hunk .code { color: var(--accent); }

  /* Hunk staging buttons */
  .hunk-actions {
    display: flex;
    align-items: center;
    margin-right: var(--space-sm);
    flex-shrink: 0;
  }
  .hunk-btn {
    width: 22px;
    height: 22px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    font-weight: 700;
    line-height: 1;
    padding: 0;
    transition: background 0.15s, border-color 0.15s;
  }
  .hunk-btn.stage {
    color: var(--diff-add-text);
  }
  .hunk-btn.stage:hover {
    background: var(--diff-add-bg);
    border-color: var(--diff-add-text);
  }
  .hunk-btn.unstage {
    color: var(--diff-del-text);
  }
  .hunk-btn.unstage:hover {
    background: var(--diff-del-bg);
    border-color: var(--diff-del-text);
  }
  .hunk-btn.stash {
    color: var(--accent);
  }
  .hunk-btn.stash:hover {
    background: var(--bg-surface);
    border-color: var(--accent);
  }

  .diff-empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    color: var(--text-muted);
  }
  .empty-icon { font-size: 32px; opacity: 0.3; }
  .empty-text { font-size: var(--font-size-sm); }
  .diff-empty, .diff-warning {
    padding: var(--space-lg);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }
  .diff-warning { color: var(--warning); }
</style>
