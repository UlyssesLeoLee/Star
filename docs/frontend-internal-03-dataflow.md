# Star 平台《Frontend Internal Design 03 — 数据流与 Realtime》

> **文档版本**: v0.1 (2026-08-26)
> **修订历史**:
>
> | 版本 | 日期 | 变更 | 审批者 |
> |---|---|---|---|
> | v0.1 | 2026-08-26 | 初始版本(25 Module 字段投影 + 6 类错误码 + 25↔NATS 表 + 5 新 ADR) | — |
>
> **上游 frontend-design**: `D:\Star\docs\frontend-design.md` v0.1 §6 / §7
> **上游 api-design**: `D:\Star\docs\api-design.md` v0.2 §3 / §4 / §5.5 / §5.6 / §8
> **上游 basic-design**: `D:\Star\docs\basic-design.md` v0.1 §6(安全边界)
> **已实施现状**: `D:\Star\frontend\src\types\ids.ts`(25 domain TS type)+ `lib/{store,seed}.ts`
> **4 份 frontend-internal 之三**: 01-架构 / 02-组件 / 03-数据流 / 04-交互

---

## 0. 文档说明

### 0.1 目的

继承 frontend-design §6(数据流契约)+ §7(Realtime 通道),做 Internal Design 级别的:
- **25 Module 字段投影表**(8 节,每节 3-5 module,**必须完整 25 module**)
- Tenant 强制 / Local Runtime 三重绑定 / 权限视图 / Secret 脱敏 / Loop 防护 5 个数据契约
- Realtime 25 Module ↔ NATS Subject 完整映射
- 6 类错误码 → UI 反馈映射
- 3 个数据流时序图
- 5 项新 ADR(ADR-FE-016~020)

### 0.2 引用关系

| 引用本文 | 位置 |
|---|---|
| frontend-internal-01 §1.5 | BFF 职责(本文 §1.4 跨模块聚合展开) |
| frontend-internal-01 §2.2 | Store 分层(本文 §1.1 type 投影) |
| frontend-internal-02 §3.6 | 6 SM 复用(本文 §5 字段含 status) |

---

## 1. 25 Module 字段投影表

### 1.1 通用 TS type(继承 types/ids.ts)

```ts
export type Uuid = string;
export type Iso8601 = string; // "2026-08-26T11:30:00Z"

export interface ActorContext {
  user_id: Uuid;
  tenant_id: Uuid;
  device_id?: Uuid;
  project_ids: Uuid[];
  roles: Array<"tenant_admin" | "project_admin" | "developer" | "viewer">;
}

export type TenantScopedKind =
  | "tenant" | "project" | "workspace" | "identity" | "permission"
  | "work_item" | "comment" | "worktree" | "agent_session"
  | "audit_event" | "automation_rule" | "scm_repository" | "notification";

export type ModuleName =
  | "worktree" | "feedback" | "validation" | "integration" | "scm"
  | "agent" | "context" | "notification" | "search"
  | "tenant" | "project" | "identity" | "work-item" | "comment"
  | "permission" | "workflow" | "development"
  | "collaboration" | "planning" | "board" | "local-runtime" | "relation"
  | "workspace" | "audit" | "automation";
```

### 1.2 字段投影矩阵总览

