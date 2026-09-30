/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts",type:"file",language:"typescript"}),
  (suite:Function {name:"worktreeGroupApi tests",type:"function",signature:"describe('WorktreeGroupApiClient', ...)",visibility:"private",complexity:"moderate"}),
  (missingToken:Function {name:"missing token case",type:"function",signature:"it('fails closed without a user token', ...)",visibility:"private",complexity:"simple"}),
  (authorization:Function {name:"authorization case",type:"function",signature:"it('requests a fresh Bearer token and encodes Worktree path', ...)",visibility:"private",complexity:"moderate"}),
  (indexCase:Function {name:"index query case",type:"function",signature:"it('sends Worktree Index filters to the project route', ...)",visibility:"private",complexity:"moderate"}),
  (planCase:Function {name:"management plan case",type:"function",signature:"it('sends an idempotent Worktree management plan', ...)",visibility:"private",complexity:"moderate"}),
  (confirmCase:Function {name:"management confirm case",type:"function",signature:"it('confirms a Worktree management plan at the scoped route', ...)",visibility:"private",complexity:"moderate"}),
  (cursor:Function {name:"cursor case",type:"function",signature:"it('sends both event cursor components', ...)",visibility:"private",complexity:"moderate"}),
  (pollerCase:Function {name:"Outbox poller retry case",type:"function",signature:"it('replays events until authorized projection refresh succeeds', ...)",visibility:"private",complexity:"complex"}),
  (canvasTaskCase:Function {name:"Canvas Task Card creation case",type:"function",signature:"it('creates a canonical WorkItem through the scoped Canvas command', ...)",visibility:"private",complexity:"moderate"}),
  (createCanvasCase:Function {name:"Worktree Canvas bootstrap case",type:"function",signature:"it('creates a Canvas through the authenticated Worktree route', ...)",visibility:"private",complexity:"moderate"}),
  (canvasElementCase:Function {name:"Canvas Element link case",type:"function",signature:"it('creates a scoped Canvas element with an idempotent entity link', ...)",visibility:"private",complexity:"moderate"}),
  (documentCase:Function {name:"Canvas Document CAS case",type:"function",signature:"it('writes a Canvas Document through the authenticated CAS route', ...)",visibility:"private",complexity:"moderate"}),
  (error:Function {name:"error case",type:"function",signature:"it('preserves API error status and code', ...)",visibility:"private",complexity:"simple"}),
  (freshToken:Function {name:"fresh token case",type:"function",signature:"it('resolves the token provider on each request', ...)",visibility:"private",complexity:"moderate"}),
  (https:Function {name:"https case",type:"function",signature:"it('rejects non-HTTPS remote origins', ...)",visibility:"private",complexity:"simple"}),
  (makeClient:Function {name:"makeClient",type:"function",signature:"makeClient(token: string | null, response?: Response)",visibility:"private",complexity:"simple"}),
  (tokenProvider:Function {name:"token provider callback",type:"function",signature:"async () => token",visibility:"private",complexity:"simple"}),
  (apiCtor:Function {name:"WorktreeGroupApiClient.constructor",type:"function",signature:"new WorktreeGroupApiClient(getAccessToken, fetcher)",visibility:"public",complexity:"simple"}),
  (groupContext:Function {name:"WorktreeGroupApiClient.getGroupContext",type:"function",signature:"getGroupContext(worktreeId)",visibility:"public",complexity:"simple"}),
  (index:Function {name:"WorktreeGroupApiClient.listProjectWorktrees",type:"function",signature:"listProjectWorktrees(projectId, filters)",visibility:"public",complexity:"moderate"}),
  (managementPlan:Function {name:"WorktreeGroupApiClient.createManagementPlan",type:"function",signature:"createManagementPlan(worktreeId, body, idempotencyKey)",visibility:"public",complexity:"simple"}),
  (managementConfirm:Function {name:"WorktreeGroupApiClient.confirmManagementPlan",type:"function",signature:"confirmManagementPlan(worktreeId, planId)",visibility:"public",complexity:"simple"}),
  (updateCanvasDocument:Function {name:"WorktreeGroupApiClient.updateCanvasDocument",type:"function",signature:"updateCanvasDocument(worktreeId, canvasId, body, idempotencyKey)",visibility:"public",complexity:"simple"}),
  (createCanvasElement:Function {name:"WorktreeGroupApiClient.createCanvasElement",type:"function",signature:"createCanvasElement(worktreeId, canvasId, body, idempotencyKey)",visibility:"public",complexity:"simple"}),
  (events:Function {name:"WorktreeGroupApiClient.listCanvasEvents",type:"function",signature:"listCanvasEvents(worktreeId, canvasId, cursor)",visibility:"public",complexity:"moderate"}),
  (poller:Class {name:"CanvasOutboxPoller",type:"class",visibility:"public"}),
  (pollOnce:Function {name:"CanvasOutboxPoller.pollOnce",type:"function",signature:"pollOnce()",visibility:"public",complexity:"moderate"}),
  (refreshCallback:Function {name:"projection refresh callback",type:"function",signature:"() => Promise<void>",visibility:"private",complexity:"simple"}),
  (apiError:Class {name:"GroupApiError",type:"class",visibility:"public"}),
  (file)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(missingToken),(suite)-[:CONTAINS]->(authorization),(suite)-[:CONTAINS]->(indexCase),(suite)-[:CONTAINS]->(planCase),(suite)-[:CONTAINS]->(confirmCase),(suite)-[:CONTAINS]->(cursor),(suite)-[:CONTAINS]->(pollerCase),(suite)-[:CONTAINS]->(canvasTaskCase),(suite)-[:CONTAINS]->(createCanvasCase),(suite)-[:CONTAINS]->(canvasElementCase),(suite)-[:CONTAINS]->(documentCase),(suite)-[:CONTAINS]->(error),(suite)-[:CONTAINS]->(freshToken),(suite)-[:CONTAINS]->(https),
  (makeClient)-[:CONTAINS]->(tokenProvider),(makeClient)-[:CALLS]->(apiCtor),
  (missingToken)-[:CALLS]->(makeClient),(authorization)-[:CALLS]->(makeClient),(authorization)-[:CALLS]->(groupContext),(indexCase)-[:CALLS]->(makeClient),(indexCase)-[:CALLS]->(index),(planCase)-[:CALLS]->(makeClient),(planCase)-[:CALLS]->(managementPlan),(confirmCase)-[:CALLS]->(makeClient),(confirmCase)-[:CALLS]->(managementConfirm),(cursor)-[:CALLS]->(makeClient),(cursor)-[:CALLS]->(events),(pollerCase)-[:CALLS]->(apiCtor),(pollerCase)-[:CALLS]->(poller),(pollerCase)-[:CALLS]->(pollOnce),(pollerCase)-[:USES]->(refreshCallback),(canvasElementCase)-[:CALLS]->(makeClient),(canvasElementCase)-[:CALLS]->(createCanvasElement),(documentCase)-[:CALLS]->(makeClient),(documentCase)-[:CALLS]->(updateCanvasDocument),(error)-[:CALLS]->(makeClient),(error)-[:CALLS]->(groupContext),(error)-[:USES]->(apiError),(freshToken)-[:CALLS]->(apiCtor),(freshToken)-[:CALLS]->(groupContext),(https)-[:CALLS]->(apiCtor),(https)-[:USES]->(apiError);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"worktreeGroupApi tests"}),
      (makeClient:Function {name:"makeClient"}),
      (status:Function {name:"WorktreeGroupApiClient.getTaskCliSessionStatus"}),
      (cancel:Function {name:"WorktreeGroupApiClient.cancelTaskCliSession"}),
      (reattach:Function {name:"WorktreeGroupApiClient.reattachTaskCliSession"}),
      (list:Function {name:"WorktreeGroupApiClient.listTaskCliSessions"});
