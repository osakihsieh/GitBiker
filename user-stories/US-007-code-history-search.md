# US-007: Code History Search — 程式碼歷史搜尋

## User Story

**作為** 一個想知道某段程式碼是何時被加入或移除的開發者，
**我想要** 搜尋 commit 歷史中特定程式碼片段的變更紀錄，
**以便** 追蹤 bug 的引入點或了解某個功能的演進。

## 驗收條件

1. CommitLog 搜尋欄新增搜尋模式切換：「Message」/「Author」/「Code」
2. Code 模式下輸入程式碼片段，搜尋新增或移除該片段的 commits
3. 搜尋結果顯示 commit 列表，標記是「新增」還是「移除」了搜尋片段
4. 點擊 commit 可查看對應 diff，搜尋片段高亮顯示
5. 支援正則表達式搜尋

## 技術備註

- Rust 端新增 `git_log_search_code` command
  - 使用 `git log -S "keyword"` (pickaxe search) 搜尋新增/移除
  - 使用 `git log -G "regex"` 搜尋正則匹配
- 搜尋結果可能較慢（需掃描所有 commit 的 diff），需要：
  - 加 loading 指示器
  - 限制搜尋範圍（最近 N 個 commits 或日期範圍）
  - 支援取消搜尋

## 優先級：P2 | 工作量：M（中）
## 依賴：CommitLog 搜尋功能（已完成）
