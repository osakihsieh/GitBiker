<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { extractErrorMessage } from '$lib/utils/error';
  import { analyzeBranches, gitDeleteBranch } from '$lib/git/commands';
  import { marked } from 'marked';

  let loading = $state(false);
  let analysisResult = $state<string | null>(null);

  async function handleAnalyze() {
    if (!app.repoPath) return;
    loading = true;
    analysisResult = null;
    try {
      const result = await analyzeBranches({
        path: app.repoPath,
        provider: app.aiProvider,
        apiKey: app.aiApiKey,
        model: app.aiModel,
        language: app.aiLanguage,
        ollamaEndpoint: app.aiOllamaEndpoint,
      });
      analysisResult = result;
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    } finally {
      loading = false;
    }
  }

  async function deleteBranch(name: string) {
    if (!app.repoPath) return;
    if (confirm(`確定要刪除分支「${name}」？`)) {
      try {
        await gitDeleteBranch(app.repoPath, name);
        app.addToast(`已刪除分支: ${name}`, 'success');
        await app.refreshAll();
        // Clear analysis to force refresh
        analysisResult = null;
      } catch (e: unknown) {
        app.addToast(extractErrorMessage(e), 'error');
      }
    }
  }
</script>

<div class="ai-branch-manager">
  <div class="header">
    <h3>AI 智能分支管理</h3>
    <button class="analyze-btn" onclick={handleAnalyze} disabled={loading || !app.aiApiKey}>
      {#if loading}
        <span class="spinning">↻</span> 正在加速分析...
      {:else}
        ✨ 開始分析
      {/if}
    </button>
  </div>

  {#if !app.aiApiKey && app.aiProvider !== 'ollama'}
    <div class="warning-box">
      請先在設定中配置 AI API Key 以啟用智能分析。
    </div>
  {/if}

  {#if analysisResult}
    <div class="analysis-content">
      <div class="markdown-body">
        {@html marked(analysisResult)}
      </div>
      <div class="action-hint">
        <p>💡 您可以根據分析建議在側邊欄手動刪除分支，或使用下方的一鍵清理（開發中）。</p>
      </div>
    </div>
  {:else if !loading}
    <div class="empty-state">
      <p>點擊上方按鈕，讓 AI 為您的倉庫生產線進行健康檢查。</p>
      <ul>
        <li>偵測已合併但未刪除的分支</li>
        <li>識別長期未活躍的「殭屍」分支</li>
        <li>提供命名規範建議</li>
      </ul>
    </div>
  {/if}
</div>

<style>
  .ai-branch-manager {
    padding: 16px;
    background: var(--bg-secondary, #1e1e1e);
    border-radius: 8px;
    border: 1px solid var(--border-color, #333);
    margin: 8px;
    color: var(--text-color, #eee);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .analyze-btn {
    padding: 6px 12px;
    background: var(--primary-color, #007bff);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: var(--text-md);
    display: flex;
    align-items: center;
    gap: 6px;
    transition: opacity 0.2s;
  }

  .analyze-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .analyze-btn:disabled {
    background: var(--bg-tertiary, #333);
    color: var(--text-muted, #888);
    cursor: not-allowed;
  }

  .warning-box {
    padding: 12px;
    background: rgba(255, 193, 7, 0.1);
    border-left: 4px solid #ffc107;
    color: #ffc107;
    font-size: var(--text-md);
    margin-bottom: 16px;
  }

  .analysis-content {
    background: var(--bg-primary, #121212);
    padding: 16px;
    border-radius: 6px;
    font-size: var(--text-lg);
    line-height: 1.6;
  }

  .markdown-body :global(h1), .markdown-body :global(h2), .markdown-body :global(h3) {
    color: var(--primary-color, #007bff);
    margin-top: 16px;
    margin-bottom: 8px;
  }

  .markdown-body :global(ul) {
    padding-left: 20px;
  }

  .action-hint {
    margin-top: 20px;
    padding-top: 12px;
    border-top: 1px solid var(--border-color, #333);
    font-size: var(--text-sm);
    color: var(--text-muted, #888);
  }

  .empty-state {
    text-align: center;
    padding: 32px 0;
    color: var(--text-muted, #888);
  }

  .empty-state ul {
    text-align: left;
    display: inline-block;
    font-size: var(--text-md);
    margin-top: 12px;
  }

  .spinning {
    display: inline-block;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
