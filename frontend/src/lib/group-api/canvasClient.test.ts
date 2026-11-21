// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/group-api/canvasClient.test.ts"})
MERGE (loadTest:Symbol {id:"frontend/src/lib/group-api/canvasClient.test.ts::load canvas snapshot from authorized group API",kind:"test_suite"})
MERGE (permissionTest:Symbol {id:"frontend/src/lib/group-api/canvasClient.test.ts::permission flag derives from scopes and roles",kind:"test_suite"})
MERGE (errorTest:Symbol {id:"frontend/src/lib/group-api/canvasClient.test.ts::conflict / readonly / disconnected error mapping",kind:"test_suite"})
MERGE (draftTest:Symbol {id:"frontend/src/lib/group-api/canvasClient.test.ts::draft store persists unconfirmed writes",kind:"test_suite"})
MERGE (client:Symbol {id:"frontend/src/lib/group-api/canvasClient.ts::GroupCanvasClient",kind:"class"})
MERGE (self)-[:TESTS]->(client)
@endcypher
*/
// =====================================================================
// 统一 Canvas Client 单测 (vitest, mock fetch 边界; 父代理合并时统一跑):
// - loadCanvas 走授权 API 组合 (group-context + canvases + elements)
// - 权限快照 (can_write) 由 granted_scopes + actor_roles 派生
// - 409 → CanvasConflictError / 403 → CanvasReadOnlyError / 网络 → CanvasDisconnectedError
// - CanvasDraftStore localStorage 草稿保留
// =====================================================================

import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  CanvasConflictError,
  CanvasDisconnectedError,
  CanvasDraftStore,
  CanvasReadOnlyError,
  GroupCanvasClient,
} from "./canvasClient";
import { GroupApiError, WorktreeGroupApiClient } from "@/lib/group/worktreeGroupApi";

const WORKTREE = "wt-unified";
const CANVAS = "canvas-server-1";

function jsonResponse(payload: unknown, status = 200): Response {
  return new Response(JSON.stringify(payload), {
    status,
    headers: { "content-type": "application/json" },
  });
}

function makeApi(handler: (url: string, init?: RequestInit) => Response): WorktreeGroupApiClient {
  const fetcher = vi.fn(async (url: string, init?: RequestInit) => handler(url, init));
  return new WorktreeGroupApiClient(async () => "test-token", fetcher as unknown as typeof fetch);
}

const groupContextPayload = (overrides: Record<string, unknown> = {}) => ({
  group_context: {
    tenant_id: "ten-1",
    workspace_id: "ws-1",
    project_id: "prj-1",
    repository_id: "repo-1",
    worktree_id: WORKTREE,
    actor_id: "usr-1",
    actor_kind: "user",
    actor_roles: ["developer"],
    granted_scopes: ["worktree:read", "worktree:write", "canvas:read", "canvas:write"],
    context_version: 1,
    resolved_at: "2026-11-20T00:00:00Z",
    permission_snapshot_ref: "b:v1",
    correlation_id: "corr-1",
    ...overrides,
  },
});

const canvasListPayload = () => ({
  worktree_id: WORKTREE,
  project_id: "prj-1",
  canvases: [
    {
      canvas_id: CANVAS,
      title: "共享画布",
      version: 3,
      created_by: "usr-1",
      created_at: "2026-11-20T00:00:00Z",
      element_count: 1,
      document: {
        version: 3,
        viewport: { x: 0, y: 0, zoom: 1 },
        frames: [],
        connectors: [],
      },
    },
  ],
});

const elementsPayload = () => ({
  canvas_id: CANVAS,
  worktree_id: WORKTREE,
  elements: [
    {
      element_id: "el-1",
      kind: "sticky_note",
      x: 10,
      y: 20,
      width: 180,
      height: 100,
      rotation: 0,
      z_index: 1,
      content: { text: "hello" },
      locked: false,
      hidden: false,
      version: 2,
      entity_ref: null,
    },
  ],
});

describe("GroupCanvasClient.loadCanvas — 授权真源快照", () => {
  it("组合 group-context + canvases + elements 返回统一快照", async () => {
    const api = makeApi((url) => {
      if (url.includes("/group-context")) return jsonResponse(groupContextPayload());
      if (url.endsWith("/canvases")) return jsonResponse(canvasListPayload());
      if (url.includes("/elements")) return jsonResponse(elementsPayload());
      return jsonResponse({}, 404);
    });
    const client = new GroupCanvasClient(api, WORKTREE);
    const snapshot = await client.loadCanvas();
    expect(snapshot.canvas?.canvasId).toBe(CANVAS);
    expect(snapshot.canvas?.version).toBe(3);
    expect(snapshot.elements).toHaveLength(1);
    expect(snapshot.elements[0].id).toBe("el-1");
    expect(snapshot.elements[0].version).toBe(2);
    expect(snapshot.permission.canWrite).toBe(true);
  });

  it("无画布时 canvas=null + elements 空 (诚实空态, 不伪造)", async () => {
    const api = makeApi((url) => {
      if (url.includes("/group-context")) return jsonResponse(groupContextPayload());
      if (url.endsWith("/canvases")) {
        return jsonResponse({ worktree_id: WORKTREE, project_id: "prj-1", canvases: [] });
      }
      return jsonResponse({}, 404);
    });
    const snapshot = await new GroupCanvasClient(api, WORKTREE).loadCanvas();
    expect(snapshot.canvas).toBeNull();
    expect(snapshot.elements).toHaveLength(0);
  });

  it("worktree 之外的回包被拒绝", async () => {
    const api = makeApi(() =>
      jsonResponse({ group_context: { worktree_id: "wt-other", project_id: "prj-1" } }),
    );
    await expect(new GroupCanvasClient(api, WORKTREE).loadCanvas()).rejects.toThrow(
      /Worktree 之外/,
    );
  });
});

