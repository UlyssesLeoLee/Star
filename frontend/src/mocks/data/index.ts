// frontend/src/mocks/data/index.ts
// re-export single source of truth (per mock-data-isolation.md §2.1)
//
// FALLBACK alias 同步导出 (per mock-msw-handlers.md §2.4 + §3.1):
//   - MOCK_*           — MSW handler 用, 返回 fetch 响应
//   - MOCK_*_FALLBACK  — page SSR 阶段兜底, 避免 UX 退化 (per §4 #1 缺标)

export { MOCK_AGENTS, MOCK_AGENTS_FALLBACK } from "./agents";
export { MOCK_NOTIFS, MOCK_NOTIFS_FALLBACK } from "./inbox";
export { MOCK_KPI, MOCK_KPI_FALLBACK, COST_SERIES, COST_SERIES_FALLBACK } from "./analytics";
export { KANBAN_COLUMNS } from "./kanban";
