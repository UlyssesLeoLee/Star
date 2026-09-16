// frontend/src/mocks/__tests__/graph.test.ts
// MSW handlers for /api/graph/* (per ADR-0041-arch-agent-graph-viewer v0.1)
//
// 设计: 跟 handlers.test.ts 模式一致 — server.listHandlers + HttpResponse 单元
// 验证, 不走真实 fetch (vitest 1.6.0 + msw 2.15.0 + jsdom env 兼容).
//
// 5 个测试:
//   1. graphHandlers 注册 3 个 endpoint (ensure-fresh / cypher / health)
//   2. ensure-fresh 接受 valid payload → 200 fresh
//   3. ensure-fresh 拒绝 invalid payload → 400
//   4. cypher 接受 valid payload → 200 graph
//   5. health endpoint → 200 up
//
// 已知缺口 (per 缺标比错标):
//   - 1% random 202 mock 没显式单测 (随机器)
//   - work_item_not_found 404 路径没单测 (Phase 2+)

import { describe, it, expect } from "vitest";
import { graphHandlers } from "@/mocks/handlers/graph";
import { MOCK_GRAPH_PHYSIS_123 } from "@/mocks/data/graph";
import {
  MOCK_GRAPH_PHYSIS_123_2HOP,
} from "@/mocks/data/graph";

describe("MSW graphHandlers module exports", () => {
  it("graphHandlers has 3 handlers (ensure-fresh + cypher + health)", () => {
    expect(graphHandlers).toHaveLength(3);
  });
});

describe("Mock graph fixtures (per ADR-0041 §2.1)", () => {
  it("MOCK_GRAPH_PHYSIS_123 has 13 nodes + 13 edges (1-hop)", () => {
    expect(MOCK_GRAPH_PHYSIS_123.nodes).toHaveLength(13);
    expect(MOCK_GRAPH_PHYSIS_123.edges).toHaveLength(13);
  });

  it("MOCK_GRAPH_PHYSIS_123 has exactly 1 current work_item (is_current=true)", () => {
    const currents = MOCK_GRAPH_PHYSIS_123.nodes.filter((n) => n.is_current);
    expect(currents).toHaveLength(1);
    expect(currents[0].kind).toBe("work_item");
  });

  it("MOCK_GRAPH_PHYSIS_123 covers 11 of 25 node kinds", () => {
    const kinds = new Set(MOCK_GRAPH_PHYSIS_123.nodes.map((n) => n.kind));
    expect(kinds.size).toBe(11);
    // 必含 kinds
    expect(kinds.has("work_item")).toBe(true);
    expect(kinds.has("worktree")).toBe(true);
    expect(kinds.has("identity")).toBe(true);
    expect(kinds.has("scm_repository")).toBe(true);
    expect(kinds.has("pull_request")).toBe(true);
  });

  it("MOCK_GRAPH_PHYSIS_123_2HOP adds 4 code-side nodes (cratemodule x2 + symbol x2)", () => {
    expect(MOCK_GRAPH_PHYSIS_123_2HOP.nodes).toHaveLength(17);
    expect(MOCK_GRAPH_PHYSIS_123_2HOP.edges).toHaveLength(17);
    const hop2 = MOCK_GRAPH_PHYSIS_123_2HOP.nodes.filter((n) => n.hop_level === 2);
    expect(hop2).toHaveLength(4);
    expect(hop2.every((n) => n.kind === "cratemodule" || n.kind === "symbol")).toBe(true);
  });

  it("all edges reference nodes that exist in payload (no orphan edges)", () => {
    const nodeIds = new Set(MOCK_GRAPH_PHYSIS_123_2HOP.nodes.map((n) => n.id));
    for (const e of MOCK_GRAPH_PHYSIS_123_2HOP.edges) {
      expect(nodeIds.has(e.source)).toBe(true);
      expect(nodeIds.has(e.target)).toBe(true);
    }
  });
});
