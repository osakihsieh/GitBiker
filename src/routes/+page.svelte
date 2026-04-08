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
  let showTerminal = $state(false);

  // Popover 互斥管理
  type ActivePopover = 'repo' | 'multiRepo' | null;
  let activePopover = $state<ActivePopover>(null);
  let autoOpenMultiRepo = $state(false);

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
    // Ctrl+`: toggle inline terminal
    if (e.ctrlKey && e.key === '`') {
      e.preventDefault();
      showTerminal = !showTerminal;
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
  <TitleBar />
  {#if showSettings}
    <Settings onClose={() => showSettings = false} />
  {:else if app.hasRepo}
    <Toolbar
      onOpenSettings={() => showSettings = true}
      onOpenPopover={togglePopover}
      onOpenMultiRepo={toggleMultiRepo}
    />
    <TabBar onOpenPopover={togglePopover} />
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
        <div class="sidebar">
          {#if app.viewMode === 'commit-detail'}
            <CommitFileList />
          {:else}
            <FileTree />
          {/if}
        </div>
        <div class="resize-handle"></div>
        <div class="center" tabindex="-1">
          <DiffViewer />
        </div>
        <div class="resize-handle"></div>
        <div class="right" tabindex="-1">
          {#if app.viewMode === 'file-history'}
            <FileHistory />
          {:else}
            <CommitLog />
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
    width: 240px;
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
    width: 320px;
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
  .resize-handle {
    width: 3px;
    cursor: col-resize;
    background: transparent;
    flex-shrink: 0;
  }
  .resize-handle:hover { background: var(--accent); }
</style>
