/*
@cypher schema=1 source_sha256=6f309a164a1777a8b9c273fd6487e47a6213b5ac9b255baa8237df7ceadaa67e
MERGE (self:File {path:"frontend/src/mocks/handlers/run-directory.test.ts"})
MERGE (handler:Symbol {id:"frontend/src/mocks/handlers/run-directory.ts::runDirectoryHandlers",kind:"constant"})
MERGE (test:Symbol {id:"frontend/src/mocks/handlers/run-directory.test.ts::run-directory-endpoint",kind:"test"})
MERGE (test)-[:TESTS]->(handler)
@endcypher
*/
// /api/run-directory 端点回归 — 走 vitest.setup 的 MSW node server 真拦截
import { describe, expect, it } from "vitest";
import { lanes, projects, worktrees } from "@/lib/seed";

describe("GET /api/run-directory (MSW 演示目录通道)", () => {
  it("返回 Project/Lane/Worktree 层级, 形状与 seed 一致", async () => {
    const res = await fetch("/api/run-directory");
    expect(res.status).toBe(200);
    const data = (await res.json()) as {
      projects: unknown[];
      lanes: unknown[];
      worktrees: unknown[];
    };
    expect(data.projects.length).toBe(projects.length);
    expect(data.lanes.length).toBe(lanes.length);
    expect(data.worktrees.length).toBe(worktrees.length);
    // 层级可连接: 每个 worktree 属于一个 lane, 每个 lane 属于一个 project
    const projectIds = new Set(projects.map((p) => p.id));
    const laneById = new Map(lanes.map((l) => [l.id, l]));
    for (const wt of worktrees) {
      if (!wt.lane_id) continue; // 无 lane 的 worktree 走「默认 LANE」降级分支
      const lane = laneById.get(wt.lane_id);
      expect(lane, `worktree ${wt.id} 的 lane 必须存在`).toBeDefined();
      expect(projectIds.has(lane!.project_id)).toBe(true);
    }
  });
});
