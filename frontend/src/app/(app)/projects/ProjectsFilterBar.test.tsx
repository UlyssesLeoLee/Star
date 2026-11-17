/*
@cypher schema=1 source_sha256=14a2ccef9eaf6d08248d7b28b3d8354973ee042c48c4cfa3a0f02e174b07cea2
MERGE (self:File {path:"frontend/src/app/(app)/projects/ProjectsFilterBar.test.tsx"})
MERGE (filterBar:Symbol {id:"frontend/src/app/(app)/projects/ProjectsFilterBar.tsx::ProjectsFilterBar",kind:"component"})
MERGE (applyKanban:Symbol {id:"frontend/src/app/(app)/projects/ProjectsFilterBar.tsx::applyKanbanFilter",kind:"function"})
MERGE (applyBacklog:Symbol {id:"frontend/src/app/(app)/projects/ProjectsFilterBar.tsx::applyBacklogFilter",kind:"function"})
MERGE (applyWorktree:Symbol {id:"frontend/src/app/(app)/projects/ProjectsFilterBar.tsx::applyWorktreeFilter",kind:"function"})
MERGE (sortBacklog:Symbol {id:"frontend/src/app/(app)/projects/ProjectsFilterBar.tsx::sortBacklogItems",kind:"function"})
MERGE (suite:Symbol {id:"frontend/src/app/(app)/projects/ProjectsFilterBar.test.tsx::ProjectsFilterBar",kind:"suite"})
MERGE (self)-[:IMPORTS]->(filterBar)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(filterBar)
MERGE (suite)-[:TESTS]->(applyKanban)
MERGE (suite)-[:TESTS]->(applyBacklog)
MERGE (suite)-[:TESTS]->(applyWorktree)
MERGE (suite)-[:TESTS]->(sortBacklog)
@endcypher
*/
// per WBS §23.32-B: /projects 下行 FilterBar 筛选标签页单元测试
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { I18nProvider } from "@/lib/i18n";
import {
  ProjectsFilterBar,
  applyKanbanFilter,
  applyBacklogFilter,
  applyWorktreeFilter,
  sortBacklogItems,
  EMPTY_PROJECTS_FILTER,
  isFilterActive,
  type ProjectsFilterState,
} from "./ProjectsFilterBar";

const identities = [
  { id: "u-1", display_name: "Ada" },
  { id: "u-2", display_name: "Bob" },
];

describe("ProjectsFilterBar helpers", () => {
  it("applyKanbanFilter filters by status / member / query", () => {
    const items = [
      { id: "w1", status: "todo" as const, assignee_id: "u-1", title: "Fix login", key: "PHY-1" },
      { id: "w2", status: "done" as const, assignee_id: "u-2", title: "Ship v1", key: "PHY-2" },
      { id: "w3", status: "todo" as const, assignee_id: null, title: "Refactor core", key: "PHY-3" },
    ];
    expect(applyKanbanFilter(items, identities, { ...EMPTY_PROJECTS_FILTER, statuses: ["todo"] }).map((w) => w.id))
      .toEqual(["w1", "w3"]);
    expect(applyKanbanFilter(items, identities, { ...EMPTY_PROJECTS_FILTER, memberIds: ["u-2"] }).map((w) => w.id))
      .toEqual(["w2"]);
    expect(applyKanbanFilter(items, identities, { ...EMPTY_PROJECTS_FILTER, query: "refactor" }).map((w) => w.id))
      .toEqual(["w3"]);
    // 空 filter = 全量
    expect(applyKanbanFilter(items, identities, EMPTY_PROJECTS_FILTER)).toHaveLength(3);
  });

  it("applyBacklogFilter filters by priority", () => {
    const items = [
      { id: "w1", status: "todo" as const, priority: "critical", title: "A" },
      { id: "w2", status: "todo" as const, priority: "low", title: "B" },
      { id: "w3", status: "in_progress" as const, priority: "critical", title: "C" },
    ];
    expect(applyBacklogFilter(items, identities, { ...EMPTY_PROJECTS_FILTER, priorities: ["critical"], statuses: ["todo"] }).map((w) => w.id))
      .toEqual(["w1"]);
  });

  it("applyWorktreeFilter filters by name/branch query", () => {
    const worktrees = [
      { id: "t1", name: "checkout-a", branch: "feat/a", status: "active" },
      { id: "t2", name: "checkout-b", branch: "fix/b", status: "archived" },
    ];
    expect(applyWorktreeFilter(worktrees, { ...EMPTY_PROJECTS_FILTER, query: "fix/b" }).map((t) => t.id))
      .toEqual(["t2"]);
  });

  it("isFilterActive reports activity", () => {
    expect(isFilterActive(EMPTY_PROJECTS_FILTER)).toBe(false);
    expect(isFilterActive({ ...EMPTY_PROJECTS_FILTER, query: "x" })).toBe(true);
  });
});

