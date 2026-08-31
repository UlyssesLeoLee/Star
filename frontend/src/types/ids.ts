// =====================================================================
// Star Platform — Shared Type Definitions
// =====================================================================
// 来源: docs/api-design.md §2.1 (25 Module Resource Model) +
//       docs/specs/* (25 个 domain spec) + docs/basic-design.md §7
//       (5 状态机: Worktree 17 / WorkItem 3-5 / Feedback 6 / Agent 14 / PR 7)
//
// 本文件不复制 backend Rust 类型,而是按 UI 渲染需要做的"投影层"。
// 严格保留 backend INV 命名(Worktree INV-WT-NN / Agent INV-AGT-NN ...)
// =====================================================================

export type Uuid = string;
export type Iso8601 = string; // "2026-08-26T11:30:00Z"

// =====================================================================
// 26. IncidentRecord (per REQ-OPS-001/002/003, test-design §6.3.4)
// =====================================================================
// IncidentRecord 是"事件发生 → ChangeSet → 修复 WorkItem → 验证证据"链路的
// 追溯锚点 (REQ-OPS-001),且必须能标注"哪些 AC 证据不充分"
// (REQ-OPS-002,不得重写历史 ValidationResult / Acceptance Coverage 判定)。
//
// 边界 (per REQ-OPS-003 / requirements.md §30.6):
//   系统**不得**主动探查生产 / 处理告警 / 自动回滚/自动修复;
//   IncidentRecord 只能通过 source = "human_entry" | "integration_webhook"
//   登记 (集成 webhook 经 §18 Integration Webhook 转登)。
//
// 设计依据:
//   - docs/test-design.md §6.3.4 (V1 Should-Have Test T3, TBD)
//   - docs/requirements.md §29.1 + §30.6
//   - 守门 (per AGENTS.md §1.2 #3 缺标比错标):
//     Schema 只覆盖允许字段, 3 项非能力端点的具体错误文案
//     ("REQ-OPS-003 boundary" 占位) 等 basic-design §30.6 跟进后回填
// =====================================================================

/** IncidentRecord 来源: 只能人工录入 或经 §18 Integration Webhook 转登 */
export type IncidentSource = "human_entry" | "integration_webhook";

/** IncidentRecord 主体 */
export interface IncidentRecord {
  id: Uuid;
  title: string;
  source: IncidentSource;
  /** REQ-OPS-001: 关联 0..N 个 WorkItem (可空, 0..N 范围) */
  linked_work_item_ids: Uuid[];
  /** REQ-OPS-002: 标注证据不充分的 AC (只能标注, 不得改写历史 ValidationResult) */
  affected_ac_ids: Uuid[];
  /** 事件发生时间 (ISO 8601) */
  occurred_at: Iso8601;
  /** 录入时间 (ISO 8601) */
  recorded_at: Iso8601;
  /** 录入者 (human user id, per REQ-OPS-003 限制) */
  recorded_by: Uuid;
  /** 自由文本备注; 不得含 auto_rollback / auto_remediation / alert_handler 关键词 */
  notes: string;
}

// ----- 通用 -----
export interface ActorContext {
  user_id: Uuid;
  tenant_id: Uuid;
  device_id?: Uuid;
  project_ids: Uuid[];
  roles: Array<"tenant_admin" | "project_admin" | "developer" | "viewer">;
}

// ----- 13 类 tenant_id 必带对象(§6.1 REQ-SEC-001) -----
export type TenantScopedKind =
  | "tenant" | "project" | "workspace" | "identity" | "permission"
  | "work_item" | "comment" | "worktree" | "agent_session"
  | "audit_event" | "automation_rule" | "scm_repository" | "notification";

// ----- 模块(MRU 上 25 个) -----
export type ModuleName =
  // Track B/C
  | "worktree" | "feedback" | "validation" | "integration" | "scm"
  | "agent" | "context" | "notification" | "search"
  // Track D
  | "tenant" | "project" | "identity" | "work-item" | "comment"
  | "permission" | "workflow" | "development"
  // Track E
  | "collaboration" | "planning" | "board" | "local-runtime" | "relation"
  | "workspace" | "audit" | "automation";

