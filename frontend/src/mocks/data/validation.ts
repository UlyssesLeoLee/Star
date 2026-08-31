// frontend/src/mocks/data/validation.ts
// MOCK_VALIDATION_RESULTS + MOCK_ACCEPTANCE_COVERAGE fixtures (per REQ-TST-001/002)
//
// 设计依据 (per 缺标比错标安全, 8/26 JST 守门):
//   - docs/test-design.md §6.2.1 (V1 Should-Have Test, 4 Level 维度)
//   - docs/requirements.md §27.6 (Test Level: 4 档 unit/integration/system/acceptance)
//   - docs/specs/domain-validation-spec.md §2 (ValidationResult + AcceptanceCoverage)
//
// 设计约束:
//   - ≥ 8 条 ValidationResult, 覆盖 4 Level 全部 (含同 work_item_id 多 level)
//   - ≥ 3 条 AcceptanceCoverageReport, 含 1 条故意缺 acceptance level
//     (per REQ-TST-002 显式暴露 uncovered_by_level.acceptance 非空)
//   - ISO 8601 UTC 时间戳, evidence_ref 字符串 (URL-like 即可, 不校验远端可达)
//   - linked_ac_ids 关联 AcceptanceCriteria 集合, 至少 wi-001 关联 2+ AC
//
// 已知缺口 (TBD 待 basic-design §4.5.6 跟进):
//   1. AcceptanceCriteria 真实 ID 体系未拍板, 本 mock 用 ac-001..ac-006 占位
//   2. work_item_id 用 wi-NNN 形式占位, 与 types/ids.ts WorkItem.id (Uuid) 不强匹配
//   3. MOCK_ACCEPTANCE_COVERAGE 的 by_level / uncovered_by_level 数字与
//      MOCK_VALIDATION_RESULTS 实际行数不强制一致 (mock 用于展示形态, 不做派生计算)

import type {
  ValidationResultRecord,
  AcceptanceCoverageReport,
} from "@/types/ids";
import { mulberry32 } from "@/mocks/seed";

// 固定 seed 保证可重现 (per docs/frontend/design/mock-data-isolation.md §2.4)
const rand = mulberry32(20260831);

const ISO = (offsetSeconds: number): string => {
  // 基准时间 2026-08-31 09:00:00 UTC (per docs/frontend/design/mock-data-isolation.md §2.4)
  const base = Date.UTC(2026, 7, 31, 9, 0, 0);
  return new Date(base + offsetSeconds * 1000).toISOString();
};

