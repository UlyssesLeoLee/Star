// crates/star-desktop/frontend/src/types/ids.ts
// =====================================================================
// Star Desktop — Tauri P5 类型定义 (PR-247 follow-up of PR #246)
// Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P5
// 复用 frontend/src/types/ids.ts WorkItemStatus + Wtm + WorkItem
//
// 守门:
//   - 0 依赖 frontend/src/types/ids.ts (Next.js alias @/) — 本文件独立
//   - 0 依赖 react/next/react-dom — 纯 TS 类型定义
//   - W/T/M swimlane enum 与 V0.1 frontend/src/types/ids.ts 严格同步
// =====================================================================

/**
 * WorkItemStatus — 6 status (per docs/data-design/ipa-detail/tables/board_board_swimlane.md)
 */
export type WorkItemStatus =
  | "todo"
  | "in_progress"
  | "review"
  | "blocked"
  | "done"
  | "wontfix";

/**
 * Wtm — W/T/M swimlane (per docs §3.4 V字模型看板)
 */
export type Wtm = "W" | "T" | "M";

/**
 * WorkItemKind — work item 类型 (per docs §3.4)
 */
export type WorkItemKind =
  | "story"
  | "task"
  | "bug"
  | "epic"
  | "spike";

/**
 * Identity — 用户/agent 身份 (per docs §6.2)
 */
export interface Identity {
  id: string;
  display_name: string;
  email?: string;
  avatar_url?: string;
}

/**
 * WorkItem — 看板核心实体 (per docs §3.4)
 *
 * 字段与 frontend/src/types/ids.ts WorkItem 严格对齐 (per PR #216)
 */
export interface WorkItem {
  id: string;
  key: string;
  title: string;
  status: WorkItemStatus;
  w_t_m: Wtm;
  kind: WorkItemKind;
  priority: "P0" | "P1" | "P2" | "P3";
  assignee?: Identity;
  story_points?: number;
  parent_id?: string;
}

/**
 * KanbanColumn — 看板列 (per docs §3.4)
 */
export interface KanbanColumn {
  id: WorkItemStatus;
  name: string;
  order: number;
  wip_limit?: number;
  color?: string;
}

/**
 * BoardCard — 看板卡片位置 (per docs §3.4)
 */
export interface BoardCard {
  id: string;
  work_item_id: string;
  column_id: WorkItemStatus;
  swimlane_id?: Wtm;
  order_in_column: number;
}

/**
 * SwimlaneGroupBy — swimlane 分组模式 (per crates/domain-board)
 */
export type SwimlaneGroupBy = "assignee" | "priority" | "epic" | "none";

/**
 * 6 default Kanban columns (per PR #246 HomePage 8 IPC commands list_work_items 数据集)
 */
export const DEFAULT_KANBAN_COLUMNS: KanbanColumn[] = [
  { id: "todo", name: "Todo", order: 0 },
  { id: "in_progress", name: "In Progress", order: 1, wip_limit: 5 },
  { id: "review", name: "Review", order: 2, wip_limit: 3 },
  { id: "blocked", name: "Blocked", order: 3 },
  { id: "done", name: "Done", order: 4 },
  { id: "wontfix", name: "Won't Fix", order: 5 },
];

/**
 * 3 default Swimlanes (W/T/M)
 */
export const DEFAULT_SWIMLANES: Wtm[] = ["W", "T", "M"];

/**
 * 兜底列 status — 任何列删除时, 列里 wi 状态统一改回 todo
 * 兜底列本身不可被删除 (per frontend/src/components/board/constants.ts)
 */
export const TODO_FALLBACK_STATUS: WorkItemStatus = "todo";

/**
 * 兜底列不可被删 — 用于 store action 拒绝判定 + UI 灰按钮判定
 */
export const isFallbackStatus = (s: WorkItemStatus): boolean =>
  s === TODO_FALLBACK_STATUS;