// ----- Test Level 维度 (per REQ-TST-001, docs/test-design.md §6.2.1) -----
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. basic-design §4.5.6 字段细节 TBD (e.g. 是否携带 fixture_path / duration_ms),
//      现阶段按 UI 投影层最小集 5 字段 (id / work_item_id / kind / status / level / evidence_ref / linked_ac_ids / created_at).
//   2. ValidationResult 6 状态机与 §27.1 spec 对齐: running/passed/failed/errored/skipped/superseded
//      (per §A.5 supersede 由 P2 ValidationPolicy 引入, 此处先列).
//   3. **命名冲突**: 现有 §14 ValidationResult 是字符串联合 ("pass" | "fail" | "skipped" | "feedback_required"),
//      充当 ValidationCase.result 的 outcome 状态 (per seed.ts + app/validation/page.tsx 已用).
//      按 spec §27.1 真正的"聚合根"应该叫 ValidationResult (本应承担 evidence_ref / level 等).
//      改名需重写 §14 既有 2 个 consumer (scope 限定不碰), 故本节先以 ValidationResultRecord
//      落地 REQ-TST-001/002 字段集合, basic-design §4.5.6 拍板时一并把 §14 字符串联合迁成
//      ValidationOutcome, 把 ValidationResultRecord 改名回 ValidationResult (per 守门 #12 docs 同步).
export type TestLevel = "unit" | "integration" | "system" | "acceptance";
export const TEST_LEVELS: readonly TestLevel[] = [
  "unit",
  "integration",
  "system",
  "acceptance",
] as const;

export type ValidationResultKind =
  | "build" | "test" | "lint" | "contract" | "security";
export type ValidationResultStatus =
  | "running" | "passed" | "failed" | "errored" | "skipped" | "superseded";

export interface ValidationResultRecord {
  id: Uuid;
  work_item_id: Uuid;
  kind: ValidationResultKind;
  status: ValidationResultStatus;
  level: TestLevel; // 必填,per REQ-TST-001
  evidence_ref: string; // INV-VL-04 必填 (basic-design §4.5.5)
  linked_ac_ids: Uuid[]; // 关联 AcceptanceCriteria (per §27.2)
  created_at: Iso8601;
}

export interface AcceptanceCoverageReport {
  work_item_id: Uuid;
  total_count: number;
  covered_count: number;
  by_level: Record<TestLevel, number>; // per Level 覆盖数
  uncovered_by_level: Record<TestLevel, Uuid[]>; // per Level 缺哪些 AC, per REQ-TST-002
}

// =====================================================================
// 1. tenant
// =====================================================================
export interface Tenant {
  id: Uuid;
  name: string;
  slug: string;
  plan: "free" | "team" | "enterprise";
  status: "active" | "suspended" | "archived";
  created_at: Iso8601;
  seat_limit: number;
}

// =====================================================================
// 2. project
// =====================================================================
export interface Project {
  id: Uuid;
  tenant_id: Uuid;
  key: string;            // e.g. "PHYSIS"
  name: string;
  visibility: "private" | "internal" | "public";
  owner_id: Uuid;
  member_count: number;
  created_at: Iso8601;
}

// =====================================================================
// 3. identity
// =====================================================================
export type IdentityProvider =
  | "password" | "github" | "gitlab" | "google" | "saml-sso" | "local-runtime-device";

export interface Identity {
  id: Uuid;
  tenant_id: Uuid;
  email: string;
  display_name: string;
  provider: IdentityProvider;
  status: "active" | "invited" | "disabled";
  mfa_enabled: boolean;
  last_login_at?: Iso8601;
}

// =====================================================================
// 4. workspace
// =====================================================================
export interface Workspace {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  name: string;
  kind: "scratch" | "shared" | "archived";
  member_ids: Uuid[];
  default_branch_policy: "fast-forward-only" | "allow-non-ff";
}

// =====================================================================
// 5. work-item  (work_item)
// =====================================================================
// §7.2 default 3 态,可扩展 workflow 驱动多态
export type WorkItemStatus =
  | "todo" | "in_progress" | "review" | "blocked" | "done" | "wontfix";
export type WorkItemKind = "story" | "task" | "bug" | "spike" | "epic";
export type WorkItemPriority = "p0" | "p1" | "p2" | "p3";

