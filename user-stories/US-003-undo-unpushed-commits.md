# US-003: Undo Un-pushed Commits — 撤銷未推送的 Commit

## User Story

**作為** 一個在本地 commit 了但尚未 push 的開發者，
**我想要** 快速撤銷最近的 commit（保留變更到 working tree），
**以便** 重新整理 commit 內容再推送。

## 驗收條件

1. CommitLog 的未推送 commit 右鍵選單新增「Undo commit（保留變更）」
2. 僅對尚未推送的 commit 顯示此選項（ahead > 0 的 commits）
3. 點擊後彈出確認對話框，說明此操作會 soft reset
4. 確認後執行 `git reset --soft HEAD~1`（撤銷單個）
5. 支援撤銷到指定 commit：`git reset --soft <hash>`（撤銷多個）
6. 成功後 refresh FileTree（變更回到 staged 狀態）和 CommitLog
7. 提供「Undo commit（丟棄變更）」選項，使用 `git reset --hard`，需額外警告

## 技術備註

- Rust 端新增 `git_reset_soft` 和 `git_reset_hard` commands
- 前端需標記哪些 commit 是 unpushed（已有 ahead 計數，需要精確到 commit 級別）
- Hard reset 需要二次確認「此操作不可撤銷」

## 優先級：P1 | 工作量：M（中）
## 依賴：CommitLog ahead/behind 資訊（已完成）
