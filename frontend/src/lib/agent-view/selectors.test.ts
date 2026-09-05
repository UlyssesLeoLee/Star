// =====================================================================
// selectors.test.ts — Agent 视图 selector helpers
// =====================================================================
// 覆盖:
//   1. isActiveAgent — 11 个 active 状态, 3 个终态 (completed/failed/cancelled)
//   2. pickDefaultAgent — active 优先, fallback 全 started_at 倒序
//   3. resolveCurrentAgent — URL 优先, 找不到 fallback auto
//   4. pickAgentWorktree — 1:1 关联, 找不到返回 null
//   5. pickAgentWorkItems — 按 worktree_id 过滤
// =====================================================================

import { describe, it, expect } from "vitest";
import {
  isActiveAgent,
  pickDefaultAgent,
  resolveCurrentAgent,
  pickAgentWorktree,
  pickAgentWorkItems,
} from "./selectors";
import type { AgentSession, Worktree, WorkItem } from "@/types/ids";

const makeAgent = (id: string, status: AgentSession["status"], started: string, worktree_id = "wt-001"): AgentSession => ({
  id,
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  worktree_id,
  agent_kind: "claude-sonnet",
  status,
  current_step: "test",
  token_usage: { input: 0, output: 0, total: 0 },
  cost_summary: { usd: 0, budget_usd: 1 },
  started_at: started,
});

const makeWorktree = (id: string): Worktree => ({
  id,
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  name: id,
  branch: "feat/x",
  base_branch: "main",
  status: "active",
  lock_version: 0,
  last_event_at: "2026-09-05T10:00:00Z",
  created_at: "2026-09-05T09:00:00Z",
});

const makeWi = (id: string, worktreeId: string | undefined): WorkItem => ({
  id,
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  key: `PHYSIS-${id.slice(3)}`,
  title: "t",
  description: "",
  kind: "task",
  status: "todo",
  priority: "p1",
  reporter_id: "usr-001",
  labels: [],
  workflow_id: "wf-default",
  worktree_id: worktreeId,
  created_at: "2026-09-05T08:00:00Z",
  updated_at: "2026-09-05T08:00:00Z",
});

describe("isActiveAgent", () => {
  it("11 个 active 状态", () => {
    const active: AgentSession["status"][] = [
      "queued", "spawning", "initializing",
      "compiling_context", "planning", "executing",
      "awaiting_feedback", "awaiting_human", "awaiting_tool",
      "validating", "paused",
    ];
    for (const s of active) {
      expect(isActiveAgent(makeAgent("a", s, "2026-09-05T00:00:00Z"))).toBe(true);
    }
  });

  it("3 个终态不算 active", () => {
    const terminal: AgentSession["status"][] = ["completed", "failed", "cancelled"];
    for (const s of terminal) {
      expect(isActiveAgent(makeAgent("a", s, "2026-09-05T00:00:00Z"))).toBe(false);
    }
  });
});

describe("pickDefaultAgent", () => {
  it("空数组 → null", () => {
    expect(pickDefaultAgent([])).toBeNull();
  });

  it("有 active 时优先选 active 中 started_at 最新的", () => {
    const agents = [
      makeAgent("ag-old", "completed", "2026-09-01T00:00:00Z"),
      makeAgent("ag-new", "executing", "2026-09-05T00:00:00Z"),
      makeAgent("ag-mid", "completed", "2026-09-03T00:00:00Z"),
    ];
    const picked = pickDefaultAgent(agents);
    expect(picked?.id).toBe("ag-new");
  });

  it("全部终态时, fallback 到 started_at 最新的", () => {
    const agents = [
      makeAgent("ag-old", "completed", "2026-09-01T00:00:00Z"),
      makeAgent("ag-new", "completed", "2026-09-05T00:00:00Z"),
    ];
    const picked = pickDefaultAgent(agents);
    expect(picked?.id).toBe("ag-new");
  });

  it("active 但 started_at 早于某个终态, 仍然选 active (active 优先)", () => {
    const agents = [
      makeAgent("ag-old", "completed", "2026-09-05T00:00:00Z"),
      makeAgent("ag-stale", "executing", "2026-09-01T00:00:00Z"),
    ];
    const picked = pickDefaultAgent(agents);
    expect(picked?.id).toBe("ag-stale");
  });
});

describe("resolveCurrentAgent", () => {
  it("URL 给了且找到了 → auto=false", () => {
    const agents = [makeAgent("ag-001", "executing", "2026-09-05T00:00:00Z")];
    const r = resolveCurrentAgent(agents, "ag-001");
    expect(r?.auto).toBe(false);
    expect(r?.agentId).toBe("ag-001");
  });

  it("URL 给了但找不到 → auto=true (fallback 默认)", () => {
    const agents = [makeAgent("ag-001", "executing", "2026-09-05T00:00:00Z")];
    const r = resolveCurrentAgent(agents, "ag-XXX");
    expect(r?.auto).toBe(true);
    expect(r?.agentId).toBe("ag-001");
  });

  it("URL 没给 → auto=true (默认选)", () => {
    const agents = [makeAgent("ag-001", "executing", "2026-09-05T00:00:00Z")];
    const r = resolveCurrentAgent(agents, null);
    expect(r?.auto).toBe(true);
  });

  it("空 agents → null", () => {
    expect(resolveCurrentAgent([], null)).toBeNull();
  });
});

describe("pickAgentWorktree", () => {
  it("1:1 关联", () => {
    const wts = [makeWorktree("wt-001"), makeWorktree("wt-002")];
    const agent = makeAgent("ag-001", "executing", "2026-09-05T00:00:00Z", "wt-002");
    expect(pickAgentWorktree(wts, agent)?.id).toBe("wt-002");
  });

  it("找不到 → null", () => {
    const wts = [makeWorktree("wt-001")];
    const agent = makeAgent("ag-001", "executing", "2026-09-05T00:00:00Z", "wt-XXX");
    expect(pickAgentWorktree(wts, agent)).toBeNull();
  });
});

describe("pickAgentWorkItems", () => {
  it("按 worktree_id 过滤", () => {
    const wis = [
      makeWi("wi-001", "wt-001"),
      makeWi("wi-002", "wt-002"),
      makeWi("wi-003", "wt-001"),
    ];
    const wt = makeWorktree("wt-001");
    const agent = makeAgent("ag-001", "executing", "2026-09-05T00:00:00Z", "wt-001");
    const result = pickAgentWorkItems(wis, agent, wt);
    expect(result.map((w) => w.id).sort()).toEqual(["wi-001", "wi-003"]);
  });

  it("worktree 为 null → 返回空数组", () => {
    const wis = [makeWi("wi-001", "wt-001")];
    const agent = makeAgent("ag-001", "executing", "2026-09-05T00:00:00Z", "wt-001");
    expect(pickAgentWorkItems(wis, agent, null)).toEqual([]);
  });
});
