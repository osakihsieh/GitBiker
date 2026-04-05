# US-002: Revert Commit — 回退指定 Commit

## User Story

**作為** 一個發現某次 commit 有問題的開發者，
**我想要** 在 CommitLog 右鍵選單中 revert 指定 commit，
**以便** 安全地撤銷該 commit 的變更（產生新的 revert commit）。

## 驗收條件

1. CommitLog 的 commit 右鍵選單新增「Revert this commit」選項
2. 點擊後彈出確認對話框，顯示 commit hash 與 message
3. 確認後執行 `git revert <hash> --no-edit`
4. 成功後自動 refresh FileTree 和 CommitLog
5. 如果 revert 產生衝突，進入衝突解決模式（複用 ConflictResolver）
6. 支援 revert merge commit（自動加 `-m 1`）

## 技術備註

- Rust 端新增 `git_revert` command（subprocess git）
- 偵測 commit.parents 數量判斷是否為 merge commit
- 衝突時複用現有 conflict resolution 流程

## 優先級：P1 | 工作量：M（中）
## 依賴：ConflictResolver（已完成）
