<script lang="ts">
  import '../app.css';
  import { app } from '$lib/stores/app.svelte';
  import { gitStatus, gitLog, gitBranches, gitDiff } from '$lib/git/commands';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import FileTree from '$lib/components/FileTree.svelte';
  import DiffViewer from '$lib/components/DiffViewer.svelte';
  import CommitLog from '$lib/components/CommitLog.svelte';
  import Welcome from '$lib/components/Welcome.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import CloneDialog from '$lib/components/CloneDialog.svelte';
  import Settings from '$lib/components/Settings.svelte';

  let showCloneDialog = $state(false);
  let showSettings = $state(false);

  async function openRepo(path: string) {
    app.loading = true;
    try {
      const [status, commits, branches] = await Promise.all([
        gitStatus(path),
        gitLog(path),
        gitBranches(path),
      ]);

      app.repoPath = path;
      app.stagedFiles = status.filter((f) => f.staging === 'Staged');
      app.unstagedFiles = status.filter((f) => f.staging === 'Unstaged');
      app.commits = commits;
      app.branches = branches;
      app.currentBranch = branches.find((b) => b.is_current)?.name || 'main';
      app.selectedFile = null;
      app.currentDiff = null;

      if (!app.recentRepos.includes(path)) {
        app.recentRepos = [path, ...app.recentRepos].slice(0, 10);
      }
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      app.loading = false;
    }
  }

  $effect(() => {
    const file = app.selectedFile;
    const repoPath = app.repoPath;
    if (file && repoPath) {
      gitDiff(repoPath, file)
        .then((diff) => { app.currentDiff = diff; })
        .catch((e) => { app.addToast(String(e), 'error'); });
    }
  });

  function handleClone() {
    showCloneDialog = true;
  }

  function handleCloned(path: string) {
    showCloneDialog = false;
    openRepo(path);
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
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
  {#if showSettings}
    <Settings onClose={() => showSettings = false} />
  {:else if app.hasRepo}
    <Toolbar onOpenSettings={() => showSettings = true} />
    <div class="main">
      <div class="sidebar">
        <FileTree />
      </div>
      <div class="resize-handle"></div>
      <div class="center" tabindex="-1">
        <DiffViewer />
      </div>
      <div class="resize-handle"></div>
      <div class="right" tabindex="-1">
        <CommitLog />
      </div>
    </div>
  {:else}
    <div class="welcome-toolbar">
      <span class="app-name">GitBiker</span>
      <button class="settings-btn" onclick={() => showSettings = true}>⚙</button>
    </div>
    <Welcome onOpenRepo={openRepo} onClone={handleClone} />
  {/if}

  {#if showCloneDialog}
    <CloneDialog
      onClose={() => showCloneDialog = false}
      onCloned={handleCloned}
    />
  {/if}

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
  .app-name {
    font-weight: 600;
    font-size: var(--font-size-lg);
    color: var(--text-secondary);
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
