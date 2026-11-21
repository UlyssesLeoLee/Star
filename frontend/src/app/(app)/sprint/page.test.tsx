/*
@cypher schema=1 source_sha256=5402ba45b84c1ddb4e90eb9d2cdfcc630b921909f296a44c4aa75900f163405c
MERGE (self:File {path:"frontend/src/app/(app)/sprint/page.test.tsx"})
MERGE (localMode:Symbol {id:"frontend/src/app/(app)/sprint/page.test.tsx::local_mode_label",kind:"test"})
MERGE (serverMode:Symbol {id:"frontend/src/app/(app)/sprint/page.test.tsx::server_mode_renders_hydrated_truth",kind:"test"})
MERGE (conflict:Symbol {id:"frontend/src/app/(app)/sprint/page.test.tsx::conflict_toast_feedback",kind:"test"})
MERGE (self)-[:DEFINES]->(localMode)
MERGE (self)-[:DEFINES]->(serverMode)
MERGE (self)-[:DEFINES]->(conflict)
MERGE (conflict)-[:TESTS]->(:Symbol {id:"frontend/src/app/(app)/sprint/page.tsx::handleServerTransition",kind:"function"})
@endcypher
*/
// =====================================================================
// Issues Page Test (per U2 任务 — 8 测试)
// =====================================================================
// 8 个测试 (per spec):
//   1. 3 view 切换 (Sprint | List | Tree) — 3 个独立 test (per 2026-09-05 19:13 JST 拍板: Kanban 已删)
//   2. "+ New issue" button
//   3. "🔍" 搜索 button
//   4. 详情侧栏 (选中 work-item)
//
// 已知缺口 (per 缺标比错标):
//   - 创建表单 (Phase 2+ 接后端) — 仅测 ?new=true banner 显示
//   - 搜索过滤 (Phase 2+ 接 U1 CommandBar) — 仅测 toggle
//   - Tree 视图展开 (Phase 2+ 接 relations) — 不测
//   - Sprint 视图拖动 (G3 a11y) — Phase Mobile
// =====================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup, within, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import IssuesPage, { buildTree, type TreeNode } from "./page";
import { I18nProvider } from "@/lib/i18n";
import { WorktreeGroupApiProvider } from "@/lib/group/groupProjection";
import { RunDirectoryProvider } from "@/lib/run/runDirectorySession";
import { useNavStore } from "@/lib/nav/navStore";

vi.mock("react-hot-toast", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));
import { toast } from "react-hot-toast";

// per 2026-08-31 i18n 补缺口: PageHeader / Sidebar 等含 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

// ---- mock next/navigation ----
const mockPush = vi.fn();
const mockPathname = vi.fn(() => "/sprint");
const mockSearchParamsGet = vi.fn();

const mockSearchParams = {
  get: (k: string) => mockSearchParamsGet(k),
  toString: () => {
    // 简化: 假定 get 已调用, 我们用最近一次 get 重建
    // Per 2026-09-05 19:13 JST 拍板: default = sprint (kanban 已删除)
    return "view=sprint";
  },
  has: () => false,
  entries: () => [],
  keys: [] as any,
  values: [] as any,
  forEach: () => {},
  [Symbol.iterator]: function* () {},
};

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush, replace: vi.fn(), prefetch: vi.fn() }),
  usePathname: () => mockPathname(),
  useSearchParams: () => mockSearchParams,
}));

// ---- mock zustand store (per useStore 全局) ----
// 引入真实 store 即可, store 是单例 zustand; 我们 reset 状态
import { useStore } from "@/lib/store";
import * as seed from "@/lib/seed";
import type { WorkItem } from "@/types/ids";

const testWorkItems: WorkItem[] = [{
  id: "test-wi-001",
  tenant_id: "test-tenant",
  project_id: "test-project",
  key: "TEST-1",
  title: "Test issue",
  description: "",
  kind: "task",
  status: "todo",
  priority: "p2",
  reporter_id: "test-user",
  labels: [],
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
}];
const testBoard = {
  ...seed.board,
  columns: seed.board.columns.map((column) => ({
    ...column,
    work_item_ids: testWorkItems.filter((item) => item.status === column.status).map((item) => item.id),
  })),
};

const resetStore = () => {
  if (typeof window !== "undefined") {
    window.localStorage.clear();
  }
  useStore.setState({
    workItems: testWorkItems,
    board: testBoard,
    sprints: seed.sprints,
    identities: seed.identities,
  } as any);
};

