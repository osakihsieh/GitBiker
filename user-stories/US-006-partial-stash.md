# US-006: Partial Stash — 按檔案/行 Stash

## User Story

**作為** 一個只想暫存部分變更的開發者，
**我想要** 選擇特定檔案或程式碼行進行 stash，
**以便** 保留正在進行的工作，只 stash 需要暫時擱置的部分。

## 驗收條件

1. StashManager 的「Stash All」旁新增「Stash Selected」按鈕
2. FileTree 的檔案可勾選要 stash 的檔案（多選）
3. 選擇後執行 `git stash push -- <file1> <file2> ...`
4. 進階模式：DiffViewer 中可選擇特定 hunk 進行 stash
5. Stash 時可輸入自訂 message
6. Un-stash 時也支援選擇性 apply（按檔案）

## 技術備註

- 按檔案 stash：`git stash push -m "message" -- file1.rs file2.ts`
- 按 hunk stash：需要 `git stash push -p`（interactive，較複雜）
  - 替代方案：手動 stage 要保留的、stash 剩餘的
  - Phase 1 先做按檔案，Phase 2 考慮按 hunk
- Rust 端新增 `git_stash_push_files` command
- 選擇性 un-stash：`git checkout stash@{n} -- <file>`

## 優先級：P2 | 工作量：M（中）
## 依賴：StashManager（已完成）
