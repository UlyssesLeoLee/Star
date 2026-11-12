/*
@cypher schema=1 source_sha256=5f3a1c9e7b2d4f6a8c0e1d3b5f7a9c2e4d6f8a0b1c3d5e7f9a0b2c4d6e8f0a
MERGE (self:File {path:"frontend/src/app/(app)/projects/ProjectsFilterBar.tsx"})
MERGE (filterBar:Symbol {id:"frontend/src/app/(app)/projects/ProjectsFilterBar.tsx::ProjectsFilterBar",kind:"component"})
MERGE (useTranslation:Symbol {id:"frontend/src/lib/i18n/index.ts::useTranslation",kind:"hook"})
MERGE (self)-[:DEFINES]->(filterBar)
MERGE (self)-[:IMPORTS]->(useTranslation)
MERGE (filterBar)-[:CALLS]->(useTranslation)
@endcypher
*/
"use client";

import { useMemo } from "react";
import { Search, X } from "lucide-react";
import { clsx } from "clsx";
import type { WorkItemStatus } from "@/types/ids";
import { useTranslation } from "@/lib/i18n";

// per WBS §23.32-B (2026-11-13 用户发令) + ui-redesign-multica-style.md §5.2:
// /projects 多 panel 上下两块标签页区域 — 上行 [Tabs] 切当前标签页(+子标签页),
// 下行 [Filter Bar] 筛选标签页里的内容 (sticky 56px, per ui-3pane-arch.md §1.3)。

export const KANBAN_STATUSES: WorkItemStatus[] = ["todo", "in_progress", "review", "blocked", "done"];
export const BACKLOG_PRIORITIES = ["critical", "high", "medium", "low"] as const;

export interface ProjectsFilterState {
  statuses: WorkItemStatus[];
  priorities: string[];
  memberIds: string[];
  query: string;
}

export const EMPTY_PROJECTS_FILTER: ProjectsFilterState = {
  statuses: [],
  priorities: [],
  memberIds: [],
  query: "",
};

/** 状态 chip 默认全选语义: 空数组 = 不筛选 (显示全部) */
export function isFilterActive(filter: ProjectsFilterState): boolean {
  return (
    filter.statuses.length > 0 ||
    filter.priorities.length > 0 ||
    filter.memberIds.length > 0 ||
    filter.query.trim().length > 0
  );
}