describe("ProjectsFilterBar component", () => {
  afterEach(cleanup);

  function renderBar(props: Partial<Parameters<typeof ProjectsFilterBar>[0]> = {}) {
    const onChange = vi.fn();
    const filter: ProjectsFilterState = { ...EMPTY_PROJECTS_FILTER };
    render(
      <I18nProvider initialLanguage="zh-CN">
        <ProjectsFilterBar
          filter={filter}
          onChange={onChange}
          tab="kanban"
          memberChips={identities.map((i) => ({ id: i.id, label: i.display_name }))}
          {...props}
        />
      </I18nProvider>,
    );
    return { onChange };
  }

  it("renders status + member chips and search for kanban tab", () => {
    renderBar();
    expect(screen.getByTestId("projects-filter-bar")).toBeTruthy();
    expect(screen.getByTestId("filter-status-todo")).toBeTruthy();
    expect(screen.getByTestId("filter-status-done")).toBeTruthy();
    expect(screen.getByTestId("filter-member-u-1")).toBeTruthy();
    expect(screen.getByTestId("filter-search-input")).toBeTruthy();
  });

  it("toggling a status chip reports the change", () => {
    const { onChange } = renderBar();
    fireEvent.click(screen.getByTestId("filter-status-todo"));
    expect(onChange).toHaveBeenCalledTimes(1);
    const next = onChange.mock.calls[0][0] as ProjectsFilterState;
    expect(next.statuses).toEqual(["todo"]);
  });

  it("shows clear-all button only when a filter is active", () => {
    renderBar();
    expect(screen.queryByTestId("filter-clear-all")).toBeNull();

    renderBar({ filter: { ...EMPTY_PROJECTS_FILTER, query: "abc" } });
    expect(screen.getByTestId("filter-clear-all")).toBeTruthy();
  });

  it("backlog tab shows priority chips instead of member chips", () => {
    renderBar({ tab: "backlog" });
    expect(screen.getByTestId("filter-priority-critical")).toBeTruthy();
    expect(screen.queryByTestId("filter-member-u-1")).toBeNull();
  });

  it("timeline tab renders month/week calendar sub-tabs (隶属于当前标签页的子标签页)", () => {
    const onCalendarViewChange = vi.fn();
    renderBar({ tab: "timeline", calendarView: "month", onCalendarViewChange });
    expect(screen.getByTestId("projects-calendar-subtabs")).toBeTruthy();
    const month = screen.getByTestId("calendar-subtab-month");
    expect(month.getAttribute("aria-selected")).toBe("true");
    fireEvent.click(screen.getByTestId("calendar-subtab-week"));
    expect(onCalendarViewChange).toHaveBeenCalledWith("week");
  });
});

describe("sortBacklogItems (per 2026-11-17 功能补齐: backlog 优先级排序)", () => {
  it("status 主序: todo > in_progress > review > blocked > done > wontfix", () => {
    const items = [
      { id: "d", status: "done" as const, priority: "p0" },
      { id: "t", status: "todo" as const, priority: "p3" },
      { id: "w", status: "wontfix" as const, priority: "p0" },
      { id: "p", status: "in_progress" as const, priority: "p2" },
    ];
    expect(sortBacklogItems(items).map((x) => x.id)).toEqual(["t", "p", "d", "w"]);
  });

  it("同 status 内 priority p0→p3, 无 priority 垫底", () => {
    const items = [
      { id: "a", status: "todo" as const, priority: "p3" },
      { id: "b", status: "todo" as const, priority: null },
      { id: "c", status: "todo" as const, priority: "p0" },
      { id: "d", status: "todo" as const, priority: "p1" },
    ];
    expect(sortBacklogItems(items).map((x) => x.id)).toEqual(["c", "d", "a", "b"]);
  });

  it("不修改原数组 (纯函数)", () => {
    const items = [{ id: "a", status: "todo" as const, priority: "p1" }];
    const out = sortBacklogItems(items);
    expect(out).not.toBe(items);
    expect(items[0].id).toBe("a");
  });

  it("空数组安全", () => {
    expect(sortBacklogItems([])).toEqual([]);
  });
});