| # | Module | TS interface | 字段数 | INV 引用 | REST 端点(api-design) |
|---|---|---|---|---|---|
| 1 | tenant | `Tenant` | 8 | REQ-SEC-001 | §3.2 |
| 2 | workspace | `Workspace` | 7 | REQ-SEC-001 | §3.3 |
| 3 | project | `Project` | 7 | REQ-SEC-001 | §3.4 |
| 4 | identity | `Identity` | 8 | REQ-SEC-001 | §3.15 |
| 5 | work-item | `WorkItem` | 17 | INV-PM-01~05 | §3.5 |
| 6 | comment | `Comment` | 9 | REQ-SEC-001 | §3.10 |
| 7 | workflow | `Workflow` | 5 | REQ-WF-001 | §3.6 |
| 8 | permission | `PermissionScheme` + `PermissionRule` | 6 + 8 | REQ-SEC-002 | §3.17 |
| 9 | development | `ChangeSet` | 12 | INV-DEV-01~05 | §3.20 |
| 10 | planning | `Sprint` / `Milestone` | 10 / 9 | REQ-PLAN-001 | §3.8 |
| 11 | board | `Board` | 4 | REQ-BOARD-001 | §3.7 |
| 12 | worktree | `Worktree` | 13 | INV-WT-01~04 | §3.21 |
| 13 | agent | `AgentSession` | 11 | INV-AGT-N01~N14 | §3.22 |
| 14 | feedback | `Feedback` | 13 | INV-FB-01~02 | §3.23 |
| 15 | context | `ContextPacket` + `ContextDecision` | 9 + 6 | INV-CT-01~10 | §3.24 |
| 16 | validation | `ValidationCase` | 10 | REQ-VAL-001 | §3.25 |
| 17 | scm | `Repository` + `PullRequest` | 6 + 12 | INV-SCM-01~08 | §3.19 |
| 18 | integration | `Integration` | 9 | REQ-INT-001 | §3.13 |
| 19 | notification | `Notification` | 12 | INV-N-07 | §3.16 |
| 20 | search | `SearchHit` + `SavedSearch` | 6 + 6 | INV-SR-01/02 | §3.11 |
| 21 | local-runtime | `LocalRuntime` | 10 | INV-LR-01~05 | §3.26 |
| 22 | collaboration | `PresenceCursor` + `Whiteboard` | 5 + 7 | REQ-RT-003 | §3.18 |
| 23 | audit | `AuditEvent` | 14 | INV-AU-01~07 | §3.12 |
| 24 | automation | `AutomationRule` | 11 | INV-AUTO-01~06 | §3.14 |
| 25 | relation | `Relation` | 8 | REQ-COLLAB-002 | §3.9 |

### 1.3 Foundational 5 Module 字段

#### 1. tenant (api-design §3.2)

```ts
interface Tenant {
  id: Uuid;                  // PK
  name: string;              // "ACME Studio"
  slug: string;              // "acme",URL-safe
  plan: "free" | "team" | "enterprise";
  status: "active" | "suspended" | "archived";
  created_at: Iso8601;
  seat_limit: number;
}
```
**INV 引用**: REQ-SEC-001(13 类 tenant_id 必带对象的根)

#### 2. workspace (api-design §3.3)

```ts
interface Workspace {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;           // 关联 project
  name: string;
  kind: "scratch" | "shared" | "archived";
  member_ids: Uuid[];         // 成员列表
  default_branch_policy: "fast-forward-only" | "allow-non-ff";
}
```

#### 3. project (api-design §3.4)

```ts
interface Project {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  key: string;                // "PHYSIS",work-item key 前缀
  name: string;
  visibility: "private" | "internal" | "public";
  owner_id: Uuid;
  member_count: number;
  created_at: Iso8601;
}
```

#### 4. identity (api-design §3.15)

```ts
interface Identity {
  id: Uuid;                   // user_id
  tenant_id: Uuid;
  email: string;
  display_name: string;
  provider: "password" | "github" | "gitlab" | "google" | "saml-sso" | "local-runtime-device";
  status: "active" | "invited" | "disabled";
  mfa_enabled: boolean;
  last_login_at?: Iso8601;
}
```

#### 5. work-item (api-design §3.5)

```ts
interface WorkItem {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;           // 关联 project
  key: string;                // "PHYSIS-123"
  title: string;
  description: string;
  kind: "story" | "task" | "bug" | "spike" | "epic" | "subtask";
  status: "todo" | "in_progress" | "review" | "blocked" | "done" | "wontfix";
  priority: "p0" | "p1" | "p2" | "p3";
  assignee_id?: Uuid;
  reporter_id: Uuid;
  story_points?: number;
  labels: string[];           // REQ-WI-001 自由文本分类
  components: string[];       // REQ-WI-001 Repository Scope 粗粒度
  sprint_id?: Uuid;
  workflow_id?: Uuid;         // 关联 workflow
  created_at: Iso8601;
  updated_at: Iso8601;
}
```

### 1.4 Work Management 5 Module 字段

#### 6. comment (api-design §3.10)

```ts
interface Comment {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  target_kind: "work_item" | "pr" | "context_packet" | "agent_session";
  target_id: Uuid;            // 跨实体引用
  author_id: Uuid;
  body: string;
  thread_root_id?: Uuid;      // 嵌套回复
  mentions: Uuid[];           // @ 通知
  created_at: Iso8601;
  edited_at?: Iso8601;
}
```

#### 7. workflow (api-design §3.6)