describe("IssuesPage (U2)", () => {
  beforeEach(() => {
    mockPush.mockClear();
    mockPathname.mockReturnValue("/sprint");
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "sprint" : null));
    mockSearchParams.toString = () => "view=sprint";
    resetStore();
    cleanup();
  });
  afterEach(() => {
    vi.clearAllMocks();
  });

  // ---- Test 1: view 切换 (Sprint default, per 2026-09-05 19:13 JST 拍板) ----
  it("renders Sprint view by default with 3 view tabs (no Kanban)", () => {
    renderWithI18n(<IssuesPage />);
    // 3 tabs 渲染 (Kanban 已删除 per 2026-09-05 19:13 JST)
    expect(screen.getByTestId("issues-view-tab-sprint")).toBeTruthy();
    expect(screen.getByTestId("issues-view-tab-list")).toBeTruthy();
    expect(screen.getByTestId("issues-view-tab-tree")).toBeTruthy();
    // Kanban 不应再存在
    expect(screen.queryByTestId("issues-view-tab-kanban")).toBeNull();

    // default Sprint tab aria-selected=true
    const sprintTab = screen.getByTestId("issues-view-tab-sprint");
    expect(sprintTab.getAttribute("aria-selected")).toBe("true");

    // Sprint view 渲染 (per 2026-09-05 19:13 JST 拍板: 默认 sprint)
    expect(screen.getByTestId("issues-view-sprint")).toBeTruthy();
    // KanbanBoard 不应再存在
    expect(screen.queryByTestId("kanban-board")).toBeNull();
  });

  // ---- Test 2: view 切换 (List) ----
  it("renders List view with table when view=list", () => {
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "list" : null));
    mockSearchParams.toString = () => "view=list";
    renderWithI18n(<IssuesPage />);
    expect(screen.getByTestId("issues-view-list")).toBeTruthy();
    expect(screen.getByTestId("issues-list-table")).toBeTruthy();
  });

  // ---- Test 3: view 切换 (Tree) ----
  it("renders Tree view with hierarchical table when view=tree", () => {
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "tree" : null));
    mockSearchParams.toString = () => "view=tree";
    renderWithI18n(<IssuesPage />);
    expect(screen.getByTestId("issues-view-tree")).toBeTruthy();
    expect(screen.getByTestId("issues-tree")).toBeTruthy();
  });

  // ---- Test 4: view 切换 (Sprint) — 看板视图 (per 2026-09-05 19:32 JST 拍板) ----
  it("renders the Sprint board view (sprint-board-view + sprint-backlog) when view=sprint", () => {
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "sprint" : null));
    mockSearchParams.toString = () => "view=sprint";
    renderWithI18n(<IssuesPage />);
    expect(screen.getByTestId("issues-view-sprint")).toBeTruthy();
    // SprintBoardView 接管
    expect(screen.getByTestId("sprint-board-view")).toBeTruthy();
    expect(screen.getByTestId("sprint-backlog")).toBeTruthy();
  });

  // ---- Test 5: 独立 New issue 按钮已移除 (per 2026-11-21 用户发令:
  //      "backlog栏可以直接增加issue，不需要单独的new issue按钮") ----
  it("has no standalone New issue button; backlog quick-add replaces it", () => {
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "sprint" : null));
    mockSearchParams.toString = () => "view=sprint";
    renderWithI18n(<IssuesPage />);
    expect(screen.queryByTestId("issues-new-button")).toBeNull();
    // Backlog 内联新增入口存在
    expect(screen.getByTestId("sprint-backlog-quick-add")).toBeTruthy();
    expect(screen.getByTestId("sprint-backlog-quick-input")).toBeTruthy();
    expect(screen.getByTestId("sprint-backlog-quick-submit")).toBeTruthy();
  });

  // ---- Test 6: "🔍" 搜索 button ----
  it("renders search button that toggles search bar", () => {
    renderWithI18n(<IssuesPage />);
    const btn = screen.getByTestId("issues-search-button");
    expect(btn).toBeTruthy();

    // 默认 search bar 不在
    expect(screen.queryByTestId("issues-search-bar")).toBeNull();

    // 点击后 search bar 出现
    fireEvent.click(btn);
    expect(screen.getByTestId("issues-search-bar")).toBeTruthy();
    expect(screen.getByTestId("issues-search-input")).toBeTruthy();

    // 再点折叠
    fireEvent.click(btn);
    expect(screen.queryByTestId("issues-search-bar")).toBeNull();
  });

  // ---- Test 7: 详情侧栏 (选中 work-item) ----
  it("shows 320px detail sidebar when a work-item row is clicked in List view", () => {
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "list" : null));
    mockSearchParams.toString = () => "view=list";
    renderWithI18n(<IssuesPage />);

    // 默认 detail sidebar 不在
    expect(screen.queryByTestId("issues-detail-sidebar")).toBeNull();

    // 点击第一行 (test-wi-001 在 todo 列, 应该出现在 list 中)
    const row = screen.getByTestId("issues-list-row-test-wi-001");
    fireEvent.click(row);

    // 详情侧栏出现 + 含 transition 按钮
    const sidebar = screen.getByTestId("issues-detail-sidebar");
    expect(sidebar).toBeTruthy();
    expect(within(sidebar).getByTestId("issues-detail-transitions")).toBeTruthy();

    // 关闭按钮可点
    fireEvent.click(within(sidebar).getByTestId("issues-detail-close"));
    expect(screen.queryByTestId("issues-detail-sidebar")).toBeNull();
  });

  // ---- Test 8: ?new=true 深链不再渲染独立表单 (per 2026-11-21 用户发令:
  //      Backlog 内联新增替代独立 new issue 表单) ----
  it("?new=true no longer renders standalone banner; backlog quick-add is available", () => {
    mockSearchParamsGet.mockImplementation((k: string) => {
      // Per 2026-09-05 19:13 JST: view 默认 sprint, kanban 已删
      if (k === "view") return "sprint";
      if (k === "new") return "true";
      return null;
    });
    mockSearchParams.toString = () => "view=sprint&new=true";
    renderWithI18n(<IssuesPage />);
    // 旧独立表单缺席
    expect(screen.queryByTestId("issues-new-banner")).toBeNull();
    expect(screen.queryByTestId("issues-new-title-input")).toBeNull();
    // 内联新增入口在 Backlog 中可用
    expect(screen.getByTestId("sprint-backlog-quick-add")).toBeTruthy();
  });
});

