// frontend/src/mocks/data/five-domain.ts
// 5 域业务 mock 数据 (per test-design.md v0.2 §2.1.2)
//
// 数据量守门 (per wt-test-mock-5d 任务书):
//   MOCK_WORKSPACES     ≥ 3 条
//   MOCK_BILLING        ≥ 4 条, 跨 3 个月
//   MOCK_WORKTREES      ≥ 5 条, 跨 3 个 project_id
//   MOCK_COMMENTS       ≥ 6 条, 跨 2 个 work_item_id
//   MOCK_TENANTS        ≥ 2 条, 含 plan = "starter" | "pro" | "enterprise"
//   MOCK_RBAC_ROLES     ≥ 3 条, 覆盖 admin/member/viewer
//
// 种子: mulberry32(1) — deterministic, CI-stable (per mock-data-isolation.md §2.4)
//
// 5 域映射见 frontend/src/mocks/schemas/five-domain.ts 文件头注释 + 5 域映射表
// 已知缺口: 5 域 Lead 真人 review §E.5/F.1 跟进 (per STAR-P3-WBS-001 §12.4 阻塞)

import { mulberry32 } from "@/mocks/seed";
import type {
  Workspace,
  BillingEntry,
  WorktreeSnapshot,
  Comment,
  Tenant,
  RbacRole,
} from "@/mocks/schemas/five-domain";
import type { WorktreeStatus } from "@/types/ids";

// =====================================================================
// 1. player 域 — MOCK_WORKSPACES (4 rows)
// =====================================================================

export const MOCK_WORKSPACES: ReadonlyArray<Workspace> = [
  {
    id: "ws-physis",
    tenant_id: "t-acme",
    name: "Physis / GVPE",
    owner_id: "ag-001",
    created_at: "2026-04-12T09:00:00Z",
    member_count: 6,
  },
  {
    id: "ws-star-core",
    tenant_id: "t-acme",
    name: "Star Core",
    owner_id: "ag-004",
    created_at: "2026-05-20T11:30:00Z",
    member_count: 4,
  },
  {
    id: "ws-docs-adr",
    tenant_id: "t-acme",
    name: "Docs / ADR",
    owner_id: "ag-002",
    created_at: "2026-06-08T14:15:00Z",
    member_count: 2,
  },
  {
    id: "ws-experimental",
    tenant_id: "t-acme",
    name: "Experimental",
    owner_id: "ag-003",
    created_at: "2026-07-22T08:45:00Z",
    member_count: 3,
  },
];

// =====================================================================
// 2. economy 域 — MOCK_BILLING (5 rows, 跨 3 个月: 6/7/8 月)
// =====================================================================

export const MOCK_BILLING: ReadonlyArray<BillingEntry> = [
  {
    id: "bill-2026-06-t-acme",
    tenant_id: "t-acme",
    period_start: "2026-06-01",
    period_end: "2026-06-30",
    amount_cents: 124830,
    currency: "USD",
    status: "paid",
    line_items: [
      { label: "Compute (SRE)", amount_cents: 68420 },
      { label: "LLM tokens", amount_cents: 41210 },
      { label: "Storage", amount_cents: 15200 },
    ],
  },
  {
    id: "bill-2026-07-t-acme",
    tenant_id: "t-acme",
    period_start: "2026-07-01",
    period_end: "2026-07-31",
    amount_cents: 148720,
    currency: "USD",
    status: "paid",
    line_items: [
      { label: "Compute (SRE)", amount_cents: 72100 },
      { label: "LLM tokens", amount_cents: 59820 },
      { label: "Storage", amount_cents: 16800 },
    ],
  },
  {
    id: "bill-2026-08-t-acme",
    tenant_id: "t-acme",
    period_start: "2026-08-01",
    period_end: "2026-08-31",
    amount_cents: 187240,
    currency: "USD",
    status: "finalized",
    line_items: [
      { label: "Compute (SRE)", amount_cents: 89400 },
      { label: "LLM tokens", amount_cents: 78640 },
      { label: "Storage", amount_cents: 19200 },
    ],
  },
  {
    id: "bill-2026-07-t-beta",
    tenant_id: "t-beta",
    period_start: "2026-07-01",
    period_end: "2026-07-31",
    amount_cents: 24180,
    currency: "EUR",
    status: "paid",
    line_items: [
      { label: "Compute (SRE)", amount_cents: 12400 },
      { label: "LLM tokens", amount_cents: 8780 },
      { label: "Storage", amount_cents: 3000 },
    ],
  },
  {
    id: "bill-2026-08-t-beta",
    tenant_id: "t-beta",
    period_start: "2026-08-01",
    period_end: "2026-08-31",
    amount_cents: 31420,
    currency: "EUR",
    status: "draft",
    line_items: [
      { label: "Compute (SRE)", amount_cents: 16200 },
      { label: "LLM tokens", amount_cents: 12100 },
      { label: "Storage", amount_cents: 3120 },
    ],
  },
];