```ts
interface WorkflowState {
  id: Uuid;
  workflow_id: Uuid;
  name: string;
  kind: "initial" | "intermediate" | "final";
  category: WorkItemStatus;   // 映射到 work-item status
  position: number;
}

interface WorkflowTransition {
  from_state_id: Uuid;
  to_state_id: Uuid;
  trigger: string;            // "user.start" / "pr.merged" / "ci.pass"
  guard?: string;             // CEL 表达式(REQ-WF-003)
}

interface Workflow {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  name: string;
  states: WorkflowState[];
  transitions: WorkflowTransition[];
  is_default: boolean;
}
```

#### 8. permission (api-design §3.17)

```ts
interface PermissionRule {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  scheme_id: Uuid;            // 关联 scheme
  resource_kind: "project" | "work_item" | "worktree" | "agent_session" | "scm_repository" | "automation_rule";
  action: "read" | "write" | "admin" | "delete";
  role: "tenant_admin" | "project_admin" | "developer" | "viewer" | "custom";
  effect: "allow" | "deny";
  condition?: string;         // CEL 表达式
}

interface PermissionScheme {
  id: Uuid;
  tenant_id: Uuid;
  project_id?: Uuid;
  name: string;
  is_default: boolean;
  rule_count: number;
}
```

#### 9. development (api-design §3.20)

```ts
interface ChangeSet {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;
  work_item_id: Uuid;         // 关联 work-item
  author_id: Uuid;
  worktree_id: Uuid;          // 关联 worktree
  title: string;
  diff_summary: string;       // "+342 / -18 / 4 files"
  status: "draft" | "applied" | "merged" | "abandoned" | "reverted";
  symbol_index: { added: number; modified: number; removed: number };
  created_at: Iso8601;
}
```

#### 10. planning (api-design §3.8)

```ts
interface Sprint {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;
  name: string;                // "Sprint 23"
  goal: string;
  status: "planned" | "active" | "completed" | "cancelled";
  start_date: Iso8601;
  end_date: Iso8601;
  capacity_points: number;
  committed_points: number;
  completed_points: number;
}

interface Milestone {           // REQ-PLAN-007
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  name: string;
  description: string;
  due_date: Iso8601;
  status: MilestoneStatus;
  work_item_ids: Uuid[];
  created_at: Iso8601;
}
```

### 1.5 Worktree/Agent 5 Module 字段

#### 11. worktree (api-design §3.21)

```ts
interface Worktree {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;
  name: string;                // "wt-worktree-sm"
  branch: string;              // "feat/worktree-sm"
  base_branch: string;         // "main"
  status: WorktreeStatus;      // 17 状态枚举
  local_runtime_id?: Uuid;     // 关联 local-runtime
  agent_session_id?: Uuid;     // 关联 agent
  pr_id?: Uuid;                // 关联 scm
  lock_version: number;        // 乐观锁
  last_event_at: Iso8601;
  created_at: Iso8601;
}
```

#### 12. agent (api-design §3.22)

```ts
interface AgentSession {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;
  worktree_id: Uuid;          // 1:1 关联 worktree
  agent_kind: "claude-sonnet" | "gpt-4o" | "codex" | "internal-vibe-coder";
  status: AgentStatus;         // 14 状态枚举
  current_step: string;        // "tool.call:grep"
  token_usage: { input: number; output: number; total: number };
  cost_summary: { usd: number; budget_usd: number };
  started_at: Iso8601;
  ended_at?: Iso8601;
}
```

#### 13. feedback (api-design §3.23)

```ts
interface Feedback {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  agent_session_id: Uuid;     // 关联 agent
  worktree_id: Uuid;          // 关联 worktree
  status: FeedbackStatus;      // 6 状态枚举
  severity: "info" | "minor" | "major" | "critical";
  category: "spec_clarification" | "implementation_bug" | "test_failure" | "ux_issue" | "performance" | "policy_violation";
  question: string;
  answer?: string;
  asked_by: Uuid;
  answered_by?: Uuid;
  asked_at: Iso8601;
  answered_at?: Iso8601;
}
```

#### 14. context (api-design §3.24)

```ts
interface ContextPacket {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  agent_session_id: Uuid;     // 关联 agent
  priority: "p0" | "p1" | "p2" | "p3";
  kind: "spec" | "code" | "history" | "tool" | "decision";
  payload_ref: string;         // URI to backend storage
  token_estimate: number;
  provenance: "spec_excerpt" | "user_input" | "tool_output" | "previous_decision" | "agent_inference";
  decision_id?: Uuid;          // 关联 decision
  created_at: Iso8601;
}

interface ContextDecision {
  id: Uuid;
  agent_session_id: Uuid;
  status: "pending" | "approved" | "rejected";
  prompt: string;
  chosen_option?: string;
  decided_by?: Uuid;
  decided_at?: Iso8601;
}
```

