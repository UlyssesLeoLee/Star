/*
@cypher schema=1 source_sha256=2d49617fb9cdf380a09da4660855090dfd1e5ae64d7e580ff22679a7316e0366
MERGE (self:File {path:"frontend/src/lib/run/runDirectorySession.test.tsx"})
MERGE (provider:Symbol {id:"frontend/src/lib/run/runDirectorySession.tsx::RunDirectoryProvider", kind:"function"})
MERGE (hook:Symbol {id:"frontend/src/lib/run/runDirectorySession.tsx::useRunDirectory", kind:"function"})
MERGE (client:Symbol {id:"frontend/src/lib/run/runDirectoryApi.ts::RunDirectoryApiClient.listProjects", kind:"method"})
MERGE (missing:Symbol {id:"frontend/src/lib/run/runDirectorySession.test.tsx::missing host session remains fail closed", kind:"test"})
MERGE (ready:Symbol {id:"frontend/src/lib/run/runDirectorySession.test.tsx::host session exposes a ready client and Run focus", kind:"test"})
MERGE (invalidated:Symbol {id:"frontend/src/lib/run/runDirectorySession.test.tsx::authorization invalidation blocks the directory and clears focus", kind:"test"})
MERGE (replaced:Symbol {id:"frontend/src/lib/run/runDirectorySession.test.tsx::session generation replacement clears focus and disposes the old client", kind:"test"})
MERGE (self)-[:DEFINES]->(missing)
MERGE (self)-[:DEFINES]->(ready)
MERGE (self)-[:DEFINES]->(invalidated)
MERGE (self)-[:DEFINES]->(replaced)
MERGE (missing)-[:TESTS]->(provider)
MERGE (missing)-[:TESTS]->(hook)
MERGE (ready)-[:TESTS]->(provider)
MERGE (ready)-[:TESTS]->(hook)
MERGE (invalidated)-[:TESTS]->(provider)
MERGE (invalidated)-[:TESTS]->(hook)
MERGE (invalidated)-[:CALLS]->(client)
MERGE (replaced)-[:TESTS]->(provider)
MERGE (replaced)-[:TESTS]->(hook)
MERGE (replaced)-[:CALLS]->(client)
@endcypher
*/

import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { PropsWithChildren } from "react";
import {
  RunDirectoryProvider,
  useRunDirectory,
  type RunDirectoryHostSession,
} from "./runDirectorySession";
import type { RunContextEnvelope } from "./runDirectoryApi";

const RUN_ID = "10000000-0000-4000-8000-000000000001";
const PROJECT_ID = "10000000-0000-4000-8000-000000000002";
const REPOSITORY_ID = "10000000-0000-4000-8000-000000000003";
const BRANCH_ID = "10000000-0000-4000-8000-000000000004";

function createHostSession(generation: string, fetcher: typeof fetch = vi.fn<typeof fetch>(async () => new Response("{}", { status: 200 }))): RunDirectoryHostSession {
  return {
    generation,
    getAccessToken: vi.fn(async () => "host-session-token"),
    fetcher,
  };
}

