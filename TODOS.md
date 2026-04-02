# TODOs

## 測試基礎設施

**What:** 建立 vitest + Svelte testing library，為現有功能寫第一批 unit tests
**Why:** 目前 0 測試覆蓋率。theme 功能有 24 條需要測試的路徑。
**Priority:** P2
**Effort:** S (CC+gstack ~15min)
**Depends on:** 無

## Popover drag-to-reorder pinned repos

**What:** 在 Popover 裡拖曳重新排序 pinned repos
**Why:** 用戶想控制 pin 的順序，但在 Svelte 5 popover 裡做拖曳複雜度高（拖曳庫選型、touch 支援、scroll 衝突、accessibility）。Phase 1 用 unpin + re-pin 替代。
**Priority:** P2
**Effort:** M (CC+gstack ~30min)
**Depends on:** Phase 1 Popover + Pin 功能完成