export interface WorkItem {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  key: string;            // e.g. "PHYSIS-123"
  title: string;
  description: string;
  kind: WorkItemKind;
  status: WorkItemStatus;
  priority: WorkItemPriority;
  assignee_id?: Uuid;
  reporter_id: Uuid;
  story_points?: number;
  labels: string[];
  sprint_id?: Uuid;
  workflow_id?: Uuid;
  // W3 Calendar: optional due_date for calendar月/周视图 drag-to-reschedule
  // 由 W3 worker 2026-08-28 加入 (per dynamic-interaction-design.md §5)
  due_date?: Iso8601;
  // per 2026-08-31 12:07 JST Ulysses 拍板 (Kanban 卡 Drawer):
  //   WorkItem 跟 Worktree 是 N:1 (一个 task 可关联到一个执行 worktree, AI 创作场景)
  //   Worktree → AgentSession (1:1) → WorkItem (N:1) 三层关联
  //   关联 wt 后可在 Drawer 展示 17 状态机 + 跳 wt 详情 (Phase 2+)
  worktree_id?: Uuid;
  created_at: Iso8601;
  updated_at: Iso8601;
}

// =====================================================================
// 5b. design-artifact (per test-design.md §6.3.3 V1 Should-Have Test)
// =====================================================================
// 来源 (per 2026-08-31 wt-test-t2-dsg / AGENTS.md §0/§1.2):
//   - docs/test-design.md §6.3.3   REQ-DSG-001/002 (V1 Should-Have Test)
//   - docs/requirements.md §8.3   DesignArtifact 字段 + ReviewRecord 互斥 Target
//   - docs/requirements.md §27.4  ReviewRecord Target 字段
//                                  "ChangeSet | DesignArtifact" 二选一
//   - docs/specs/domain-work-item-spec.md  WorkItem 6 状态
//                                          (todo/in_progress/review/blocked/done/wontfix)
//
// 5 状态机:
//   draft      — 初稿,未送审
//   in_review  — 送审中,等待 reviewer 决策
//   approved   — 通过审批 (WorkItem Guard 视为"已批准")
//   rejected   — 被拒绝,需返回 draft 修订
//   superseded — 已被新版本取代 (历史版本, 视为"已批准" 不阻塞 Guard)
//
// ReviewRecord 互斥 Target (per requirements.md §27.4):
//   - 此处 review_record_id 在 DesignArtifact target 时存值
//   - ChangeSet target 时存 ChangeSet.id (互斥, 不同时存)
//   - **TBD**: basic-design §27.4 字段精确化后, 补 discriminated union 形态
//     (现以 nullable Uuid 表达, 守门 缺标比错标 安全)
export type DesignArtifactStatus =
  | "draft" | "in_review" | "approved" | "rejected" | "superseded";

export const DESIGN_ARTIFACT_STATUSES: DesignArtifactStatus[] = [
  "draft",
  "in_review",
  "approved",
  "rejected",
  "superseded",
];

export interface DesignArtifact {
  id: Uuid;
  work_item_id: Uuid;
  title: string;
  status: DesignArtifactStatus;
  version: number; // 单调递增, per REQ-DSG-001 "Version 历史" (>= 1)
  author_id: Uuid;
  created_at: Iso8601;
  updated_at: Iso8601;
  review_record_id: Uuid | null; // §27.4 Target 互斥, 此处 DesignArtifact target 时存
}

// =====================================================================
// 6. comment
// =====================================================================
export interface Comment {
  id: Uuid;
  tenant_id: Uuid;
  target_kind: "work_item" | "pr" | "context_packet" | "agent_session";
  target_id: Uuid;
  author_id: Uuid;
  body: string;
  thread_root_id?: Uuid;
  mentions: Uuid[];
  created_at: Iso8601;
  edited_at?: Iso8601;
}

// =====================================================================
// 7. permission (PermissionScheme + rules)
// =====================================================================
export type ResourceKind =
  | "project" | "work_item" | "worktree" | "agent_session" | "scm_repository" | "automation_rule";

export interface PermissionRule {
  id: Uuid;
  tenant_id: Uuid;
  scheme_id: Uuid;
  resource_kind: ResourceKind;
  action: "read" | "write" | "admin" | "delete";
  role: "tenant_admin" | "project_admin" | "developer" | "viewer" | "custom";
  effect: "allow" | "deny";
  condition?: string;     // CEL-like expr
}

export interface PermissionScheme {
  id: Uuid;
  tenant_id: Uuid;
  project_id?: Uuid;
  name: string;
  is_default: boolean;
  rule_count: number;
}

// =====================================================================
// 8. workflow
// =====================================================================
export type WorkflowStateKind = "initial" | "intermediate" | "final";

export interface WorkflowState {
  id: Uuid;
  workflow_id: Uuid;
  name: string;
  kind: WorkflowStateKind;
  category: WorkItemStatus;
  position: number;
}