#### 15. validation (api-design §3.25)

```ts
interface ValidationCase {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;
  work_item_id?: Uuid;
  changeset_id?: Uuid;
  name: string;
  kind: "unit" | "integration" | "contract" | "e2e" | "policy" | "security";
  result: "pass" | "fail" | "skipped" | "feedback_required";
  coverage: number;            // 0-1
  feedback_id?: Uuid;          // when result == feedback_required
  executed_at: Iso8601;
}
```

### 1.6 Integration & Search 4 Module 字段

#### 16. scm (api-design §3.19)

```ts
interface Repository {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;
  provider: "github" | "gitlab" | "gitea" | "self-host";
  full_name: string;           // "acme/physis"
  default_branch: string;
  webhook_idempotency_key?: string;  // INV-SCM-08
  last_event_at?: Iso8601;
}

interface PullRequest {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  repository_id: Uuid;
  number: number;              // PR #101
  title: string;
  author_id: Uuid;
  source_branch: string;
  target_branch: string;
  status: PullRequestStatus;   // 7 状态枚举
  review_state: "none" | "changes_requested" | "approved";
  ci_state: "none" | "pending" | "passing" | "failing";
  created_at: Iso8601;
  merged_at?: Iso8601;
}
```

#### 17. integration (api-design §3.13)

```ts
interface Integration {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  kind: "github" | "gitlab" | "jira" | "slack" | "lark" | "linear" | "webhook";
  display_name: string;
  status: "active" | "paused" | "error" | "circuit_open";
  config_masked: string;       // 已脱敏
  loop_protection_key?: string;  // INV 防止风暴
  last_sync_at?: Iso8601;
  error_count_24h: number;
}
```

#### 18. notification (api-design §3.16)

```ts
interface Notification {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  recipient_id: Uuid;
  kind: "agent_decision_required" | "feedback_question" | "ci_failed" | "review_requested" | "merge_conflict" | "budget_alert" | "policy_violation";
  channel: "inbox" | "email" | "im" | "suppressed";
  status: "pending" | "delivered" | "suppressed" | "read";
  subject: string;
  body: string;
  ref_kind?: string;
  ref_id?: Uuid;
  suppression_reason?: string;  // INV-N-07
  created_at: Iso8601;
}
```

#### 19. search (api-design §3.11)

```ts
interface SearchHit {
  id: Uuid;
  kind: TenantScopedKind;     // 13 类必带
  tenant_id: Uuid;            // INV-SR-01 强制
  title: string;
  snippet: string;
  score: number;              // 0-1
}

interface SavedSearch {
  id: Uuid;
  tenant_id: Uuid;
  name: string;
  query: string;
  filters: Record<string, unknown>;
  created_by: Uuid;
}
```

### 1.7 Runtime/Collaboration 4 Module 字段

#### 20. local-runtime (api-design §3.26)

```ts
interface LocalRuntime {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  device_id: Uuid;            // 设备指纹(三重绑定 LRT-001)
  hostname: string;
  status: "registered" | "online" | "offline" | "compromised" | "revoked";
  bound_user_id: Uuid;        // 三重绑定 LRT-002
  bound_tenant_id: Uuid;      // 三重绑定
  mount_root: string;         // 必须在 policy.allowlist
  last_heartbeat_at: Iso8601;
  policy_violations: number;
}
```

#### 21. collaboration (api-design §3.18)

```ts
interface PresenceCursor {
  user_id: Uuid;
  workspace_id: Uuid;
  x: number;
  y: number;
  selection?: string;         // 当前选中文件
  updated_at: Iso8601;
}

interface Whiteboard {
  id: Uuid;
  tenant_id: Uuid;
  workspace_id: Uuid;
  title: string;
  collaborator_ids: Uuid[];
  snapshot_url: string;
  updated_at: Iso8601;
}
```

#### 22. audit (api-design §3.12)

```ts
interface AuditEvent {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  actor_id: Uuid;
  category: "auth" | "permission" | "data_access" | "config_change" | "ai_decision" | "policy_violation" | "integration" | "system" | "billing";
  action: string;              // "workitem.create" / "agent.start"
  target_kind?: string;
  target_id?: Uuid;
  payload: Record<string, unknown>;
  ai_metadata?: {              // 9 AI 问题的元数据
    agent_session_id?: Uuid;
    prompt_hash?: string;
    decision_id?: Uuid;
    confidence?: number;
  };
  prev_hash: string;           // append-only 链
  hash: string;                // 当前 hash
  created_at: Iso8601;
}
```

