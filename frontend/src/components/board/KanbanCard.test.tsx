// =====================================================================
// KanbanCard.test.tsx — 卡片测试 (per §11.3 测试基线)
// =====================================================================
// 4 个测试:
//   1. dragstart: dataTransfer.setData("text/issue-id", id) 正确
//   2. dragging state: opacity-50 + ring-2
//   3. arch 按钮 (per ADR-0041): onArchClick 传了才显示
//   4. arch 按钮点击触发 onArchClick + e.stopPropagation 不冒泡到 onClick
//
// 已知缺口 (per 缺标比错标):
//   - 暂未装 vitest / @testing-library/react (per W1 守门)
//   - 文件按 vitest 语法编写, runner 安装后直接跑
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest"; // eslint-disable-line @typescript-eslint/no-unused-vars
import { render, screen, fireEvent, cleanup } from "@testing-library/react"; // eslint-disable-line @typescript-eslint/no-unused-vars
import type { ReactNode } from "react";
import { KanbanCard } from "./KanbanCard";
import type { WorkItem } from "@/types/ids";
import { I18nProvider } from "@/lib/i18n";

// per 2026-08-31 i18n 补缺口 v2: KanbanCard 内 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

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
    const card = renderWithI18n(<KanbanCard
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
    const { rerender } = renderWithI18n(
      <KanbanCard workItem={mockWorkItem} isDragging={false} />
    );

    const cardBefore = screen.getByTestId("kanban-card-wi-007");
    // 默认不透明
    expect(cardBefore.className).not.toMatch(/opacity-50/);
    expect(cardBefore.className).not.toMatch(/ring-2/);

    // 切到 dragging — rerender 用同一个 I18nProvider 包
    rerender(
      <I18nProvider initialLanguage="zh-CN">
        <KanbanCard workItem={mockWorkItem} isDragging={true} />
      </I18nProvider>
    );
    const cardAfter = screen.getByTestId("kanban-card-wi-007");
    expect(cardAfter.className).toMatch(/opacity-50/);
    expect(cardAfter.className).toMatch(/ring-2/);
  });

  // ---- Test 3: arch 按钮 (per ADR-0041) ----
  //   - onArchClick 传了才显示
  //   - 不传 = 按钮不渲染
  it("renders 🕸 Arch button when onArchClick is provided, hidden otherwise", () => {
    // 1) 不传 onArchClick → 按钮不渲染
    const { rerender } = renderWithI18n(
      <KanbanCard workItem={mockWorkItem} />
    );
    expect(screen.queryByTestId("kanban-card-arch-wi-007")).toBeNull();

    // 2) 传 onArchClick → 按钮渲染
    const onArchClick = vi.fn();
    rerender(
      <I18nProvider initialLanguage="zh-CN">
        <KanbanCard workItem={mockWorkItem} onArchClick={onArchClick} />
      </I18nProvider>
    );
    const archBtn = screen.getByTestId("kanban-card-arch-wi-007");
    expect(archBtn).toBeTruthy();
    expect(archBtn.getAttribute("aria-label")).toContain("PHYSIS-7");
  });

  // ---- Test 4: arch 按钮点击触发 onArchClick + stopPropagation 不冒泡到 onClick ----
  it("arch button click → onArchClick + does NOT bubble to onClick (router.push)", () => {
    const onArchClick = vi.fn();
    const onClick = vi.fn();
    renderWithI18n(
      <KanbanCard
        workItem={mockWorkItem}
        onArchClick={onArchClick}
        onClick={onClick}
      />,
    );

    const archBtn = screen.getByTestId("kanban-card-arch-wi-007");
    fireEvent.click(archBtn);

    // onArchClick 必须被调, 传 workItem
    expect(onArchClick).toHaveBeenCalledTimes(1);
    expect(onArchClick.mock.calls[0][0].id).toBe("wi-007");

    // onClick 不应被调 (因 stopPropagation)
    expect(onClick).not.toHaveBeenCalled();
  });
});
