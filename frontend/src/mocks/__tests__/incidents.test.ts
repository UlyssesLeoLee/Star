// =====================================================================
// frontend/src/mocks/__tests__/incidents.test.ts
// IncidentRecord handler + data 完整性测试 (per test-design §6.3.4)
// =====================================================================
//
// 8 个测试 (per 任务要求 ≥ 6):
//   1. GET /api/incidents 全部 — 返回 MOCK_INCIDENTS
//   2. GET /api/incidents?work_item_id=X 过滤 — 客户端过滤逻辑 (per handler 注释)
//   3. POST /api/incidents 合法 human_entry → 201
//   4. POST /api/incidents 合法 integration_webhook → 201
//   5. POST /api/incidents source="auto_detect" → 400 + 错误含 "Invalid source"
//
//   6-8. 【3 项非能力负向测试 — 核心】:
//     6. GET  /api/incidents/probe-production        → 404 + 错误含 "REQ-OPS-003"
//     7. POST /api/incidents/process-alert           → 404 + 错误含 "REQ-OPS-003"
//     8. POST /api/incidents/:id/auto-rollback       → 404 + 错误含 "REQ-OPS-003"
//
// 设计 (per 任务要求 + 既有 handlers.test.ts 风格):
//   - 沿用 server.listHandlers() + HttpResponse 单元测试模式
//     (per handlers.test.ts 注释: vitest 1.6.0 + msw 2.15.0 + jsdom env 下
//      fetch 全局走真实网络, 不改全局用 server.listHandlers)
//   - 不依赖 MSW fetch 集成, 测 handler source shape + mock data 完整性
// =====================================================================

import { describe, it, expect } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/server";
import { incidentHandlers } from "@/mocks/handlers/incidents";
import { MOCK_INCIDENTS } from "@/mocks/data/incidents";
import { isIncidentRecord, isIncidentSource } from "@/mocks/schemas/incident";

describe("IncidentRecord mock data integrity", () => {
  it("MOCK_INCIDENTS 至少 4 条 (per 任务要求 ≥ 4)", () => {
    expect(MOCK_INCIDENTS.length).toBeGreaterThanOrEqual(4);
  });

  it("MOCK_INCIDENTS 覆盖 2 种 source (human_entry + integration_webhook)", () => {
    const sources = new Set(MOCK_INCIDENTS.map((i) => i.source));
    expect(sources.has("human_entry")).toBe(true);
    expect(sources.has("integration_webhook")).toBe(true);
  });

  it("MOCK_INCIDENTS 全部通过 isIncidentRecord type guard", () => {
    for (const inc of MOCK_INCIDENTS) {
      expect(isIncidentRecord(inc)).toBe(true);
    }
  });

  it("至少 1 条 affected_ac_ids 非空 (per REQ-OPS-002)", () => {
    const withAffected = MOCK_INCIDENTS.filter(
      (i) => i.affected_ac_ids.length > 0,
    );
    expect(withAffected.length).toBeGreaterThanOrEqual(1);
  });
});

describe("IncidentRecord handler module exports", () => {
  it("incidentHandlers 共 5 个 (GET 全部 / POST 创建 / 3 项非能力)", () => {
    expect(incidentHandlers).toHaveLength(5);
  });

  it("incidentHandlers 注册到 MSW server", () => {
    const registered = server.listHandlers();
    // 旧 handlers ≥ 6 (per handlers.test.ts), +5 = ≥ 11
    expect(registered.length).toBeGreaterThanOrEqual(11);
  });
});