#### 23. automation (api-design §3.14)

```ts
interface AutomationRule {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;
  name: string;
  enabled: boolean;
  trigger_kind: "workitem_status_changed" | "pr_status_changed" | "agent_session_completed" | "schedule_cron" | "feedback_received" | "audit_event";
  trigger_filter: Record<string, unknown>;
  condition_expr?: string;     // CEL
  actions: Array<{
    kind: "assign_user" | "set_label" | "send_notification" | "create_worktree" | "call_webhook" | "dispatch_agent";
    config: Record<string, unknown>;
  }>;
  execution_count_24h: number;
  last_fired_at?: Iso8601;
}
```

### 1.8 Meta 2 Module 字段

#### 24. relation (api-design §3.9)

```ts
interface Relation {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  from_kind: "work_item" | "worktree" | "agent_session" | "changeset";
  from_id: Uuid;
  to_kind: "work_item" | "worktree" | "agent_session" | "changeset";
  to_id: Uuid;
  kind: "blocks" | "duplicates" | "relates_to" | "parent_of" | "cloned_from";
  created_at: Iso8601;
}
```

#### 25. board (api-design §3.7)

```ts
interface Board {
  id: Uuid;
  tenant_id: Uuid;            // 13 类必带
  project_id: Uuid;
  name: string;
  columns: Array<{
    status: WorkItemStatus;   // 映射 work-item status
    work_item_ids: Uuid[];    // 排序后
    wip_limit?: number;        // 99 = 无限
  }>;
}
```

---

## 2. Tenant 强制(继承 frontend-design §6.1)

### 2.1 tenant context 类型

**全部 25 Module 都有 `tenant_id: Uuid` 必带字段**(上面 §1.2-1.8 已确认)。

### 2.2 13 类 tenant_id 必带对象清单(继承 basic-design §6.1)

| # | 类型 | 是否在 25 Module 中 |
|---|---|---|
| 1 | tenant | ✓ |
| 2 | project | ✓ |
| 3 | workspace | ✓ |
| 4 | identity | ✓ |
| 5 | permission | ✓ (PermissionScheme) |
| 6 | work_item | ✓ |
| 7 | comment | ✓ |
| 8 | worktree | ✓ |
| 9 | agent_session | ✓ |
| 10 | audit_event | ✓ |
| 11 | automation_rule | ✓ |
| 12 | scm_repository | ✓ |
| 13 | notification | ✓ |

**13 / 13 全部对应 25 module 中的具体类型**。

### 2.3 Topbar tenant switcher 实现(只读)

- `frontend/src/components/Topbar.tsx` 第 7-9 行:显示当前 tenant 名称("ACME Studio")+ chevron
- **禁止** 切换 tenant(切换属 admin 操作,需走 admin panel)
- 显示当前 project 名称("Physis / GVPE")+ chevron

### 2.4 错误反馈文案(含 tenant 名)

```ts
// SEC-001 cross-tenant 错误的 UI 表现
toast.error(`无权访问 tenant '${actor.tenant_id}' 的资源`);

// 或 banner
banner.error(`Cross-tenant 拒绝:当前 session tenant='${actor.tenant_id}' 但目标资源属于其他 tenant`);
```

---

## 3. Local Runtime 三重绑定(继承 frontend-design §6.2)

### 3.1 三重绑定类型

| 字段 | 来源 | INV |
|---|---|---|
| `device_id` | 设备指纹(TPM / Secure Enclave) | LRT-001 |
| `tenant_id` | 登录用户 tenant | LRT-002 |
| `user_id` | 设备登录态 | LRT-002 |

### 3.2 mount_root policy.allowlist 检查

```ts
// frontend 展示(后端强制)
interface LocalRuntime {
  mount_root: string;  // eg "/Users/ulysses/dev"
  // 不在 allowlist 时 status=compromised + audit.policy_violation
}
```

### 3.3 mismatch 触发的 UI 表现

- `status=compromised` 整行 err 色
- badge "❌ 设备失绑"
- 点击查看 mismatch 详情(audit.policy_violation 链接)

---

## 4. 权限视图(继承 frontend-design §6.4)

### 4.1 PermissionGate 组件 Props

```ts
interface PermissionGateProps {
  /** 受保护资源种类 */
  resource: "work_item" | "worktree" | "agent_session" | "automation_rule" | "project";
  /** 操作类型 */
  action: "read" | "write" | "admin" | "delete";
  /** 子节点(有权限时渲染) */
  children: React.ReactNode;
  /** 无权限时渲染(可选) */
  fallback?: React.ReactNode;
  /** 目标资源 ID(用于 CEL condition 评估) */
  resourceId?: Uuid;
}
```

