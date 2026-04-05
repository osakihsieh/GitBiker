<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import {
    gitBranches,
    gitCreateBranch,
    gitDeleteBranch,
    gitRenameBranch,
    gitCheckoutRemoteBranch,
    gitBranchMergeStatus,
    gitSwitchBranch,
    gitPush,
    gitMergeBranch,
    gitMergeAbort,
    gitMergeDryRun,
  } from '$lib/git/commands';
  import { clickOutside } from '$lib/actions/clickOutside';
  import { slugifyBranchName } from '$lib/utils/slugify';
  import type { Branch, BranchMergeStatus } from '$lib/git/types';

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  // ── State ──
  let searchQuery = $state('');
  let createExpanded = $state(false);
  let createName = $state('');
  let creating = $state(false);
  let staleExpanded = $state(false);
  let selectedIndex = $state(-1);
  let renamingBranch = $state<string | null>(null);
  let renameValue = $state('');
  let renameError = $state('');
  let confirmDelete = $state<{ name: string; mergeStatus: BranchMergeStatus | null } | null>(null);
  let deleting = $state(false);
  let batchCleaning = $state(false);
  let batchProgress = $state('');
  let pushingBranch = $state<string | null>(null);
  let mergingBranch = $state<string | null>(null);
  let mergeConflicts = $state<string[]>([]);
  let dryRunResult = $state<{ branch: string; conflicts: string[] } | null>(null);

  const STALE_DAYS = 30;

  // ── Derived ──
  const slugPreview = $derived(slugifyBranchName(createName));
  const canCreate = $derived(slugPreview.length > 0 && !creating);

  const localBranches = $derived(() => {
    const q = searchQuery.toLowerCase().trim();
    return app.branches
      .filter((b) => !b.is_remote)
      .filter((b) => !q || b.name.toLowerCase().includes(q));
  });

  const remoteBranches = $derived(() => {
    const q = searchQuery.toLowerCase().trim();
    const localNames = new Set(app.branches.filter((b) => !b.is_remote).map((b) => b.name));
    return app.branches
      .filter((b) => b.is_remote)
      .filter((b) => {
        // Hide remote branches that already have a local tracking branch
        const localName = b.name.split('/').slice(1).join('/');
        return !localNames.has(localName);
      })
      .filter((b) => !q || b.name.toLowerCase().includes(q));
  });

  const staleBranches = $derived(() => {
    const now = Math.floor(Date.now() / 1000);
    const threshold = now - STALE_DAYS * 86400;
    const q = searchQuery.toLowerCase().trim();
    return app.branches
      .filter((b) => !b.is_remote && !b.is_current)
      .filter((b) => b.last_commit_timestamp !== null && b.last_commit_timestamp < threshold)
      .filter((b) => !q || b.name.toLowerCase().includes(q));
  });

  // ── Helpers ──
  function daysAgo(timestamp: number | null): string {
    if (timestamp === null) return '?';
    const days = Math.floor((Date.now() / 1000 - timestamp) / 86400);
    if (days === 0) return 'today';
    if (days === 1) return '1d ago';
    return `${days}d ago`;
  }

  // ── Handlers ──
  async function handleCreate() {
    if (!app.repoPath || !canCreate) return;
    creating = true;
    try {
      await gitCreateBranch(app.repoPath, slugPreview);
      app.currentBranch = slugPreview;
      createName = '';
      createExpanded = false;
      app.addToast(`已建立並切換到 ${slugPreview}`, 'success');
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      creating = false;
    }
  }

  async function handleDelete(name: string) {
    if (!app.repoPath) return;
    try {
      const status = await gitBranchMergeStatus(app.repoPath, name);
      if (!status.merged) {
        confirmDelete = { name, mergeStatus: status };
        return;
      }
      // Merged: safe delete
      deleting = true;
      await gitDeleteBranch(app.repoPath, name);
      app.addToast(`已刪除 ${name}`, 'success');
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      deleting = false;
    }
  }

  async function handleForceDelete() {
    if (!app.repoPath || !confirmDelete) return;
    deleting = true;
    try {
      await gitDeleteBranch(app.repoPath, confirmDelete.name, true);
      app.addToast(`已強制刪除 ${confirmDelete.name}`, 'success');
      confirmDelete = null;
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      deleting = false;
    }
  }

  function startRename(name: string) {
    renamingBranch = name;
    renameValue = name;
    renameError = '';
  }

  async function handleRename() {
    if (!app.repoPath || !renamingBranch) return;
    const newName = slugifyBranchName(renameValue);
    if (!newName || newName === renamingBranch) {
      renamingBranch = null;
      return;
    }
    try {
      const wasCurrent = app.currentBranch === renamingBranch;
      await gitRenameBranch(app.repoPath, renamingBranch, newName);
      if (wasCurrent) {
        app.currentBranch = newName;
      }
      // Check if the renamed branch had upstream
      const branch = app.branches.find((b) => b.name === renamingBranch);
      if (branch?.upstream) {
        app.addToast(
          `已重命名為 ${newName}。注意：upstream tracking 仍指向舊名稱，下次 push 時會自動建立新的 remote branch。`,
          'info',
        );
      }
      renamingBranch = null;
      await app.refreshAll();
    } catch (e: unknown) {
      renameError = String(e);
    }
  }

  async function handleCheckoutRemote(remoteName: string) {
    if (!app.repoPath) return;
    try {
      const localName = await gitCheckoutRemoteBranch(app.repoPath, remoteName);
      app.currentBranch = localName;
      app.addToast(`已 checkout ${localName}`, 'success');
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleSwitchBranch(name: string) {
    if (!app.repoPath) return;
    try {
      await gitSwitchBranch(app.repoPath, name);
      app.currentBranch = name;
      app.addToast(`已切換到 ${name}`, 'success');
      onClose();
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handlePushBranch(name: string) {
    if (!app.repoPath || pushingBranch) return;
    pushingBranch = name;
    try {
      const result = await gitPush(app.repoPath, 'origin', name);
      if (result.success) {
        app.addToast(`已 push 到 origin/${name}`, 'success');
        await app.refreshAll();
      } else {
        app.addToast(result.message, 'error', false);
      }
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      pushingBranch = null;
    }
  }

  async function handleBatchCleanStale() {
    if (!app.repoPath) return;
    const mergedStale = [];
    for (const b of staleBranches()) {
      try {
        const status = await gitBranchMergeStatus(app.repoPath, b.name);
        if (status.merged) mergedStale.push(b.name);
      } catch {
        // skip
      }
    }
    if (mergedStale.length === 0) {
      app.addToast('沒有已 merge 的 stale branches 可清理', 'info');
      return;
    }
    // Confirm before batch delete
    const confirmed = true; // Inline confirm handled by UI state
    if (!confirmed) return;

    batchCleaning = true;
    let deleted = 0;
    const failures: string[] = [];
    for (let i = 0; i < mergedStale.length; i++) {
      batchProgress = `刪除中 (${i + 1}/${mergedStale.length})...`;
      try {
        await gitDeleteBranch(app.repoPath!, mergedStale[i]);
        deleted++;
      } catch (e: unknown) {
        failures.push(`${mergedStale[i]}: ${String(e)}`);
      }
    }
    batchCleaning = false;
    batchProgress = '';

    if (failures.length === 0) {
      app.addToast(`已清理 ${deleted} 個 stale branches`, 'success');
    } else {
      app.addToast(
        `已刪除 ${deleted} 個，${failures.length} 個失敗：${failures.join('; ')}`,
        'error',
        false,
      );
    }
    await app.refreshAll();
  }

  async function handleMerge(name: string) {
    if (!app.repoPath || mergingBranch) return;
    mergingBranch = name;
    mergeConflicts = [];
    dryRunResult = null;

    try {
      // Step 1: Dry-run preview
      const preview = await gitMergeDryRun(app.repoPath, name);
      if (preview.method !== 'skipped' && preview.has_conflicts) {
        // Show dry-run preview dialog
        dryRunResult = { branch: name, conflicts: preview.conflict_files };
        mergingBranch = null;
        return;
      }

      // Step 2: Execute merge (no conflicts predicted, or dry-run skipped)
      await executeMerge(name);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
      mergingBranch = null;
    }
  }

  async function executeMerge(name: string) {
    if (!app.repoPath) return;
    mergingBranch = name;
    dryRunResult = null;
    try {
      const result = await gitMergeBranch(app.repoPath, name);
      if (result.success) {
        app.addToast(`已將 ${name} merge 到 ${app.currentBranch}`, 'success');
        onClose();
      } else {
        mergeConflicts = result.conflicts;
        app.addToast(`Merge 衝突：${result.conflicts.length} 個檔案需要解決`, 'error', false);
        await app.refreshAll();
        // Enter conflict mode
        onClose();
        await app.enterConflictMode();
        return;
      }
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      mergingBranch = null;
    }
  }

  async function handleMergeAbort() {
    if (!app.repoPath) return;
    try {
      await gitMergeAbort(app.repoPath);
      mergeConflicts = [];
      dryRunResult = null;
      app.addToast('已取消 merge', 'info');
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && createExpanded && canCreate) {
      e.preventDefault();
      handleCreate();
    }
  }
</script>

{#if open}
  <div class="branch-manager" use:clickOutside={onClose} role="dialog" aria-label="Branch manager">
    {#if dryRunResult}
      <!-- Dry-run preview dialog -->
      <div class="confirm-overlay">
        <div class="confirm-dialog">
          <p class="confirm-text"><strong>合併 {dryRunResult.branch} 將產生 {dryRunResult.conflicts.length} 個衝突</strong></p>
          <div class="conflict-list">
            {#each dryRunResult.conflicts as file}
              <div class="conflict-item">{file}</div>
            {/each}
          </div>
          <p class="confirm-text" style="margin-top: var(--space-sm); font-size: 11px; color: var(--text-muted);">
            繼續合併後可以在衝突解決面板中處理衝突。
          </p>
          <div class="confirm-actions">
            <button class="btn-cancel" onclick={() => (dryRunResult = null)}>取消</button>
            <button class="btn-danger" onclick={() => executeMerge(dryRunResult!.branch)}>
              繼續合併
            </button>
          </div>
        </div>
      </div>
    {:else if mergeConflicts.length > 0}
      <!-- Merge conflict overlay (fallback for when dry-run was skipped) -->
      <div class="confirm-overlay">
        <div class="confirm-dialog">
          <p class="confirm-text"><strong>Merge 衝突</strong></p>
          <div class="conflict-list">
            {#each mergeConflicts as conflict}
              <div class="conflict-item">{conflict}</div>
            {/each}
          </div>
          <div class="confirm-actions">
            <button class="btn-danger" onclick={handleMergeAbort}>取消 Merge</button>
            <button class="btn-cancel" onclick={() => { mergeConflicts = []; onClose(); }}>關閉</button>
          </div>
        </div>
      </div>
    {:else if confirmDelete}
      <!-- Confirm delete overlay -->
      <div class="confirm-overlay">
        <div class="confirm-dialog">
          <p class="confirm-text">
            Branch <strong>{confirmDelete.name}</strong> 有
            {confirmDelete.mergeStatus?.unmerged_count ?? 0} 個未 merge 的 commit。確定刪除？
          </p>
          <div class="confirm-actions">
            <button class="btn-cancel" onclick={() => (confirmDelete = null)}>取消</button>
            <button class="btn-danger" onclick={handleForceDelete} disabled={deleting}>
              {#if deleting}<span class="spinner"></span>{/if}
              強制刪除
            </button>
          </div>
        </div>
      </div>
    {:else}
      <!-- Search -->
      <div class="search-bar">
        <input
          type="text"
          placeholder="Search branches..."
          bind:value={searchQuery}
          role="searchbox"
          aria-label="Search branches"
        />
      </div>

      <!-- Create -->
      <div class="create-section">
        {#if !createExpanded}
          <button class="create-toggle" onclick={() => (createExpanded = true)}>+ New Branch</button>
        {:else}
          <div class="create-form" onkeydown={handleKeydown}>
            <input
              type="text"
              placeholder="branch name..."
              bind:value={createName}
              class="create-input"
              autofocus
            />
            {#if createName}
              <div class="slug-preview">{slugPreview || '(invalid name)'}</div>
            {/if}
            <div class="create-actions">
              <button class="btn-create" onclick={handleCreate} disabled={!canCreate}>
                {#if creating}<span class="spinner"></span>{:else}建立{/if}
              </button>
              <button class="btn-text" onclick={() => { createExpanded = false; createName = ''; }}>取消</button>
            </div>
          </div>
        {/if}
      </div>

      <!-- Scrollable content -->
      <div class="branch-list" role="listbox">
        <!-- Local -->
        {#if localBranches().length > 0}
          <div class="section-header" role="group" aria-label="Local branches">LOCAL</div>
          {#each localBranches() as branch}
            <div
              class="branch-item"
              class:active={branch.is_current}
              role="option"
              aria-selected={branch.is_current}
            >
              {#if renamingBranch === branch.name}
                <input
                  type="text"
                  class="rename-input"
                  class:error={renameError}
                  bind:value={renameValue}
                  autofocus
                  onkeydown={(e) => {
                    if (e.key === 'Enter') handleRename();
                    if (e.key === 'Escape') { renamingBranch = null; renameError = ''; }
                  }}
                  onblur={handleRename}
                />
                {#if renameError}
                  <div class="inline-error">{renameError}</div>
                {/if}
              {:else}
                <button class="branch-name" onclick={() => !branch.is_current && handleSwitchBranch(branch.name)}>
                  {branch.name}
                  {#if branch.is_current}<span class="current-dot">●</span>{/if}
                </button>
                <div class="branch-meta">
                  {#if branch.ahead !== null || branch.behind !== null}
                    <span class="ahead-behind">
                      {#if branch.ahead}↑{branch.ahead}{/if}
                      {#if branch.behind}↓{branch.behind}{/if}
                      {#if !branch.ahead && !branch.behind}—{/if}
                    </span>
                  {/if}
                  {#if !branch.upstream && !branch.is_current}
                    <button
                      class="action-btn push-btn"
                      title="Push to origin"
                      disabled={pushingBranch === branch.name}
                      onclick={() => handlePushBranch(branch.name)}
                    >
                      {#if pushingBranch === branch.name}<span class="spinner-sm"></span>{:else}↑{/if}
                    </button>
                  {/if}
                    <div class="action-group">
                      <button
                        class="action-btn compare-btn"
                        title="與 {app.currentBranch} 比較"
                        onclick={() => app.compareBranches(app.currentBranch, branch.name)}
                      >⇄</button>
                      {#if !branch.is_current}
                        <button
                          class="action-btn merge-btn"
                        title="Merge into {app.currentBranch}"
                        disabled={mergingBranch === branch.name}
                        onclick={() => handleMerge(branch.name)}
                      >
                        {#if mergingBranch === branch.name}<span class="spinner-sm"></span>{:else}⤵{/if}
                      </button>
                      <button class="action-btn" title="重命名" onclick={() => startRename(branch.name)}>✎</button>
                      <button class="action-btn delete-btn" title="刪除" onclick={() => handleDelete(branch.name)}>×</button>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        {/if}

        <!-- Remote -->
        {#if remoteBranches().length > 0}
          <div class="section-header" role="group" aria-label="Remote branches">REMOTE</div>
          {#each remoteBranches() as branch}
            <div class="branch-item" role="option" aria-selected="false">
              <span class="branch-name remote-name">{branch.name}</span>
              <div class="action-group">
                <button
                  class="action-btn compare-btn"
                  title="與 {app.currentBranch} 比較"
                  onclick={() => app.compareBranches(app.currentBranch, branch.name)}
                >⇄</button>
                <button
                  class="action-btn checkout-btn"
                  title="Checkout as local branch"
                  onclick={() => handleCheckoutRemote(branch.name)}
                >⬇</button>
              </div>
            </div>
          {/each}
        {/if}

        <!-- Stale -->
        {#if staleBranches().length > 0}
          <button class="section-header stale-header" onclick={() => (staleExpanded = !staleExpanded)}>
            <span class="chevron" class:expanded={staleExpanded}>▸</span>
            STALE ({staleBranches().length})
          </button>
          {#if staleExpanded}
            {#each staleBranches() as branch}
              <div class="branch-item stale-item" role="option" aria-selected="false">
                <span class="branch-name stale-name">{branch.name}</span>
                <span class="stale-meta">{daysAgo(branch.last_commit_timestamp)}</span>
                <button class="action-btn delete-btn" title="刪除" onclick={() => handleDelete(branch.name)}>×</button>
              </div>
            {/each}
            <button
              class="batch-cleanup-btn"
              onclick={handleBatchCleanStale}
              disabled={batchCleaning}
            >
              {#if batchCleaning}{batchProgress}{:else}清理已 merge 的 stale branches{/if}
            </button>
          {/if}
        {/if}

        <!-- Empty state -->
        {#if localBranches().length === 0 && remoteBranches().length === 0 && staleBranches().length === 0}
          <div class="empty-state">
            {#if searchQuery}
              No branches matching "{searchQuery}"
            {:else}
              No branches found
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .branch-manager {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: var(--space-xs);
    width: 340px;
    max-height: 60vh;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 60;
    display: flex;
    flex-direction: column;
    animation: popoverIn 0.15s ease-out;
  }

  @media (max-width: 900px) {
    .branch-manager { width: calc(100vw - 24px); }
  }

  @keyframes popoverIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .search-bar {
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }
  .search-bar input {
    width: 100%;
    background: none;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: var(--font-size-md);
    font-family: var(--font-ui);
  }
  .search-bar input::placeholder { color: var(--text-muted); }

  .create-section {
    padding: var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--border);
  }
  .create-toggle {
    background: none;
    border: none;
    color: var(--accent);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    cursor: pointer;
    padding: var(--space-xs) 0;
  }
  .create-toggle:hover { text-decoration: underline; }

  .create-form { display: flex; flex-direction: column; gap: var(--space-xs); }
  .create-input {
    width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-sm);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    outline: none;
  }
  .create-input:focus { border-color: var(--accent); }
  .slug-preview {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    padding-left: 2px;
  }
  .create-actions { display: flex; gap: var(--space-xs); align-items: center; }
  .btn-create {
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
  .btn-create:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-create:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-text {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }
  .btn-text:hover { color: var(--text-primary); }

  .branch-list {
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .section-header {
    padding: var(--space-sm) var(--space-md);
    font-size: 10px;
    font-family: var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    user-select: none;
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    cursor: default;
  }

  .stale-header {
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
  .stale-header:hover { color: var(--text-secondary); }
  .chevron {
    display: inline-block;
    transition: transform 0.15s ease;
    font-size: 10px;
  }
  .chevron.expanded { transform: rotate(90deg); }

  .branch-item {
    display: flex;
    align-items: center;
    padding: 4px var(--space-md);
    gap: var(--space-xs);
    min-height: 28px;
  }
  .branch-item:hover { background: var(--bg-hover); }
  .branch-item:hover .action-group { opacity: 1; }

  .branch-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    color: var(--text-primary);
    background: none;
    border: none;
    text-align: left;
    cursor: pointer;
    padding: 0;
  }
  .branch-name:hover { color: var(--accent); }
  .branch-item.active .branch-name { color: var(--accent); cursor: default; }
  .remote-name { cursor: default; color: var(--text-secondary); }
  .stale-name { color: var(--text-muted); }
  .current-dot { color: var(--accent); font-size: 8px; margin-left: var(--space-xs); }

  .branch-meta {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    flex-shrink: 0;
  }
  .ahead-behind {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    white-space: nowrap;
  }
  .stale-meta {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .action-group {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.1s;
  }
  .action-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 12px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }
  .action-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .delete-btn:hover { color: var(--error); }
  .checkout-btn { color: var(--text-muted); opacity: 1; }
  .checkout-btn:hover { color: var(--accent); }
  .compare-btn:hover { color: var(--accent); }
  .push-btn { opacity: 1; }
  .push-btn:hover { color: var(--accent); }
  .merge-btn:hover { color: var(--success, #4ec9b0); }

  .conflict-list {
    max-height: 120px;
    overflow-y: auto;
    margin: var(--space-sm) 0;
  }
  .conflict-item {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--error);
    padding: 2px 0;
    word-break: break-all;
  }

  .rename-input {
    flex: 1;
    background: var(--bg-surface);
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    padding: 2px var(--space-xs);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    outline: none;
    min-width: 0;
  }
  .rename-input.error { border-color: var(--error); }
  .inline-error {
    font-size: var(--font-size-sm);
    color: var(--error);
    padding: 0 var(--space-xs);
  }

  .batch-cleanup-btn {
    display: block;
    width: calc(100% - var(--space-md) * 2);
    margin: var(--space-xs) var(--space-md) var(--space-sm);
    padding: var(--space-xs);
    background: none;
    border: none;
    color: var(--accent);
    font-size: var(--font-size-sm);
    font-family: var(--font-ui);
    cursor: pointer;
    text-align: center;
  }
  .batch-cleanup-btn:hover { text-decoration: underline; }
  .batch-cleanup-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .empty-state {
    padding: var(--space-lg) var(--space-md);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .confirm-overlay {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-lg);
    min-height: 200px;
  }
  .confirm-dialog {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-lg);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    max-width: 300px;
  }
  .confirm-text {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    margin: 0 0 var(--space-md);
    line-height: 1.5;
  }
  .confirm-actions { display: flex; gap: var(--space-sm); justify-content: flex-end; }
  .btn-cancel {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }
  .btn-cancel:hover { background: var(--bg-hover); }
  .btn-danger {
    background: var(--error);
    border: none;
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    color: white;
    font-size: var(--font-size-sm);
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
  .btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }

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
