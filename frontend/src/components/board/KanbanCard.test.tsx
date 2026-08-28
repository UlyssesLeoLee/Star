// =====================================================================
// KanbanCard.test.tsx — 卡片测试 (per §11.3 测试基线)
// =====================================================================
// 2 个测试:
//   1. dragstart: dataTransfer.setData("text/issue-id", id) 正确
//   2. dragging state: opacity-50 + ring-2
//
// 已知缺口 (per 缺标比错标):
//   - 暂未装 vitest / @testing-library/react (per W1 守门)
//   - 文件按 vitest 语法编写, runner 安装后直接跑
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest"; // eslint-disable-line @typescript-eslint/no-unused-vars
import { render, screen, fireEvent, cleanup } from "@testing-library/react"; // eslint-disable-line @typescript-eslint/no-unused-vars
import { KanbanCard } from "./KanbanCard";
import type { WorkItem } from "@/types/ids";

const mockWorkItem: WorkItem = {
  id: "wi-007",
  tenant_id: "t-1",
  project_id: "p-1",
  key: "PHYSIS-7",
  title: "Test card drag",
  description: "",
  kind: "story",
  status: "in_progress",
  priority: "p0",
  reporter_id: "usr-001",
  assignee_id: "usr-002",
  story_points: 5,
  labels: ["backend", "perf"],
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

describe("KanbanCard", () => {
  beforeEach(() => {
    cleanup();
  });

  // ---- Test 1: dragstart setData 正确 ----
  it("calls dataTransfer.setData('text/issue-id', id) on dragstart", () => {
    const onDragStart = vi.fn();
    const card = render(
      <KanbanCard
        workItem={mockWorkItem}
        onDragStart={onDragStart}
      />,
    );

    // 卡片可拖动
    const cardEl = screen.getByTestId("kanban-card-wi-007");
    expect(cardEl.getAttribute("draggable")).toBe("true");
    expect(cardEl.getAttribute("data-issue-id")).toBe("wi-007");

    // 模拟 dragstart — 用 stub 跟踪 setData
    const setDataMock = vi.fn();
    const dataTransfer = { setData: setDataMock, effectAllowed: "" };
    fireEvent.dragStart(cardEl, { dataTransfer });

    // 必须 setData("text/issue-id", "wi-007")
    expect(setDataMock).toHaveBeenCalledWith("text/issue-id", "wi-007");
    // effectAllowed 设为 "move"
    expect(dataTransfer.effectAllowed).toBe("move");
    // 父组件 onDragStart 钩子也被调
    expect(onDragStart).toHaveBeenCalled();
  });

  // ---- Test 2: dragging state — opacity-50 + ring-2 ----
  it("applies opacity-50 + ring-2 when isDragging=true", () => {
    const { rerender } = render(
      <KanbanCard workItem={mockWorkItem} isDragging={false} />,
    );

    const cardBefore = screen.getByTestId("kanban-card-wi-007");
    // 默认不透明
    expect(cardBefore.className).not.toMatch(/opacity-50/);
    expect(cardBefore.className).not.toMatch(/ring-2/);

    // 切到 dragging
    rerender(<KanbanCard workItem={mockWorkItem} isDragging={true} />);
    const cardAfter = screen.getByTestId("kanban-card-wi-007");
    expect(cardAfter.className).toMatch(/opacity-50/);
    expect(cardAfter.className).toMatch(/ring-2/);
  });
});
