/*
@cypher schema=1 source_sha256=41dd226ce4791b959581581ce6336dc149db4b38a23e406ecdcd15f04e18e1ca
MERGE (self:File {path:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts::worktreeGroupApi tests",kind:"suite"})
MERGE (selectedCanvasTest:Symbol {id:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts::keeps one Worktree stream across Strict Mode cleanup and selected Canvas changes",kind:"test"})
MERGE (timerAbortTest:Symbol {id:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts::cancels stream backoff and failed catch-up retry timers when the Worktree scope leaves",kind:"test"})
MERGE (abortAuthTest:Symbol {id:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts::does not issue or invalidate an authorized read when its scope aborts during token resolution",kind:"test"})
MERGE (projection:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::useWorktreeGroupProjection",kind:"function"})
MERGE (provider:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::WorktreeGroupApiProvider",kind:"function"})
MERGE (sessionInvalidationTest:Symbol {id:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts::invalidates host session for 401 and preserves 403 case",kind:"test"})
MERGE (invalidateAuthorization:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::WorktreeGroupApiClient.invalidateAuthorization",kind:"method"})
MERGE (list:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::WorktreeGroupApiClient.listWorktreeCanvasEvents",kind:"method"})
MERGE (catchUp:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::CanvasOutboxCatchUp.catchUp",kind:"method"})
MERGE (streamTest:Symbol {id:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts::Worktree Canvas stream API case",kind:"test"})
MERGE (malformedTest:Symbol {id:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts::malformed Worktree Canvas page case",kind:"test"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:DEFINES]->(sessionInvalidationTest)
MERGE (suite)-[:DEFINES]->(streamTest)
MERGE (suite)-[:DEFINES]->(malformedTest)
MERGE (suite)-[:DEFINES]->(selectedCanvasTest)
MERGE (suite)-[:DEFINES]->(timerAbortTest)
MERGE (suite)-[:DEFINES]->(abortAuthTest)
MERGE (abortAuthTest)-[:TESTS]->(list)
MERGE (timerAbortTest)-[:TESTS]->(catchUp)
MERGE (selectedCanvasTest)-[:CALLS]->(provider)
MERGE (selectedCanvasTest)-[:CALLS]->(projection)
MERGE (streamTest)-[:CALLS]->(list)
MERGE (malformedTest)-[:CALLS]->(catchUp)
MERGE (streamTest)-[:TESTS]->(list)
MERGE (malformedTest)-[:TESTS]->(catchUp)
MERGE (taskRunListTest:Symbol {id:"frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts::Task Run list cursor case",kind:"test"})
MERGE (runList:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::WorktreeGroupApiClient.listTaskRuns",kind:"method"})
MERGE (suite)-[:DEFINES]->(taskRunListTest)
MERGE (taskRunListTest)-[:CALLS]->(runList)
MERGE (taskRunListTest)-[:TESTS]->(runList)
MERGE (sessionInvalidationTest)-[:TESTS]->(invalidateAuthorization)
MERGE (suite)-[:DEFINES]->(sessionInvalidationTest)
@endcypher
*/
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
  (catchUpCase:Function {name:"Outbox catch-up retry case",type:"function",signature:"it('replays events until authorized projection refresh succeeds', ...)",visibility:"private",complexity:"complex"}),
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
  (catchUp:Class {name:"CanvasOutboxCatchUp",type:"class",visibility:"public"}),
  (catchUpRun:Function {name:"CanvasOutboxCatchUp.catchUp",type:"function",signature:"catchUp(signal)",visibility:"public",complexity:"complex"}),
  (refreshCallback:Function {name:"projection refresh callback",type:"function",signature:"() => Promise<void>",visibility:"private",complexity:"simple"}),
  (apiError:Class {name:"GroupApiError",type:"class",visibility:"public"}),
  (file)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(missingToken),(suite)-[:CONTAINS]->(authorization),(suite)-[:CONTAINS]->(indexCase),(suite)-[:CONTAINS]->(planCase),(suite)-[:CONTAINS]->(confirmCase),(suite)-[:CONTAINS]->(cursor),(suite)-[:CONTAINS]->(catchUpCase),(suite)-[:CONTAINS]->(canvasTaskCase),(suite)-[:CONTAINS]->(createCanvasCase),(suite)-[:CONTAINS]->(canvasElementCase),(suite)-[:CONTAINS]->(documentCase),(suite)-[:CONTAINS]->(error),(suite)-[:CONTAINS]->(freshToken),(suite)-[:CONTAINS]->(https),
  (makeClient)-[:CONTAINS]->(tokenProvider),(makeClient)-[:CALLS]->(apiCtor),
  (missingToken)-[:CALLS]->(makeClient),(authorization)-[:CALLS]->(makeClient),(authorization)-[:CALLS]->(groupContext),(indexCase)-[:CALLS]->(makeClient),(indexCase)-[:CALLS]->(index),(planCase)-[:CALLS]->(makeClient),(planCase)-[:CALLS]->(managementPlan),(confirmCase)-[:CALLS]->(makeClient),(confirmCase)-[:CALLS]->(managementConfirm),(cursor)-[:CALLS]->(makeClient),(cursor)-[:CALLS]->(events),(catchUpCase)-[:CALLS]->(apiCtor),(catchUpCase)-[:CALLS]->(catchUp),(catchUpCase)-[:CALLS]->(catchUpRun),(catchUpCase)-[:USES]->(refreshCallback),(canvasElementCase)-[:CALLS]->(makeClient),(canvasElementCase)-[:CALLS]->(createCanvasElement),(documentCase)-[:CALLS]->(makeClient),(documentCase)-[:CALLS]->(updateCanvasDocument),(error)-[:CALLS]->(makeClient),(error)-[:CALLS]->(groupContext),(error)-[:USES]->(apiError),(freshToken)-[:CALLS]->(apiCtor),(freshToken)-[:CALLS]->(groupContext),(https)-[:CALLS]->(apiCtor),(https)-[:USES]->(apiError);
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

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"worktreeGroupApi tests"}),
      (provider:Function {name:"WorktreeGroupApiProvider"}),
      (projection:Function {name:"useWorktreeGroupProjection"});
