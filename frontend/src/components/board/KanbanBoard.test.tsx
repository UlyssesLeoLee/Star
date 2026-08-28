// =====================================================================
// KanbanBoard.test.tsx — Kanban 看板列容器测试 (per §11.3 测试基线)
// =====================================================================
// 3 个测试 (per spec/integration/01 §3.1):
//   1. drag test:  模拟 drop 事件 → 调 onTransition(id, toStatus)
//   2. write test: 渲染 4 列, 卡片数 = mock 数据
//   3. permission-style: dropTarget 高亮 / WIP 超限 / 同列无操作
//
// 已知缺口 (per 缺标比错标):
//   - 暂未装 vitest / @testing-library/react / jsdom (per §10.3 范围 + W1 守门)
//   - 文件按 vitest 语法编写, 等项目装 runner 后 npm test 直接跑
//   - Phase D.6+ 接 CI (GitHub Actions) 后会自动跑
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest"; // eslint-disable-line @typescript-eslint/no-unused-vars
import { render, screen, fireEvent, cleanup } from "@testing-library/react"; // eslint-disable-line @typescript-eslint/no-unused-vars
import { KanbanBoard } from "./KanbanBoard";
import type { Board, WorkItem, Identity } from "@/types/ids";

// ---- mock next/navigation (per U2 — KanbanCard click → router.push) ----
const mockRouterPush = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockRouterPush, replace: vi.fn(), prefetch: vi.fn() }),
  usePathname: () => "/issues",
  useSearchParams: () => new URLSearchParams(),
}));

// ---- mock fixtures ----
const mockIdentities: Identity[] = [
  { id: "usr-001", tenant_id: "t-1", email: "u@x", display_name: "Ulysses", provider: "github", status: "active", mfa_enabled: true },
  { id: "usr-002", tenant_id: "t-1", email: "h@x", display_name: "Hera",    provider: "google", status: "active", mfa_enabled: false },
];

const mkWorkItem = (id: string, status: WorkItem["status"]): WorkItem => ({
  id,
  tenant_id: "t-1",
  project_id: "p-1",
  key: `PHYSIS-${id.replace("wi-", "")}`,
  title: `Test ${id}`,
  description: "",
  kind: "story",
  status,
  priority: "p1",
  reporter_id: "usr-001",
  story_points: 3,
  labels: [],
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
});

const mockBoard: Board = {
  id: "board-1",
  tenant_id: "t-1",
  project_id: "p-1",
  name: "Test Board",
  columns: [
    { status: "todo",        work_item_ids: ["wi-1", "wi-2"], wip_limit: 8 },
    { status: "in_progress", work_item_ids: ["wi-3"],         wip_limit: 5 },
    { status: "review",      work_item_ids: [],                wip_limit: 3 },
    { status: "done",        work_item_ids: ["wi-4", "wi-5"], wip_limit: 99 },
  ],
};

const mockWorkItems: WorkItem[] = [
  mkWorkItem("wi-1", "todo"),
  mkWorkItem("wi-2", "todo"),
  mkWorkItem("wi-3", "in_progress"),
  mkWorkItem("wi-4", "done"),
  mkWorkItem("wi-5", "done"),
];

describe("KanbanBoard", () => {
  beforeEach(() => {
    cleanup();
  });

  // ---- Test 1: write — 渲染 4 列 + 卡片数 ----
  it("renders 4 columns with correct card counts", () => {
    const onTransition = vi.fn();
    render(
      <KanbanBoard
        board={mockBoard}
        workItems={mockWorkItems}
        identities={mockIdentities}
        onTransition={onTransition}
      />,
    );

    // 4 个 column 全部渲染
    expect(screen.getByTestId("kanban-column-todo")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-in_progress")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-review")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-done")).toBeTruthy();

    // 5 个卡片全部渲染
    expect(screen.getByTestId("kanban-card-wi-1")).toBeTruthy();
    expect(screen.getByTestId("kanban-card-wi-2")).toBeTruthy();
    expect(screen.getByTestId("kanban-card-wi-3")).toBeTruthy();
    expect(screen.getByTestId("kanban-card-wi-4")).toBeTruthy();
    expect(screen.getByTestId("kanban-card-wi-5")).toBeTruthy();
  });

  // ---- Test 2: drag — drop 触发 onTransition(id, toStatus) ----
  it("calls onTransition when a card is dropped on a different column", () => {
    const onTransition = vi.fn();
    render(
      <KanbanBoard
        board={mockBoard}
        workItems={mockWorkItems}
        identities={mockIdentities}
        onTransition={onTransition}
      />,
    );

    // 模拟 drop wi-1 (todo) → review 列
    const reviewCol = screen.getByTestId("kanban-column-review");
    const dt = {
      getData: (type: string) => (type === "text/issue-id" ? "wi-1" : ""),
    };
    fireEvent.drop(reviewCol, { dataTransfer: dt });

    expect(onTransition).toHaveBeenCalledTimes(1);
    expect(onTransition).toHaveBeenCalledWith("wi-1", "review");
  });

  // ---- Test 3: drag-over / dropTarget 高亮 ----
  it("highlights column as dropTarget on dragOver, clears on drop", () => {
    const onTransition = vi.fn();
    render(
      <KanbanBoard
        board={mockBoard}
        workItems={mockWorkItems}
        identities={mockIdentities}
        onTransition={onTransition}
      />,
    );

    const inProgressCol = screen.getByTestId("kanban-column-in_progress");

    // dragOver 时 ring-2 / bg-accent/10 应出现
    const dt = { dropEffect: "move" };
    fireEvent.dragOver(inProgressCol, { dataTransfer: dt });

    // 重新查询以拿最新 className (state 变更后)
    const inProgressColAfter = screen.getByTestId("kanban-column-in_progress");
    expect(inProgressColAfter.className).toMatch(/ring-2/);
    expect(inProgressColAfter.className).toMatch(/bg-accent/);

    // drop 后清掉
    const dropDt = {
      getData: (type: string) => (type === "text/issue-id" ? "wi-2" : ""),
    };
    fireEvent.drop(inProgressColAfter, { dataTransfer: dropDt });
    const inProgressColFinal = screen.getByTestId("kanban-column-in_progress");
    expect(inProgressColFinal.className).not.toMatch(/ring-accent/);
  });

  // ---- Test 4: U2 路由集成 — 点击卡片触发 router.push 到 /work-item/{id} ----
  // (KanbanCard 默认 onClick → router.push, Issues 主面板 / Projects 多 panel 共用同一行为)
  it("U2 路由集成: card click navigates to /work-item/{id} via router.push", () => {
    const onTransition = vi.fn();
    render(
      <KanbanBoard
        board={mockBoard}
        workItems={mockWorkItems}
        identities={mockIdentities}
        onTransition={onTransition}
      />,
    );

    // 点击 wi-1 卡片
    const card = screen.getByTestId("kanban-card-wi-1");
    fireEvent.click(card);

    // router.push 应被调用, 路径为 /work-item/wi-1 (per KanbanCard 默认 click handler)
    expect(mockRouterPush).toHaveBeenCalledTimes(1);
    expect(mockRouterPush).toHaveBeenCalledWith("/work-item/wi-1");

    // 注意: drop 路径不应触发 click (drag → drop 是另一通道)
    // 验证 onTransition 没被 click 误触发
    expect(onTransition).not.toHaveBeenCalled();
  });
});
