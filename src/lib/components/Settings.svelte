<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { gitRemoteList, gitRemoteAdd, gitRemoteRemove, gitRemoteRename, detectEditors, listAiModels } from '$lib/git/commands';
  import type { EditorInfo, AiModelInfo } from '$lib/git/commands';
  import type { RemoteInfo } from '$lib/git/types';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  // Editor settings state
  let detectedEditors = $state<EditorInfo[]>([]);
  let detectingEditors = $state(false);
  let customEditorCommand = $state('');
  const CUSTOM_VALUE = '__custom__';

  // Derive current selection value for the dropdown
  let editorSelectValue = $derived.by(() => {
    const pref = app.preferredEditor;
    if (pref === null) return '';
    if (detectedEditors.some((e) => e.command === pref)) return pref;
    return CUSTOM_VALUE;
  });

  async function loadEditors() {
    detectingEditors = true;
    try {
      detectedEditors = await detectEditors();
    } catch {
      detectedEditors = [];
    } finally {
      detectingEditors = false;
    }
  }

  async function handleEditorChange(value: string) {
    if (value === '') {
      await app.savePreferredEditor(null);
      app.addToast('已切換為自動偵測編輯器', 'success');
    } else if (value === CUSTOM_VALUE) {
      customEditorCommand = app.preferredEditor ?? '';
    } else {
      await app.savePreferredEditor(value);
      const name = detectedEditors.find((e) => e.command === value)?.name ?? value;
      app.addToast(`已設定預設編輯器：${name}`, 'success');
    }
  }

  async function handleCustomEditorSave() {
    const cmd = customEditorCommand.trim();
    if (!cmd) return;
    await app.savePreferredEditor(cmd);
    app.addToast(`已設定自訂編輯器：${cmd}`, 'success');
  }

  // Remote management state
  let remotes = $state<RemoteInfo[]>([]);
  let loadingRemotes = $state(false);
  let showAddForm = $state(false);
  let newRemoteName = $state('');
  let newRemoteUrl = $state('');
  let renamingRemote = $state<string | null>(null);
  let renameValue = $state('');

  async function loadRemotes() {
    if (!app.repoPath) return;
    loadingRemotes = true;
    try {
      remotes = await gitRemoteList(app.repoPath);
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    } finally {
      loadingRemotes = false;
    }
  }

  async function handleAddRemote() {
    if (!app.repoPath || !newRemoteName.trim() || !newRemoteUrl.trim()) return;
    try {
      await gitRemoteAdd(app.repoPath, newRemoteName.trim(), newRemoteUrl.trim());
      app.addToast(`已新增 remote: ${newRemoteName.trim()}`, 'success');
      newRemoteName = '';
      newRemoteUrl = '';
      showAddForm = false;
      await loadRemotes();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  async function handleRemoveRemote(name: string) {
    if (!app.repoPath) return;
    try {
      await gitRemoteRemove(app.repoPath, name);
      app.addToast(`已移除 remote: ${name}`, 'success');
      await loadRemotes();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  function startRename(name: string) {
    renamingRemote = name;
    renameValue = name;
  }

  async function handleRenameRemote() {
    if (!app.repoPath || !renamingRemote || !renameValue.trim()) return;
    try {
      await gitRemoteRename(app.repoPath, renamingRemote, renameValue.trim());
      app.addToast(`已重新命名 remote: ${renamingRemote} → ${renameValue.trim()}`, 'success');
      renamingRemote = null;
      await loadRemotes();
    } catch (e: unknown) {
      app.addToast(String(e), 'error');
    }
  }

  // AI model listing state
  let aiModels = $state<AiModelInfo[]>([]);
  let loadingModels = $state(false);
  let modelsError = $state('');

  async function fetchAiModels() {
    modelsError = '';
    loadingModels = true;
    try {
      aiModels = await listAiModels(
        app.aiProvider,
        app.aiApiKey,
        app.aiProvider === 'ollama' ? app.aiOllamaEndpoint : undefined,
      );
    } catch (e: unknown) {
      aiModels = [];
      modelsError = String(e);
    } finally {
      loadingModels = false;
    }
  }

  // Fetch models when provider, API key, or Ollama endpoint changes
  $effect(() => {
    const _provider = app.aiProvider;
    const _key = app.aiApiKey;
    const _endpoint = app.aiOllamaEndpoint;
    // Only fetch when we have the required credential
    if (_provider === 'ollama' || _key) {
      fetchAiModels();
    } else {
      aiModels = [];
    }
  });

  // Load editors on mount
  $effect(() => {
    loadEditors();
  });

  // Load remotes when settings opens and repo is available
  $effect(() => {
    if (app.repoPath) loadRemotes();
  });

  const shortcuts = [
    { keys: 'Ctrl+Enter', action: 'Commit' },
    { keys: 'Ctrl+Shift+P', action: '命令面板（Coming soon）' },
    { keys: 'Ctrl+1', action: '聚焦 File Tree' },
    { keys: 'Ctrl+2', action: '聚焦 Diff Viewer' },
    { keys: 'Ctrl+3', action: '聚焦 Commit History' },
    { keys: 'Alt+E', action: '在編輯器開啟' },
    { keys: 'Alt+O', action: '在資料夾開啟' },
    { keys: 'Alt+T', action: '在終端機開啟' },
    { keys: 'Escape', action: '關閉 Dialog / Settings' },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings">
  <div class="settings-header">
    <button class="back-btn" onclick={onClose}>← Back</button>
    <span class="settings-title">Settings</span>
  </div>

  <div class="settings-body">
    <div class="section">
      <div class="section-title">Appearance</div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Theme</span>
          <span class="setting-desc">
            {#if app.theme === 'system'}
              跟隨系統主題（目前：{app.resolvedTheme === 'dark' ? '深色' : '淺色'}）
            {:else if app.theme === 'dark'}
              手動深色主題
            {:else}
              手動淺色主題
            {/if}
          </span>
        </div>
        <div class="theme-segmented" role="radiogroup" aria-label="Theme">
          {#each [
            { value: 'system' as const, label: '⚙ System' },
            { value: 'dark' as const, label: '☽ Dark' },
            { value: 'light' as const, label: '☀ Light' },
          ] as option}
            <button
              class="theme-option"
              class:active={app.theme === option.value}
              role="radio"
              aria-checked={app.theme === option.value}
              onclick={() => app.setTheme(option.value)}
            >
              {option.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <div class="section">
      <div class="section-title">預設編輯器</div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">編輯器</span>
          <span class="setting-desc">
            {#if detectingEditors}
              偵測中…
            {:else if app.preferredEditor === null}
              將自動偵測可用的編輯器
            {:else if editorSelectValue === CUSTOM_VALUE}
              自訂指令：{app.preferredEditor}
            {:else}
              已選擇 {detectedEditors.find((e) => e.command === app.preferredEditor)?.name ?? app.preferredEditor}
            {/if}
          </span>
        </div>
        <select
          class="editor-select"
          value={editorSelectValue}
          onchange={(e) => handleEditorChange(e.currentTarget.value)}
          disabled={detectingEditors}
        >
          <option value="">自動偵測</option>
          {#each detectedEditors as editor (editor.id)}
            <option value={editor.command}>{editor.name}</option>
          {/each}
          <option value={CUSTOM_VALUE}>自訂指令…</option>
        </select>
      </div>
      {#if editorSelectValue === CUSTOM_VALUE}
        <div class="custom-editor-row">
          <input
            type="text"
            class="remote-input"
            placeholder="e.g. notepad++, vim, emacs"
            bind:value={customEditorCommand}
            onkeydown={(e) => e.key === 'Enter' && handleCustomEditorSave()}
          />
          <button
            class="remote-action-btn primary"
            onclick={handleCustomEditorSave}
            disabled={!customEditorCommand.trim()}
          >儲存</button>
        </div>
      {/if}
    </div>

    <div class="section">
      <div class="section-title">Auto Fetch</div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">自動 Fetch</span>
          <span class="setting-desc">
            {#if app.autoFetchEnabled}
              每 {app.autoFetchInterval} 分鐘自動 fetch remote 更新
            {:else}
              關閉（手動 fetch）
            {/if}
          </span>
        </div>
        <div class="auto-fetch-controls">
          <select
            class="editor-select"
            value={app.autoFetchEnabled ? String(app.autoFetchInterval) : 'off'}
            onchange={(e) => {
              const val = e.currentTarget.value;
              if (val === 'off') {
                app.setAutoFetch(false);
              } else {
                app.setAutoFetch(true, Number(val));
              }
            }}
          >
            <option value="off">關閉</option>
            <option value="1">每 1 分鐘</option>
            <option value="3">每 3 分鐘</option>
            <option value="5">每 5 分鐘</option>
            <option value="10">每 10 分鐘</option>
            <option value="15">每 15 分鐘</option>
          </select>
        </div>
      </div>
    </div>

    <div class="section">
      <div class="section-title">AI Commit Message</div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">AI 提供者</span>
          <span class="setting-desc">選擇用來生成 commit message 的 AI 服務</span>
        </div>
        <select
          class="editor-select"
          value={app.aiProvider}
          onchange={(e) => {
            const val = e.currentTarget.value as 'gemini' | 'openai' | 'ollama';
            app.aiProvider = val;
            app.aiModel = '';
            app.saveAiSettings();
          }}
        >
          <option value="gemini">Gemini</option>
          <option value="openai">OpenAI</option>
          <option value="ollama">Ollama (本地)</option>
        </select>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">模型</span>
          <span class="setting-desc">
            {#if loadingModels}
              載入模型列表中...
            {:else if modelsError}
              {app.aiProvider === 'ollama' ? '無法連接 Ollama' : '請先填入 API Key'}
            {:else if aiModels.length > 0}
              共 {aiModels.length} 個可用模型
            {:else}
              {app.aiProvider === 'ollama' ? '請確認 Ollama 服務已啟動' : '請先填入 API Key 以載入模型'}
            {/if}
          </span>
        </div>
        <div class="model-select-row">
          <select
            class="editor-select"
            value={app.aiModel}
            disabled={loadingModels}
            onchange={(e) => {
              app.aiModel = e.currentTarget.value;
              app.saveAiSettings();
            }}
          >
            {#if app.aiProvider === 'gemini'}
              <option value="">gemini-2.0-flash（預設）</option>
            {:else if app.aiProvider === 'openai'}
              <option value="">gpt-4o-mini（預設）</option>
            {:else}
              <option value="">llama3（預設）</option>
            {/if}
            {#each aiModels as model}
              <option value={model.id}>{model.name}</option>
            {/each}
          </select>
          <button
            class="refresh-btn"
            title="重新載入模型列表"
            disabled={loadingModels}
            onclick={fetchAiModels}
          >
            {#if loadingModels}
              ⟳
            {:else}
              ↻
            {/if}
          </button>
        </div>
      </div>

      {#if app.aiProvider !== 'ollama'}
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">API Key</span>
            <span class="setting-desc">
              {app.aiProvider === 'gemini' ? 'Google AI Studio API Key' : 'OpenAI API Key'}
            </span>
          </div>
          <input
            type="password"
            class="remote-input ai-key-input"
            placeholder="輸入 API Key..."
            value={app.aiApiKey}
            onchange={(e) => {
              app.aiApiKey = e.currentTarget.value;
              app.saveAiSettings();
            }}
          />
        </div>
      {/if}

      {#if app.aiProvider === 'ollama'}
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Ollama Endpoint</span>
            <span class="setting-desc">Ollama 服務位址</span>
          </div>
          <input
            type="text"
            class="remote-input ai-key-input"
            placeholder="http://localhost:11434"
            value={app.aiOllamaEndpoint}
            onchange={(e) => {
              app.aiOllamaEndpoint = e.currentTarget.value;
              app.saveAiSettings();
            }}
          />
        </div>
      {/if}

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">生成語言</span>
          <span class="setting-desc">Commit message 的語言偏好</span>
        </div>
        <select
          class="editor-select"
          value={app.aiLanguage}
          onchange={(e) => {
            app.aiLanguage = e.currentTarget.value as 'zh-TW' | 'en' | 'auto';
            app.saveAiSettings();
          }}
        >
          <option value="zh-TW">中文（前綴英文）</option>
          <option value="en">English</option>
          <option value="auto">自動（跟隨歷史風格）</option>
        </select>
      </div>

      <div class="setting-row" style="align-items: flex-start;">
        <div class="setting-info">
          <span class="setting-label">自訂提示詞</span>
          <span class="setting-desc">額外指示給 AI（選填）</span>
        </div>
      </div>
      <textarea
        class="ai-prompt-textarea"
        placeholder="例如：保持簡潔、不超過一行..."
        value={app.aiCustomPrompt}
        onchange={(e) => {
          app.aiCustomPrompt = e.currentTarget.value;
          app.saveAiSettings();
        }}
      ></textarea>
    </div>

    <div class="section">
      <div class="section-title">Keyboard Shortcuts</div>
      <div class="shortcuts-list">
        {#each shortcuts as shortcut}
          <div class="shortcut-row">
            <kbd class="shortcut-keys">{shortcut.keys}</kbd>
            <span class="shortcut-action">{shortcut.action}</span>
          </div>
        {/each}
      </div>
    </div>

    {#if app.hasRepo}
    <div class="section">
      <div class="section-title">Repository — Remotes</div>
      {#if loadingRemotes}
        <div class="remote-loading">Loading...</div>
      {:else}
        <div class="remote-list">
          {#each remotes as remote (remote.name)}
            <div class="remote-item">
              {#if renamingRemote === remote.name}
                <div class="remote-rename-form">
                  <input
                    type="text"
                    class="remote-input"
                    bind:value={renameValue}
                    onkeydown={(e) => e.key === 'Enter' && handleRenameRemote()}
                  />
                  <button class="remote-action-btn" onclick={handleRenameRemote}>Save</button>
                  <button class="remote-action-btn" onclick={() => renamingRemote = null}>Cancel</button>
                </div>
              {:else}
                <div class="remote-info">
                  <span class="remote-name">{remote.name}</span>
                  <span class="remote-url">{remote.url}</span>
                </div>
                <div class="remote-actions">
                  <button class="remote-action-btn" onclick={() => startRename(remote.name)}>Rename</button>
                  <button class="remote-action-btn danger" onclick={() => handleRemoveRemote(remote.name)}>Remove</button>
                </div>
              {/if}
            </div>
          {:else}
            <div class="remote-empty">No remotes configured. Add one to push and pull.</div>
          {/each}
        </div>

        {#if showAddForm}
          <div class="add-remote-form">
            <input type="text" class="remote-input" placeholder="Name (e.g. origin)" bind:value={newRemoteName} />
            <input type="text" class="remote-input" placeholder="URL (https:// or git@...)" bind:value={newRemoteUrl} />
            <div class="add-remote-actions">
              <button class="remote-action-btn" onclick={() => { showAddForm = false; newRemoteName = ''; newRemoteUrl = ''; }}>Cancel</button>
              <button class="remote-action-btn primary" onclick={handleAddRemote} disabled={!newRemoteName.trim() || !newRemoteUrl.trim()}>Add</button>
            </div>
          </div>
        {:else}
          <button class="add-remote-btn" onclick={() => showAddForm = true}>+ Add Remote</button>
        {/if}
      {/if}
    </div>
    {/if}

    <div class="section">
      <div class="section-title">About</div>
      <div class="about-info">
        <div class="about-row">
          <span class="about-label">Version</span>
          <span class="about-value">0.2.0</span>
        </div>
        <div class="about-row">
          <span class="about-label">Framework</span>
          <span class="about-value">Tauri 2.x + Svelte 5</span>
        </div>
        <div class="about-row">
          <span class="about-label">License</span>
          <span class="about-value">MIT</span>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }
  .settings-header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .back-btn {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--font-size-md);
  }
  .back-btn:hover { text-decoration: underline; }
  .settings-title {
    font-size: var(--font-size-lg);
    font-weight: 600;
  }
  .settings-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
    max-width: 600px;
    margin: 0 auto;
    width: 100%;
  }
  .section {
    margin-bottom: 32px;
  }
  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    margin-bottom: var(--space-md);
    padding-bottom: var(--space-xs);
    border-bottom: 1px solid var(--border);
  }
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) 0;
  }
  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .setting-label { font-size: var(--font-size-md); }
  .setting-desc { font-size: 11px; color: var(--text-muted); }
  .theme-segmented {
    display: flex;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .theme-option {
    flex: 1;
    padding: 6px 12px;
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    white-space: nowrap;
  }
  .theme-option:hover:not(.active) {
    background: var(--bg-hover);
  }
  .theme-option.active {
    background: var(--accent);
    color: var(--bg-primary);
    font-weight: 600;
  }
  .theme-option:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) 0;
  }
  .shortcut-keys {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
  }
  .shortcut-action {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  .about-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }
  .about-row {
    display: flex;
    justify-content: space-between;
    font-size: var(--font-size-sm);
  }
  .about-label { color: var(--text-secondary); }
  .about-value { color: var(--text-primary); font-family: var(--font-mono); }

  /* Remote Management */
  .remote-loading {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    padding: var(--space-sm) 0;
  }
  .remote-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  .remote-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .remote-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }
  .remote-name {
    font-size: var(--font-size-md);
    font-weight: 600;
    color: var(--text-primary);
  }
  .remote-url {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .remote-actions {
    display: flex;
    gap: var(--space-xs);
    flex-shrink: 0;
  }
  .remote-action-btn {
    font-size: 11px;
    color: var(--accent);
    background: none;
    border: none;
    cursor: pointer;
    font-family: var(--font-ui);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }
  .remote-action-btn:hover { background: var(--bg-hover); }
  .remote-action-btn.danger { color: var(--error); }
  .remote-action-btn.danger:hover { background: rgba(255, 107, 107, 0.1); }
  .remote-action-btn.primary {
    background: var(--accent);
    color: var(--bg-primary);
  }
  .remote-action-btn.primary:hover { filter: brightness(1.1); }
  .remote-action-btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .remote-rename-form {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
  }
  .remote-input {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    padding: var(--space-xs) var(--space-sm);
    outline: none;
  }
  .remote-input:focus { border-color: var(--accent); }
  .remote-input::placeholder { color: var(--text-muted); }
  .remote-empty {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    padding: var(--space-md) 0;
    text-align: center;
  }
  .add-remote-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    margin-top: var(--space-sm);
    padding: var(--space-sm);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .add-remote-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-xs);
  }
  .add-remote-btn {
    margin-top: var(--space-sm);
    font-size: var(--font-size-sm);
    color: var(--accent);
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    cursor: pointer;
    font-family: var(--font-ui);
  }
  .add-remote-btn:hover { border-color: var(--accent); background: var(--bg-hover); }

  /* Editor Settings */
  .editor-select {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    padding: 6px 12px;
    cursor: pointer;
    outline: none;
  }
  .editor-select:focus { border-color: var(--accent); }
  .editor-select:disabled { opacity: 0.5; cursor: not-allowed; }
  .custom-editor-row {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    margin-top: var(--space-sm);
  }

  /* AI Settings */
  .ai-key-input {
    width: 200px;
    flex: none;
  }
  .ai-model-input {
    width: 160px;
    flex: none;
  }
  .model-select-row {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
  .refresh-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 14px;
    padding: 4px 6px;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .refresh-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .ai-prompt-textarea {
    width: 100%;
    min-height: 60px;
    max-height: 120px;
    resize: vertical;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    padding: var(--space-xs) var(--space-sm);
    outline: none;
  }
  .ai-prompt-textarea:focus { border-color: var(--accent); }
  .ai-prompt-textarea::placeholder { color: var(--text-muted); }
</style>
