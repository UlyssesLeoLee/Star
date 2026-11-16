/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/mocks/data/index.ts"})
MERGE (agents:File {path:"frontend/src/mocks/data/agents.ts"})
MERGE (inbox:File {path:"frontend/src/mocks/data/inbox.ts"})
MERGE (analytics:File {path:"frontend/src/mocks/data/analytics.ts"})
MERGE (kanban:File {path:"frontend/src/mocks/data/kanban.ts"})
MERGE (self)-[:IMPORTS]->(agents)
MERGE (self)-[:IMPORTS]->(inbox)
MERGE (self)-[:IMPORTS]->(analytics)
MERGE (self)-[:IMPORTS]->(kanban)
@endcypher
*/
// frontend/src/mocks/data/index.ts
// re-export single source of truth (per mock-data-isolation.md §2.1)
//
// FALLBACK alias 同步导出 (per mock-msw-handlers.md §2.4 + §3.1):
//   - MOCK_*           — MSW handler 用, 返回 fetch 响应
//   - MOCK_*_FALLBACK  — page SSR 阶段兜底 (2026-11-17: agents 已接真实运行时,
//                        MOCK_AGENTS_FALLBACK 删除; 其余模块未清 mock 前保留)

export { MOCK_AGENTS } from "./agents";
export { MOCK_NOTIFS, MOCK_NOTIFS_FALLBACK } from "./inbox";
export { MOCK_KPI, MOCK_KPI_FALLBACK, COST_SERIES, COST_SERIES_FALLBACK } from "./analytics";
export { KANBAN_COLUMNS } from "./kanban";
