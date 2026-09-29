/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/lib/group/__tests__/groupProjection.test.tsx",type:"file",language:"tsx"}),
  (suite:Function {name:"group projection tests",type:"function",signature:"describe('Worktree Group projection', ...)",visibility:"private",complexity:"moderate"}),
  (previewCase:Function {name:"missing provider case",type:"function",signature:"it('uses preview mode only when no host provider is installed', ...)",visibility:"private",complexity:"simple"}),
  (liveCase:Function {name:"authorized projection case",type:"function",signature:"it('loads API projections through the host user token', ...)",visibility:"private",complexity:"moderate"}),
  (canvasCase:Function {name:"Canvas projection case",type:"function",signature:"it('loads scoped Canvas data and starts the outbox poller for the Canvas app', ...)",visibility:"private",complexity:"complex"}),
  (failClosedCase:Function {name:"missing session case",type:"function",signature:"it('fails closed instead of displaying seed data when session is missing', ...)",visibility:"private",complexity:"moderate"}),
  (scopeCase:Function {name:"mismatched Project case",type:"function",signature:"it('rejects a GroupContext from another Project', ...)",visibility:"private",complexity:"moderate"}),
  (refreshCase:Function {name:"manual projection refresh case",type:"function",signature:"it('reloads the authorized projection when the refresh key changes', ...)",visibility:"private",complexity:"moderate"}),
  (file)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(previewCase),(suite)-[:CONTAINS]->(liveCase),(suite)-[:CONTAINS]->(canvasCase),(suite)-[:CONTAINS]->(failClosedCase),(suite)-[:CONTAINS]->(scopeCase),(suite)-[:CONTAINS]->(refreshCase),
  (previewCase)-[:CALLS]->(useProjection:Function {name:"useWorktreeGroupProjection",type:"function"}),(liveCase)-[:CALLS]->(provider:Function {name:"WorktreeGroupApiProvider",type:"function"}),(liveCase)-[:CALLS]->(useProjection),(canvasCase)-[:CALLS]->(provider),(canvasCase)-[:CALLS]->(useProjection),(failClosedCase)-[:CALLS]->(provider),(failClosedCase)-[:CALLS]->(useProjection),(scopeCase)-[:CALLS]->(provider),(scopeCase)-[:CALLS]->(useProjection),(refreshCase)-[:CALLS]->(provider),(refreshCase)-[:CALLS]->(useProjection);
*/

import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";

import { WorktreeGroupApiProvider, useWorktreeGroupProjection } from "../groupProjection";

