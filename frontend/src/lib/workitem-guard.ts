// =====================================================================
// workitem-guard.ts — DesignArtifact Approval Guard (per wt-test-t2-dsg 2026-08-31)
// =====================================================================
// 上游依据:
//   - docs/test-design.md §6.3.3   REQ-DSG-001/002 (V1 Should-Have Test)
//   - docs/test-design.md §8.2     REQ-WF-003 (Workflow 状态转换 Guard)
//   - docs/requirements.md §8.3   DesignArtifact 字段
//   - docs/specs/domain-work-item-spec.md  WorkItem 6 状态
//
// 职责 (纯函数, 不依赖 React/store):
//   - checkAllArtifactsApproved(workItemId, artifacts, requireApproval)
//   - 4 种 reason:
//       "all_approved"            — 所有 artifact approved/superseded, 允许转换
//       "pending_artifacts"       — 有未批准 artifact, 失败, 指出 pending
//       "no_artifacts_attached"   — 无 artifact 且 requireApproval=true, 失败
//       "no_artifacts_required"   — 无 artifact 且 requireApproval=false, 允许
//   - 失败时 pending 字段明确指出未批准 artifact (id + title)
//
// 守门 派生 (per AGENTS.md §4.1 v1-v14 累积规):
//   - 0 副作用 (纯函数)
//   - 0 React/store 依赖 (per 任务 scope 严格限定)
//   - 0 不安全类型 (TS 严模式)
//
// 已知缺口 (per 缺标比错标, 8/26 JST 守门 #1 + #12 引用):
//   1. artifacts 数组中混 workItemId 异 id 时, 行为"过滤后判断":
//      函数不主动过滤, 由 caller 保证传入是已过滤的 artifacts
//      (per task spec "artifacts 数组含异 workItemId 时的行为" 测试)
//   2. WorkItem 状态机层 Guard 调用点 P2 — 现 pure-function, 等 store.ts
//      transitionWorkItem 接入 (per 8/31 12:07 JST Ulysses 拍板 scope 限定)
//   3. ReviewRecord 互斥 Target 字段 TBD — basic-design §27.4 跟进
// =====================================================================

import type { DesignArtifact, Uuid } from "@/types/ids";

export type GuardReason =
  | "all_approved"
  | "pending_artifacts"
  | "no_artifacts_attached"
  | "no_artifacts_required";

export interface GuardResult {
  allowed: boolean;
  reason: GuardReason;
  /** 未批准 artifact 详情; 仅 reason="pending_artifacts" 时非空 */
  pending: DesignArtifact[];
}

/**
 * WorkItem 状态转换 Guard (per REQ-DSG-002 + §8.2 REQ-WF-003):
 *  - "全部 DesignArtifact APPROVED" 才允许从 todo → in_progress
 *  - superseded 视为已批准 (历史版本, 不阻塞)
 *  - 无 DesignArtifact:
 *      requireApproval=true  → 拒绝 (no_artifacts_attached)
 *      requireApproval=false → 允许 (no_artifacts_required)
 *  - 失败时明确指出 pending artifact (id + title 都在返回数组里)
 *
 * @param workItemId    WorkItem id (仅用于 caller-side 调试标识, 函数本身不强制过滤)
 * @param artifacts     候选 artifacts 数组 (caller 应预先按 workItemId 过滤;
 *                      若传入异 workItemId 的 artifact, 也会被视为"非 approved"
 *                      并进入 pending 列表 — 这是有意行为, 见 test 7)
 * @param requireApproval 是否要求 artifact 全部 approved (default: true)
 * @returns GuardResult { allowed, reason, pending }
 */
export function checkAllArtifactsApproved(
  workItemId: Uuid,
  artifacts: DesignArtifact[],
  requireApproval: boolean = true,
): GuardResult {
  // 抑制 unused 警告 (workItemId 用于 caller-side 调试标识, 函数本身不强制过滤)
  void workItemId;

  // 1. 无 artifact 分支
  if (artifacts.length === 0) {
    if (requireApproval) {
      return {
        allowed: false,
        reason: "no_artifacts_attached",
        pending: [],
      };
    } else {
      return {
        allowed: true,
        reason: "no_artifacts_required",
        pending: [],
      };
    }
  }

  // 2. 找出非"已批准" (approved/superseded 视为已批准)
  const pending = artifacts.filter(
    (a) => a.status !== "approved" && a.status !== "superseded",
  );

  // 3. 有 pending → 拒绝, 明确指出未批准 artifact
  if (pending.length > 0) {
    return {
      allowed: false,
      reason: "pending_artifacts",
      pending,
    };
  }

  // 4. 全部 approved (或 superseded) → 允许
  return {
    allowed: true,
    reason: "all_approved",
    pending: [],
  };
}
