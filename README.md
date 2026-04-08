# GitBiker 極！拜客

> 快速、極簡、開源的 Git GUI 客戶端 — 基於 Rust + Tauri + Svelte 打造

![version](https://img.shields.io/badge/version-0.3.0-blue)
![platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey)
![license](https://img.shields.io/badge/license-Friendly%20Non--Commercial-green)

---

## 為什麼選擇 GitBiker？

現有 Git GUI 工具都有取捨：

| 工具 | 問題 |
|------|------|
| GitKraken | Electron 底層，記憶體吃重，WSL2 支援差 |
| Sourcetree | 無 Linux 支援 |
| Fork / Sublime Merge | 閉源 |
| GitButler | Fair Source 授權，側重虛擬分支模式 |

**GitBiker** 目標是一個**快速、跨平台、開源**的 Git GUI，同時深度整合 AI 能力，讓提交訊息不再是負擔。

---

## 核心特色

### AI 自動生成 Commit Message

GitBiker 內建 AI 提交訊息生成器，支援三種 AI 提供者：

- **Google Gemini**（gemini-2.0-flash，預設）
- **OpenAI / ChatGPT**（相容 GPT-4o 等）
- **Ollama**（本地離線模型，完全免費）

只需一鍵，AI 會自動：
1. 分析已暫存的 Diff 內容
2. 參考最近 5 筆提交風格，維持一致性
3. 套用 Conventional Commits 格式（`feat:` / `fix:` / `refactor:` 等）
4. 以繁體中文撰寫內文描述

支援自訂系統提示詞，讓 AI 輸出符合你的團隊規範。

---

### 三欄工作區

```
┌─────────────┬───────────────────┬──────────────────┐
│  檔案樹      │   Diff 預覽        │   提交歷史圖      │
│  Staged /   │   Hunk 級暫存      │   多分支視覺化    │
│  Unstaged   │   行內 +/- 標記    │   搜尋 / 篩選     │
└─────────────┴───────────────────┴──────────────────┘
```

### Hunk 級精細操作

不只能暫存整個檔案，還能：
- **Stage Hunk** — 只暫存某一段程式碼
- **Unstage Hunk** — 精準移除暫存
- **Stash Hunk** — 臨時保存單一改動區塊

### 分支管理

- 建立、切換、重命名、刪除本地/遠端分支
- 視覺化 Ahead / Behind 計數
- 批量清理超過 30 天未更新的過期分支
- 分支比較：一目了然差異提交與修改檔案

### Stash 管理器

完整的 Stash 工作流：push、pop、apply、drop、show diff，全部在 UI 內完成。

### 衝突解決 UI

- 衝突文件清單
- Hunk 級對比（Ours / Theirs / Base）
- 自訂編輯模式
- 一鍵完成合併提交

### 多倉庫標籤管理

同時開啟多個 Git 倉庫，標籤式切換，狀態獨立管理。

### 命令面板

`Ctrl+Shift+P` 叫出命令面板，快速執行 Push / Pull / Fetch / Stash，或動態切換分支，全鍵盤操作。

### 內嵌終端

整合 xterm.js 終端模擬器，支援 Git Bash / PowerShell / Zsh / Fish，直接在 GUI 內敲命令。

### 即時檔案監視

監控 `.git` 目錄，分支切換、推拉、外部 commit 等操作均自動更新 UI，零延遲感知。

### 深色 / 淺色主題

跟隨系統或手動切換，CSS Design Tokens 驅動，即時生效。

---

## 功能一覽

| 功能 | 狀態 |
|------|------|
| 三欄工作區（檔案樹 / Diff / 提交圖） | ✅ |
| Hunk 級 Stage / Unstage / Stash | ✅ |
| AI 提交訊息（Gemini / OpenAI / Ollama） | ✅ |
| 提交歷史多分支視覺化圖 | ✅ |
| 全文提交搜尋（訊息 / 作者 / Diff 內容） | ✅ |
| 分支建立 / 切換 / 刪除 / 重命名 | ✅ |
| 分支比較（ahead/behind + diff） | ✅ |
| Stash 完整管理 | ✅ |
| 衝突解決 UI | ✅ |
| 多倉庫標籤管理 | ✅ |
| Clone 含進度條 | ✅ |
| Push / Pull / Fetch / Push Tags | ✅ |
| Revert / Reset (soft/hard) / Cherry-pick | ✅ |
| Tag 建立 / 刪除 / 推送 | ✅ |
| 遠端管理 | ✅ |
| 命令面板（Ctrl+Shift+P） | ✅ |
| 內嵌終端（xterm.js） | ✅ |
| 即時 .git 目錄監視 | ✅ |
| 深色 / 淺色 / 系統主題 | ✅ |
| 自動偵測本機 Shell 與編輯器 | ✅ |
| 系統通知（Toast + 原生通知） | ✅ |
| WSL2 支援 | 規劃中 |
| macOS 支援 | 規劃中 |
| Plugin 系統 | 規劃中 |

---

## 技術架構

```
┌──────────────────────────────────────────────────┐
│  UI — Svelte 5 (Runes) + TypeScript               │
│  FileTree │ DiffViewer │ CommitLog │ Terminal     │
├──────────────────────────────────────────────────┤
│  Tauri IPC (invoke / events)                      │
├──────────────────────────────────────────────────┤
│  Core — Rust                                      │
│  git2-rs (reads)  │  subprocess git (writes)      │
│  AI HTTP Client   │  FS Watcher (notify)          │
└──────────────────────────────────────────────────┘
```

### 技術棧

| 層級 | 技術 |
|------|------|
| 桌面框架 | Tauri 2.x |
| 前端 | SvelteKit + TypeScript |
| 狀態管理 | Svelte 5 Runes ($state / $derived / $effect) |
| 後端 | Rust |
| Git 讀取 | git2-rs |
| Git 寫入 | subprocess git 2.30+ |
| AI 整合 | Gemini / OpenAI / Ollama（async HTTP） |
| 終端 | xterm.js |
| 設定持久化 | tauri-plugin-store |
| 檔案監視 | notify-debouncer-full |
| 日誌 | tracing crate |

---

## 平台支援

| 平台 | 狀態 |
|------|------|
| Windows 11 | ✅ v0.3.0 |
| Ubuntu 24.04+ | ✅ v0.3.0 |
| Fedora 40+ | ✅ v0.3.0 |
| macOS | 規劃中 |

---

## 開始使用

### 環境需求

- [Rust](https://rustup.rs/) 1.75+
- Node.js 20+
- Git 2.30+
- （AI 功能）Gemini / OpenAI API Key，或本機 Ollama

### 安裝與開發

```bash
# 安裝依賴
npm install

# 開發模式
npm run tauri:dev

# 正式建置
npm run tauri:build
```

### AI 設定

1. 開啟設定（`Ctrl+,`）
2. 選擇 AI 提供者（Gemini / OpenAI / Ollama）
3. 輸入 API Key
4. 選擇模型與語言偏好
5. 在提交表單點擊「✨ AI 生成」即可

---

## 授權

Friendly Non-Commercial License (with Approval for Commercial Use) v1.1

非商業使用免費，商業使用需取得作者書面同意。詳見 [LICENSE](./LICENSE)。

Copyright (c) 2026 [osakihsieh](https://github.com/osakihsieh)