### 4.2 4 级权限粒度 → UI 显隐映射

| 角色 | 可做 | 不可做 |
|---|---|---|
| viewer | 列表 / 详情 read | transition 按钮 / 配置修改 |
| developer | + transition / 自有资源 | 跨租户 / tenant settings |
| project_admin | + 项目内任意 transition / settings | 跨项目 |
| tenant_admin | + 全部 | (无) |

### 4.3 transition button 是否可见的判定逻辑

```ts
// 在 DetailPage 内
const showTransitionButton = 
  actor.roles.includes("developer") &&  // 角色够
  resource.project_id in actor.project_ids;  // 在 actor 项目列表中

// REQ-WF-003 Guard 校验在 transition 触发时(后端强制,前端可预览)
const guardMet = checkGuard(rule.condition, resource, actor);
```

---

## 5. Secret 脱敏(继承 frontend-design §6.5)

### 5.1 脱敏格式

```
****<prefix>***REDACTED***<suffix>
```

例: GitHub PAT `ghp_abc123def456ghi789jkl012mno345pqr` → `****ghp_***REDACTED***pqr`

### 5.2 触发 hover 5 秒显示完整(V1 候选 + audit 记录)

- MVP:始终脱敏,hover 显示 tooltip "完整值请到 Settings 查看"
- V1:5 秒后显示完整 + 写 audit("secret.viewed")

### 5.3 必须脱敏字段清单

| 字段 | 类型 | 出现位置 |
|---|---|---|
| `*_token` | string | integration.config_masked(已脱敏) |
| `*_key` | string | integration / scm webhook |
| `webhook_secret` | string | integration |
| `password` | string | identity |
| `api_key` | string | integration |
| `private_key` | string | (V1 候选)identity |

---

## 6. Loop 防护(继承 frontend-design §6.6)

### 6.1 loop_protection_key 字段

```ts
interface Integration {
  loop_protection_key?: string;  // 例 "lp-github-9981"
}
```

### 6.2 Integration 列表展示 + hover 提示

- 列表中 `loop_protection_key` 列 warn 色 pill
- hover tooltip: "webhook idempotency key,用于去重避免风暴"

### 6.3 24h error_count > 5 整行 err 色

```ts
const errorTone = i.error_count_24h > 5 ? "err" : i.error_count_24h > 0 ? "warn" : "ok";
```

---

## 7. Realtime 通道(继承 frontend-design §7)

### 7.1 WebSocket 客户端契约(V1 候选)

```ts
// BFF 端点 /v1/realtime(经 BFF fan-out,不直连 NATS)
const ws = new WebSocket("/v1/realtime?tenant_id=ten-acme");
ws.onmessage = (e) => {
  const event: RealtimeEvent = JSON.parse(e.data);
  // event.subject = "star.worktree.transition"
  // event.data = { worktree_id, from, to, ... }
};
```

### 7.2 25 Module ↔ NATS Subject 完整映射表

| Module | NATS Subject 前缀 | 前端订阅动作 | MVP/V1 |
|---|---|---|---|
| **worktree** | `star.worktree.*` | worktree 列表 + 选中行 SM 高亮 | V1 |
| **agent** | `star.agent.*` | agent 列表 + 选中行 token gauge | V1 |
| **feedback** | `star.feedback.*` | feedback inbox + 未读计数 | V1 |
| **context** | `star.context.*` | context packet 列表 + decision pending | V1 |
| **validation** | `star.validation.*` | validation result 实时 + coverage 进度 | V1 |
| **scm** | `star.scm.*` | PR SM + CI 状态 | V1 |
| **integration** | `star.integration.*` | 24h error 计数实时 | V1 |
| **notification** | `star.notification.*` | bell badge 实时 | V1 |
| **audit** | `star.audit.*` | audit 流(append-only 显示) | V1 |
| **work-item** | `star.workitem.*` | 列表过滤实时更新 | V1 |
| **comment** | `star.comment.*` | 评论实时 | V1 |
| **planning** | `star.planning.*` | burndown 实时 | V1 |
| **board** | `star.board.*` | (V1 拖拽) board 实时 | V2 |
| **tenant** | (静态,低频) | - | - |
| **project** | (静态,低频) | - | - |
| **identity** | (静态,低频) | - | - |
| **workflow** | (静态,低频) | - | - |
| **permission** | (静态,低频) | - | - |
| **development** | (静态,低频) | - | - |
| **local-runtime** | `star.local-runtime.*`(V1) | heartbeat 实时 | V1 |
| **collaboration** | `star.collab.*`(WS 优先) | cursor 实时 | V1 |
| **automation** | `star.automation.*` | 24h fired 计数 | V1 |
| **relation** | (静态,低频) | - | - |
| **workspace** | (静态,低频) | - | - |
| **search** | (拉模式,无 WS) | - | - |

