<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitLogSearch, gitTagCreate, gitRevert, gitResetSoft, gitResetHard, gitCherryPick } from '$lib/git/commands';
  import type { Commit } from '$lib/git/types';
  import ContextMenu, { type MenuItem } from './ContextMenu.svelte';

  let searchQuery = $state('');
  let searchType = $state<'message' | 'author' | 'code' | 'branch'>('message');
  let searchResults = $state<Commit[] | null>(null);
  let searching = $state(false);
  let authorFilter = $state('');
  let contextMenu = $state<{ commit: Commit; x: number; y: number } | null>(null);
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  /** Matching branches when searchType is 'branch' */
  const matchingBranches = $derived.by(() => {
    if (searchType !== 'branch' || !searchQuery.trim()) return [];
    const q = searchQuery.toLowerCase().trim();
    return app.branches.filter((b) => b.name.toLowerCase().includes(q));
  });

  /** Unique authors from current commit list */
  const uniqueAuthors = $derived.by(() => {
    const commits = searchResults ?? app.commits;
    const authors = new Set(commits.map((c) => c.author));
    return [...authors].sort();
  });

  const displayCommits = $derived.by(() => {
    const base = searchResults ?? app.commits;
    if (!authorFilter) return base;
    return base.filter((c) => c.author === authorFilter);
  });

  // ── Commit Graph Layout ──

  const LANE_WIDTH = 14;
  const LANE_COLORS = [
    'var(--accent)',
    '#e5c07b',
    '#c678dd',
    '#98c379',
    '#e06c75',
    '#56b6c2',
    '#d19a66',
    '#61afef',
  ];

  interface GraphNode {
    lane: number;
    /** Lines to draw: [fromLane, toLane] pairs for connections to next row */
    lines: Array<{ fromLane: number; toLane: number; color: string }>;
    /** Lanes that pass through this row without stopping */
    passingLanes: Array<{ lane: number; color: string }>;
    color: string;
    maxLane: number;
  }

  const graphLayout = $derived.by((): GraphNode[] => {
    const commits = displayCommits;
    if (commits.length === 0) return [];

    // Map commit ID -> index in displayCommits
    const idToIdx = new Map<string, number>();
    for (let i = 0; i < commits.length; i++) {
      idToIdx.set(commits[i].id, i);
    }

    // activeLanes: tracks which lane each expected parent commit will appear on
    // key = commit ID, value = lane number
    const activeLanes = new Map<string, number>();
    const laneOccupied: boolean[] = [];
    const result: GraphNode[] = [];

    function allocateLane(): number {
      for (let i = 0; i < laneOccupied.length; i++) {
        if (!laneOccupied[i]) {
          laneOccupied[i] = true;
          return i;
        }
      }
      laneOccupied.push(true);
      return laneOccupied.length - 1;
    }

    function freeLane(lane: number): void {
      if (lane >= 0 && lane < laneOccupied.length) {
        laneOccupied[lane] = false;
      }
    }

    for (let i = 0; i < commits.length; i++) {
      const commit = commits[i];
      let myLane: number;

      // Check if this commit was expected on a lane
      if (activeLanes.has(commit.id)) {
        myLane = activeLanes.get(commit.id)!;
        activeLanes.delete(commit.id);
      } else {
        myLane = allocateLane();
      }

      const color = LANE_COLORS[myLane % LANE_COLORS.length];
      const lines: GraphNode['lines'] = [];
      const passingLanes: GraphNode['passingLanes'] = [];

      // Process parents
      const parents = commit.parents;
      for (let p = 0; p < parents.length; p++) {
        const parentId = parents[p];
        const parentIdx = idToIdx.get(parentId);

        // Only draw if parent is in our visible list
        if (parentIdx === undefined) continue;

        if (p === 0) {
          // First parent — continue in same lane
          if (!activeLanes.has(parentId)) {
            activeLanes.set(parentId, myLane);
            lines.push({ fromLane: myLane, toLane: myLane, color });
          } else {
            // Parent already expected on another lane — draw merge line
            const targetLane = activeLanes.get(parentId)!;
            lines.push({ fromLane: myLane, toLane: targetLane, color });
            freeLane(myLane);
          }
        } else {
          // Additional parents (merge) — allocate or find lane
          if (!activeLanes.has(parentId)) {
            const mergeLane = allocateLane();
            activeLanes.set(parentId, mergeLane);
            const mergeColor = LANE_COLORS[mergeLane % LANE_COLORS.length];
            lines.push({ fromLane: myLane, toLane: mergeLane, color: mergeColor });
          } else {
            const targetLane = activeLanes.get(parentId)!;
            const mergeColor = LANE_COLORS[targetLane % LANE_COLORS.length];
            lines.push({ fromLane: myLane, toLane: targetLane, color: mergeColor });
          }
        }
      }

      // If no parents link back to this lane, free it
      if (parents.length === 0) {
        freeLane(myLane);
      }

      // Collect passing-through lanes (lanes occupied by other commits' ancestry)
      for (const [, lane] of activeLanes) {
        if (lane !== myLane) {
          passingLanes.push({ lane, color: LANE_COLORS[lane % LANE_COLORS.length] });
        }
      }

      const maxLane = Math.max(myLane, ...passingLanes.map((p) => p.lane), ...lines.map((l) => Math.max(l.fromLane, l.toLane)));

      result.push({ lane: myLane, lines, passingLanes, color, maxLane });
    }

    return result;
  });

  const graphWidth = $derived(
    Math.max(20, (Math.max(0, ...graphLayout.map((g) => g.maxLane)) + 1) * LANE_WIDTH + 8)
  );

  function laneX(lane: number): number {
    return lane * LANE_WIDTH + LANE_WIDTH / 2 + 4;
  }

  // ── Utilities ──

  function timeAgo(timestamp: number): string {
    const seconds = Math.floor(Date.now() / 1000 - timestamp);
    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  function shortHash(id: string): string {
    return id.substring(0, 7);
  }

  function firstLine(message: string): string {
    return message.split('\n')[0] || '';
  }

  function handleSearchInput() {
    clearTimeout(debounceTimer);
    if (!searchQuery.trim()) {
      searchResults = null;
      return;
    }
    debounceTimer = setTimeout(executeSearch, 300);
  }

  async function executeSearch() {
    if (!app.repoPath || !searchQuery.trim()) return;
    if (searchType === 'branch') {
      // Branch search is client-side — no need to call git
      searchResults = null;
      return;
    }
    searching = true;
    try {
      searchResults = await gitLogSearch(app.repoPath, searchQuery.trim(), searchType, 200);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
      searchResults = null;
    } finally {
      searching = false;
    }
  }

  function selectBranch(branchName: string) {
    app.setLogFilter({ type: 'Branch', value: branchName });
    searchQuery = '';
    searchResults = null;
  }

  function clearSearch() {
    searchQuery = '';
    searchResults = null;
  }

  function handleCommitClick(commit: Commit) {
    if (app.selectedCommit?.id === commit.id) {
      app.backToWorktree();
    } else {
      app.selectCommit(commit);
    }
  }

  function handleContextMenu(e: MouseEvent, commit: Commit) {
    e.preventDefault();
    contextMenu = { commit, x: e.clientX, y: e.clientY };
  }

  function getContextMenuItems(commit: Commit): MenuItem[] {
    const items: MenuItem[] = [
      { id: 'copyHash', label: '複製 Hash' },
      { id: '_sep1', label: '', separator: true },
      { id: 'createTag', label: '建立 Tag...' },
      { id: '_sep2', label: '', separator: true },
      { id: 'revert', label: '撤回此 Commit (Revert)' },
      { id: 'cherryPick', label: 'Cherry-pick 此 Commit' },
    ];

    const currentBranch = app.branches.find((b) => b.is_current);
    const aheadCount = currentBranch?.ahead ?? 0;
    const commitIndex = displayCommits.findIndex((c) => c.id === commit.id);
    const isUnpushed = commitIndex >= 0 && commitIndex < aheadCount;

    if (isUnpushed) {
      items.push(
        { id: 'undoSoft', label: '撤銷到此 Commit (保留變更)' },
        { id: 'undoHard', label: '撤銷到此 Commit (丟棄變更)' },
      );
    }

    return items;
  }

  const contextMenuItems = $derived(
    contextMenu ? getContextMenuItems(contextMenu.commit) : [],
  );

  async function handleContextSelect(actionId: string) {
    if (!contextMenu || !app.repoPath) return;
    const { commit } = contextMenu;
    try {
      switch (actionId) {
        case 'copyHash':
          await navigator.clipboard.writeText(commit.id);
          app.addToast('已複製 commit hash', 'success');
          break;
        case 'createTag': {
          const tagName = prompt('Tag 名稱:');
          if (tagName?.trim()) {
            await gitTagCreate(app.repoPath, tagName.trim(), commit.id);
            app.addToast(`已建立 tag: ${tagName.trim()}`, 'success');
            await app.refreshAll();
          }
          break;
        }
        case 'revert': {
          const isMerge = commit.parents.length > 1;
          const label = isMerge ? '（Merge commit，將使用 -m 1）' : '';
          if (confirm(`確定要 Revert commit ${commit.id.substring(0, 7)}？${label}\n\n${commit.message}`)) {
            await gitRevert(app.repoPath, commit.id, isMerge);
            app.addToast('Revert 成功', 'success');
            await app.refreshAll();
          }
          break;
        }
        case 'undoSoft': {
          if (confirm(`確定要撤銷到 ${commit.id.substring(0, 7)}？\n變更將保留在 staged 區域。`)) {
            await gitResetSoft(app.repoPath, commit.id);
            app.addToast('已撤銷 commit（變更已保留）', 'success');
            await app.refreshAll();
          }
          break;
        }
        case 'undoHard': {
          if (confirm(`⚠️ 確定要撤銷到 ${commit.id.substring(0, 7)}？\n\n此操作不可撤銷，所有變更將被丟棄！`)) {
            await gitResetHard(app.repoPath, commit.id);
            app.addToast('已撤銷 commit（變更已丟棄）', 'success');
            await app.refreshAll();
          }
          break;
        }
        case 'cherryPick': {
          if (confirm(`確定要 Cherry-pick commit ${commit.id.substring(0, 7)}？\n\n${commit.message}`)) {
            const result = await gitCherryPick(app.repoPath, commit.id);
            if (result.success) {
              app.addToast('Cherry-pick 成功', 'success');
            } else if (result.conflicts.length > 0) {
              app.addToast(`Cherry-pick 衝突：需要解決衝突`, 'error', false);
              await app.enterConflictMode();
            }
            await app.refreshAll();
          }
          break;
        }
      }
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      clearSearch();
    }
  }

  function handleFilterChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    const val = target.value;
    if (val === 'head') {
      app.setLogFilter({ type: 'Head' });
    } else if (val === 'all') {
      app.setLogFilter({ type: 'All' });
    } else {
      app.setLogFilter({ type: 'Branch', value: val });
    }
  }

  const currentFilterValue = $derived.by(() => {
    const f = app.logFilter;
    if (f.type === 'Head') return 'head';
    if (f.type === 'All') return 'all';
    return f.value;
  });
