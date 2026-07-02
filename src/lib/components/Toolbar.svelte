<script lang="ts">
  import { type Snippet } from 'svelte';
  import { extractErrorMessage } from '$lib/utils/error';
  import { app } from '$lib/stores/app.svelte';
  import { conflicts } from '$lib/stores/conflictStore.svelte';
  import { multiRepo } from '$lib/stores/multiRepoStore.svelte';
  import {
    gitPush,
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

{#snippet pullIcon()}
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="M12 17V3" /><path d="m6 11 6 6 6-6" /><path d="M19 21H5" />
  </svg>
{/snippet}

{#snippet pushIcon()}
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="M12 3v14" /><path d="m6 9 6-6 6 6" /><path d="M19 21H5" />
  </svg>
{/snippet}

{#snippet fetchIcon()}
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" /><path d="M21 3v5h-5" /><path
      d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"
    /><path d="M3 21v-5h5" />
  </svg>
{/snippet}

{#snippet spinnerIcon()}
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2.5"
    stroke-linecap="round"
  >
    <path d="M21 12a9 9 0 1 1-6.219-8.56" />
  </svg>
{/snippet}

{#snippet gitAction({
  label,
  title,
  loading,
  onclick,
  icon,
}: {
  label: string;
  title: string;
  loading: boolean;
  onclick: () => void;
  icon: Snippet;
})}
  <button
    class="toolbar-action relative flex flex-col items-center justify-center w-14 h-14 rounded transition-all duration-100 active:scale-90 active:bg-accent/20 hover:bg-white/5 hover:text-accent {loading
      ? 'is-loading'
      : ''}"
    {onclick}
    disabled={loading}
    {title}
  >
    <span class="action-icon">{@render icon()}</span>
    <span class="action-spinner">{@render spinnerIcon()}</span>
    <span class="text-[10px] uppercase font-bold text-text-muted mt-1 action-label">{label}</span>
  </button>
{/snippet}

<div class="toolbar h-16 bg-bg-secondary border-b border-border flex items-center px-4 gap-2">
  <!-- Left: repo + branch info -->
  <div class="left-section flex-shrink-0 flex items-center gap-1">
    <button
      class="multi-repo-btn w-8 h-8 flex-shrink-0 flex items-center justify-center rounded hover:bg-white/10 transition-colors"
      onclick={onOpenMultiRepo}
      aria-label="Multi-repo manager ({app.isMac ? 'Cmd+M' : 'Ctrl+M'})"
      title="多倉庫管理 ({app.isMac ? 'Cmd+M' : 'Ctrl+M'})"
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="var(--text-muted)"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" />
        <rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" />
      </svg>
      {#if multiRepo.dirtyCount > 0}
        <span
          class="multi-repo-badge absolute top-1 right-1 w-2 h-2 bg-accent rounded-full"
          aria-label="{multiRepo.dirtyCount} repositories with changes"
        >
        </span>
      {/if}
    </button>

    <div class="toolbar-sep w-[1px] h-8 bg-border mx-2 flex-shrink-0"></div>

    <button
      class="repo-btn flex-shrink-0 min-w-[120px] flex flex-col items-start px-2 py-1 rounded hover:bg-white/5 transition-colors group"
      onclick={onOpenPopover}
      aria-label="Open repo switcher ({app.isMac ? 'Cmd+T' : 'Ctrl+T'})"
      title="切換倉庫 ({app.isMac ? 'Cmd+T' : 'Ctrl+T'})"
    >
      <span
        class="info-label text-[10px] uppercase tracking-wider text-text-muted font-bold group-hover:text-accent transition-colors"
        >repository</span
      >
      <div class="flex items-center gap-2">
        <span class="info-name text-[13px] text-text-primary font-medium">{repoName}</span>
        <span class="chevron text-[9px] text-text-muted">▼</span>
      </div>
    </button>

    <div class="toolbar-sep w-[1px] h-8 bg-border mx-2 flex-shrink-0"></div>

    <div class="branch-wrapper relative flex-shrink-0">
      <button
        class="branch-btn min-w-[120px] flex flex-col items-start px-2 py-1 rounded hover:bg-white/5 transition-colors group"
        onclick={toggleBranchDropdown}
        aria-label="Switch branch"
      >
        <span
          class="info-label text-[10px] uppercase tracking-wider text-text-muted font-bold group-hover:text-accent transition-colors"
          >branch</span
        >
        <div class="flex items-center gap-2">
          <span class="branch-icon text-monokai-purple">⑇</span>
          <span class="info-name text-[13px] text-text-primary font-medium"
            >{app.currentBranch || 'main'}</span
          >
          <span class="chevron text-[9px] text-text-muted">▼</span>
        </div>
      </button>

      {#if branchDropdownOpen}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="fixed inset-0 z-40" onclick={closeBranchDropdown}></div>
        <div
          class="branch-dropdown absolute top-full left-0 mt-1 w-56 bg-bg-surface border border-border rounded-lg shadow-2xl z-50 overflow-hidden backdrop-blur-xl"
        >
          <div class="max-h-64 overflow-y-auto py-1">
            {#each app.branches.filter((b) => !b.is_remote) as branch}
              <button
                class="w-full text-left px-4 py-2 text-[13px] hover:bg-accent hover:text-bg-primary transition-colors flex justify-between items-center"
                class:text-accent={branch.is_current}
                onclick={() => handleSwitchBranch(branch.name)}
              >
                {branch.name}
                {#if branch.is_current}<span class="w-2 h-2 bg-current rounded-full"></span>{/if}
              </button>
            {/each}
          </div>
          <div class="border-t border-border p-1">
            <button
              class="w-full text-left px-4 py-2 text-[12px] text-text-muted hover:text-text-primary transition-colors"
              onclick={openBranchManager}>管理分支...</button
            >
          </div>
        </div>
      {/if}

      {#if branchManagerOpen}
        <BranchManager open={branchManagerOpen} onClose={closeBranchManager} />
      {/if}
    </div>

    {#if conflicts.files.length > 0}
      <button
        class="conflict-badge ml-2 px-2 py-0.5 bg-error text-bg-primary text-[10px] font-bold rounded flex items-center gap-1 animate-pulse"
        onclick={() =>
          conflicts.isInConflictMode ? conflicts.exitConflictMode() : conflicts.enterConflictMode()}
        title={app.isMac ? 'Cmd+Shift+M' : 'Ctrl+Shift+M'}
      >
        <span>⚠</span>
        <span>{conflicts.files.length}</span>
      </button>
    {/if}
  </div>

  <div class="toolbar-sep tall w-[1px] h-10 bg-border mx-4"></div>

  <!-- Middle: external tools + git actions -->
  <div class="mid-section flex-1 flex items-center justify-between min-w-0">
    <div class="tool-group flex items-center gap-1">
      <button
        class="w-9 h-9 flex items-center justify-center rounded hover:bg-white/10 text-text-muted hover:text-text-primary transition-colors"
        onclick={handleOpenFolder}
        title="在檔案總管開啟 (Alt+O)"
        aria-label="Open in file explorer"
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
        </svg>
      </button>
      <button
        class="w-9 h-9 flex items-center justify-center rounded hover:bg-white/10 text-text-muted hover:text-text-primary transition-colors"
        onclick={handleOpenEditor}
        title="在編輯器開啟 (Alt+E)"
        aria-label="Open in editor"
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" />
        </svg>
      </button>
      <button
        class="w-9 h-9 flex items-center justify-center rounded hover:bg-white/10 text-text-muted hover:text-text-primary transition-colors {app.showAgentDashboard
          ? 'text-accent bg-white/5'
          : ''}"
        onclick={() => app.toggleAgentDashboard()}
        title="Agent Radar"
        aria-label="Toggle Agent Dashboard"
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="10" /><circle cx="12" cy="12" r="4" /><circle
            cx="12"
            cy="12"
            r="1"
          />
        </svg>
      </button>
      <button
        class="w-9 h-9 flex items-center justify-center rounded hover:bg-white/10 text-text-muted hover:text-text-primary transition-colors"
        onclick={handleOpenTerminal}
        title="開啟外部終端機 (Alt+T)"
        aria-label="Open external terminal"
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="4 17 10 11 4 5" /><line x1="12" y1="19" x2="20" y2="19" />
        </svg>
      </button>
      <button
        class="w-9 h-9 flex items-center justify-center rounded hover:bg-white/10 text-text-muted hover:text-text-primary transition-colors {app.showTerminal
          ? 'text-accent bg-white/5'
          : ''}"
        onclick={() => app.toggleTerminal()}
        title="切換內建終端機 ({app.isMac ? 'Cmd+`' : 'Ctrl+`'})"
        aria-label="Toggle inline terminal"
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="2" y="14" width="20" height="8" rx="2" ry="2" /><path d="M6 18l2-2-2-2" /><line
            x1="12"
            y1="18"
            x2="16"
            y2="18"
          />
        </svg>
      </button>
    </div>

    <div class="actions flex items-center gap-1">
      <!-- Undo/Redo (Visual Placeholder) -->
      <button
        class="toolbar-action flex flex-col items-center justify-center w-14 h-14 rounded opacity-50 cursor-not-allowed"
        title="Undo (Coming Soon)"
        disabled
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M9 14 4 9l5-5" /><path
            d="M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5v0a5.5 5.5 0 0 1-5.5 5.5H11"
          />
        </svg>
        <span class="text-[10px] uppercase font-bold text-text-muted mt-1">Undo</span>
      </button>
      <button
        class="toolbar-action flex flex-col items-center justify-center w-14 h-14 rounded opacity-50 cursor-not-allowed"
        title="Redo (Coming Soon)"
        disabled
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="m15 14 5-5-5-5" /><path
            d="M20 9H9.5A5.5 5.5 0 0 0 4 14.5v0A5.5 5.5 0 0 0 9.5 20H13"
          />
        </svg>
        <span class="text-[10px] uppercase font-bold text-text-muted mt-1">Redo</span>
      </button>

      <div class="toolbar-sep w-[1px] h-8 bg-border mx-1"></div>

      {@render gitAction({
        label: 'Pull',
        title: 'Git Pull',
        loading: pulling,
        onclick: handlePull,
        icon: pullIcon,
      })}
      {@render gitAction({
        label: 'Push',
        title: 'Git Push',
        loading: pushing,
        onclick: handlePush,
        icon: pushIcon,
      })}
      {@render gitAction({
        label: 'Fetch',
        title: 'Git Fetch',
        loading: fetching,
        onclick: handleFetch,
        icon: fetchIcon,
      })}

      <div class="toolbar-sep w-[1px] h-8 bg-border mx-1"></div>

      <button
        class="toolbar-action flex flex-col items-center justify-center w-14 h-14 rounded transition-all duration-100 active:scale-90 active:bg-accent/20 hover:bg-white/5 hover:text-accent"
        onclick={handleCleanup}
        title="AI 分支管理"
      >
        <span class="text-lg action-icon">✨</span>
        <span class="text-[10px] uppercase font-bold text-text-muted mt-1 action-label"
          >AI Clear</span
        >
      </button>
    </div>
  </div>

  <div class="drag-spacer flex-1 min-w-[20px] self-stretch" data-tauri-drag-region></div>

  <button
    class="w-10 h-10 flex items-center justify-center rounded-full hover:bg-white/10 text-text-muted hover:text-text-primary transition-colors"
    onclick={onOpenSettings}
    title="設定"
    aria-label="Settings"
  >
    <svg
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="3" />
      <path
        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
      />
    </svg>
  </button>
</div>
