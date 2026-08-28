// =====================================================================
// Gantt Action Stubs (W2)
//
// 设计 (per docs/frontend/design/dynamic-interaction-design.md §4.3):
// - transitionMilestone(id, newDueDate)  store action (W5 负责实装)
// - transitionSprint(id, newStart, newEnd)  store action (W5 负责实装)
// - transitionWorkItemSprint(id, newSprintId)  store action (W5 负责实装)
//
// W2 提供 stub: 控制台日志 + 1s 后调 /api/audit mock 写 audit log
// (per W2 任务 §3 "拖动 milestone 改 due_date → 1s 后调 /api/audit mock 写 audit log")
//
// W5 接手时,把这些 stub 替换成 zustand store 的真实 action。
// 签名保持一致, 替换不会影响调用方代码。
// =====================================================================

import type { Uuid, Iso8601 } from "@/types/ids";

type AuditMockPayload = Record<string, unknown>;

/**
 * Fire-and-forget mock audit log writer. W5 接手后会替换为真实 /api/audit POST
 * (现在 Next.js 没有 /api/audit 路由, fetch 会自然 reject, 我们 swallow 错误).
 */
function fireAuditMock(action: string, target_kind: string, target_id: Uuid, payload: AuditMockPayload) {
  if (typeof window === "undefined") return; // SSR guard
  setTimeout(() => {
    try {
      void fetch("/api/audit", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          action,
          target_kind,
          target_id,
          payload,
          actor_id: "user-current",
          tenant_id: "tenant-current",
        }),
      }).catch(() => {
        // mock 失败不抛 — 是预期 (无后端路由)
      });
    } catch {
      // 极端情况 (e.g. fetch 不存在) 也不抛
    }
  }, 1000);
}

export interface MilestoneTransitionResult {
  ok: boolean;
  applied_at: Iso8601;
  pending_audit: boolean;
}

export function transitionMilestone(
  id: Uuid,
  newDueDate: Iso8601,
): MilestoneTransitionResult {
  // stub: console + audit mock; 不动 store (W5 接手后)
  if (typeof console !== "undefined") {
    console.info(`[gantt stub] transitionMilestone ${id} -> due_date=${newDueDate}`);
  }
  fireAuditMock("milestone.update_due_date", "milestone", id, { new_due_date: newDueDate });
  return { ok: true, applied_at: new Date().toISOString(), pending_audit: true };
}

export interface SprintTransitionResult {
  ok: boolean;
  applied_at: Iso8601;
  pending_audit: boolean;
}

export function transitionSprint(
  id: Uuid,
  newStart: Iso8601,
  newEnd: Iso8601,
): SprintTransitionResult {
  if (typeof console !== "undefined") {
    console.info(`[gantt stub] transitionSprint ${id} -> ${newStart} / ${newEnd}`);
  }
  fireAuditMock("sprint.update_dates", "sprint", id, { new_start: newStart, new_end: newEnd });
  return { ok: true, applied_at: new Date().toISOString(), pending_audit: true };
}

export interface WorkItemSprintMoveResult {
  ok: boolean;
  applied_at: Iso8601;
  pending_audit: boolean;
}

export function transitionWorkItemSprint(
  workItemId: Uuid,
  newSprintId: Uuid,
): WorkItemSprintMoveResult {
  if (typeof console !== "undefined") {
    console.info(`[gantt stub] transitionWorkItemSprint ${workItemId} -> sprint=${newSprintId}`);
  }
  fireAuditMock("workitem.change_sprint", "work_item", workItemId, { new_sprint_id: newSprintId });
  return { ok: true, applied_at: new Date().toISOString(), pending_audit: true };
}
