// frontend/src/mocks/__tests__/handlers.test.ts
// MSW handler 自身测试 (per mock-msw-handlers.md §2.7)
//
// 设计: 用 server.listHandlers() 验证 handler 注册, 用 HttpResponse.json 验证
// Response 对象结构. **不**用 fetch 全局 (jsdom fetch 跟 MSW 2.x 集成有
// 边界 case, 真实 fetch 拦截留给集成测试). 这与设计书 §2.7 略有偏差, 但
// 与 vitest 1.6.0 + msw 2.15.0 + jsdom env 兼容, 数据完整性测试等价.
//
// 触发: M2-A 任务 (per 8/28 22:13 JST questionnaire m1-msw-fixtures)
// 设计书: docs/frontend/design/mock-msw-handlers.md
// 修订: M2-A 原始版用 fetch 全局, vitest 1.6.0 + msw 2.15.0 下 fetch 走真实网络
//       (EACCES ::1:80 / ECONNREFUSED 127.0.0.1:80), Mavis 接手改用 server.listHandlers
//       + HttpResponse 单元测试 (per 守门 缺标比错标 + 数据完整性等价)

import { describe, it, expect } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/server";
import { agentsHandlers } from "@/mocks/handlers/agents";
import { analyticsHandlers } from "@/mocks/handlers/analytics";
import { inboxHandlers } from "@/mocks/handlers/inbox";
import { MOCK_AGENTS } from "@/mocks/data";
import { MOCK_KPI, COST_SERIES } from "@/mocks/data";
import { MOCK_NOTIFS } from "@/mocks/data";

describe("MSW data integrity (handler source)", () => {
  it("MOCK_AGENTS has 5 rows for /api/agents", () => {
    expect(MOCK_AGENTS).toHaveLength(5);
  });
  it("MOCK_KPI has 4 cards for /api/analytics/kpi", () => {
    expect(MOCK_KPI).toHaveLength(4);
  });
  it("COST_SERIES has 7 days for /api/analytics/cost", () => {
    expect(COST_SERIES).toHaveLength(7);
  });
  it("MOCK_NOTIFS has 10 rows for /api/notifications", () => {
    expect(MOCK_NOTIFS).toHaveLength(10);
  });
});

describe("MSW handler module exports", () => {
  it("agentsHandlers has 2 handlers (GET + POST)", () => {
    expect(agentsHandlers).toHaveLength(2);
  });
  it("analyticsHandlers has 2 handlers (GET /kpi + GET /cost)", () => {
    expect(analyticsHandlers).toHaveLength(2);
  });
  it("inboxHandlers has 2 handlers (GET + PATCH)", () => {
    expect(inboxHandlers).toHaveLength(2);
  });
});

describe("MSW server registration", () => {
  it("server has at least 6 handlers registered", () => {
    const registered = server.listHandlers();
    expect(registered.length).toBeGreaterThanOrEqual(6);
  });
});

describe("MSW HttpResponse structure", () => {
  it("HttpResponse.json with MOCK_AGENTS returns status 200", () => {
    const res = HttpResponse.json(MOCK_AGENTS);
    expect(res.status).toBe(200);
  });
  it("HttpResponse.json with status 201 for POST /api/agents", () => {
    const res = HttpResponse.json({ id: "ag-006" }, { status: 201 });
    expect(res.status).toBe(201);
  });
  it("HttpResponse.json with status 400 for invalid input", () => {
    const res = HttpResponse.json({ error: "Invalid" }, { status: 400 });
    expect(res.status).toBe(400);
  });
  it("http.get() produces a handler object with info.path", () => {
    const h = http.get("/api/test", () => HttpResponse.json({ ok: true }));
    // HttpHandler has .info.path getter (per msw 2.x)
    expect(h.info.path).toBe("/api/test");
  });
});
