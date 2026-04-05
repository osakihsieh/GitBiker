# US-010: Pre-commit Hook 支援

## User Story

**作為** 一個使用 pre-commit hooks（lint、format、test）的開發者，
**我想要** GitBiker 在 commit 時正確執行 hooks 並顯示結果，
**以便** 在 GUI 中也能享受 hooks 帶來的品質保障。

## 驗收條件

1. Commit 時正確觸發 `.git/hooks/pre-commit`（以及 commit-msg 等 hooks）
2. Hook 執行成功：正常完成 commit
3. Hook 執行失敗：
   - 阻止 commit
   - 在 Toast 或專用面板顯示 hook 的 stderr/stdout 輸出
   - 錯誤訊息可展開查看完整 log
4. 支援 husky、lint-staged 等常見 hook 工具
5. Hook 執行時顯示 loading 狀態（「正在執行 pre-commit hook...」）

## 技術備註

- 目前 commit 使用 subprocess `git commit`，理論上已會觸發 hooks
- 需確認 Tauri 子行程環境變數是否正確（PATH、NODE_PATH 等）
- 可能需要設定 `GIT_EXEC_PATH` 確保 hook 能找到必要工具
- 主要工作在於：正確捕獲 hook 輸出並顯示給使用者
- Rust 端修改 `git_commit` 回傳結構，包含 hook 輸出資訊

## 優先級：P3 | 工作量：S（小）
## 依賴：Commit 功能（已完成）
