# fixtures/ — read-only JSON 备份

⚠️ **本目录是 read-only 人工 review 备份**, 实际代码从 `../data/*.ts` 导入.

## 用途
- 人工 review mock 数据 (diff 友好)
- IDE/编辑器 JSON 语法高亮
- 跨语言导入 (Python/Go 后端 mock 测试)

## 同步规则
- `data/*.ts` 是 source of truth
- 改了 `data/*.ts` 必须同步改 `fixtures/*.json`
- vitest `__tests__/fixtures-sync.test.ts` 自动校验一致性 (改了 data 但忘改 fixture 会 fail)
- 自动化 sync (npm script) 留 Phase E.3+ (per docs/frontend/design/mock-msw-handlers.md §4 缺口 #3)

## 文件清单
- `agents.json` — 5 agent row
- `inbox.json` — 10 notification row
- `analytics-kpi.json` — 4 KPI card
- `analytics-cost-series.json` — 7 day cost series
