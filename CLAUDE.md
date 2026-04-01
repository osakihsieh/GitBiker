## 基本規範

- 說中文
- git commit 用中文填寫，標題前綴（feat: fix: 等）保持英文
- 計畫時需要確認的地方跳 prompt 跟用戶溝通
- 設計保持一致風格，代碼風格一致
- 遵循 clean code & clean architecture 原則，不要重複相同代碼

### 快捷指令

使用者說以下關鍵字時，主 Claude 自動派發對應 agent，不需額外確認：

| 使用者輸入 | 自動派發           | 模式                      |
| ---------- | ------------------ | ------------------------- |
| 站立會議   | scrum-master-agent | Mode A：Standup           |
| 規劃下一輪 | scrum-master-agent | Mode B：Planning          |
| 分析阻塞   | scrum-master-agent | Mode C：Blocker Detection |
| 跑回顧     | scrum-master-agent | Mode E：Retro             |
| review     | review-agent       | 審查最近 dev-agent 的修改 |
| AI 評估    | ai-eval-agent      | Mode A：例行評估          |
| 合規審查   | accountant-agent   | Mode B：全面掃描          |
