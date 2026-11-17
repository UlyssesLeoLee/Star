/*
@cypher schema=1 source_sha256=e5fbcf6df56ae4b2568352b0a593148e274ef74cff6f009a2fe8adb557e411fb
MERGE (self:File {path:"frontend/src/components/board/KanbanFilters.test.tsx"})
MERGE (mod:File {path:"frontend/src/components/board/KanbanFilters.tsx"})
MERGE (applyFn:Symbol {id:"frontend/src/components/board/KanbanFilters.tsx::applyBoardFilters",kind:"function"})
MERGE (filtersComp:Symbol {id:"frontend/src/components/board/KanbanFilters.tsx::KanbanFilters",kind:"component"})
MERGE (self)-[:IMPORTS]->(mod)
MERGE (self)-[:TESTS]->(applyFn)
MERGE (self)-[:TESTS]->(filtersComp)
@endcypher
*/
// =====================================================================
// KanbanFilters.test.tsx — applyBoardFilters 纯函数 + 组件接线 (per 2026-11-17 功能补齐)
// KanbanFilters 原为无接线孤儿组件, 接入 /kanban 任务看板后补测:
//   1. applyBoardFilters: all 不过滤 / kind / assignee / priority / 组合
//   2. 组件渲染 + 选中 kind 回调
// =====================================================================

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { applyBoardFilters, KanbanFilters, type KanbanFiltersValue } from "./KanbanFilters";
import type { WorkItem } from "@/types/ids";

const ALL: KanbanFiltersValue = { kind: "all", assignee_id: "all", priority: "all" };

function wi(over: Partial<WorkItem>): WorkItem {
  return {
    id: `id-${Math.random().toString(36).slice(2, 8)}`,
    tenant_id: "tn-001",
    project_id: "prj-physis",
    key: "T-1",
    title: "x",
    description: "",
    kind: "task",
    status: "todo",
    priority: "p2",
    reporter_id: "usr-001",
    labels: [],
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...over,
  };
}

describe("applyBoardFilters", () => {
  const items: WorkItem[] = [
    wi({ id: "w1", kind: "story", priority: "p1", assignee_id: "usr-a" }),
    wi({ id: "w2", kind: "task", priority: "p0", assignee_id: "usr-b" }),
    wi({ id: "w3", kind: "spike", priority: "p2", assignee_id: null }),
  ];

  it("all 默认不过滤", () => {
    expect(applyBoardFilters(items, ALL).map((w) => w.id)).toEqual(["w1", "w2", "w3"]);
  });

  it("按 kind 过滤", () => {
    expect(applyBoardFilters(items, { ...ALL, kind: "story" }).map((w) => w.id)).toEqual(["w1"]);
  });

  it("按 assignee 过滤 (null assignee 不命中具体 id)", () => {
    expect(
      applyBoardFilters(items, { ...ALL, assignee_id: "usr-b" }).map((w) => w.id),
    ).toEqual(["w2"]);
  });

  it("按 priority 过滤", () => {
    expect(applyBoardFilters(items, { ...ALL, priority: "p0" }).map((w) => w.id)).toEqual(["w2"]);
  });

  it("组合过滤 (kind + priority)", () => {
    expect(
      applyBoardFilters(items, { ...ALL, kind: "task", priority: "p2" }).map((w) => w.id),
    ).toEqual([]);
  });

  it("空数组安全", () => {
    expect(applyBoardFilters([], { ...ALL, kind: "epic" })).toEqual([]);
  });
});

describe("KanbanFilters 组件", () => {
  it("渲染 3 个 select + 计数, 选中 kind 触发 onChange", () => {
    const onChange = vi.fn();
    render(
      <KanbanFilters
        value={ALL}
        onChange={onChange}
        assignees={[{ id: "usr-a", display_name: "A" }]}
        total={5}
        shown={5}
      />,
    );
    expect(screen.getByTestId("kanban-filters")).toBeTruthy();
    expect(screen.getByLabelText("kind")).toBeTruthy();
    expect(screen.getByLabelText("assignee")).toBeTruthy();
    expect(screen.getByLabelText("priority")).toBeTruthy();

    fireEvent.change(screen.getByLabelText("kind"), { target: { value: "epic" } });
    expect(onChange).toHaveBeenCalledWith({ kind: "epic", assignee_id: "all", priority: "all" });
  });

  it("筛选激活时显示 clear 按钮", () => {
    render(
      <KanbanFilters
        value={{ kind: "task", assignee_id: "all", priority: "all" }}
        onChange={vi.fn()}
        assignees={[]}
        total={3}
        shown={1}
      />,
    );
    expect(screen.getByText("clear")).toBeTruthy();
  });
});
