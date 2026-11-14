/*
@cypher schema=1 source_sha256=7cc9db1fb78dc538dfc6de32e0986aeedb0bc3f492b25b23b4ee75e42f282ec9
MERGE (self:File {path:"frontend/src/components/gantt/GanttView.tsx"})
MERGE (v:Symbol {id:"frontend/src/components/gantt/GanttView.tsx::GanttView",kind:"component"})
MERGE (self)-[:DEFINES]->(v)
@endcypher
*/
"use client";

// =====================================================================
// GanttView — kanban 页 时间线 tab 挂载的甘特图视图
// per WBS-BACKEND-SUPPORT-001 W5: 替换旧 /projects?tab=timeline 跳转
// 将 Board(列) + WorkItem[] → GanttChart 所需 sprints/milestones/dateRange
// =====================================================================

import { useMemo } from "react";
import { GanttChart } from "./GanttChart";
import type { Board, WorkItem, Sprint, Milestone, Relation, Identity, SprintStatus } from "@/types/ids";

interface GanttViewProps {
  board: Board;
  workItems: WorkItem[];
  identities: Identity[];
}

const SPRINT_START = "2026-11-01";

export function GanttView({ board, workItems, identities }: GanttViewProps) {
  void identities;

  // board.columns → sprints (生命周期列映射为 sprint 行)
  const sprints = useMemo((): Sprint[] => {
    const statusToSprintStatus: Record<string, SprintStatus> = {
      todo: "planned", in_progress: "active", review: "active",
      blocked: "active", done: "completed", wontfix: "cancelled",
    };
    const today = Date.now();
    const day = 86400000;
    return board.columns.map((col, idx) => {
      const start = new Date(today - day * (10 - idx)).toISOString().slice(0, 10);
      const end = new Date(today + day * (idx + 3)).toISOString().slice(0, 10);
      return {
        id: `sp-${col.status}`,
        tenant_id: board.tenant_id,
        project_id: board.project_id,
        key: `SP-${idx + 1}`,
        name: col.name ?? col.status,
        goal: `${col.status} 列 (${col.work_item_ids.length} 任务)`,
        status: statusToSprintStatus[col.status] ?? "active",
        start_date: start,
        end_date: end,
        capacity_points: 20,
        committed_points: col.work_item_ids.length * 3,
        completed_points: col.status === "done" ? col.work_item_ids.length * 3 : 0,
      };
    });
  }, [board]);

  // milestones: p0/p1 任务卡 → 里程碑行
  const milestones = useMemo((): Milestone[] => {
    return workItems
      .filter((w) => w.priority === "p0" || w.priority === "p1")
      .slice(0, 6)
      .map((w) => ({
        id: `ms-${w.id}`,
        tenant_id: w.tenant_id,
        project_id: w.project_id,
        key: `MS-${w.key}`,
        name: w.title,
        due_date: w.due_date ?? new Date(Date.now() + 7 * 86400000).toISOString(),
        work_item_ids: [w.id],
        progress: w.status === "done" ? 1 : w.status === "in_progress" ? 0.5 : 0,
      }));
  }, [workItems]);

  const relations = useMemo((): Relation[] => [], []);

  const dateRange = useMemo(() => {
    const all = [
      ...sprints.map((s) => s.start_date),
      ...sprints.map((s) => s.end_date),
      ...milestones.map((m) => m.due_date),
      ...workItems.map((w) => w.due_date).filter((d): d is string => !!d),
    ].sort();
    if (all.length === 0) return { start: SPRINT_START, end: "2026-12-15" };
    return { start: all[0].slice(0, 10), end: all[all.length - 1].slice(0, 10) };
  }, [sprints, milestones, workItems]);

  return (
    <div data-testid="kanban-gantt-chart" className="h-full min-h-[420px] overflow-auto">
      <GanttChart
        sprints={sprints}
        milestones={milestones}
        workItems={workItems}
        relations={relations}
        dateRange={dateRange}
      />
    </div>
  );
}
