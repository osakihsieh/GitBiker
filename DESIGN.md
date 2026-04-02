# GitBiker Design System

## 視覺方向：極簡主義

Sublime Text 風格。最少的 chrome，最大的內容空間。無裝飾元素。
專業開發者工具，不是消費級 app。密集資訊、緊湊間距、monospace 程式碼。

---

## Design Tokens (CSS Custom Properties)

### 色彩 — Dark Theme (預設)

```css
:root {
  /* 背景 */
  --bg-primary:    #1a1a2e;
  --bg-secondary:  #16162a;
  --bg-surface:    #232340;
  --bg-hover:      #2a2a4a;

  /* 文字 */
  --text-primary:  #e0e0e0;
  --text-secondary:#888888;
  --text-muted:    #555555;

  /* 強調色 */
  --accent:        #4fc1ff;
  --error:         #ff6b6b;
  --success:       #51cf66;
  --warning:       #ffd43b;

  /* Diff */
  --diff-add-bg:   rgba(46, 160, 67, 0.44);
  --diff-del-bg:   rgba(248, 81, 73, 0.44);
  --diff-add-text: #7ee787;
  --diff-del-text: #ffa198;

  /* 邊框 */
  --border:        #2d2d50;
}
```

### 色彩 — Light Theme

```css
[data-theme="light"] {
  --bg-primary:    #fafafa;
  --bg-secondary:  #f0f0f0;
  --bg-surface:    #ffffff;
  --bg-hover:      #e8e8e8;

  --text-primary:  #1a1a1a;
  --text-secondary:#666666;
  --text-muted:    #999999;

  --accent:        #0066cc;
  --error:         #d32f2f;
  --success:       #2e7d32;
  --warning:       #f57f17;

  --diff-add-bg:   rgba(46, 160, 67, 0.15);
  --diff-del-bg:   rgba(248, 81, 73, 0.15);
  --diff-add-text: #1a7f37;
  --diff-del-text: #cf222e;

  --border:        #d0d0d0;
}
```

### 字體

```css
:root {
  --font-ui:   system-ui, -apple-system, sans-serif;
  --font-mono: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', monospace;

  --font-size-sm: 12px;
  --font-size-md: 13px;
  --font-size-lg: 14px;
}
```

- UI 文字：`--font-ui`，`--font-size-md` (13px)
- 程式碼 / diff / 檔案路徑：`--font-mono`，`--font-size-md` (13px)
- Section headers：`--font-ui`，11px，uppercase，letter-spacing 0.5px
- Commit messages：`--font-ui`，12px
- Commit hash / metadata：`--font-mono`，10-11px

### 間距

```css
:root {
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 12px;
  --space-lg: 16px;
}
```

### 圓角

```css
:root {
  --radius-sm: 2px;  /* 按鈕、輸入框、checkbox */
  --radius-md: 4px;  /* dropdown、toast、dialog */
}
```

面板邊界用 `1px solid var(--border)` 分隔線，不用圓角。

---

## 佈局

### 主工作區 — 三欄式

```
┌─────────────────────────────────────────────────────────────────┐
│ Toolbar (40px)                                                   │
│ [repo名] [branch▼] [fetch] [pull] [push]              [⚙ 設定] │
├──────────┬────────────────────────────┬──────────────────────────┤
│ Sidebar  │     Center: Diff Viewer    │  Right: Commit History   │
│ 240px    │     flex (剩餘空間)         │  320px fixed             │
│          │                            │                          │
│ Staged   │  line numbers (40px)       │  Commit graph +          │
│ ☑ files  │  monospace 13px            │  message + author        │
│ ─────── │  line-height 1.6           │  12px text, 40px/row     │
│ Unstaged │  +/- color coding          │                          │
│ ☐ files  │                            │                          │
│ ─────── │                            │                          │
│ Commit   │                            │                          │
│ [msg]    │                            │                          │
│ [送出]   │                            │                          │
└──────────┴────────────────────────────┴──────────────────────────┘
```

- Sidebar 和 Right panel 可拖拽調整寬度 (min 180px, max 400px)
- Center 面板吃剩餘空間 (min 300px)
- 最小視窗：900 x 600px
- 視窗 < 900px 寬：Right panel 收合為 tab

### Welcome 頁（無 repo 時）

- 居中 gitbiker SVG logo
- 兩個主要 CTA：「Clone a Repo」「Open Local Repo」
- 下方「Recent Repos」列表
- 首次啟動時列表為空，顯示引導文字

