// frontend/src/mocks/handlers/analytics.ts
// MSW handlers for /api/analytics/* (per mock-msw-handlers.md §2.2)
//
// Endpoints:
//   GET /api/analytics/kpi  — 返回 MOCK_KPI (4 cards)
//   GET /api/analytics/cost — 返回 COST_SERIES (7 days)
//
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. 真实 cost / token 数据 P2 (Phase F+)
//   2. 错误率 / leaderboard P2 (Phase F+)

import { http, HttpResponse } from "msw";
import { MOCK_KPI, COST_SERIES } from "@/mocks/data";

export const analyticsHandlers = [
  http.get("/api/analytics/kpi", () => {
    return HttpResponse.json(MOCK_KPI);
  }),
  http.get("/api/analytics/cost", () => {
    return HttpResponse.json(COST_SERIES);
  }),
];
