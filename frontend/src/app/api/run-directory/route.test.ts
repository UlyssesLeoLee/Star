/*
@cypher schema=1 source_sha256=14f158637b298a999973595f9b73ca29b610296ad98dbefd10bc80aaa5dbc94d
MERGE (self:File {path:"frontend/src/app/api/run-directory/route.test.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/run-directory/route.ts::GET",kind:"route"})
MERGE (test:Symbol {id:"frontend/src/app/api/run-directory/route.test.ts::run-directory-route",kind:"test"})
MERGE (test)-[:TESTS]->(r)
@endcypher
*/
// Next route handler 直测 (production 通道, MSW 不参与)
import { describe, expect, it } from "vitest";
import { GET } from "./route";
import { lanes, projects, worktrees } from "@/lib/seed";

describe("GET /api/run-directory (Next route, production 降级通道)", () => {
  it("返回与 seed 一致的 Project/Lane/Worktree 层级", async () => {
    const res = await GET();
    expect(res.status).toBe(200);
    const data = (await res.json()) as {
      projects: unknown[];
      lanes: unknown[];
      worktrees: unknown[];
    };
    expect(data.projects.length).toBe(projects.length);
    expect(data.lanes.length).toBe(lanes.length);
    expect(data.worktrees.length).toBe(worktrees.length);
  });
});
