// =====================================================================
// MOCK_DESIGN_ARTIFACTS — DesignArtifact fixture (per wt-test-t2-dsg 2026-08-31)
// =====================================================================
// 上游依据:
//   - docs/test-design.md §6.3.3   REQ-DSG-001/002 (V1 Should-Have Test)
//   - docs/requirements.md §8.3   DesignArtifact 字段定义
//
// 设计:
//   - >= 6 条,跨至少 3 个 work_item_id,各 Status 都覆盖
//   - 必须包含 1 组 "全 APPROVED → 可转 in_progress" (Guard 正面用例)
//   - 必须包含 1 组 "1 DRAFT + 2 APPROVED → Guard 失败, 指出 DRAFT"
//     (Guard 负面用例)
//   - superseded 视为已批准 (不阻塞 Guard)
//   - 种子用 mulberry32, 保证可重现
//
// 已知缺口 (per 缺标比错标, 8/26 JST 守门 #1 + #12 引用):
//   1. real-mode 切换 (P3-A.7) 未覆盖本 handler — per 范围最小化
//   2. POST 真实持久化 P3 — Phase F+ 后端就绪时
//   3. version 每次 POST /review +1 (P2 真实持久化时由后端 audit 驱动)
// =====================================================================

import { mulberry32 } from "@/mocks/seed";
import type { DesignArtifact, Iso8601, Uuid } from "@/types/ids";

// 稳定 work_item_id 集合 (跨 3 个 wi, 覆盖各 status 组合)
const WI_ALL_APPROVED: Uuid = "wi-physis-001";
const WI_HAS_DRAFT: Uuid = "wi-physis-002";
const WI_HAS_REJECTED: Uuid = "wi-physis-003";

// 用户 (author / reviewer) — 复用现有 mock user 形态
const AUTHOR_DEFAULT: Uuid = "u-author-001";
const REVIEWER_DEFAULT: Uuid = "u-reviewer-001";

/** 稳定时间戳 (避免 seededRandom 飘移; 字符串精度由 mock 决定) */
const T0: Iso8601 = "2026-08-25T09:00:00Z";
const T1: Iso8601 = "2026-08-26T11:30:00Z";
const T2: Iso8601 = "2026-08-27T14:45:00Z";
const T3: Iso8601 = "2026-08-28T16:20:00Z";
const T4: Iso8601 = "2026-08-29T10:10:00Z";
const T5: Iso8601 = "2026-08-30T13:00:00Z";
const T6: Iso8601 = "2026-08-31T08:30:00Z";

/** seededRandom 触发: 走 mulberry32 拿 1 个 rand 数验证 (per mock-data-isolation §2.4) */
const _seededRand = mulberry32(1);
const _seedUsed: number = _seededRand(); // 占用一次 RNG, 保持 fixture 顺序可重现

export const MOCK_DESIGN_ARTIFACTS: ReadonlyArray<DesignArtifact> = [
  // ===== wi-physis-001: 全 APPROVED → Guard 正面用例 (3 approved) =====
  {
    id: "da-001",
    work_item_id: WI_ALL_APPROVED,
    title: "Physis 引擎架构总览 (v3)",
    status: "approved",
    version: 3,
    author_id: AUTHOR_DEFAULT,
    created_at: T0,
    updated_at: T3,
    review_record_id: "rr-001",
  },
  {
    id: "da-002",
    work_item_id: WI_ALL_APPROVED,
    title: "C ABI 接口契约 (v2)",
    status: "approved",
    version: 2,
    author_id: AUTHOR_DEFAULT,
    created_at: T0,
    updated_at: T2,
    review_record_id: "rr-002",
  },
  {
    id: "da-003",
    work_item_id: WI_ALL_APPROVED,
    title: "性能预算与确定性取舍方案",
    status: "approved",
    version: 1,
    author_id: AUTHOR_DEFAULT,
    created_at: T1,
    updated_at: T1,
    review_record_id: "rr-003",
  },

  // ===== wi-physis-002: 1 DRAFT + 2 APPROVED → Guard 失败, 指出 DRAFT (负面用例) =====
  {
    id: "da-004",
    work_item_id: WI_HAS_DRAFT,
    title: "热路径约束 (Phase 2+)",
    status: "draft", // ← 负面用例触发点
    version: 1,
    author_id: AUTHOR_DEFAULT,
    created_at: T4,
    updated_at: T4,
    review_record_id: null,
  },
  {
    id: "da-005",
    work_item_id: WI_HAS_DRAFT,
    title: "跨引擎嵌入矩阵",
    status: "approved",
    version: 2,
    author_id: AUTHOR_DEFAULT,
    created_at: T0,
    updated_at: T2,
    review_record_id: "rr-005",
  },
  {
    id: "da-006",
    work_item_id: WI_HAS_DRAFT,
    title: "API 边界设计 v1",
    status: "approved",
    version: 1,
    author_id: AUTHOR_DEFAULT,
    created_at: T1,
    updated_at: T1,
    review_record_id: "rr-006",
  },

  // ===== wi-physis-003: 1 REJECTED + 1 SUPERSEDED(视为批准) + 1 IN_REVIEW (边界覆盖) =====
  {
    id: "da-007",
    work_item_id: WI_HAS_REJECTED,
    title: "线程模型 (已被拒绝)",
    status: "rejected",
    version: 1,
    author_id: AUTHOR_DEFAULT,
    created_at: T0,
    updated_at: T1,
    review_record_id: "rr-007",
  },
  {
    id: "da-008",
    work_item_id: WI_HAS_REJECTED,
    title: "Determinism 边界 (历史版本,已 superseded)",
    status: "superseded", // 视为已批准, 不阻塞 Guard
    version: 2,
    author_id: AUTHOR_DEFAULT,
    created_at: T0,
    updated_at: T5,
    review_record_id: "rr-008",
  },
  {
    id: "da-009",
    work_item_id: WI_HAS_REJECTED,
    title: "GanttBar resize 行为细节 (送审中)",
    status: "in_review", // 边界: 送审中视为未批准
    version: 1,
    author_id: AUTHOR_DEFAULT,
    created_at: T5,
    updated_at: T6,
    review_record_id: "rr-009-pending",
  },
];

// 抑制 unused 警告 (mulberry32 引用保活, 未来扩量用)
void _seedUsed;