describe("permission — can_write 派生与只读", () => {
  it("scope 缺 canvas:write → canWrite=false (只读按钮禁用)", async () => {
    const api = makeApi((url) => {
      if (url.includes("/group-context")) {
        return jsonResponse(
          groupContextPayload({
            actor_roles: ["member"],
            granted_scopes: ["worktree:read", "canvas:read"],
          }),
        );
      }
      if (url.includes("/elements")) return jsonResponse(elementsPayload());
      return jsonResponse(canvasListPayload());
    });
    const snapshot = await new GroupCanvasClient(api, WORKTREE).loadCanvas();
    expect(snapshot.permission.canWrite).toBe(false);
    expect(snapshot.permission.canCreateCanvas).toBe(false);
  });

  it("member 角色即使带 canvas:write scope 仍不可写 (角色授权真源)", async () => {
    const api = makeApi((url) => {
      if (url.includes("/group-context")) {
        return jsonResponse(
          groupContextPayload({
            actor_roles: ["member"],
            granted_scopes: ["worktree:read", "worktree:write", "canvas:read", "canvas:write"],
          }),
        );
      }
      if (url.includes("/elements")) return jsonResponse(elementsPayload());
      return jsonResponse(canvasListPayload());
    });
    const snapshot = await new GroupCanvasClient(api, WORKTREE).loadCanvas();
    expect(snapshot.permission.canWrite).toBe(false);
  });
});

describe("错误分层 — 409 冲突 / 403 只读 / 断线", () => {
  it("409 → CanvasConflictError (冲突双方草稿可见)", async () => {
    const api = new WorktreeGroupApiClient(async () => "t", async () =>
      jsonResponse({ error: { code: "version_conflict" } }, 409),
    );
    const client = new GroupCanvasClient(api, WORKTREE);
    await expect(client.createElement(CANVAS, {
      kind: "sticky_note",
      x: 1,
      y: 2,
      width: 100,
      height: 80,
      z_index: 1,
      content: {},
    }, "key-1")).rejects.toBeInstanceOf(CanvasConflictError);
  });

  it("403 → CanvasReadOnlyError (权限决定写按钮)", async () => {
    const api = new WorktreeGroupApiClient(async () => "t", async () =>
      jsonResponse({ error: { code: "insufficient_scope" } }, 403),
    );
    const client = new GroupCanvasClient(api, WORKTREE);
    await expect(client.updateElementPosition(CANVAS, "el-1", { x: 1, y: 2, expectedVersion: 1 }, "k")).rejects.toBeInstanceOf(
      CanvasReadOnlyError,
    );
  });

  it("网络失败 → CanvasDisconnectedError (触发断线重连提示)", async () => {
    const api = new WorktreeGroupApiClient(async () => "t", async () => {
      throw new TypeError("Failed to fetch");
    });
    const client = new GroupCanvasClient(api, WORKTREE);
    await expect(client.loadCanvas()).rejects.toBeInstanceOf(CanvasDisconnectedError);
  });

  it("直接 GroupApiError(401) → CanvasReadOnlyError (会话失效按只读处理)", async () => {
    const api = new WorktreeGroupApiClient(async () => "t", async () =>
      jsonResponse({ error: { code: "session_required" } }, 401),
    );
    const client = new GroupCanvasClient(api, WORKTREE);
    await expect(client.loadCanvas()).rejects.toBeInstanceOf(CanvasReadOnlyError);
  });
});

describe("CanvasDraftStore — 未确认草稿保留", () => {
  beforeEach(() => {
    globalThis.localStorage?.clear();
  });

  it("草稿写读清空往返", () => {
    CanvasDraftStore.save(WORKTREE, CANVAS, {
      kind: "element-move",
      elementId: "el-1",
      payload: { x: 120, y: 240, expectedVersion: 2 },
      savedAt: "2026-11-20T00:00:00Z",
    });
    const draft = CanvasDraftStore.load(WORKTREE, CANVAS);
    expect(draft?.kind).toBe("element-move");
    expect(draft?.elementId).toBe("el-1");
    CanvasDraftStore.clear(WORKTREE, CANVAS);
    expect(CanvasDraftStore.load(WORKTREE, CANVAS)).toBeNull();
  });

  it("损坏数据 → null (不冒充草稿)", () => {
    globalThis.localStorage?.setItem(
      CanvasDraftStore.storageKey(WORKTREE, CANVAS),
      "{broken json",
    );
    expect(CanvasDraftStore.load(WORKTREE, CANVAS)).toBeNull();
  });
});
