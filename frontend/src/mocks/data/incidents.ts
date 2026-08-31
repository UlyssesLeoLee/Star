// =====================================================================
// frontend/src/mocks/data/incidents.ts
// Mock IncidentRecord 种子数据 (per test-design §6.3.4 / REQ-OPS-001/002/003)
// =====================================================================
//
// 设计依据:
//   - docs/test-design.md v0.3 §6.3.4 (V1 Should-Have Test T3, TBD)
//   - docs/requirements.md §29.1 (REQ-OPS-001/002/003) + §30.6 (边界)
//   - 守门 (per AGENTS.md §1.2 #3 缺标比错标 + §1.2 #4 子代理授权):
//     数据覆盖 2 种 source (human_entry + integration_webhook), 至少 1 条
//     affected_ac_ids 非空 (用于测试 REQ-OPS-002 标注)
//
// 种子 (per mock-data-isolation.md §2.4 + mulberry32):
//   seed = 0xC0FFEE (固定) → 保证 CI 稳定可重现
//   当前 4 条 mock 是硬编码 (seed 用于运行时生成新数据时)
// =====================================================================

import type { IncidentRecord } from "@/types/ids";
import { isIncidentRecord } from "@/mocks/schemas/incident";
import { mulberry32 } from "@/mocks/seed";

/** mulberry32 实例 — seed = 0xC0FFEE, 用于 runtime 生成新数据 */
export const incidentRand = mulberry32(0xc0ffee);

/**
 * MOCK_INCIDENTS — 4 条种子 IncidentRecord
 *
 * 覆盖 (per 任务要求):
 *   - 2 条 human_entry (人工录入, 包含 1 条 affected_ac_ids 非空)
 *   - 2 条 integration_webhook (经 §18 Integration Webhook 转登)
 *   - 至少 1 条 linked_work_item_ids 为空 (per REQ-OPS-001 "0..N WorkItem")
 *   - 至少 1 条 affected_ac_ids 非空 (per REQ-OPS-002 标注证据不充分)
 */
export const MOCK_INCIDENTS: IncidentRecord[] = [
  {
    id: "inc-001",
    title: "Production deploy failed — staging differs from production",
    source: "human_entry",
    linked_work_item_ids: ["wi-1001", "wi-1002"],
    // REQ-OPS-002 标注: AC-T3 部署回滚的证据不充分 (无 staging diff 截图)
    affected_ac_ids: ["ac-t3-deploy-rollback"],
    occurred_at: "2026-08-25T14:23:00Z",
    recorded_at: "2026-08-25T15:01:00Z",
    recorded_by: "user-001",
    notes: "Reporter noted that rollback test never executed in staging environment.",
  },
  {
    id: "inc-002",
    title: "API timeout spike (5xx 12% → 31%) detected by Prometheus webhook",
    source: "integration_webhook",
    // REQ-OPS-001 允许 0 个关联 WorkItem (per test-design §6.3.4 "0..N")
    linked_work_item_ids: [],
    affected_ac_ids: [],
    occurred_at: "2026-08-26T03:47:00Z",
    recorded_at: "2026-08-26T03:47:30Z",
    recorded_by: "user-system-webhook", // 系统 user 来自 §18 integration
    notes: "Auto-routed from Prometheus alertmanager via §18 Integration Webhook. No auto-remediation attempted.",
  },
  {
    id: "inc-003",
    title: "Auth provider SAML response malformed — 3 tenants affected",
    source: "integration_webhook",
    linked_work_item_ids: ["wi-1050"],
    // REQ-OPS-002 标注: AC-LOGIN-003 锁定策略证据不充分
    affected_ac_ids: ["ac-login-003"],
    occurred_at: "2026-08-27T09:12:00Z",
    recorded_at: "2026-08-27T09:15:00Z",
    recorded_by: "user-system-webhook",
    notes: "SAML response truncated at signature element. Webhook re-fired 3 times; loop_protection_key deduped.",
  },
  {
    id: "inc-004",
    title: "Data export CSV contains stale tenant data (3 records from 2026-07)",
    source: "human_entry",
    linked_work_item_ids: ["wi-1100", "wi-1101", "wi-1102"],
    affected_ac_ids: [],
    occurred_at: "2026-08-28T11:30:00Z",
    recorded_at: "2026-08-28T11:45:00Z",
    recorded_by: "user-002",
    notes: "Manual review found export query misses 2026-08 filter. Not auto-detected.",
  },
];

/**
 * Self-check (per mock-data-isolation.md §2.6):
 * 全部 4 条必须通过 isIncidentRecord type guard,
 * 否则启动时立即 throw (CI 必能发现)
 */
function assertAllValid(): void {
  for (const inc of MOCK_INCIDENTS) {
    if (!isIncidentRecord(inc)) {
      throw new Error(
        `MOCK_INCIDENTS contains invalid record: ${JSON.stringify(inc)}`,
      );
    }
  }
}
assertAllValid();
