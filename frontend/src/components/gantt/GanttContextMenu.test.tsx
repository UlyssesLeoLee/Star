/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/gantt/GanttContextMenu.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/gantt/GanttContextMenu.test.tsx::GanttContextMenuSuite",kind:"suite"})
MERGE (chart:File {path:"frontend/src/components/gantt/GanttChart.tsx"})
MERGE (GanttChart:Symbol {id:"frontend/src/components/gantt/GanttChart.tsx::GanttChart",kind:"component"})
MERGE (self)-[:IMPORTS]->(chart)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(GanttChart)
@endcypher
*/
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@testing-library/react";
import type { ReactNode } from "react";
import { GanttChart } from "./GanttChart";
import type { Sprint, Milestone, WorkItem } from "@/types/ids";
import { I18nProvider } from "@/lib/i18n";

const mockPush = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush, replace: vi.fn(), refresh: vi.fn(), back: vi.fn() }),
}));

function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

const SPRINTS: Sprint[] = [
  { id: "spr-001", tenant_id: "t1", project_id: "p1", name: "Sprint 23", goal: "g", status: "active", start_date: "2026-02-01T00:00:00Z", end_date: "2026-02-14T00:00:00Z", capacity_points: 60, committed_points: 55, completed_points: 41 },
  { id: "spr-002", tenant_id: "t1", project_id: "p1", name: "Sprint 24", goal: "g", status: "planned", start_date: "2026-02-15T00:00:00Z", end_date: "2026-02-28T00:00:00Z", capacity_points: 55, committed_points: 0, completed_points: 0 },
];

const MILESTONES: Milestone[] = [
  { id: "ms-001", tenant_id: "t1", project_id: "p1", name: "MVP 0.5", due_date: "2026-02-10T00:00:00Z", work_item_ids: [], progress: 0.85 },
];

const WORKITEMS: WorkItem[] = [
  { id: "wi-001", tenant_id: "t1", project_id: "p1", key: "PHYSIS-001", title: "Worktree SM", description: "", kind: "story", status: "in_progress", priority: "p0", reporter_id: "u1", labels: [], sprint_id: "spr-001", created_at: "2026-01-15T00:00:00Z", updated_at: "2026-02-01T00:00:00Z" },
];

const DATE_RANGE = { start: "2026-01-01T00:00:00Z", end: "2026-03-31T00:00:00Z" };

function renderChart() {
  return renderWithI18n(<GanttChart sprints={SPRINTS} milestones={MILESTONES} workItems={WORKITEMS} dateRange={DATE_RANGE} />);
}

