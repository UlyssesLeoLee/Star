// =====================================================================
// GanttChart integration tests (per W2 任务 §4 + design §11.3)
//
// 验收:
//   - 渲染 sprint + milestone 行
//   - 缩放切换 (week/month) 改变列宽 (pxPerDay via header data-attr + bar width)
//   - 模拟 milestone drop -> 调 onMilestoneUpdate
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent } from "@testing-library/react";
import type { ReactNode } from "react";
import { GanttChart } from "./GanttChart";
import type { Sprint, Milestone, WorkItem } from "@/types/ids";
import { I18nProvider } from "@/lib/i18n";

// per 2026-08-31 i18n 补缺口 v2: GanttChart 内 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

// next/navigation mock — 用模块级 mockPush 共享同一 spy
const mockPush = vi.fn();
const mockReplace = vi.fn();
const mockRefresh = vi.fn();
const mockBack = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push: mockPush,
    replace: mockReplace,
    refresh: mockRefresh,
    back: mockBack,
  }),
}));

const SPRINT_FIXTURES: Sprint[] = [
  {
    id: "spr-001",
    tenant_id: "t1",
    project_id: "p1",
    name: "Sprint 23",
    goal: "Worktree SM",
    status: "active",
    start_date: "2026-02-01T00:00:00Z",
    end_date: "2026-02-14T00:00:00Z",
    capacity_points: 60,
    committed_points: 55,
    completed_points: 41,
  },
  {
    id: "spr-002",
    tenant_id: "t1",
    project_id: "p1",
    name: "Sprint 24",
    goal: "AI Auto-Approve",
    status: "planned",
    start_date: "2026-02-15T00:00:00Z",
    end_date: "2026-02-28T00:00:00Z",
    capacity_points: 55,
    committed_points: 0,
    completed_points: 0,
  },
];

const MILESTONE_FIXTURES: Milestone[] = [
  {
    id: "ms-001",
    tenant_id: "t1",
    project_id: "p1",
    name: "MVP 0.5",
    due_date: "2026-02-10T00:00:00Z",
    work_item_ids: ["wi-001", "wi-002"],
    progress: 0.85, // not critical
  },
  {
    id: "ms-002",
    tenant_id: "t1",
    project_id: "p1",
    name: "MVP 0.6",
    due_date: "2026-02-25T00:00:00Z",
    work_item_ids: ["wi-007", "wi-008"],
    progress: 0.3, // < 50% -> critical
  },
];

const WORKITEM_FIXTURES: WorkItem[] = [
  {
    id: "wi-001",
    tenant_id: "t1",
    project_id: "p1",
    key: "PHYSIS-001",
    title: "Worktree SM",
    description: "",
    kind: "story",
    status: "in_progress",
    priority: "p0",
    reporter_id: "u1",
    labels: [],
    sprint_id: "spr-001",
    created_at: "2026-01-15T00:00:00Z",
    updated_at: "2026-02-01T00:00:00Z",
  },
  {
    id: "wi-002",
    tenant_id: "t1",
    project_id: "p1",
    key: "PHYSIS-002",
    title: "Agent Auto",
    description: "",
    kind: "task",
    status: "todo",
    priority: "p1",
    reporter_id: "u1",
    labels: [],
    sprint_id: "spr-001",
    created_at: "2026-01-15T00:00:00Z",
    updated_at: "2026-02-01T00:00:00Z",
  },
];

const DATE_RANGE = {
  start: "2026-01-01T00:00:00Z",
  end: "2026-03-31T00:00:00Z",
};

