// frontend/src/mocks/schemas/validation.ts
// ValidationResult + AcceptanceCoverageReport schemas (per REQ-TST-001/002)
//
// 设计依据 (per 缺标比错标安全, 8/26 JST 守门):
//   - docs/test-design.md §6.2.1 (V1 Should-Have Test, 4 Level 维度)
//   - docs/requirements.md §27.6 (Test Level: UnitTestLevel/IntegrationTestLevel/
//     SystemTestLevel/AcceptanceTestLevel, 与 Type 正交)
//   - docs/requirements.md REQ-TST-001/002 (per Level 聚合 + uncovered_by_level)
//   - docs/specs/domain-validation-spec.md (ValidationResult 6 状态机 +
//     evidence_ref 必填 INV-VL-04)
//
// 已知缺口 (TBD 待 basic-design §4.5.6 跟进):
//   1. fixture_path / duration_ms / started_at / ended_at 等运行时字段
//      是否进入 ValidationResult 暂未拍板, UI 投影层先按最小集 5 字段
//   2. AcceptanceCriteria 本身的对象定义在 basic-design §4.4.x (per test-design §6.2.1),
//      本 mock 只引用其 id (linked_ac_ids / uncovered_by_level)
//
// 守门 8/26 JST 引用 (per AGENTS.md §1.2 #1+#4): 禁止回溯叙事, 本文件不引
// "per X 历史形态" / "原本是" 等无 git 证据的回溯叙事; commit 引用以
// `git log -p --follow frontend/src/mocks/schemas/validation.ts` 为准.

import type {
  TestLevel,
  ValidationResultRecord,
  AcceptanceCoverageReport,
} from "@/types/ids";
import { TEST_LEVELS } from "@/types/ids";

export type {
  TestLevel,
  ValidationResultRecord,
  AcceptanceCoverageReport,
};
export { TEST_LEVELS };

// =====================================================================
// Level type guard
// =====================================================================
export function isTestLevel(x: unknown): x is TestLevel {
  return (
    typeof x === "string" && (TEST_LEVELS as readonly string[]).includes(x)
  );
}

// =====================================================================
// ValidationResultRecord type guard
// =====================================================================
const VALIDATION_KINDS = [
  "build",
  "test",
  "lint",
  "contract",
  "security",
] as const;
const VALIDATION_STATUSES = [
  "running",
  "passed",
  "failed",
  "errored",
  "skipped",
  "superseded",
] as const;

export function isValidationResultRecord(
  x: unknown,
): x is ValidationResultRecord {
  if (typeof x !== "object" || x === null) return false;
  const v = x as Record<string, unknown>;
  if (typeof v.id !== "string" || v.id.length === 0) return false;
  if (typeof v.work_item_id !== "string" || v.work_item_id.length === 0) return false;
  if (typeof v.kind !== "string" || !(VALIDATION_KINDS as readonly string[]).includes(v.kind)) return false;
  if (typeof v.status !== "string" || !(VALIDATION_STATUSES as readonly string[]).includes(v.status)) return false;
  if (!isTestLevel(v.level)) return false;
  // INV-VL-04 evidence_ref 必填 (basic-design §4.5.5)
  if (typeof v.evidence_ref !== "string" || v.evidence_ref.length === 0) return false;
  if (!Array.isArray(v.linked_ac_ids)) return false;
  for (const ac of v.linked_ac_ids) {
    if (typeof ac !== "string" || ac.length === 0) return false;
  }
  if (typeof v.created_at !== "string" || v.created_at.length === 0) return false;
  return true;
}

// =====================================================================
// AcceptanceCoverageReport type guard
// =====================================================================
export function isAcceptanceCoverageReport(
  x: unknown,
): x is AcceptanceCoverageReport {
  if (typeof x !== "object" || x === null) return false;
  const r = x as Record<string, unknown>;
  if (typeof r.work_item_id !== "string" || r.work_item_id.length === 0) return false;
  if (typeof r.total_count !== "number" || !Number.isFinite(r.total_count)) return false;
  if (typeof r.covered_count !== "number" || !Number.isFinite(r.covered_count)) return false;
  if (typeof r.by_level !== "object" || r.by_level === null) return false;
  if (typeof r.uncovered_by_level !== "object" || r.uncovered_by_level === null) return false;
  // 4 Level 字段全检
  for (const lvl of TEST_LEVELS) {
    if (typeof (r.by_level as Record<string, unknown>)[lvl] !== "number") return false;
    const arr = (r.uncovered_by_level as Record<string, unknown>)[lvl];
    if (!Array.isArray(arr)) return false;
    for (const ac of arr) {
      if (typeof ac !== "string" || ac.length === 0) return false;
    }
  }
  return true;
}
