// =====================================================================
// MSW handlers for /api/graph/* (per ADR-0041 §2.3.4)
// =====================================================================
// Endpoints (3):
//   POST /api/graph/ensure-fresh   幂等+排他 触发 agent 增量 (本 Phase 1 = mock 直接返)
//   POST /api/graph/cypher         1-hop 查询 (mock 返固定 fixture)
//   GET  /api/graph/health         健康检查
//
// 守门 (per AGENTS.md §0/§1.2 + handlers/cli.ts 模式):
//   - maybeReal 头插入 real-mode bypass (P3-A.7 派生, per 8/27 commit 模式)
//   - 404 / 400 / 202 / 200 全显式分支
//   - tenant_id 必带 (per REQ-SEC-001, 13 类)
//   - work_item_id 不存在 → 404 (不返 mock 兜底, 避免污染前端逻辑)
//
// 已知缺口 (per 缺标比错标, 8/26 JST 守门 #1):
//   1. ensure-fresh 不会真触发后端 agent, mock 直接返 fresh
//   2. fingerprint / lock / 多人 coalesce 全部留 Phase 2
//   3. real-mode bypass 留 P3-A.7 实证后接
// =====================================================================

import { http, HttpResponse } from "msw";
import {
  MOCK_GRAPH_PHYSIS_123,
  MOCK_GRAPH_PHYSIS_123_2HOP,
  MOCK_GRAPH_EMPTY,
} from "@/mocks/data/graph";
import type {
  EnsureFreshRequest,
  GraphCypherRequest,
} from "@/types/graph";

/** 已知 work_item mock 集合 (per mocks/data/graph.ts 实证) */
const MOCK_WI_IDS = new Set(["wi-arch-001"]);

function isValidEnsureFresh(body: unknown): body is EnsureFreshRequest {
  if (typeof body !== "object" || body === null) return false;
  const b = body as Record<string, unknown>;
  return (
    typeof b.work_item_id === "string" && b.work_item_id.length > 0
    && typeof b.tenant_id === "string" && b.tenant_id.length > 0
    && (b.source === "local" || b.source === "git")
  );
}

function isValidCypher(body: unknown): body is GraphCypherRequest {
  if (typeof body !== "object" || body === null) return false;
  const b = body as Record<string, unknown>;
  return (
    typeof b.work_item_id === "string" && b.work_item_id.length > 0
    && typeof b.tenant_id === "string" && b.tenant_id.length > 0
    && (b.max_hop === 1 || b.max_hop === 2)
  );
}

export const graphHandlers = [
  // 1. ensure-fresh (per ADR-0041 §2.3.4)
  //   - mock 直接返 fresh, 不真触发 agent
  //   - 留 202 路径分支代码 (Phase 2 真接时用)
  http.post("/api/graph/ensure-fresh", async ({ request }) => {
    const body = await request.json();
    if (!isValidEnsureFresh(body)) {
      return HttpResponse.json(
        { error: "Invalid ensure-fresh payload" },
        { status: 400 },
      );
    }
    if (!MOCK_WI_IDS.has(body.work_item_id)) {
      return HttpResponse.json(
        { error: "work_item_not_found", work_item_id: body.work_item_id },
        { status: 404 },
      );
    }
    // mock 99% 返 fresh, 1% 模拟 202 (testing pending state)
    const isPending = Math.random() < 0.01;
    if (isPending) {
      return HttpResponse.json(
        { status: "running", retry_after_ms: 3000, phase: "llm_infer" },
        { status: 202 },
      );
    }
    return HttpResponse.json(
      { status: "fresh", graph: MOCK_GRAPH_PHYSIS_123 },
      { status: 200 },
    );
  }),

  // 2. cypher (1-hop 查询)
  http.post("/api/graph/cypher", async ({ request }) => {
    const body = await request.json();
    if (!isValidCypher(body)) {
      return HttpResponse.json(
        { error: "Invalid cypher payload" },
        { status: 400 },
      );
    }
    if (!MOCK_WI_IDS.has(body.work_item_id)) {
      return HttpResponse.json(
        { error: "work_item_not_found", work_item_id: body.work_item_id },
        { status: 404 },
      );
    }
    const graph = body.max_hop === 2
      ? MOCK_GRAPH_PHYSIS_123_2HOP
      : MOCK_GRAPH_PHYSIS_123;
    return HttpResponse.json(graph, { status: 200 });
  }),

  // 3. health (用于 modal 错误诊断)
  http.get("/api/graph/health", () => {
    return HttpResponse.json({
      memgraph: "up",
      agent_runtime: "up",
      last_successful_run: "2026-09-02T00:55:00Z",
      queue_depth: 2,
    });
  }),
];
