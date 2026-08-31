// =====================================================================
// frontend/src/lib/incident-guard.ts
// IncidentRecord 边界守门 (per test-design §6.3.4 / REQ-OPS-003)
// =====================================================================
//
// 纯函数 (no IO, no React, no store), 客户端提交前预校验 IncidentRecord
// 是否违反 REQ-OPS-003 §30.6 边界 (主动探查 / 处理告警 / 自动回滚)。
//
// 设计依据:
//   - docs/test-design.md v0.3 §6.3.4 (V1 Should-Have Test T3, TBD)
//   - docs/requirements.md §29.1 (REQ-OPS-001/002/003) + §30.6 (边界)
//
// 守门硬约束 (per AGENTS.md §1.2 #4 子代理授权):
//   - 守门 #1 禁回溯叙事
//   - 守门 #3 缺标比错标 (校验宽松度宁低勿高)
//   - 8/26 JST 守门: 引用必 git 实证
//
// 已知缺口 (per 缺标比错标 + §1.2 #3 显式列):
//   1. notes 关键词清单当前 3 个 (auto_rollback / auto_remediation /
//      alert_handler); 若 basic-design §30.6 补新关键词需同步回填
//   2. linked_work_item_ids / affected_ac_ids 的 Uuid 格式 (e.g. uuid-v4
//      regex) 暂未强校验, 仅 type-level string 检查 (per 当前 V1 schema
//      简化)
//   3. 配套 MSW handler 端 (mocks/handlers/incidents.ts) 也做相同校验,
//      两层独立 (defense in depth)
// =====================================================================

import type { IncidentRecord, IncidentSource, Uuid } from "@/types/ids";

/** 守门 #3 失败分类 (per test-design §6.3.4 备注 + 任务要求) */
export type IncidentValidationReason =
  | "ok"
  | "invalid_source"
  | "missing_work_item"
  | "missing_recorder"
  | "auto_action_attempted";

/** 守门结果 */
export interface IncidentValidation {
  valid: boolean;
  reason: IncidentValidationReason;
  /** 失败时指出哪个字段触发失败 (per 任务要求 violation_field) */
  violation_field?: string;
}

/** 允许的 source 值 (per REQ-OPS-003, 仅 2 源) */
const ALLOWED_SOURCES: readonly IncidentSource[] = [
  "human_entry",
  "integration_webhook",
] as const;

/**
 * 禁止的关键词清单 (per REQ-OPS-003 / §30.6 边界):
 *   - auto_rollback   — 自动回滚
 *   - auto_remediation — 自动修复
 *   - alert_handler   — 主动处理告警
 *
 * 匹配规则: 不区分大小写 substring (covers "auto_rollback"/"AUTO_ROLLBACK"/
 * "请执行 auto_rollback 修复")。
 */
const FORBIDDEN_NOTES_KEYWORDS: readonly string[] = [
  "auto_rollback",
  "auto_remediation",
  "alert_handler",
] as const;

/** Uuid 简化校验: 非空字符串即可 (per 当前 V1 schema 简化) */
function isValidUuid(v: unknown): v is Uuid {
  return typeof v === "string" && v.length > 0;
}

/**
 * 验证 IncidentRecord 不违反 REQ-OPS-003 边界:
 *   - source 必须是 human_entry / integration_webhook
 *   - 任何字段(含 notes)不含 auto_rollback / auto_remediation / alert_handler 关键词
 *   - recorded_by 必须存在
 *   - linked_work_item_ids / affected_ac_ids 可空, 但若提供必须是有效 Uuid
 *
 * @param record  — partial record (测试时方便构造边界)
 * @returns       — 守门结果
 */
export function validateIncidentRecord(
  record: Partial<IncidentRecord>,
): IncidentValidation {
  // 1. source 必为 human_entry / integration_webhook
  if (
    typeof record.source !== "string" ||
    !(ALLOWED_SOURCES as readonly string[]).includes(record.source)
  ) {
    return {
      valid: false,
      reason: "invalid_source",
      violation_field: "source",
    };
  }

  // 2. recorded_by 必须存在 (per REQ-OPS-003 "human user id")
  if (!isValidUuid(record.recorded_by)) {
    return {
      valid: false,
      reason: "missing_recorder",
      violation_field: "recorded_by",
    };
  }

  // 3. linked_work_item_ids / affected_ac_ids 可空, 但若提供必须全为 string
  if (record.linked_work_item_ids !== undefined) {
    if (
      !Array.isArray(record.linked_work_item_ids) ||
      !record.linked_work_item_ids.every(isValidUuid)
    ) {
      return {
        valid: false,
        reason: "missing_work_item",
        violation_field: "linked_work_item_ids",
      };
    }
  }
  if (record.affected_ac_ids !== undefined) {
    if (
      !Array.isArray(record.affected_ac_ids) ||
      !record.affected_ac_ids.every(isValidUuid)
    ) {
      return {
        valid: false,
        reason: "missing_work_item",
        violation_field: "affected_ac_ids",
      };
    }
  }

  // 4. notes 必为 string 且不得含禁止关键词 (auto_rollback / 等)
  if (typeof record.notes !== "string") {
    return {
      valid: false,
      reason: "auto_action_attempted",
      violation_field: "notes",
    };
  }
  const notesLower = record.notes.toLowerCase();
  for (const kw of FORBIDDEN_NOTES_KEYWORDS) {
    if (notesLower.includes(kw)) {
      return {
        valid: false,
        reason: "auto_action_attempted",
        violation_field: "notes",
      };
    }
  }

  return { valid: true, reason: "ok" };
}
