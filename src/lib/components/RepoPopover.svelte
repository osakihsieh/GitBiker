<script lang="ts">
  import { extractErrorMessage } from '$lib/utils/error';
  import { app } from '$lib/stores/app.svelte';
  import { gitStatus, gitBranches, gitLog } from '$lib/git/commands';

  interface Props {
    open: boolean;
    onClose: () => void;
    onClone: () => void;
  }

  let { open, onClose, onClone }: Props = $props();

  let searchQuery = $state('');
  let searchInput: HTMLInputElement | undefined = $state();
  let selectedIndex = $state(-1);

  // Repo status cache: { path: { branch, dirtyCount, lastActivity } }
  let repoStatusCache = $state<Record<string, {
    branch: string;
    dirtyCount: number;
    lastActivity: string;
    loading: boolean;
  }>>({});

  // Combined list: pinned first, then recent (excluding pinned)
  const allRepos = $derived(() => {
    const pinned = app.pinnedRepos;
    const recent = app.recentRepos.filter((r) => !pinned.includes(r));
    return { pinned, recent };
  });

  // Filtered repos
  const filtered = $derived(() => {
    const { pinned, recent } = allRepos();
    const q = searchQuery.toLowerCase().trim();
    if (!q) return { pinned, recent };
    const match = (path: string) => repoNameFromPath(path).toLowerCase().includes(q);
    return {
      pinned: pinned.filter(match),
      recent: recent.filter(match),
    };
  });

  const flatList = $derived(() => {
    const { pinned, recent } = filtered();
    return [...pinned, ...recent];
  });

  function repoNameFromPath(path: string): string {
    const parts = path.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1] || '';
  }

  // Highlight matching chars in repo name
  function highlightName(name: string): string {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return name;
    const idx = name.toLowerCase().indexOf(q);
    if (idx === -1) return name;
    const before = name.slice(0, idx);
    const match = name.slice(idx, idx + q.length);
    const after = name.slice(idx + q.length);
    return `${before}<span class="highlight">${match}</span>${after}`;
  }

  function timeAgo(timestamp: number): string {
    const seconds = Math.floor(Date.now() / 1000 - timestamp);
    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;
    if (seconds < 2592000) return `${Math.floor(seconds / 604800)}w ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  // Fetch lightweight status for a repo
  async function fetchRepoStatus(path: string) {
    if (repoStatusCache[path]?.loading) return;
    repoStatusCache = {
      ...repoStatusCache,
      [path]: { branch: '...', dirtyCount: 0, lastActivity: '', loading: true },
    };

    try {
      const [status, branches, commits] = await Promise.all([
        gitStatus(path),
        gitBranches(path),
        gitLog(path, 1),
      ]);

      const dirtyCount = status.filter((f) => f.staging === 'Staged' || f.staging === 'Unstaged').length;
      const branch = branches.find((b) => b.is_current)?.name || 'main';
      const lastActivity = commits.length > 0 ? timeAgo(commits[0].timestamp) : '';

      repoStatusCache = {
        ...repoStatusCache,
        [path]: { branch, dirtyCount, lastActivity, loading: false },
      };
    } catch {
      repoStatusCache = {
        ...repoStatusCache,
        [path]: { branch: '?', dirtyCount: 0, lastActivity: '', loading: false },
      };
    }
  }

  // Load status for all visible repos when popover opens
  $effect(() => {
    if (open) {
      searchQuery = '';
      selectedIndex = -1;

      // Focus search input
      setTimeout(() => searchInput?.focus(), 50);

      // Fetch status for all repos
      const allPaths = [...app.pinnedRepos, ...app.recentRepos];
      const unique = [...new Set(allPaths)];
      for (const path of unique) {
        fetchRepoStatus(path);
      }
    }
  });

  function handleRepoClick(path: string) {
    app.openRepo(path);
    onClose();
  }

  function handleMiddleClick(e: MouseEvent, path: string) {
    if (e.button === 1) {
      e.preventDefault();
      app.openRepo(path, true);
    }
  }

  async function handleOpenLocal() {
    try {
      const { open: openDialog } = await import('@tauri-apps/plugin-dialog');
      const selected = await openDialog({
        directory: true,
        title: 'Select Git Repository',
      });
      if (selected) {
        app.openRepo(selected);
        onClose();
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    const list = flatList();
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, list.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, -1);
    } else if (e.key === 'Enter' && selectedIndex >= 0 && selectedIndex < list.length) {
      e.preventDefault();
      handleRepoClick(list[selectedIndex]);
    } else if (e.key === 'Escape') {
      onClose();
    }
  }

  function handlePinClick(e: MouseEvent, path: string) {
    e.stopPropagation();
    app.togglePin(path);
  }

  // ── Drag-to-reorder pinned repos ──
  let dragIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  function handleDragStart(e: DragEvent, index: number) {
    dragIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', String(index));
    }
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    dragOverIndex = index;
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  function handleDrop(e: DragEvent, targetIndex: number) {
    e.preventDefault();
    if (dragIndex === null || dragIndex === targetIndex) {
      dragIndex = null;
      dragOverIndex = null;
      return;
    }

    const pinned = [...app.pinnedRepos];
    const [moved] = pinned.splice(dragIndex, 1);
    pinned.splice(targetIndex, 0, moved);
    app.reorderPinnedRepos(pinned);

    dragIndex = null;
    dragOverIndex = null;
  }

  function handleDragEnd() {
    dragIndex = null;
    dragOverIndex = null;
  }
</script>

{#if open}
  <div class="popover-overlay" onclick={onClose}></div>
  <div
    class="popover"
    role="dialog"
    aria-label="Repo switcher"
    onkeydown={handleKeydown}
  >
    <div class="popover-search">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        placeholder="Search repos..."
        bind:value={searchQuery}
        bind:this={searchInput}
      />
    </div>

    <div class="popover-actions">
      <button class="popover-action" onclick={() => { onClone(); onClose(); }}>
        <span>⇣</span> Clone
      </button>
      <button class="popover-action" onclick={handleOpenLocal}>
        <span>📂</span> Open Local
      </button>
    </div>

    <div class="popover-scrollable">
      {#if filtered().pinned.length > 0}
        <div class="popover-section">
          <div class="popover-section-header">📌 Pinned</div>
          {#each filtered().pinned as path, i (path)}
            {@const status = repoStatusCache[path]}
            {@const globalIdx = i}
            <button
              class="popover-repo"
              class:selected={selectedIndex === globalIdx}
              class:drag-over={dragOverIndex === i && dragIndex !== i}
              class:dragging={dragIndex === i}
              draggable="true"
              ondragstart={(e) => handleDragStart(e, i)}
              ondragover={(e) => handleDragOver(e, i)}
              ondragleave={handleDragLeave}
              ondrop={(e) => handleDrop(e, i)}
              ondragend={handleDragEnd}
              onclick={() => handleRepoClick(path)}
              onauxclick={(e) => handleMiddleClick(e, path)}
            >
              <span
                class="pin-icon pinned"
                role="button"
                aria-label="Unpin"
                onclick={(e) => handlePinClick(e, path)}
              >★</span>
              <span class="repo-name">{@html highlightName(repoNameFromPath(path))}</span>
              <span class="repo-branch">{status?.branch ?? '...'}</span>
              {#if status && !status.loading}
                <span class="repo-status" class:dirty={status.dirtyCount > 0}>
                  {status.dirtyCount > 0 ? `${status.dirtyCount}▲` : 'clean'}
                </span>
                {#if status.lastActivity}
                  <span class="repo-time">{status.lastActivity}</span>
                {/if}
              {:else}
                <span class="repo-status">...</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}

      {#if filtered().pinned.length > 0 && filtered().recent.length > 0}
        <div class="divider"></div>
      {/if}

      {#if filtered().recent.length > 0}
        <div class="popover-section">
          <div class="popover-section-header">🕐 Recent</div>
          {#each filtered().recent as path, i (path)}
            {@const status = repoStatusCache[path]}
            {@const globalIdx = filtered().pinned.length + i}
            <button
              class="popover-repo"
              class:selected={selectedIndex === globalIdx}
              onclick={() => handleRepoClick(path)}
              onauxclick={(e) => handleMiddleClick(e, path)}
            >
              <span
                class="pin-icon"
                role="button"
                aria-label="Pin"
                onclick={(e) => handlePinClick(e, path)}
              >☆</span>
              <span class="repo-name">{@html highlightName(repoNameFromPath(path))}</span>
              <span class="repo-branch">{status?.branch ?? '...'}</span>
              {#if status && !status.loading}
                <span class="repo-status" class:dirty={status.dirtyCount > 0}>
                  {status.dirtyCount > 0 ? `${status.dirtyCount}▲` : 'clean'}
                </span>
                {#if status.lastActivity}
                  <span class="repo-time">{status.lastActivity}</span>
                {/if}
              {:else}
                <span class="repo-status">...</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}

      {#if filtered().pinned.length === 0 && filtered().recent.length === 0}
        <div class="popover-empty">
          {#if searchQuery}
            No repos matching "{searchQuery}"
          {:else}
            No repos yet. Clone or open one to get started.
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .popover-overlay {
    position: fixed;
    inset: 0;
    z-index: 99;
  }
  .popover {
    position: fixed;
    top: 44px;
    left: var(--space-md);
    width: 340px;
    max-height: 60vh;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
    z-index: 100;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: popover-enter 0.15s ease-out;
  }
  @keyframes popover-enter {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
  @media (max-width: 899px) {
    .popover {
      width: calc(100vw - 24px);
      left: 12px;
    }
  }
  .popover-search {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }
  .search-icon { font-size: 14px; color: var(--text-muted); }
  .popover-search input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--font-ui);
    outline: none;
  }
  .popover-search input::placeholder { color: var(--text-muted); }
  .popover-actions {
    display: flex;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }
  .popover-action {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-sm);
    font-size: 12px;
    color: var(--accent);
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-ui);
    flex: 1;
    justify-content: center;
  }
  .popover-action:hover { background: var(--bg-hover); border-color: var(--accent); }
  .popover-scrollable {
    overflow-y: auto;
    flex: 1;
  }
  .popover-section {
    padding: var(--space-sm) var(--space-md);
  }
  .popover-section-header {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin-bottom: var(--space-xs);
  }
  .popover-repo {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 6px var(--space-sm);
    border-radius: var(--radius-sm);
    cursor: pointer;
    width: 100%;
    background: none;
    border: none;
    color: var(--text-primary);
    font-family: var(--font-ui);
    text-align: left;
  }
  .popover-repo:hover,
  .popover-repo.selected { background: var(--bg-hover); }
  .popover-repo.dragging { opacity: 0.4; }
  .popover-repo.drag-over {
    border-top: 2px solid var(--accent);
    margin-top: -2px;
  }
  .pin-icon {
    font-size: 10px;
    color: var(--text-muted);
    flex-shrink: 0;
    width: 14px;
    text-align: center;
    cursor: pointer;
    padding: 2px;
  }
  .pin-icon.pinned { color: var(--accent); }
  .pin-icon:hover { color: var(--accent); }
  .repo-name {
    font-size: 12px;
    font-family: var(--font-mono);
    font-weight: 500;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.repo-name .highlight) {
    color: var(--accent);
  }
  .repo-branch {
    font-size: 11px;
    color: var(--text-muted);
    flex-shrink: 0;
  }
  .repo-status {
    font-size: 11px;
    color: var(--text-muted);
    flex-shrink: 0;
    min-width: 36px;
    text-align: right;
  }
  .repo-status.dirty { color: var(--warning); }
  .repo-time {
    font-size: 10px;
    color: var(--text-muted);
    flex-shrink: 0;
    min-width: 50px;
    text-align: right;
  }
  .divider {
    height: 1px;
    background: var(--border);
  }
  .popover-empty {
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }
</style>
