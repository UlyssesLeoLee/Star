/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/(app)/kanban/page.test.tsx"})
MERGE (localMode:Symbol {id:"frontend/src/app/(app)/kanban/page.test.tsx::shows_local_mode_label_without_host_session",kind:"test"})
MERGE (serverMode:Symbol {id:"frontend/src/app/(app)/kanban/page.test.tsx::renders_server_truth_when_host_session_and_worktree_context_are_present",kind:"test"})
MERGE (conflict:Symbol {id:"frontend/src/app/(app)/kanban/page.test.tsx::move_conflict_shows_explicit_feedback",kind:"test"})
MERGE (self)-[:DEFINES]->(localMode)
MERGE (self)-[:DEFINES]->(serverMode)
MERGE (self)-[:DEFINES]->(conflict)
MERGE (conflict)-[:TESTS]->(:Symbol {id:"frontend/src/app/(app)/kanban/page.tsx::handleServerTransition",kind:"function"})
@endcypher
*/
// =====================================================================
// Kanban Page Test — P1 服务端真源接线 (本地模式标注 / 服务端渲染 / 冲突反馈)
// 不跑 (frontend worktree 无 node_modules), 父代理合并时统一跑 vitest。
// mock 的是网络边界 (fetch), 不是业务数据 (per 共享约束前端纪律)。
// =====================================================================

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import KanbanPage from "./page";
import { I18nProvider } from "@/lib/i18n";
import { WorktreeGroupApiProvider } from "@/lib/group/groupProjection";
import { RunDirectoryProvider } from "@/lib/run/runDirectorySession";
import { useStore } from "@/lib/store";
import { useNavStore } from "@/lib/nav/navStore";

vi.mock("react-hot-toast", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));
import { toast } from "react-hot-toast";

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), prefetch: vi.fn() }),
}));

const WORKTREE_ID = "11111111-1111-4111-8111-111111111111";
const WORK_ITEM_ID = "22222222-2222-4222-8222-222222222222";
const PROJECT_ID = "33333333-3333-4333-8333-333333333333";

function serverItem(overrides: Record<string, unknown> = {}) {
  return {
    work_item_id: WORK_ITEM_ID,
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
    ...overrides,
  };
}

function listEnvelope(items: unknown[]) {
  return { worktree_id: WORKTREE_ID, project_id: PROJECT_ID, work_items: items };
}

function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

function renderServerHarness(fetcher: typeof fetch) {
  const token = "user-jwt";
  return renderWithI18n(
    <RunDirectoryProvider session={{ generation: "s-1", getAccessToken: async () => token, fetcher }}>
      <WorktreeGroupApiProvider getAccessToken={async () => token} sessionKey="s-1" fetcher={fetcher}>
        <KanbanPage />
      </WorktreeGroupApiProvider>
    </RunDirectoryProvider>,
  );
}

