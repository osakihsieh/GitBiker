<script lang="ts">
  import { extractErrorMessage } from '$lib/utils/error';
  import { app } from '$lib/stores/app.svelte';
  import { gitInit, scanGitRepos } from '$lib/git/commands';
  import UiButton from '$lib/components/ui/button.svelte';
  import { cn } from '$lib/utils/cn';

  interface Props {
    onOpenRepo: (path: string) => void;
    onClone: () => void;
    onOpenMultiRepo?: (scanPath: string) => void;
  }

  let { onOpenRepo, onClone, onOpenMultiRepo }: Props = $props();

  async function handleOpenLocal() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        directory: true,
        title: 'Select Git Repository',
      });
      if (selected) {
        onOpenRepo(selected);
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleOpenMultiRepo() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        directory: true,
        title: '選擇包含多個 Git Repos 的資料夾',
      });
      if (selected) {
        const repos = await scanGitRepos(selected);
        if (repos.length === 0) {
          app.addToast('此資料夾中沒有找到 Git repositories', 'info');
        } else {
          onOpenMultiRepo?.(selected);
        }
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  async function handleInitRepo() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        directory: true,
        title: '選擇資料夾來初始化 Git Repository',
      });
      if (selected) {
        await gitInit(selected);
        app.addToast('Git repository 初始化成功', 'success');
        onOpenRepo(selected);
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }
</script>

<div class="flex h-full flex-col items-center justify-center gap-10 px-12 py-12">
  <!-- Logo -->
  <div class="flex flex-col items-center gap-3">
    <svg class="h-14 w-14" viewBox="0 0 64 64" fill="none">
      <circle cx="16" cy="44" r="10" stroke="var(--accent)" stroke-width="2" fill="none"/>
      <circle cx="48" cy="44" r="10" stroke="var(--accent)" stroke-width="2" fill="none"/>
      <path d="M16 44 L28 24 L40 44 L48 44" stroke="var(--accent)" stroke-width="2" fill="none" stroke-linejoin="round"/>
      <path d="M28 24 L40 24" stroke="var(--accent)" stroke-width="2"/>
      <circle cx="28" cy="24" r="2" fill="var(--accent)"/>
      <path d="M38 12 L44 12" stroke="var(--text-primary)" stroke-width="1.5" opacity="0.4"/>
      <circle cx="36" cy="12" r="2" fill="var(--text-primary)" opacity="0.4"/>
      <circle cx="46" cy="12" r="2" fill="var(--text-primary)" opacity="0.4"/>
    </svg>
    <div class="text-2xl font-light tracking-widest text-[var(--text-primary)]">GitBiker</div>
    <div class="text-xs tracking-wide text-[var(--text-muted)]">fast. minimal. yours.</div>
  </div>

  <!-- Actions -->
  <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
    {#each [
      { icon: '⇣', label: 'Clone a Repo', hint: 'from URL', action: onClone },
      { icon: '📂', label: 'Open Local', hint: 'from disk', action: handleOpenLocal },
      { icon: '+', label: 'Init New Repo', hint: 'git init', action: handleInitRepo },
      { icon: '⊞', label: 'Multi-Repo', hint: 'bulk ops', action: handleOpenMultiRepo },
    ] as item}
      <button
        class={cn(
          'flex flex-col items-center gap-2 rounded-md border border-[var(--border)]',
          'bg-[var(--bg-surface)] px-6 py-5 transition-all duration-150',
          'hover:border-[var(--accent)] hover:bg-[var(--bg-hover)]',
          'cursor-pointer min-w-[140px]',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]'
        )}
        onclick={item.action}
      >
        <span class="text-2xl text-[var(--accent)]">{item.icon}</span>
        <div class="flex flex-col items-center gap-0.5">
          <span class="text-sm font-medium text-[var(--text-primary)]">{item.label}</span>
          <span class="text-[10px] text-[var(--text-muted)]">{item.hint}</span>
        </div>
      </button>
    {/each}
  </div>

  <!-- Recent repos -->
  <div class="w-full max-w-md">
    <div class="mb-2 border-b border-[var(--border)] pb-1.5 text-[10px] font-semibold uppercase tracking-wide text-[var(--text-secondary)]">
      Recent Repos
    </div>

    {#if app.recentRepos.length > 0}
      <div class="flex flex-col">
        {#each app.recentRepos as repoPath}
          <button
            class={cn(
              'flex items-center gap-3 rounded-sm px-3 py-2 text-left',
              'bg-transparent border-none cursor-pointer w-full',
              'hover:bg-[var(--bg-hover)] transition-colors',
              'focus-visible:outline-none focus-visible:bg-[var(--bg-hover)]'
            )}
            onclick={() => onOpenRepo(repoPath)}
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="shrink-0">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            <div class="min-w-0 flex-1">
              <div class="text-sm font-medium text-[var(--text-primary)]">
                {repoPath.replace(/\\/g, '/').split('/').pop()}
              </div>
              <div class="truncate font-mono text-[10px] text-[var(--text-muted)]">{repoPath}</div>
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <div class="py-6 text-center text-sm text-[var(--text-muted)]">
        No repos yet. Clone or open one to get started.
      </div>
    {/if}
  </div>

  <div class="fixed bottom-3 text-[10px] text-[var(--text-muted)] opacity-40">v0.3.0</div>
</div>
