/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/agent-view/layout.math.test.ts"})
MERGE (self)-[:TESTS]->(t:File {path:"frontend/src/lib/agent-view/layout.ts"})
@endcypher
*/

/**
 * layout.ts 数学性质测试 (2026-11-21, per "完善自动测试脚本和mock项目，加入数学测试的内容")
 *
 * 与 layout.test.ts (示例驱动) 互补, 覆盖几何不变量:
 *   L1. 圆环等距: 内圈任意两相邻 work-item 中心距相等 (正多边形)
 *   L2. 圆环半径: 所有内圈节点中心到 worktree 中心距离 = RING1_RADIUS
 *   L3. 内外圈不重叠: 外圈节点中心到 worktree 中心距离 > 内圈半径 + 节点半宽
 *   L4. bbox 包含性: 所有节点矩形 ⊆ bbox (含 80px padding)
 *   L5. fitToContentViewport 居中: bbox 中心映射到容器中心
 *   L6. fitToContentViewport 包含性: 非 clamp 场景下 bbox 投影 ⊆ 容器 (含 padding)
 *   L7. fitToContentViewport zoom 单调性: bbox 放大 ⇒ zoom 不增
 *   L8. 确定性: 相同输入 ⇒ 逐字段相等 (含浮点)
 */

import { describe, expect, it } from "vitest";
import { layoutAgentCanvas, fitToContentViewport } from "./layout";
import type { AgentSession, Worktree, WorkItem } from "@/types/ids";

const baseAgent: AgentSession = {
  id: "ag-001",
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  worktree_id: "wt-001",
  agent_kind: "claude-sonnet",
  status: "executing",
  current_step: "tool.call:grep",
  token_usage: { input: 1000, output: 200, total: 1200 },
  cost_summary: { usd: 0.5, budget_usd: 5.0 },
  started_at: "2026-09-05T10:00:00Z",
};

const baseWorktree: Worktree = {
  id: "wt-001",
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  name: "wt-1",
  branch: "feat/test",
  base_branch: "main",
  status: "active",
  lock_version: 1,
  last_event_at: "2026-09-05T10:00:00Z",
  created_at: "2026-09-05T09:00:00Z",
};

const makeWi = (i: number): WorkItem => ({
  id: `test-wi-${i.toString().padStart(3, "0")}`,
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  key: `PHYSIS-${i}`,
  title: `Task ${i}`,
  description: "",
  kind: "task",
  status: "todo",
  priority: "p1",
  reporter_id: "usr-001",
  labels: [],
  workflow_id: "wf-default",
  worktree_id: "wt-001",
  created_at: "2026-09-05T08:00:00Z",
  updated_at: "2026-09-05T08:00:00Z",
});

function center(n: { x: number; y: number; width: number; height: number }) {
  return { x: n.x + n.width / 2, y: n.y + n.height / 2 };
}

describe("layoutAgentCanvas 几何不变量", () => {
  const out8 = layoutAgentCanvas({
    agent: baseAgent,
    worktree: baseWorktree,
    workItems: Array.from({ length: 8 }, (_, i) => makeWi(i)),
  });
  const wtNode = out8.nodes.find((n) => n.kind === "worktree")!;
  const wtCenter = center(wtNode);
  const wiNodes = out8.nodes.filter((n) => n.kind === "work_item");

  it("L1: 内圈正多边形 — 相邻节点中心距全部相等", () => {
    expect(wiNodes).toHaveLength(8);
    const centers = wiNodes.map(center);
    const dists = centers.map((c, i) => {
      const next = centers[(i + 1) % centers.length];
      return Math.hypot(c.x - next.x, c.y - next.y);
    });
    for (const d of dists) {
      expect(Math.abs(d - dists[0])).toBeLessThan(1e-6);
    }
    // 正八边形边长 = 2R·sin(π/8), R=280 (RING1_RADIUS)
    const expected = 2 * 280 * Math.sin(Math.PI / 8);
    expect(Math.abs(dists[0] - expected)).toBeLessThan(1e-6);
  });

  it("L2: 内圈半径恒定 — 所有节点中心到 worktree 中心 = 280", () => {
    for (const n of wiNodes) {
      const c = center(n);
      const r = Math.hypot(c.x - wtCenter.x, c.y - wtCenter.y);
      expect(Math.abs(r - 280)).toBeLessThan(1e-6);
    }
  });

  it("L3: 外圈半径 460 > 内圈 280 + 节点半宽 (不重叠)", () => {
    const out12 = layoutAgentCanvas({
      agent: baseAgent,
      worktree: baseWorktree,
      workItems: Array.from({ length: 12 }, (_, i) => makeWi(i)),
    });
    const outer = out12.nodes.filter((n) => n.kind === "work_item").slice(8);
    expect(outer).toHaveLength(4);
    for (const n of outer) {
      const c = center(n);
      const r = Math.hypot(c.x - wtCenter.x, c.y - wtCenter.y);
      expect(Math.abs(r - 460)).toBeLessThan(1e-6);
      // 外圈最近点与内圈最远点恰好相切 (460−90 = 280+90 = 370, 设计如此)
      expect(r - n.width / 2).toBeGreaterThanOrEqual(280 + wiNodes[0].width / 2);
    }
  });

  it("L4: bbox 包含所有节点矩形 + 80px padding", () => {
    for (const out of [out8, layoutAgentCanvas({ agent: baseAgent, worktree: baseWorktree, workItems: Array.from({ length: 15 }, (_, i) => makeWi(i)) })]) {
      const { bbox } = out;
      for (const n of out.nodes) {
        expect(n.x).toBeGreaterThanOrEqual(bbox.minX);
        expect(n.y).toBeGreaterThanOrEqual(bbox.minY);
        expect(n.x + n.width).toBeLessThanOrEqual(bbox.maxX);
        expect(n.y + n.height).toBeLessThanOrEqual(bbox.maxY);
      }
      // padding 恰好 80: bbox 边界 = 极值节点 ± 80
      const minX = Math.min(...out.nodes.map((n) => n.x));
      const maxY = Math.max(...out.nodes.map((n) => n.y + n.height));
      expect(bbox.minX).toBe(minX - 80);
      expect(bbox.maxY).toBe(maxY + 80);
    }
  });

  it("L8: 确定性 — 相同输入逐字段相等", () => {
    const input = {
      agent: baseAgent,
      worktree: baseWorktree,
      workItems: Array.from({ length: 10 }, (_, i) => makeWi(i)),
    };
    const a = layoutAgentCanvas(input);
    const b = layoutAgentCanvas(input);
    expect(JSON.stringify(a)).toBe(JSON.stringify(b));
  });
});

