// frontend/src/mocks/schemas/five-domain.ts
// 5 域业务子域统一 schemas (per test-design.md v0.2 §2.1.2 + docs/ddd/01-player-bc.md ~ 05-admin-bc.md)
//
// 设计依据:
//   - test-design.md v0.2 (2026-08-26) §2.1.2 — 前端 Vitest mock 完整化
//   - test-design.md v0.2 (2026-08-26) §3.1 + §3.3 — Infrastructure 层 Mock Adapter 完整化
//   - docs/ddd/01-player-bc.md ~ 05-admin-bc.md — 5 域 DDD 边界
//   - docs/specs/domain-*-spec.md — 25 domain spec 字段
//   - frontend/src/types/ids.ts — 复用 WorktreeStatus (17 状态)
//
// 5 域映射表 (per wt-test-mock-5d 任务书 + 既有 mock 命名, 不重命名):
//   ┌──────────┬──────────────────────────────┬────────────────────────────────────────────┐
//   │ 域       │ 子域 (DDD)                    │ mock 端点 (本文件支持)                       │
//   ├──────────┼──────────────────────────────┼────────────────────────────────────────────┤
//   │ player   │ user / identity / workspace  │ /api/workspaces (本任务新加)                  │
//   │ economy  │ billing / pricing / cost     │ /api/billing, /api/billing/usage (本任务新加) │
//   │ match    │ workflow / 状态机 / saga     │ /api/worktrees, /transition (本任务新加)      │
//   │ social   │ collaboration / 通知          │ /api/comments (本任务新加; inbox 已有)        │
//   │ admin    │ RBAC / permission / tenant   │ /api/tenants, /api/rbac/roles (本任务新加)    │
//   └──────────┴──────────────────────────────┴────────────────────────────────────────────┘
//
// TBD (per 守门 #1 缺标比错标安全):
//   - 5 域 Lead 真人到位后, BoundedContext 命名/边界请 DDD Review 拍板
//   - 当前文件头注释只列端点 + 域映射, 不写"per X 历史形态"/"per X 升版前" 等回溯叙事
//   - 业务子域字段遵循 5 域 DDD 文档, 但 mock 端点的具体 shape (snake_case / ISO 8601) 仅作前端测试用
//   - Phase F+ 后端真实接入时, 这些 type 与 backend 真实类型保持一致, 只改 data 文件
//
// 守门引用:
//   - 8/26 JST 守门: 缺标比错标安全 (per AGENTS.md §4 #11)
//   - 8/26 JST 守门: 禁回溯叙事 (per AGENTS.md §4 #8 + §1.2 #1)
//   - 8/26 JST 守门: 子代理授权边界要写明"无证据叙事 = 禁止" (per AGENTS.md §4 #12 + §1.2 #4)
//
// 5 域 Lead 真人 review 跟进: §E.5 / §F.1 (per STAR-P3-WBS-001 §12.4 阻塞)

import type { WorktreeStatus } from "@/types/ids";

// =====================================================================
// 1. player 域 — Workspace (id/tenant_id/name/owner_id)
// =====================================================================

export interface Workspace {
  id: string;
  tenant_id: string;
  name: string;
  owner_id: string;
  /** ISO 8601 timestamp (e.g. "2026-08-15T10:00:00Z") */
  created_at: string;
  /** number of members (mock convenience) */
  member_count: number;
}

export function isWorkspace(x: unknown): x is Workspace {
  if (typeof x !== "object" || x === null) return false;
  const r = x as Record<string, unknown>;
  return (
    typeof r.id === "string" &&
    r.id.length > 0 &&
    typeof r.tenant_id === "string" &&
    r.tenant_id.length > 0 &&
    typeof r.name === "string" &&
    r.name.length > 0 &&
    typeof r.owner_id === "string" &&
    r.owner_id.length > 0 &&
    typeof r.created_at === "string" &&
    r.created_at.length > 0 &&
    typeof r.member_count === "number" &&
    Number.isFinite(r.member_count)
  );
}

// =====================================================================
// 2. economy 域 — BillingEntry (id/tenant_id/period_start/period_end/amount_cents)
// =====================================================================

export interface BillingEntry {
  id: string;
  tenant_id: string;
  /** ISO 8601 date (e.g. "2026-08-01") — period start (inclusive) */
  period_start: string;
  /** ISO 8601 date (e.g. "2026-08-31") — period end (inclusive) */
  period_end: string;
  /** amount in cents (integer) — keeps money handling deterministic */
  amount_cents: number;
  /** ISO 4217 currency code (default USD) */
  currency: "USD" | "EUR" | "JPY" | "CNY";
  status: "draft" | "finalized" | "paid" | "overdue";
  /** Itemized cost breakdown (mock convenience) */
  line_items: { label: string; amount_cents: number }[];
}