CREATE (cliLifecycleCase:Function {name:"Task CLI lifecycle API case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (suite)-[:CONTAINS]->(cliLifecycleCase),
       (cliLifecycleCase)-[:CALLS]->(makeClient),
       (cliLifecycleCase)-[:CALLS]->(status),
       (cliLifecycleCase)-[:CALLS]->(cancel),
       (cliLifecycleCase)-[:CALLS]->(reattach),
       (cliLifecycleCase)-[:CALLS]->(list);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts"}),
      (normalizeGitLock:Function {name:"normalizeWorktreeGitLock"});
CREATE (gitLockSuite:Function {name:"Worktree Git lock observation tests",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
       (gitLockFreshCase:Function {name:"fresh Git lock observation case",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
       (gitLockFailClosedCase:Function {name:"invalid Git lock observation case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (file)-[:CONTAINS]->(gitLockSuite),
       (gitLockSuite)-[:CONTAINS]->(gitLockFreshCase),(gitLockSuite)-[:CONTAINS]->(gitLockFailClosedCase),
       (gitLockFreshCase)-[:CALLS]->(normalizeGitLock),(gitLockFailClosedCase)-[:CALLS]->(normalizeGitLock);
*/

import { describe, expect, it, vi } from "vitest";

import { CanvasOutboxPoller, GroupApiError, normalizeWorktreeGitLock, WorktreeGroupApiClient } from "../worktreeGroupApi";

function makeClient(token: string | null, response = new Response(JSON.stringify({ ok: true }), { status: 200 })) {
  const fetcher = vi.fn<typeof fetch>().mockResolvedValue(response);
  const api = new WorktreeGroupApiClient(async () => token, fetcher);
  return { api, fetcher };
}

describe("Worktree Git lock observation", () => {
  const now = Date.parse("2026-09-30T00:00:30.000Z");

  it("preserves a fresh host runtime locked or unlocked observation", () => {
    for (const state of ["locked", "unlocked"] as const) {
      expect(normalizeWorktreeGitLock({
        state,
        source: "host_runtime",
        observed_at: "2026-09-30T00:00:00.000Z",
      }, now)).toEqual({ state, observedAt: "2026-09-30T00:00:00.000Z" });
    }
  });

  it("fails closed for unavailable, invalid, stale, and future observations", () => {
    const fresh = { state: "locked", source: "host_runtime", observed_at: "2026-09-30T00:00:00.000Z" };
    const invalid = [
      { ...fresh, source: "unavailable" },
      { ...fresh, observed_at: "not-a-time" },
      { ...fresh, observed_at: "2026-09-29T23:59:59.999Z" },
      { ...fresh, observed_at: "2026-09-30T00:00:31.000Z" },
      { state: "locked", source: "host_runtime" },
      null,
    ];

    for (const observation of invalid) {
      expect(normalizeWorktreeGitLock(observation, now).state).toBe("unknown");
    }
  });
});

describe("WorktreeGroupApiClient", () => {
  it("fails closed without a user token", async () => {
    const { api, fetcher } = makeClient(null);

    await expect(api.getGroupContext("wt-1")).rejects.toMatchObject({
      status: 401,
      code: "session_required",
    });
    expect(fetcher).not.toHaveBeenCalled();
  });

  it("requests a fresh Bearer token and encodes Worktree path", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    await api.getGroupContext("wt one");

    const [input, init] = fetcher.mock.calls[0];
    expect(input).toBe("/api/v1/worktrees/wt%20one/group-context");
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(init?.credentials).toBe("omit");
    expect(init?.cache).toBe("no-store");
  });

  it("sends Worktree Index filters to the project route", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    await api.listProjectWorktrees("project one", {
      limit: 25,
      cursor: "cursor-token",
      owner_user_id: "owner-1",
      human_state: "active",
      include_archived: true,
    });

    const [input] = fetcher.mock.calls[0];
    const url = new URL(String(input), "http://localhost");
    expect(url.pathname).toBe("/api/v1/projects/project%20one/worktrees");
    expect(Object.fromEntries(url.searchParams.entries())).toEqual({
      limit: "25",
      cursor: "cursor-token",
      owner_user_id: "owner-1",
      human_state: "active",
      include_archived: "true",
    });
  });

  it("lists only server-authorized Projects with a bounded cursor query", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    await api.listAuthorizedProjects({ limit: 80, cursor: "22222222-2222-4222-8222-222222222222" });

    const [input, init] = fetcher.mock.calls[0];
    const url = new URL(String(input), "http://localhost");
    expect(url.pathname).toBe("/api/v1/projects");
    expect(Object.fromEntries(url.searchParams.entries())).toEqual({
      limit: "80",
      cursor: "22222222-2222-4222-8222-222222222222",
    });
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(init?.cache).toBe("no-store");
  });

  it("keeps Hook policy reads, draft writes, publish, and rollback on scoped authenticated routes", async () => {
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async () =>
      new Response(JSON.stringify({ ok: true }), { status: 200 }),
    );
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
    const draft = {
      expected_draft_version: 0,
      expected_current_policy_set_id: null,
      correlation_id: "correlation-1",
      policy_document: { schema_version: 1 },
    };
    const publish = { expected_draft_version: 1, correlation_id: "correlation-2" };
    const rollback = {
      target_policy_set_id: "policy-old",
      expected_current_policy_set_id: "policy-current",
      correlation_id: "correlation-3",
    };

    await api.listAuthorizedProjects({ limit: 200 });
    await api.getProjectHookPolicy("project one");
    await api.getWorktreeHookPolicy("worktree one");
    await api.saveProjectHookDraft("project one", draft);
    await api.saveWorktreeHookDraft("worktree one", draft);
    await api.publishProjectHookDraft("project one", publish);
    await api.publishWorktreeHookDraft("worktree one", publish);
    await api.rollbackProjectHookPolicy("project one", rollback);
    await api.rollbackWorktreeHookPolicy("worktree one", rollback);

    expect(fetcher.mock.calls.map(([input, init]) => [
      new URL(String(input), "http://localhost").pathname,
      init?.method ?? "GET",
    ])).toEqual([
      ["/api/v1/projects", "GET"],
      ["/api/v1/projects/project%20one/hook-policy", "GET"],
      ["/api/v1/worktrees/worktree%20one/hook-policy/effective", "GET"],
      ["/api/v1/projects/project%20one/hook-policy/draft", "PUT"],
      ["/api/v1/worktrees/worktree%20one/hook-policy/draft", "PUT"],
      ["/api/v1/projects/project%20one/hook-policy/publish", "POST"],
      ["/api/v1/worktrees/worktree%20one/hook-policy/publish", "POST"],
      ["/api/v1/projects/project%20one/hook-policy/rollback", "POST"],
      ["/api/v1/worktrees/worktree%20one/hook-policy/rollback", "POST"],
    ]);
    expect(JSON.parse(String(fetcher.mock.calls[3][1]?.body))).toEqual(draft);
    expect(JSON.parse(String(fetcher.mock.calls[5][1]?.body))).toEqual(publish);
    expect(JSON.parse(String(fetcher.mock.calls[7][1]?.body))).toEqual(rollback);
    expect(fetcher.mock.calls.every(([, init]) =>
      new Headers(init?.headers).get("Authorization") === "Bearer user-jwt"
      && init?.credentials === "omit"
      && init?.cache === "no-store",
    )).toBe(true);
  });

  it("uses Project-scoped routes for repository discovery, candidate listing, create, and import", async () => {
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async () =>
      new Response(JSON.stringify({ ok: true }), { status: 200 }),
    );
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
    const createBody = { repository_id: "repo-1", branch: "feature/one", base_ref: "main", correlation_id: "corr-1" };
    const importBody = { repository_id: "repo-1", candidate_id: "candidate-1", correlation_id: "corr-2" };

    await api.listProjectWorktreeRepositories("project one");
    await api.listWorktreeImportCandidates("project one", "repo one", 12);
    await api.createProjectWorktree("project one", createBody, "idem-create");
    await api.importProjectWorktree("project one", importBody, "idem-import");

    expect(fetcher).toHaveBeenCalledTimes(4);
    expect(fetcher.mock.calls.map(([input, init]) => [String(input), init?.method ?? "GET"])).toEqual([
      ["/api/v1/projects/project%20one/worktree-repositories", "GET"],
      ["/api/v1/projects/project%20one/worktree-import-candidates?repository_id=repo+one&limit=12", "GET"],
      ["/api/v1/projects/project%20one/worktrees", "POST"],
      ["/api/v1/projects/project%20one/worktrees/import", "POST"],
    ]);
    expect(new Headers(fetcher.mock.calls[2][1]?.headers).get("Idempotency-Key")).toBe("idem-create");
    expect(new Headers(fetcher.mock.calls[3][1]?.headers).get("Idempotency-Key")).toBe("idem-import");
    expect(JSON.parse(String(fetcher.mock.calls[2][1]?.body))).toEqual(createBody);
    expect(JSON.parse(String(fetcher.mock.calls[3][1]?.body))).toEqual(importBody);
    expect(fetcher.mock.calls.every(([, init]) => init?.credentials === "omit" && init?.cache === "no-store")).toBe(true);
  });

  it("sends an idempotent Worktree management plan", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    await api.createManagementPlan("wt-1", { operation: "owner_change", expected_version: 4 }, "idem-1");

    const [input, init] = fetcher.mock.calls[0];
    expect(input).toBe("/api/v1/worktrees/wt-1/management-plans");
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("Idempotency-Key")).toBe("idem-1");
    expect(JSON.parse(String(init?.body))).toEqual({ operation: "owner_change", expected_version: 4 });
  });

  it("confirms a Worktree management plan at the scoped route", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    await api.confirmManagementPlan("wt-1", "plan-1");

    const [input, init] = fetcher.mock.calls[0];
    expect(input).toBe("/api/v1/worktrees/wt-1/management-plans/plan-1/confirm");
    expect(init?.method).toBe("POST");
  });

  it("uses scoped listing, status, cancel, and fresh-ticket reattach routes for Task CLI", async () => {
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async () =>
      new Response(JSON.stringify({ ok: true }), { status: 200 }),
    );
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);

    await api.listTaskCliSessions("wt one", "wi one", "corr-list", 12);
    await api.getTaskCliSessionStatus("wt one", "wi one", "session one", "corr-status");
    await api.cancelTaskCliSession("wt one", "wi one", "session one", "corr-cancel");
    await api.reattachTaskCliSession("wt one", "wi one", "session one", "corr-reattach");

    expect(fetcher).toHaveBeenCalledTimes(4);
    expect(fetcher.mock.calls.map(([input, init]) => [String(input), init?.method ?? "GET"])).toEqual([
      ["/api/v1/worktrees/wt%20one/work-items/wi%20one/cli-sessions?limit=12", "GET"],
      ["/api/v1/worktrees/wt%20one/work-items/wi%20one/cli-sessions/session%20one", "GET"],
      ["/api/v1/worktrees/wt%20one/work-items/wi%20one/cli-sessions/session%20one", "DELETE"],
      ["/api/v1/worktrees/wt%20one/work-items/wi%20one/cli-sessions/session%20one/attachment-tickets", "POST"],
    ]);
    expect(fetcher.mock.calls.map(([, init]) => new Headers(init?.headers).get("X-Correlation-ID"))).toEqual([
      "corr-list",
      "corr-status",
      "corr-cancel",
      "corr-reattach",
    ]);
    expect(fetcher.mock.calls.every(([, init]) => init?.cache === "no-store")).toBe(true);
  });

  it("creates a canonical WorkItem through the scoped Canvas command", async () => {
    const { api, fetcher } = makeClient("user-jwt");
    const body = {
      item_type: "task",
      title: "Investigate Worktree drift",
      description: "Compare the agent checkout with the index.",
      priority: "medium",
      labels: [],
      ai_task_data: null,
      x: 300,
      y: 200,
      width: 240,
      height: 120,
      correlation_id: "correlation-1",
    };

    await api.createWorkItemOnCanvas("wt-1", "canvas-1", body, "idem-1");

    const [input, init] = fetcher.mock.calls[0];
    expect(input).toBe("/api/v1/worktrees/wt-1/canvases/canvas-1/work-items");
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(new Headers(init?.headers).get("Idempotency-Key")).toBe("idem-1");
    expect(JSON.parse(String(init?.body))).toEqual(body);
  });

  it("creates a Canvas through the authenticated Worktree route", async () => {
    const { api, fetcher } = makeClient("user-jwt");
    const body = { title: "Worktree Canvas", correlation_id: "correlation-2" };

    await api.createCanvas("wt-1", body, "idem-canvas-1");

    const [input, init] = fetcher.mock.calls[0];
    expect(input).toBe("/api/v1/worktrees/wt-1/canvases");
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(new Headers(init?.headers).get("Idempotency-Key")).toBe("idem-canvas-1");
    expect(JSON.parse(String(init?.body))).toEqual(body);
  });

  it("creates a scoped Canvas element with an idempotent entity link", async () => {
    const { api, fetcher } = makeClient("user-jwt");
    const body = {
      kind: "work_item_card",
      x: 120,
      y: 80,
      width: 240,
      height: 120,
      rotation: 0,
      z_index: 4,
      content: {},
      entity_ref: { ref_type: "work_item", ref_id: "wi-1", worktree_id: "wt-1" },
      correlation_id: "correlation-4",
    };

    await api.createCanvasElement("wt-1", "canvas-1", body, "idem-element-1");

    const [input, init] = fetcher.mock.calls[0];
    expect(input).toBe("/api/v1/worktrees/wt-1/canvases/canvas-1/elements");
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(new Headers(init?.headers).get("Idempotency-Key")).toBe("idem-element-1");
    expect(JSON.parse(String(init?.body))).toEqual(body);
  });

  it("writes a Canvas Document through the authenticated CAS route", async () => {
    const { api, fetcher } = makeClient("user-jwt");
    const body = {
      expected_version: 3,
      viewport: { x: 40, y: 20, zoom: 1.2 },
      frames: [],
      connectors: [],
      correlation_id: "correlation-3",
    };

    await api.updateCanvasDocument("wt-1", "canvas-1", body, "idem-document-1");

    const [input, init] = fetcher.mock.calls[0];
    expect(input).toBe("/api/v1/worktrees/wt-1/canvases/canvas-1/document");
    expect(init?.method).toBe("PUT");
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(new Headers(init?.headers).get("Idempotency-Key")).toBe("idem-document-1");
    expect(JSON.parse(String(init?.body))).toEqual(body);
  });

  it("sends both event cursor components", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    await api.listCanvasEvents("wt-1", "canvas-1", {
      cursor_at: "2026-09-29T10:00:00Z",
      cursor_event_id: "event-1",
    });

    const [input] = fetcher.mock.calls[0];
    const url = new URL(String(input), "http://localhost");
    expect(url.searchParams.get("cursor_at")).toBe("2026-09-29T10:00:00Z");
    expect(url.searchParams.get("cursor_event_id")).toBe("event-1");
  });

  it("lists Task Runs using an encoded Task scope and bounded cursor", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    await api.listTaskRuns("wt one", "task/one", {
      limit: 20,
      cursor: {
        cursor_started_at: "2026-09-29T10:00:00Z",
        cursor_run_id: "run-1",
      },
    });

    const [input, init] = fetcher.mock.calls[0];
    const url = new URL(String(input), "http://localhost");
    expect(url.pathname).toBe("/api/v1/worktrees/wt%20one/work-items/task%2Fone/runs");
    expect(url.searchParams.get("limit")).toBe("20");
    expect(url.searchParams.get("cursor_started_at")).toBe("2026-09-29T10:00:00Z");
    expect(url.searchParams.get("cursor_run_id")).toBe("run-1");
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(init?.cache).toBe("no-store");
  });

  it("rejects an out-of-range Task Run page limit before fetching", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    expect(() => api.listTaskRuns("wt-1", "task-1", { limit: 51 })).toThrow(GroupApiError);
    expect(fetcher).not.toHaveBeenCalled();
  });

  it("replays events until authorized projection refresh succeeds", async () => {
    const page = JSON.stringify({
      events: [{ event_id: "event-1", occurred_at: "2026-09-29T10:00:00Z" }],
      next_cursor: { cursor_at: "2026-09-29T10:00:00Z", cursor_event_id: "event-1" },
    });
    const fetcher = vi.fn<typeof fetch>()
      .mockResolvedValueOnce(new Response(page, { status: 200 }))
      .mockResolvedValueOnce(new Response(page, { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ events: [], next_cursor: null }), { status: 200 }));
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
    const refreshProjection = vi.fn<() => Promise<void>>()
      .mockRejectedValueOnce(new Error("projection refresh failed"))
      .mockResolvedValue(undefined);
    const poller = new CanvasOutboxPoller(api, "wt-1", "canvas-1", refreshProjection);

    await expect(poller.pollOnce()).rejects.toThrow("projection refresh failed");
    await expect(poller.pollOnce()).resolves.toBe(1);
    await expect(poller.pollOnce()).resolves.toBe(0);

    const urls = fetcher.mock.calls.map(([input]) => new URL(String(input), "http://localhost"));
    expect(urls[0].searchParams.has("cursor_at")).toBe(false);
    expect(urls[1].searchParams.has("cursor_at")).toBe(false);
    expect(urls[2].searchParams.get("cursor_at")).toBe("2026-09-29T10:00:00Z");
    expect(urls[2].searchParams.get("cursor_event_id")).toBe("event-1");
    expect(refreshProjection).toHaveBeenCalledTimes(2);
  });

  it("preserves API error status and code", async () => {
    const authenticatedApi = new WorktreeGroupApiClient(
      async () => "user-jwt",
      vi.fn<typeof fetch>().mockResolvedValue(new Response(JSON.stringify({ error: { code: "forbidden" } }), { status: 403 })),
    );

    const request = authenticatedApi.getGroupContext("wt-1");
    await expect(request).rejects.toBeInstanceOf(GroupApiError);
    await expect(request).rejects.toMatchObject({
      status: 403,
      code: "forbidden",
    });
  });

  it("resolves the token provider on each request", async () => {
    const getAccessToken = vi.fn<() => Promise<string | null>>()
      .mockResolvedValueOnce("user-jwt-1")
      .mockResolvedValueOnce("user-jwt-2");
    const fetcher = vi.fn<typeof fetch>()
      .mockResolvedValueOnce(new Response(JSON.stringify({ ok: true }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ ok: true }), { status: 200 }));
    const api = new WorktreeGroupApiClient(getAccessToken, fetcher);

    await api.getGroupContext("wt-1");
    await api.getGroupContext("wt-1");

    expect(getAccessToken).toHaveBeenCalledTimes(2);
    expect(fetcher.mock.calls.map(([, init]) => new Headers(init?.headers).get("Authorization"))).toEqual([
      "Bearer user-jwt-1",
      "Bearer user-jwt-2",
    ]);
  });

  it("rejects non-HTTPS remote origins", () => {
    expect(() => new WorktreeGroupApiClient(async () => "jwt", fetch, "http://api.example.test"))
      .toThrow(GroupApiError);
    expect(() => new WorktreeGroupApiClient(async () => "jwt", fetch, "http://localhost:8080"))
      .not.toThrow();
  });
});

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"worktreeGroupApi tests"}),
      (api:Class {name:"WorktreeGroupApiClient"});
