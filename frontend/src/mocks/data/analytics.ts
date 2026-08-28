// frontend/src/mocks/data/analytics.ts
// MOCK_KPI + COST_SERIES (per mock-data-isolation.md §2.1)
// Source: previously inline in frontend/src/app/(app)/analytics/page.tsx

import type { KpiCard, CostPoint } from "@/mocks/schemas/analytics";

export const MOCK_KPI: ReadonlyArray<KpiCard> = [
  { label: "Cost (24h)",   value: "$12.48", hint: "mock aggregated",                tone: "warn" },
  { label: "Tokens (24h)", value: "1.24M",  hint: "input 0.78M / output 0.46M",     tone: "info" },
  { label: "Tasks (24h)",  value: 87,       hint: "completed 74 / in_progress 13",  tone: "ok"   },
  { label: "Errors (24h)", value: 3,        hint: "ci_failed 2 / failed 1",         tone: "err"  },
];

export const COST_SERIES: ReadonlyArray<CostPoint> = [
  { day: "Mon", usd: 9.4  },
  { day: "Tue", usd: 11.2 },
  { day: "Wed", usd: 8.7  },
  { day: "Thu", usd: 13.1 },
  { day: "Fri", usd: 12.48 },
  { day: "Sat", usd: 6.3  },
  { day: "Sun", usd: 4.2  },
];

// FALLBACK alias for page SSR — page 改 useEffect+fetch 后, SSR 阶段用 FALLBACK 兜底
// (per mock-msw-handlers.md §2.4 + §4 #1 缺标)
export const MOCK_KPI_FALLBACK: ReadonlyArray<KpiCard> = MOCK_KPI;
export const COST_SERIES_FALLBACK: ReadonlyArray<CostPoint> = COST_SERIES;
