// =====================================================================
// DesignArtifact schema + type guards (per wt-test-t2-dsg 2026-08-31)
// =====================================================================
// 上游依据:
//   - docs/test-design.md §6.3.3   REQ-DSG-001/002 (V1 Should-Have Test)
//   - docs/requirements.md §8.3   DesignArtifact 字段定义
//   - docs/requirements.md §27.4  ReviewRecord Target 字段
//                                  "ChangeSet | DesignArtifact" 二选一
//
// 设计:
//   - 5 Status type guard (draft/in_review/approved/rejected/superseded)
//   - 完整对象 type guard 含 version >= 1 + review_record_id 可空
//   - 守门 缺标比错标安全: review_record_id 保持 nullable,
//     basic-design §27.4 字段精确化后, 升级为 discriminated union
//     (ChangeSetId | DesignArtifactId) (per 8/26 JST 守门 #12 派生约束)
//
// 已知缺口 (per 缺标比错标, 8/26 JST 守门 #1 + #12 引用):
//   1. ReviewRecord 互斥 Target 字段 TBD — basic-design §27.4 跟进
//      缺标 = 当前 nullable Uuid 表达; 错标 = 提前写死 discriminated union
//   2. version 单调递增约束由 data 层保证 (creator 用 max+1),
//      schema 层仅校验 >= 1
//   3. phase F+ 后端接入后, 此 type 与 backend 真实类型对齐, 仅改 data 文件
// =====================================================================

import type { DesignArtifact, DesignArtifactStatus, Uuid } from "@/types/ids";
import { DESIGN_ARTIFACT_STATUSES } from "@/types/ids";

export type { DesignArtifact, DesignArtifactStatus };
export { DESIGN_ARTIFACT_STATUSES };

/** Status 5 值 type guard (per REQ-DSG-001) */
export function isDesignArtifactStatus(x: unknown): x is DesignArtifactStatus {
  return (
    typeof x === "string" &&
    (DESIGN_ARTIFACT_STATUSES as readonly string[]).includes(x)
  );
}

/** 完整对象 type guard — 含 version >= 1, review_record_id 可空 */
export function isDesignArtifact(x: unknown): x is DesignArtifact {
  if (typeof x !== "object" || x === null) return false;
  const o = x as Record<string, unknown>;
  return (
    typeof o.id === "string" &&
    typeof o.work_item_id === "string" &&
    typeof o.title === "string" &&
    o.title.length > 0 &&
    isDesignArtifactStatus(o.status) &&
    typeof o.version === "number" &&
    Number.isInteger(o.version) &&
    o.version >= 1 &&
    typeof o.author_id === "string" &&
    typeof o.created_at === "string" &&
    typeof o.updated_at === "string" &&
    (o.review_record_id === null || typeof o.review_record_id === "string")
  );
}

/** 实用工具: review request body 校验 (POST /api/design-artifacts/:id/review) */
export function isReviewRequestBody(
  x: unknown,
): x is { decision: "approve" | "request_changes"; reviewer_id: Uuid; comment?: string } {
  if (typeof x !== "object" || x === null) return false;
  const o = x as Record<string, unknown>;
  return (
    (o.decision === "approve" || o.decision === "request_changes") &&
    typeof o.reviewer_id === "string" &&
    o.reviewer_id.length > 0 &&
    (o.comment === undefined || typeof o.comment === "string")
  );
}
