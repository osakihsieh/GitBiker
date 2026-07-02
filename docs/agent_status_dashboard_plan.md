# 被動式 Agent 狀態監聽看板 (Passive Agent Status Dashboard) 設計文件

## 1. 專案概述
本文件旨在規劃「被動式 Agent 狀態監聽看板」的系統設計與實作步驟。該看板的主要目標是實時監控系統中正在運行的各種 Agent 進程，並透過解析日誌檔來呈現它們的最新活動狀態（如：思考中、搜尋中、正在修改程式碼等），提供開發者一個清晰、直覺的全域視角。

## 2. 核心功能需求

### 2.1 日誌監聽 (Log Monitoring)
- **目標路徑**：`~/.hermes/profiles/*/sessions/*.jsonl`
- **行為**：持續監聽上述路徑下的 JSONL 檔案變動，當有新的日誌寫入時，能夠即時擷取最新內容。

### 2.2 狀態解析 (Status Parsing)
- **目標**：從 JSONL 格式的日誌中，解析出 Agent 當前的具體行為。
- **支援狀態包含**（但不限於）：
  - 🧠 **思考中 (Thinking)**：Agent 正在規劃或推論。
  - 🔍 **搜尋中 (Search)**：正在讀取文件或執行 grep/glob 搜尋。
  - 🛠️ **修改中 (Patching/Editing)**：正在寫入或修改檔案。
  - ✅ **閒置/完成 (Idle/Done)**：任務結束等待輸入。

### 2.3 活躍進程偵測 (Active Process Detection)
- **目標**：顯示目前系統中活躍的 Agent 進程（Processes）。
- **行為**：透過系統級的進程掃描（如 `ps` 或是 Node.js 的進程管理套件），辨識出哪些 Agent（例如 Hermes, Kiro, OpenClaw 等）正在運行，並與日誌狀態進行對應。

### 2.4 Svelte 儀表板 UI (Dashboard UI)
- **前端框架**：Svelte / SvelteKit
- **UI 規劃**：
  - **總覽區**：顯示當前活躍的 Agent 數量與系統資源概況。
  - **進程列表區 (Agent List)**：卡片式或列表式呈現每一個 Agent 的 PID、Profile 名稱、Session ID。
  - **即時狀態區 (Real-time Status)**：針對選定的 Agent，顯示其最新的解析動態（配合動畫，如「打字機效果」或「閃爍燈號」提示活躍度）。
  - **日誌預覽區 (Log Tail)**：可選的摺疊面板，顯示原始的 tail 日誌以供進階除錯。

---

## 3. 架構設計

- **後端服務 (Agent Status Daemon)**：
  - 負責執行文件監控（如使用 `chokidar` 套件）。
  - 負責掃描系統 Process。
  - 透過 WebSocket Server 將即時狀態推播給前端。
- **前端介面 (Svelte Dashboard)**：
  - 連接 WebSocket，接收後端推播。
  - 使用響應式變數 (Stores) 自動更新 UI。

---

## 4. 實作步驟 (Implementation Steps)

### Phase 1: 後端基礎設施與日誌監聽 (Backend & Watcher)
1. **初始化專案**：在專案內建立 `backend` 目錄，初始化 Node.js 專案並安裝必要的依賴（如 `chokidar`, `ws`, `fastify` 等）。
2. **實作 File Watcher**：
   - 撰寫腳本監聽 `~/.hermes/profiles/*/sessions/*.jsonl` 的 `change` 與 `add` 事件。
   - 實作讀取最新一行 (Tail) 的功能。
3. **實作 JSONL 解析器**：
   - 將讀取到的 JSON 字串反序列化，提取關鍵字（如 tool_calls, content 等），轉換為易讀的狀態（Search, Patch, Thinking）。

### Phase 2: 活躍進程追蹤與 WebSocket (Process & WS)
1. **實作進程偵測**：
   - 撰寫系統指令包裝（如 `ps aux | grep hermes`）或使用 `find-process` 套件來列出活躍的 Agent PID。
2. **建立 WebSocket Server**：
   - 將 File Watcher 解析的狀態與進程清單，封裝成統一的 Event Payload（如 `{ type: 'AGENT_UPDATE', data: {...} }`）。
   - 廣播給所有連線的 Client 端。

### Phase 3: Svelte 前端儀表板 (Frontend Dashboard)
1. **初始化 Svelte 專案**：使用 Vite 建立 Svelte 專案 (`npm create vite@latest frontend -- --template svelte`)。
2. **設計共用元件 (Components)**：
   - `AgentCard.svelte`：顯示單一 Agent 狀態、圖示及正在執行的動作。
   - `StatusBadge.svelte`：動態顏色標籤（如綠色代表 Patching，黃色代表 Thinking）。
3. **狀態管理 (Store)**：
   - 建立 `store.js`，管理 WebSocket 連線及統一儲存 Agent 列表狀態。
4. **介面整合**：
   - 將總覽、列表及即時狀態區塊整合至主頁面 `App.svelte`。

### Phase 4: 測試與優化 (Testing & Polish)
1. **模擬測試**：撰寫一個 mock 腳本，隨機向 `~/.hermes/...` 寫入假 JSONL 日誌，驗證前端更新延遲與正確性。
2. **UI 優化**：加入平滑的動畫過渡（Svelte transitions）、處理 WebSocket 斷線重連邏輯。
3. **打包與部署**：將前端編譯為靜態檔案，並可選擇由後端框架統一 serve，達成單一執行檔或簡單的啟動腳本。