**12 Module 启用 Realtime**,13 Module 静态(列表/详情走 REST 拉取)。

### 7.3 订阅实现模式(V1 候选)

```ts
// hooks/useRealtime.ts(V1 启用)
function useRealtime(subject: string, onMessage: (data: any) => void) {
  useEffect(() => {
    const ws = new WebSocket("/v1/realtime");
    ws.onmessage = (e) => {
      const event = JSON.parse(e.data);
      if (event.subject === subject || event.subject.startsWith(subject)) {
        onMessage(event.data);
      }
    };
    return () => ws.close();
  }, [subject]);
}
```

### 7.4 降级策略(WS 不可用 → SSE)

```ts
// BFF 支持 EventSource 降级
const es = new EventSource("/v1/realtime/sse?subject=star.worktree.*");
es.onmessage = (e) => { /* 同 ws.onmessage */ };
```

### 7.5 背压策略(高频 cursor 10Hz → 2Hz)

- BFF 检测单浏览器 cursor 推送 > 10Hz → 降采样到 2Hz
- 客户端检测 WS 队列积压 > 100 msg → 显示 "实时较慢" 提示

---

## 8. 错误码 → UI 反馈映射(继承 frontend-design §8.2)

### 8.1 6 类错误 UI 表现

| 错误码 | 含义 | UI 表现 | 触发 |
|---|---|---|---|
| **SEC-001** | 跨 tenant 访问 | 顶部 red banner(固定 3s)+ 点击跳 Dashboard | 任何带 tenant_id 的请求 |
| **WF-403** | effect=deny | button disabled + tooltip "无权限:<rule summary>" | 权限检查失败 |
| **WF-409** | InvalidTransition | toast yellow + revert SM 状态 | 状态机 transition 失败 |
| **API-429** | rate limit | toast yellow + Retry-After 倒计时 | 频率超限 |
| **API-500** | internal | red banner + "上报 Sentry" 按钮(V1) | 服务端错误 |
| **SC-001** | lock_version 不一致 | toast yellow + 重新 fetch + 高亮 stale 字段 | 乐观锁冲突 |

### 8.2 toast 组件 Props

```ts
interface ToastProps {
  /** 错误码(决定色码) */
  code: "SEC-001" | "WF-403" | "WF-409" | "API-429" | "API-500" | "SC-001" | "default";
  /** 标题(1 句) */
  title: string;
  /** 详细消息 */
  message?: string;
  /** 自动消失 ms(0 = 不消失) */
  duration?: number;          // default 3000
  /** 行动按钮(可选) */
  action?: { label: string; onClick: () => void };
}
```

### 8.3 banner 组件 Props

```ts
interface BannerProps {
  /** 严重度 */
  severity: "err" | "warn" | "info";
  /** 标题 */
  title: string;
  /** 详细消息 */
  message?: string;
  /** 行动按钮(可选) */
  action?: { label: string; onClick: () => void };
  /** 可关闭 */
  dismissible?: boolean;
}
```

---

## 9. 数据流时序图

### 9.1 Worktree state 推送(NATS → WS → Zustand → StateMachineDiagram)

```mermaid
sequenceDiagram
    participant NATS as NATS JetStream
    participant BFF as Frontend BFF
    participant Browser as Browser WS
    participant Store as Zustand Store
    participant SMD as StateMachineDiagram

    NATS->>BFF: star.worktree.transition<br/>{ worktree_id, from, to }
    BFF->>BFF: fan-out to subscribed browsers
    BFF->>Browser: WS message { subject, data }
    Browser->>Store: setWorktreeStatus(id, to)
    Store->>Store: trigger re-render
    Store->>SMD: highlightState 更新
    SMD->>SMD: 重绘高亮 + 边
```

### 9.2 WorkItem transition(POST → 乐观 → 失败 revert)

