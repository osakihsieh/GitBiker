<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitRebaseInteractive } from '$lib/git/commands';
  import { extractErrorMessage } from '$lib/utils/error';
  import type { RebaseAction, RebaseCommit } from '$lib/git/types';
  import UiButton from '$lib/components/ui/button.svelte';

  let commits = $state<RebaseCommit[]>([]);
  let rebaseRunning = $state(false);

  // Sync with store
  $effect(() => {
    commits = [...app.rebaseCommits];
  });

  const actions: RebaseAction[] = ['Pick', 'Reword', 'Edit', 'Squash', 'Fixup', 'Drop'];

  let dragIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  function handleDragStart(i: number) {
    dragIndex = i;
  }

  function handleDragOver(e: DragEvent, i: number) {
    e.preventDefault();
    dragOverIndex = i;
  }

  function handleDrop(targetIdx: number) {
    if (dragIndex === null || dragIndex === targetIdx) return;
    const items = [...commits];
    const [moved] = items.splice(dragIndex, 1);
    items.splice(targetIdx, 0, moved);
    commits = items;
    dragIndex = null;
    dragOverIndex = null;
  }

  async function handleConfirm() {
    if (!app.repoPath || !app.rebaseBase) return;
    rebaseRunning = true;
    try {
      const result = await gitRebaseInteractive(app.repoPath, app.rebaseBase, commits);
      if (result.success) {
        app.addToast('Rebase 成功', 'success');
        app.viewMode = 'worktree';
        await app.refreshAll();
      } else if (result.conflicts.length > 0) {
        app.addToast('Rebase 衝突，請手動解決', 'error');
        app.viewMode = 'conflict-resolution';
        await app.refreshStatus();
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      rebaseRunning = false;
    }
  }

  function handleCancel() {
    app.viewMode = 'worktree';
  }
</script>

<div class="rebase-editor">
  <div class="header">
    <div class="title">互動式 Rebase</div>
    <div class="subtitle">正在 Rebase 到 {app.rebaseBase?.substring(0, 7)}</div>
    <div class="actions">
      <UiButton variant="secondary" onclick={handleCancel} disabled={rebaseRunning}>取消</UiButton>
      <UiButton onclick={handleConfirm} loading={rebaseRunning}>執行 Rebase</UiButton>
    </div>
  </div>

  <div class="commit-list">
    {#each commits as commit, i (commit.id)}
      <div
        class="rebase-item"
        class:dragging={dragIndex === i}
        class:drag-over={dragOverIndex === i && dragIndex !== i}
        draggable="true"
        ondragstart={() => handleDragStart(i)}
        ondragover={(e) => handleDragOver(e, i)}
        ondrop={() => handleDrop(i)}
      >
        <div class="drag-handle">⋮⋮</div>
        <select bind:value={commit.action} class="action-select">
          {#each actions as action}
            <option value={action}>{action}</option>
          {/each}
        </select>
        <span class="hash">{commit.id.substring(0, 7)}</span>
        <span class="message" title={commit.message}>{commit.message.split('\n')[0]}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .rebase-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }
  .header {
    padding: var(--space-md);
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .title {
    font-size: 18px;
    font-weight: 600;
  }
  .subtitle {
    font-size: 12px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
  .actions {
    display: flex;
    gap: var(--space-sm);
    margin-top: var(--space-sm);
  }
  .commit-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-sm);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .rebase-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 6px var(--space-sm);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: default;
  }
  .rebase-item.dragging {
    opacity: 0.5;
  }
  .rebase-item.drag-over {
    border-top: 2px solid var(--accent);
  }
  .drag-handle {
    cursor: grab;
    color: var(--text-muted);
    user-select: none;
  }
  .action-select {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 11px;
    padding: 2px 4px;
    outline: none;
  }
  .hash {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--accent);
    min-width: 60px;
  }
  .message {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }
</style>
