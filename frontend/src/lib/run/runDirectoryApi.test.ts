/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/lib/run/runDirectoryApi.test.ts",type:"file",language:"typescript"}),
 (runFixture:Function {name:"runFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (taskFixture:Function {name:"taskFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (pageFixture:Function {name:"pageFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (clientFixture:Function {name:"clientFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (routeTest:Function {name:"lists bounded Run-owned Task Cards with the current session",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
 (scopeTest:Function {name:"rejects Task Card pages returned for another Run",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (aliasTest:Function {name:"rejects mismatched Task Card identity aliases",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (f)-[:CONTAINS]->(runFixture),(f)-[:CONTAINS]->(taskFixture),(f)-[:CONTAINS]->(pageFixture),(f)-[:CONTAINS]->(clientFixture),(f)-[:CONTAINS]->(routeTest),(f)-[:CONTAINS]->(scopeTest),(f)-[:CONTAINS]->(aliasTest),
 (pageFixture)-[:CALLS]->(taskFixture),(clientFixture)-[:CALLS]->(pageFixture),(routeTest)-[:CALLS]->(clientFixture),(routeTest)-[:CALLS]->(pageFixture),(scopeTest)-[:CALLS]->(clientFixture),(scopeTest)-[:CALLS]->(pageFixture),(aliasTest)-[:CALLS]->(clientFixture),(aliasTest)-[:CALLS]->(pageFixture);
*/

import { describe, expect, it, vi } from "vitest";
import { RUN_TASK_CARD_PAGE_SIZE, RunDirectoryApiClient, type EngineeringRun } from "./runDirectoryApi";

const RUN_ID = "10000000-0000-4000-8000-000000000001";
const PROJECT_ID = "10000000-0000-4000-8000-000000000002";
const REPOSITORY_ID = "10000000-0000-4000-8000-000000000003";
const BRANCH_ID = "10000000-0000-4000-8000-000000000004";

function runFixture(): EngineeringRun {
  return {
    engineering_run_id: RUN_ID,
    project_id: PROJECT_ID,
    repository_id: REPOSITORY_ID,
    branch_id: BRANCH_ID,
    title: "Run",
    state: "active",
    owner_user_id: null,
    version: 1,
  };
}

function taskFixture(index: number) {
  const id = `20000000-0000-4000-8000-${String(index).padStart(12, "0")}`;
  return {
    work_item_id: id,
    task_card_id: id,
    item_type: "task",
    title: `Task ${index}`,
    description: "",
    priority: "medium",
    labels: [],
    lifecycle: { status: "pending", review_state: "not_required", active_worktree_id: null, version: 1, updated_at: "2026-10-02T00:00:00Z" },
    metadata_updated_at: "2026-10-02T00:00:00Z",
  };
}

function pageFixture(items: ReturnType<typeof taskFixture>[], runId = RUN_ID) {
  return {
    engineering_run_id: runId,
    project_id: PROJECT_ID,
    repository_id: REPOSITORY_ID,
    branch_id: BRANCH_ID,
    role: "developer",
    permission_snapshot_ref: "binding:v1",
    run_role: "developer",
    run_permission_snapshot_ref: "run-binding:v1",
    limit: RUN_TASK_CARD_PAGE_SIZE,
    next_after: items.length === RUN_TASK_CARD_PAGE_SIZE ? items[items.length - 1]?.work_item_id ?? null : null,
    work_items: items,
  };
}

function clientFixture(body: unknown) {
  const fetcher = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => new Response(JSON.stringify(body), { status: 200 }));
  const getAccessToken = vi.fn(async () => "user-jwt");
  return { client: new RunDirectoryApiClient({ getAccessToken, fetcher }), fetcher, getAccessToken };
}

describe("RunDirectoryApiClient Run Task Cards", () => {
  it("lists bounded Run-owned Task Cards with the current session", async () => {
    const items = Array.from({ length: RUN_TASK_CARD_PAGE_SIZE }, (_unused, index) => taskFixture(RUN_TASK_CARD_PAGE_SIZE - index));
    const fixture = clientFixture(pageFixture(items));

    const result = await fixture.client.listRunWorkItems(runFixture());

    expect(result.rows).toHaveLength(RUN_TASK_CARD_PAGE_SIZE);
    expect(result.next_after).toBe(items[items.length - 1]?.work_item_id);
    expect(String(fixture.fetcher.mock.calls[0]?.[0])).toBe(`/api/v1/engineering-runs/${RUN_ID}/work-items?limit=${RUN_TASK_CARD_PAGE_SIZE}`);
    expect(new Headers(fixture.fetcher.mock.calls[0]?.[1]?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(fixture.fetcher.mock.calls[0]?.[1]?.credentials).toBe("omit");
    expect(fixture.getAccessToken).toHaveBeenCalledOnce();
    fixture.client.dispose();
  });

  it("rejects Task Card pages returned for another Run", async () => {
    const fixture = clientFixture(pageFixture([], "10000000-0000-4000-8000-000000000099"));

    await expect(fixture.client.listRunWorkItems(runFixture())).rejects.toMatchObject({ code: "scope_mismatch" });
    fixture.client.dispose();
  });

  it("rejects mismatched Task Card identity aliases", async () => {
    const item = { ...taskFixture(1), task_card_id: "20000000-0000-4000-8000-000000000002" };
    const fixture = clientFixture(pageFixture([item]));

    await expect(fixture.client.listRunWorkItems(runFixture())).rejects.toMatchObject({ code: "scope_mismatch" });
    fixture.client.dispose();
  });
});
