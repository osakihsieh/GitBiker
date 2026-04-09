<script lang="ts">
  import { extractErrorMessage } from '$lib/utils/error';
  import { invoke } from '@tauri-apps/api/core';
  import { app } from '$lib/stores/app.svelte';
  import UiDialog from '$lib/components/ui/dialog.svelte';
  import UiButton from '$lib/components/ui/button.svelte';
  import UiInput from '$lib/components/ui/input.svelte';
  import UiLabel from '$lib/components/ui/label.svelte';

  interface Props {
    onClose: () => void;
    onCloned: (path: string) => void;
  }

  let { onClose, onCloned }: Props = $props();

  let url = $state('');
  let destPath = $state('');
  let cloning = $state(false);
  let progress = $state('');
  let error = $state('');

  let open = $state(true);

  function handleOpenChange(v: boolean) {
    if (!v && !cloning) onClose();
  }

  async function handleClone() {
    if (!url.trim() || !destPath.trim() || cloning) return;
    error = '';
    cloning = true;
    progress = 'Cloning...';

    try {
      await invoke('git_clone', { url: url.trim(), dest: destPath.trim() });
      progress = 'Done!';
      app.addToast('Clone 完成', 'success');
      onCloned(destPath.trim());
    } catch (e: unknown) {
      const msg = extractErrorMessage(e);
      if (msg.includes('Authentication') || msg.includes('could not read Username')) {
        error = '認證失敗 — 請確認 URL 和 credential helper 設定';
      } else if (msg.includes('not found') || msg.includes('Repository not found')) {
        error = '找不到 repository — 請確認 URL 是否正確';
      } else {
        error = msg;
      }
    } finally {
      cloning = false;
    }
  }

  function guessDestName(): string {
    if (!url) return '';
    const match = url.match(/\/([^/]+?)(?:\.git)?$/);
    return match ? match[1] : '';
  }

  function handleUrlChange() {
    if (!destPath) {
      const name = guessDestName();
      if (name) {
        destPath = `C:\\Users\\${name}`;
      }
    }
  }
</script>

<UiDialog bind:open onOpenChange={handleOpenChange} title="Clone Repository" description="從 URL 複製 Git repository">
  {#snippet footer()}
    <UiButton variant="secondary" onclick={onClose} disabled={cloning}>Cancel</UiButton>
    <UiButton
      onclick={handleClone}
      disabled={!url.trim() || !destPath.trim()}
      loading={cloning}
    >
      {cloning ? 'Cloning...' : 'Clone'}
    </UiButton>
  {/snippet}

  <div class="flex flex-col gap-4">
    <div class="flex flex-col gap-1.5">
      <UiLabel for="clone-url">Repository URL</UiLabel>
      <UiInput
        id="clone-url"
        type="text"
        bind:value={url}
        oninput={handleUrlChange}
        placeholder="https://github.com/user/repo.git"
        disabled={cloning}
        autofocus
      />
    </div>

    <div class="flex flex-col gap-1.5">
      <UiLabel for="clone-dest">Clone to</UiLabel>
      <UiInput
        id="clone-dest"
        type="text"
        bind:value={destPath}
        placeholder="C:\Users\you\projects\repo"
        disabled={cloning}
      />
    </div>

    {#if error}
      <div class="flex items-start gap-2 rounded-sm border border-[var(--error)]/30 bg-[var(--error)]/10 px-3 py-2.5 text-xs text-[var(--error)]">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" class="mt-0.5 shrink-0">
          <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        {error}
      </div>
    {/if}

    {#if cloning}
      <div class="flex flex-col gap-1.5">
        <div class="h-1 w-full overflow-hidden rounded-full bg-[var(--bg-surface)]">
          <div class="h-full w-2/5 animate-[indeterminate_1.5s_ease-in-out_infinite] rounded-full bg-[var(--accent)]"></div>
        </div>
        <span class="text-[10px] text-[var(--text-muted)]">{progress}</span>
      </div>
    {/if}
  </div>
</UiDialog>

<style>
  @keyframes indeterminate {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(350%); }
  }
</style>
