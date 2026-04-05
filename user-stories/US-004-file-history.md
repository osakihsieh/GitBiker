# US-004: File History — 單一檔案歷史

## User Story

**作為** 一個想了解某個檔案變更紀錄的開發者，
**我想要** 查看任意檔案的完整 commit 歷史和每次變更的 diff，
**以便** 追蹤是誰、在什麼時候、為什麼修改了這個檔案。

## 驗收條件

1. FileTree 的檔案右鍵選單新增「查看檔案歷史」
2. CommitLog commit 展開的檔案列表也可右鍵「查看此檔案歷史」
3. 開啟 File History 面板，顯示該檔案相關的所有 commits
4. 每個 commit 顯示：hash、message、author、timestamp
5. 點擊某個 commit 可在 DiffViewer 中查看該 commit 對此檔案的 diff
6. 支援搜尋/篩選 commit message

## 技術備註

- Rust 端新增 `git_file_log` command：`git log --follow -- <file_path>`
- `--follow` 可追蹤檔案重新命名
- 新增 `git_show_file_at_commit` command 取得特定版本的檔案內容
- 前端：可複用 CommitLog 元件，傳入 file filter 模式
- 考慮用右側面板的切換 view 模式實現

## 優先級：P1 | 工作量：M（中）
## 依賴：CommitLog、DiffViewer（已完成）
