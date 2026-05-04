<script lang="ts">
  import '../app.css';
  import { app } from '$lib/stores/app.svelte';
  import {
    gitDiff,
    openInEditor,
    openInFolder,
    openInTerminal,
  } from '$lib/git/commands';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import TabBar from '$lib/components/TabBar.svelte';
  import BranchSidebar from '$lib/components/BranchSidebar.svelte';
  import CommitLog from '$lib/components/CommitLog.svelte';
  import FileTree from '$lib/components/FileTree.svelte';
  import CommitFileList from '$lib/components/CommitFileList.svelte';
  import DiffViewer from '$lib/components/DiffViewer.svelte';
  import Welcome from '$lib/components/Welcome.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import RepoPopover from '$lib/components/RepoPopover.svelte';
  import MultiRepoPopover from '$lib/components/MultiRepoPopover.svelte';
  import CloneDialog from '$lib/components/CloneDialog.svelte';
  import ConflictResolver from '$lib/components/ConflictResolver.svelte';\n  import GitHubDashboard from '$lib/components/GitHubDashboard.svelte';
  import InlineTerminal from '$lib/components/InlineTerminal.svelte';
  import { multiRepo } from '$lib/stores/multiRepoStore.svelte';

  let showCloneDialog = $state(false);
  let showSettings = $state(false);
  let showCommandPalette = $state(false);
  let activePopover = $state<'repo' | 'multiRepo' | null>(null);

  // Resize state
  let sidebarWidth = $state(260);
  let rightWidth = $state(320);
  let resizing = $state<'sidebar' | 'right' | null>(null);

  function handleResizeStart(panel: 'sidebar' | 'right') {
    return (e: MouseEvent) => {
      e.preventDefault();
      resizing = panel;
      const startX = e.clientX;
      const startWidth = panel === 'sidebar' ? sidebarWidth : rightWidth;
      function onMove(ev: MouseEvent) {
        const delta = panel === 'sidebar' ? ev.clientX - startX : startX - ev.clientX;
        const newWidth = Math.max(200, Math.min(450, startWidth + delta));
        if (panel === 'sidebar') sidebarWidth = newWidth;
        else rightWidth = newWidth;
      }
      function onUp() {
        resizing = null;
        window.removeEventListener('mousemove', onMove);
        window.removeEventListener('mouseup', onUp);
      }
      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', onUp);
    };
  }

  
  function handleGlobalKeydown(e: KeyboardEvent) {
    const modKey = app.isMac ? e.metaKey : e.ctrlKey;
    const isInput = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;

    // Command palette (Primary)
    if (modKey && e.shiftKey && e.key === 'P') {
      e.preventDefault();
      showCommandPalette = !showCommandPalette;
      return;
    }

    // Toggle inline terminal
    if (modKey && e.key === '`') {
      e.preventDefault();
      app.toggleTerminal();
      return;
    }

    // Global Refresh (Cmd+R)
    if (modKey && e.key === 'r') {
      e.preventDefault();
      app.refreshAll();
      app.addToast('生產線同步中...', 'success');
      return;
    }

    // Focus Search (Cmd+F) - Focus Command Palette or Local Search if added
    if (modKey && e.key === 'f') {
      e.preventDefault();
      showCommandPalette = true;
      return;
    }

    // Git Operations
    if (modKey && !e.shiftKey && e.key === 'g') {
      e.preventDefault();
      // Logic for Pull could go here
      app.addToast('準備拉取最新代碼...', 'success');
    }

    // Tab Management
    if (e.ctrlKey && e.key === 'Tab') {
      e.preventDefault();
      const tabs = app.tabs;
      if (tabs.length <= 1) return;
      const currentIdx = tabs.findIndex((t) => t.id === app.activeTabId);
      const nextIdx = e.shiftKey
        ? (currentIdx - 1 + tabs.length) % tabs.length
        : (currentIdx + 1) % tabs.length;
      app.switchTab(tabs[nextIdx].id);
      return;
    }

    if (modKey && e.key === 'w') {
      e.preventDefault();
      if (app.activeTabId) app.closeTab(app.activeTabId);
      return;
    }

    // Repo Switching (Cmd+T / Cmd+M)
    if (modKey && e.key === 't') {
      e.preventDefault();
      activePopover = activePopover === 'repo' ? null : 'repo';
      return;
    }
    if (modKey && e.key === 'm') {
      e.preventDefault();
      activePopover = activePopover === 'multiRepo' ? null : 'multiRepo';
      return;
    }

    // Quick Panel Focus (1: Branch, 2: Log, 3: Files)
    if (modKey && !e.shiftKey) {
        if (e.key === '1') { e.preventDefault(); sidebarWidth = sidebarWidth > 0 ? sidebarWidth : 260; }
        if (e.key === '2') { e.preventDefault(); app.selectedFile = null; }
        if (e.key === '3') { e.preventDefault(); rightWidth = rightWidth > 0 ? rightWidth : 320; }
    }

    // Settings
    if (modKey && e.key === ',') {
      e.preventDefault();
      showSettings = !showSettings;
    }
  }

    if (modKey && e.key === '`') {
        e.preventDefault();
        app.toggleTerminal();
    }
  }

  // Init
  app.loadAppSettings();
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="h-screen flex flex-col bg-bg-primary text-text-normal overflow-hidden select-none">
  <!-- Top Bar: Navigation & Tabs -->
  <header class="flex-shrink-0 z-50 sticky top-0 backdrop-blur-md bg-bg-primary/60 border-b border-white/5">
    <TitleBar>
      <TabBar onOpenPopover={() => activePopover = 'repo'} />
    </TitleBar>
  </header>

  {#if showSettings}
    <div class="flex-1 overflow-hidden">
        <Settings onClose={() => (showSettings = false)} />
    </div>
  {:else if app.hasRepo}
    <!-- Secondary Tooling -->
    <Toolbar
      onOpenSettings={() => (showSettings = true)}
      onOpenPopover={() => activePopover = 'repo'}
      onOpenMultiRepo={() => activePopover = 'multiRepo'}
    />

    <!-- Main Accelerated Workspace -->
    <main class="flex-1 flex overflow-hidden">
      
      <!-- Left: Navigation (Branches, Tags, etc.) -->
      <aside 
        class="bg-bg-secondary/70 backdrop-blur-xl border-r border-white/5 flex flex-col flex-shrink-0 transition-[width] duration-75"
        style:width="{sidebarWidth}px"
      >
        <BranchSidebar />
      </aside>

      <!-- Resize Handle -->
      <div 
        class="w-1 cursor-col-resize hover:bg-monokai-blue/50 active:bg-monokai-blue transition-colors z-10"
        onmousedown={handleResizeStart('sidebar')}
      ></div>

      <!-- Center: Execution & Insight (Log, Diff) -->
      <section class="flex-1 flex flex-col bg-bg-primary min-w-0">
        {#if app.viewMode === 'conflict-resolution'}
            <ConflictResolver />
        {:else if app.selectedFile || app.currentDiff}
            <!-- Breadcrumb Navigation -->
            <div class="h-8 flex items-center px-md gap-sm bg-bg-secondary/50 border-b border-bg-tertiary text-[11px]">
                <button class="text-monokai-blue hover:underline" onclick={() => {app.selectedFile = null; app.currentDiff = null;}}>Commit Log</button>
                <span class="text-text-dimmed">/</span>
                <span class="truncate text-text-bright italic">{app.selectedFile?.split('/').pop()}</span>
            </div>
            <div class="flex-1 overflow-hidden">
                <DiffViewer />
            </div>
        {:else}
            <CommitLog />
        {/if}
      </section>

      <!-- Resize Handle -->
      <div 
        class="w-1 cursor-col-resize hover:bg-monokai-blue/50 active:bg-monokai-blue transition-colors z-10"
        onmousedown={handleResizeStart('right')}
      ></div>

      <!-- Right: Staging & Details -->
      <aside 
        class="bg-bg-secondary/70 backdrop-blur-xl border-l border-white/5 flex flex-col flex-shrink-0 transition-[width] duration-75"
        style:width="{rightWidth}px"
      >
        {#if app.viewMode === 'commit-detail'}
            <CommitFileList />
        {:else}
            <FileTree />
        {/if}
      </aside>
    </main>

    <!-- Accelerated Terminal Sidecar -->
    <InlineTerminal visible={app.showTerminal} onClose={() => (app.showTerminal = false)} />

  {:else}
    <!-- Welcome Screen (Factory Idle) -->
    <div class="flex-1 flex items-center justify-center">
        <Welcome 
            onOpenRepo={(path) => app.openRepo(path)}
            onClone={() => showCloneDialog = true}
        />
    </div>
  {/if}

  <!-- Modals & Popovers -->
  <RepoPopover open={activePopover === 'repo'} onClose={() => activePopover = null} />
  <MultiRepoPopover open={activePopover === 'multiRepo'} onClose={() => activePopover = null} />
  <CommandPalette open={showCommandPalette} onClose={() => showCommandPalette = false} onOpenSettings={() => {showCommandPalette = false; showSettings = true;}} />
  {#if showCloneDialog} <CloneDialog onClose={() => showCloneDialog = false} onCloned={(path) => {showCloneDialog = false; app.openRepo(path);}} /> {/if}
  <Toast />
</div>
