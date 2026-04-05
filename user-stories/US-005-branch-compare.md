# US-005: Branch Compare — 分支差異比較

## User Story

**作為** 一個想在 merge 前了解兩個分支差異的開發者，
**我想要** 選擇兩個分支並查看它們之間的差異（commits 和檔案變更），
**以便** 在合併前評估變更範圍和潛在風險。

## 驗收條件

1. BranchManager 新增「比較分支」功能入口
2. 可選擇 base branch 和 compare branch
3. 顯示兩分支之間的差異 commits 列表
4. 顯示檔案差異摘要（新增/修改/刪除的檔案數和行數統計）
5. 點擊任意檔案可查看詳細 diff
6. 支援與當前分支快速比較（只需選擇另一個分支）

## 技術備註

- Rust 端新增 `git_branch_diff` command
  - commits 差異：`git log base..compare --oneline`
  - 檔案差異：`git diff base...compare --stat` + `git diff base...compare -- <file>`
- 前端：新增 BranchCompare 元件，包含兩個分支選擇器 + diff 顯示
- 可複用 DiffViewer 元件顯示檔案差異

## 優先級：P2 | 工作量：M（中）
## 依賴：BranchManager、DiffViewer（已完成）