beforeEach(() => {
  useNavStore.setState({ selectedWorktreeId: "", selectedProjectId: "", selectedLaneId: "", selectedBranch: "" });
  useStore.setState({
    workItems: [],
    board: { id: "b-1", tenant_id: "t-1", project_id: "p-1", name: "board", columns: [] },
  });
  window.history.replaceState(null, "", "/kanban");
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("KanbanPage P1 统一通道", () => {
  it("本地模式标注: 无宿主会话时显示本地全局数据 (store.workItems), 不再显示 unavailable 占位", () => {
    renderWithI18n(<KanbanPage />);
    const source = screen.getByTestId("kanban-data-source");
    expect(source.textContent).toContain("本地模式");
    // per 2026-11-22 用户发令: 未接入时显示本地全局数据，不显示 unavailable 提示
    expect(screen.queryByTestId("kanban-unavailable")).not.toBeInTheDocument();
    // 看板仍渲染 (可能显示本地 seed 数据)
    expect(screen.getByTestId("kanban-board-view")).toBeInTheDocument();
  });

  it("服务端模式渲染: 宿主会话 + Worktree 上下文齐备时渲染授权服务端投影", async () => {
    useNavStore.setState({ selectedWorktreeId: WORKTREE_ID });
    const fetcher = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes(`/api/v1/worktrees/${WORKTREE_ID}/work-items`)) {
        return new Response(JSON.stringify(listEnvelope([serverItem()])), { status: 200 });
      }
      return new Response(JSON.stringify({ error: { code: "not_found" } }), { status: 404 });
    }) as unknown as typeof fetch;

    renderServerHarness(fetcher);

    const source = await screen.findByTestId("kanban-data-source");
    expect(source.textContent).toContain("服务端真源模式");
    expect(source.textContent).toContain(WORKTREE_ID);
    // 服务端真源卡片出现在统一列内 (todo = pending)
    expect(await screen.findByText("P1 统一卡")).toBeInTheDocument();
    expect(screen.getByTestId("kanban-column-todo")).toBeInTheDocument();
    expect(screen.getByTestId("kanban-column-in_progress")).toBeInTheDocument();
    expect(screen.getByTestId("kanban-column-blocked")).toBeInTheDocument();
    expect(screen.getByTestId("kanban-column-done")).toBeInTheDocument();
    expect(screen.getByTestId("kanban-column-wontfix")).toBeInTheDocument();
    // 列编辑/列内新建在服务端模式隐藏 (本地 board 语义不产生与真源分裂的写)
    expect(screen.queryByTestId("kanban-add-column")).not.toBeInTheDocument();
  });

  it("冲突反馈: 服务端 409 版本冲突 → 明确 toast「版本冲突, 已刷新最新状态」", async () => {
    useNavStore.setState({ selectedWorktreeId: WORKTREE_ID });
    const fetcher = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.includes(`/api/v1/worktrees/${WORKTREE_ID}/work-items`) && init?.method !== "POST") {
        return new Response(JSON.stringify(listEnvelope([serverItem()])), { status: 200 });
      }
      if (url.includes("/lifecycle") && init?.method === "POST") {
        return new Response(JSON.stringify({ error: { code: "version_conflict" } }), { status: 409 });
      }
      return new Response(JSON.stringify({ error: { code: "not_found" } }), { status: 404 });
    }) as unknown as typeof fetch;

    renderServerHarness(fetcher);

    await screen.findByText("P1 统一卡");
    const sharedTransfer = new Map<string, string>();
    const dataTransfer = {
      setData: (kind: string, value: string) => {
        sharedTransfer.set(kind, value);
      },
      getData: (kind: string) => sharedTransfer.get(kind) ?? "",
      effectAllowed: "all",
      dropEffect: "move",
    };
    const card = document.querySelector('[draggable="true"]');
    expect(card).not.toBeNull();
    fireEvent.dragStart(card!, { dataTransfer });
    fireEvent.drop(screen.getByTestId("kanban-column-in_progress"), { dataTransfer });

    await waitFor(() => expect(toast.error).toHaveBeenCalledWith("版本冲突, 已刷新最新状态"));
  });

  it("深链 ?work_item= : 服务端真源水合后打开统一详情 Drawer (Lane 任务卡入口)", async () => {
    useNavStore.setState({ selectedWorktreeId: WORKTREE_ID });
    window.history.replaceState(null, "", `/kanban?work_item=${WORK_ITEM_ID}`);
    const fetcher = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes(`/api/v1/worktrees/${WORKTREE_ID}/work-items`)) {
        return new Response(JSON.stringify(listEnvelope([serverItem()])), { status: 200 });
      }
      return new Response(JSON.stringify({ error: { code: "not_found" } }), { status: 404 });
    }) as unknown as typeof fetch;

    renderServerHarness(fetcher);

    // 水合完成后 Drawer 打开, 展示同一张服务端真源卡
    const drawer = await screen.findByTestId("work-item-drawer-root", {}, { timeout: 5000 });
    expect(drawer).toBeInTheDocument();
    expect(await screen.findByTestId("work-item-drawer-title")).toHaveValue("P1 统一卡");
  });

  it("深链 ?work_item= 指向未水合的卡时不开空 Drawer (本地模式诚实留痕)", async () => {
    window.history.replaceState(null, "", `/kanban?work_item=${WORK_ITEM_ID}`);
    renderWithI18n(<KanbanPage />);
    expect(screen.getByTestId("kanban-data-source").textContent).toContain("本地模式");
    // 给 effect 充分时间; store 无此卡 → 不开 Drawer
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(screen.queryByTestId("work-item-drawer-root")).not.toBeInTheDocument();
  });
});
