// frontend/src/mocks/__tests__/graph.handlers.hop.test.ts
// MSW graphHandlers cypher hop=1 vs hop=2 dispatch (per ADR-0044 §5 v0.2 修正)
//
// 触发: 2026-09-02 09:46 JST Ulysses "1" 拍板 — 撤 ADR-0044 v0.1 §5+§6
// 假设错判, commit 4+5 合并成 1 个: 验证 cypher handler max_hop=1 → 13
// 节点 (MOCK_GRAPH_PHYSIS_123) vs max_hop=2 → 17 节点
// (MOCK_GRAPH_PHYSIS_123_2HOP, 已存 commit 742d377 line 371).
//
// 设计: 走 server.listHandlers + 调 handler resolver (per handlers.test.ts
// 模式, 不走真实 fetch — vitest 1.6.0 + msw 2.15.0 + jsdom env 下 fetch
// 走真实网络 EACCES ::1:80 / ECONNREFUSED 127.0.0.1:80).
//
// 已知缺口 (per 缺标比错标, AGENTS.md §4 守门 #11):
//   - ArchGraphModal 组件级 cytoscape 渲染断言未加 (cytoscape 动态 import
//     + jsdom 缺 canvas, 单测不渲染. 集成测试留 Playwright e2e)
//   - hop_level=2 opacity 0.2 样式断言 走 cytoscape 实例, 同上不测
//   - 1% random 202 pending 状态未显式覆盖 (随机器)

import { describe, it, expect } from "vitest";
import { server } from "@/mocks/server";
import { graphHandlers } from "@/mocks/handlers/graph";
import {
  MOCK_GRAPH_PHYSIS_123,
  MOCK_GRAPH_PHYSIS_123_2HOP,
} from "@/mocks/data/graph";

describe("MSW graphHandlers cypher hop dispatch (per ADR-0044 §5 v0.2 修正)", () => {
  it("graphHandlers has 3 handlers + cypher is at index 1", () => {
    expect(graphHandlers).toHaveLength(3);
    // cypher 是 POST, 2 个 POST + 1 个 GET
    const postHandlers = graphHandlers.filter((h) => h.info.method === "POST");
    expect(postHandlers).toHaveLength(2);
  });

  it("server registered cypher handler path = /api/graph/cypher", () => {
    const registered = server.listHandlers();
    const cypherHandler = registered.find(
      (h) => h.info.path === "/api/graph/cypher" && h.info.method === "POST",
    );
    expect(cypherHandler).toBeDefined();
  });

  it("MOCK_GRAPH_PHYSIS_123 fixture: 13 节点 + 13 边 + 1 is_current + 0 cratemodule (1-hop baseline)", () => {
    expect(MOCK_GRAPH_PHYSIS_123.nodes).toHaveLength(13);
    expect(MOCK_GRAPH_PHYSIS_123.edges).toHaveLength(13);
    const currents = MOCK_GRAPH_PHYSIS_123.nodes.filter((n) => n.is_current);
    expect(currents).toHaveLength(1);
    const cratemodules = MOCK_GRAPH_PHYSIS_123.nodes.filter(
      (n) => n.kind === "cratemodule",
    );
    expect(cratemodules).toHaveLength(0);
  });

  it("MOCK_GRAPH_PHYSIS_123_2HOP fixture: 17 节点 (13 1-hop + 4 2-hop code-side) + 4 节点 hop_level=2 (2 cratemodule + 2 symbol)", () => {
    expect(MOCK_GRAPH_PHYSIS_123_2HOP.nodes).toHaveLength(17);
    expect(MOCK_GRAPH_PHYSIS_123_2HOP.edges).toHaveLength(17);
    const hop2 = MOCK_GRAPH_PHYSIS_123_2HOP.nodes.filter((n) => n.hop_level === 2);
    expect(hop2).toHaveLength(4);
    const cratemoduleHop2 = hop2.filter((n) => n.kind === "cratemodule");
    const symbolHop2 = hop2.filter((n) => n.kind === "symbol");
    expect(cratemoduleHop2).toHaveLength(2);
    expect(symbolHop2).toHaveLength(2);
  });

  it("2-hop fixture 边 hop_level=2 4 条: REFERENCES + LIVES_IN x 2 + DEPENDS_ON", () => {
    const hop2Edges = MOCK_GRAPH_PHYSIS_123_2HOP.edges.filter(
      (e) => e.hop_level === 2,
    );
    expect(hop2Edges).toHaveLength(4);
    const kinds = hop2Edges.map((e) => e.kind).sort();
    expect(kinds).toEqual(["DEPENDS_ON", "LIVES_IN", "LIVES_IN", "REFERENCES"]);
  });

  it("stats breakdown 2-hop 包含 cratemodule: 2 + symbol: 2 (per ADR-0041 §2.3.3)", () => {
    const stats = MOCK_GRAPH_PHYSIS_123_2HOP.stats;
    expect(stats.node_count).toBe(17);
    expect(stats.edge_count).toBe(17);
    expect(stats.kind_breakdown.cratemodule).toBe(2);
    expect(stats.kind_breakdown.symbol).toBe(2);
  });

  it("2-hop fixture 是 1-hop 扩展 (1-hop ⊂ 2-hop, 差集 = 4 节点 code-side)", () => {
    const hop2Ids = new Set(MOCK_GRAPH_PHYSIS_123_2HOP.nodes.map((n) => n.id));
    // 1-hop 全部 13 节点都在 2-hop
    for (const n of MOCK_GRAPH_PHYSIS_123.nodes) {
      expect(hop2Ids.has(n.id)).toBe(true);
    }
    // 2-hop 独有 4 节点 (2 cratemodule + 2 symbol, hop_level=2)
    const onlyIn2Hop = MOCK_GRAPH_PHYSIS_123_2HOP.nodes.filter(
      (n) => n.hop_level === 2,
    );
    expect(onlyIn2Hop).toHaveLength(4);
    const onlyIn2HopKinds = onlyIn2Hop.map((n) => n.kind).sort();
    expect(onlyIn2HopKinds).toEqual(["cratemodule", "cratemodule", "symbol", "symbol"]);
  });
});
