# TODOs

## (已完成) 測試基礎設施 ✅
## (已完成) git_commands.rs 拆分 ✅
## (已完成) Merge Branch Into Current ✅
## (已完成) Stash 操作 ✅
## (已完成) Command Palette ✅
## (已完成) Inline Terminal ✅
## (已完成) Popover drag-to-reorder pinned repos ✅

---

## (已完成) Conflict Resolution UI ✅

---

## GitFiend 對標 — 實作路線圖

### Sprint 1：核心 Git 操作補齊（P1）

| # | User Story | 工作量 | 依賴 |
|---|-----------|--------|------|
| US-001 | [Git Init](user-stories/US-001-git-init.md) | S | 無 ✅ |
| US-002 | [Revert Commit](user-stories/US-002-revert-commit.md) | M | ConflictResolver ✅ |
| US-003 | [Undo Un-pushed Commits](user-stories/US-003-undo-unpushed-commits.md) | M | CommitLog ✅ |
| US-004 | [File History](user-stories/US-004-file-history.md) | M | CommitLog + DiffViewer ✅ |

### Sprint 2：進階使用者體驗（P2）

| # | User Story | 工作量 | 依賴 |
|---|-----------|--------|------|
| US-008 | [Commit Filter by Branch](user-stories/US-008-commit-filter-by-branch.md) | S | CommitLog ✅ ✅ |
| US-005 | [Branch Compare](user-stories/US-005-branch-compare.md) | M | BranchManager ✅ |
| US-006 | [Partial Stash](user-stories/US-006-partial-stash.md) | M | StashManager ✅ |
| US-007 | [Code History Search](user-stories/US-007-code-history-search.md) | M | CommitLog ✅ |

### Sprint 3：品質提升（P3）

| # | User Story | 工作量 | 依賴 |
|---|-----------|--------|------|
| US-009 | [Auto Fetch](user-stories/US-009-auto-fetch.md) | S | Fetch + Settings ✅ |
| US-010 | [Pre-commit Hook 支援](user-stories/US-010-pre-commit-hook-support.md) | S | Commit ✅ |

---

## 既有待辦

## Conflict Resolution UI (CEO Plan Ready — 已完成)

**What:** 完整的 merge conflict 解決介面 — inline accept ours/theirs/both per hunk、衝突檔案清單、進度追蹤、完成 merge commit
**Why:** 目前 merge 衝突只顯示清單和 abort，用戶必須切到外部編輯器解決。
**Priority:** P1 (升級 — CEO Plan 已完成)
**Effort:** M (with CC+gstack: ~30-60 min)
**Depends on:** Merge Branch 完成 ✅
**CEO Plan:** `~/.gstack/projects/osakihsieh-GitBiker/ceo-plans/2026-04-03-merge-conflict-resolution.md`
**Scope:** Approach B (VS Code style) + 5 擴展：dry-run preview、進度條+自動跳轉、Toolbar badge、Pull 衝突整合、FileTree 衝突標記
**Key decisions:** conflict state on RepoState (per-tab)、整檔案一次寫入（非逐 hunk）、Git >= 2.38 用 merge-tree dry-run / < 2.38 跳過

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
