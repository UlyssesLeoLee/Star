// =====================================================================
// Kanban Board — 集中常量 (避免散落魔数)
// =====================================================================
// 设计原则 (per 2026-08-31 11:24 JST Ulysses 拍板):
//   - 数据源: workItems[].status 为主, board.columns[].work_item_ids 是派生视图
//   - 兜底列: "todo" — 任何列删除时, 列里 wi 状态统一改回 todo, 并挪到 todo 列
//   - 兜底保护: 删 todo 列直接被 store 拒绝, UI 上 ✕ 置灰
//
// 修改这些常量前请确认:
//   1. 跟 docs/frontend/design/dynamic-interaction-design.md §3.4 的状态枚举对齐
//   2. seed.ts workItems.status 跟 KANBAN_COLUMNS 一致
// =====================================================================

import type { WorkItemStatus } from "@/types/ids";

/**
 * 兜底列 status — 任何列删除时, 列里 wi 状态统一改回 todo, 并挪到 todo 列
 * 兜底列本身不可被删除 (store removeBoardColumn 拒绝, UI ✕ 置灰)
 */
export const TODO_FALLBACK_STATUS: WorkItemStatus = "todo";

/**
 * 兜底列不可被删 — 用于 store action 拒绝判定 + UI 灰按钮判定
 */
export const isFallbackStatus = (s: WorkItemStatus): boolean =>
  s === TODO_FALLBACK_STATUS;