CREATE (lifecycleRoutes:Function {name:"Project Worktree lifecycle route case",type:"function",visibility:"private",complexity:"moderate"});
CREATE (suite)-[:CONTAINS]->(lifecycleRoutes),
       (lifecycleRoutes)-[:CALLS]->(api);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"worktreeGroupApi tests"}),
      (makeClient:Function {name:"makeClient"}),
      (listRuns:Function {name:"WorktreeGroupApiClient.listTaskRuns"});
CREATE (taskRunListCase:Function {name:"Task Run list API case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
       (taskRunLimitCase:Function {name:"Task Run bounded page limit case",type:"function",language:"typescript",visibility:"private",complexity:"simple"});
CREATE (suite)-[:CONTAINS]->(taskRunListCase),(suite)-[:CONTAINS]->(taskRunLimitCase),
       (taskRunListCase)-[:CALLS]->(makeClient),(taskRunListCase)-[:CALLS]->(listRuns),
       (taskRunLimitCase)-[:CALLS]->(makeClient),(taskRunLimitCase)-[:CALLS]->(listRuns);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"worktreeGroupApi tests"}),
      (api:Class {name:"WorktreeGroupApiClient"});
CREATE (hookRoutes:Function {name:"Hook policy scoped routes case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (suite)-[:CONTAINS]->(hookRoutes),(hookRoutes)-[:CALLS]->(api);
*/