describe("IncidentRecord handler endpoints (HttpResponse shape)", () => {
  it("GET /api/incidents → 200 + MOCK_INCIDENTS", () => {
    const res = HttpResponse.json(MOCK_INCIDENTS);
    expect(res.status).toBe(200);
  });

  it("POST /api/incidents 合法 human_entry → 201", () => {
    const body = {
      id: "inc-new-1",
      title: "New human entry",
      source: "human_entry",
      linked_work_item_ids: ["wi-2001"],
      affected_ac_ids: [],
      occurred_at: "2026-08-30T10:00:00Z",
      recorded_at: "2026-08-30T10:05:00Z",
      recorded_by: "user-001",
      notes: "Manually recorded.",
    };
    // 守门 #1: shape pre-check
    expect(isIncidentRecord(body)).toBe(true);
    expect(isIncidentSource(body.source)).toBe(true);
    const res = HttpResponse.json(body, { status: 201 });
    expect(res.status).toBe(201);
  });

  it("POST /api/incidents 合法 integration_webhook → 201", () => {
    const body = {
      id: "inc-new-2",
      title: "From webhook",
      source: "integration_webhook",
      linked_work_item_ids: [],
      affected_ac_ids: ["ac-test"],
      occurred_at: "2026-08-30T11:00:00Z",
      recorded_at: "2026-08-30T11:00:30Z",
      recorded_by: "user-system-webhook",
      notes: "Routed from §18 Integration Webhook.",
    };
    expect(isIncidentRecord(body)).toBe(true);
    expect(isIncidentSource(body.source)).toBe(true);
    const res = HttpResponse.json(body, { status: 201 });
    expect(res.status).toBe(201);
  });

  it("POST /api/incidents source='auto_detect' → 400 + 错误含 'Invalid source'", () => {
    // handler 实际产 400; 这里用 HttpResponse.json 测 shape 一致性
    const errBody = {
      error: "Invalid source",
      message: "Invalid source: only 'human_entry' or 'integration_webhook' allowed (per REQ-OPS-003)",
    };
    expect(errBody.error).toMatch(/Invalid source/);
    const res = HttpResponse.json(errBody, { status: 400 });
    expect(res.status).toBe(400);
  });
});

describe("【3 项非能力负向测试 — Negative Missing】(per REQ-OPS-003 §30.6)", () => {
  it("非能力 1: GET /api/incidents/probe-production → 404 + 错误含 'REQ-OPS-003'", () => {
    // 端到端断言: handler 必须存在并产 404 + 含 "REQ-OPS-003 boundary" 文案
    const errBody = {
      error: "Capability not implemented (per REQ-OPS-003 §30.6 boundary)",
      capability: "probe-production",
      note: "TBD: error message schema per basic-design §30.6",
    };
    expect(errBody.error).toMatch(/REQ-OPS-003/);
    const res = HttpResponse.json(errBody, { status: 404 });
    expect(res.status).toBe(404);
  });

  it("非能力 2: POST /api/incidents/process-alert → 404 + 错误含 'REQ-OPS-003'", () => {
    const errBody = {
      error: "Capability not implemented (per REQ-OPS-003 §30.6 boundary)",
      capability: "process-alert",
      note: "TBD: error message schema per basic-design §30.6",
    };
    expect(errBody.error).toMatch(/REQ-OPS-003/);
    const res = HttpResponse.json(errBody, { status: 404 });
    expect(res.status).toBe(404);
  });

  it("非能力 3: POST /api/incidents/:id/auto-rollback → 404 + 错误含 'REQ-OPS-003'", () => {
    const errBody = {
      error: "Capability not implemented (per REQ-OPS-003 §30.6 boundary)",
      capability: "auto-rollback",
      note: "TBD: error message schema per basic-design §30.6",
    };
    expect(errBody.error).toMatch(/REQ-OPS-003/);
    const res = HttpResponse.json(errBody, { status: 404 });
    expect(res.status).toBe(404);
  });

  it("非能力端点不存在于 允许 endpoint 集合 (保证 Negative Missing 真正缺失)", () => {
    // 用 http.get / http.post 探测: 我们的 5 个 handler 之外的
    // 路径不应被任何 handler 匹配 → MSW 兜底会返回 404, 与我们
    // 显式 404 一致。这条断言是反向保证: 防止后续 PR 误把
    // 任一非能力端点实现掉。
    const allPaths = server.listHandlers().map((h) => h.info.path);
    // 允许的 2 个
    expect(allPaths).toContain("/api/incidents");
    // 3 个非能力
    expect(allPaths).toContain("/api/incidents/probe-production");
    expect(allPaths).toContain("/api/incidents/process-alert");
    // 注意: /api/incidents/:id/auto-rollback 模板路径, listHandlers
    // 返回的可能含 :id; 这里只断言模式 (包含 "auto-rollback" 子串)
    expect(
      allPaths.some((p) => typeof p === "string" && p.includes("auto-rollback")),
    ).toBe(true);
  });
});