CREATE (selectedCanvasStreamCase:Function {name:"single Worktree stream across Canvas selection case",type:"function",language:"typescript",visibility:"private",complexity:"complex"});
CREATE (suite)-[:CONTAINS]->(selectedCanvasStreamCase),
       (selectedCanvasStreamCase)-[:CALLS]->(provider),
       (selectedCanvasStreamCase)-[:CALLS]->(projection);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"worktreeGroupApi tests"}),
      (makeClient:Function {name:"makeClient"}),
      (api:Function {name:"WorktreeGroupApiClient.listWorktreeCanvasEvents"}),
      (catchUp:Class {name:"CanvasOutboxCatchUp"}),
      (catchUpRun:Function {name:"CanvasOutboxCatchUp.catchUp"});
CREATE (worktreeEventsCase:Function {name:"Worktree Canvas stream API case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
       (invalidPageCase:Function {name:"malformed Worktree Canvas page case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (suite)-[:CONTAINS]->(worktreeEventsCase),(suite)-[:CONTAINS]->(invalidPageCase),
       (worktreeEventsCase)-[:CALLS]->(makeClient),(worktreeEventsCase)-[:CALLS]->(api),
       (invalidPageCase)-[:CALLS]->(makeClient),(invalidPageCase)-[:CALLS]->(catchUp),(invalidPageCase)-[:CALLS]->(catchUpRun);
*/

import { createElement, StrictMode, type ReactNode } from "react";
import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { WorktreeGroupApiProvider, useWorktreeGroupProjection } from "../groupProjection";

import {
  CanvasOutboxCatchUp,
  CanvasOutboxWakeSubscriber,
  GroupApiError,
  normalizeWorktreeGitLock,
  WorktreeGroupApiClient,
} from "../worktreeGroupApi";

function makeClient(token: string | null, response = new Response(JSON.stringify({ ok: true }), { status: 200 })) {
  const fetcher = vi.fn<typeof fetch>().mockResolvedValue(response);
  const api = new WorktreeGroupApiClient(async () => token, fetcher);
  return { api, fetcher };
}

function makeWakeStream() {
  let controller: ReadableStreamDefaultController<Uint8Array> | undefined;
  let cancelled = false;
  const response = new Response(new ReadableStream<Uint8Array>({
    start(nextController) {
      controller = nextController;
    },
    cancel() {
      cancelled = true;
    },
  }), {
    status: 200,
    headers: { "Content-Type": "text/event-stream; charset=utf-8" },
  });
  const encoder = new TextEncoder();
  return {
    response,
    send(value: string) {
      controller?.enqueue(encoder.encode(value));
    },
    close() {
      controller?.close();
    },
    wasCancelled() {
      return cancelled;
    },
  };
}

function makeWakeThenClosedResponse(): Response {
  const encoder = new TextEncoder();
  return new Response(new ReadableStream<Uint8Array>({
    start(controller) {
      controller.enqueue(encoder.encode("event: canvas-wake\n\n"));
      controller.close();
    },
  }), {
    status: 200,
    headers: { "Content-Type": "text/event-stream" },
  });
}

function makeClosedWakeResponse(): Response {
  return new Response(new ReadableStream<Uint8Array>({ start(controller) { controller.close(); } }), {
    status: 200,
    headers: { "Content-Type": "text/event-stream" },
  });
}

function emptyWorktreeCanvasPage(offset = "0"): string {
  return JSON.stringify({ events: [], has_more: false, next_offset: offset });
}

async function flushAsyncWork(): Promise<void> {
  for (let index = 0; index < 30; index += 1) await Promise.resolve();
}

async function waitForAsyncCondition(condition: () => boolean): Promise<void> {
  for (let index = 0; index < 100; index += 1) {
    if (condition()) return;
    await Promise.resolve();
  }
  throw new Error("Asynchronous condition did not settle within the test turn.");
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
  it("does not issue or invalidate an authorized read when its scope aborts during token resolution", async () => {
    let resolveToken!: (token: string | null) => void;
    const token = new Promise<string | null>((resolve) => { resolveToken = resolve; });
    const fetcher = vi.fn<typeof fetch>();
    const onAuthorizationError = vi.fn();
    const api = new WorktreeGroupApiClient(() => token, fetcher, "", onAuthorizationError);
    const controller = new AbortController();
    const request = api.listWorktreeCanvasEvents("wt-1", "0", 50, controller.signal);

    controller.abort();
    resolveToken("user-jwt");
    await expect(request).rejects.toMatchObject({ name: "AbortError" });
    expect(fetcher).not.toHaveBeenCalled();
    expect(onAuthorizationError).not.toHaveBeenCalled();
  });

  it("invalidates the host session on missing token or HTTP 401 but keeps 403 scoped", async () => {
    const missingTokenInvalidation = vi.fn<(error: GroupApiError) => void>();
    const missingTokenFetcher = vi.fn<typeof fetch>();
    const missingTokenApi = new WorktreeGroupApiClient(async () => null, missingTokenFetcher, "", missingTokenInvalidation);
    await expect(missingTokenApi.getGroupContext("wt-1")).rejects.toMatchObject({ status: 401, code: "session_required" });
    expect(missingTokenInvalidation).toHaveBeenCalledTimes(1);
    expect(missingTokenFetcher).not.toHaveBeenCalled();

    const unauthorizedInvalidation = vi.fn<(error: GroupApiError) => void>();
    const unauthorizedApi = new WorktreeGroupApiClient(
      async () => "user-jwt",
      vi.fn<typeof fetch>().mockResolvedValue(new Response(JSON.stringify({ error: { code: "session_expired" } }), { status: 401 })),
      "",
      unauthorizedInvalidation,
    );
    await expect(unauthorizedApi.getGroupContext("wt-1")).rejects.toMatchObject({ status: 401, code: "session_expired" });
    await expect(unauthorizedApi.getGroupContext("wt-1")).rejects.toMatchObject({ status: 401, code: "session_invalidated" });
    expect(unauthorizedInvalidation).toHaveBeenCalledTimes(1);

    const forbiddenInvalidation = vi.fn<(error: GroupApiError) => void>();
    const forbiddenApi = new WorktreeGroupApiClient(
      async () => "user-jwt",
      vi.fn<typeof fetch>().mockResolvedValue(new Response(JSON.stringify({ error: { code: "forbidden" } }), { status: 403 })),
      "",
      forbiddenInvalidation,
    );
    await expect(forbiddenApi.getGroupContext("wt-1")).rejects.toMatchObject({ status: 403, code: "forbidden" });
    expect(forbiddenInvalidation).not.toHaveBeenCalled();
  });

  it("lists Project-scoped Hook execution events with a bounded cursor", async () => {
    const responseBody = {
      events: [],
      next_cursor: null,
      coverage: {
        scope: "hook_execution_event_ledger",
        status: "partial",
        reported_percentage: null,
        instrumented_phases: ["worktree_archive"],
        not_yet_instrumented_phases: ["run_admission", "tool"],
        note: "Coverage is partial.",
      },
    };
    const { api, fetcher } = makeClient("user-jwt", new Response(JSON.stringify(responseBody), { status: 200 }));

    await expect(api.listHookEvents("project one", { limit: 30, cursor: "event-cursor" })).resolves.toEqual(responseBody);

    const [input, init] = fetcher.mock.calls[0];
    const url = new URL(String(input), "http://localhost");
    expect(url.pathname).toBe("/api/v1/projects/project%20one/hook-events");
    expect(Object.fromEntries(url.searchParams.entries())).toEqual({ limit: "30", cursor: "event-cursor" });
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(init?.credentials).toBe("omit");
    expect(init?.cache).toBe("no-store");
  });

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

  it("lists Worktree Canvas events with an opaque 64-bit decimal offset", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    await api.listWorktreeCanvasEvents("wt one", "9007199254740993", 50);

    const [input, init] = fetcher.mock.calls[0];
    const url = new URL(String(input), "http://localhost");
    expect(url.pathname).toBe("/api/v1/worktrees/wt%20one/canvas-events");
    expect(url.searchParams.get("after_offset")).toBe("9007199254740993");
    expect(url.searchParams.get("limit")).toBe("50");
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
  });

  it("lists Task Runs using an encoded Task scope and bounded cursor", async () => {
    const { api, fetcher } = makeClient("user-jwt");

    await api.listTaskRuns("wt one", "task/one", {
      limit: 20,
      cursor: {
        cursor_started_at: "2026-09-29T10:00:00Z",
        cursor_ordered_at: "2026-09-29T10:00:00Z",
        cursor_run_id: "run-1",
      },
    });

    const [input, init] = fetcher.mock.calls[0];
    const url = new URL(String(input), "http://localhost");
    expect(url.pathname).toBe("/api/v1/worktrees/wt%20one/work-items/task%2Fone/runs");
    expect(url.searchParams.get("limit")).toBe("20");
    expect(url.searchParams.get("cursor_started_at")).toBe("2026-09-29T10:00:00Z");
    expect(url.searchParams.get("cursor_ordered_at")).toBe("2026-09-29T10:00:00Z");
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
    const offset = "9007199254740993";
    const page = JSON.stringify({
      events: [{
        stream_offset: offset,
        event_id: "11111111-1111-4111-8111-111111111111",
        canvas_id: "22222222-2222-4222-8222-222222222222",
        event_type: "element.created",
        occurred_at: "2026-09-29T10:00:00Z",
      }],
      has_more: false,
      next_offset: offset,
    });
    const emptyPage = JSON.stringify({ events: [], has_more: false, next_offset: offset });
    const fetcher = vi.fn<typeof fetch>()
      .mockResolvedValueOnce(new Response(page, { status: 200 }))
      .mockResolvedValueOnce(new Response(page, { status: 200 }))
      .mockResolvedValueOnce(new Response(emptyPage, { status: 200 }));
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
    const refreshProjection = vi.fn<() => Promise<void>>()
      .mockRejectedValueOnce(new Error("projection refresh failed"))
      .mockResolvedValue(undefined);
    let savedCursor: { after_offset: string } | undefined;
    const catchUp = new CanvasOutboxCatchUp(api, "wt-1", refreshProjection, {
      cursorStore: {
        load: () => savedCursor,
        save: (_scopeKey, cursor) => { savedCursor = cursor; },
      },
    });

    const controller = new AbortController();
    await expect(catchUp.catchUp(controller.signal)).rejects.toThrow("projection refresh failed");
    await expect(catchUp.catchUp(controller.signal)).resolves.toBe(1);
    await expect(catchUp.catchUp(controller.signal)).resolves.toBe(0);

    const urls = fetcher.mock.calls.map(([input]) => new URL(String(input), "http://localhost"));
    expect(urls.map((url) => url.pathname)).toEqual([
      "/api/v1/worktrees/wt-1/canvas-events",
      "/api/v1/worktrees/wt-1/canvas-events",
      "/api/v1/worktrees/wt-1/canvas-events",
    ]);
    expect(urls[0].searchParams.get("after_offset")).toBe("0");
    expect(urls[1].searchParams.get("after_offset")).toBe("0");
    expect(urls[2].searchParams.get("after_offset")).toBe(offset);
    expect(savedCursor).toEqual({ after_offset: offset });
    expect(refreshProjection).toHaveBeenCalledTimes(2);
  });

  it("rejects malformed empty Worktree Canvas pages without advancing the cursor", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValueOnce(
      new Response(JSON.stringify({ events: [], has_more: true, next_offset: "1" }), { status: 200 }),
    );
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
    const refreshProjection = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    let savedCursor: { after_offset: string } | undefined;
    const catchUp = new CanvasOutboxCatchUp(api, "wt-1", refreshProjection, {
      cursorStore: {
        load: () => savedCursor,
        save: (_scopeKey, cursor) => { savedCursor = cursor; },
      },
    });

    await expect(catchUp.catchUp(new AbortController().signal)).rejects.toThrow("inconsistent cursor");
    expect(savedCursor).toBeUndefined();
    expect(refreshProjection).not.toHaveBeenCalled();
  });

  it("shares one authenticated Worktree wake stream and fans out resource wakes", async () => {
    const wakeStream = makeWakeStream();
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(wakeStream.response);
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
    const firstEvents: string[] = [];
    const secondEvents: string[] = [];
    const unsubscribeFirst = api.subscribeWorktreeCanvasWake("wt-1", (event) => firstEvents.push(event.type));
    const unsubscribeSecond = api.subscribeWorktreeCanvasWake("wt-1", (event) => secondEvents.push(event.type));
    await flushAsyncWork();

    expect(fetcher).toHaveBeenCalledTimes(1);
    const [input, init] = fetcher.mock.calls[0];
    expect(new URL(String(input), "http://localhost").pathname).toBe(
      "/api/v1/worktrees/wt-1/canvas-events/stream",
    );
    expect(new Headers(init?.headers).get("Accept")).toBe("text/event-stream");
    expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer user-jwt");
    expect(init?.credentials).toBe("omit");
    expect(init?.cache).toBe("no-store");
    expect(firstEvents).toContain("resync_required");
    expect(secondEvents).toContain("resync_required");

    wakeStream.send("event: canvas-wake\ndata: {}\n\n");
    await flushAsyncWork();
    expect(firstEvents).toContain("canvas-wake");
    expect(secondEvents).toContain("canvas-wake");

    const streamSignal = init?.signal;
    unsubscribeFirst();
    expect(streamSignal?.aborted).toBe(false);
    unsubscribeSecond();
    await flushAsyncWork();
    expect(streamSignal?.aborted).toBe(true);
    expect(wakeStream.wasCancelled()).toBe(true);
  });

  it("keeps one Worktree stream across Strict Mode cleanup and selected Canvas changes", async () => {
    const streamSignals: AbortSignal[] = [];
    const wakeStreams: ReturnType<typeof makeWakeStream>[] = [];
    const canvases = ["canvas-1", "canvas-2"].map((canvasId) => ({
      canvas_id: canvasId,
      title: canvasId,
      version: 1,
      document: { version: 1, viewport: { x: 0, y: 0, zoom: 1 }, frames: [], connectors: [] },
    }));
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input), "http://localhost");
      if (url.pathname.endsWith("/canvas-events/stream")) {
        if (init?.signal) streamSignals.push(init.signal);
        const wakeStream = makeWakeStream();
        wakeStreams.push(wakeStream);
        return wakeStream.response;
      }
      if (url.pathname.endsWith("/group-context")) {
        return new Response(JSON.stringify({
          group_context: { worktree_id: "wt-1", project_id: "project-1" },
          worktree: { id: "wt-1", project_id: "project-1", name: "Agent WT", branch: "codex/agent", human_state: "active", archived: false },
        }), { status: 200 });
      }
      if (url.pathname.endsWith("/work-items")) {
        return new Response(JSON.stringify({ worktree_id: "wt-1", project_id: "project-1", work_items: [] }), { status: 200 });
      }
      if (url.pathname.endsWith("/canvases")) {
        return new Response(JSON.stringify({ worktree_id: "wt-1", project_id: "project-1", canvases }), { status: 200 });
      }
      if (url.pathname.endsWith("/elements")) {
        const canvasId = url.pathname.split("/").at(-2);
        return new Response(JSON.stringify({ worktree_id: "wt-1", canvas_id: canvasId, elements: [] }), { status: 200 });
      }
      if (url.pathname.endsWith("/canvas-events")) {
        return new Response(emptyWorktreeCanvasPage(url.searchParams.get("after_offset") ?? "0"), { status: 200 });
      }
      throw new Error(`Unexpected request ${url.pathname}`);
    });
    const getAccessToken = async () => "user-jwt";
    const wrapper = ({ children }: { children: ReactNode }) => createElement(
      StrictMode,
      null,
      createElement(WorktreeGroupApiProvider, {
        getAccessToken,
        sessionKey: "session-1",
        fetcher,
      }, children),
    );
    const { result, rerender, unmount } = renderHook(
      ({ selectedCanvasId }: { selectedCanvasId: string }) => useWorktreeGroupProjection("wt-1", true, 0, selectedCanvasId),
      { initialProps: { selectedCanvasId: "canvas-1" }, wrapper },
    );

    await waitFor(() => expect(result.current).toMatchObject({ mode: "live", canvas: { id: "canvas-1" } }));
    await waitFor(() => expect(streamSignals.filter((signal) => !signal.aborted)).toHaveLength(1));
    const initialStreamCount = streamSignals.length;
    expect(initialStreamCount).toBeGreaterThan(0);

    rerender({ selectedCanvasId: "canvas-2" });
    await waitFor(() => expect(result.current).toMatchObject({ mode: "live", canvas: { id: "canvas-2" } }));
    expect(streamSignals).toHaveLength(initialStreamCount);
    expect(streamSignals.filter((signal) => !signal.aborted)).toHaveLength(1);

    act(() => unmount());
    expect(streamSignals.every((signal) => signal.aborted)).toBe(true);
    expect(wakeStreams.every((stream) => stream.wasCancelled())).toBe(true);
  });

  it("coalesces same-turn wake frames and catches up through the authorized Outbox API", async () => {
    const wakeStream = makeWakeStream();
    let outboxRequests = 0;
    const offset = "3";
    const eventPage = JSON.stringify({
      events: [{
        stream_offset: offset,
        event_id: "11111111-1111-4111-8111-111111111111",
        canvas_id: "22222222-2222-4222-8222-222222222222",
        event_type: "document.updated",
        occurred_at: "2026-10-27T10:00:00Z",
      }],
      has_more: false,
      next_offset: offset,
    });
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input), "http://localhost");
      if (url.pathname.endsWith("/canvas-events/stream")) return wakeStream.response;
      if (url.pathname.endsWith("/canvas-events")) {
        outboxRequests += 1;
        return new Response(outboxRequests === 1 ? emptyWorktreeCanvasPage() : eventPage, { status: 200 });
      }
      throw new Error(`Unexpected request ${url.pathname}`);
    });
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
    const refreshProjection = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    let savedCursor: { after_offset: string } | undefined;
    const subscriber = new CanvasOutboxWakeSubscriber(api, "wt-1", async () => refreshProjection(), {
      cursorStore: {
        load: () => savedCursor,
        save: (_scopeKey, cursor) => { savedCursor = cursor; },
      },
    });
    const controller = new AbortController();
    const unsubscribe = subscriber.subscribe(controller.signal);
    await waitForAsyncCondition(() => outboxRequests === 1);
    await flushAsyncWork();

    wakeStream.send("event: canvas-wake\n\nevent: canvas-wake\n\nevent: canvas-wake\n\n");
    await waitForAsyncCondition(() => outboxRequests === 2 && refreshProjection.mock.calls.length === 1 && savedCursor !== undefined);
    expect(outboxRequests).toBe(2);
    expect(savedCursor).toEqual({ after_offset: offset });
    expect(fetcher.mock.calls.filter(([input]) => new URL(String(input), "http://localhost").pathname.endsWith("/canvas-events/stream"))).toHaveLength(1);

    unsubscribe();
    await flushAsyncWork();
  });

  it("reconnects with a fresh token and bounded exponential backoff after stream close", async () => {
    vi.useFakeTimers();
    try {
      const streams = [makeWakeThenClosedResponse(), makeWakeThenClosedResponse(), makeWakeStream().response];
      let streamIndex = 0;
      let tokenIndex = 0;
      const fetcher = vi.fn<typeof fetch>(async (input) => {
        const url = new URL(String(input), "http://localhost");
        if (!url.pathname.endsWith("/canvas-events/stream")) throw new Error(`Unexpected request ${url.pathname}`);
        const response = streams[streamIndex];
        streamIndex += 1;
        return response;
      });
      const api = new WorktreeGroupApiClient(async () => `user-jwt-${++tokenIndex}`, fetcher);
      const unsubscribe = api.subscribeWorktreeCanvasWake("wt-1", () => undefined);
      await flushAsyncWork();
      expect(fetcher).toHaveBeenCalledTimes(1);

      await vi.advanceTimersByTimeAsync(499);
      await flushAsyncWork();
      expect(fetcher).toHaveBeenCalledTimes(1);
      await vi.advanceTimersByTimeAsync(1);
      await flushAsyncWork();
      expect(fetcher).toHaveBeenCalledTimes(2);
      expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe("Bearer user-jwt-1");
      expect(new Headers(fetcher.mock.calls[1][1]?.headers).get("Authorization")).toBe("Bearer user-jwt-2");

      await vi.advanceTimersByTimeAsync(999);
      await flushAsyncWork();
      expect(fetcher).toHaveBeenCalledTimes(2);
      await vi.advanceTimersByTimeAsync(1);
      await flushAsyncWork();
      expect(fetcher).toHaveBeenCalledTimes(3);
      unsubscribe();
    } finally {
      vi.useRealTimers();
    }
  });

  it("cancels stream backoff and failed catch-up retry timers when the Worktree scope leaves", async () => {
    vi.useFakeTimers();
    try {
      let streamRequests = 0;
      let outboxRequests = 0;
      const fetcher = vi.fn<typeof fetch>(async (input) => {
        const url = new URL(String(input), "http://localhost");
        if (url.pathname.endsWith("/canvas-events/stream")) {
          streamRequests += 1;
          return makeClosedWakeResponse();
        }
        if (url.pathname.endsWith("/canvas-events")) {
          outboxRequests += 1;
          return new Response(JSON.stringify({ error: { code: "temporarily_unavailable" } }), { status: 503 });
        }
        throw new Error(`Unexpected request ${url.pathname}`);
      });
      const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
      const subscriber = new CanvasOutboxWakeSubscriber(api, "wt-1", async () => undefined);
      const unsubscribe = subscriber.subscribe(new AbortController().signal);
      await waitForAsyncCondition(() => streamRequests === 1 && outboxRequests === 1);
      await flushAsyncWork();

      unsubscribe();
      await vi.advanceTimersByTimeAsync(60_000);
      await flushAsyncWork();
      expect(streamRequests).toBe(1);
      expect(outboxRequests).toBe(1);
    } finally {
      vi.useRealTimers();
    }
  });

  it("catches up on resync_required and reconnects after the bounded delay", async () => {
    vi.useFakeTimers();
    try {
      const streams = [makeWakeStream(), makeWakeStream()];
      let streamIndex = 0;
      let outboxRequests = 0;
      const fetcher = vi.fn<typeof fetch>(async (input) => {
        const url = new URL(String(input), "http://localhost");
        if (url.pathname.endsWith("/canvas-events/stream")) {
          const stream = streams[streamIndex];
          streamIndex += 1;
          return stream.response;
        }
        if (url.pathname.endsWith("/canvas-events")) {
          outboxRequests += 1;
          return new Response(emptyWorktreeCanvasPage(), { status: 200 });
        }
        throw new Error(`Unexpected request ${url.pathname}`);
      });
      const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
      const controller = new AbortController();
      const subscriber = new CanvasOutboxWakeSubscriber(api, "wt-1", async () => undefined);
      const unsubscribe = subscriber.subscribe(controller.signal);
      await waitForAsyncCondition(() => outboxRequests === 1 && streamIndex === 1);
      await flushAsyncWork();

      streams[0].send("event: resync_required\n\n");
      await waitForAsyncCondition(() => outboxRequests >= 2);
      await flushAsyncWork();
      expect(streamIndex).toBe(1);
      expect(outboxRequests).toBeGreaterThanOrEqual(2);

      await vi.advanceTimersByTimeAsync(499);
      await flushAsyncWork();
      expect(streamIndex).toBe(1);
      await vi.advanceTimersByTimeAsync(1);
      await waitForAsyncCondition(() => streamIndex === 2);
      expect(outboxRequests).toBeGreaterThanOrEqual(3);
      unsubscribe();
    } finally {
      vi.useRealTimers();
    }
  });

  it("aborts the shared reader and in-flight authorized catch-up when scope is removed", async () => {
    const wakeStream = makeWakeStream();
    let streamSignal: AbortSignal | null | undefined;
    let pageSignal: AbortSignal | null | undefined;
    const fetcher = vi.fn<typeof fetch>((input, init) => {
      const url = new URL(String(input), "http://localhost");
      if (url.pathname.endsWith("/canvas-events/stream")) {
        streamSignal = init?.signal;
        return Promise.resolve(wakeStream.response);
      }
      pageSignal = init?.signal;
      return new Promise<Response>((_resolve, reject) => {
        pageSignal?.addEventListener("abort", () => reject(new DOMException("Aborted", "AbortError")), { once: true });
      });
    });
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
    const controller = new AbortController();
    const subscriber = new CanvasOutboxWakeSubscriber(api, "wt-1", async () => undefined);
    const unsubscribe = subscriber.subscribe(controller.signal);
    await waitForAsyncCondition(() => pageSignal !== undefined && streamSignal !== undefined);

    unsubscribe();
    await flushAsyncWork();
    expect(pageSignal?.aborted).toBe(true);
    expect(streamSignal?.aborted).toBe(true);
    expect(wakeStream.wasCancelled()).toBe(true);
  });

  it.each([
    { status: 401, invalidatesSession: true },
    { status: 403, invalidatesSession: false },
  ])("stops a rejected wake stream for status $status and preserves the 403 session scope", async ({ status, invalidatesSession }) => {
    const onAuthorizationError = vi.fn();
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(JSON.stringify({ error: { code: "denied", message: "Denied." } }), { status }),
    );
    const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher, "", onAuthorizationError);
    const errors: unknown[] = [];
    const unsubscribe = api.subscribeWorktreeCanvasWake("wt-1", (event) => {
      if (event.type === "error") errors.push(event.error);
    });
    await flushAsyncWork();

    expect(errors).toHaveLength(1);
    expect(errors[0]).toMatchObject({ status, code: "denied" });
    expect(onAuthorizationError).toHaveBeenCalledTimes(invalidatesSession ? 1 : 0);
    expect(fetcher).toHaveBeenCalledTimes(1);
    unsubscribe();
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

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"worktreeGroupApi tests"}),
      (api:Class {name:"WorktreeGroupApiClient"});
CREATE (hookEvents:Function {name:"Hook execution events API case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (suite)-[:CONTAINS]->(hookEvents),(hookEvents)-[:CALLS]->(api);
*/
