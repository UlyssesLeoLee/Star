// =====================================================================
// Mock data store — 25 domain 全量 seed
// =====================================================================
// 数据规模:每域 8-30 条;够真实感且不爆炸
// 所有 ID 形如 "<prefix>-<n>" 便于人读
// =====================================================================

import type {
  Tenant, Project, Identity, Workspace, WorkItem, Comment, PermissionScheme,
  PermissionRule, Workflow, ChangeSet, Worktree, AgentSession, Feedback,
  ContextPacket, ContextDecision, ValidationCase, LocalRuntime, Repository,
  PullRequest, Notification, SearchHit, SavedSearch, Integration, PresenceCursor,
  Whiteboard, Canvas, CanvasElement, CanvasConnector, Sprint, Milestone,
  BurndownPoint, Board, Relation, AuditEvent, AutomationRule,
} from "@/types/ids";

const now = () => new Date().toISOString();
const ago = (mins: number) => new Date(Date.now() - mins * 60_000).toISOString();

// ---------- 基础标识 ----------
const TENANT_ID = "ten-acme";
const PROJECT_ID = "prj-physis";

// =====================================================================
// tenants (1)
// =====================================================================
export const tenants: Tenant[] = [
  { id: TENANT_ID, name: "ACME Studio", slug: "acme", plan: "enterprise", status: "active", created_at: ago(60 * 24 * 90), seat_limit: 200 },
];

// =====================================================================
// projects (3)
// =====================================================================
export const projects: Project[] = [
  { id: PROJECT_ID, tenant_id: TENANT_ID, key: "PHYSIS", name: "Physis / GVPE", visibility: "private", owner_id: "usr-arch", member_count: 14, created_at: ago(60 * 24 * 120) },
  { id: "prj-stargate", tenant_id: TENANT_ID, key: "SG", name: "StarGate Dashboard", visibility: "internal", owner_id: "usr-pm", member_count: 7, created_at: ago(60 * 24 * 60) },
  { id: "prj-mobile", tenant_id: TENANT_ID, key: "MOB", name: "Mobile Companion", visibility: "private", owner_id: "usr-pm", member_count: 5, created_at: ago(60 * 24 * 30) },
];

// =====================================================================
// identities (10)
// =====================================================================
const ident = (i: number, role: Identity["provider"], status: Identity["status"], mfa: boolean): Identity => ({
  id: `usr-${i.toString().padStart(3, "0")}`,
  tenant_id: TENANT_ID,
  email: `user${i}@acme.studio`,
  display_name: ["Ulysses", "Hera", "Athena", "Ares", "Hermes", "Apollo", "Artemis", "Hephaestus", "Demeter", "Dionysus"][i - 1] ?? `User ${i}`,
  provider: role,
  status,
  mfa_enabled: mfa,
  last_login_at: ago(60 * (24 - i)),
});
export const identities: Identity[] = [
  ident(1, "github", "active", true),
  ident(2, "google", "active", true),
  ident(3, "saml-sso", "active", true),
  ident(4, "password", "active", false),
  ident(5, "github", "active", true),
  ident(6, "local-runtime-device", "active", true),
  ident(7, "password", "invited", false),
  ident(8, "github", "active", true),
  ident(9, "saml-sso", "disabled", true),
  ident(10, "google", "active", true),
];