export interface WorkflowTransition {
  from_state_id: Uuid;
  to_state_id: Uuid;
  trigger: string;        // "manual" | "ci_pass" | "review_approved" | ...
  guard?: string;         // CEL expression
}

export interface Workflow {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  name: string;
  states: WorkflowState[];
  transitions: WorkflowTransition[];
  is_default: boolean;
}

// =====================================================================
// 9. development (ChangeSet 5 状态机 + INV-DEV-01~05)
// =====================================================================
export type ChangeSetStatus =
  | "draft" | "applied" | "merged" | "abandoned" | "reverted";

export interface ChangeSet {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  work_item_id: Uuid;
  author_id: Uuid;
  worktree_id: Uuid;
  title: string;
  diff_summary: string;   // +/-/files
  status: ChangeSetStatus;
  symbol_index: { added: number; modified: number; removed: number };
  created_at: Iso8601;
}

// =====================================================================
// 10. worktree  ★ 17 状态机(§7.1)
// =====================================================================
export type WorktreeStatus =
  // 创建期
  | "initializing" | "cloning" | "syncing"
  // 工作期
  | "active" | "dirty" | "behind" | "diverged" | "conflict"
  // 提交期
  | "committing" | "pushing" | "ci_running" | "review_requested"
  // 完成期
  | "merged" | "closed" | "abandoned" | "archived" | "reverted";

export interface Worktree {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  name: string;
  branch: string;
  base_branch: string;
  status: WorktreeStatus;
  local_runtime_id?: Uuid;
  agent_session_id?: Uuid;
  pr_id?: Uuid;
  lock_version: number;
  last_event_at: Iso8601;
  created_at: Iso8601;
}

// =====================================================================
// 11. agent  ★ 14 状态机(§7.4)
// =====================================================================
export type AgentStatus =
  | "queued" | "spawning" | "initializing"
  | "compiling_context" | "planning" | "executing"
  | "awaiting_feedback" | "awaiting_human" | "awaiting_tool"
  | "validating" | "paused" | "completed"
  | "failed" | "cancelled";

export interface AgentSession {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  worktree_id: Uuid;
  agent_kind: "claude-sonnet" | "gpt-4o" | "codex" | "internal-vibe-coder";
  status: AgentStatus;
  current_step: string;
  token_usage: { input: number; output: number; total: number };
  cost_summary: { usd: number; budget_usd: number };
  started_at: Iso8601;
  ended_at?: Iso8601;
}

// =====================================================================
// 12. feedback  ★ 6 状态机(§7.3)
// =====================================================================
export type FeedbackStatus =
  | "open" | "acknowledged" | "in_progress" | "resolved" | "wontfix" | "reopened";

export type FeedbackSeverity = "info" | "minor" | "major" | "critical";
export type FeedbackCategory =
  | "spec_clarification" | "implementation_bug" | "test_failure"
  | "ux_issue" | "performance" | "policy_violation";

export interface Feedback {
  id: Uuid;
  tenant_id: Uuid;
  agent_session_id: Uuid;
  worktree_id: Uuid;
  status: FeedbackStatus;
  severity: FeedbackSeverity;
  category: FeedbackCategory;
  question: string;
  answer?: string;
  asked_by: Uuid;
  answered_by?: Uuid;
  asked_at: Iso8601;
  answered_at?: Iso8601;
}

// =====================================================================
// 13. context  (ContextPacket 5 字段 + Provenance + Decision 3 状态)
// =====================================================================
export type ContextPriority = "p0" | "p1" | "p2" | "p3";
export type ContextProvenance =
  | "spec_excerpt" | "user_input" | "tool_output" | "previous_decision" | "agent_inference";
export type DecisionStatus = "pending" | "approved" | "rejected";

export interface ContextPacket {
  id: Uuid;
  tenant_id: Uuid;
  agent_session_id: Uuid;
  priority: ContextPriority;
  kind: "spec" | "code" | "history" | "tool" | "decision";
  payload_ref: string;    // URI to backend storage
  token_estimate: number;
  provenance: ContextProvenance;
  decision_id?: Uuid;
  created_at: Iso8601;
}

export interface ContextDecision {
  id: Uuid;
  agent_session_id: Uuid;
  status: DecisionStatus;
  prompt: string;
  chosen_option?: string;
  decided_by?: Uuid;
  decided_at?: Iso8601;
}

// =====================================================================
// 14. validation
// =====================================================================
export type ValidationResult = "pass" | "fail" | "skipped" | "feedback_required";

