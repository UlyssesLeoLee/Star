// =====================================================================
// Production API mock — W5 给多人协同 hook 用的伪后端
// =====================================================================
// 真实数据源待 Phase I+ 接 star-mcp stdio / Streamable HTTP 通道。
// 本文件仅提供 mock,所有 method 直接返当前 store 状态 + cursor。
// cursor = 当前 ISO 时间,since cursor 过滤; 暂不过滤 (mock 全量回包)
// =====================================================================
import { useStore } from "./store";
import type { Board, WorkItem } from "@/types/ids";

export interface BoardSyncResponse {
  cursor: string;
  snapshot: {
    board: Board;
    recentActivity: Array<{
      work_item_id: string;
      to_status: string;
      actor_id: string;
      at: string;
    }>;
  };
}

export interface WorkItemSyncResponse {
  cursor: string;
  items: WorkItem[];
}

// ---- cursor helper ----
const nowCursor = (): string => new Date().toISOString();

/**
 * 拉取 projectId 下的 board 当前状态 + 增量 activity。
 * since cursor 在 mock 中忽略 (返全量)。
 * Phase I+ 接 star-mcp 时,since 用作增量游标。
 */
export function boardSync(
  projectId: string,
  _since?: string
): BoardSyncResponse {
  const s = useStore.getState();
  // 简单过滤:仅保留 project 一致的 board (mock 只有 1 个 board)
  const board = s.board.project_id === projectId || true
    ? s.board
    : { ...s.board, columns: [] };

  // 模拟 "最近活动" — 取最近 5 条状态非 done 的 work-item 作为 activity hint
  const recent = s.workItems
    .filter((w) => w.project_id === projectId)
    .slice(0, 5)
    .map((w) => ({
      work_item_id: w.id,
      to_status: w.status,
      actor_id: w.assignee_id ?? "usr-001",
      at: w.updated_at,
    }));

  return {
    cursor: nowCursor(),
    snapshot: { board, recentActivity: recent },
  };
}

/**
 * 拉取 projectId 下的 work-item 列表 + cursor。
 */
export function workItemSync(
  projectId: string,
  _since?: string
): WorkItemSyncResponse {
  const s = useStore.getState();
  const items = s.workItems.filter((w) => w.project_id === projectId);
  return { cursor: nowCursor(), items };
}

/**
 * 单 work-item 状态机迁移 (per §2.3 / §7.2)
 * Mock 写 store + 返新状态;真实环境会调 star-mcp transitionWorkItem。
 */
export function transitionWorkItemApi(
  workItemId: string,
  toStatus: string
): { ok: boolean; work_item_id: string; status: string } {
  const s = useStore.getState();
  s.transitionWorkItem(workItemId, toStatus as any);
  return { ok: true, work_item_id: workItemId, status: toStatus };
}

// 统一 export (alias 给 hook / 组件用)
export const productionApi = {
  boardSync,
  workItemSync,
  transitionWorkItem: transitionWorkItemApi,
};
