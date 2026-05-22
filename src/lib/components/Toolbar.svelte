<script lang="ts">
  import { extractErrorMessage } from '$lib/utils/error';
  import { app } from '$lib/stores/app.svelte';
  import { conflicts } from '$lib/stores/conflictStore.svelte';
  import { multiRepo } from '$lib/stores/multiRepoStore.svelte';
  import {
    gitPush,
    gitPushTags,
    gitPull,
    gitFetch,
    gitBranches,
    gitSwitchBranch,
    openInFolder,
    openInEditor,
    openInTerminal,
  } from '$lib/git/commands';
  import BranchManager from './BranchManager.svelte';

  let branchDropdownOpen = $state(false);
  let branchManagerOpen = $state(false);
  let pushing = $state(false);
  let pushingTags = $state(false);
  let pulling = $state(false);
  let fetching = $state(false);

  // Derive repo display name from path
  let repoName = $derived(
    app.repoPath ? (app.repoPath.split(/[/\\]/).filter(Boolean).pop() ?? app.repoPath) : 'No repo',
  );

  async function handleOpenFolder() {
    if (!app.repoPath) return;
    try {
      await openInFolder(app.repoPath);
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleOpenEditor() {
    if (!app.repoPath) return;
    try {
      await openInEditor(app.repoPath, app.preferredEditor ?? undefined);
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleOpenTerminal() {
    if (!app.repoPath) return;
    try {
      await openInTerminal(app.repoPath);
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handlePush() {
    if (!app.repoPath || pushing) return;
    pushing = true;
    try {
      const result = await gitPush(app.repoPath);
      if (result.success) {
        app.addToast(`Pushed to ${result.remote}/${result.branch}`, 'success');
      } else {
        app.addToast(result.message, 'error', false);
      }
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error', false);
    } finally {
      pushing = false;
    }
  }

  async function handlePushTags() {
    if (!app.repoPath || pushingTags) return;
    pushingTags = true;
    try {
      const result = await gitPushTags(app.repoPath);
      if (result.success) {
        app.addToast(`已推送所有 Tags 到 ${result.remote}`, 'success');
      } else {
        app.addToast(result.message, 'error', false);
      }
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error', false);
    } finally {
      pushingTags = false;
    }
  }

  async function handlePull() {
    if (!app.repoPath || pulling) return;
    pulling = true;
    try {
      const result = await gitPull(app.repoPath);
      if (result.success) {
        app.addToast('Pull 完成', 'success');
      } else if (result.conflicts.length > 0) {
        app.addToast(`Pull 衝突：${result.conflicts.length} 個檔案需要解決`, 'error', false);
        await app.refreshAll();
        await app.enterConflictMode();
        return;
      } else {
        app.addToast(result.message, 'error', false);
      }
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error', false);
    } finally {
      pulling = false;
    }
  }

  async function handleFetch() {
    if (!app.repoPath || fetching) return;
    fetching = true;
    try {
      await gitFetch(app.repoPath);
      await app.refreshAll();
      app.addToast('Fetch 完成', 'success');
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      fetching = false;
    }
  }

  async function handleSwitchBranch(name: string) {
    if (!app.repoPath) return;
    try {
      await gitSwitchBranch(app.repoPath, name);
      app.currentBranch = name;
      branchDropdownOpen = false;
      app.addToast(`已切換到 ${name}`, 'success');
      await app.refreshAll();
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  function toggleBranchDropdown() {
    branchDropdownOpen = !branchDropdownOpen;
    if (branchDropdownOpen && app.repoPath) {
      gitBranches(app.repoPath).then((b) => {
        app.branches = b;
      });
    }
  }

  function closeBranchDropdown() {
    branchDropdownOpen = false;
  }

  function openBranchManager() {
    branchDropdownOpen = false;
    branchManagerOpen = true;
    if (app.repoPath) {
      gitBranches(app.repoPath).then((b) => {
        app.branches = b;
      });
    }
  }

  function closeBranchManager() {
    branchManagerOpen = false;
  }

  async function handleCleanup() {
    app.viewMode = 'ai-branch-manager';
  }

  function handleBranchKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && branchDropdownOpen) {
      e.stopPropagation();
      closeBranchDropdown();
    }
  }

  interface Props {
    onOpenSettings?: () => void;
    onOpenPopover?: () => void;
    onOpenMultiRepo?: () => void;
  }
  let { onOpenSettings, onOpenPopover, onOpenMultiRepo }: Props = $props();
</script>

<svelte:window onkeydown={handleBranchKeydown} />

<div class="toolbar h-11 bg-bg-deep border-b border-ink-10 flex items-center px-6 gap-6">
  <!-- Left: repo + branch info -->
  <div class="left-section flex items-center gap-4">
    <button
      class="multi-repo-btn p-1.5 rounded-lg hover:bg-ink-05 text-ink-50 hover:text-ink transition-colors relative"
      onclick={onOpenMultiRepo}
      aria-label="Multi-repo manager"
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1" />
        <rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1" />
      </svg>
      {#if multiRepo.dirtyCount > 0}
        <span
          class="absolute -top-1 -right-1 min-w-[14px] h-[14px] px-1 rounded-full bg-warn text-bg font-bold text-[9px] flex items-center justify-center"
        >
          {multiRepo.dirtyCount > 9 ? '9+' : multiRepo.dirtyCount}
        </span>
      {/if}
    </button>

    <div class="h-4 w-px bg-ink-10"></div>

    <button
      class="repo-btn flex flex-col items-start gap-0.5 hover:opacity-70 transition-opacity"
      onclick={onOpenPopover}
    >
      <span class="text-[9px] uppercase font-semibold text-ink-35 tracking-wider">REPOSITORY</span>
      <span class="flex items-center gap-1">
        <span class="text-[13px] font-semibold text-ink leading-none">{repoName}</span>
        <span class="text-[10px] text-ink-35">▼</span>
      </span>
    </button>

    <div class="h-4 w-px bg-ink-10"></div>

    <div class="branch-wrapper relative">
      <button class="branch-btn flex flex-col items-start gap-0.5 hover:opacity-70 transition-opacity" onclick={toggleBranchDropdown}>
        <span class="text-[9px] uppercase font-semibold text-ink-35 tracking-wider">BRANCH</span>
        <span class="flex items-center gap-1.5">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-accent"><line x1="6" y1="3" x2="6" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg>
          <span class="text-[13px] font-semibold text-ink leading-none">{app.currentBranch || 'main'}</span>
          <span class="text-[10px] text-ink-35">▼</span>
        </span>
      </button>

      {#if branchDropdownOpen}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="fixed inset-0 z-40" onclick={closeBranchDropdown}></div>
        <div class="absolute top-full left-0 mt-2 w-48 bg-card border border-ink-10 rounded-xl shadow-modal p-1.5 z-50 animate-fade-up">
          {#each app.branches.filter((b) => !b.is_remote) as branch}
            <button
              class="w-full text-left px-3 py-2 rounded-lg text-[13px] transition-colors flex items-center justify-between"
              class:bg-ink={branch.is_current}
              class:text-bg={branch.is_current}
              class:hover:bg-ink-05={!branch.is_current}
              onclick={() => handleSwitchBranch(branch.name)}
            >
              <span class="truncate">{branch.name}</span>
              {#if branch.is_current}<span class="w-1.5 h-1.5 rounded-full bg-accent"></span>{/if}
            </button>
          {/each}
          <div class="h-px bg-ink-10 my-1"></div>
          <button class="w-full text-left px-3 py-2 rounded-lg text-[11px] font-semibold text-accent hover:bg-accent-bg transition-colors" onclick={openBranchManager}>MANAGE BRANCHES...</button>
        </div>
      {/if}
    </div>
  </div>

  <div class="h-6 w-px bg-ink-10 mx-2"></div>

  <!-- Middle: external tools + git actions -->
  <div class="mid-section flex items-center gap-4">
    <div class="flex items-center gap-1 bg-ink-05 p-1 rounded-lg">
      <button class="p-1.5 rounded-md hover:bg-bg text-ink-50 hover:text-ink transition-all" onclick={handleOpenFolder} title="Open Explorer">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
      </button>
      <button class="p-1.5 rounded-md hover:bg-bg text-ink-50 hover:text-ink transition-all" onclick={handleOpenEditor} title="Open Editor">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" /></svg>
      </button>
      <button class="p-1.5 rounded-md hover:bg-bg text-ink-50 hover:text-ink transition-all" onclick={handleOpenTerminal} title="Open External Terminal">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5" /><line x1="12" y1="19" x2="20" y2="19" /></svg>
      </button>
      <div class="w-px h-3 bg-ink-10 mx-0.5"></div>
      <button class="p-1.5 rounded-md hover:bg-bg text-ink-50 hover:text-ink transition-all" class:bg-bg={app.showAgentDashboard} onclick={() => app.toggleAgentDashboard()} title="Agent Radar">
        <span class="text-[12px]">📡</span>
      </button>
    </div>

    <div class="flex items-center gap-2">
      <button class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-ink text-bg hover:opacity-90 transition-opacity disabled:opacity-50" onclick={handlePull} disabled={pulling}>
        {#if pulling}<span class="w-3 h-3 border-2 border-bg-deeper border-t-bg rounded-full animate-spin"></span>{:else}↓{/if}
        <span class="text-[12px] font-semibold">Pull</span>
      </button>
      <button class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-ink text-bg hover:opacity-90 transition-opacity disabled:opacity-50" onclick={handlePush} disabled={pushing}>
        {#if pushing}<span class="w-3 h-3 border-2 border-bg-deeper border-t-bg rounded-full animate-spin"></span>{:else}↑{/if}
        <span class="text-[12px] font-semibold">Push</span>
      </button>
    </div>
  </div>

  <div class="flex-1" data-tauri-drag-region></div>

  <button class="p-2 rounded-lg hover:bg-ink-05 text-ink-50 transition-colors" onclick={onOpenSettings}>
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
  </button>
</div>