```mermaid
sequenceDiagram
    participant User
    participant UI as Transition Button
    participant Store as Zustand Store
    participant API as Backend REST
    participant Toast

    User->>UI: click "→ done"
    UI->>Store: transitionWorkItem(id, done)<br/>乐观更新 status
    Store->>UI: re-render (status=done)
    UI->>API: POST /v1/work-items/{id}/transitions<br/>{ to: "done" }
    alt 成功 (200)
        API-->>UI: { current_status: "done" }
        UI->>User: 完成
    else 失败 409 InvalidTransition
        API-->>UI: { code: "WF-409", current_status: "in_progress" }
        UI->>Store: revert status
        UI->>Toast: 黄色 toast
        Toast->>User: 显示错误
    end
```

### 9.3 Cross-tenant 请求拒绝(SEC-001)

```mermaid
sequenceDiagram
    participant User
    participant UI as Page Component
    participant API as Backend
    participant Banner as Top Banner

    User->>UI: 访问 /work-item/wi-001(假设 wi-001 在其他 tenant)
    UI->>API: GET /v1/work-items/wi-001<br/>X-Tenant-Id: ten-acme
    API-->>UI: 401 { code: "SEC-001" }
    UI->>Banner: show red banner
    Banner->>User: "无权访问 tenant 'ten-acme' 的资源 wi-001"
    Note over UI,Banner: 3 秒后自动消失
    UI->>UI: 重定向到 Dashboard(可选)
```

---

## 10. 5 项新 ADR(ADR-FE-016~020)

### ADR-FE-016:Zustand 持有 UI 投影,TanStack Query 持有 REST 缓存

- **状态**: Accepted
- **决策**:
  - Zustand 持 25 域 + 6 mutator
  - V1 启用 TanStack Query 时,**只**接管 REST 缓存(不接管 mutator)
  - 严禁混用(任何组件写 `useQuery` 不能同时 `useStore.setState` 同一字段)
- **验收**: 任何 store mutator 不包含 `fetch` 调用

### ADR-FE-017:NATS Subject 必须经过 1:1 映射表

- **状态**: Accepted
- **决策**:
  - 25 Module ↔ NATS Subject 映射见 §7.2 表格
  - 禁止 page 硬编码 subject 字符串(必须 import 映射)
- **验收**: `grep "star\\." frontend/src/app | grep -v "_sm\\.ts"` 应为空

### ADR-FE-018:错误码 → UI 反馈映射是 1:1 单一来源

- **状态**: Accepted
- **决策**:
  - 6 类错误码 → UI 表现 见 §8.1 表格
  - 禁止 page 内联错误文案(必须 import 映射)
- **验收**: `grep -rn "toast.error" frontend/src/app | wc -l` 全部走映射

### ADR-FE-019:Secret 脱敏是渲染层职责

- **状态**: Accepted
- **决策**:
  - 脱敏在 page / molecule 组件层做(不依赖后端不发完整 secret)
  - 防御性:即使后端意外发完整 secret,前端也脱敏
  - 显示完整值时**必须**走 audit 记录

### ADR-FE-020:Realtime 推送必须经 BFF fan-out

- **状态**: Accepted
- **决策**:
  - 浏览器不直连 NATS(通过 BFF 转发)
  - BFF 单点 fan-out + 降采样 + 鉴权
  - 任何 NATS Subject 改动先在 BFF 灰度
- **安全理由**: NATS 不暴露公网,直连意味着 NATS 鉴权 token 暴露给浏览器

---

## 11. 已知缺口(V1/V2 候选)

| 编号 | 描述 | 优先级 |
|---|---|---|
| INT03-OI-01 | (V1) TanStack Query 启用 + 与 Zustand 协同(ADR-FE-016) | P1 |
| INT03-OI-02 | (V1) WebSocket Client + useRealtime hook(§7.3) | P1 |
| INT03-OI-03 | (V1) 12 Module 启用 Realtime(§7.2 表) | P1 |
| INT03-OI-04 | (V1) 25 page 错误码映射(§8.1) | P1 |
| INT03-OI-05 | (V2) SSE 降级(§7.4) | P2 |
| INT03-OI-06 | (V2) WS 背压 + 客户端降采样(§7.5) | P2 |
| INT03-OI-07 | (V2) OpenAPI 自动生成 NATS Subject 映射 | P2 |

---

> **下游交接**:
> 1. frontend-internal-04 §1 引用本文 §7 Realtime 通道表
> 2. frontend-internal-04 §2 引用本文 §8 错误码 UI 反馈
> 3. frontend-internal-04 §8 引用本文 §1 25 module 字段
> 4. 任何 store / fetch 变更必须走 §10 ADR-FE-016~020