describe("GanttChart", () => {
  beforeEach(() => {
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
  });

  it("renders sprint rows and milestone rows", () => {
    const { container } = renderWithI18n(<GanttChart
        sprints={SPRINT_FIXTURES}
        milestones={MILESTONE_FIXTURES}
        workItems={WORKITEM_FIXTURES}
        dateRange={DATE_RANGE}
      />,
    );

    // Sprint rows: 2 sprints, 1 row each (in timeline)
    const sprintRows = container.querySelectorAll('[data-row-kind="sprint"]');
    expect(sprintRows.length).toBeGreaterThanOrEqual(2);

    // Milestone rows: 2 milestones
    const milestoneRows = container.querySelectorAll('[data-row-kind="milestone"]');
    expect(milestoneRows.length).toBeGreaterThanOrEqual(2);

    // Y-axis labels
    const sprintLabels = container.querySelectorAll('[data-row-label-kind="sprint"]');
    expect(sprintLabels.length).toBe(2);
    const milestoneLabels = container.querySelectorAll('[data-row-label-kind="milestone"]');
    expect(milestoneLabels.length).toBe(2);

    // Sprint 23 and Sprint 24 visible
    expect(sprintLabels[0].textContent).toContain("Sprint 23");
    expect(sprintLabels[1].textContent).toContain("Sprint 24");
  });

  it("marks milestones with progress < 50% as critical path", () => {
    const { container } = renderWithI18n(<GanttChart
        sprints={SPRINT_FIXTURES}
        milestones={MILESTONE_FIXTURES}
        workItems={WORKITEM_FIXTURES}
        dateRange={DATE_RANGE}
      />,
    );

    // MVP 0.5 (progress 0.85) should NOT be critical
    const ms001 = container.querySelector('[data-row-label-id="ms-001"]') as HTMLElement;
    expect(ms001.dataset.rowCritical).toBe("false");

    // MVP 0.6 (progress 0.3) SHOULD be critical
    const ms002 = container.querySelector('[data-row-label-id="ms-002"]') as HTMLElement;
    expect(ms002.dataset.rowCritical).toBe("true");
  });

  it("zoom switching changes column width (pxPerDay) and header total width", () => {
    const { container, rerender } = renderWithI18n(<GanttChart
        sprints={SPRINT_FIXTURES}
        milestones={MILESTONE_FIXTURES}
        workItems={WORKITEM_FIXTURES}
        dateRange={DATE_RANGE}
      />,
    );

    // Default: week (60 px/day)
    const headerWeek = container.querySelector('[data-testid="gantt-header"]') as HTMLElement;
    expect(headerWeek.dataset.pxPerDay).toBe("60");
    expect(headerWeek.dataset.zoom).toBe("week");

    // Click "month" button
    const monthBtn = container.querySelector('[data-zoom-button="month"]') as HTMLButtonElement;
    fireEvent.click(monthBtn);

    const headerMonth = container.querySelector('[data-testid="gantt-header"]') as HTMLElement;
    expect(headerMonth.dataset.pxPerDay).toBe("20");
    expect(headerMonth.dataset.zoom).toBe("month");

    // Click "quarter"
    const quarterBtn = container.querySelector('[data-zoom-button="quarter"]') as HTMLButtonElement;
    fireEvent.click(quarterBtn);

    const headerQuarter = container.querySelector('[data-testid="gantt-header"]') as HTMLElement;
    expect(headerQuarter.dataset.pxPerDay).toBe("8");
    expect(headerQuarter.dataset.zoom).toBe("quarter");

    // Verify the chart itself has data-zoom updated
    const chart = container.querySelector('[data-testid="gantt-chart"]') as HTMLElement;
    expect(chart.dataset.zoom).toBe("quarter");
  });

  it("dragging a milestone bar fires onMilestoneUpdate with new due_date", () => {
    const onMilestoneUpdate = vi.fn();
    const { container } = renderWithI18n(<GanttChart
        sprints={SPRINT_FIXTURES}
        milestones={MILESTONE_FIXTURES}
        workItems={WORKITEM_FIXTURES}
        dateRange={DATE_RANGE}
        onMilestoneUpdate={onMilestoneUpdate}
      />,
    );

    // Find milestone bar (ms-001) and trigger mousedown / mousemove / mouseup
    const msBar = container.querySelector('[data-bar-id="ms-001"]') as HTMLElement;
    expect(msBar).toBeTruthy();
    expect(msBar.dataset.barVariant).toBe("milestone");

    // Week zoom (60 px/day) -> drag +60px = +1 day from 2026-02-10 -> 2026-02-11
    fireEvent.mouseDown(msBar, { clientX: 100, preventDefault: () => {}, stopPropagation: () => {} });
    fireEvent.mouseMove(document, { clientX: 160 });
    fireEvent.mouseUp(document);

    expect(onMilestoneUpdate).toHaveBeenCalledTimes(1);
    const [id, newDue] = onMilestoneUpdate.mock.calls[0];
    expect(id).toBe("ms-001");
    expect(newDue).toBe("2026-02-11");
  });

  it("dragging a sprint bar fires onSprintUpdate with new start/end (date RangeMode)", () => {
    const onSprintUpdate = vi.fn();
    const { container } = renderWithI18n(<GanttChart
        sprints={SPRINT_FIXTURES}
        milestones={MILESTONE_FIXTURES}
        workItems={WORKITEM_FIXTURES}
        dateRange={DATE_RANGE}
        onSprintUpdate={onSprintUpdate}
      />,
    );

    const sprintBar = container.querySelector('[data-bar-id="spr-001"]') as HTMLElement;
    expect(sprintBar).toBeTruthy();
    expect(sprintBar.dataset.barVariant).toBe("sprint");

    // Week zoom: drag +120px = +2 days
    fireEvent.mouseDown(sprintBar, { clientX: 200, preventDefault: () => {}, stopPropagation: () => {} });
    fireEvent.mouseMove(document, { clientX: 320 });
    fireEvent.mouseUp(document);

    expect(onSprintUpdate).toHaveBeenCalledTimes(1);
    const [id, newStart, newEnd] = onSprintUpdate.mock.calls[0];
    expect(id).toBe("spr-001");
    // spr-001 original 2026-02-01 -> 2026-02-14; +2 days -> 2026-02-03 -> 2026-02-16
    expect(newStart).toBe("2026-02-03");
    expect(newEnd).toBe("2026-02-16");
  });

  it("milestone onClick triggers router push to /work-item?milestone={id}", () => {
    mockPush.mockClear();
    const { container } = renderWithI18n(<GanttChart
        sprints={SPRINT_FIXTURES}
        milestones={MILESTONE_FIXTURES}
        workItems={WORKITEM_FIXTURES}
        dateRange={DATE_RANGE}
      />,
    );
    const msLabel = container.querySelector('[data-row-label-id="ms-002"]') as HTMLElement;
    fireEvent.click(msLabel);
    expect(mockPush).toHaveBeenCalledWith("/work-item?milestone=ms-002");
  });
});
