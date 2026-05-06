<script lang="ts">
  import { marked } from 'marked';
  import UiDialog from '$lib/components/ui/dialog.svelte';
  import UiButton from '$lib/components/ui/button.svelte';

  interface Props {
    open: boolean;
    explanation: string;
    onClose: () => void;
  }

  let { open, explanation, onClose }: Props = $props();

  const renderedHtml = $derived.by(() => {
    try {
      return marked.parse(explanation);
    } catch (e) {
      return explanation;
    }
  });

  function handleOpenChange(v: boolean) {
    if (!v) onClose();
  }
</script>

<UiDialog
  bind:open
  onOpenChange={handleOpenChange}
  title="AI 代碼解釋"
  description="由 AI 分析提供的代碼邏輯說明"
>
  <div class="ai-explanation-content markdown-body">
    {@html renderedHtml}
  </div>

  {#snippet footer()}
    <UiButton onclick={onClose}>關閉</UiButton>
  {/snippet}
</UiDialog>

<style>
  .ai-explanation-content {
    max-height: 400px;
    overflow-y: auto;
    font-size: var(--text-lg);
    line-height: 1.6;
    color: var(--text-primary);
    padding: var(--space-sm);
    background: var(--bg-surface);
    border-radius: var(--radius-md);
  }

  /* Basic Markdown styling */
  :global(.markdown-body pre) {
    background: var(--bg-secondary);
    padding: var(--space-sm);
    border-radius: var(--radius-sm);
    overflow-x: auto;
    margin: var(--space-sm) 0;
  }
  :global(.markdown-body code) {
    font-family: var(--font-mono);
    background: var(--bg-secondary);
    padding: 2px 4px;
    border-radius: 4px;
    font-size: 0.9em;
  }
  :global(.markdown-body p) {
    margin-bottom: var(--space-sm);
  }
  :global(.markdown-body ul, .markdown-body ol) {
    padding-left: var(--space-md);
    margin-bottom: var(--space-sm);
  }
</style>