function contextFixture(): RunContextEnvelope {
  const run = {
    engineering_run_id: RUN_ID,
    project_id: PROJECT_ID,
    repository_id: REPOSITORY_ID,
    branch_id: BRANCH_ID,
    title: "Run",
    state: "active",
    owner_user_id: null,
    version: 1,
  };
  return {
    run_context: {
      tenant_id: "10000000-0000-4000-8000-000000000005",
      project_id: PROJECT_ID,
      repository_id: REPOSITORY_ID,
      branch_id: BRANCH_ID,
      engineering_run_id: RUN_ID,
      actor_id: "10000000-0000-4000-8000-000000000006",
      actor_kind: "user",
      authorization: {
        project: { binding_id: "40000000-0000-4000-8000-000000000001", role: "developer", version: 1 },
        branch: { binding_id: "40000000-0000-4000-8000-000000000002", role: "developer", version: 1 },
        engineering_run: { binding_id: "40000000-0000-4000-8000-000000000003", role: "developer", version: 1 },
      },
      granted_scopes: [],
      context_version: "ctx:v1",
      resolved_at: "2026-10-15T00:00:00Z",
      correlation_id: "10000000-0000-4000-8000-000000000007",
    },
    branch: {
      branch_id: BRANCH_ID,
      project_id: PROJECT_ID,
      repository_id: REPOSITORY_ID,
      name: "feature/run-session",
      full_ref: "refs/heads/feature/run-session",
      head_commit_id: null,
      state: "active",
      version: 1,
    },
    engineering_run: run,
    focus: {
      worktree_id: "30000000-0000-4000-8000-000000000001",
      engineering_run_id: RUN_ID,
      project_id: PROJECT_ID,
      repository_id: REPOSITORY_ID,
      workspace_id: "30000000-0000-4000-8000-000000000002",
      name: "run-focus",
      branch: "feature/run-session",
      human_state: "active",
      machine_state: "ready",
      agent_id: null,
      agent_session_id: null,
      runtime_id: null,
      archived: false,
      version: 1,
      binding_id: "30000000-0000-4000-8000-000000000003",
      binding_version: 1,
      project_binding_id: "30000000-0000-4000-8000-000000000004",
      project_binding_version: 1,
    },
    focus_version: "focus:v1",
    capabilities: { task_cards_available: true, run_owned_apps_available: false, execution_admission_available: false },
  };
}

function wrapperFor(session: RunDirectoryHostSession | null) {
  return function RunDirectorySessionWrapper({ children }: PropsWithChildren) {
    return <RunDirectoryProvider session={session}>{children}</RunDirectoryProvider>;
  };
}

describe("RunDirectoryProvider host session", () => {
  it("keeps the directory fail closed when the host session is missing", () => {
    const { result } = renderHook(() => useRunDirectory(), { wrapper: wrapperFor(null) });

    expect(result.current.status).toBe("missing");
    expect(result.current.client).toBeNull();
    expect(result.current.focus).toBeNull();
  });

  it("exposes a ready client and accepts Run focus from a host session", () => {
    const { result } = renderHook(() => useRunDirectory(), { wrapper: wrapperFor(createHostSession("generation-1")) });
    const focus = contextFixture();

    expect(result.current.status).toBe("ready");
    expect(result.current.client).not.toBeNull();
    act(() => result.current.setFocus(focus));
    expect(result.current.focus).toBe(focus);
  });

  it("blocks the directory and clears focus when authorization is invalidated", async () => {
    const fetcher = vi.fn<typeof fetch>(async () => new Response(null, { status: 401 }));
    const { result } = renderHook(() => useRunDirectory(), { wrapper: wrapperFor(createHostSession("generation-1", fetcher)) });
    const focus = contextFixture();
    act(() => result.current.setFocus(focus));

    let requestError: unknown;
    await act(async () => {
      try {
        await result.current.client!.listProjects();
      } catch (error) {
        requestError = error;
      }
    });

    expect(requestError).toMatchObject({ code: "authorization_failed", status: 401 });
    await waitFor(() => {
      expect(result.current.status).toBe("blocked");
      expect(result.current.client).toBeNull();
      expect(result.current.focus).toBeNull();
    });
  });

  it("clears focus and disposes the old client when the host session generation changes", async () => {
    let session = createHostSession("generation-1");
    const oldSession = session;
    const wrapper = ({ children }: PropsWithChildren) => <RunDirectoryProvider session={session}>{children}</RunDirectoryProvider>;
    const { result, rerender } = renderHook(() => useRunDirectory(), { wrapper });
    const oldClient = result.current.client!;
    const dispose = vi.spyOn(oldClient, "dispose");
    act(() => result.current.setFocus(contextFixture()));
    expect(result.current.focus).not.toBeNull();

    session = createHostSession("generation-2");
    rerender();

    await waitFor(() => {
      expect(result.current.status).toBe("ready");
      expect(result.current.client).not.toBe(oldClient);
      expect(result.current.focus).toBeNull();
      expect(dispose).toHaveBeenCalledOnce();
    });
    await expect(oldClient.listProjects()).rejects.toMatchObject({ name: "AbortError" });
    expect(oldSession.getAccessToken).not.toHaveBeenCalled();
  });
});
