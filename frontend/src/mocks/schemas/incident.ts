// =====================================================================
// frontend/src/mocks/schemas/incident.ts
// IncidentRecord schemas (per test-design §6.3.4 / REQ-OPS-001/002/003)
// =====================================================================
//
// 设计依据 (per AGENTS.md §1.2 #2 引用必 git 实证):
//   - docs/test-design.md v0.3 §6.3.4 (V1 Should-Have Test T3, TBD)
//   - docs/requirements.md §29.1 (REQ-OPS-001/002/003) + §30.6 (边界)
//   - docs/specs/domain-audit-spec.md (Audit 9 问必答, 旁证: IncidentRecord
//     与 AuditEvent 是两类事件 — Audit 写"系统动作", Incident 写"事故痕迹")
//
// 已知缺口 (per 守门 缺标比错标 + §1.2 #3 显式列):
//   1. IncidentRecord 严重度/状态/分类字段  TBD 等 basic-design §30.6 跟进
//      (test-design §6.3.4 §6.2 备注: 当前无法在 Schema 中分类)
//   2. 3 项非能力端点的具体错误文案  TBD 占位 "REQ-OPS-003 boundary",
//      等 basic-design §30.6 拍板后回填
//   3. IncidentRecord ↔ AuditEvent 联表查询 (例如 "按 actor 找所有事故")
//      当前仅 IncidentRecord 独立建模, AuditEvent 写入路径未实装
//   4. 集成 webhook (§18 Integration Webhook) 触发的 side effect 留 P2+
//      完整路径
//
// 守门引用 (per AGENTS.md §1.2 #4 子代理授权写明边界):
//   - 守门 #1 禁回溯叙事
//   - 守门 #3 缺标比错标安全 (incident-guard.test.ts 宁少勿错)
//   - 守门 #4 AI 协作文档治理 (本 schema 不可用作真实生产 schema)
//   - 8/26 JST 守门: 引用必 git 实证 (BAS 引用前 git log -p --follow)
// =====================================================================

import type { IncidentRecord, IncidentSource } from "@/types/ids";

/** IncidentSource 仅 2 个允许值 (per REQ-OPS-003) */
const ALLOWED_SOURCES: readonly IncidentSource[] = ["human_entry", "integration_webhook"] as const;

/** isIncidentSource — 2 值 type guard */
export function isIncidentSource(x: unknown): x is IncidentSource {
  return typeof x === "string" && (ALLOWED_SOURCES as readonly string[]).includes(x);
}

/**
 * isIncidentRecord — 严格 type guard
 *
 * 校验项 (per REQ-OPS-001/002/003 + §30.6 边界):
 *   1. source 必为 human_entry | integration_webhook (拒绝 auto_detect /
 *      alert_processing / auto_rollback 任何伪造)
 *   2. linked_work_item_ids / affected_ac_ids 必为 string[] (允许空 = 0..N)
 *   3. occurred_at / recorded_at / recorded_by / title 必为 string (非空)
 *   4. notes 必为 string (空字符串允许, 但内容不得含 auto_rollback 等
 *      关键词 — 该层校验交由 incident-guard.ts 负责)
 */
export function isIncidentRecord(x: unknown): x is IncidentRecord {
  if (typeof x !== "object" || x === null) return false;
  const o = x as Record<string, unknown>;
  if (typeof o.id !== "string" || o.id.length === 0) return false;
  if (typeof o.title !== "string") return false;
  if (!isIncidentSource(o.source)) return false;
  if (!Array.isArray(o.linked_work_item_ids)) return false;
  if (!o.linked_work_item_ids.every((v) => typeof v === "string")) return false;
  if (!Array.isArray(o.affected_ac_ids)) return false;
  if (!o.affected_ac_ids.every((v) => typeof v === "string")) return false;
  if (typeof o.occurred_at !== "string") return false;
  if (typeof o.recorded_at !== "string") return false;
  if (typeof o.recorded_by !== "string") return false;
  if (typeof o.notes !== "string") return false;
  return true;
}