### Settings 頁

獨立頁面，取代三欄佈局。左側導航 + 右側內容。

---

## 元件規格

### Toolbar (40px 高)

- 背景：`--bg-secondary`
- 底部邊框：`1px solid var(--border)`
- 內容：repo 名稱 (14px bold) → branch selector → 間距 → fetch/pull/push 按鈕 → settings

### 按鈕

- 背景：`--bg-surface`
- 邊框：`1px solid var(--border)`
- 圓角：`--radius-sm`
- Padding：`4px 8px`
- 字體：12px，`--font-ui`
- Hover：背景 `--bg-hover`，文字 `--text-primary`

### Commit 按鈕（主要動作）

- 背景：`--accent`
- 文字：`--bg-primary`（深色在亮色上）
- Bold 600
- 無邊框
- Hover：`filter: brightness(1.1)`

### 輸入框 / Textarea

- 背景：`--bg-surface`
- 邊框：`1px solid var(--border)`
- 圓角：`--radius-sm`
- Padding：`8px`
- 字體：12px，`--font-ui`
- Focus：邊框變 `--accent`
- Placeholder：`--text-muted`

### File Tree Item

- 單擊 = 顯示 diff
- Checkbox 做 stage/unstage（每檔前有 ☑/☐）
- Active item：背景 `--bg-surface`，左邊框 `2px solid var(--accent)`
- Hover：背景 `--bg-hover`
- 字體：12px，`--font-mono`
- 狀態標記：M = `--warning`，A = `--success`，D = `--error`

### Commit History Item

- 40px 行高
- 左側 commit graph（dot 8px + line 2px）
- Branch tag：`--accent` 背景，`--bg-primary` 文字，10px bold
- Remote tag：`--bg-surface` 背景 + border
- Commit message：12px
- Metadata（hash + author + time）：10-11px，`--text-muted`

### Toast 通知

- 位置：固定右上角，距頂 52px（toolbar 下方）
- 圓角：`--radius-md`
- 成功：綠色背景 `#1a3d2a`，文字 `--success`，2s 自動消失
- 錯誤：紅色背景 `#3d1a1a`，文字 `--error`
  - 一般錯誤：5s 自動消失
  - 重大錯誤（auth、conflict）：不自動消失，需手動關閉
- 動畫：fade in + slide down 0.2s

### Inline 錯誤

- 紅色文字 `--error`，12px，顯示在輸入框下方
- 用於：clone URL 驗證、commit message 為空

---

## 互動狀態

| Feature | Loading | Empty | Error | Success | Partial |
|---------|---------|-------|-------|---------|---------|
| File tree | skeleton | "No changes" + icon | toast | 顯示列表 | virtual scroll 1000+ |
| Diff viewer | skeleton | "Select a file" | inline 紅色 | 顯示 diff | >10MB 截斷提示 |
| Commit log | skeleton | "No commits yet" | toast | 顯示歷史 | 分頁載入 |
| Commit form | 按鈕禁用+spinner | N/A | inline (空訊息) | 綠色 toast 2s | n/m files staged |
| Branch switcher | dropdown 載入中 | "No branches" | toast (index.lock) | toast 1s | merge 進行中標記 |
| Clone | 進度條 %+階段 | N/A | inline (URL/網路) | 自動開啟 repo | 進度 % |
| Push/Pull | toolbar spinner | N/A | toast 5s (auth) | 綠色 toast 2s | conflict 列表 |

---

## 鍵盤快捷鍵 (Phase 1)

| 快捷鍵 | 動作 |
|--------|------|
| Ctrl+Enter | Commit |
| Ctrl+Shift+P | 命令面板 |
| Ctrl+1 / 2 / 3 | 切換面板焦點 |
| Tab | 面板間導航 |
| ↑↓ | 檔案列表 / commit 歷史導航 |

---

## 無障礙

- WCAG AA 色彩對比度（4.5:1 最低）
- 焦點指示器：`2px solid var(--accent)` outline，offset 2px
- 所有互動元素可 Tab 到達
- Screen reader：ARIA landmarks（nav、main、aside）

---

## Mockups

| 畫面 | 路徑 |
|------|------|
| 主工作區 | ~/.gstack/projects/gitbiker/designs/mockups-20260402/main-workspace.html |
| Welcome 頁 | ~/.gstack/projects/gitbiker/designs/mockups-20260402/welcome-page.html |
