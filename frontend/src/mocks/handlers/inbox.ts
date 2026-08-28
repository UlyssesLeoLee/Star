// frontend/src/mocks/handlers/inbox.ts
// MSW handlers for /api/notifications (per mock-msw-handlers.md §2.2)
//
// Endpoints:
//   GET   /api/notifications      — 返回 MOCK_NOTIFS (10 rows)
//   PATCH /api/notifications/:id  — 标记 read=true (P3 真实持久化)
//
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. PATCH 真实持久化 P3 — Phase F+
//   2. SSE 实时推送 P3 (Phase I+)
//   3. 联动 useStore.notifications P3

import { http, HttpResponse } from "msw";
import { MOCK_NOTIFS } from "@/mocks/data";

export const inboxHandlers = [
  http.get("/api/notifications", () => {
    return HttpResponse.json(MOCK_NOTIFS);
  }),
  http.patch("/api/notifications/:id", async ({ params }) => {
    return HttpResponse.json({ id: params.id, read: true });
  }),
];
