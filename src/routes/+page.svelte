<script lang="ts">
  import '../app.css';
  import { app } from '$lib/stores/app.svelte';
  import { gitDiff, openInEditor, openInFolder, openInTerminal } from '$lib/git/commands';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import TabBar from '$lib/components/TabBar.svelte';
  import FileTree from '$lib/components/FileTree.svelte';
  import DiffViewer from '$lib/components/DiffViewer.svelte';
  import CommitLog from '$lib/components/CommitLog.svelte';
  import CommitFileList from '$lib/components/CommitFileList.svelte';
  import BranchSidebar from '$lib/components/BranchSidebar.svelte';
  import Welcome from '$lib/components/Welcome.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import CloneDialog from '$lib/components/CloneDialog.svelte';
  import RepoPopover from '$lib/components/RepoPopover.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import InlineTerminal from '$lib/components/InlineTerminal.svelte';
  import ConflictResolver from '$lib/components/ConflictResolver.svelte';
  import FileHistory from '$lib/components/FileHistory.svelte';
  import BranchCompare from '$lib/components/BranchCompare.svelte';
  import MultiRepoPopover from '$lib/components/MultiRepoPopover.svelte';
  import { multiRepo } from '$lib/stores/multiRepoStore.svelte';

  let showCloneDialog = $state(false);
  let showSettings = $state(false);
  let showCommandPalette = $state(false);
  // Popover 互斥管理
  type ActivePopover = 'repo' | 'multiRepo' | null;
  let activePopover = $state<ActivePopover>(null);
  let autoOpenMultiRepo = $state(false);

  // ── Resize handles ──
  let sidebarWidth = $state(240);
  let rightWidth = $state(320);
  let resizing = $state<'sidebar' | 'right' | null>(null);

  function handleResizeStart(panel: 'sidebar' | 'right') {
    return (e: MouseEvent) => {
      e.preventDefault();
      resizing = panel;
      const startX = e.clientX;
      const startWidth = panel === 'sidebar' ? sidebarWidth : rightWidth;

      function onMove(ev: MouseEvent) {
        const delta = panel === 'sidebar'
          ? ev.clientX - startX
          : startX - ev.clientX;
        const newWidth = Math.max(180, Math.min(400, startWidth + delta));
        if (panel === 'sidebar') {
          sidebarWidth = newWidth;
        } else {
          rightWidth = newWidth;
        }
      }

      function onUp() {
        resizing = null;
        window.removeEventListener('mousemove', onMove);
        window.removeEventListener('mouseup', onUp);
        // Persist widths
        try {
          localStorage.setItem('gitbiker-sidebar-width', String(sidebarWidth));
          localStorage.setItem('gitbiker-right-width', String(rightWidth));
        } catch {}
      }

      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', onUp);
    };
  }

  // Restore persisted widths
  if (typeof localStorage !== 'undefined') {
    const sw = localStorage.getItem('gitbiker-sidebar-width');
    const rw = localStorage.getItem('gitbiker-right-width');
    if (sw) sidebarWidth = Math.max(180, Math.min(400, Number(sw)));
    if (rw) rightWidth = Math.max(180, Math.min(400, Number(rw)));
  }

  // 啟動時從 Tauri Store 載入應用設定 + multi-repo store
  app.loadAppSettings();
  multiRepo.init(app.repoPath);

  // Load diff when selected file changes
  $effect(() => {
    const file = app.selectedFile;
    const repoPath = app.repoPath;
    if (file && repoPath) {
      app.loadDiff(file);
    }
  });

  function handleClone() {
    showCloneDialog = true;
  }

  function handleCloned(path: string) {
    showCloneDialog = false;
    app.openRepo(path);
  }

  function togglePopover() {
    activePopover = activePopover === 'repo' ? null : 'repo';
  }

  function toggleMultiRepo() {
    activePopover = activePopover === 'multiRepo' ? null : 'multiRepo';
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    // Esc: clear file selection, return to CommitLog view
    // Guard: only handle if no modal/popover/palette is open (they handle their own Esc)
    if (e.key === 'Escape' && !showCommandPalette && !showSettings && !showCloneDialog && activePopover === null) {
      if (app.stashDiff !== null) {
        e.preventDefault();
        app.stashDiff = null;
        return;
      }
      if (app.selectedFile || app.currentDiff) {
        e.preventDefault();
        app.selectedFile = null;
        app.currentDiff = null;
        return;
      }
      if (app.viewMode === 'commit-detail') {
        e.preventDefault();
        app.backToWorktree();
        return;
      }
    }

    // Ctrl+`: toggle inline terminal
    if (e.ctrlKey && e.key === '`') {
      e.preventDefault();
      app.toggleTerminal();
      return;
    }

    // Ctrl+Shift+P: command palette
    if (e.ctrlKey && e.shiftKey && e.key === 'P') {
      e.preventDefault();
      showCommandPalette = !showCommandPalette;
      return;
    }

    // Ctrl+Shift+M: toggle conflict resolver
    if (e.ctrlKey && e.shiftKey && e.key === 'M') {
      e.preventDefault();
      if (app.isInConflictMode) {
        app.exitConflictMode();
      } else if (app.repoPath) {
        app.enterConflictMode();
      }
      return;
    }

    // Ctrl+Tab / Ctrl+Shift+Tab: switch tabs
    if (e.ctrlKey && e.key === 'Tab') {
      e.preventDefault();
      const tabs = app.tabs;
      if (tabs.length <= 1) return;
      const currentIdx = tabs.findIndex((t) => t.id === app.activeTabId);
      if (currentIdx === -1) return;
      const nextIdx = e.shiftKey
        ? (currentIdx - 1 + tabs.length) % tabs.length
        : (currentIdx + 1) % tabs.length;
      app.switchTab(tabs[nextIdx].id);
      return;
    }

    // Ctrl+W: close current tab
    if (e.ctrlKey && e.key === 'w') {
      e.preventDefault();
      if (app.activeTabId) app.closeTab(app.activeTabId);
      return;
    }

    // Ctrl+T: open repo popover
    if (e.ctrlKey && !e.shiftKey && e.key === 't') {
      e.preventDefault();
      activePopover = 'repo';
      return;
    }

    // Ctrl+M: toggle multi-repo popover
    if (e.ctrlKey && !e.shiftKey && e.key === 'm') {
      e.preventDefault();
      toggleMultiRepo();
      return;
    }

    // Alt+E: open in editor (GitKraken style)
    if (e.altKey && e.key === 'e') {
      e.preventDefault();
      if (app.repoPath) {
        openInEditor(app.repoPath, app.preferredEditor ?? undefined)
          .catch((err: unknown) => app.addToast(String(err), 'error'));
      }
      return;
    }

    // Alt+O: open in folder (GitKraken style)
    if (e.altKey && e.key === 'o') {
      e.preventDefault();
      if (app.repoPath) {
        openInFolder(app.repoPath)
          .catch((err: unknown) => app.addToast(String(err), 'error'));
      }
      return;
    }

    // Alt+T: open in terminal (GitKraken style)
    if (e.altKey && e.key === 't') {
      e.preventDefault();
      if (app.repoPath) {
        openInTerminal(app.repoPath)
          .catch((err: unknown) => app.addToast(String(err), 'error'));
      }
      return;
    }

    // Ctrl+1/2/3: focus panels
    if (e.ctrlKey && !e.shiftKey) {
      if (e.key === '1') {
        e.preventDefault();
        const el = document.querySelector('.sidebar') as HTMLElement | null;
        el?.querySelector('button, input, textarea')?.dispatchEvent(new Event('focus'));
      } else if (e.key === '2') {
        e.preventDefault();
        const el = document.querySelector('.center') as HTMLElement | null;
        el?.focus();
      } else if (e.key === '3') {
        e.preventDefault();
        const el = document.querySelector('.right') as HTMLElement | null;
        el?.focus();
      }
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="app-root">
  <TitleBar>
    <TabBar onOpenPopover={togglePopover} />
  </TitleBar>
  {#if showSettings}
    <Settings onClose={() => showSettings = false} />
  {:else if app.hasRepo}
    <Toolbar
      onOpenSettings={() => showSettings = true}
      onOpenPopover={togglePopover}
      onOpenMultiRepo={toggleMultiRepo}
    />
    <div class="main">
      {#if app.viewMode === 'conflict-resolution'}
        <ConflictResolver />
        <div class="resize-handle"></div>
        <div class="right" tabindex="-1">
          <CommitLog />
        </div>
      {:else if app.viewMode === 'branch-compare'}
        <BranchCompare />
      {:else}
        <div class="sidebar" style:width="{sidebarWidth}px">
          <BranchSidebar />
        </div>
        <div
          class="resize-handle"
          class:active={resizing === 'sidebar'}
          onmousedown={handleResizeStart('sidebar')}
          role="separator"
          aria-orientation="vertical"
        ></div>
        <div class="center" tabindex="-1">
          {#if app.stashDiff !== null}
            <div class="breadcrumb-bar">
              <span class="breadcrumb-current">Stash Diff</span>
              <button class="breadcrumb-item" style="margin-left:auto" onclick={() => app.stashDiff = null}>✕ 關閉</button>
            </div>
            <pre class="stash-diff-viewer">{app.stashDiff}</pre>
          {:else if app.selectedFile || app.currentDiff}
            <div class="breadcrumb-bar">
              <button class="breadcrumb-item" onclick={() => { app.selectedFile = null; app.currentDiff = null; if (app.viewMode === 'commit-detail') app.backToWorktree(); }}>
                CommitLog
              </button>
              {#if app.viewMode === 'commit-detail' && app.selectedCommit}
                <span class="breadcrumb-sep">&gt;</span>
                <button class="breadcrumb-item" onclick={() => { app.selectedFile = null; app.currentDiff = null; }}>
                  {app.selectedCommit.id.substring(0, 7)} "{app.selectedCommit.message.split('\n')[0].substring(0, 30)}{app.selectedCommit.message.split('\n')[0].length > 30 ? '...' : ''}"
                </button>
              {/if}
              {#if app.selectedFile}
                <span class="breadcrumb-sep">&gt;</span>
                <span class="breadcrumb-current">{app.selectedFile.replace(/\\/g, '/').split('/').pop()}</span>
              {/if}
            </div>
            <DiffViewer />
          {:else if app.viewMode === 'file-history'}
            <FileHistory />
          {:else}
            <CommitLog />
          {/if}
        </div>
        <div
          class="resize-handle"
          class:active={resizing === 'right'}
          onmousedown={handleResizeStart('right')}
          role="separator"
          aria-orientation="vertical"
        ></div>
        <div class="right" style:width="{rightWidth}px" tabindex="-1">
          {#if app.viewMode === 'commit-detail'}
            <CommitFileList />
          {:else}
            <FileTree />
          {/if}
        </div>
      {/if}
    </div>
    <InlineTerminal visible={app.showTerminal} onClose={() => (app.showTerminal = false)} />
  {:else}
    <div class="welcome-toolbar">
      <div class="spacer"></div>
      <button class="settings-btn" onclick={() => showSettings = true}>⚙</button>
    </div>
    <Welcome
      onOpenRepo={(path) => app.openRepo(path)}
      onClone={handleClone}
      onOpenMultiRepo={async (scanPath) => {
        await multiRepo.addScanPath(scanPath, null);
        const repos = multiRepo.repos;
        if (repos.length > 0) {
          await app.openRepo(repos[0].path);
          // Auto-open multi-repo popover after workspace loads
          setTimeout(() => { activePopover = 'multiRepo'; }, 300);
        }
      }}
    />
  {/if}

  <RepoPopover
    open={activePopover === 'repo'}
    onClose={() => activePopover = null}
    onClone={handleClone}
  />

  <MultiRepoPopover
    open={activePopover === 'multiRepo'}
    onClose={() => activePopover = null}
  />

  {#if showCloneDialog}
    <CloneDialog
      onClose={() => showCloneDialog = false}
      onCloned={handleCloned}
    />
  {/if}

  <CommandPalette
    open={showCommandPalette}
    onClose={() => (showCommandPalette = false)}
    onOpenSettings={() => { showCommandPalette = false; showSettings = true; }}
  />

  <Toast />
</div>

<style>
  .app-root {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }
  .welcome-toolbar {
    display: flex;
    align-items: center;
    padding: var(--space-xs) var(--space-md);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    height: 40px;
    flex-shrink: 0;
  }
  .spacer {
    flex: 1;
  }
  .settings-btn {
    margin-left: auto;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 16px;
    padding: var(--space-xs);
  }
  .settings-btn:hover { color: var(--text-primary); }
  .main {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .sidebar {
    min-width: 180px;
    max-width: 400px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 0;
  }
  .center {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 300px;
    outline: none;
  }
  .right {
    min-width: 180px;
    max-width: 400px;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 0;
    outline: none;
  }
  .breadcrumb-bar {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-md);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    font-family: var(--font-ui);
    flex-shrink: 0;
    min-height: 28px;
  }
  .breadcrumb-item {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: 12px;
    font-family: var(--font-ui);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
  }
  .breadcrumb-item:hover { background: var(--bg-hover); text-decoration: underline; }
  .breadcrumb-sep { color: var(--text-muted); font-size: 11px; }
  .breadcrumb-current {
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .stash-diff-viewer {
    flex: 1;
    overflow: auto;
    margin: 0;
    padding: var(--space-md);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-primary);
    background: var(--bg-primary);
    white-space: pre;
    tab-size: 4;
  }
  .resize-handle {
    width: 3px;
    cursor: col-resize;
    background: transparent;
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .resize-handle:hover, .resize-handle.active { background: var(--accent); }
</style>
