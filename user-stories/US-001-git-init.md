# US-001: Git Init — 初始化本地 Repo

## User Story

**作為** 一個想要開始新專案的開發者，
**我想要** 在 GitBiker 內直接初始化一個新的 git repository，
**以便** 不需要切到終端機手動執行 `git init`。

## 驗收條件

1. Welcome 畫面新增「初始化新 Repo」按鈕（與 Clone / Open 並列）
2. 點擊後彈出資料夾選擇對話框
3. 選擇資料夾後執行 `git init`，成功後自動開啟該 Repo 為新 Tab
4. 如果資料夾已是 git repo，顯示提示「此資料夾已經是 Git repository」
5. 初始化成功後顯示 Toast 通知

## 技術備註

- Rust 端新增 `git_init` command，使用 `git2::Repository::init()`
- 前端 Welcome.svelte 加入第三個 action button
- 複用現有 Tauri dialog plugin 選擇資料夾

## 優先級：P1 | 工作量：S（小）
## 依賴：無