export interface ValidationCase {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  work_item_id?: Uuid;
  changeset_id?: Uuid;
  name: string;
  kind: "unit" | "integration" | "contract" | "e2e" | "policy" | "security";
  result: ValidationResult;
  coverage: number;       // 0-1
  feedback_id?: Uuid;     // when result == feedback_required
  executed_at: Iso8601;
}

// =====================================================================
// 15. local-runtime
// =====================================================================
export type RuntimeStatus =
  | "registered" | "online" | "offline" | "compromised" | "revoked";

export interface LocalRuntime {
  id: Uuid;
  tenant_id: Uuid;
  device_id: Uuid;
  hostname: string;
  status: RuntimeStatus;
  bound_user_id: Uuid;
  bound_tenant_id: Uuid;
  mount_root: string;
  last_heartbeat_at: Iso8601;
  policy_violations: number;
}

// =====================================================================
// 16. scm  ★ PR 7 状态机(§7.5) + Webhook Idempotency
// =====================================================================
export type PullRequestStatus =
  | "draft" | "open" | "ci_failed" | "review_required"
  | "approved" | "merged" | "closed";

export interface Repository {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  provider: "github" | "gitlab" | "gitea" | "self-host";
  full_name: string;
  default_branch: string;
  webhook_idempotency_key?: string;
  last_event_at?: Iso8601;
}

export interface PullRequest {
  id: Uuid;
  tenant_id: Uuid;
  repository_id: Uuid;
  number: number;
  title: string;
  author_id: Uuid;
  source_branch: string;
  target_branch: string;
  status: PullRequestStatus;
  review_state: "none" | "changes_requested" | "approved";
  ci_state: "none" | "pending" | "passing" | "failing";
  created_at: Iso8601;
  merged_at?: Iso8601;
}

// =====================================================================
// 17. notification (INV-N-07 抑制)
// =====================================================================
export type NotificationKind =
  | "agent_decision_required" | "feedback_question"
  | "ci_failed" | "review_requested" | "merge_conflict"
  | "budget_alert" | "policy_violation";
export type NotificationChannel = "inbox" | "email" | "im" | "suppressed";
export type NotificationStatus = "pending" | "delivered" | "suppressed" | "read";

export interface Notification {
  id: Uuid;
  tenant_id: Uuid;
  recipient_id: Uuid;
  kind: NotificationKind;
  channel: NotificationChannel;
  status: NotificationStatus;
  subject: string;
  body: string;
  ref_kind?: string;
  ref_id?: Uuid;
  suppression_reason?: string;   // INV-N-07
  created_at: Iso8601;
}

// =====================================================================
// 18. search (Projection + tenant 隔离)
// =====================================================================
export interface SearchHit {
  id: Uuid;
  kind: TenantScopedKind;
  tenant_id: Uuid;
  title: string;
  snippet: string;
  score: number;
}

export interface SavedSearch {
  id: Uuid;
  tenant_id: Uuid;
  name: string;
  query: string;
  filters: Record<string, unknown>;
  created_by: Uuid;
}

// =====================================================================
// 19. integration (Loop 防护)
// =====================================================================
export type IntegrationKind =
  | "github" | "gitlab" | "jira" | "slack" | "lark" | "linear" | "webhook";

export interface Integration {
  id: Uuid;
  tenant_id: Uuid;
  kind: IntegrationKind;
  display_name: string;
  status: "active" | "paused" | "error" | "circuit_open";
  config_masked: string;        // 显示脱敏后的配置
  loop_protection_key?: string; // 用于去重,避免 webhook 风暴
  last_sync_at?: Iso8601;
  error_count_24h: number;
}

// =====================================================================
// 20. collaboration (Presence + Whiteboard)
// =====================================================================
export interface PresenceCursor {
  user_id: Uuid;
  workspace_id: Uuid;
  x: number;
  y: number;
  selection?: string;
  updated_at: Iso8601;
}

export interface Whiteboard {
  id: Uuid;
  tenant_id: Uuid;
  workspace_id: Uuid;
  title: string;
  collaborator_ids: Uuid[];
  snapshot_url: string;
  updated_at: Iso8601;
}

// =====================================================================
// Canvas(无限画布)— 来自 frontend-canvas-design.md v0.1
// 替代 Whiteboard 作为 collaboration 域主入口
// 保留 Whiteboard 实体(向后兼容),新增 Canvas / CanvasElement / CanvasConnector
// =====================================================================

/** Canvas 视口状态 */
export interface CanvasViewport {
  x: number;
  y: number;
  zoom: number; // 0.1 ~ 4
}