// 故意选一组时间戳, 跨 4 Level 均可观察
// 注意: 命名用 ValidationResultRecord 而非 ValidationResult, 避免与 types/ids.ts §14
// 既有字符串联合 ("pass" | "fail" | ...) 命名冲突 (per ids.ts 已知缺口 #3, 守门 #12 docs 同步).
export const MOCK_VALIDATION_RESULTS: ReadonlyArray<ValidationResultRecord> = [
  // ---- wi-001: full 4-level coverage (happy path) ----
  {
    id: "vr-001",
    work_item_id: "wi-001",
    kind: "test",
    status: "passed",
    level: "unit",
    evidence_ref: "s3://artifacts/wi-001/unit/jest-run-20260831.log",
    linked_ac_ids: ["ac-001", "ac-002"],
    created_at: ISO(60),
  },
  {
    id: "vr-002",
    work_item_id: "wi-001",
    kind: "test",
    status: "passed",
    level: "integration",
    evidence_ref: "s3://artifacts/wi-001/integration/api-test-20260831.log",
    linked_ac_ids: ["ac-001", "ac-002", "ac-003"],
    created_at: ISO(120),
  },
  {
    id: "vr-003",
    work_item_id: "wi-001",
    kind: "test",
    status: "passed",
    level: "system",
    evidence_ref: "s3://artifacts/wi-001/system/e2e-20260831.log",
    linked_ac_ids: ["ac-001", "ac-003"],
    created_at: ISO(180),
  },
  {
    id: "vr-004",
    work_item_id: "wi-001",
    kind: "test",
    status: "passed",
    level: "acceptance",
    evidence_ref: "s3://artifacts/wi-001/acceptance/uat-20260831.md",
    linked_ac_ids: ["ac-001", "ac-002", "ac-003"],
    created_at: ISO(240),
  },
  // ---- wi-002: 3 levels (unit + integration + system, no acceptance) ----
  {
    id: "vr-005",
    work_item_id: "wi-002",
    kind: "test",
    status: "passed",
    level: "unit",
    evidence_ref: "s3://artifacts/wi-002/unit/vitest-20260831.log",
    linked_ac_ids: ["ac-004"],
    created_at: ISO(300),
  },
  {
    id: "vr-006",
    work_item_id: "wi-002",
    kind: "test",
    status: "failed",
    level: "integration",
    evidence_ref: "s3://artifacts/wi-002/integration/api-test-20260831.log",
    linked_ac_ids: ["ac-004"],
    created_at: ISO(360),
  },
  {
    id: "vr-007",
    work_item_id: "wi-002",
    kind: "lint",
    status: "errored",
    level: "system",
    evidence_ref: "s3://artifacts/wi-002/system/lint-fail-20260831.log",
    linked_ac_ids: [],
    created_at: ISO(420),
  },
  // ---- wi-003: only unit (initial spike) ----
  {
    id: "vr-008",
    work_item_id: "wi-003",
    kind: "build",
    status: "passed",
    level: "unit",
    evidence_ref: "s3://artifacts/wi-003/unit/cargo-build-20260831.log",
    linked_ac_ids: ["ac-005"],
    created_at: ISO(480),
  },
  {
    id: "vr-009",
    work_item_id: "wi-003",
    kind: "security",
    status: "skipped",
    level: "integration",
    evidence_ref: "s3://artifacts/wi-003/integration/security-skipped.md",
    linked_ac_ids: ["ac-005"],
    created_at: ISO(540),
  },
  // ---- wi-004: superseded record (lifecycle demo) ----
  {
    id: "vr-010",
    work_item_id: "wi-004",
    kind: "contract",
    status: "superseded",
    level: "system",
    evidence_ref: "s3://artifacts/wi-004/system/contract-v1-superseded.json",
    linked_ac_ids: ["ac-006"],
    created_at: ISO(600),
  },
];

// 3 条覆盖报告, wi-002 故意缺 acceptance level (per REQ-TST-002 显式)
export const MOCK_ACCEPTANCE_COVERAGE: ReadonlyArray<AcceptanceCoverageReport> = [
  {
    work_item_id: "wi-001",
    total_count: 3,
    covered_count: 3,
    by_level: {
      unit: 2,
      integration: 3,
      system: 2,
      acceptance: 3,
    },
    uncovered_by_level: {
      unit: [],
      integration: [],
      system: [],
      acceptance: [],
    },
  },
  {
    work_item_id: "wi-002",
    total_count: 1,
    covered_count: 0,
    by_level: {
      unit: 1,
      integration: 0,
      system: 0,
      acceptance: 0,
    },
    // per REQ-TST-002: 缺 acceptance level 时 UI/CLI 明确指出
    uncovered_by_level: {
      unit: [],
      integration: ["ac-004"],
      system: ["ac-004"],
      acceptance: ["ac-004"],
    },
  },
  {
    work_item_id: "wi-003",
    total_count: 1,
    covered_count: 1,
    by_level: {
      unit: 1,
      integration: 1,
      system: 0,
      acceptance: 0,
    },
    uncovered_by_level: {
      unit: [],
      integration: [],
      system: ["ac-005"],
      acceptance: ["ac-005"],
    },
  },
];

// 抑制 lint warning: rand 当前未使用, 留作未来数据扩展入口
void rand;
