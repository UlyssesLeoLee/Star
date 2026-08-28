"use client";

// =====================================================================
// KanbanFilters — Kanban 顶部 filter (kind + assignee + priority)
// =====================================================================
// 状态由父组件 (page.tsx) 持有, 此组件为受控组件
// 选中 / 取消即调 onChange, 父组件 setState 后重渲染 KanbanBoard
//
// 已知缺口 (per 缺标比错标):
//   - 不接 WIP-limit / 关键字搜索 (Phase D.6+ 补, 与 W5 store persist 一起)
//   - filter 状态不持久化 (reload 后丢失, W5 加 zustand/persist 后再补)
// =====================================================================

import type { WorkItemKind, WorkItemPriority, Identity } from "@/types/ids";

export interface KanbanFiltersValue {
  kind: WorkItemKind | "all";
  assignee_id: string | "all";
  priority: WorkItemPriority | "all";
}

export interface KanbanFiltersProps {
  value: KanbanFiltersValue;
  onChange: (next: KanbanFiltersValue) => void;
  assignees: Identity[]; // 提供下拉选项
  total: number;
  shown: number;
}

const KIND_OPTIONS: Array<WorkItemKind> = ["story", "task", "bug", "spike", "epic"];
const PRIORITY_OPTIONS: Array<WorkItemPriority> = ["p0", "p1", "p2", "p3"];

export function KanbanFilters({ value, onChange, assignees, total, shown }: KanbanFiltersProps) {
  const set = <K extends keyof KanbanFiltersValue>(k: K, v: KanbanFiltersValue[K]) =>
    onChange({ ...value, [k]: v });

  return (
    <div
      data-testid="kanban-filters"
      className="card mb-3 flex items-center gap-2 text-xs flex-wrap"
    >
      <span className="text-ink-dim">Filter:</span>

      <select
        aria-label="kind"
        value={value.kind}
        onChange={(e) => set("kind", e.target.value as KanbanFiltersValue["kind"])}
        className="bg-bg-soft border border-line rounded px-2 py-1 text-xs"
      >
        <option value="all">all kinds</option>
        {KIND_OPTIONS.map((k) => (
          <option key={k} value={k}>{k}</option>
        ))}
      </select>

      <select
        aria-label="assignee"
        value={value.assignee_id}
        onChange={(e) => set("assignee_id", e.target.value as KanbanFiltersValue["assignee_id"])}
        className="bg-bg-soft border border-line rounded px-2 py-1 text-xs"
      >
        <option value="all">all assignees</option>
        {assignees.map((u) => (
          <option key={u.id} value={u.id}>{u.display_name}</option>
        ))}
      </select>

      <select
        aria-label="priority"
        value={value.priority}
        onChange={(e) => set("priority", e.target.value as KanbanFiltersValue["priority"])}
        className="bg-bg-soft border border-line rounded px-2 py-1 text-xs"
      >
        <option value="all">all priorities</option>
        {PRIORITY_OPTIONS.map((p) => (
          <option key={p} value={p}>{p.toUpperCase()}</option>
        ))}
      </select>

      {(value.kind !== "all" || value.assignee_id !== "all" || value.priority !== "all") && (
        <button
          onClick={() => onChange({ kind: "all", assignee_id: "all", priority: "all" })}
          className="text-[10px] text-info hover:underline"
        >
          clear
        </button>
      )}

      <span className="ml-auto text-ink-mute font-mono text-[10px]">
        {shown} of {total} cards
      </span>
    </div>
  );
}
