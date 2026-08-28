// frontend/src/mocks/handlers/agents.ts
// MSW handlers for /api/agents (per mock-msw-handlers.md §2.2)
//
// 触发: M2-A 任务 (per 8/28 22:13 JST questionnaire m1-msw-fixtures)
// 设计书: docs/frontend/design/mock-msw-handlers.md
//
// Endpoints:
//   GET  /api/agents        — 返回 MOCK_AGENTS
//   POST /api/agents        — 校验 body 满足 isAgentRow, 201 或 400
//
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. POST 真实持久化 P3 — Phase F+ 后端就绪时
//   2. response shape 与 backend 真实 schema 100% 一致 P2 (Phase F+)

import { http, HttpResponse } from "msw";
import { MOCK_AGENTS } from "@/mocks/data";
import { isAgentRow } from "@/mocks/schemas/agent";

export const agentsHandlers = [
  http.get("/api/agents", () => {
    return HttpResponse.json(MOCK_AGENTS);
  }),
  http.post("/api/agents", async ({ request }) => {
    const body = await request.json();
    if (!isAgentRow(body)) {
      return HttpResponse.json({ error: "Invalid agent row" }, { status: 400 });
    }
    // P3 缺口: 真实持久化
    return HttpResponse.json(body, { status: 201 });
  }),
];