</script>

<div class="history-panel">
  <!-- Search Bar -->
  <div class="search-bar">
    <select class="search-type-select" bind:value={searchType} onchange={executeSearch}>
      <option value="message">Msg</option>
      <option value="author">User</option>
      <option value="code">Code</option>
      <option value="branch">Branch</option>
    </select>
    <input
      type="text"
      class="search-input"
      placeholder="Search..."
      bind:value={searchQuery}
      oninput={handleSearchInput}
      onkeydown={handleSearchKeydown}
      aria-label="Search commits"
    />
    {#if searchQuery}
      <button class="search-clear" onclick={clearSearch} aria-label="Clear search">✕</button>
    {/if}
    {#if searching}
      <span class="search-spinner"></span>
    {/if}
  </div>

  {#if searchType === 'branch' && searchQuery.trim() && matchingBranches.length > 0}
    <div class="branch-results">
      {#each matchingBranches as branch}
        <button class="branch-result-item" onclick={() => selectBranch(branch.name)}>
          <span class="branch-result-icon">{branch.is_remote ? '☁' : '⎇'}</span>
          <span class="branch-result-name">{branch.name}</span>
          {#if branch.is_current}
            <span class="branch-current-badge">current</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}

  <div class="history-header">
    <span class="header-title">Commits</span>
    <select class="filter-select" value={currentFilterValue} onchange={handleFilterChange}>
      <option value="head">Current Branch</option>
      <option value="all">All Branches</option>
      <optgroup label="Local Branches">
        {#each app.branches.filter(b => !b.is_remote) as b}
          <option value={b.name}>{b.name}</option>
        {/each}
      </optgroup>
      <optgroup label="Remote Branches">
        {#each app.branches.filter(b => b.is_remote) as b}
          <option value={b.name}>{b.name}</option>
        {/each}
      </optgroup>
    </select>
    {#if uniqueAuthors.length > 1}
      <select
        class="filter-select author-filter"
        value={authorFilter}
        onchange={(e) => authorFilter = e.currentTarget.value}
      >
        <option value="">All Authors</option>
        {#each uniqueAuthors as author}
          <option value={author}>{author}</option>
        {/each}
      </select>
    {/if}
  </div>

  <!-- Commit List -->
  <div class="history-list">
    {#if searchResults !== null && displayCommits.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🔍</div>
        <div>No commits matching "{searchQuery}"</div>
      </div>
    {:else}
      <!-- WIP row -->
      {@const wipCount = app.stagedFiles.length + app.unstagedFiles.length}
      {#if wipCount > 0 && !searchResults}
        <button
          class="commit-item wip-item"
          class:selected={!app.selectedCommit && app.viewMode === 'worktree'}
          onclick={() => app.backToWorktree()}
        >
          <div class="commit-graph" style:width="{graphWidth}px">
            <svg class="graph-svg" width={graphWidth} height="100%">
              {#if graphLayout[0]}
                <line
                  x1={laneX(graphLayout[0].lane)} y1="50%"
                  x2={laneX(graphLayout[0].lane)} y2="100%"
                  stroke={graphLayout[0].color} stroke-width="2" opacity="0.4"
                />
                <circle
                  cx={laneX(graphLayout[0].lane)} cy="50%"
                  r="4" fill="none" stroke={graphLayout[0].color} stroke-width="2"
                />
              {/if}
            </svg>
          </div>
          <div class="commit-info">
            <div class="wip-badge">// WIP</div>
            <div class="commit-msg">{wipCount} file{wipCount !== 1 ? 's' : ''} changed</div>
          </div>
        </button>
      {/if}
      {#each displayCommits as commit, i (commit.id)}
        {@const graph = graphLayout[i]}
        <button
          class="commit-item"
          class:selected={app.selectedCommit?.id === commit.id}
          onclick={() => handleCommitClick(commit)}
          oncontextmenu={(e) => handleContextMenu(e, commit)}
        >
          <div class="commit-graph" style:width="{graphWidth}px">
            <svg class="graph-svg" width={graphWidth} height="100%">
              <!-- Passing-through lanes (vertical lines) -->
              {#if graph}
                {#each graph.passingLanes as pl}
                  <line
                    x1={laneX(pl.lane)} y1="0"
                    x2={laneX(pl.lane)} y2="100%"
                    stroke={pl.color} stroke-width="2" opacity="0.4"
                  />
                {/each}
                <!-- Connection lines to next row -->
                {#each graph.lines as line}
                  {#if line.fromLane === line.toLane}
                    <!-- Straight line down -->
                    <line
                      x1={laneX(line.fromLane)} y1="50%"
                      x2={laneX(line.toLane)} y2="100%"
                      stroke={line.color} stroke-width="2" opacity="0.4"
                    />
                  {:else}
                    <!-- Curved merge/branch line -->
                    <path
                      d="M {laneX(line.fromLane)} 50% Q {laneX(line.fromLane)} 100%, {laneX(line.toLane)} 100%"
                      fill="none" stroke={line.color} stroke-width="2" opacity="0.4"
                    />
                  {/if}
                {/each}
                <!-- Vertical line from top to dot (if this commit continues a lane) -->
                <line
                  x1={laneX(graph.lane)} y1="0"
                  x2={laneX(graph.lane)} y2="50%"
                  stroke={graph.color} stroke-width="2" opacity="0.4"
                />
                <!-- Commit dot -->
                <circle
                  cx={laneX(graph.lane)} cy="50%"
                  r="4" fill={graph.color}
                />
              {/if}
            </svg>
          </div>
          <div class="commit-info">
            {#if commit.refs && commit.refs.length > 0}
              <div class="commit-tags">
                {#each commit.refs.slice(0, 3) as ref}
                  <span
                    class="ref-tag"
                    class:ref-local={ref.kind === 'Local'}
                    class:ref-remote={ref.kind === 'Remote'}
                    class:ref-tag-badge={ref.kind === 'Tag'}
                  >{ref.name}</span>
                {/each}
                {#if commit.refs.length > 3}
                  <span class="ref-overflow" title={commit.refs.slice(3).map(r => r.name).join(', ')}>+{commit.refs.length - 3}</span>
                {/if}
              </div>
            {/if}
            <div class="commit-msg" title={commit.message}>{firstLine(commit.message)}</div>
            <div class="commit-meta">
              <span class="commit-hash">{shortHash(commit.id)}</span>
              <span>{commit.author}</span>
              <span>{timeAgo(commit.timestamp)}</span>
            </div>
          </div>
        </button>
      {:else}
        <div class="empty-state">
          <div class="empty-icon">◯</div>
          <div>No commits yet</div>
        </div>
      {/each}
    {/if}
  </div>
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={contextMenuItems}
    onSelect={handleContextSelect}
    onClose={() => contextMenu = null}
  />
{/if}

<style>
  .history-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Search */
  .search-bar {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .search-type-select {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-size: 10px;
    padding: 1px 2px;
    outline: none;
    cursor: pointer;
  }
  .search-type-select:hover { color: var(--text-primary); border-color: var(--accent); }
  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-ui);
    outline: none;
    min-width: 0;
  }
  .search-input::placeholder { color: var(--text-muted); }
  .search-clear {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 10px;
    padding: 2px 4px;
  }
  .search-clear:hover { color: var(--text-primary); }
  .search-spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid var(--text-muted);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  /* Header */
  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    gap: var(--space-sm);
  }
  .header-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    white-space: nowrap;
  }
  .filter-select {
    flex: 1;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 11px;
    padding: 2px 4px;
    outline: none;
    max-width: 150px;
  }
  .filter-select:hover { border-color: var(--accent); }
  .author-filter { max-width: 120px; }

  /* List */
  .history-list {
    overflow-y: auto;
    flex: 1;
  }
  .commit-item {
    display: flex;
    gap: 0;
    padding: var(--space-sm) var(--space-sm) var(--space-sm) 0;
    border-bottom: 1px solid var(--border);
    background: none;
    border-left: 2px solid transparent;
    border-right: none;
    border-top: none;
    width: 100%;
    text-align: left;
    cursor: pointer;
    color: var(--text-primary);
    font-family: var(--font-ui);
    min-height: 48px;
  }
  .commit-item:hover { background: var(--bg-hover); }
  .commit-item.selected {
    background: var(--bg-surface);
    border-left-color: var(--accent);
  }

  /* Graph column */
  .commit-graph {
    position: relative;
    flex-shrink: 0;
    min-width: 20px;
  }
  .graph-svg {
    display: block;
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
  }

  .commit-info { flex: 1; min-width: 0; padding-left: var(--space-xs); }

  /* Ref Tags */
  .commit-tags {
    display: flex;
    gap: var(--space-xs);
    margin-bottom: 2px;
    flex-wrap: wrap;
  }
  .ref-tag {
    font-size: 10px;
    padding: 1px 4px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    font-family: var(--font-mono);
    white-space: nowrap;
  }
  .ref-local {
    background: var(--accent);
    color: var(--bg-primary);
  }
  .ref-remote {
    background: var(--bg-surface);
    border: 1px solid var(--accent);
    color: var(--accent);
  }
  .ref-tag-badge {
    background: rgba(81, 207, 102, 0.2);
    color: var(--success);
  }
  .ref-overflow {
    font-size: 10px;
    color: var(--text-muted);
    padding: 1px 4px;
  }

  .commit-msg {
    font-size: var(--font-size-sm);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.4;
  }
  .commit-meta {
    font-size: 11px;
    color: var(--text-muted);
    display: flex;
    gap: var(--space-sm);
    margin-top: 2px;
  }
  .commit-hash { font-family: var(--font-mono); font-size: 10px; }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: var(--space-lg);
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    height: 100%;
  }
  .empty-icon { font-size: 24px; opacity: 0.3; }

  /* WIP row */
  .wip-item {
    background: var(--bg-surface);
    border-bottom: 2px solid var(--accent);
  }
  .wip-item:hover { background: var(--bg-hover); }
  .wip-badge {
    display: inline-block;
    font-size: 10px;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--accent);
    background: rgba(97, 175, 239, 0.15);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    margin-bottom: 2px;
  }

  /* Branch search results */
  .branch-results {
    border-bottom: 1px solid var(--border);
    max-height: 150px;
    overflow-y: auto;
  }
  .branch-result-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    width: 100%;
    padding: var(--space-xs) var(--space-md);
    background: none;
    border: none;
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    cursor: pointer;
    text-align: left;
  }
  .branch-result-item:hover { background: var(--bg-hover); }
  .branch-result-icon { font-size: 12px; color: var(--accent); flex-shrink: 0; }
  .branch-result-name {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .branch-current-badge {
    font-size: 9px;
    padding: 1px 4px;
    background: var(--accent);
    color: var(--bg-primary);
    border-radius: var(--radius-sm);
    font-weight: 600;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>