export function isBillingEntry(x: unknown): x is BillingEntry {
  if (typeof x !== "object" || x === null) return false;
  const r = x as Record<string, unknown>;
  return (
    typeof r.id === "string" &&
    r.id.length > 0 &&
    typeof r.tenant_id === "string" &&
    r.tenant_id.length > 0 &&
    typeof r.period_start === "string" &&
    /^\d{4}-\d{2}-\d{2}$/.test(r.period_start) &&
    typeof r.period_end === "string" &&
    /^\d{4}-\d{2}-\d{2}$/.test(r.period_end) &&
    typeof r.amount_cents === "number" &&
    Number.isInteger(r.amount_cents) &&
    typeof r.currency === "string" &&
    ["USD", "EUR", "JPY", "CNY"].includes(r.currency) &&
    typeof r.status === "string" &&
    ["draft", "finalized", "paid", "overdue"].includes(r.status) &&
    Array.isArray(r.line_items)
  );
}

// =====================================================================
// 3. match 域 — WorktreeSnapshot (id/project_id/branch/agent_id/status)
// =====================================================================

/**
 * Lightweight projection of Worktree (per types/ids.ts) used in mock responses.
 * The full Worktree type lives in @/types/ids; this snapshot keeps mock
 * payloads small and explicit about which fields matter to the frontend.
 */
export interface WorktreeSnapshot {
  id: string;
  project_id: string;
  branch: string;
  agent_id: string | null;
  status: WorktreeStatus;
  /** ISO 8601 timestamp */
  created_at: string;
  /** ISO 8601 timestamp — last status change */
  last_event_at: string;
}

export function isWorktreeSnapshot(x: unknown): x is WorktreeSnapshot {
  if (typeof x !== "object" || x === null) return false;
  const r = x as Record<string, unknown>;
  return (
    typeof r.id === "string" &&
    r.id.length > 0 &&
    typeof r.project_id === "string" &&
    r.project_id.length > 0 &&
    typeof r.branch === "string" &&
    r.branch.length > 0 &&
    (r.agent_id === null || typeof r.agent_id === "string") &&
    typeof r.status === "string" &&
    r.status.length > 0 &&
    typeof r.created_at === "string" &&
    r.created_at.length > 0 &&
    typeof r.last_event_at === "string" &&
    r.last_event_at.length > 0
  );
}

// =====================================================================
// 4. social 域 — Comment (id/work_item_id/author_id/body/created_at)
// =====================================================================

export interface Comment {
  id: string;
  work_item_id: string;
  author_id: string;
  body: string;
  /** ISO 8601 timestamp */
  created_at: string;
  /** Soft-delete flag — DELETE endpoint flips to true (mock convenience) */
  deleted: boolean;
}

export function isComment(x: unknown): x is Comment {
  if (typeof x !== "object" || x === null) return false;
  const r = x as Record<string, unknown>;
  return (
    typeof r.id === "string" &&
    r.id.length > 0 &&
    typeof r.work_item_id === "string" &&
    r.work_item_id.length > 0 &&
    typeof r.author_id === "string" &&
    r.author_id.length > 0 &&
    typeof r.body === "string" &&
    typeof r.created_at === "string" &&
    r.created_at.length > 0 &&
    typeof r.deleted === "boolean"
  );
}

// =====================================================================
// 5. admin 域 — Tenant (id/name/plan/created_at) + RbacRole (id/tenant_id/name/permissions[])
// =====================================================================

export type TenantPlan = "starter" | "pro" | "enterprise";

export interface Tenant {
  id: string;
  name: string;
  plan: TenantPlan;
  /** ISO 8601 timestamp */
  created_at: string;
  /** True if tenant is active (mock convenience) */
  active: boolean;
}

export function isTenant(x: unknown): x is Tenant {
  if (typeof x !== "object" || x === null) return false;
  const r = x as Record<string, unknown>;
  return (
    typeof r.id === "string" &&
    r.id.length > 0 &&
    typeof r.name === "string" &&
    r.name.length > 0 &&
    typeof r.plan === "string" &&
    ["starter", "pro", "enterprise"].includes(r.plan) &&
    typeof r.created_at === "string" &&
    r.created_at.length > 0 &&
    typeof r.active === "boolean"
  );
}

export interface RbacRole {
  id: string;
  tenant_id: string;
  name: string;
  /** Permission keys — e.g. "workspace:read", "billing:read", "admin:tenant:write" */
  permissions: string[];
  /** ISO 8601 timestamp */
  created_at: string;
}

export function isRbacRole(x: unknown): x is RbacRole {
  if (typeof x !== "object" || x === null) return false;
  const r = x as Record<string, unknown>;
  return (
    typeof r.id === "string" &&
    r.id.length > 0 &&
    typeof r.tenant_id === "string" &&
    r.tenant_id.length > 0 &&
    typeof r.name === "string" &&
    r.name.length > 0 &&
    Array.isArray(r.permissions) &&
    r.permissions.every((p) => typeof p === "string") &&
    typeof r.created_at === "string" &&
    r.created_at.length > 0
  );
}