// =====================================================================
// 3. match 域 — MOCK_WORKTREES (6 rows, 跨 3 个 project_id)
// 状态分布覆盖 WorktreeStatus 17 状态中的 6 个 (per frontend/src/types/ids.ts)
// =====================================================================

export const MOCK_WORKTREES: ReadonlyArray<WorktreeSnapshot> = [
  {
    id: "wt-physis-gvpe",
    project_id: "proj-physis",
    branch: "feat/physx-step",
    agent_id: "ag-002",
    status: "active",
    created_at: "2026-08-12T10:00:00Z",
    last_event_at: "2026-08-30T22:18:00Z",
  },
  {
    id: "wt-star-frontend",
    project_id: "proj-star",
    branch: "feat/projects-tabs",
    agent_id: "ag-003",
    status: "ci_running",
    created_at: "2026-08-15T14:00:00Z",
    last_event_at: "2026-08-30T19:08:00Z",
  },
  {
    id: "wt-docs-adr-v0-22",
    project_id: "proj-docs",
    branch: "docs/adr-0026",
    agent_id: "ag-004",
    status: "review_requested",
    created_at: "2026-08-20T08:30:00Z",
    last_event_at: "2026-08-30T11:00:00Z",
  },
  {
    id: "wt-experimental-mock",
    project_id: "proj-star",
    branch: "feat/test-mock-5d",
    agent_id: "ag-005",
    status: "active",
    created_at: "2026-08-29T20:00:00Z",
    last_event_at: "2026-08-31T12:30:00Z",
  },
  {
    id: "wt-physis-old-step",
    project_id: "proj-physis",
    branch: "fix/legacy-rts-collision",
    agent_id: null,
    status: "merged",
    created_at: "2026-07-04T10:00:00Z",
    last_event_at: "2026-08-01T16:42:00Z",
  },
  {
    id: "wt-star-abandoned",
    project_id: "proj-star",
    branch: "feat/abandoned-experiment",
    agent_id: null,
    status: "abandoned",
    created_at: "2026-06-18T09:00:00Z",
    last_event_at: "2026-07-02T18:00:00Z",
  },
];

/** 7 个常用状态用于 transition mock (e2e 验证) */
export const SUPPORTED_TRANSITION_STATES: ReadonlyArray<WorktreeStatus> = [
  "initializing",
  "cloning",
  "active",
  "dirty",
  "committing",
  "ci_running",
  "merged",
];

// =====================================================================
// 4. social 域 — MOCK_COMMENTS (7 rows, 跨 2 个 work_item_id)
// =====================================================================