describe("fitToContentViewport 几何不变量", () => {
  it("L5: 居中 — bbox 中心投影到容器中心", () => {
    const bboxes = [
      { minX: 0, minY: 0, maxX: 600, maxY: 400 },
      { minX: -500, minY: 200, maxX: 700, maxY: 900 },
      { minX: 100, minY: 100, maxX: 1100, maxY: 700 },
    ];
    for (const bbox of bboxes) {
      const vp = fitToContentViewport(bbox);
      const cx = (bbox.minX + bbox.maxX) / 2;
      const cy = (bbox.minY + bbox.maxY) / 2;
      // 世界→屏幕: (world - vp.origin) * zoom; 中心应落在 (600, 400)
      expect(Math.abs((cx - vp.x) * vp.zoom - 600)).toBeLessThan(1e-6);
      expect(Math.abs((cy - vp.y) * vp.zoom - 400)).toBeLessThan(1e-6);
    }
  });

  it("L6: 包含性 — 非 clamp 场景 bbox 投影 ⊆ 容器 (含 40px padding)", () => {
    // 适中 bbox: zoom 不被 0.2/1.5 clamp
    const bbox = { minX: 0, minY: 0, maxX: 800, maxY: 500 };
    const vp = fitToContentViewport(bbox);
    expect(vp.zoom).toBeGreaterThan(0.2);
    expect(vp.zoom).toBeLessThan(1.5);
    const left = (bbox.minX - vp.x) * vp.zoom;
    const right = (bbox.maxX - vp.x) * vp.zoom;
    const top = (bbox.minY - vp.y) * vp.zoom;
    const bottom = (bbox.maxY - vp.y) * vp.zoom;
    expect(left).toBeGreaterThanOrEqual(40 - 1e-6);
    expect(right).toBeLessThanOrEqual(1200 - 40 + 1e-6);
    expect(top).toBeGreaterThanOrEqual(40 - 1e-6);
    expect(bottom).toBeLessThanOrEqual(800 - 40 + 1e-6);
  });

  it("L7: zoom 单调性 — bbox 等比放大 ⇒ zoom 不增", () => {
    let prevZoom = Infinity;
    for (const scale of [0.5, 1, 2, 4, 8, 16]) {
      const bbox = { minX: 0, minY: 0, maxX: 400 * scale, maxY: 300 * scale };
      const vp = fitToContentViewport(bbox);
      expect(vp.zoom).toBeLessThanOrEqual(prevZoom + 1e-12);
      prevZoom = vp.zoom;
    }
  });

  it("L7b: zoom clamp 边界 — 极大 bbox → 0.2, 极小 bbox → 1.5", () => {
    const huge = fitToContentViewport({ minX: 0, minY: 0, maxX: 100000, maxY: 100000 });
    expect(huge.zoom).toBe(0.2);
    const tiny = fitToContentViewport({ minX: 0, minY: 0, maxX: 10, maxY: 10 });
    expect(tiny.zoom).toBe(1.5);
  });
});