/** Canvas Frame(画布分区,可作 slide) */
export interface CanvasFrame {
  id: Uuid;
  canvas_id: Uuid;
  title: string;
  x: number;
  y: number;
  width: number;
  height: number;
  element_ids: Uuid[];
  is_slide: boolean;
  order: number;
}

/** Canvas 主体 */
export interface Canvas {
  id: Uuid;
  tenant_id: Uuid;
  workspace_id: Uuid;
  title: string;
  ref_kind?: "work_item" | "worktree" | "project" | "free";
  ref_id?: Uuid;
  viewport: CanvasViewport;
  frames: CanvasFrame[];
  creator_id: Uuid;
  collaborator_ids: Uuid[];
  created_at: Iso8601;
  updated_at: Iso8601;
  snapshot_url?: string;
}

/** Canvas 元素类型 */
export type CanvasElementKind =
  | "sticky_note"
  | "text"
  | "shape"
  | "image"
  | "embed"
  | "work_item_card"
  | "worktree_node"
  | "agent_cursor"
  | "automation_node"
  | "comment_pin";

/** Canvas 元素 */
export interface CanvasElement {
  id: Uuid;
  canvas_id: Uuid;
  kind: CanvasElementKind;
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  z_index: number;
  content: {
    text?: string;
    color?: string;
    image_url?: string;
    embed_url?: string;
    work_item_id?: Uuid;
    worktree_id?: Uuid;
    agent_session_id?: Uuid;
    automation_id?: Uuid;
    comment_id?: Uuid;
  };
  locked: boolean;
  hidden: boolean;
  created_by: Uuid;
  created_at: Iso8601;
  updated_at: Iso8601;
}

/** Canvas 连接线 */
export type CanvasConnectorKind =
  | "work_item_relation"
  | "agent_handoff"
  | "free"
  | "dependency";

export interface CanvasConnector {
  id: Uuid;
  canvas_id: Uuid;
  kind: CanvasConnectorKind;
  from_element_id: Uuid;
  to_element_id: Uuid;
  routing: "straight" | "curved" | "orthogonal";
  arrow_start: boolean;
  arrow_end: boolean;
  color: string;
  width: number;
  label?: string;
  relation_id?: Uuid;
}

/** Canvas 实时视口(每用户) */
export interface CanvasViewportState {
  canvas_id: Uuid;
  user_id: Uuid;
  x: number;
  y: number;
  zoom: number;
  selected_element_ids: Uuid[];
  updated_at: Iso8601;
}

// =====================================================================
// 21. planning (Sprint + Milestone + Burndown)
// =====================================================================
export type SprintStatus = "planned" | "active" | "completed" | "cancelled";

export interface Sprint {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  name: string;
  goal: string;
  status: SprintStatus;
  start_date: Iso8601;
  end_date: Iso8601;
  capacity_points: number;
  committed_points: number;
  completed_points: number;
}

export interface Milestone {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  name: string;
  due_date: Iso8601;
  work_item_ids: Uuid[];
  progress: number;     // 0-1
}

export interface BurndownPoint {
  date: Iso8601;
  remaining_points: number;
  ideal_points: number;
}

// =====================================================================
// 22. board (Kanban + WIP limit)
// =====================================================================
export interface Board {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  name: string;
  columns: Array<{
    status: WorkItemStatus;
    /** 自定义列名 (per 2026-08-29 18:52 JST 拍板); undefined 时回退到 status */
    name?: string;
    work_item_ids: Uuid[];
    wip_limit?: number;
  }>;
}

// =====================================================================
// 23. relation (graph + BFS)
// =====================================================================
export type RelationKind =
  | "blocks" | "duplicates" | "relates_to" | "parent_of" | "cloned_from";

export interface Relation {
  id: Uuid;
  tenant_id: Uuid;
  from_kind: "work_item" | "worktree" | "agent_session" | "changeset";
  from_id: Uuid;
  to_kind: "work_item" | "worktree" | "agent_session" | "changeset";
  to_id: Uuid;
  kind: RelationKind;
  created_at: Iso8601;
}

// =====================================================================
// 24. audit (Append-only + 9 AI Q + cross-tenant 100%)
// =====================================================================
export type AuditCategory =
  | "auth" | "permission" | "data_access" | "config_change"
  | "ai_decision" | "policy_violation" | "integration" | "system" | "billing";