// 用 mulberry32(2) 选 (避免 hard-coded 看起来假) — 7 行跨 wi-001/wi-002
const commentSeeds: ReadonlyArray<Omit<Comment, "id" | "created_at" | "deleted">> = [
  { work_item_id: "wi-001", author_id: "ag-001", body: "Spec excerpt conflicts with INV-RT-03 — please confirm direction." },
  { work_item_id: "wi-001", author_id: "ag-002", body: "Test:integration:rt_step failing 3/12 — investigating. Will push fix in 30m." },
  { work_item_id: "wi-001", author_id: "ag-004", body: "ADR-0026 cross-ref sync done (per 2026-08-29 RGS-CROSS-REF-SYNC-REPORT)." },
  { work_item_id: "wi-002", author_id: "ag-003", body: "5 tab 实装完成 (Kanban / Timeline / Backlog / Agents / Worktrees) per 7d85c34." },
  { work_item_id: "wi-002", author_id: "ag-005", body: "CI typecheck green; waiting for review." },
  { work_item_id: "wi-002", author_id: "ag-001", body: "Confirmed scope — merge approved." },
  { work_item_id: "wi-001", author_id: "ag-002", body: "Fix pushed (commit sha visible in worktree log). Re-run CI." },
];

function buildComments(): ReadonlyArray<Comment> {
  const rand = mulberry32(2);
  return commentSeeds.map((seed, idx) => {
    // mock 简化: created_at 在 2026-08-25 ~ 2026-08-31 之间均匀分布
    const dayOffset = 1 + Math.floor(rand() * 7); // 1..7
    const hour = Math.floor(rand() * 24);
    const created = new Date(Date.UTC(2026, 7, 25 - dayOffset + 7, hour, 0, 0));
    return {
      id: `cm-${String(idx + 1).padStart(3, "0")}`,
      ...seed,
      created_at: created.toISOString(),
      deleted: false,
    };
  });
}

export const MOCK_COMMENTS: ReadonlyArray<Comment> = buildComments();

// =====================================================================
// 5. admin 域 — MOCK_TENANTS (3 rows, 含 plan = starter/pro/enterprise) +
//               MOCK_RBAC_ROLES (4 rows, 覆盖 admin/member/viewer)
// =====================================================================

export const MOCK_TENANTS: ReadonlyArray<Tenant> = [
  {
    id: "t-acme",
    name: "ACME Studio",
    plan: "enterprise",
    created_at: "2026-01-15T08:00:00Z",
    active: true,
  },
  {
    id: "t-beta",
    name: "Beta Labs",
    plan: "pro",
    created_at: "2026-03-22T10:30:00Z",
    active: true,
  },
  {
    id: "t-gamma",
    name: "Gamma Hobby",
    plan: "starter",
    created_at: "2026-06-10T14:00:00Z",
    active: false,
  },
];

export const MOCK_RBAC_ROLES: ReadonlyArray<RbacRole> = [
  {
    id: "role-acme-admin",
    tenant_id: "t-acme",
    name: "admin",
    permissions: [
      "workspace:read",
      "workspace:write",
      "billing:read",
      "billing:write",
      "worktree:read",
      "worktree:write",
      "worktree:transition",
      "comment:read",
      "comment:write",
      "comment:delete",
      "tenant:read",
      "tenant:write",
      "rbac:read",
      "rbac:write",
    ],
    created_at: "2026-01-15T08:00:00Z",
  },
  {
    id: "role-acme-member",
    tenant_id: "t-acme",
    name: "member",
    permissions: [
      "workspace:read",
      "worktree:read",
      "worktree:write",
      "comment:read",
      "comment:write",
    ],
    created_at: "2026-01-15T08:00:00Z",
  },
  {
    id: "role-acme-viewer",
    tenant_id: "t-acme",
    name: "viewer",
    permissions: ["workspace:read", "worktree:read", "comment:read", "billing:read"],
    created_at: "2026-01-15T08:00:00Z",
  },
  {
    id: "role-beta-admin",
    tenant_id: "t-beta",
    name: "admin",
    permissions: [
      "workspace:read",
      "workspace:write",
      "billing:read",
      "billing:write",
      "worktree:read",
      "worktree:write",
      "comment:read",
      "comment:write",
    ],
    created_at: "2026-03-22T10:30:00Z",
  },
];