export function applyKanbanFilter<T extends { id: string; status: WorkItemStatus; assignee_id?: string | null; title: string; key?: string }>(
  workItems: T[],
  identities: Array<{ id: string; display_name: string }>,
  filter: ProjectsFilterState,
): T[] {
  const q = filter.query.trim().toLowerCase();
  return workItems.filter((w) => {
    if (filter.statuses.length > 0 && !filter.statuses.includes(w.status)) return false;
    if (filter.memberIds.length > 0 && !filter.memberIds.includes(w.assignee_id ?? "")) return false;
    if (q) {
      const assignee = identities.find((i) => i.id === w.assignee_id)?.display_name ?? "";
      const hay = `${w.title} ${w.key ?? ""} ${w.id} ${assignee}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });
}

export function applyBacklogFilter<T extends { id: string; status: WorkItemStatus; priority?: string | null; assignee_id?: string | null; title: string; key?: string }>(
  workItems: T[],
  identities: Array<{ id: string; display_name: string }>,
  filter: ProjectsFilterState,
): T[] {
  const q = filter.query.trim().toLowerCase();
  return workItems.filter((w) => {
    if (filter.statuses.length > 0 && !filter.statuses.includes(w.status)) return false;
    if (filter.priorities.length > 0 && !filter.priorities.includes(w.priority ?? "")) return false;
    if (q) {
      const assignee = identities.find((i) => i.id === w.assignee_id)?.display_name ?? "";
      const hay = `${w.title} ${w.key ?? ""} ${w.id} ${assignee}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });
}

export function applyWorktreeFilter<T extends { id: string; name: string; branch?: string | null; status?: string | null }>(
  worktrees: T[],
  filter: ProjectsFilterState,
): T[] {
  const q = filter.query.trim().toLowerCase();
  return worktrees.filter((w) => {
    if (filter.statuses.length > 0 && !filter.statuses.includes(w.status as WorkItemStatus)) return false;
    if (q) {
      const hay = `${w.name} ${w.branch ?? ""} ${w.id}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });
}

interface ProjectsFilterBarProps {
  filter: ProjectsFilterState;
  onChange: (next: ProjectsFilterState) => void;
  /** 当前 tab 支持的筛选能力 */
  tab: "kanban" | "timeline" | "backlog" | "agents" | "worktrees" | "ipa";
  /** 成员 chip 数据 (kanban 成员筛选) */
  memberChips?: Array<{ id: string; label: string }>;
  /** timeline 子标签页: month/week (隶属于当前标签页的子标签页) */
  calendarView?: "month" | "week";
  onCalendarViewChange?: (view: "month" | "week") => void;
}

export function ProjectsFilterBar({
  filter,
  onChange,
  tab,
  memberChips = [],
  calendarView,
  onCalendarViewChange,
}: ProjectsFilterBarProps) {
  const { t } = useTranslation();

  const chips = useMemo(() => {
    switch (tab) {
      case "kanban":
        return {
          statusChips: KANBAN_STATUSES,
          priorityChips: [] as string[],
          showMember: true,
          showSearch: true,
        };
      case "backlog":
        return {
          statusChips: KANBAN_STATUSES,
          priorityChips: [...BACKLOG_PRIORITIES] as string[],
          showMember: false,
          showSearch: true,
        };
      case "worktrees":
        return {
          statusChips: [] as WorkItemStatus[],
          priorityChips: [] as string[],
          showMember: false,
          showSearch: true,
        };
      default:
        return {
          statusChips: [] as WorkItemStatus[],
          priorityChips: [] as string[],
          showMember: false,
          showSearch: true,
        };
    }
  }, [tab]);

  const toggleStatus = (s: WorkItemStatus) => {
    const next = filter.statuses.includes(s)
      ? filter.statuses.filter((x) => x !== s)
      : [...filter.statuses, s];
    onChange({ ...filter, statuses: next });
  };
  const togglePriority = (p: string) => {
    const next = filter.priorities.includes(p)
      ? filter.priorities.filter((x) => x !== p)
      : [...filter.priorities, p];
    onChange({ ...filter, priorities: next });
  };
  const toggleMember = (id: string) => {
    const next = filter.memberIds.includes(id)
      ? filter.memberIds.filter((x) => x !== id)
      : [...filter.memberIds, id];
    onChange({ ...filter, memberIds: next });
  };
  const clearAll = () => onChange({ ...EMPTY_PROJECTS_FILTER });

  // timeline tab 时上行 Tabs 之下再挂一行子标签页 (month/week) — "隶属于当前标签页的子标签页"
  const showCalendarSubTabs = tab === "timeline" && calendarView && onCalendarViewChange;

  return (
    <div
      data-testid="projects-filter-bar"
      className="sticky top-[var(--page-header-h,56px)] z-30 flex flex-wrap items-center gap-2 border-b border-line bg-[var(--color-surface,var(--bg-soft))] px-4 py-2"
      style={{ minHeight: 56 }}
    >
      {showCalendarSubTabs && (
        <div data-testid="projects-calendar-subtabs" className="flex items-center gap-1" role="tablist" aria-label={t.projectsFilterBar.calendarSubTabs}>
          {(["month", "week"] as const).map((v) => (
            <button
              key={v}
              type="button"
              role="tab"
              aria-selected={calendarView === v}
              data-testid={`calendar-subtab-${v}`}
              onClick={() => onCalendarViewChange?.(v)}
              className={clsx(
                "px-2.5 py-1 text-[11px] font-mono rounded border transition-colors",
                calendarView === v
                  ? "bg-accent text-white border-accent font-bold"
                  : "border-line text-ink-mute hover:text-ink hover:border-accent/50",
              )}
            >
              {v === "month" ? t.projectsFilterBar.monthView : t.projectsFilterBar.weekView}
            </button>
          ))}
        </div>
      )}

      {chips.statusChips.length > 0 && (
        <div data-testid="filter-status-chips" className="flex flex-wrap items-center gap-1" role="group" aria-label={t.projectsFilterBar.statusGroup}>
          <span className="text-[10px] font-mono text-ink-mute uppercase tracking-wider">{t.projectsFilterBar.statusLabel}</span>
          {chips.statusChips.map((s) => (
            <button
              key={s}
              type="button"
              data-testid={`filter-status-${s}`}
              aria-pressed={filter.statuses.includes(s)}
              onClick={() => toggleStatus(s)}
              className={clsx(
                "px-2 py-0.5 text-[10px] font-mono rounded-full border transition-colors",
                filter.statuses.includes(s)
                  ? "bg-accent/15 text-accent border-accent font-bold"
                  : "border-line text-ink-mute hover:text-ink hover:border-accent/50",
              )}
            >
              {s}
            </button>
          ))}
        </div>
      )}

      {chips.priorityChips.length > 0 && (
        <div data-testid="filter-priority-chips" className="flex flex-wrap items-center gap-1" role="group" aria-label={t.projectsFilterBar.priorityGroup}>
          <span className="text-[10px] font-mono text-ink-mute uppercase tracking-wider">{t.projectsFilterBar.priorityLabel}</span>
          {chips.priorityChips.map((p) => (
            <button
              key={p}
              type="button"
              data-testid={`filter-priority-${p}`}
              aria-pressed={filter.priorities.includes(p)}
              onClick={() => togglePriority(p)}
              className={clsx(
                "px-2 py-0.5 text-[10px] font-mono rounded-full border transition-colors",
                filter.priorities.includes(p)
                  ? "bg-accent/15 text-accent border-accent font-bold"
                  : "border-line text-ink-mute hover:text-ink hover:border-accent/50",
              )}
            >
              {p}
            </button>
          ))}
        </div>
      )}

      {chips.showMember && memberChips.length > 0 && (
        <div data-testid="filter-member-chips" className="flex flex-wrap items-center gap-1" role="group" aria-label={t.projectsFilterBar.memberGroup}>
          <span className="text-[10px] font-mono text-ink-mute uppercase tracking-wider">{t.projectsFilterBar.memberLabel}</span>
          {memberChips.map((m) => (
            <button
              key={m.id}
              type="button"
              data-testid={`filter-member-${m.id}`}
              aria-pressed={filter.memberIds.includes(m.id)}
              onClick={() => toggleMember(m.id)}
              className={clsx(
                "px-2 py-0.5 text-[10px] font-mono rounded-full border transition-colors",
                filter.memberIds.includes(m.id)
                  ? "bg-accent/15 text-accent border-accent font-bold"
                  : "border-line text-ink-mute hover:text-ink hover:border-accent/50",
              )}
            >
              {m.label}
            </button>
          ))}
        </div>
      )}

      {chips.showSearch && (
        <div className="relative ml-auto flex items-center gap-1">
          <Search size={13} className="text-ink-mute" aria-hidden="true" />
          <input
            type="search"
            value={filter.query}
            onChange={(e) => onChange({ ...filter, query: e.target.value })}
            placeholder={t.projectsFilterBar.searchPlaceholder}
            data-testid="filter-search-input"
            className="w-40 md:w-56 rounded border border-line bg-transparent px-2 py-1 text-xs placeholder:text-ink-mute/70 focus:outline-none focus:border-accent"
            aria-label={t.projectsFilterBar.searchPlaceholder}
          />
        </div>
      )}

      {isFilterActive(filter) && (
        <button
          type="button"
          data-testid="filter-clear-all"
          onClick={clearAll}
          className="flex items-center gap-1 px-2 py-0.5 text-[10px] font-mono rounded border border-line text-ink-mute hover:text-accent hover:border-accent transition-colors"
        >
          <X size={11} aria-hidden="true" />
          {t.projectsFilterBar.clearAll}
        </button>
      )}
    </div>
  );
}
