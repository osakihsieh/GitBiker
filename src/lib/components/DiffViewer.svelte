<script lang="ts">
  import { app } from '$lib/stores/app.svelte';

  function fileName(path: string): string {
    return path.replace(/\\/g, '/');
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
