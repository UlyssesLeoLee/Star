/*
@cypher schema=1 source_sha256=0df3f3be36ee8f9c8d373aa8b3c62c1104228c2d4fe045afa007d5e049a98ae4
MERGE (self:File {path:"frontend/src/lib/run/runDirectoryApi.test.ts"})
MERGE (create:Symbol {id:"frontend/src/lib/run/runDirectoryApi.test.ts::creates through the focused Worktree route with RunContext scope and idempotency",kind:"test"})
MERGE (scope:Symbol {id:"frontend/src/lib/run/runDirectoryApi.test.ts::rejects task creation before network when RunContext lacks the write scope",kind:"test"})
MERGE (api:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.createRunTaskCard",kind:"method"})
MERGE (self)-[:DEFINES]->(create)
MERGE (self)-[:DEFINES]->(scope)
MERGE (create)-[:TESTS]->(api)
MERGE (scope)-[:TESTS]->(api)
@endcypher
*/
/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/lib/run/runDirectoryApi.test.ts",type:"file",language:"typescript"}),
 (runFixture:Function {name:"runFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (contextFixture:Function {name:"contextFixture",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
 (taskFixture:Function {name:"taskFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (pageFixture:Function {name:"pageFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (clientFixture:Function {name:"clientFixture",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (routeTest:Function {name:"lists bounded Run-owned Task Cards with the current session",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
 (scopeTest:Function {name:"rejects Task Card pages returned for another Run",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (aliasTest:Function {name:"rejects mismatched Task Card identity aliases",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (createTest:Function {name:"creates through the focused Worktree route with RunContext scope and idempotency",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
 (createScopeTest:Function {name:"rejects a Worktree focus outside the Run before sending a create request",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
 (f)-[:CONTAINS]->(runFixture),(f)-[:CONTAINS]->(contextFixture),(f)-[:CONTAINS]->(taskFixture),(f)-[:CONTAINS]->(pageFixture),(f)-[:CONTAINS]->(clientFixture),(f)-[:CONTAINS]->(routeTest),(f)-[:CONTAINS]->(scopeTest),(f)-[:CONTAINS]->(aliasTest),(f)-[:CONTAINS]->(createTest),(f)-[:CONTAINS]->(createScopeTest),
 (contextFixture)-[:CALLS]->(runFixture),(pageFixture)-[:CALLS]->(taskFixture),(clientFixture)-[:CALLS]->(pageFixture),(routeTest)-[:CALLS]->(clientFixture),(routeTest)-[:CALLS]->(pageFixture),(scopeTest)-[:CALLS]->(clientFixture),(scopeTest)-[:CALLS]->(pageFixture),(aliasTest)-[:CALLS]->(clientFixture),(aliasTest)-[:CALLS]->(pageFixture),(createTest)-[:CALLS]->(contextFixture),(createTest)-[:CALLS]->(taskFixture),(createTest)-[:CALLS]->(clientFixture),(createScopeTest)-[:CALLS]->(contextFixture),(createScopeTest)-[:CALLS]->(clientFixture);
*/

import { describe, expect, it, vi } from "vitest";
import { RUN_TASK_CARD_PAGE_SIZE, RunDirectoryApiClient, type CreateRunTaskCardInput, type EngineeringRun, type RunContextEnvelope } from "./runDirectoryApi";

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

function contextFixture(): RunContextEnvelope {
  const run = runFixture();
  return {
    run_context: {
      tenant_id: "10000000-0000-4000-8000-000000000005", project_id: PROJECT_ID,
      repository_id: REPOSITORY_ID, branch_id: BRANCH_ID, engineering_run_id: RUN_ID,
      actor_id: "10000000-0000-4000-8000-000000000006", actor_kind: "user",
      authorization: {
        project: { binding_id: "40000000-0000-4000-8000-000000000001", role: "developer", version: 1 },
        branch: { binding_id: "40000000-0000-4000-8000-000000000002", role: "developer", version: 1 },
        engineering_run: { binding_id: "40000000-0000-4000-8000-000000000003", role: "developer", version: 1 },
      },
      granted_scopes: ["work-item:write"], context_version: "ctx:v1",
      resolved_at: "2026-10-02T00:00:00Z", correlation_id: "10000000-0000-4000-8000-000000000007",
    },
    branch: {
      branch_id: BRANCH_ID, project_id: PROJECT_ID, repository_id: REPOSITORY_ID,
      name: "feature/run-task", full_ref: "refs/heads/feature/run-task", head_commit_id: null, state: "active", version: 1,
    },
    engineering_run: run,
    focus: {
      worktree_id: "30000000-0000-4000-8000-000000000001", engineering_run_id: RUN_ID,
      project_id: PROJECT_ID, repository_id: REPOSITORY_ID, workspace_id: "30000000-0000-4000-8000-000000000002",
      name: "run-focus", branch: "feature/run-task", human_state: "active", machine_state: "ready",
      agent_id: null, agent_session_id: null, runtime_id: null, archived: false, version: 1,
      binding_id: "30000000-0000-4000-8000-000000000003", binding_version: 1,
      project_binding_id: "30000000-0000-4000-8000-000000000004", project_binding_version: 1,
    },
    focus_version: "focus:v1", capabilities: { run_owned_apps_available: true, execution_admission_available: false },
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

  it("creates through the focused Worktree route with RunContext scope and idempotency", async () => {
    const input: CreateRunTaskCardInput = {
      title: "Create via Run", description: "Bounded body", priority: "high",
      correlation_id: "10000000-0000-4000-8000-000000000008",
    };
    const task = taskFixture(9);
    const fixture = clientFixture({ ...task, task_card_id: task.work_item_id, title: input.title });

    const result = await fixture.client.createRunTaskCard(contextFixture(), input, "create-run-task-1");

    const [url, init] = fixture.fetcher.mock.calls[0] ?? [];
    expect(String(url)).toBe("/api/v1/worktrees/30000000-0000-4000-8000-000000000001/work-items");
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(new Headers(init?.headers).get("Idempotency-Key")).toBe("create-run-task-1");
    expect(init?.credentials).toBe("omit");
    expect(JSON.parse(String(init?.body))).toEqual({
      item_type: "task", title: input.title, description: input.description, priority: input.priority,
      labels: [], correlation_id: input.correlation_id,
    });
    expect(result.title).toBe(input.title);
    fixture.client.dispose();
  });

  it("rejects a Worktree focus outside the Run before sending a create request", async () => {
    const fixture = clientFixture(taskFixture(9));
    const context = contextFixture();
    const invalidContext = {
      ...context,
      focus: { ...context.focus!, engineering_run_id: "10000000-0000-4000-8000-000000000099" },
    };

    await expect(fixture.client.createRunTaskCard(invalidContext, {
      title: "Out of scope", description: "", priority: "medium",
      correlation_id: "10000000-0000-4000-8000-000000000008",
    }, "create-run-task-2")).rejects.toMatchObject({ code: "scope_mismatch" });
    expect(fixture.fetcher).not.toHaveBeenCalled();
    fixture.client.dispose();
  });

  it("rejects task creation before network when RunContext lacks the write scope", async () => {
    const fixture = clientFixture(taskFixture(10));
    const context = contextFixture();
    context.run_context.granted_scopes = ["work-item:read"];
    await expect(fixture.client.createRunTaskCard(context, {
      title: "Denied", description: "", priority: "medium",
      correlation_id: "10000000-0000-4000-8000-000000000008",
    }, "create-run-task-denied")).rejects.toMatchObject({ code: "insufficient_scope" });
    expect(fixture.fetcher).not.toHaveBeenCalled();
    fixture.client.dispose();
  });
});