export interface AuditEvent {
  id: Uuid;
  tenant_id: Uuid;
  actor_id: Uuid;
  category: AuditCategory;
  action: string;          // "workitem.create" / "agent.start" ...
  target_kind?: string;
  target_id?: Uuid;
  payload: Record<string, unknown>;
  ai_metadata?: {          // 9 AI 问题的元数据
    agent_session_id?: Uuid;
    prompt_hash?: string;
    decision_id?: Uuid;
    confidence?: number;
  };
  prev_hash: string;       // append-only 链
  hash: string;
  created_at: Iso8601;
}

// =====================================================================
// 25. automation (Rule + Trigger + Condition + Action + 6 INV)
// =====================================================================
export type AutomationTriggerKind =
  | "workitem_status_changed" | "pr_status_changed"
  | "agent_session_completed" | "schedule_cron"
  | "feedback_received" | "audit_event";

export type AutomationActionKind =
  | "assign_user" | "set_label" | "send_notification"
  | "create_worktree" | "call_webhook" | "dispatch_agent";

export interface AutomationRule {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  name: string;
  enabled: boolean;
  trigger_kind: AutomationTriggerKind;
  trigger_filter: Record<string, unknown>;
  condition_expr?: string;     // CEL
  actions: Array<{
    kind: AutomationActionKind;
    config: Record<string, unknown>;
  }>;
  execution_count_24h: number;
  last_fired_at?: Iso8601;
}

// =====================================================================
// 状态机迁移规则(用于状态机可视化页)
// =====================================================================
export interface StateMachineTransition {
  from: string;
  to: string;
  trigger: string;
  guard?: string;
}

export interface StateMachine {
  name: string;
  states: string[];
  initial: string;
  transitions: StateMachineTransition[];
  invariant_ids: string[];    // 相关 INV
}

export const WORKTREE_SM: StateMachine = {
  name: "Worktree 17 状态机",
  states: [
    "initializing", "cloning", "syncing",
    "active", "dirty", "behind", "diverged", "conflict",
    "committing", "pushing", "ci_running", "review_requested",
    "merged", "closed", "abandoned", "archived", "reverted",
  ],
  initial: "initializing",
  invariant_ids: ["INV-WT-01", "INV-WT-02", "INV-WT-03", "INV-WT-04"],
  transitions: [
    { from: "initializing", to: "cloning", trigger: "git.clone.start" },
    { from: "cloning",      to: "syncing", trigger: "git.clone.done" },
    { from: "syncing",      to: "active",  trigger: "sync.complete" },
    { from: "active",       to: "dirty",   trigger: "file.modified" },
    { from: "active",       to: "behind",  trigger: "remote.advanced" },
    { from: "behind",       to: "diverged",trigger: "local.commit.pushed" },
    { from: "diverged",     to: "conflict",trigger: "merge.attempt" },
    { from: "conflict",     to: "active",  trigger: "conflict.resolved" },
    { from: "dirty",        to: "committing", trigger: "user.commit" },
    { from: "committing",   to: "pushing", trigger: "commit.complete" },
    { from: "pushing",      to: "ci_running", trigger: "push.complete" },
    { from: "ci_running",   to: "review_requested", trigger: "ci.pass" },
    { from: "ci_running",   to: "dirty",  trigger: "ci.fail" },
    { from: "review_requested", to: "merged", trigger: "pr.approved.merge" },
    { from: "review_requested", to: "closed", trigger: "pr.closed" },
    { from: "merged",       to: "reverted", trigger: "revert.commit" },
    { from: "merged",       to: "archived", trigger: "user.archive" },
    { from: "active",       to: "abandoned", trigger: "user.abandon" },
  ],
};

