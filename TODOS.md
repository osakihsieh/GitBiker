# TODOs

## (已完成) 測試基礎設施 ✅
## (已完成) git_commands.rs 拆分 ✅
## (已完成) Merge Branch Into Current ✅
## (已完成) Stash 操作 ✅
## (已完成) Command Palette ✅
## (已完成) Inline Terminal ✅
## (已完成) Popover drag-to-reorder pinned repos ✅

---

## Conflict Resolution UI

**What:** 完整的 merge conflict 解決介面 — 顯示衝突檔案清單、inline 3-way diff、標記已解決、commit merge
**Why:** 目前 merge 衝突只顯示清單和 abort，用戶必須切到外部編輯器解決。
**Priority:** P2
**Effort:** L (需要 3-way diff 解析和 inline conflict marker 編輯器)
**Depends on:** Merge Branch 完成

## Interactive Rebase

**What:** 在 CommitLog 上選取 commits 做 rebase（reorder, squash, drop）
**Why:** 進階 git 工作流，但複雜度高。
**Priority:** P3
**Effort:** L
**Depends on:** Commit History 穩定

## Full PTY Terminal

**What:** 升級 Inline Terminal 為完整 PTY 終端機（支援所有命令、autocomplete、色彩）
**Why:** 目前只支援 git 命令，完整 PTY 需要 portable-pty 或 conpty 整合。
**Priority:** P3
**Effort:** L (需要 Rust PTY crate + Tauri IPC 串流)
**Depends on:** Inline Terminal 穩定