// =====================================================================
// buildTree (per 2026-11-17 功能补齐: Tree 视图层级改用 parent_id 真实父子关系)
// =====================================================================
describe("buildTree (parent_id 层级)", () => {
  const mk = (over: Partial<WorkItem> & { id: string; key: string }): WorkItem => ({
    id: over.id,
    tenant_id: "t",
    project_id: "p",
    key: over.key,
    title: over.key,
    description: "",
    kind: "task",
    status: "todo",
    priority: "p2",
    reporter_id: "u",
    labels: [],
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...over,
  });
  const flatten = (nodes: TreeNode[]): string[] =>
    nodes.flatMap((n) => [n.id, ...flatten(n.children)]);

  it("epic → story → task 链 (parent_id 真实父子关系)", () => {
    const items = [
      mk({ id: "e1", key: "E-1", kind: "epic" }),
      mk({ id: "s1", key: "S-1", kind: "story", parent_id: "e1" }),
      mk({ id: "t1", key: "T-1", kind: "task", parent_id: "s1" }),
    ];
    const tree = buildTree(items);
    expect(tree.map((n) => n.id)).toEqual(["e1"]);
    expect(tree[0].depth).toBe(0);
    expect(tree[0].children.map((c) => c.id)).toEqual(["s1"]);
    expect(tree[0].children[0].depth).toBe(1);
    expect(tree[0].children[0].children.map((c) => c.id)).toEqual(["t1"]);
    expect(tree[0].children[0].children[0].depth).toBe(2);
  });

  it("无 parent_id 旧数据: 非 epic 归第一个 epic 下 (旧行为兜底)", () => {
    const items = [
      mk({ id: "t1", key: "T-1" }),
      mk({ id: "e1", key: "E-1", kind: "epic" }),
      mk({ id: "t2", key: "T-2", kind: "spike" }),
    ];
    const tree = buildTree(items);
    expect(tree.map((n) => n.id)).toEqual(["e1"]);
    expect(tree[0].children.map((c) => c.id)).toEqual(["t1", "t2"]);
    expect(tree[0].children[0].depth).toBe(1);
  });

  it("parent 断链 → 视为孤儿, 归第一个 epic (防丢节点)", () => {
    const items = [
      mk({ id: "t1", key: "T-1", parent_id: "ghost" }),
      mk({ id: "e1", key: "E-1", kind: "epic" }),
    ];
    const tree = buildTree(items);
    expect(flatten(tree).sort()).toEqual(["e1", "t1"]);
  });

  it("parent 链成环 → 环上节点自当根, 不丢", () => {
    const items = [
      mk({ id: "a", key: "A-1", parent_id: "b" }),
      mk({ id: "b", key: "B-1", parent_id: "a" }),
    ];
    const tree = buildTree(items);
    expect(flatten(tree).sort()).toEqual(["a", "b"]);
  });

  it("空数组安全", () => {
    expect(buildTree([])).toEqual([]);
  });
});

// =====================================================================
// P1: 服务端真源通道 (本地模式标注 / 服务端渲染 / 冲突反馈)
// 不跑 (frontend worktree 无 node_modules), 父代理合并时统一跑 vitest。
// mock 的是网络边界 (fetch), 不是业务数据 (per 共享约束前端纪律)。
// =====================================================================
const P1_WORKTREE_ID = "11111111-1111-4111-8111-111111111111";
const P1_WORK_ITEM_ID = "22222222-2222-4222-8222-222222222222";

