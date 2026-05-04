# GitBiker 極！拜客 — Accelerated AI Factory for Git

>  — 專為 M4 時代打造，基於 Svelte 5 (Runes) 與 Rust (Tauri 2) 的極速 Git AI 工廠。

![version](https://img.shields.io/badge/version-0.4.0--accelerated-orange)
![platform](https://img.shields.io/badge/platform-MacOS%20(M4%20Optimized)%20%7C%20Windows%20%7C%20Linux-brightgreen)
![ui](https://img.shields.io/badge/UI-Glassmorphism-blueviolet)

---

## 為什麼選擇 GitBiker？ (老黃的維度)

當代軟體開發已經進入了「AI 工廠」模式。傳統的 Electron 工具已經跟不上 M4 晶片的並行算力。**GitBiker** 不是一個簡單的 GUI，它是一個加速引擎，旨在抹除開發者與代碼倉庫之間的所有延遲。

| 特性 | GitBiker (M4 加速版) | 傳統工具 (Electron / 舊架構) |
| :--- | :--- | :--- |
| **底層效能** | Rust (git2-rs) + Tauri 2 | Electron (V8) + Node |
| **UI 響應** | Svelte 5 Runes (極致並行) | 傳統組件重繪 (高延遲) |
| **AI 整合** | 原生 AI 工廠 (Review/Fusion/Commit) | 外掛式插件 (ROI 低) |
| **視覺美學** | MacOS 毛玻璃 (Glassmorphism) | 固定框架 UI |
| **並行工作流** | Worktree / Submodule / LFS 原生加速 | 需手動處理，阻塞嚴重 |

---

## 核心特色：加速你的生產線

### 1. AI 智慧工廠 (AI Factory)
- **AI Fusion (智慧融合)**：一鍵解決衝突。AI 自動分析 Base/Ours/Theirs，生成最優化的融合代碼。
- **AI Review (智慧預檢)**：在提交前進行 M4 級別的代碼質量檢查，防範 Bug 於未然。
- **AI Branch Cleanup**：自動識別已合併或過期的分支，並提供清理建議，保持工廠整潔。
- **AI Commit Generator**：分析 Diff 並參考歷史風格，自動生成高品質的提交訊息。

### 2. 極速並行工作流
- **Git Worktree 原生支援**：針對 M4 的並行算力優化，支援物理隔離的並行開發目錄，零等待切換 Context。
- **Git LFS & Submodule 加速**：異步偵測狀態，大型倉庫也能流暢滾動。
- **命令面板 (⌘+Shift+P)**：所有 Git 與 AI 指令都在你的指尖，加速你的操作 ROI。

### 3. MacOS 原生美學 (Glassmorphism)
- **毛玻璃 UI**：深度整合 MacOS 背景模糊效果，讓介面具備通透感與層次感。
- **Monokai Pro 設計系統**：統一的視覺語彙，降低認知負荷，專注於最重要的代碼變更。

---

## 技術架構：全棧並行運算

```
┌──────────────────────────────────────────────────┐
│  UI — Svelte 5 (Runes) + Tailwind CSS v4          │
│  Glassmorphism Layer │ Design Tokens │ KBar       │
├──────────────────────────────────────────────────┤
│  Tauri 2 IPC (Event-Driven Bridge)                │
├──────────────────────────────────────────────────┤
│  Core — Rust (Accelerated Backend)                │
│  git2-rs (Parallel IO) │ Sidecar (CLI Speed)      │
│  AI Engine (Gemini/OpenAI) │ FS Watcher           │
└──────────────────────────────────────────────────┘
```

---

## 平台支援：買越多，省越多

| 平台 | 狀態 | 備註 |
| :--- | :--- | :--- |
| **macOS (Apple Silicon)** | ✅ **v0.4.0 (M4 Optimized)** | **最佳體驗推薦** |
| **Windows 11** | ✅ v0.4.0 | 支援 WSL2 與原生 Git |
| **Linux (Ubuntu/Fedora)** | ✅ v0.4.0 | 支援 Wayland 原生加速 |

---

## 開始安裝：加入加速行列

### 環境需求
- **M4 Mac mini / MacBook (或任意 MacOS/Win/Linux)**
- [Rust](https://rustup.rs/) 1.75+
- Node.js 20+
- Git 2.30+

### 開發與建置
```bash
# 安裝生產依賴
npm install

# 啟動 M4 開發引擎
npm run tauri dev

# 建置正式版本
npm run tauri build
```

---

## 授權

Friendly Non-Commercial License (with Approval for Commercial Use) v1.1

非商業使用免費，商業使用需取得作者書面同意。



Copyright (c) 2026 [osakihsieh](https://github.com/osakihsieh)
