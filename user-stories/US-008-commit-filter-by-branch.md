# US-008: Commit Filter by Branch — 按分支篩選 Commit

## User Story

**作為** 一個想要查看特定分支 commit 紀錄的開發者，
**我想要** 在 CommitLog 中篩選只顯示某個分支的 commits，
**以便** 專注於特定分支的變更歷史，而不被其他分支的 commits 干擾。

## 驗收條件

1. CommitLog 頂部新增分支篩選下拉選單
2. 預設顯示「當前分支」的 commits
3. 可選擇其他本地或遠端分支
4. 選擇後只顯示該分支上的 commits（不含其他分支的 commits）
5. 可選擇「所有分支」模式（`git log --all`）
6. 篩選與現有搜尋可疊加使用

## 技術備註

- Rust 端修改現有 `git_log` command，新增 `branch` 參數
  - 指定分支：`git log <branch> --`
  - 所有分支：加 `--all` flag
- 前端：CommitLog 元件頂部加入分支選擇器
  - 複用 BranchManager 的分支列表資料
  - 切換分支時重新 fetch log

## 優先級：P2 | 工作量：S（小）
## 依賴：CommitLog、Branch 列表（已完成）
