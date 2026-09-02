// =====================================================================
// board-refactor-constants.ts — Refactor 看板常量 + 工具 (per 2026-09-02 10:41 JST 拍板)
// =====================================================================
// 跟 components/board/constants.ts (Kanban fallback) 形态 1:1
// 沿用 Kanban 的兜底列保护语义, 防止用户误删导致数据丢失
//
// 默认 5 列 (per 2026-09-02 10:41 JST 拍板: doing 和 review 中间加 testing):
//   todo → doing → testing → review → done
// =====================================================================

import {
  REFACTOR_FALLBACK_STATUS,
  type RefactorColumn,
  type RefactorStatus,
} from "@/types/ids";

/** 默认 5 列 (todo + doing + testing + review + done) */
export const REFACTOR_DEFAULT_COLUMNS: readonly RefactorColumn[] = [
  { status: "todo",    position: 0 },
  { status: "doing",   position: 1 },
  { status: "testing", position: 2 }, // 新增 per 2026-09-02 10:41 JST
  { status: "review",  position: 3 },
  { status: "done",    position: 4 },
] as const;

/** 默认 batch size (UI pull next batch 取数) */
export const REFACTOR_DEFAULT_BATCH_SIZE = 5;

/** 兜底 status 判定 (跟 Kanban `isFallbackStatus` 1:1 风格) */
export function isRefactorFallbackStatus(status: RefactorStatus): boolean {
  return status === REFACTOR_FALLBACK_STATUS;
}

/** 给定 status 找 column (for UI render) */
export function findRefactorColumn(
  columns: RefactorColumn[],
  status: RefactorStatus,
): RefactorColumn | undefined {
  return columns.find((c) => c.status === status);
}

/** 按 position 排序 (store 写入时 enforce, 防止 position 漂移) */
export function sortRefactorColumns(
  columns: RefactorColumn[],
): RefactorColumn[] {
  return [...columns].sort((a, b) => a.position - b.position);
}

/** 重置为默认 5 列 (UI 按钮"重置为默认"调用) */
export function makeDefaultRefactorColumns(): RefactorColumn[] {
  return REFACTOR_DEFAULT_COLUMNS.map((c) => ({ ...c }));
}