export const AGENT_SM: StateMachine = {
  name: "AgentSession 14 状态机",
  states: [
    "queued", "spawning", "initializing",
    "compiling_context", "planning", "executing",
    "awaiting_feedback", "awaiting_human", "awaiting_tool",
    "validating", "paused", "completed", "failed", "cancelled",
  ],
  initial: "queued",
  invariant_ids: ["INV-AGT-N01", "INV-AGT-N02", "INV-AGT-N07", "INV-AGT-N14"],
  transitions: [
    { from: "queued",            to: "spawning",          trigger: "scheduler.dispatch" },
    { from: "spawning",          to: "initializing",      trigger: "runtime.spawn.ok" },
    { from: "initializing",      to: "compiling_context", trigger: "init.complete" },
    { from: "compiling_context", to: "planning",          trigger: "context.ready" },
    { from: "planning",          to: "executing",         trigger: "plan.approved" },
    { from: "executing",         to: "awaiting_feedback", trigger: "feedback.request" },
    { from: "awaiting_feedback", to: "executing",         trigger: "feedback.received" },
    { from: "executing",         to: "awaiting_human",    trigger: "human.decision.required" },
    { from: "awaiting_human",    to: "executing",         trigger: "human.decided" },
    { from: "executing",         to: "awaiting_tool",     trigger: "tool.call" },
    { from: "awaiting_tool",     to: "executing",         trigger: "tool.returned" },
    { from: "executing",         to: "validating",        trigger: "agent.done" },
    { from: "validating",        to: "completed",         trigger: "validation.pass" },
    { from: "validating",        to: "failed",            trigger: "validation.fail" },
    { from: "executing",         to: "paused",            trigger: "user.pause" },
    { from: "paused",            to: "executing",         trigger: "user.resume" },
    { from: "queued",            to: "cancelled",         trigger: "user.cancel" },
    { from: "executing",         to: "cancelled",         trigger: "user.cancel" },
  ],
};

export const FEEDBACK_SM: StateMachine = {
  name: "Feedback 6 状态机",
  states: ["open", "acknowledged", "in_progress", "resolved", "wontfix", "reopened"],
  initial: "open",
  invariant_ids: ["INV-FB-01", "INV-FB-02"],
  transitions: [
    { from: "open",         to: "acknowledged", trigger: "human.ack" },
    { from: "acknowledged", to: "in_progress",  trigger: "agent.start" },
    { from: "in_progress",  to: "resolved",     trigger: "fix.deployed" },
    { from: "in_progress",  to: "wontfix",      trigger: "user.wontfix" },
    { from: "resolved",     to: "reopened",     trigger: "regression.detected" },
    { from: "reopened",     to: "in_progress",  trigger: "agent.repick" },
  ],
};

export const PR_SM: StateMachine = {
  name: "PR 7 状态机",
  states: [
    "draft", "open", "ci_failed", "review_required",
    "approved", "merged", "closed",
  ],
  initial: "draft",
  invariant_ids: ["INV-SCM-05", "INV-SCM-06", "INV-SCM-07", "INV-SCM-08"],
  transitions: [
    { from: "draft",           to: "open",             trigger: "ready_for_review" },
    { from: "open",            to: "ci_failed",        trigger: "ci.fail" },
    { from: "ci_failed",       to: "open",             trigger: "fix.push" },
    { from: "open",            to: "review_required",  trigger: "review.requested" },
    { from: "review_required", to: "approved",         trigger: "review.approved" },
    { from: "review_required", to: "open",             trigger: "review.changes_requested" },
    { from: "approved",        to: "merged",           trigger: "user.merge" },
    { from: "open",            to: "closed",           trigger: "user.close" },
  ],
};

export const WORKITEM_SM: StateMachine = {
  name: "WorkItem 6 状态 (默认 3 态 + 扩展)",
  states: ["todo", "in_progress", "review", "blocked", "done", "wontfix"],
  initial: "todo",
  invariant_ids: ["INV-PM-01", "INV-PM-02", "INV-PM-03"],
  transitions: [
    { from: "todo",        to: "in_progress", trigger: "user.start" },
    { from: "in_progress", to: "review",      trigger: "pr.opened" },
    { from: "in_progress", to: "blocked",     trigger: "blocker.detected" },
    { from: "blocked",     to: "in_progress", trigger: "blocker.cleared" },
    { from: "review",      to: "in_progress", trigger: "review.changes_requested" },
    { from: "review",      to: "done",        trigger: "pr.merged" },
    { from: "in_progress", to: "wontfix",     trigger: "user.wontfix" },
  ],
};

export const CHANGESET_SM: StateMachine = {
  name: "ChangeSet 5 状态机 (INV-DEV-01~05)",
  states: ["draft", "applied", "merged", "abandoned", "reverted"],
  initial: "draft",
  invariant_ids: ["INV-DEV-01", "INV-DEV-02", "INV-DEV-03", "INV-DEV-04", "INV-DEV-05"],
  transitions: [
    { from: "draft",     to: "applied",   trigger: "user.apply" },
    { from: "applied",   to: "merged",    trigger: "pr.merged" },
    { from: "applied",   to: "reverted",  trigger: "user.revert" },
    { from: "draft",     to: "abandoned", trigger: "user.abandon" },
    { from: "merged",    to: "reverted",  trigger: "user.revert" },
  ],
};
