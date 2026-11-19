// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/canvas/[id]/page.test.tsx"})
MERGE (groupWrite:Symbol {id:"frontend/src/app/canvas/[id]/page.test.tsx::group 可写模式把三个写回调交给 CanvasView",kind:"test_suite"})
MERGE (groupReadOnly:Symbol {id:"frontend/src/app/canvas/[id]/page.test.tsx::group 只读模式禁用全部写回调并提示",kind:"test_suite"})
MERGE (localMode:Symbol {id:"frontend/src/app/canvas/[id]/page.test.tsx::无 group 会话回落本地模式并明确标注",kind:"test_suite"})
MERGE (conflict:Symbol {id:"frontend/src/app/canvas/[id]/page.test.tsx::冲突时展示双方内容对话框",kind:"test_suite"})
MERGE (disconnect:Symbol {id:"frontend/src/app/canvas/[id]/page.test.tsx::断线显示重试中横幅",kind:"test_suite"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/[id]/page.tsx::CanvasDetailPage",kind:"component"})
MERGE (self)-[:TESTS]->(page)
@endcypher
*/
// =====================================================================
// /canvas/[id] 页测试 (vitest + @testing-library/react; 父代理合并时统一跑):
// - group 可写 → CanvasView 收到 onCreateElement/onDeleteElements/
//   onElementPositionChange + readOnly=false (移动与新增同一权限)
// - group 只读 → 三个写回调 undefined + readOnly=true + 只读提示
// - 无 group 会话 → 本地降级 + 「本地画布 — 未连接服务端」标注
// - 移动冲突 → 冲突对话框展示双方内容
// - wake 断线 → 「连接中断, 重试中」横幅
// =====================================================================

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

// ---------------- 受控 mock 状态 (vi.hoisted, 工厂可安全引用) ----------------
const mocks = vi.hoisted(() => ({
  groupApiPresent: false,
  snapshotCanWrite: true,
  snapshotHasCanvas: true,
  loadRejects: null as Error | null,
  wakeOnError: null as ((error: unknown) => void) | null,
  wakeUnsubscribe: vi.fn(),
  lastCanvasViewProps: null as Record<string, unknown> | null,
  createElementImpl: vi.fn(async () => ({ id: "el-new" })),
  moveImpl: vi.fn(async () => undefined),
  storeState: {
    canvases: [
      {
        id: "canvas-1",
        tenant_id: "ten-1",
        workspace_id: "ws-1",
        title: "本地画布",
        ref_kind: "worktree",
        ref_id: "wt-001",
        viewport: { x: 0, y: 0, zoom: 1 },
        frames: [],
        creator_id: "usr-001",
        collaborator_ids: ["usr-001"],
        created_at: "2026-11-20T00:00:00Z",
        updated_at: "2026-11-20T00:00:00Z",
      },
    ],
    canvasElements: [],
    canvasConnectors: [],
    workItems: [],
    worktrees: [{ id: "wt-001", project_id: "prj-1", lane_id: "lane-1" }],
    projects: [],
    lanes: [{ id: "lane-1", project_id: "prj-1" }],
  },
}));

const snapshot = () => ({
  worktreeId: "wt-001",
  projectId: "prj-1",
  canvas: mocks.snapshotHasCanvas
    ? {
        canvasId: "canvas-server-1",
        title: "服务端画布",
        version: 3,
        viewport: { x: 0, y: 0, zoom: 1 },
        frames: [],
        connectors: [],
      }
    : null,
  elements: [
    {
      id: "el-1",
      canvas_id: "canvas-server-1",
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
    },
  ],
  permission: {
    canWrite: mocks.snapshotCanWrite,
    canCreateCanvas: mocks.snapshotCanWrite,
    roles: ["developer"],
    scopes: ["canvas:read", "canvas:write"],
  },
});

vi.mock("@/lib/store", () => {
  const useStoreMock = vi.fn((selector: (state: unknown) => unknown) => selector(mocks.storeState));
  useStoreMock.getState = vi.fn(() => ({
    worktrees: mocks.storeState.worktrees,
    addCanvasElement: vi.fn(),
  }));
  return { useStore: useStoreMock };
});

vi.mock("@/lib/nav/navStore", () => ({
  useNavStore: (selector: (state: Record<string, unknown>) => unknown) =>
    selector({
      selectedProjectId: "prj-1",
      selectedLaneId: "lane-1",
      selectedWorktreeId: "wt-001",
    }),
}));

vi.mock("@/lib/group/groupProjection", () => ({
  useWorktreeGroupApi: () => (mocks.groupApiPresent ? { worktreeId: "wt-001" } : null),
  useWorktreeGroupSessionState: () =>
    mocks.groupApiPresent ? { status: "ready" } : { status: "unavailable" },
}));

vi.mock("@/lib/group-api/canvasClient", () => {
  class CanvasConflictErrorMock extends Error {
    readonly status = 409;
    constructor(readonly code: string, message: string) {
      super(message);
      this.name = "CanvasConflictError";
    }
  }
  class CanvasDisconnectedErrorMock extends Error {
    constructor(message: string) {
      super(message);
      this.name = "CanvasDisconnectedError";
    }
  }
  class CanvasReadOnlyErrorMock extends Error {
    readonly status = 403;
    constructor(message: string) {
      super(message);
      this.name = "CanvasReadOnlyError";
    }
  }
  class GroupCanvasClientMock {
    worktreeId: string;
    constructor(_api: unknown, worktreeId: string) {
      this.worktreeId = worktreeId;
    }
    async loadCanvas() {
      if (mocks.loadRejects) throw mocks.loadRejects;
      return snapshot();
    }
    createElement = mocks.createElementImpl;
    updateElementPosition = mocks.moveImpl;
    deleteElement = vi.fn(async () => undefined);
    createCanvas = vi.fn(async () => ({ canvasId: "canvas-server-1" }));
    makeWakeSubscriber() {
      return {
        subscribe: (_signal: AbortSignal, onError: (error: unknown) => void) => {
          mocks.wakeOnError = onError;
          return mocks.wakeUnsubscribe;
        },
      };
    }
  }
  return {
    CanvasConflictError: CanvasConflictErrorMock,
    CanvasDisconnectedError: CanvasDisconnectedErrorMock,
    CanvasReadOnlyError: CanvasReadOnlyErrorMock,
    CanvasDraftStore: {
      save: vi.fn(),
      clear: vi.fn(),
      load: vi.fn(() => null),
      storageKey: () => "star:canvas-draft:v1:test",
    },
    GroupCanvasClient: GroupCanvasClientMock,
  };
});

vi.mock("@/components/CanvasView", () => ({
  CanvasView: (props: Record<string, unknown>) => {
    mocks.lastCanvasViewProps = props;
    return <div data-testid="canvas-view-mock" />;
  },
}));

vi.mock("@/components/canvas/CanvasScopeSelector", () => ({
  CanvasScopeSelector: () => <div data-testid="scope-selector-mock" />,
}));

vi.mock("@/components/agent-view/AgentViewPage", () => ({
  AgentViewPage: () => <div data-testid="agent-page-mock" />,
}));

vi.mock("@/components/agent-workflow/WorkflowBuilder", () => ({
  WorkflowBuilder: () => <div data-testid="workflow-builder-mock" />,
}));

vi.mock("@/app/(app)/agent-relationships/view/RelationshipView", () => ({
  RelationshipView: () => <div data-testid="relationship-view-mock" />,
}));

vi.mock("../components/GitCanvas", () => ({
  GitCanvas: () => <div data-testid="git-canvas-mock" />,
}));

import CanvasDetailPage from "./page";

function renderPage() {
  return render(<CanvasDetailPage params={Promise.resolve({ id: "canvas-1" })} />);
}

describe("/canvas/[id] — group 模式接线", () => {
  beforeEach(() => {
    mocks.groupApiPresent = true;
    mocks.snapshotCanWrite = true;
    mocks.snapshotHasCanvas = true;
    mocks.loadRejects = null;
    mocks.wakeOnError = null;
    mocks.wakeUnsubscribe = vi.fn();
    mocks.lastCanvasViewProps = null;
    mocks.createElementImpl = vi.fn(async () => ({ id: "el-new" }));
    mocks.moveImpl = vi.fn(async () => undefined);
  });

  it("可写模式把三个写回调交给 CanvasView, readOnly=false (移动与新增同一权限)", async () => {
    renderPage();
    await waitFor(() => expect(mocks.lastCanvasViewProps).not.toBeNull());
    expect(mocks.lastCanvasViewProps?.readOnly).toBe(false);
    expect(typeof mocks.lastCanvasViewProps?.onCreateElement).toBe("function");
    expect(typeof mocks.lastCanvasViewProps?.onElementPositionChange).toBe("function");
    expect(typeof mocks.lastCanvasViewProps?.onDeleteElements).toBe("function");
    expect(screen.queryByTestId("canvas-group-readonly-hint")).toBeNull();
  });

  it("只读模式三个写回调全部 undefined + readOnly=true + 只读提示", async () => {
    mocks.snapshotCanWrite = false;
    renderPage();
    await waitFor(() => expect(mocks.lastCanvasViewProps).not.toBeNull());
    expect(mocks.lastCanvasViewProps?.readOnly).toBe(true);
    expect(mocks.lastCanvasViewProps?.onCreateElement).toBeUndefined();
    expect(mocks.lastCanvasViewProps?.onElementPositionChange).toBeUndefined();
    expect(mocks.lastCanvasViewProps?.onDeleteElements).toBeUndefined();
    await waitFor(() => expect(screen.getByTestId("canvas-group-readonly-hint")).toBeTruthy());
  });

  it("新增走 GroupCanvasClient.createElement", async () => {
    renderPage();
    await waitFor(() => expect(mocks.lastCanvasViewProps).not.toBeNull());
    const create = mocks.lastCanvasViewProps?.onCreateElement as (
      input: unknown,
    ) => Promise<{ id?: string } | void>;
    const result = await create({
      kind: "sticky_note",
      x: 10,
      y: 20,
      width: 180,
      height: 100,
      z_index: 1,
      content: {},
    });
    expect(mocks.createElementImpl).toHaveBeenCalledTimes(1);
    expect(result).toEqual({ id: "el-new" });
  });

  it("移动冲突 → 冲突对话框展示双方内容, 保留双方草稿可关闭", async () => {
    class Conflict extends Error {
      readonly status = 409;
      readonly code = "version_conflict";
      constructor() {
        super("conflict");
        this.name = "CanvasConflictError";
      }
    }
    mocks.moveImpl = vi.fn(async () => {
      throw new Conflict();
    });
    renderPage();
    await waitFor(() => expect(mocks.lastCanvasViewProps).not.toBeNull());
    const move = mocks.lastCanvasViewProps?.onElementPositionChange as (
      position: unknown,
    ) => Promise<void>;
    await move({ elementId: "el-1", x: 300, y: 300, expectedVersion: 2 }).catch(() => undefined);
    await waitFor(() => expect(screen.getByTestId("canvas-conflict-dialog")).toBeTruthy());
    expect(screen.getByTestId("canvas-conflict-server").textContent).toContain("version_conflict");
    expect(screen.getByTestId("canvas-conflict-draft").textContent).toContain("element-move");
    fireEvent.click(screen.getByTestId("canvas-conflict-keep-both"));
    await waitFor(() => expect(screen.queryByTestId("canvas-conflict-dialog")).toBeNull());
  });

  it("wake 断线 → 「连接中断, 重试中」横幅", async () => {
    renderPage();
    await waitFor(() => expect(mocks.wakeOnError).not.toBeNull());
    mocks.wakeOnError?.(new Error("stream closed"));
    await waitFor(() =>
      expect(screen.getByTestId("canvas-connection-reconnecting").textContent).toContain(
        "连接中断",
      ),
    );
  });

  it("无服务端画布 → 诚实空态 + 创建按钮 (canCreateCanvas)", async () => {
    mocks.snapshotHasCanvas = false;
    renderPage();
    await waitFor(() => expect(screen.getByTestId("canvas-group-no-canvas-hint")).toBeTruthy());
    expect(screen.getByTestId("canvas-group-create-canvas")).toBeTruthy();
  });
});

describe("/canvas/[id] — local 模式降级", () => {
  beforeEach(() => {
    mocks.groupApiPresent = false;
    mocks.lastCanvasViewProps = null;
    mocks.loadRejects = null;
  });

  it("无 group 会话 → store 通道 + 「本地画布 — 未连接服务端」标注", async () => {
    renderPage();
    await waitFor(() => expect(mocks.lastCanvasViewProps).not.toBeNull());
    expect(screen.getByTestId("canvas-local-mode-badge").textContent).toContain("本地画布");
    expect(mocks.lastCanvasViewProps?.readOnly).toBe(false);
    expect(typeof mocks.lastCanvasViewProps?.onCreateElement).toBe("function");
  });
});
