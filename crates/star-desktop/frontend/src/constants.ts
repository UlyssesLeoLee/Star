// crates/star-desktop/frontend/src/constants.ts
// =====================================================================
// Star Desktop — Tauri P5 集中常量 (PR-247 follow-up of PR #246)
// 复用 frontend/src/components/board/constants.ts (94 LOC)
// 适配: 不依赖 @/ alias + i18n (独立 Tauri 端常量)
// =====================================================================

/*
@cypher schema=1 source_sha256=ec7768ba2ee9064fc53c51c8e1e841fbe8226ba109ca24d32c5414852f27d5d0
MERGE (self:File {path:"crates/star-desktop/frontend/src/constants.ts"})
MERGE (ids:File {path:"crates/star-desktop/frontend/src/types/ids.ts"})
MERGE (self)-[:IMPORTS]->(ids)
@endcypher
*/

import {
  DEFAULT_KANBAN_COLUMNS,
  TODO_FALLBACK_STATUS,
  isFallbackStatus,
  type WorkItemStatus,
} from "./types/ids";

// Re-export shared board defaults and fallback status helpers.
export { DEFAULT_KANBAN_COLUMNS, TODO_FALLBACK_STATUS, isFallbackStatus };

/**
 * 4 种 sort modes (per docs §3.4 + §7)
 */
export type SortMode = "manual" | "priority" | "story_points" | "created_at";

/**
 * 3 种 filter modes
 */
export type FilterMode = "all" | "mine" | "unassigned" | "blocked";

/**
 * Drag drop visual states
 */
export type DragVisualState = "idle" | "dragging" | "drop_target";

/**
 * Default sort mode
 */
export const DEFAULT_SORT_MODE: SortMode = "manual";

/**
 * Default filter mode
 */
export const DEFAULT_FILTER_MODE: FilterMode = "all";

/**
 * Drag opacity when card is being dragged
 */
export const DRAG_OPACITY = 0.5;

/**
 * Drop zone highlight border color (per frontend globals.css)
 */
export const DROP_ZONE_BORDER = "#1976d2";

/**
 * Drop zone background tint (10% accent opacity)
 */
export const DROP_ZONE_BG = "rgba(25, 118, 210, 0.10)";

/**
 * Column count (6 by default)
 */
export const KANBAN_COLUMN_COUNT = DEFAULT_KANBAN_COLUMNS.length;

/**
 * W/T/M swimlane order
 */
export const SWIMLANE_ORDER = ["W", "T", "M"] as const;

/**
 * 6 WorkItemStatus values (per docs §8.7)
 */
export const ALL_WORK_ITEM_STATUSES: WorkItemStatus[] = [
  "todo",
  "in_progress",
  "review",
  "blocked",
  "done",
  "wontfix",
];

/**
 * Helper: get column by status
 */
export const getColumnByStatus = (status: WorkItemStatus) =>
  DEFAULT_KANBAN_COLUMNS.find((c) => c.id === status);

/**
 * Helper: get next column (for transition arrows)
 */
export const getNextColumn = (status: WorkItemStatus): WorkItemStatus | null => {
  const idx = DEFAULT_KANBAN_COLUMNS.findIndex((c) => c.id === status);
  if (idx === -1 || idx === DEFAULT_KANBAN_COLUMNS.length - 1) return null;
  return DEFAULT_KANBAN_COLUMNS[idx + 1].id;
};
