<script lang="ts">
  import '../app.css';
  import { app } from '$lib/stores/app.svelte';
  import {
    gitDiff,
    openInEditor,
    openInFolder,
    openInTerminal,
  } from '$lib/git/commands';
  import { 
    LayoutGrid, 
    Folders, 
    Terminal, 
    Settings, 
    Plus, 
    FolderOpen, 
    ExternalLink, 
    ChevronDown,
    ArrowDown,
    ArrowUp,
    Tag,
    RefreshCw,
    Sparkles,
    Radio
  } from 'lucide-react';
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
  import SettingsComponent from '$lib/components/Settings.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import RepoPopover from '$lib/components/RepoPopover.svelte';
  import MultiRepoPopover from '$lib/components/MultiRepoPopover.svelte';
  import CloneDialog from '$lib/components/CloneDialog.svelte';
  import ConflictResolver from '$lib/components/ConflictResolver.svelte';
  import InlineTerminal from '$lib/components/InlineTerminal.svelte';
  import AgentDashboard from '$lib/components/AgentDashboard.svelte';
  import GitHubDashboard from '$lib/components/GitHubDashboard.svelte';
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

  // Init
  app.loadAppSettings();
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="h-screen flex flex-col bg-bg text-ink overflow-hidden select-none font-sans antialiased">
  <!-- TitleBar & Tabs -->
  <header class="flex-shrink-0 z-50 border-b border-ink-10 bg-bg">
    <TitleBar>
      <div class="flex items-center gap-1">
        <div class="w-6 h-6 rounded bg-ink flex items-center justify-center text-bg font-bold text-xs">G</div>
        <span class="text-[13px] font-semibold tracking-tight">GitBiker</span>
      </div>
    </TitleBar>
  </header>

  {#if showSettings}
    <div class="flex-1 overflow-hidden animate-fade-up">
        <SettingsComponent onClose={() => (showSettings = false)} />
    </div>
  {:else if app.hasRepo}
    <!-- Secondary Tooling -->
    <Toolbar
      onOpenSettings={() => (showSettings = true)}
      onOpenPopover={() => activePopover = 'repo'}
      onOpenMultiRepo={() => activePopover = 'multiRepo'}
    />

    {#if app.showAgentDashboard}
      <div class="flex-1 overflow-hidden border-t border-ink-10 animate-fade-up">
        <AgentDashboard />
      </div>
    {:else}
      <!-- Main Accelerated Workspace -->
      <main class="flex-1 flex overflow-hidden">
        
        <!-- Sidebar -->
        <aside 
          class="bg-bg border-r border-ink-10 flex flex-col flex-shrink-0 transition-[width] duration-150 ease-in-out"
          style:width="{sidebarWidth}px"
        >
          <BranchSidebar />
        </aside>

        <!-- Resize Handle -->
        <div 
          class="w-px bg-ink-10 hover:bg-accent hover:w-1 transition-all cursor-col-resize z-10"
          onmousedown={handleResizeStart('sidebar')}
        ></div>

        <!-- Center View -->
        <section class="flex-1 flex flex-col bg-bg-deep min-w-0">
          {#if app.viewMode === 'github'}
              <GitHubDashboard />
          {:else if app.viewMode === 'conflict-resolution'}
              <ConflictResolver />
          {:else if app.selectedFile || app.currentDiff}
              <div class="h-9 flex items-center px-6 gap-2 border-b border-ink-10 bg-bg/50 backdrop-blur-sm text-[11px] font-medium text-ink-50">
                  <button class="hover:text-ink transition-colors" onclick={() => {app.selectedFile = null; app.currentDiff = null;}}>Commit Log</button>
                  <span>/</span>
                  <span class="truncate text-ink italic font-semibold">{app.selectedFile?.split('/').pop()}</span>
              </div>
              <div class="flex-1 overflow-hidden bg-bg">
                  <DiffViewer />
              </div>
          {:else}
              <CommitLog />
          {/if}
        </section>

        <!-- Resize Handle -->
        <div 
          class="w-px bg-ink-10 hover:bg-accent hover:w-1 transition-all cursor-col-resize z-10"
          onmousedown={handleResizeStart('right')}
        ></div>

        <!-- Right Pane -->
        <aside 
          class="bg-bg border-l border-ink-10 flex flex-col flex-shrink-0 transition-[width] duration-150 ease-in-out"
          style:width="{rightWidth}px"
        >
          {#if app.viewMode === 'commit-detail'}
              <CommitFileList />
          {:else}
              <FileTree />
          {/if}
        </aside>
      </main>
    {/if}

    <InlineTerminal visible={app.showTerminal} onClose={() => (app.showTerminal = false)} />

  {:else}
    <div class="flex-1 flex items-center justify-center bg-bg-deep">
        <Welcome 
            onOpenRepo={(path) => app.openRepo(path)}
            onClone={() => showCloneDialog = true}
            gitEnv={app.gitEnv}
        />
    </div>
  {/if}

  <!-- Modals -->
  <RepoPopover open={activePopover === 'repo'} onClose={() => activePopover = null} onClone={() => {activePopover = null; showCloneDialog = true;}} />
  <MultiRepoPopover open={activePopover === 'multiRepo'} onClose={() => activePopover = null} />
  <CommandPalette open={showCommandPalette} onClose={() => showCommandPalette = false} onOpenSettings={() => {showCommandPalette = false; showSettings = true;}} />
  {#if showCloneDialog} <CloneDialog onClose={() => showCloneDialog = false} onCloned={(path) => {showCloneDialog = false; app.openRepo(path);}} /> {/if}
  <Toast />
</div>