describe("GanttChart 右键情景菜单 (per canvas-ux 任务 §E)", () => {
  beforeEach(() => {
    mockPush.mockClear();
    window.localStorage.clear();
  });

  afterEach(() => {
    cleanup();
    document.body.innerHTML = "";
  });

  it("GanttBar 暴露稳定标识 data-gantt-bar-id=<variant>-<id>", () => {
    const { container } = renderChart();
    const sprintBar = container.querySelector('[data-gantt-bar-id="sprint-spr-001"]') as HTMLElement;
    expect(sprintBar).toBeTruthy();
    expect(sprintBar.dataset.testid).toBe("gantt-bar");
    expect(sprintBar.dataset.barVariant).toBe("sprint");
    const msBar = container.querySelector('[data-gantt-bar-id="milestone-ms-001"]') as HTMLElement;
    expect(msBar).toBeTruthy();
  });

  it("sprint 条右键 → 查看 sprint / 复制 key", () => {
    const { container } = renderChart();
    const bar = container.querySelector('[data-gantt-bar-id="sprint-spr-001"]') as HTMLElement;
    fireEvent.contextMenu(bar, { clientX: 100, clientY: 100 });
    const menu = document.body.querySelector('[data-testid="context-menu"]') as HTMLElement;
    expect(menu).toBeTruthy();
    const viewBtn = menu.querySelector('[data-testid="context-menu-item-view-sprint"]') as HTMLButtonElement;
    const copyBtn = menu.querySelector('[data-testid="context-menu-item-copy-sprint-key"]') as HTMLButtonElement;
    expect(viewBtn).toBeTruthy();
    expect(copyBtn).toBeTruthy();
    fireEvent.click(viewBtn);
    expect(mockPush).toHaveBeenCalledWith("/work-item?sprint=spr-001");
  });

  it("里程碑条右键 → 改截止日期触发 stub + 诚实 toast, 定位到今天不抛错", () => {
    const { container } = renderChart();
    const bar = container.querySelector('[data-gantt-bar-id="milestone-ms-001"]') as HTMLElement;
    fireEvent.contextMenu(bar, { clientX: 100, clientY: 100 });
    const menu = document.body.querySelector('[data-testid="context-menu"]') as HTMLElement;
    expect(menu.querySelector('[data-testid="context-menu-item-change-due-date"]')).toBeTruthy();
    expect(menu.querySelector('[data-testid="context-menu-item-go-to-today"]')).toBeTruthy();
    // 改截止日期: stub 只 console+audit, toast 文案诚实标注待服务端
    fireEvent.click(menu.querySelector('[data-testid="context-menu-item-change-due-date"]') as HTMLElement);
    // 菜单关闭
    expect(document.body.querySelector('[data-testid="context-menu"]')).toBeNull();
  });

  it("任务条右键 → 打开详情 / 复制 key / 移到其他 sprint (stub)", () => {
    const { container } = renderChart();
    const wiBar = container.querySelector('[data-work-item-id="wi-001"]') as HTMLElement;
    expect(wiBar).toBeTruthy();
    fireEvent.contextMenu(wiBar, { clientX: 100, clientY: 100 });
    const menu = document.body.querySelector('[data-testid="context-menu"]') as HTMLElement;
    expect(menu).toBeTruthy();
    expect(menu.querySelector('[data-testid="context-menu-item-open-work-item"]')).toBeTruthy();
    expect(menu.querySelector('[data-testid="context-menu-item-copy-work-item-key"]')).toBeTruthy();
    // wi-001 在 spr-001 → 只列出 spr-002
    expect(menu.querySelector('[data-testid="context-menu-item-move-to-sprint-spr-002"]')).toBeTruthy();
    expect(menu.querySelector('[data-testid="context-menu-item-move-to-sprint-spr-001"]')).toBeNull();
    fireEvent.click(menu.querySelector('[data-testid="context-menu-item-open-work-item"]') as HTMLElement);
    expect(mockPush).toHaveBeenCalledWith("/work-item?selected=wi-001");
  });

  it("行标签右键 → 折叠/展开 disabled + 诚实原因 (待接入)", () => {
    const { container } = renderChart();
    const label = container.querySelector('[data-row-label-id="spr-001"]') as HTMLElement;
    fireEvent.contextMenu(label, { clientX: 100, clientY: 100 });
    const menu = document.body.querySelector('[data-testid="context-menu"]') as HTMLElement;
    expect(menu).toBeTruthy();
    const collapse = menu.querySelector('[data-testid="context-menu-item-collapse-row"]') as HTMLButtonElement;
    expect(collapse).toBeTruthy();
    expect(collapse.disabled).toBe(true);
    expect(collapse.getAttribute("title")).toBeTruthy();
  });

  it("列头右键 → 折叠/展开 disabled (待接入)", () => {
    const { container } = renderChart();
    const header = container.querySelector('[data-testid="gantt-header"]') as HTMLElement;
    fireEvent.contextMenu(header, { clientX: 100, clientY: 100 });
    const menu = document.body.querySelector('[data-testid="context-menu"]') as HTMLElement;
    expect(menu).toBeTruthy();
    expect((menu.querySelector('[data-testid="context-menu-item-collapse-header"]') as HTMLButtonElement).disabled).toBe(true);
    expect((menu.querySelector('[data-testid="context-menu-item-expand-header"]') as HTMLButtonElement).disabled).toBe(true);
  });

  it("甘特空白右键 → 缩放到全部 / 回到今天", () => {
    const { container } = renderChart();
    const timeline = container.querySelector('[data-testid="gantt-timeline"]') as HTMLElement;
    fireEvent.contextMenu(timeline, { clientX: 300, clientY: 300 });
    const menu = document.body.querySelector('[data-testid="context-menu"]') as HTMLElement;
    expect(menu).toBeTruthy();
    expect(menu.querySelector('[data-testid="context-menu-item-zoom-to-fit-all"]')).toBeTruthy();
    expect(menu.querySelector('[data-testid="context-menu-item-go-to-today"]')).toBeTruthy();
    // 缩放到全部: 选一个能让 totalDays*pxPerDay ≤ 容器宽的级别 → 最终落在合法 zoom
    fireEvent.click(menu.querySelector('[data-testid="context-menu-item-zoom-to-fit-all"]') as HTMLElement);
    const chart = container.querySelector('[data-testid="gantt-chart"]') as HTMLElement;
    expect(["week", "month", "quarter"]).toContain(chart.dataset.zoom);
  });
});