describe("Worktree Group projection", () => {
  it("uses preview mode only when no host provider is installed", () => {
    const { result } = renderHook(() => useWorktreeGroupProjection("wt-1", false));
    expect(result.current).toEqual({ mode: "preview" });
  });

  it("loads API projections through the host user token", async () => {
    let token = "user-jwt";
    let sessionKey = "session-1";
    let refreshKey = 0;
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async (input) => {
      const path = new URL(String(input), "http://localhost").pathname;
      if (path.endsWith("/group-context")) {
        return new Response(JSON.stringify({
          group_context: { worktree_id: "wt-1", project_id: "project-1" },
          worktree: { id: "wt-1", project_id: "project-1", name: "Agent WT", branch: "codex/agent", human_state: "active", archived: false },
        }), { status: 200 });
      }
      if (path.endsWith("/work-items")) {
        return new Response(JSON.stringify({
          worktree_id: "wt-1", project_id: "project-1", work_items: [{
            work_item_id: "wi-1", item_type: "ai_task", title: "Implement Group UI", description: "Wire the authorized view.",
            priority: "high", labels: ["group"], reporter_user_id: "user-1", lifecycle: { status: "claimed", version: 2 },
          }],
        }), { status: 200 });
      }
      return new Response(JSON.stringify({ worktree_id: "wt-1", project_id: "project-1", canvases: [] }), { status: 200 });
    });
    const getAccessToken = vi.fn(async () => token);
    const wrapper = ({ children }: { children: ReactNode }) => (
      <WorktreeGroupApiProvider getAccessToken={getAccessToken} sessionKey={sessionKey} fetcher={fetcher}>{children}</WorktreeGroupApiProvider>
    );

    const { result, rerender } = renderHook(() => useWorktreeGroupProjection("wt-1", false, refreshKey), { wrapper });
    await waitFor(() => expect(result.current.mode).toBe("live"));

    expect(result.current).toMatchObject({
      mode: "live",
      worktree: { id: "wt-1", project_id: "project-1", branch: "codex/agent" },
      work_items: [{ id: "wi-1", status: "in_progress", priority: "p1", kind: "task" }],
      canvas: undefined,
    });
    expect(fetcher).toHaveBeenCalledTimes(2);
    expect(fetcher.mock.calls.every(([, init]) => new Headers(init?.headers).get("Authorization") === "Bearer user-jwt")).toBe(true);

    token = "user-jwt-after-session-change";
    sessionKey = "session-2";
    rerender();
    expect(result.current).toEqual({ mode: "loading" });
    await waitFor(() => expect(result.current.mode).toBe("live"));
    expect(fetcher).toHaveBeenCalledTimes(4);
    expect(fetcher.mock.calls.slice(2).every(([, init]) => new Headers(init?.headers).get("Authorization") === "Bearer user-jwt-after-session-change")).toBe(true);

    refreshKey += 1;
    rerender();
    expect(result.current).toEqual({ mode: "loading" });
    await waitFor(() => expect(result.current.mode).toBe("live"));
    expect(fetcher).toHaveBeenCalledTimes(6);
  });

  it("loads scoped Canvas data and starts the outbox poller for the Canvas app", async () => {
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async (input) => {
      const path = new URL(String(input), "http://localhost").pathname;
      if (path.endsWith("/group-context")) {
        return new Response(JSON.stringify({
          group_context: { worktree_id: "wt-1", project_id: "project-1" },
          worktree: { id: "wt-1", project_id: "project-1", name: "Agent WT", branch: "codex/agent", human_state: "active", archived: false },
        }), { status: 200 });
      }
      if (path.endsWith("/work-items")) {
        return new Response(JSON.stringify({ worktree_id: "wt-1", project_id: "project-1", work_items: [] }), { status: 200 });
      }
      if (path.endsWith("/canvases")) {
        return new Response(JSON.stringify({
          worktree_id: "wt-1", project_id: "project-1", canvases: [
            {
              canvas_id: "canvas-1", title: "First Canvas", version: 2,
              document: { version: 2, viewport: { x: 4, y: 5, zoom: 1.2 }, frames: [], connectors: [] },
            },
            {
              canvas_id: "canvas-2", title: "Selected Canvas", version: 3,
              document: { version: 3, viewport: { x: 40, y: 50, zoom: 1.4 }, frames: [], connectors: [] },
            },
          ],
        }), { status: 200 });
      }
      if (path.endsWith("/elements")) {
        return new Response(JSON.stringify({
          worktree_id: "wt-1", canvas_id: "canvas-2", elements: [{
            element_id: "element-1", version: 1, kind: "sticky_note", x: 10, y: 20, width: 120, height: 80,
            rotation: 0, z_index: 1, content: { text: "API-backed" }, locked: false, hidden: false, entity_ref: null,
          }],
        }), { status: 200 });
      }
      if (path.endsWith("/events")) {
        return new Response(JSON.stringify({ events: [], next_cursor: null }), { status: 200 });
      }
      return new Response(JSON.stringify({ error: { code: "unexpected_route" } }), { status: 404 });
    });
    const wrapper = ({ children }: { children: ReactNode }) => (
      <WorktreeGroupApiProvider getAccessToken={async () => "canvas-user-jwt"} sessionKey="canvas-session-1" fetcher={fetcher}>{children}</WorktreeGroupApiProvider>
    );

    const { result } = renderHook(() => useWorktreeGroupProjection("wt-1", true, 0, "canvas-2"), { wrapper });
    await waitFor(() => expect(result.current.mode).toBe("live"));
    await waitFor(() => expect(fetcher.mock.calls.some(([input]) => String(input).includes("/canvases/canvas-2/events"))).toBe(true));

    expect(result.current).toMatchObject({
      mode: "live",
      canvases: [{ id: "canvas-1" }, { id: "canvas-2" }],
      canvas: { id: "canvas-2", version: 3, viewport: { x: 40, y: 50, zoom: 1.4 } },
      elements: [{ id: "element-1", canvas_id: "canvas-2", content: { text: "API-backed" } }],
      connectors: [],
    });
  });

  it("fails closed instead of displaying seed data when session is missing", async () => {
    const fetcher = vi.fn<typeof fetch>();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <WorktreeGroupApiProvider getAccessToken={async () => null} sessionKey="session-1" fetcher={fetcher}>{children}</WorktreeGroupApiProvider>
    );

    const { result } = renderHook(() => useWorktreeGroupProjection("wt-1", false), { wrapper });
    await waitFor(() => expect(result.current.mode).toBe("error"));

    expect(result.current).toMatchObject({ mode: "error", message: "Group API requires an authenticated user session." });
    expect(fetcher).not.toHaveBeenCalled();
  });

  it("rejects a GroupContext from another Project", async () => {
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async (input) => {
      const path = new URL(String(input), "http://localhost").pathname;
      if (path.endsWith("/group-context")) {
        return new Response(JSON.stringify({
          group_context: { worktree_id: "wt-1", project_id: "project-other" },
          worktree: { id: "wt-1", project_id: "project-1", name: "Agent WT", branch: "codex/agent", human_state: "active", archived: false },
        }), { status: 200 });
      }
      return new Response(JSON.stringify({ worktree_id: "wt-1", project_id: "project-1", work_items: [] }), { status: 200 });
    });
    const wrapper = ({ children }: { children: ReactNode }) => (
      <WorktreeGroupApiProvider getAccessToken={async () => "user-jwt"} sessionKey="session-1" fetcher={fetcher}>{children}</WorktreeGroupApiProvider>
    );

    const { result } = renderHook(() => useWorktreeGroupProjection("wt-1", false), { wrapper });
    await waitFor(() => expect(result.current.mode).toBe("error"));
    expect(result.current).toMatchObject({ mode: "error", message: "GroupContext Project does not match the requested Worktree." });
  });
});
