/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/run/demoDirectory.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/lib/run/demoDirectory.test.ts::demo directory separation",kind:"test_suite"})
MERGE (self)-[:DEFINES]->(suite)
@endcypher
*/
// demoDirectory 演示目录独立模块回归 (P0-lane 2026-11-20)
import { describe, expect, it } from "vitest";
import {
  DEMO_RUN_DIRECTORY_SOURCE,
  demoRunDirectoryPayload,
  demoRunDirectoryPayloadIsValid,
} from "./demoDirectory";

describe("demoDirectory (演示目录独立存放)", () => {
  it("payload 引用完整: 每个 lane 属于存在的 project, 每个 worktree 指向存在的 lane", () => {
    const payload = demoRunDirectoryPayload();
    expect(demoRunDirectoryPayloadIsValid(payload)).toBe(true);
    const projectIds = new Set(payload.projects.map((project) => project.id));
    const laneIds = new Set(payload.lanes.map((lane) => lane.id));
    for (const lane of payload.lanes) expect(projectIds.has(lane.project_id)).toBe(true);
    for (const worktree of payload.worktrees) {
      expect(projectIds.has(worktree.project_id)).toBe(true);
      expect(worktree.lane_id).toBeDefined();
      expect(laneIds.has(worktree.lane_id!)).toBe(true);
    }
  });

  it("每次返回新对象, 就地改写不影响后续注入 (演示数据不被污染)", () => {
    const first = demoRunDirectoryPayload();
    const second = demoRunDirectoryPayload();
    expect(first).not.toBe(second);
    first.lanes[0].name = "mutated";
    expect(demoRunDirectoryPayload().lanes[0].name).not.toBe("mutated");
  });

  it("数据来源标记与真实 API 来源分离", () => {
    expect(DEMO_RUN_DIRECTORY_SOURCE).toBe("demo-directory");
  });

  it("非法 payload 被拒绝 (重复 id / 悬空引用)", () => {
    const payload = demoRunDirectoryPayload();
    expect(demoRunDirectoryPayloadIsValid(payload)).toBe(true);
    expect(demoRunDirectoryPayloadIsValid({ projects: [], lanes: [], worktrees: [] })).toBe(true);
    expect(demoRunDirectoryPayloadIsValid({
      projects: payload.projects,
      lanes: payload.lanes,
      worktrees: [{ ...payload.worktrees[0], lane_id: "00000000-0000-4000-8000-00000000dead" }],
    })).toBe(false);
    expect(demoRunDirectoryPayloadIsValid({
      projects: [...payload.projects, { ...payload.projects[0] }],
      lanes: payload.lanes,
      worktrees: payload.worktrees,
    })).toBe(false);
  });
});
