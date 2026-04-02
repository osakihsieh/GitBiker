# TODOs

## 測試基礎設施

**What:** 建立 vitest + Svelte testing library，為現有功能寫第一批 unit tests
**Why:** 目前 0 測試覆蓋率。9 項新功能沒有測試很危險，Batch 0 拆分是寫測試的最佳時機。
**Priority:** P1
**Effort:** S (CC+gstack ~15min)
**Depends on:** 無（建議跟 Batch 0 app.svelte.ts 拆分一起做）

## Popover drag-to-reorder pinned repos

**What:** 在 Popover 裡拖曳重新排序 pinned repos
**Why:** 用戶想控制 pin 的順序，但在 Svelte 5 popover 裡做拖曳複雜度高（拖曳庫選型、touch 支援、scroll 衝突、accessibility）。Phase 1 用 unpin + re-pin 替代。
**Priority:** P2
**Effort:** M (CC+gstack ~30min)
**Depends on:** Phase 1 Popover + Pin 功能完成

## Stash 操作

**What:** Stash All + Pop Stash + stash list UI
**Why:** 常用 git 操作，用戶現在得去 terminal 才能 stash。需要設計 stash list 管理介面（顯示 stash 列表、apply、drop、pop）。
**Priority:** P2
**Effort:** M (CC+gstack ~30min)
**Depends on:** File context menu (Batch 2) 完成後

## Inline Terminal

**What:** 在 app 內嵌入終端機面板（類似 VS Code 底部終端機）
**Why:** 用戶不需切換到外部終端機即可執行 git 指令或其他 CLI 操作，提升工作流連貫性。
**Priority:** P2
**Effort:** L (需要 xterm.js + Tauri shell plugin 整合)
**Depends on:** 基礎 UI 架構穩定後

## Merge Branch Into Current

**What:** 在 BranchManager 面板裡選一個 branch，點「Merge into {current}」合併。合併前顯示 ahead 計數，衝突時顯示 conflicted files。
**Why:** Git GUI 的重要功能，完成後 branch 工作流才算完整。需要專門設計 conflict resolution UI（顯示衝突檔案清單、標記已解決、abort merge）。
**Priority:** P2
**Effort:** L (human) → M (CC+gstack ~30min)
**Depends on:** Branch Management Phase 1 完成

## git_commands.rs 拆分

**What:** 將 git_commands.rs（~800 行）拆分為 `branch_commands.rs`、`remote_commands.rs`、`file_commands.rs` 等模組
**Why:** 可維護性。目前所有 git 命令都在一個檔案裡，接近 800 行上限。
**Priority:** P3
**Effort:** S (CC+gstack ~15min)
**Depends on:** Branch Management 完成後

## Command Palette

**What:** VS Code 風格 command palette (Ctrl+Shift+P)
**Why:** 隨著功能增多，快速存取所有功能而不用記快捷鍵。Settings 頁已提到此快捷鍵但未實作。
**Priority:** P2
**Effort:** M (CC+gstack ~30min)
**Depends on:** Phase 2 功能全部完成後（palette 需要列出所有可用命令）