function p1ServerItem() {
  return {
    work_item_id: P1_WORK_ITEM_ID,
    item_type: "task",
    title: "P1 统一卡",
    description: "",
    priority: "medium",
    labels: [],
    reporter_user_id: "44444444-4444-4444-8444-444444444444",
    lifecycle: {
      status: "pending",
      review_state: "none",
      active_worktree_id: null,
      version: 1,
      updated_at: new Date().toISOString(),
    },
    metadata_updated_at: new Date().toISOString(),
  };
}

function renderP1Harness(fetcher: typeof fetch) {
  const token = "user-jwt";
  return renderWithI18n(
    <RunDirectoryProvider session={{ generation: "p1", getAccessToken: async () => token, fetcher }}>
      <WorktreeGroupApiProvider getAccessToken={async () => token} sessionKey="p1" fetcher={fetcher}>
        <IssuesPage />
      </WorktreeGroupApiProvider>
    </RunDirectoryProvider>,
  );
}

describe("IssuesPage P1 统一通道", () => {
  beforeEach(() => {
    mockPush.mockClear();
    mockPathname.mockReturnValue("/sprint");
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "sprint" : null));
    mockSearchParams.toString = () => "view=sprint";
    resetStore();
    useNavStore.setState({ selectedWorktreeId: "", selectedProjectId: "", selectedLaneId: "", selectedBranch: "" });
    cleanup();
  });
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("本地模式标注: 无宿主会话时明确标注「本地模式」, 不冒充服务端数据", () => {
    renderWithI18n(<IssuesPage />);
    const source = screen.getByTestId("sprint-data-source");
    expect(source.textContent).toContain("本地模式");
  });

  it("服务端模式渲染: 宿主会话 + Worktree 上下文齐备时拉取服务端真源并水合共享视图", async () => {
    useNavStore.setState({ selectedWorktreeId: P1_WORKTREE_ID });
    const fetcher = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes(`/api/v1/worktrees/${P1_WORKTREE_ID}/work-items`)) {
        return new Response(
          JSON.stringify({
            worktree_id: P1_WORKTREE_ID,
            project_id: "33333333-3333-4333-8333-333333333333",
            work_items: [p1ServerItem()],
          }),
          { status: 200 },
        );
      }
      return new Response(JSON.stringify({ error: { code: "not_found" } }), { status: 404 });
    }) as unknown as typeof fetch;

    renderP1Harness(fetcher);

    const source = await screen.findByTestId("sprint-data-source");
    expect(source.textContent).toContain("服务端真源模式");
    expect(source.textContent).toContain(P1_WORKTREE_ID);
    // 服务端真源水合进 store (SprintBoardView / List / Tree 共享视图读 store 渲染)
    await waitFor(() => {
      expect(useStore.getState().workItems.some((w) => w.id === P1_WORK_ITEM_ID)).toBe(true);
    });
    expect(useStore.getState().workItems.find((w) => w.id === P1_WORK_ITEM_ID)?.status).toBe("todo");
  });

  it("冲突反馈: 详情侧栏状态转换遇 409 → 明确 toast「版本冲突, 已刷新最新状态」", async () => {
    useNavStore.setState({ selectedWorktreeId: P1_WORKTREE_ID });
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "list" : null));
    mockSearchParams.toString = () => "view=list";
    const fetcher = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.includes(`/api/v1/worktrees/${P1_WORKTREE_ID}/work-items`) && init?.method !== "POST") {
        return new Response(
          JSON.stringify({
            worktree_id: P1_WORKTREE_ID,
            project_id: "33333333-3333-4333-8333-333333333333",
            work_items: [p1ServerItem()],
          }),
          { status: 200 },
        );
      }
      if (url.includes("/lifecycle") && init?.method === "POST") {
        return new Response(JSON.stringify({ error: { code: "version_conflict" } }), { status: 409 });
      }
      return new Response(JSON.stringify({ error: { code: "not_found" } }), { status: 404 });
    }) as unknown as typeof fetch;

    renderP1Harness(fetcher);

    // 水合后 List 行出现 → 点击选中 → 详情侧栏
    const row = await screen.findByTestId(`issues-list-row-${P1_WORK_ITEM_ID}`);
    fireEvent.click(row);
    const sidebar = screen.getByTestId("issues-detail-sidebar");
    fireEvent.click(within(sidebar).getByTestId("issues-detail-transition-in_progress"));

    await waitFor(() => expect(toast.error).toHaveBeenCalledWith("版本冲突, 已刷新最新状态"));
  });
});