// =====================================================================
// workspaces (6)
// =====================================================================
export const workspaces: Workspace[] = [
  { id: "ws-001", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Physis Engine", kind: "shared", member_ids: ["usr-001","usr-002","usr-003","usr-004","usr-005"], default_branch_policy: "fast-forward-only" },
  { id: "ws-002", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "GVPE C ABI",         kind: "shared", member_ids: ["usr-001","usr-006"],          default_branch_policy: "allow-non-ff" },
  { id: "ws-003", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Architecture Docs",  kind: "scratch", member_ids: ["usr-001","usr-002","usr-003"], default_branch_policy: "allow-non-ff" },
  { id: "ws-004", tenant_id: TENANT_ID, project_id: "prj-stargate", name: "Dashboard Frontend", kind: "shared", member_ids: ["usr-002","usr-005","usr-007"], default_branch_policy: "fast-forward-only" },
  { id: "ws-005", tenant_id: TENANT_ID, project_id: "prj-stargate", name: "Realtime Gateway", kind: "shared", member_ids: ["usr-006","usr-008"], default_branch_policy: "fast-forward-only" },
  { id: "ws-006", tenant_id: TENANT_ID, project_id: "prj-mobile", name: "iOS / Android", kind: "archived", member_ids: ["usr-009","usr-010"], default_branch_policy: "fast-forward-only" },
];

// =====================================================================
// work_items (30)
// =====================================================================
const workItem = (i: number, status: WorkItem["status"], kind: WorkItem["kind"], priority: WorkItem["priority"], title: string): WorkItem => ({
  id: `wi-${i.toString().padStart(3, "0")}`,
  tenant_id: TENANT_ID,
  project_id: i < 20 ? PROJECT_ID : (i < 25 ? "prj-stargate" : "prj-mobile"),
  key: i < 20 ? `PHYSIS-${i}` : (i < 25 ? `SG-${i - 19}` : `MOB-${i - 24}`),
  title,
  description: `Auto-generated seed for ${title}`,
  kind,
  status,
  priority,
  assignee_id: `usr-${((i % 10) + 1).toString().padStart(3, "0")}`,
  reporter_id: "usr-001",
  story_points: [1, 2, 3, 5, 8, 13][i % 6],
  labels: [["backend"], ["backend", "perf"], ["frontend"], ["infra"], ["ai"], ["spec"]][i % 6],
  sprint_id: i % 3 === 0 ? "spr-001" : (i % 3 === 1 ? "spr-002" : undefined),
  workflow_id: "wf-default",
  // W3 Calendar: 给每个 work-item 派 due_date, 散布在 [now-30d, now+30d] 区间内
  // 用 i*2 - 30 天偏移 (i 偶正奇负交错) 让 calendar 有数据可显示
  // per dynamic-interaction-design.md §5.2 月视图 "每格显示 due work-item 数"
  due_date: ago(-60 * 24 * (i * 2 - 30)),
  created_at: ago(60 * 24 * (30 - i)),
  updated_at: ago(60 * (i * 2)),
});
export const workItems: WorkItem[] = [
  workItem(1, "in_progress", "story", "p0", "Implement Worktree 17-state machine"),
  workItem(2, "review", "story", "p0", "AgentSession 14 状态机 + 12 强制迁移"),
  workItem(3, "done", "task", "p1", "Webhook Idempotency-Key for SCM integration"),
  workItem(4, "todo", "story", "p0", "Cross-tenant audit (INV-AU-04)"),
  workItem(5, "in_progress", "bug", "p1", "Feedback 状态机 reopen 不触发 PR"),
  workItem(6, "blocked", "task", "p2", "ContextPacket priority p0 缓存命中率"),
  workItem(7, "in_progress", "story", "p0", "Automation Rule executor (Rule + Trigger + Condition + Action)"),
  workItem(8, "review", "task", "p1", "PermissionScheme rules-based RBAC"),
  workItem(9, "done", "story", "p0", "ChangeSet 5 状态机 + INV-DEV-01~05"),
  workItem(10, "in_progress", "spike", "p3", "Local Runtime 三重绑定 device/tenant/user"),
  workItem(11, "todo", "story", "p1", "Search Projection tenant 隔离 (INV-SR-02)"),
  workItem(12, "review", "task", "p1", "Notification INV-N-07 抑制策略"),
  workItem(13, "done", "task", "p2", "Validation 7 实体 + 5 状态机"),
  workItem(14, "in_progress", "story", "p0", "Relation graph BFS 性能"),
  workItem(15, "todo", "bug", "p1", "Worktree sync loop on offline reconnect"),
  workItem(16, "done", "task", "p2", "Workspace member role change audit"),
  workItem(17, "in_progress", "story", "p0", "Planning Sprint burndown chart"),
  workItem(18, "review", "story", "p0", "Board Kanban WIP limit"),
  workItem(19, "wontfix", "task", "p3", "Mobile push notification 噪音抑制"),
  workItem(20, "done", "task", "p2", "Audit append-only hash chain"),
  workItem(21, "in_progress", "story", "p1", "Dashboard realtime work-item event"),
  workItem(22, "review", "task", "p1", "Realtime cursor presence"),
  workItem(23, "todo", "task", "p2", "Whiteboard snapshot export"),
  workItem(24, "in_progress", "bug", "p1", "Linear integration loop protection"),
  workItem(25, "todo", "story", "p1", "iOS companion: worktree quick switch"),
  workItem(26, "in_progress", "task", "p2", "Android: agent session telemetry"),
  workItem(27, "review", "task", "p2", "Mobile: offline feedback queue"),
  workItem(28, "done", "task", "p3", "Mobile: deep link to work-item"),
  workItem(29, "todo", "spike", "p3", "Mobile: secure enclave for local-runtime token"),
  workItem(30, "in_progress", "task", "p1", "Mobile: feedback notification grouping"),
];

// =====================================================================
// comments
// =====================================================================
export const comments: Comment[] = [
  { id: "cm-001", tenant_id: TENANT_ID, target_kind: "work_item", target_id: "wi-001", author_id: "usr-001", body: "Worktree SM 已经过 §7.1 状态机评审, 17 个状态 + 4 个核心迁移。", mentions: ["usr-002"], created_at: ago(60 * 8) },
  { id: "cm-002", tenant_id: TENANT_ID, target_kind: "work_item", target_id: "wi-001", author_id: "usr-002", body: "已合并 wt-1 → main. 14 状态机 + INV-WT-01~04 全绿。", mentions: [], created_at: ago(60 * 4) },
  { id: "cm-003", tenant_id: TENANT_ID, target_kind: "pr", target_id: "pr-001", author_id: "usr-003", body: "ci.yml 需要把 trunk-based feature flag 打开。", mentions: ["usr-004"], created_at: ago(60 * 2) },
  { id: "cm-004", tenant_id: TENANT_ID, target_kind: "agent_session", target_id: "ag-001", author_id: "usr-006", body: "Agent 进了 awaiting_human, 需要 decision 节点确认是否回滚 INV-AU-04 hotfix。", mentions: ["usr-001"], created_at: ago(30) },
  { id: "cm-005", tenant_id: TENANT_ID, target_kind: "work_item", target_id: "wi-005", author_id: "usr-005", body: "Feedback reopen 状态机漏了 → 1.1 版本修复中", mentions: [], created_at: ago(15) },
  { id: "cm-006", tenant_id: TENANT_ID, target_kind: "context_packet", target_id: "ctx-001", author_id: "usr-002", body: "ContextPacket priority p0 cache miss 3 次, 走 context-engine 路径", mentions: [], created_at: ago(10) },
];

// =====================================================================
// permission (PermissionScheme + 18 rules)
// =====================================================================
export const permissionSchemes: PermissionScheme[] = [
  { id: "ps-default", tenant_id: TENANT_ID, name: "Default (Team)", is_default: true, rule_count: 6 },
  { id: "ps-strict",  tenant_id: TENANT_ID, name: "Strict (Admin only)", is_default: false, rule_count: 8 },
  { id: "ps-ai",      tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "AI Auto-Approve", is_default: false, rule_count: 4 },
];

export const permissionRules: PermissionRule[] = [
  { id: "pr-001", tenant_id: TENANT_ID, scheme_id: "ps-default", resource_kind: "project", action: "admin", role: "tenant_admin", effect: "allow" },
  { id: "pr-002", tenant_id: TENANT_ID, scheme_id: "ps-default", resource_kind: "project", action: "write", role: "project_admin", effect: "allow" },
  { id: "pr-003", tenant_id: TENANT_ID, scheme_id: "ps-default", resource_kind: "work_item", action: "read", role: "viewer", effect: "allow" },
  { id: "pr-004", tenant_id: TENANT_ID, scheme_id: "ps-default", resource_kind: "work_item", action: "write", role: "developer", effect: "allow" },
  { id: "pr-005", tenant_id: TENANT_ID, scheme_id: "ps-default", resource_kind: "worktree", action: "admin", role: "developer", effect: "allow" },
  { id: "pr-006", tenant_id: TENANT_ID, scheme_id: "ps-default", resource_kind: "agent_session", action: "write", role: "developer", effect: "allow", condition: "actor.id == resource.author_id" },
  { id: "pr-007", tenant_id: TENANT_ID, scheme_id: "ps-strict", resource_kind: "automation_rule", action: "admin", role: "tenant_admin", effect: "allow" },
  { id: "pr-008", tenant_id: TENANT_ID, scheme_id: "ps-strict", resource_kind: "automation_rule", action: "write", role: "project_admin", effect: "deny", condition: "resource.kind == 'call_webhook' && resource.config.url.host not in allowlist" },
  { id: "pr-009", tenant_id: TENANT_ID, scheme_id: "ps-ai", resource_kind: "agent_session", action: "write", role: "developer", effect: "allow", condition: "resource.budget_usd <= 1.0" },
];

// =====================================================================
// workflow (3 + states/transitions)
// =====================================================================
export const workflows: Workflow[] = [
  {
    id: "wf-default", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Default 3-State", is_default: true,
    states: [
      { id: "st-1", workflow_id: "wf-default", name: "To Do",       kind: "initial",     category: "todo",        position: 0 },
      { id: "st-2", workflow_id: "wf-default", name: "In Progress", kind: "intermediate", category: "in_progress", position: 1 },
      { id: "st-3", workflow_id: "wf-default", name: "Done",        kind: "final",       category: "done",        position: 2 },
    ],
    transitions: [
      { from_state_id: "st-1", to_state_id: "st-2", trigger: "user.start" },
      { from_state_id: "st-2", to_state_id: "st-3", trigger: "user.done" },
      { from_state_id: "st-1", to_state_id: "st-3", trigger: "user.wontfix" },
    ],
  },
  {
    id: "wf-extended", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Extended 6-State", is_default: false,
    states: [
      { id: "st-1", workflow_id: "wf-extended", name: "To Do",       kind: "initial",     category: "todo",        position: 0 },
      { id: "st-2", workflow_id: "wf-extended", name: "In Progress", kind: "intermediate", category: "in_progress", position: 1 },
      { id: "st-3", workflow_id: "wf-extended", name: "Review",      kind: "intermediate", category: "review",      position: 2 },
      { id: "st-4", workflow_id: "wf-extended", name: "Blocked",     kind: "intermediate", category: "blocked",     position: 3 },
      { id: "st-5", workflow_id: "wf-extended", name: "Done",        kind: "final",       category: "done",        position: 4 },
      { id: "st-6", workflow_id: "wf-extended", name: "Won't Fix",   kind: "final",       category: "wontfix",     position: 5 },
    ],
    transitions: [
      { from_state_id: "st-1", to_state_id: "st-2", trigger: "user.start" },
      { from_state_id: "st-2", to_state_id: "st-3", trigger: "pr.opened" },
      { from_state_id: "st-2", to_state_id: "st-4", trigger: "blocker.detected" },
      { from_state_id: "st-4", to_state_id: "st-2", trigger: "blocker.cleared" },
      { from_state_id: "st-3", to_state_id: "st-5", trigger: "pr.merged" },
      { from_state_id: "st-3", to_state_id: "st-2", trigger: "review.changes_requested" },
      { from_state_id: "st-2", to_state_id: "st-6", trigger: "user.wontfix" },
    ],
  },
  {
    id: "wf-stargate", tenant_id: TENANT_ID, project_id: "prj-stargate", name: "Stargate Default", is_default: true,
    states: [
      { id: "st-1", workflow_id: "wf-stargate", name: "Backlog",   kind: "initial",     category: "todo",        position: 0 },
      { id: "st-2", workflow_id: "wf-stargate", name: "Building",  kind: "intermediate", category: "in_progress", position: 1 },
      { id: "st-3", workflow_id: "wf-stargate", name: "Shipped",   kind: "final",       category: "done",        position: 2 },
    ],
    transitions: [
      { from_state_id: "st-1", to_state_id: "st-2", trigger: "user.start" },
      { from_state_id: "st-2", to_state_id: "st-3", trigger: "deploy.prod" },
    ],
  },
];

// =====================================================================
// change_sets (12)
// =====================================================================
export const changeSets: ChangeSet[] = [
  { id: "cs-001", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-001", author_id: "usr-002", worktree_id: "wt-001", title: "Worktree SM types",   diff_summary: "+342 / -18 / 4 files", status: "merged",    symbol_index: { added: 18, modified: 4, removed: 1 }, created_at: ago(60 * 30) },
  { id: "cs-002", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-002", author_id: "usr-002", worktree_id: "wt-002", title: "Agent 14 state SM",     diff_summary: "+612 / -89 / 7 files", status: "merged",    symbol_index: { added: 22, modified: 7, removed: 3 }, created_at: ago(60 * 24) },
  { id: "cs-003", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-007", author_id: "usr-003", worktree_id: "wt-003", title: "Automation executor",    diff_summary: "+880 / -23 / 11 files", status: "applied",  symbol_index: { added: 41, modified: 11, removed: 2 }, created_at: ago(60 * 12) },
  { id: "cs-004", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-009", author_id: "usr-002", worktree_id: "wt-004", title: "ChangeSet 5 SM",         diff_summary: "+422 / -8 / 5 files",   status: "merged",   symbol_index: { added: 19, modified: 5, removed: 0 }, created_at: ago(60 * 36) },
  { id: "cs-005", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-014", author_id: "usr-004", worktree_id: "wt-005", title: "Relation BFS v0",         diff_summary: "+210 / -12 / 3 files",  status: "draft",    symbol_index: { added: 12, modified: 3, removed: 0 }, created_at: ago(60 * 4) },
  { id: "cs-006", tenant_id: TENANT_ID, project_id: "prj-stargate", work_item_id: "wi-021", author_id: "usr-005", worktree_id: "wt-006", title: "Realtime work-item event", diff_summary: "+156 / -3 / 2 files",   status: "applied",  symbol_index: { added: 7,  modified: 2, removed: 0 }, created_at: ago(60 * 8) },
  { id: "cs-007", tenant_id: TENANT_ID, project_id: "prj-stargate", work_item_id: "wi-022", author_id: "usr-005", worktree_id: "wt-007", title: "Presence cursor",         diff_summary: "+98 / -0 / 1 files",    status: "draft",    symbol_index: { added: 4,  modified: 1, removed: 0 }, created_at: ago(60 * 2) },
  { id: "cs-008", tenant_id: TENANT_ID, project_id: "prj-mobile", work_item_id: "wi-025", author_id: "usr-009", worktree_id: "wt-008", title: "iOS worktree switch",     diff_summary: "+234 / -8 / 4 files",   status: "draft",    symbol_index: { added: 9,  modified: 4, removed: 0 }, created_at: ago(60 * 6) },
  { id: "cs-009", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-005", author_id: "usr-005", worktree_id: "wt-009", title: "Feedback reopen fix",     diff_summary: "+45 / -12 / 2 files",   status: "abandoned", symbol_index: { added: 2, modified: 2, removed: 0 }, created_at: ago(60 * 20) },
  { id: "cs-010", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-006", author_id: "usr-002", worktree_id: "wt-010", title: "ContextPacket p0 cache",  diff_summary: "+310 / -2 / 3 files",   status: "reverted",  symbol_index: { added: 11, modified: 3, removed: 0 }, created_at: ago(60 * 16) },
  { id: "cs-011", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-013", author_id: "usr-003", worktree_id: "wt-011", title: "Validation 7 entities",   diff_summary: "+512 / -28 / 6 files",  status: "merged",   symbol_index: { added: 23, modified: 6, removed: 1 }, created_at: ago(60 * 22) },
  { id: "cs-012", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-008", author_id: "usr-003", worktree_id: "wt-012", title: "Permission rules RBAC",   diff_summary: "+410 / -5 / 8 files",   status: "applied",  symbol_index: { added: 17, modified: 8, removed: 0 }, created_at: ago(60 * 14) },
];

// =====================================================================
// worktrees (12)
// =====================================================================
export const worktrees: Worktree[] = [
  { id: "wt-001", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "wt-worktree-sm",  branch: "feat/worktree-sm",     base_branch: "main", status: "merged",          local_runtime_id: "lr-001", agent_session_id: "ag-001", pr_id: "pr-001", lock_version: 3, last_event_at: ago(60 * 30), created_at: ago(60 * 36) },
  { id: "wt-002", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "wt-agent-14sm",   branch: "feat/agent-14-sm",     base_branch: "main", status: "merged",          local_runtime_id: "lr-001", agent_session_id: "ag-002", pr_id: "pr-002", lock_version: 5, last_event_at: ago(60 * 24), created_at: ago(60 * 30) },
  { id: "wt-003", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "wt-automation",   branch: "feat/automation-v2",   base_branch: "main", status: "review_requested",local_runtime_id: "lr-002", agent_session_id: "ag-003", pr_id: "pr-003", lock_version: 7, last_event_at: ago(60 * 1),  created_at: ago(60 * 14) },
  { id: "wt-004", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "wt-changeset-5sm",branch: "feat/changeset-5sm",  base_branch: "main", status: "merged",          local_runtime_id: "lr-001", agent_session_id: "ag-004", pr_id: "pr-004", lock_version: 4, last_event_at: ago(60 * 35), created_at: ago(60 * 40) },
  { id: "wt-005", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "wt-relation-bfs", branch: "feat/relation-bfs",    base_branch: "main", status: "active",          local_runtime_id: "lr-002", agent_session_id: "ag-005",                      lock_version: 1, last_event_at: ago(60 * 0.5), created_at: ago(60 * 4) },
  { id: "wt-006", tenant_id: TENANT_ID, project_id: "prj-stargate", name: "wt-realtime-wi",  branch: "feat/realtime-workitem", base_branch: "main", status: "ci_running",    local_runtime_id: "lr-003", agent_session_id: "ag-006", pr_id: "pr-006", lock_version: 2, last_event_at: ago(60 * 0.2), created_at: ago(60 * 8) },
  { id: "wt-007", tenant_id: TENANT_ID, project_id: "prj-stargate", name: "wt-presence",     branch: "feat/presence-cursor", base_branch: "main", status: "dirty",          local_runtime_id: "lr-003", agent_session_id: "ag-007",                       lock_version: 0, last_event_at: ago(60 * 0.1), created_at: ago(60 * 2) },
  { id: "wt-008", tenant_id: TENANT_ID, project_id: "prj-mobile", name: "wt-ios-switch",    branch: "feat/ios-worktree-switch", base_branch: "main", status: "active",    local_runtime_id: "lr-004", agent_session_id: "ag-008",                       lock_version: 1, last_event_at: ago(60 * 0.4), created_at: ago(60 * 6) },
  { id: "wt-009", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "wt-feedback-fb",   branch: "fix/feedback-reopen",  base_branch: "main", status: "abandoned",       local_runtime_id: "lr-002", agent_session_id: "ag-009", pr_id: "pr-009", lock_version: 0, last_event_at: ago(60 * 20), created_at: ago(60 * 24) },
  { id: "wt-010", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "wt-context-cache", branch: "perf/context-p0",      base_branch: "main", status: "reverted",        local_runtime_id: "lr-002", agent_session_id: "ag-010", pr_id: "pr-010", lock_version: 2, last_event_at: ago(60 * 16), created_at: ago(60 * 18) },
  { id: "wt-011", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "wt-validation-7",  branch: "feat/validation-7",    base_branch: "main", status: "merged",          local_runtime_id: "lr-001", agent_session_id: "ag-011", pr_id: "pr-011", lock_version: 3, last_event_at: ago(60 * 22), created_at: ago(60 * 26) },
  { id: "wt-012", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "wt-permission-rb", branch: "feat/perm-rules",      base_branch: "main", status: "review_requested",local_runtime_id: "lr-002", agent_session_id: "ag-012", pr_id: "pr-012", lock_version: 4, last_event_at: ago(60 * 0.3), created_at: ago(60 * 14) },
];

// =====================================================================
// agent_sessions (12)
// =====================================================================
export const agentSessions: AgentSession[] = [
  { id: "ag-001", tenant_id: TENANT_ID, project_id: PROJECT_ID, worktree_id: "wt-001", agent_kind: "claude-sonnet", status: "completed",          current_step: "validation.pass",       token_usage: { input: 124_000, output: 38_000, total: 162_000 }, cost_summary: { usd: 0.92,  budget_usd: 5.0 }, started_at: ago(60 * 35), ended_at: ago(60 * 30) },
  { id: "ag-002", tenant_id: TENANT_ID, project_id: PROJECT_ID, worktree_id: "wt-002", agent_kind: "codex",          status: "completed",          current_step: "validation.pass",       token_usage: { input: 168_000, output: 51_000, total: 219_000 }, cost_summary: { usd: 1.18,  budget_usd: 5.0 }, started_at: ago(60 * 30), ended_at: ago(60 * 24) },
  { id: "ag-003", tenant_id: TENANT_ID, project_id: PROJECT_ID, worktree_id: "wt-003", agent_kind: "claude-sonnet", status: "awaiting_human",     current_step: "human.decision.required", token_usage: { input: 245_000, output: 72_000, total: 317_000 }, cost_summary: { usd: 1.81,  budget_usd: 3.0 }, started_at: ago(60 * 5) },
  { id: "ag-004", tenant_id: TENANT_ID, project_id: PROJECT_ID, worktree_id: "wt-004", agent_kind: "codex",          status: "completed",          current_step: "validation.pass",       token_usage: { input: 132_000, output: 41_000, total: 173_000 }, cost_summary: { usd: 0.97,  budget_usd: 5.0 }, started_at: ago(60 * 38), ended_at: ago(60 * 35) },
  { id: "ag-005", tenant_id: TENANT_ID, project_id: PROJECT_ID, worktree_id: "wt-005", agent_kind: "claude-sonnet", status: "executing",          current_step: "tool.call:grep",        token_usage: { input: 89_000,  output: 22_000, total: 111_000 }, cost_summary: { usd: 0.62,  budget_usd: 5.0 }, started_at: ago(60 * 0.6) },
  { id: "ag-006", tenant_id: TENANT_ID, project_id: "prj-stargate", worktree_id: "wt-006", agent_kind: "codex",      status: "validating",         current_step: "test:e2e:realtime",     token_usage: { input: 67_000,  output: 18_000, total: 85_000 },  cost_summary: { usd: 0.48,  budget_usd: 2.0 }, started_at: ago(60 * 0.4) },
  { id: "ag-007", tenant_id: TENANT_ID, project_id: "prj-stargate", worktree_id: "wt-007", agent_kind: "claude-sonnet", status: "compiling_context", current_step: "context.fetch:p2",      token_usage: { input: 23_000,  output: 5_000,  total: 28_000 },  cost_summary: { usd: 0.16,  budget_usd: 2.0 }, started_at: ago(60 * 0.1) },
  { id: "ag-008", tenant_id: TENANT_ID, project_id: "prj-mobile", worktree_id: "wt-008", agent_kind: "codex",          status: "planning",            current_step: "plan.step:3/8",         token_usage: { input: 41_000,  output: 9_000,  total: 50_000 },  cost_summary: { usd: 0.28,  budget_usd: 2.0 }, started_at: ago(60 * 0.3) },
  { id: "ag-009", tenant_id: TENANT_ID, project_id: PROJECT_ID, worktree_id: "wt-009", agent_kind: "claude-sonnet", status: "failed",              current_step: "validation.fail:INV-FB-02", token_usage: { input: 88_000, output: 12_000, total: 100_000 }, cost_summary: { usd: 0.51, budget_usd: 3.0 }, started_at: ago(60 * 22), ended_at: ago(60 * 20) },
  { id: "ag-010", tenant_id: TENANT_ID, project_id: PROJECT_ID, worktree_id: "wt-010", agent_kind: "codex",          status: "completed",          current_step: "revert.applied",        token_usage: { input: 73_000,  output: 19_000, total: 92_000 },  cost_summary: { usd: 0.49,  budget_usd: 2.0 }, started_at: ago(60 * 18), ended_at: ago(60 * 16) },
  { id: "ag-011", tenant_id: TENANT_ID, project_id: PROJECT_ID, worktree_id: "wt-011", agent_kind: "claude-sonnet", status: "completed",          current_step: "validation.pass",       token_usage: { input: 156_000, output: 44_000, total: 200_000 }, cost_summary: { usd: 1.05,  budget_usd: 5.0 }, started_at: ago(60 * 25), ended_at: ago(60 * 22) },
  { id: "ag-012", tenant_id: TENANT_ID, project_id: PROJECT_ID, worktree_id: "wt-012", agent_kind: "codex",          status: "awaiting_feedback",  current_step: "feedback.request",      token_usage: { input: 112_000, output: 33_000, total: 145_000 }, cost_summary: { usd: 0.79,  budget_usd: 5.0 }, started_at: ago(60 * 0.5) },
];

// =====================================================================
// feedbacks
// =====================================================================
export const feedbacks: Feedback[] = [
  { id: "fb-001", tenant_id: TENANT_ID, agent_session_id: "ag-003", worktree_id: "wt-003", status: "in_progress", severity: "major",     category: "spec_clarification", question: "AutomationRule 的 CEL condition 在 cron 触发下是否启用相同的 actor context?", asked_by: "ag-003", asked_at: ago(60 * 3) },
  { id: "fb-002", tenant_id: TENANT_ID, agent_session_id: "ag-012", worktree_id: "wt-012", status: "open",        severity: "minor",     category: "implementation_bug", question: "PermissionRule 的 condition CEL 解析失败时, 是否 fallback 到 deny?", asked_by: "ag-012", asked_at: ago(60 * 0.4) },
  { id: "fb-003", tenant_id: TENANT_ID, agent_session_id: "ag-006", worktree_id: "wt-006", status: "acknowledged",severity: "info",      category: "test_failure",        question: "E2E test: realtime event 在 200ms 内是否需要 ack 确认?", asked_by: "ag-006", answered_by: "usr-001", asked_at: ago(60 * 2), answered_at: ago(60 * 1.5) },
  { id: "fb-004", tenant_id: TENANT_ID, agent_session_id: "ag-005", worktree_id: "wt-005", status: "resolved",    severity: "minor",     category: "ux_issue",            question: "Relation graph 的 BFS 深度上限建议?", answer: "建议 5-7 层, 防止跨度过大。", asked_by: "ag-005", answered_by: "usr-002", asked_at: ago(60 * 5), answered_at: ago(60 * 4) },
  { id: "fb-005", tenant_id: TENANT_ID, agent_session_id: "ag-009", worktree_id: "wt-009", status: "wontfix",     severity: "critical",  category: "policy_violation",    question: "Feedback reopen 状态机是否需要单独审批流?", answer: "v1 不实现, 走通用工作流审批。", asked_by: "ag-009", answered_by: "usr-001", asked_at: ago(60 * 22), answered_at: ago(60 * 21) },
  { id: "fb-006", tenant_id: TENANT_ID, agent_session_id: "ag-008", worktree_id: "wt-008", status: "open",        severity: "info",      category: "spec_clarification", question: "iOS worktree switch 是否在 offline 模式下可用?", asked_by: "ag-008", asked_at: ago(60 * 0.2) },
];

// =====================================================================
// context packets + decisions
// =====================================================================
export const contextPackets: ContextPacket[] = [
  { id: "ctx-001", tenant_id: TENANT_ID, agent_session_id: "ag-005", priority: "p0", kind: "spec",  payload_ref: "s3://star/ctx/spec-worktree.md",    token_estimate: 3_200, provenance: "spec_excerpt",         created_at: ago(60 * 0.5) },
  { id: "ctx-002", tenant_id: TENANT_ID, agent_session_id: "ag-005", priority: "p0", kind: "code",  payload_ref: "s3://star/ctx/code-relation.rs",    token_estimate: 8_400, provenance: "tool_output",          created_at: ago(60 * 0.4) },
  { id: "ctx-003", tenant_id: TENANT_ID, agent_session_id: "ag-005", priority: "p1", kind: "history", payload_ref: "s3://star/ctx/hist-wi014.json",   token_estimate: 1_800, provenance: "previous_decision",    decision_id: "dec-002", created_at: ago(60 * 0.3) },
  { id: "ctx-004", tenant_id: TENANT_ID, agent_session_id: "ag-006", priority: "p1", kind: "tool",  payload_ref: "s3://star/ctx/tool-realtime.ts",    token_estimate: 2_100, provenance: "tool_output",          created_at: ago(60 * 0.2) },
  { id: "ctx-005", tenant_id: TENANT_ID, agent_session_id: "ag-008", priority: "p2", kind: "decision", payload_ref: "s3://star/ctx/dec-ios-001.json", token_estimate: 600,   provenance: "agent_inference",      decision_id: "dec-003", created_at: ago(60 * 0.1) },
];
export const contextDecisions: ContextDecision[] = [
  { id: "dec-001", agent_session_id: "ag-001", status: "approved", prompt: "Worktree SM 初始 state 用 initializing vs active?", chosen_option: "initializing (3-stage bootstrap)", decided_by: "usr-001", decided_at: ago(60 * 30) },
  { id: "dec-002", agent_session_id: "ag-005", status: "approved", prompt: "Relation BFS 上限 5 vs 7 层?", chosen_option: "5 (有 2 层 prefetch buffer)", decided_by: "usr-002", decided_at: ago(60 * 0.3) },
  { id: "dec-003", agent_session_id: "ag-008", status: "pending",  prompt: "iOS worktree switch 是否需要 device biometric?" },
  { id: "dec-004", agent_session_id: "ag-003", status: "rejected", prompt: "Automation rule 在 cron 触发下, 跨 project 边界是否允许?", decided_by: "usr-001", decided_at: ago(60 * 1) },
];

// =====================================================================
// validation (10)
// =====================================================================
export const validationCases: ValidationCase[] = [
  { id: "vc-001", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-001", changeset_id: "cs-001", name: "Worktree SM unit",            kind: "unit",        result: "pass",              coverage: 0.96, executed_at: ago(60 * 30) },
  { id: "vc-002", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-001", changeset_id: "cs-001", name: "Worktree SM invariant",       kind: "unit",        result: "pass",              coverage: 1.00, executed_at: ago(60 * 30) },
  { id: "vc-003", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-002", changeset_id: "cs-002", name: "Agent 14 SM all transitions", kind: "unit",        result: "pass",              coverage: 0.94, executed_at: ago(60 * 25) },
  { id: "vc-004", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-002", changeset_id: "cs-002", name: "Agent INV-AGT-N07",           kind: "unit",        result: "pass",              coverage: 1.00, executed_at: ago(60 * 25) },
  { id: "vc-005", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-007", changeset_id: "cs-003", name: "Automation CEL compile",      kind: "unit",        result: "feedback_required",  coverage: 0.78, feedback_id: "fb-001", executed_at: ago(60 * 3) },
  { id: "vc-006", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-009", changeset_id: "cs-004", name: "ChangeSet 5 SM",              kind: "unit",        result: "pass",              coverage: 1.00, executed_at: ago(60 * 36) },
  { id: "vc-007", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-008", changeset_id: "cs-012", name: "Permission rule deny",        kind: "unit",        result: "feedback_required",  coverage: 0.85, feedback_id: "fb-002", executed_at: ago(60 * 0.4) },
  { id: "vc-008", tenant_id: TENANT_ID, project_id: PROJECT_ID, work_item_id: "wi-013", changeset_id: "cs-011", name: "Validation 7 entity round-trip", kind: "integration", result: "pass",           coverage: 0.92, executed_at: ago(60 * 23) },
  { id: "vc-009", tenant_id: TENANT_ID, project_id: "prj-stargate", work_item_id: "wi-021", changeset_id: "cs-006", name: "Realtime event timing",       kind: "e2e",         result: "pass",              coverage: 0.81, executed_at: ago(60 * 0.3) },
  { id: "vc-010", tenant_id: TENANT_ID, project_id: "prj-mobile", work_item_id: "wi-025", changeset_id: "cs-008", name: "iOS switch policy",           kind: "policy",      result: "skipped",           coverage: 0.00, executed_at: ago(60 * 5) },
];

// =====================================================================
// local runtimes (5)
// =====================================================================
export const localRuntimes: LocalRuntime[] = [
  { id: "lr-001", tenant_id: TENANT_ID, device_id: "dev-001", hostname: "Ulysses-MBP",  status: "online",   bound_user_id: "usr-001", bound_tenant_id: TENANT_ID, mount_root: "/Users/ulysses/dev", last_heartbeat_at: ago(0.2), policy_violations: 0 },
  { id: "lr-002", tenant_id: TENANT_ID, device_id: "dev-002", hostname: "Hera-Win11",   status: "online",   bound_user_id: "usr-002", bound_tenant_id: TENANT_ID, mount_root: "D:/Dev",            last_heartbeat_at: ago(0.3), policy_violations: 1 },
  { id: "lr-003", tenant_id: TENANT_ID, device_id: "dev-003", hostname: "Athena-Linux", status: "online",   bound_user_id: "usr-005", bound_tenant_id: TENANT_ID, mount_root: "/home/athena/ws",  last_heartbeat_at: ago(0.1), policy_violations: 0 },
  { id: "lr-004", tenant_id: TENANT_ID, device_id: "dev-004", hostname: "Ares-MacStudio", status: "offline", bound_user_id: "usr-009", bound_tenant_id: TENANT_ID, mount_root: "/Users/ares/dev", last_heartbeat_at: ago(60 * 4), policy_violations: 0 },
  { id: "lr-005", tenant_id: TENANT_ID, device_id: "dev-005", hostname: "Hermes-CI",    status: "registered", bound_user_id: "usr-006", bound_tenant_id: TENANT_ID, mount_root: "/ci/runner",      last_heartbeat_at: ago(60 * 8), policy_violations: 0 },
];

// =====================================================================
// repositories + PRs
// =====================================================================
export const repositories: Repository[] = [
  { id: "repo-001", tenant_id: TENANT_ID, project_id: PROJECT_ID,    provider: "github", full_name: "acme/physis",          default_branch: "main", webhook_idempotency_key: "wh-physis", last_event_at: ago(0.5) },
  { id: "repo-002", tenant_id: TENANT_ID, project_id: PROJECT_ID,    provider: "github", full_name: "acme/gvpe-cabi",       default_branch: "main", webhook_idempotency_key: "wh-gvpe",   last_event_at: ago(60 * 3) },
  { id: "repo-003", tenant_id: TENANT_ID, project_id: "prj-stargate", provider: "gitlab", full_name: "acme/stargate-fe",    default_branch: "main", webhook_idempotency_key: "wh-sg",     last_event_at: ago(0.2) },
  { id: "repo-004", tenant_id: TENANT_ID, project_id: "prj-mobile",  provider: "gitea",  full_name: "acme/mobile-companion", default_branch: "main", webhook_idempotency_key: "wh-mob",    last_event_at: ago(60 * 6) },
];

export const pullRequests: PullRequest[] = [
  { id: "pr-001", tenant_id: TENANT_ID, repository_id: "repo-001", number: 101, title: "Worktree 17 状态机 + INV-WT-01~04",     author_id: "usr-002", source_branch: "feat/worktree-sm",   target_branch: "main", status: "merged",           review_state: "approved",           ci_state: "passing",  created_at: ago(60 * 30), merged_at: ago(60 * 28) },
  { id: "pr-002", tenant_id: TENANT_ID, repository_id: "repo-001", number: 102, title: "Agent 14 状态机 + 12 强制迁移",         author_id: "usr-002", source_branch: "feat/agent-14-sm",   target_branch: "main", status: "merged",           review_state: "approved",           ci_state: "passing",  created_at: ago(60 * 24), merged_at: ago(60 * 23) },
  { id: "pr-003", tenant_id: TENANT_ID, repository_id: "repo-001", number: 103, title: "Automation Rule executor v2",           author_id: "usr-003", source_branch: "feat/automation-v2", target_branch: "main", status: "review_required",  review_state: "changes_requested",  ci_state: "failing",  created_at: ago(60 * 5) },
  { id: "pr-004", tenant_id: TENANT_ID, repository_id: "repo-001", number: 104, title: "ChangeSet 5 状态机 + INV-DEV-01~05",   author_id: "usr-002", source_branch: "feat/changeset-5sm", target_branch: "main", status: "merged",           review_state: "approved",           ci_state: "passing",  created_at: ago(60 * 36), merged_at: ago(60 * 35) },
  { id: "pr-005", tenant_id: TENANT_ID, repository_id: "repo-002", number: 21,  title: "GVPE C ABI 边界类型",                   author_id: "usr-006", source_branch: "feat/gvpe-cabi",     target_branch: "main", status: "draft",            review_state: "none",               ci_state: "none",     created_at: ago(60 * 1) },
  { id: "pr-006", tenant_id: TENANT_ID, repository_id: "repo-003", number: 44,  title: "Realtime work-item event",              author_id: "usr-005", source_branch: "feat/realtime-wi",   target_branch: "main", status: "ci_failed",        review_state: "none",               ci_state: "failing",  created_at: ago(60 * 0.5) },
  { id: "pr-007", tenant_id: TENANT_ID, repository_id: "repo-003", number: 45,  title: "Presence cursor presence",              author_id: "usr-005", source_branch: "feat/presence",      target_branch: "main", status: "open",             review_state: "none",               ci_state: "pending",  created_at: ago(60 * 0.1) },
  { id: "pr-008", tenant_id: TENANT_ID, repository_id: "repo-004", number: 7,   title: "iOS worktree quick switch",             author_id: "usr-009", source_branch: "feat/ios-switch",    target_branch: "main", status: "open",             review_state: "approved",           ci_state: "passing",  created_at: ago(60 * 3) },
  { id: "pr-009", tenant_id: TENANT_ID, repository_id: "repo-001", number: 105, title: "Feedback reopen fix (abandoned)",       author_id: "usr-005", source_branch: "fix/feedback-reopen",target_branch: "main", status: "closed",           review_state: "none",               ci_state: "passing",  created_at: ago(60 * 22) },
  { id: "pr-010", tenant_id: TENANT_ID, repository_id: "repo-001", number: 106, title: "ContextPacket p0 cache (reverted)",     author_id: "usr-002", source_branch: "perf/context-p0",    target_branch: "main", status: "closed",           review_state: "none",               ci_state: "passing",  created_at: ago(60 * 18) },
  { id: "pr-011", tenant_id: TENANT_ID, repository_id: "repo-001", number: 107, title: "Validation 7 entities",                 author_id: "usr-003", source_branch: "feat/validation-7",  target_branch: "main", status: "merged",           review_state: "approved",           ci_state: "passing",  created_at: ago(60 * 22), merged_at: ago(60 * 21) },
  { id: "pr-012", tenant_id: TENANT_ID, repository_id: "repo-001", number: 108, title: "PermissionScheme rules RBAC",           author_id: "usr-003", source_branch: "feat/perm-rules",    target_branch: "main", status: "review_required",  review_state: "changes_requested",  ci_state: "passing",  created_at: ago(60 * 0.3) },
];

// =====================================================================
// notifications (INV-N-07)
// =====================================================================
export const notifications: Notification[] = [
  { id: "nt-001", tenant_id: TENANT_ID, recipient_id: "usr-001", kind: "agent_decision_required", channel: "inbox",  status: "delivered", subject: "Agent ag-003 等待人类决策: Automation rule cron 跨 project 边界",        body: "dec-004 已 rejected, 需要新 decision。",  ref_kind: "agent_session", ref_id: "ag-003", created_at: ago(60 * 1) },
  { id: "nt-002", tenant_id: TENANT_ID, recipient_id: "usr-002", kind: "feedback_question",       channel: "inbox",  status: "delivered", subject: "Feedback fb-002 等待回答: Permission rule CEL 解析失败",                body: "Question: fallback deny?",                       ref_kind: "feedback",       ref_id: "fb-002", created_at: ago(60 * 0.4) },
  { id: "nt-003", tenant_id: TENANT_ID, recipient_id: "usr-005", kind: "ci_failed",               channel: "inbox",  status: "read",      subject: "PR #44 CI failed: realtime-workitem e2e",                                   body: "3 tests failed in e2e/realtime",                ref_kind: "pr",             ref_id: "pr-006", created_at: ago(60 * 0.5) },
  { id: "nt-004", tenant_id: TENANT_ID, recipient_id: "usr-003", kind: "review_requested",        channel: "inbox",  status: "delivered", subject: "PR #103 review requested: Automation v2",                                  body: "@usr-003 needs review",                          ref_kind: "pr",             ref_id: "pr-003", created_at: ago(60 * 5) },
  { id: "nt-005", tenant_id: TENANT_ID, recipient_id: "usr-001", kind: "merge_conflict",          channel: "inbox",  status: "delivered", subject: "Merge conflict: wt-005 与 main 分歧",                                       body: "3 files need resolve",                           ref_kind: "worktree",       ref_id: "wt-005", created_at: ago(60 * 2) },
  { id: "nt-006", tenant_id: TENANT_ID, recipient_id: "usr-001", kind: "budget_alert",            channel: "inbox",  status: "read",      subject: "Agent ag-003 cost 60% of $3 budget",                                         body: "Consider raising budget or pause",               ref_kind: "agent_session",  ref_id: "ag-003", created_at: ago(60 * 3) },
  // 抑制示例(INV-N-07)
  { id: "nt-007", tenant_id: TENANT_ID, recipient_id: "usr-001", kind: "agent_decision_required", channel: "suppressed", status: "suppressed", subject: "[已抑制] Agent ag-002 等待人类决策", body: "(同 nt-001 同 actor, 抑制)", ref_kind: "agent_session", ref_id: "ag-002", suppression_reason: "INV-N-07: 同一 actor 60min 内同 kind 第 2 次", created_at: ago(60 * 25) },
  { id: "nt-008", tenant_id: TENANT_ID, recipient_id: "usr-001", kind: "ci_failed",               channel: "suppressed", status: "suppressed", subject: "[已抑制] PR #103 CI 第二次失败",    body: "(同 nt-004 关联 PR, 抑制)",   ref_kind: "pr",             ref_id: "pr-003", suppression_reason: "INV-N-07: 同 PR 24h 内第 2 次", created_at: ago(60 * 1) },
];

// =====================================================================
// search + saved
// =====================================================================
export const searchHits: SearchHit[] = [
  { id: "wi-001",  kind: "work_item", tenant_id: TENANT_ID, title: "Implement Worktree 17-state machine",                          snippet: "Worktree SM 已经过 §7.1 状态机评审, 17 个状态 + 4 个核心迁移。",  score: 0.98 },
  { id: "wt-001",  kind: "worktree",  tenant_id: TENANT_ID, title: "feat/worktree-sm",                                              snippet: "Worktree SM types + 14 状态机",                                    score: 0.94 },
  { id: "fb-001",  kind: "work_item", tenant_id: TENANT_ID, title: "AutomationRule cron 触发下是否启用 actor context?",            snippet: "Feedback fb-001 in_progress major",                                 score: 0.89 },
  { id: "pr-003",  kind: "work_item", tenant_id: TENANT_ID, title: "PR #103 review required: Automation v2",                        snippet: "changes_requested",                                                 score: 0.81 },
];
export const savedSearches: SavedSearch[] = [
  { id: "ss-001", tenant_id: TENANT_ID, name: "My work-items (in progress)",  query: "status:in_progress",          filters: { assignee: "me" },        created_by: "usr-001" },
  { id: "ss-002", tenant_id: TENANT_ID, name: "Awaiting feedback",            query: "status:awaiting_feedback",   filters: { project: PROJECT_ID },  created_by: "usr-001" },
  { id: "ss-003", tenant_id: TENANT_ID, name: "Overdue worktrees",            query: "status:active last_event:<24h", filters: { project: PROJECT_ID }, created_by: "usr-002" },
];

// =====================================================================
// integrations
// =====================================================================
export const integrations: Integration[] = [
  { id: "int-001", tenant_id: TENANT_ID, kind: "github",   display_name: "GitHub (acme org)",         status: "active",      config_masked: "****ghp_***REDACTED***", loop_protection_key: "lp-github", last_sync_at: ago(0.4), error_count_24h: 0 },
  { id: "int-002", tenant_id: TENANT_ID, kind: "gitlab",   display_name: "GitLab (stargate)",         status: "active",      config_masked: "****glpat-***REDACTED***", loop_protection_key: "lp-gitlab", last_sync_at: ago(0.2), error_count_24h: 0 },
  { id: "int-003", tenant_id: TENANT_ID, kind: "jira",     display_name: "Jira Mirror (read-only)",   status: "active",      config_masked: "jira://acme.atlassian.net?***", loop_protection_key: "lp-jira", last_sync_at: ago(5), error_count_24h: 1 },
  { id: "int-004", tenant_id: TENANT_ID, kind: "slack",    display_name: "#star-platform",            status: "active",      config_masked: "slack://T02***/C03***", loop_protection_key: "lp-slack", last_sync_at: ago(0.6), error_count_24h: 0 },
  { id: "int-005", tenant_id: TENANT_ID, kind: "lark",     display_name: "Lark Bot",                  status: "paused",      config_masked: "lark://cli_a8***", loop_protection_key: "lp-lark", last_sync_at: ago(60 * 2), error_count_24h: 0 },
  { id: "int-006", tenant_id: TENANT_ID, kind: "linear",   display_name: "Linear Mirror",             status: "circuit_open",config_masked: "lin_api_***REDACTED***", loop_protection_key: "lp-linear", last_sync_at: ago(0.1), error_count_24h: 12 },
  { id: "int-007", tenant_id: TENANT_ID, kind: "webhook",  display_name: "Physis CI outbound",        status: "active",      config_masked: "https://ci.physis.dev/hook?key=***", loop_protection_key: "lp-physis-ci", last_sync_at: ago(0.3), error_count_24h: 0 },
];

// =====================================================================
// collaboration
// =====================================================================
export const presenceCursors: PresenceCursor[] = [
  { user_id: "usr-001", workspace_id: "ws-001", x: 240,  y: 120, selection: "Worktree SM 类型声明", updated_at: ago(0.1) },
  { user_id: "usr-002", workspace_id: "ws-001", x: 510,  y: 340, selection: "INV-WT-03",            updated_at: ago(0.2) },
  { user_id: "usr-003", workspace_id: "ws-001", x: 920,  y: 80,  selection: "状态机迁移表",          updated_at: ago(0.05) },
];
export const whiteboards: Whiteboard[] = [
  { id: "wb-001", tenant_id: TENANT_ID, workspace_id: "ws-001", title: "Worktree SM 设计",        collaborator_ids: ["usr-001","usr-002","usr-003"], snapshot_url: "/wb/001.png", updated_at: ago(60 * 4) },
  { id: "wb-002", tenant_id: TENANT_ID, workspace_id: "ws-003", title: "Agent 14 SM 状态迁移",    collaborator_ids: ["usr-001","usr-002"],         snapshot_url: "/wb/002.png", updated_at: ago(60 * 8) },
  { id: "wb-003", tenant_id: TENANT_ID, workspace_id: "ws-004", title: "Dashboard 信息架构",       collaborator_ids: ["usr-002","usr-005","usr-007"], snapshot_url: "/wb/003.png", updated_at: ago(60 * 1) },
];

// =====================================================================
// Canvas(无限画布)— Miro 模式
// =====================================================================
export const canvases: Canvas[] = [
  {
    id: "canvas-001",
    tenant_id: TENANT_ID,
    workspace_id: "ws-001",
    title: "Physis Sprint 23 — Worktree + Agent 工作流",
    ref_kind: "project",
    ref_id: PROJECT_ID,
    viewport: { x: 0, y: 0, zoom: 1 },
    frames: [
      { id: "frame-001", canvas_id: "canvas-001", title: "Worktree 状态",     x: 0,   y: 0,   width: 500, height: 360, element_ids: ["el-wi-001","el-wi-002","el-wt-001","el-wt-002","el-wt-003"], is_slide: true,  order: 0 },
      { id: "frame-002", canvas_id: "canvas-001", title: "Agent Session",      x: 600, y: 0,   width: 460, height: 360, element_ids: ["el-ag-001","el-ag-002","el-ag-003"],                          is_slide: true,  order: 1 },
      { id: "frame-003", canvas_id: "canvas-001", title: "Feedback Inbox",     x: 0,   y: 460, width: 500, height: 300, element_ids: ["el-fb-001","el-fb-002","el-sn-001"],                            is_slide: false, order: 2 },
      { id: "frame-004", canvas_id: "canvas-001", title: "Automation Rules",   x: 600, y: 460, width: 460, height: 300, element_ids: ["el-au-001","el-au-002"],                                            is_slide: true,  order: 3 },
    ],
    creator_id: "usr-001",
    collaborator_ids: ["usr-001","usr-002","usr-003","usr-005"],
    created_at: ago(60 * 24 * 5),
    updated_at: ago(60 * 2),
    snapshot_url: "/canvas/001.png",
  },
  {
    id: "canvas-002",
    tenant_id: TENANT_ID,
    workspace_id: "ws-004",
    title: "Dashboard 信息架构 — Realtime 拓扑",
    ref_kind: "free",
    viewport: { x: 0, y: 0, zoom: 1 },
    frames: [],
    creator_id: "usr-002",
    collaborator_ids: ["usr-002","usr-005","usr-007"],
    created_at: ago(60 * 24 * 2),
    updated_at: ago(60 * 1),
  },
];

// Canvas Element — 25-30 个,跨 7 种 kind,演示 25 module 联动
export const canvasElements: CanvasElement[] = [
  // === Frame 1: Worktree 状态 ===
  { id: "el-wi-001", canvas_id: "canvas-001", kind: "work_item_card", x: 30,  y: 50,  width: 200, height: 90,  rotation: 0, z_index: 1, content: { work_item_id: "wi-001", text: "Implement Worktree 17-state machine" }, locked: false, hidden: false, created_by: "usr-001", created_at: ago(60 * 24 * 5), updated_at: ago(60 * 4) },
  { id: "el-wi-002", canvas_id: "canvas-001", kind: "work_item_card", x: 30,  y: 180, width: 200, height: 90,  rotation: 0, z_index: 1, content: { work_item_id: "wi-002", text: "Agent 14 状态机 + 12 强制迁移" },         locked: false, hidden: false, created_by: "usr-001", created_at: ago(60 * 24 * 5), updated_at: ago(60 * 4) },
  { id: "el-wt-001", canvas_id: "canvas-001", kind: "worktree_node",  x: 260, y: 50,  width: 220, height: 70,  rotation: 0, z_index: 2, content: { worktree_id: "wt-001", text: "feat/worktree-sm" },         locked: false, hidden: false, created_by: "usr-001", created_at: ago(60 * 24 * 5), updated_at: ago(60 * 4) },
  { id: "el-wt-002", canvas_id: "canvas-001", kind: "worktree_node",  x: 260, y: 140, width: 220, height: 70,  rotation: 0, z_index: 2, content: { worktree_id: "wt-002", text: "feat/agent-14-sm" },         locked: false, hidden: false, created_by: "usr-001", created_at: ago(60 * 24 * 5), updated_at: ago(60 * 4) },
  { id: "el-wt-003", canvas_id: "canvas-001", kind: "worktree_node",  x: 260, y: 230, width: 220, height: 70,  rotation: 0, z_index: 2, content: { worktree_id: "wt-003", text: "feat/automation-v2" },      locked: false, hidden: false, created_by: "usr-001", created_at: ago(60 * 24 * 5), updated_at: ago(60 * 4) },
  // === Frame 2: Agent Session ===
  { id: "el-ag-001", canvas_id: "canvas-001", kind: "agent_cursor",    x: 630, y: 50,  width: 180, height: 80,  rotation: 0, z_index: 2, content: { agent_session_id: "ag-001", text: "ag-001 / claude-sonnet" },   locked: false, hidden: false, created_by: "usr-002", created_at: ago(60 * 24 * 4), updated_at: ago(60 * 2) },
  { id: "el-ag-002", canvas_id: "canvas-001", kind: "agent_cursor",    x: 630, y: 160, width: 180, height: 80,  rotation: 0, z_index: 2, content: { agent_session_id: "ag-002", text: "ag-002 / codex" },         locked: false, hidden: false, created_by: "usr-002", created_at: ago(60 * 24 * 4), updated_at: ago(60 * 2) },
  { id: "el-ag-003", canvas_id: "canvas-001", kind: "agent_cursor",    x: 630, y: 270, width: 180, height: 80,  rotation: 0, z_index: 2, content: { agent_session_id: "ag-003", text: "ag-003 / awaiting_human" },  locked: false, hidden: false, created_by: "usr-002", created_at: ago(60 * 24 * 4), updated_at: ago(60 * 1) },
  // === Frame 3: Feedback Inbox ===
  { id: "el-fb-001", canvas_id: "canvas-001", kind: "sticky_note",     x: 30,  y: 510, width: 140, height: 100, rotation: 0, z_index: 1, content: { text: "Need spec clarification on CEL guard scope", color: "#f9d77e" }, locked: false, hidden: false, created_by: "usr-005", created_at: ago(60 * 24 * 3), updated_at: ago(60 * 3) },
  { id: "el-fb-002", canvas_id: "canvas-001", kind: "sticky_note",     x: 200, y: 510, width: 140, height: 100, rotation: 0, z_index: 1, content: { text: "Permission rule CEL 解析失败 fallback", color: "#ffb3c1" }, locked: false, hidden: false, created_by: "usr-005", created_at: ago(60 * 24 * 3), updated_at: ago(60 * 0.4) },
  { id: "el-sn-001", canvas_id: "canvas-001", kind: "comment_pin",     x: 370, y: 540, width: 90,  height: 60,  rotation: 0, z_index: 3, content: { comment_id: "cm-001", text: "3 comments" },          locked: false, hidden: false, created_by: "usr-002", created_at: ago(60 * 24 * 3), updated_at: ago(60 * 6) },
  // === Frame 4: Automation Rules ===
  { id: "el-au-001", canvas_id: "canvas-001", kind: "automation_node", x: 630, y: 510, width: 200, height: 90,  rotation: 0, z_index: 2, content: { automation_id: "au-rule-001", text: "Auto-assign P0" }, locked: false, hidden: false, created_by: "usr-003", created_at: ago(60 * 24 * 3), updated_at: ago(60 * 2) },
  { id: "el-au-002", canvas_id: "canvas-001", kind: "automation_node", x: 850, y: 510, width: 200, height: 90,  rotation: 0, z_index: 2, content: { automation_id: "au-rule-003", text: "Dispatch agent on PR review" }, locked: false, hidden: false, created_by: "usr-003", created_at: ago(60 * 24 * 3), updated_at: ago(60 * 3) },
  // === 自由 element ===
  { id: "el-tx-001", canvas_id: "canvas-001", kind: "text",           x: 1100, y: 200, width: 240, height: 60,  rotation: 0, z_index: 0, content: { text: "INV-AGT-N07: Agent 14 状态机 + 12 强制迁移" },     locked: false, hidden: false, created_by: "usr-001", created_at: ago(60 * 24 * 5), updated_at: ago(60 * 4) },
];

// Canvas Connector — 演示 Relation 域联动
export const canvasConnectors: CanvasConnector[] = [
  { id: "el-cn-001", canvas_id: "canvas-001", kind: "work_item_relation", from_element_id: "el-wi-001", to_element_id: "el-wt-001", routing: "curved",  arrow_start: false, arrow_end: true,  color: "#2f81f7", width: 2, label: "parent_of",  relation_id: "rl-001" },
  { id: "el-cn-002", canvas_id: "canvas-001", kind: "work_item_relation", from_element_id: "el-wi-002", to_element_id: "el-wt-002", routing: "curved",  arrow_start: false, arrow_end: true,  color: "#2f81f7", width: 2, label: "parent_of",  relation_id: "rl-002" },
  { id: "el-cn-003", canvas_id: "canvas-001", kind: "work_item_relation", from_element_id: "el-wi-001", to_element_id: "el-wi-002", routing: "curved",  arrow_start: true,  arrow_end: true,  color: "#d29922", width: 2, label: "duplicates", relation_id: "rl-005" },
  { id: "el-cn-004", canvas_id: "canvas-001", kind: "agent_handoff",      from_element_id: "el-wt-001", to_element_id: "el-ag-001", routing: "curved",  arrow_start: false, arrow_end: true,  color: "#3fb950", width: 2, label: "executes" },
  { id: "el-cn-005", canvas_id: "canvas-001", kind: "agent_handoff",      from_element_id: "el-wt-002", to_element_id: "el-ag-002", routing: "curved",  arrow_start: false, arrow_end: true,  color: "#3fb950", width: 2, label: "executes" },
  { id: "el-cn-006", canvas_id: "canvas-001", kind: "agent_handoff",      from_element_id: "el-wt-003", to_element_id: "el-ag-003", routing: "curved",  arrow_start: false, arrow_end: true,  color: "#3fb950", width: 2, label: "executes" },
  { id: "el-cn-007", canvas_id: "canvas-001", kind: "free",               from_element_id: "el-fb-001", to_element_id: "el-wi-001", routing: "orthogonal", arrow_start: false, arrow_end: true, color: "#8b949e", width: 1, label: "blocks" },
  { id: "el-cn-008", canvas_id: "canvas-001", kind: "dependency",         from_element_id: "el-au-001", to_element_id: "el-wi-001", routing: "straight", arrow_start: false, arrow_end: true,  color: "#d29922", width: 2, label: "triggers" },
];

// =====================================================================
// planning
// =====================================================================
export const sprints: Sprint[] = [
  { id: "spr-001", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Sprint 23 — Worktree SM", goal: "17 状态机 + INV-WT-01~04 全部合入", status: "active", start_date: ago(60 * 24 * 14), end_date: ago(-60 * 24 * 0), capacity_points: 60, committed_points: 55, completed_points: 41 },
  { id: "spr-002", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Sprint 24 — AI Auto-Approve", goal: "Automation + Permission + Validation", status: "planned", start_date: ago(-60 * 24 * 7), end_date: ago(-60 * 24 * 21), capacity_points: 55, committed_points: 0, completed_points: 0 },
  { id: "spr-003", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Sprint 22 — 已归档",         goal: "Worktree 17 SM 准备",             status: "completed", start_date: ago(60 * 24 * 28), end_date: ago(60 * 24 * 14), capacity_points: 50, committed_points: 48, completed_points: 48 },
  { id: "spr-004", tenant_id: TENANT_ID, project_id: "prj-stargate", name: "SG Sprint 5 — Realtime", goal: "Realtime work-item event",        status: "active",   start_date: ago(60 * 24 * 7),  end_date: ago(-60 * 24 * 7), capacity_points: 40, committed_points: 32, completed_points: 18 },
];
export const milestones: Milestone[] = [
  { id: "ms-001", tenant_id: TENANT_ID, project_id: PROJECT_ID,    name: "MVP 0.5 — Worktree + Agent 全绿", due_date: ago(-60 * 24 * 7),  work_item_ids: ["wi-001","wi-002","wi-009","wi-013"], progress: 0.85 },
  { id: "ms-002", tenant_id: TENANT_ID, project_id: PROJECT_ID,    name: "MVP 0.6 — Automation + Permission", due_date: ago(-60 * 24 * 30), work_item_ids: ["wi-007","wi-008"],                progress: 0.45 },
  { id: "ms-003", tenant_id: TENANT_ID, project_id: "prj-stargate", name: "Stargate Beta",                  due_date: ago(-60 * 24 * 14), work_item_ids: ["wi-021","wi-022"],                progress: 0.65 },
  { id: "ms-004", tenant_id: TENANT_ID, project_id: "prj-mobile",  name: "iOS TestFlight",                 due_date: ago(-60 * 24 * 21), work_item_ids: ["wi-025","wi-026","wi-027"],     progress: 0.30 },
];
export const burndownSeries: BurndownPoint[] = Array.from({ length: 14 }, (_, i) => {
  const remaining = Math.max(0, Math.round(55 - i * 4 - (i > 5 ? 2 : 0) - (i > 9 ? 1 : 0)));
  return { date: ago(60 * 24 * (14 - i)), remaining_points: remaining, ideal_points: 55 - i * 4 };
});

// =====================================================================
// board
// =====================================================================
export const board: Board = {
  id: "board-001", tenant_id: TENANT_ID, project_id: PROJECT_ID,
  name: "Physis Sprint 23",
  columns: [
    { status: "todo",        work_item_ids: ["wi-004","wi-011","wi-015","wi-023"],            wip_limit: 8 },
    { status: "in_progress", work_item_ids: ["wi-001","wi-005","wi-007","wi-010","wi-017"], wip_limit: 5 },
    { status: "review",      work_item_ids: ["wi-002","wi-008","wi-018","wi-022"],            wip_limit: 3 },
    { status: "done",        work_item_ids: ["wi-003","wi-009","wi-013","wi-016","wi-020"], wip_limit: 99 },
  ],
};

// =====================================================================
// relations (10)
// =====================================================================
export const relations: Relation[] = [
  { id: "rl-001", tenant_id: TENANT_ID, from_kind: "work_item", from_id: "wi-001", to_kind: "worktree", to_id: "wt-001", kind: "parent_of", created_at: ago(60 * 35) },
  { id: "rl-002", tenant_id: TENANT_ID, from_kind: "work_item", from_id: "wi-002", to_kind: "worktree", to_id: "wt-002", kind: "parent_of", created_at: ago(60 * 28) },
  { id: "rl-003", tenant_id: TENANT_ID, from_kind: "worktree", from_id: "wt-005", to_kind: "worktree", to_id: "wt-001", kind: "relates_to", created_at: ago(60 * 4) },
  { id: "rl-004", tenant_id: TENANT_ID, from_kind: "work_item", from_id: "wi-007", to_kind: "work_item", to_id: "wi-008", kind: "blocks", created_at: ago(60 * 12) },
  { id: "rl-005", tenant_id: TENANT_ID, from_kind: "work_item", from_id: "wi-005", to_kind: "work_item", to_id: "wi-002", kind: "duplicates", created_at: ago(60 * 20) },
  { id: "rl-006", tenant_id: TENANT_ID, from_kind: "work_item", from_id: "wi-014", to_kind: "work_item", to_id: "wi-011", kind: "relates_to", created_at: ago(60 * 6) },
  { id: "rl-007", tenant_id: TENANT_ID, from_kind: "work_item", from_id: "wi-021", to_kind: "work_item", to_id: "wi-022", kind: "blocks", created_at: ago(60 * 0.5) },
  { id: "rl-008", tenant_id: TENANT_ID, from_kind: "agent_session", from_id: "ag-005", to_kind: "worktree", to_id: "wt-005", kind: "parent_of", created_at: ago(60 * 0.5) },
  { id: "rl-009", tenant_id: TENANT_ID, from_kind: "changeset", from_id: "cs-003", to_kind: "work_item", to_id: "wi-007", kind: "parent_of", created_at: ago(60 * 11) },
  { id: "rl-010", tenant_id: TENANT_ID, from_kind: "worktree", from_id: "wt-007", to_kind: "worktree", to_id: "wt-006", kind: "cloned_from", created_at: ago(60 * 1) },
];

// =====================================================================
// audit (20) — 包含 9 AI 问题
// =====================================================================
export const auditEvents: AuditEvent[] = [
  { id: "au-001", tenant_id: TENANT_ID, actor_id: "usr-001", category: "permission",    action: "scheme.update",       target_kind: "permission_scheme", target_id: "ps-strict", payload: { before: { rule_count: 6 }, after: { rule_count: 8 } }, prev_hash: "0x00", hash: "0xae3f", created_at: ago(60 * 24) },
  { id: "au-002", tenant_id: TENANT_ID, actor_id: "usr-002", category: "data_access",   action: "workitem.read",       target_kind: "work_item",         target_id: "wi-001",   payload: { read_fields: ["title","description"] }, prev_hash: "0xae3f", hash: "0x11c2", created_at: ago(60 * 12) },
  { id: "au-003", tenant_id: TENANT_ID, actor_id: "usr-002", category: "ai_decision",   action: "agent.decision",      target_kind: "context_decision",   target_id: "dec-001",  payload: { chosen: "initializing", rejected: ["active"] }, ai_metadata: { agent_session_id: "ag-001", prompt_hash: "sha256:ab12", decision_id: "dec-001", confidence: 0.92 }, prev_hash: "0x11c2", hash: "0x9f8a", created_at: ago(60 * 30) },
  { id: "au-004", tenant_id: TENANT_ID, actor_id: "usr-001", category: "config_change", action: "automation.toggle",   target_kind: "automation_rule",   target_id: "au-rule-007", payload: { enabled: false }, prev_hash: "0x9f8a", hash: "0x44e1", created_at: ago(60 * 8) },
  { id: "au-005", tenant_id: TENANT_ID, actor_id: "usr-002", category: "data_access",   action: "worktree.commit",     target_kind: "worktree",          target_id: "wt-001",   payload: { diff_summary: "+342 / -18 / 4 files" }, prev_hash: "0x44e1", hash: "0x77b2", created_at: ago(60 * 30) },
  { id: "au-006", tenant_id: TENANT_ID, actor_id: "usr-001", category: "auth",          action: "session.login",       target_kind: "identity",          target_id: "usr-001", payload: { provider: "github", ip: "10.0.0.4", ua: "Mozilla/5.0" }, prev_hash: "0x77b2", hash: "0xab33", created_at: ago(60 * 6) },
  { id: "au-007", tenant_id: TENANT_ID, actor_id: "usr-005", category: "integration",   action: "integration.error",   target_kind: "integration",       target_id: "int-006", payload: { kind: "linear", error: "rate_limit", retry_after_s: 60 }, prev_hash: "0xab33", hash: "0x22dd", created_at: ago(0.1) },
  { id: "au-008", tenant_id: TENANT_ID, actor_id: "usr-002", category: "ai_decision",   action: "feedback.answer",     target_kind: "feedback",          target_id: "fb-004",   payload: { question: "BFS depth", answer: "5" }, ai_metadata: { agent_session_id: "ag-005", decision_id: "dec-002", confidence: 0.88 }, prev_hash: "0x22dd", hash: "0xee51", created_at: ago(60 * 4) },
  { id: "au-009", tenant_id: TENANT_ID, actor_id: "usr-001", category: "policy_violation", action: "policy.deny",     target_kind: "automation_rule",   target_id: "au-rule-007", payload: { rule: "call_webhook:url not in allowlist" }, prev_hash: "0xee51", hash: "0x6a40", created_at: ago(60 * 3) },
  { id: "au-010", tenant_id: TENANT_ID, actor_id: "usr-003", category: "system",        action: "ci.run",              target_kind: "pull_request",      target_id: "pr-003",   payload: { workflow: "ci.yml", run_id: 9981, status: "failing" }, prev_hash: "0x6a40", hash: "0x12fe", created_at: ago(60 * 0.4) },
  { id: "au-011", tenant_id: TENANT_ID, actor_id: "usr-002", category: "ai_decision",   action: "agent.handoff",       target_kind: "agent_session",     target_id: "ag-005",  payload: { from: "compiling_context", to: "planning" }, ai_metadata: { agent_session_id: "ag-005", confidence: 0.95 }, prev_hash: "0x12fe", hash: "0x80a1", created_at: ago(60 * 0.5) },
  { id: "au-012", tenant_id: TENANT_ID, actor_id: "usr-001", category: "data_access",   action: "audit.read.cross_tenant", target_kind: "audit_event",   target_id: "au-013",   payload: { from_tenant: TENANT_ID, to_tenant: TENANT_ID, reason: "compliance_review" }, prev_hash: "0x80a1", hash: "0xfa10", created_at: ago(60 * 24 * 2) },
  { id: "au-013", tenant_id: TENANT_ID, actor_id: "usr-001", category: "billing",       action: "billing.usage",       payload: { service: "agent.token", units: 162_000, usd: 0.92 }, prev_hash: "0xfa10", hash: "0x39ce", created_at: ago(60 * 30) },
  { id: "au-014", tenant_id: TENANT_ID, actor_id: "usr-001", category: "permission",    action: "role.grant",          target_kind: "identity",          target_id: "usr-007", payload: { role: "developer" }, prev_hash: "0x39ce", hash: "0xb022", created_at: ago(60 * 1) },
  { id: "au-015", tenant_id: TENANT_ID, actor_id: "usr-005", category: "ai_decision",   action: "agent.tool_call",     target_kind: "agent_session",     target_id: "ag-005",  payload: { tool: "grep", args: { pattern: "INV-WT" } }, ai_metadata: { agent_session_id: "ag-005", confidence: 1.0 }, prev_hash: "0xb022", hash: "0x57a9", created_at: ago(60 * 0.4) },
  { id: "au-016", tenant_id: TENANT_ID, actor_id: "usr-001", category: "data_access",   action: "tenant.export",       target_kind: "tenant",            target_id: TENANT_ID, payload: { format: "jsonl", events: 16_842 }, prev_hash: "0x57a9", hash: "0xc18e", created_at: ago(60 * 12) },
  { id: "au-017", tenant_id: TENANT_ID, actor_id: "usr-002", category: "ai_decision",   action: "agent.wait_human",    target_kind: "agent_session",     target_id: "ag-003",  payload: { reason: "cross-tenant boundary" }, ai_metadata: { agent_session_id: "ag-003", confidence: 0.71 }, prev_hash: "0xc18e", hash: "0x2ab7", created_at: ago(60 * 1) },
  { id: "au-018", tenant_id: TENANT_ID, actor_id: "usr-003", category: "integration",   action: "webhook.received",    target_kind: "integration",       target_id: "int-001", payload: { event: "pull_request", delivery_id: "d-9981", idempotency_key: "wh-physis:9981" }, prev_hash: "0x2ab7", hash: "0x9d44", created_at: ago(60 * 0.3) },
  { id: "au-019", tenant_id: TENANT_ID, actor_id: "usr-001", category: "policy_violation", action: "local_runtime.alert", target_kind: "local_runtime",  target_id: "lr-002", payload: { alert: "mount_path not in policy.allowlist" }, prev_hash: "0x9d44", hash: "0x66c1", created_at: ago(60 * 5) },
  { id: "au-020", tenant_id: TENANT_ID, actor_id: "usr-002", category: "ai_decision",   action: "agent.complete",      target_kind: "agent_session",     target_id: "ag-002",  payload: { tokens: 219_000, cost_usd: 1.18 }, ai_metadata: { agent_session_id: "ag-002", confidence: 1.0 }, prev_hash: "0x66c1", hash: "0x7710", created_at: ago(60 * 23) },
];

// =====================================================================
// automation rules
// =====================================================================
export const automationRules: AutomationRule[] = [
  { id: "au-rule-001", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Auto-assign P0 to on-call",            enabled: true,  trigger_kind: "workitem_status_changed",   trigger_filter: { kind: "story", priority: "p0" }, actions: [{ kind: "assign_user", config: { strategy: "round_robin_oncall" } }], execution_count_24h: 4, last_fired_at: ago(60 * 2) },
  { id: "au-rule-002", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Block PR if no work-item link",         enabled: true,  trigger_kind: "pr_status_changed",        trigger_filter: { to: "open" },                 condition_expr: "resource.work_item_id == null", actions: [{ kind: "set_label", config: { label: "needs-spec" } }], execution_count_24h: 1, last_fired_at: ago(60 * 6) },
  { id: "au-rule-003", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Dispatch agent on PR review",          enabled: true,  trigger_kind: "pr_status_changed",        trigger_filter: { to: "review_required" },     actions: [{ kind: "dispatch_agent", config: { kind: "codex", budget_usd: 0.5 } }], execution_count_24h: 2, last_fired_at: ago(60 * 3) },
  { id: "au-rule-004", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Notify on CI failed (1st only)",       enabled: true,  trigger_kind: "pr_status_changed",        trigger_filter: { to: "ci_failed" },            condition_expr: "context.run_number == 1", actions: [{ kind: "send_notification", config: { kind: "ci_failed", suppress_within_h: 24 } }], execution_count_24h: 1, last_fired_at: ago(0.5) },
  { id: "au-rule-005", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Auto-create worktree for story",       enabled: true,  trigger_kind: "workitem_status_changed",   trigger_filter: { to: "in_progress", kind: "story" }, actions: [{ kind: "create_worktree", config: { base: "main" } }], execution_count_24h: 3, last_fired_at: ago(60 * 1) },
  { id: "au-rule-006", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Nightly audit export (cron 02:00)",    enabled: true,  trigger_kind: "schedule_cron",            trigger_filter: { cron: "0 2 * * *" },        actions: [{ kind: "call_webhook", config: { url: "https://audit.physis.dev/daily", secret_ref: "audit-daily-key" } }], execution_count_24h: 1, last_fired_at: ago(60 * 6) },
  { id: "au-rule-007", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "DISABLED: external webhook (denied)",   enabled: false, trigger_kind: "agent_session_completed",  trigger_filter: {},                       actions: [{ kind: "call_webhook", config: { url: "https://evil.example.com/x" } }], execution_count_24h: 0 },
  { id: "au-rule-008", tenant_id: TENANT_ID, project_id: "prj-stargate", name: "Realtime work-item event → WS broadcast", enabled: true, trigger_kind: "workitem_status_changed", trigger_filter: { project: "prj-stargate" }, actions: [{ kind: "send_notification", config: { kind: "realtime_event", channel: "ws" } }], execution_count_24h: 5, last_fired_at: ago(0.3) },
  { id: "au-rule-009", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Feedback received → re-prioritize",    enabled: true,  trigger_kind: "feedback_received",        trigger_filter: { severity_in: ["major","critical"] }, actions: [{ kind: "set_label", config: { label: "hot" } }], execution_count_24h: 1, last_fired_at: ago(60 * 0.4) },
  { id: "au-rule-010", tenant_id: TENANT_ID, project_id: PROJECT_ID, name: "Audit event → Sentry (sample 1%)",    enabled: true,  trigger_kind: "audit_event",             trigger_filter: { category: "policy_violation" }, actions: [{ kind: "call_webhook", config: { url: "https://sentry.io/api/...", sample_rate: 0.01 } }], execution_count_24h: 0 },
];
