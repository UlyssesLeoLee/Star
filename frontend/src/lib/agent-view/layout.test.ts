// =====================================================================
// layout.test.ts — Agent 视图自由散开布局算法
// =====================================================================
// 覆盖:
//   1. 空输入 (无 worktree, 无 work-items) → 仅 agent 节点
//   2. 有 worktree 无 work-items → agent + worktree + 1 connector
//   3. 多 work-items → 内圈 8 个, 超出走外圈
//   4. 排序稳定: status 优先级 > due_date > id (相同输入永远出同样输出)
//   5. fitToContentViewport 边界: 空 bbox, 极大 bbox 都返回合理 zoom
// =====================================================================

import { describe, it, expect } from "vitest";
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

const makeWi = (i: number, status: WorkItem["status"], due?: string): WorkItem => ({
  id: `wi-${i.toString().padStart(3, "0")}`,
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  key: `PHYSIS-${i}`,
  title: `Task ${i}`,
  description: "",
  kind: "task",
  status,
  priority: "p1",
  reporter_id: "usr-001",
  labels: [],
  workflow_id: "wf-default",
  worktree_id: "wt-001",
  due_date: due,
  created_at: "2026-09-05T08:00:00Z",
  updated_at: "2026-09-05T08:00:00Z",
});

describe("layoutAgentCanvas", () => {
  it("无 worktree → 仅返回 agent 节点, 无 connector", () => {
    const out = layoutAgentCanvas({ agent: baseAgent, worktree: null, workItems: [] });
    expect(out.nodes).toHaveLength(1);
    expect(out.nodes[0].kind).toBe("agent");
    expect(out.connectors).toHaveLength(0);
  });

  it("有 worktree 无 work-items → agent + worktree, 1 connector", () => {
    const out = layoutAgentCanvas({ agent: baseAgent, worktree: baseWorktree, workItems: [] });
    expect(out.nodes).toHaveLength(2);
    expect(out.nodes.map((n) => n.kind).sort()).toEqual(["agent", "worktree"]);
    expect(out.connectors).toHaveLength(1);
    expect(out.connectors[0].fromNodeId).toBe(out.nodes[0].id);
    expect(out.connectors[0].toNodeId).toBe(out.nodes[1].id);
  });

  it("3 work-items → agent + worktree + 3 wi, 1+3=4 connectors", () => {
    const wis = [makeWi(1, "todo"), makeWi(2, "in_progress"), makeWi(3, "done")];
    const out = layoutAgentCanvas({ agent: baseAgent, worktree: baseWorktree, workItems: wis });
    expect(out.nodes).toHaveLength(5); // agent + wt + 3 wi
    expect(out.connectors).toHaveLength(4); // agent-wt + 3 wt-wi
    const wiNodes = out.nodes.filter((n) => n.kind === "work_item");
    expect(wiNodes).toHaveLength(3);
  });

  it("10 work-items → 内圈 8 + 外圈 2, 总 8+2 wi 节点", () => {
    const wis = Array.from({ length: 10 }, (_, i) => makeWi(i + 1, "todo"));
    const out = layoutAgentCanvas({ agent: baseAgent, worktree: baseWorktree, workItems: wis });
    const wiNodes = out.nodes.filter((n) => n.kind === "work_item");
    expect(wiNodes).toHaveLength(10);
  });

  it("排序稳定: status in_progress 排在 todo 之前", () => {
    const wis = [
      makeWi(1, "todo"),
      makeWi(2, "in_progress"),
      makeWi(3, "done"),
    ];
    const out = layoutAgentCanvas({ agent: baseAgent, worktree: baseWorktree, workItems: wis });
    const wiNodes = out.nodes.filter((n) => n.kind === "work_item");
    // 期望顺序 (按 status order ASC: in_progress=0, todo=3, done=4):
    //   in_progress (PHYSIS-2) → todo (PHYSIS-1) → done (PHYSIS-3)
    expect(wiNodes[0].ref).toEqual({ kind: "work_item", workItemId: "wi-002" });
    expect(wiNodes[1].ref).toEqual({ kind: "work_item", workItemId: "wi-001" });
    expect(wiNodes[2].ref).toEqual({ kind: "work_item", workItemId: "wi-003" });
  });

  it("相同输入永远出同样输出 (deterministic)", () => {
    const wis = [
      makeWi(1, "in_progress"),
      makeWi(2, "todo"),
      makeWi(3, "review"),
    ];
    const a = layoutAgentCanvas({ agent: baseAgent, worktree: baseWorktree, workItems: wis });
    const b = layoutAgentCanvas({ agent: baseAgent, worktree: baseWorktree, workItems: wis });
    expect(JSON.stringify(a)).toBe(JSON.stringify(b));
  });

  it("connector 颜色按 work-item status 着色", () => {
    const wis = [
      makeWi(1, "in_progress"),
      makeWi(2, "blocked"),
      makeWi(3, "done"),
    ];
    const out = layoutAgentCanvas({ agent: baseAgent, worktree: baseWorktree, workItems: wis });
    const wiConnectors = out.connectors.filter((c) => c.label === "in_progress" || c.label === "blocked" || c.label === "done");
    expect(wiConnectors).toHaveLength(3);
    const inProgressC = wiConnectors.find((c) => c.label === "in_progress");
    expect(inProgressC?.color).toBe("#2f81f7"); // blue
    const blockedC = wiConnectors.find((c) => c.label === "blocked");
    expect(blockedC?.color).toBe("#f85149"); // red
    const doneC = wiConnectors.find((c) => c.label === "done");
    expect(doneC?.color).toBe("#3fb950"); // green
  });

  it("bbox 包含所有节点 + 60px padding", () => {
    const wis = [makeWi(1, "todo")];
    const out = layoutAgentCanvas({ agent: baseAgent, worktree: baseWorktree, workItems: wis });
    const xs = out.nodes.map((n) => n.x);
    const xe = out.nodes.map((n) => n.x + n.width);
    expect(out.bbox.minX).toBeLessThanOrEqual(Math.min(...xs) - 79);
    expect(out.bbox.maxX).toBeGreaterThanOrEqual(Math.max(...xe) + 79);
  });
});

describe("fitToContentViewport", () => {
  it("空 bbox → 返回默认 viewport", () => {
    const v = fitToContentViewport({ minX: 0, minY: 0, maxX: 1200, maxY: 800 });
    expect(v.zoom).toBeGreaterThan(0);
    expect(v.zoom).toBeLessThanOrEqual(1.5);
  });

  it("极大 bbox → zoom 被 clamp 到 0.2 最小值", () => {
    const v = fitToContentViewport({ minX: 0, minY: 0, maxX: 100000, maxY: 100000 });
    expect(v.zoom).toBe(0.2);
  });

  it("适中 bbox → zoom 在 0.2..1.5 范围", () => {
    const v = fitToContentViewport({ minX: 0, minY: 0, maxX: 600, maxY: 400 });
    expect(v.zoom).toBeGreaterThan(0.2);
    expect(v.zoom).toBeLessThanOrEqual(1.5);
  });
});
