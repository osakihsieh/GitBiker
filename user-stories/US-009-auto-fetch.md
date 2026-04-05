# US-009: Auto Fetch — 定時自動 Fetch

## User Story

**作為** 一個經常需要保持 remote 同步的開發者，
**我想要** GitBiker 自動定期 fetch remote 更新，
**以便** 隨時看到最新的遠端分支狀態和 ahead/behind 計數。

## 驗收條件

1. Settings 新增「Auto Fetch」開關（預設開啟）
2. 可設定 fetch 間隔：1分鐘 / 5分鐘 / 10分鐘 / 30分鐘
3. 開啟後，每隔指定時間自動執行 `git fetch --all`
4. Fetch 完成後自動更新 Toolbar 的 ahead/behind 計數
5. 自動 fetch 不顯示 loading spinner（靜默執行）
6. 如果 fetch 失敗（離線），靜默忽略，下次再試
7. 手動 fetch 會重置計時器

## 技術備註

- 前端用 `setInterval` 管理定時器
- 使用現有的 `git_fetch` command
- 每個 Tab 獨立計時（或全域統一 fetch 所有已開啟 repo）
- 持久化設定到 Tauri store
- Tab 關閉時清除對應計時器

## 優先級：P3 | 工作量：S（小）
## 依賴：Fetch 功能、Settings（已完成）
