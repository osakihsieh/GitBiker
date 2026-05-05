# GitBiker 極！拜客 — Modern Git Client with AI Power

> 基於 Svelte 5 (Runes) 與 Rust (Tauri 2) 打造的現代化極速 Git 客戶端。

![version](https://img.shields.io/badge/version-0.3.1-orange)
![platform](https://img.shields.io/badge/platform-MacOS%20%7C%20Windows%20%7C%20Linux-brightgreen)
![ui](https://img.shields.io/badge/UI-Glassmorphism-blueviolet)

---

## 為什麼選擇 GitBiker？

當代軟體開發節奏飛快。**GitBiker** 不僅僅是一個 Git GUI，它更是一個開發者助手，旨在消除開發者與程式碼倉庫之間的所有摩擦。

| 特性 | GitBiker | 傳統工具 (Electron / 舊架構) |
| :--- | :--- | :--- |
| **底層效能** | Rust (git2-rs) + Tauri 2 | Electron (V8) + Node |
| **UI 響應** | Svelte 5 Runes (高效能渲染) | 傳統組件重繪 (高延遲) |
| **AI 整合** | 原生 AI 輔助 (Review/Fusion/Commit) | 外掛式插件 |
| **視覺美學** | macOS 原生毛玻璃 (Glassmorphism) | 固定框架 UI |
| **工作流** | Worktree / Submodule / LFS 原生支援 | 需手動處理，流程繁瑣 |

---

## 核心特色

### 1. AI 輔助功能
- **AI Fusion (智慧融合)**：一鍵解決衝突。AI 自動分析 Base/Ours/Theirs，生成最優化的融合代碼。
- **AI Review (智慧預檢)**：在提交前進行代碼質量檢查，防範潛在問題。
- **AI Branch Cleanup**：自動識別已合併或過期的分支，提供清理建議。
- **AI Commit Generator**：分析 Diff 並參考歷史風格，自動生成高品質的提交訊息。

### 2. 極速工作流
- **Git Worktree 原生支援**：支援物理隔離的並行開發目錄，零等待切換開發上下文。
- **Git LFS & Submodule 支援**：異步偵測狀態，大型倉庫也能流暢操作。
- **命令面板 (⌘+Shift+P)**：所有 Git 與 AI 指令都在指尖，快速執行各種操作。

### 3. 原生設計美學
- **毛玻璃 UI**：深度整合系統背景模糊效果，介面具備現代層次感。
- **Monokai Pro 設計系統**：精美、統一的視覺語彙，降低認知負荷。

---

## 技術架構

```
┌──────────────────────────────────────────────────┐
│  UI — Svelte 5 (Runes) + Tailwind CSS v4          │
│  Glassmorphism Layer │ Design Tokens │ KBar       │
├──────────────────────────────────────────────────┤
│  Tauri 2 IPC (Event-Driven Bridge)                │
├──────────────────────────────────────────────────┤
│  Core — Rust Backend                              │
│  git2-rs (Parallel IO) │ Sidecar (CLI Speed)      │
│  AI Engine (Gemini/OpenAI) │ FS Watcher           │
└──────────────────────────────────────────────────┘
```

---

## 平台支援

| 平台 | 狀態 | 備註 |
| :--- | :--- | :--- |
| **macOS (Apple Silicon)** | ✅ | **原生系統適配，包含快捷鍵優化** |
| **Windows 11** | ✅ | 支援 WSL2 與原生 Git |
| **Linux (Ubuntu/Fedora)** | ✅ | 支援 Wayland |

---

## 開始安裝

### 環境需求
- [Rust](https://rustup.rs/) 1.75+
- Node.js 20+
- Git 2.30+

### 開發與建置
```bash
# 安裝依賴
npm install

# 啟動開發模式
npm run tauri dev

# 建置正式版本
npm run tauri build
```

---

## 授權

Friendly Non-Commercial License (with Approval for Commercial Use) v1.1

非商業使用免費，商業使用需取得作者書面同意。

---

Copyright (c) 2026 [osakihsieh](https://github.com/osakihsieh)
