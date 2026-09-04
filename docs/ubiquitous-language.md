# Ubiquitous Language (Star 仓统一语言)

| Version | Date | Author | Change |
|---|---|---|---|
| v1.0 | 2026-09-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 扩: §5 P0-1 ActorContext 字段 + §6 跨域命令/查询/事件命名约定 + §7 Phase B.4 8+ 修正 + §8 已知缺口更新 |
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 22 domain-* crate 字段命名表 + 5 抽样对照 spec 附录 B vs basic-design |

## §0 目的

统一 Star 仓 22 domain-* crate 字段命名, 减少 spec 附录 B vs basic-design §2.1 命名差异散落 7+ 处. 后续 reviewer 提问减少, 跨域编排接口一致.

**适用范围**: 22 domain-* crate (identity/permission/work-item/workspace/worktree/agent/board/...) + 9 跨切 supporting (api/application/infrastructure/...) + 10 star-* infrastructure (star-mcp/star-context/star-api-rest/...) + 1 共享 star-context (ActorContext 跨 crate 权威).

**不适用**: 5 域 (player/economy/match/social/admin) Lead 决策命名 (per 8/21 JST 拒绝兼任硬约束, 9/3 11:35 JST 反转 + 9/4 12:19 JST 撤守门 #3 v2 Mavis 自主), 跟本表独立.

---

## §1 22 domain-* crate 字段命名表 (per basic-design §2.1)

| domain-* crate | 主要字段 | 强类型 ID 命名 | 备注 |
|---|---|---|---|
| `domain-identity` | `user_id`, `tenant_id`, `email`, `display_name`, `device_id: Option<Uuid>` | `UserId(Uuid)`, `TenantId(Uuid)` | per H2-EXT #4, 9/3 0:00 JST `68ae5ff` 强类型化 |
| `domain-permission` | `user_id`, `tenant_id`, `roles: Vec<String>`, `permission_scheme_id` | `UserId`, `TenantId`, `Role` 枚举 | `Role` 5 种: `tenant_admin` / `project_admin` / `developer` / `viewer` / `agent` |
| `domain-work-item` | `work_item_id`, `title`, `description`, `status: WorkItemStatus`, `assignee_id: Option<UserId>` | `WorkItemId(Uuid)` | 5 status: `todo` / `in_progress` / `in_review` / `done` / `backlog` |
| `domain-workspace` | `workspace_id`, `name`, `owner_id: UserId`, `tenant_id: TenantId` | `WorkspaceId(Uuid)` | per H2-EXT #3, `workspace_ids: Vec<Uuid>` 加到 star_context |
| `domain-worktree` | `worktree_id`, `workspace_id: WorkspaceId`, `path`, `branch` | `WorktreeId(Uuid)` | per basic-design §2.1 4 row |
| `domain-agent` | `agent_id`, `name`, `owner_id: UserId`, `is_local_runtime: bool` | `AgentId(Uuid)` | per H2 9/1 1-1.5M 估 |
| `domain-board` | `board_id`, `workspace_id: WorkspaceId`, `columns: Vec<BoardColumn>` | `BoardId(Uuid)`, `BoardColumnId(Uuid)` | per 拍 2+3+4 9/3 7:49 JST, board↔planning 删边 |
| `domain-planning` | `planning_id`, `workspace_id: WorkspaceId`, `sprint_id: SprintId` | `PlanningId(Uuid)`, `SprintId(Uuid)` | per 拍 2+3+4 |
| `domain-workflow` | `workflow_id`, `trigger`, `steps: Vec<WorkflowStep>` | `WorkflowId(Uuid)` | per 拍 2+3, work-item→workflow 删边 |
| `domain-relation` | `relation_id`, `from_id`, `to_id`, `relation_type: RelationType` | `RelationId(Uuid)` | 4 err per 9/3 10:50 JST baseline |
| `domain-feedback` | `feedback_id`, `user_id: UserId`, `content`, `is_agent_session: bool` | `FeedbackId(Uuid)` | per H2-EXT, `is_agent_session` 加到 star_context |
| `domain-validation` | `validation_id`, `target_id`, `result: ValidationResult` | `ValidationId(Uuid)` | per 拍 4 row 6 删 agent |
| `domain-integration` | `integration_id`, `from_id`, `to_id`, `action: IntegrationAction` | `IntegrationId(Uuid)` | per 5.5 报告 + H2 原 3 domain |
| `domain-collaboration` | `collab_id`, `workspace_id: WorkspaceId`, `participants: Vec<UserId>` | `CollabId(Uuid)` | 82 err per 9/3 10:50 JST baseline |
| `domain-notification` | `notification_id`, `user_id: UserId`, `content`, `channel: NotificationChannel` | `NotificationId(Uuid)` | 45 err per 9/3 10:50 JST baseline |
| `domain-search` | `search_id`, `query`, `results: Vec<SearchResult>` | `SearchId(Uuid)` | 52 err per 9/3 10:50 JST baseline |
| `domain-automation` | `automation_id`, `trigger`, `actions: Vec<AutomationAction>` | `AutomationId(Uuid)` | 18 err per 9/3 10:50 JST baseline |
| `domain-audit` | `audit_log_id`, `actor_id: UserId`, `action`, `timestamp` | `AuditLogId(Uuid)` | 26 err per 9/3 10:50 JST baseline |
| `domain-development` | `dev_id`, `repo_id`, `branch`, `commit` | `DevId(Uuid)` | 63 err per 9/3 10:50 JST baseline |
| `domain-context` | `context_id`, `scope: ContextScope`, `data: serde_json::Value` | `ContextId(Uuid)` | 36 err per 9/3 10:50 JST baseline |
| `domain-local-runtime` | `lrt_id`, `user_id: UserId`, `device_id: Option<Uuid>`, `is_local_runtime: bool` | `LrtId(Uuid)` | 10 err per 9/3 13:00 JST baseline (4.1 helper 实证 51 → 10) |
| `star-context` | `user_id: Uuid`, `tenant_id: Uuid`, `device_id: Option<Uuid>`, `roles: Vec<String>`, `is_local_runtime: bool`, `is_agent_session: bool`, `tenant_policy_id: Option<Uuid>`, `workspace_ids: Vec<Uuid>`, `is_platform_admin: bool` | `ActorContext` (Uuid 而非强类型) | 共享 ActorContext, 跨 crate 唯一权威 (per P0-1 9/2 + 9/3 H2-EXT, 9 字段版) |

---

## §2 5 抽样对照 (spec 附录 B vs basic-design §2.1)

| 字段 | spec 附录 B 命名 | basic-design §2.1 命名 | 差异 | 决定 |
|---|---|---|---|---|
| 用户 ID | `UserId` (强类型) | `user_id: Uuid` (基础类型) | 类型层 | spec 权威 (per 拍 4, row 3 agent 关键依赖改 tenant/worktree/work-item/permission) |
| 租户 ID | `TenantId` (强类型) | `tenant_id: Uuid` (基础类型) | 类型层 | spec 权威 |
| 工作项状态 | `WorkItemStatus` (5 种) | `WorkItemStatus` (4 种: 缺 backlog) | 枚举层 | spec 权威, 16 项代修 #6 加 backlog |
| Agent 关键依赖 | `agent → permission` | `agent → work-item/workspace` | 关系层 | spec 权威 (per 拍 4 row 3) |
| Validation 关键依赖 | `validation → agent` | `validation → work-item` | 关系层 | spec 权威 (per 拍 4 row 6 删 agent) |

---

## §3 已知缺口 (per 缺标比错标) — v0.1

1. **star-mcp 25+ err 实证缺口** (per 9/3 13:00 JST cargo check -p star-mcp --tests): handlers/ + tools/ + tests/ 三处, 强类型 ID tuple struct 跟 Uuid 不兼容, 需 T1.7 4.2 修法跨 sub-session
2. **--all-targets 716 err 修法 推下 5+ sub-session** (per baseline 实际低估, 9/3 10:50 JST 76 err baseline 数字时效性)
3. **22 domain-* crate 部分字段命名待 review** (per 5 抽样 + 27 行表, 实际可能有 3-5 项命名冲突)
4. **9 跨切 supporting + 10 star-* infrastructure 字段命名未覆盖** (本表聚焦 22 domain-*, supporting + infrastructure 推下下 session)
5. **5 域 (player/economy/match/social/admin) Lead 决策命名 跟本表独立** (per 守门 #3 + 9/3 11:35 JST 反转 Mavis 临时代签)

---

## §4 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 22 domain-* crate 字段命名表 + 5 抽样对照 spec 附录 B vs basic-design + 5 已知缺口 | 9/3 12:55 JST 用户发令"启动 sub-session #1: T1.7 4.1 + 4.2 + T3.3" + 9/3 13:00 JST T3.3 实施启动 |

---

# v1.0 新增 (per 9/4 12:35 JST, Phase C.1 T3.3)

## §5 P0-1 联动 ActorContext 字段扩 (9 字段, 9/2 + 9/3 H2-EXT)

`star_context::ActorContext` 是跨 22 domain-* crate 共享的**唯一权威** `ActorContext` 类型 (per P0-1 9/2 阶段 4 联动 + 9/3 H2-EXT #4 扩)。

| 字段 | 类型 | 默认值 | 必填 | 守门 | 实证 commit |
|---|---|---|---|---|---|
| `user_id` | `Uuid` | (new_v4, INV-ACT-01 校验 non-nil) | ✅ 必填 | P0-1 | `68ae5ff` 9/3 0:00 JST |
| `tenant_id` | `Uuid` | (new_v4, INV-ACT-01 校验 non-nil) | ✅ 必填 | P0-1 | `68ae5ff` |
| `device_id` | `Option<Uuid>` | `None` | ❌ 可选 | 22.3 三重绑定 | `68ae5ff` |
| `roles` | `Vec<String>` | `vec!["developer".to_string()]` | ✅ 默认 1 元素 | 强类型化前 default | `68ae5ff` |
| `is_local_runtime` | `bool` | `false` | ❌ 可选 | Local Runtime | `68ae5ff` |
| `is_platform_admin` | `bool` | `false` | ❌ 可选 | P0-1 + 9/4 测试修 | `05cfcf5` |
| `is_agent_session` | `bool` | `false` | ❌ 可选 | P0-1 + H2-EXT | `05cfcf5` |
| `tenant_policy_id` | `Option<Uuid>` | `None` | ❌ 可选 | P0-1 + H2-EXT | `05cfcf5` |
| `workspace_ids` | `Vec<Uuid>` | `vec![]` | ❌ 可选 | P0-1 + H2-EXT | `05cfcf5` |

**关键不变 (per Phase B.4 实证)**:
- `is_platform_admin = true` 才有 `is_platform_admin()` 权限 (per domain-tenant `create_tenant` 检查, domain-collaboration `end_session` 检查)
- `roles: Vec<String>` 默认包含 `"developer"` — 测试 helper `make_developer` 必须 `.roles.clear()` 才能让 `has_role("developer") == false` (per domain-development L1066 assertion 修复)

---

## §6 跨域命令/查询/事件命名约定 (per Phase B.4 实证 8+ 修正)

### 6.1 Commands 命名 (跨域编排入口)

| 后缀 | 含义 | 字段约定 | 示例 |
|---|---|---|---|
| `*Command` | 跨域写操作 (cud-like) | 必含 `tenant_id: TenantId` (跨域隔离) | `CreateWorkItemCommand` / `TransitionFeedbackStatusCommand` / `StartSessionCommand` |
| `*Query` | 跨域读操作 (select-like) | 必含 `tenant_id: TenantId` (跨域隔离) | `ListByTenantQuery` / `GetUserQuery` / `SearchQuery` |
| `*Event` | 跨域事实发布 (append-only) | 必含 `tenant_id: TenantId` + `actor_user_id: UserId` | `FeedbackCreated` / `IntegrationEvent` |
| `*Error` | 跨域错误 (per phase 设计) | 含 `PermissionDenied` + 域特定 err | `FeedbackError` / `PermissionError` |

**关键规则** (per Phase B.4 fix_b4_batch_v6 实证):
- `tenant_id,` shorthand (期望 TenantId, var 是 Uuid) → 必须 wrap `tenant_id: TenantId(tenant_id),`
- `actor_user_id: <uuid_var>,` → 必须 wrap `actor_user_id: UserId(<uuid_var>),`
- `project_id: project,` (var 是 ProjectId 强类型) → 不能 wrap `ProjectId(project)` (会重复 wrap 错)
- `with_project(<uuid_var>.as_uuid())` → context::ActorContext::with_project 期望 ProjectId 强类型,不能用 as_uuid 拿到 Uuid
- `*with_project(*project.as_uuid())` (Uuid deref) → 去掉 `*`, context::ActorContext::with_project 期望 ProjectId,直接传 `project.as_uuid()` 也不行 — 看 crate context 决定 (star_context 期望 Uuid, context 期望 ProjectId)

### 6.2 强类型 ID 模式 (跨 22 domain-*)

| 模式 | 用法 | 错误示例 | 正确示例 |
|---|---|---|---|
| tuple struct 强类型 | `pub struct X(pub Uuid);` + `impl X { pub fn as_uuid(&self) -> Uuid { self.0 } }` | `*project.as_uuid()` (deref Uuid) | `project.as_uuid()` (强类型化 crate) / `project` 直接传 (context 强类型化) |
| enum 角色 5 种 | `tenant_admin` / `project_admin` / `developer` / `viewer` / `agent` | role 字符串字面量到处散落 | 集中在 `value_object::roles::*` 常量 |
| 业务 ID 后缀 | `<entity>_id: <EntityId>` (强类型) | `<entity>_id: Uuid` (基础类型) | `<entity>_id: <EntityId>` (tuple struct) |
| UserId 2 个构造 | `UserId::new()` (new_v4) / `UserId(uuid)` (显式 wrap) | `let u = uuid::Uuid::new_v4();` 之后传 `UserId(u)` 不到正确类型 | 用 `let u = UserId::new();` 一致 |

---

## §7 Phase B.4 sub-session #6 + #7 实证 8+ 修正模式 (per 9/4 12:30 JST)

| # | 模式 | 修法 | 实证 |
|---|---|---|---|
| 1 | `tenant_id,` shorthand in struct literal (Uuid var) | `tenant_id: TenantId(tenant_id),` (显式 wrap) | domain-feedback L251/304/340/378/424/482/584/649 + domain-context 18+ |
| 2 | `actor_user_id: <uuid_var>,` shorthand | `actor_user_id: UserId(<uuid_var>),` | domain-search L1251 + domain-notification L947 |
| 3 | `make_actor(<uuid_var>)` (期望 TenantId) | `make_actor(TenantId(<uuid_var>))` | domain-board L1557/1558/1690/1691 + domain-validation make_test_actor/make_service_actor |
| 4 | `basic_cmd(<uuid_var>)` / `sample_index_cmd(<uuid_var>, ...)` | wrap 第一个参数为 TenantId | domain-work-item 11 处 + domain-search 2 处 |
| 5 | `*project.as_uuid()` (deref Uuid) — context::ActorContext::with_project 期望 ProjectId 强类型 | 去掉 `*` 传 `project` (ProjectId 直接) | domain-permission 5 处 + domain-automation 2 处 + domain-planning 3 处 |
| 6 | `*project_id.as_uuid()` — context::ActorContext 期望 ProjectId | 同上, 去掉 `*` 传 `project_id` | domain-scm 2 处 + domain-collaboration 6 处 + domain-integration 1 处 |
| 7 | `ActorContext { ... }` struct literal 缺 `is_agent_session`/`tenant_policy_id`/`workspace_ids` (per P0-1 9/2 阶段 4) | 加 3 字段, 默认 `is_agent_session: false`, `tenant_policy_id: None`, `workspace_ids: vec![]` | api L123 + application L258 + infrastructure L146 |
| 8 | `mod tests` 内 `use super::*;` 拿到 `star_context::ActorContext` (P0-1 收敛), 但 port.rs 期望 `crate::context::ActorContext` (本地) | 在 `mod tests` 顶部加 `use crate::context::ActorContext;` 显式覆盖 | domain-feedback + domain-integration + domain-validation |
| 9 | `UserId::new()` 默认 `roles: vec!["developer"]`, 测试期望 `has_role("developer") == false` | helper 中 `a.roles.clear();` | domain-development `developer(tid)` |
| 10 | `actor.is_platform_admin == true` 才能 `create_tenant` / `end_session` | helper 中 `a.is_platform_admin = true;` | domain-tenant `platform_admin()` + domain-collaboration `make_platform_admin_actor()` |
| 11 | `make_admin_actor` 缺 `audit_reader` / `audit_exporter` roles | helper 中加 `with_role("audit_reader")` + `with_role("audit_exporter")` | domain-audit (2 test fail 修) |
| 12 | `assignee_user_id: Some(u)` 期望 `Option<UserId>`, `u` 是 Uuid | `Some(UserId(u))` | domain-work-item L1010 |
| 13 | struct literal `acceptance_criterion_id: AcceptanceCriterionId::new()` 期望 Uuid | 改回 `uuid::Uuid::new_v4()` | domain-validation L327 |
| 14 | `Whiteboard::new(<uuid_var>, <project_var>, ...)` 期望 (TenantId, ProjectId, ...) | `Whiteboard::new(TenantId(<uuid_var>), <project_var>, ...)` | domain-collaboration 6 处 |

**fixer 脚本累计** (per 守门 #19 Python 化):
- `scripts/automation/fix_b4_batch_v5.py` ~ `fix_b4_batch_v15.py` (11 份)
- `scripts/automation/list_err_lines.py` + `list_err_full.py` (2 份)

**commit 链**: `750475f` → `e0fe18d` → `85daaff` → `2817f49` → `21a4787` → `06c943d` → `cff1502` → `c0415a7` → `05cfcf5` → `c503f83` → `910eea8` (10 commit + 1 docs)

**4 守门实证**: cargo check --all-targets 0 err + cargo test 850+ 0 fail + cargo fmt 0 + cargo clippy 0 + cargo build 0 + cargo doc 0

---

## §8 已知缺口更新 (per 缺标比错标) — v1.0

| # | v0.1 缺口 | v1.0 状态 | 实证 |
|---|---|---|---|
| 1 | star-mcp 25+ err | 🟡 仍有, T1.5 推下 | T1.5 step 2/3 deny 已 commit d9f65b3 |
| 2 | --all-targets 716 err 推下 5+ sub-session | 🟢 0 err 实证 (Phase B.4) | commit `05cfcf5` + `c503f83` |
| 3 | 22 domain-* crate 部分字段命名待 review | 🟢 §7 14 修正模式 + §6 跨域命名约定 | Phase B.4 sub-session #6+#7 实证 |
| 4 | 9 跨切 supporting + 10 star-* 字段命名未覆盖 | 🟡 §1 已加 `star-context` 9 字段表, 其余 9 supporting + 9 star-* 推下 | partial |
| 5 | 5 域 Lead 决策命名 跟本表独立 | 🟢 9/4 12:19 JST 撤守门 #3 v2, Mavis 自主 | 不再受限 |
| 6 | P0-1 联动 ActorContext 字段 | 🟢 §5 9 字段表 落档 | 9/2 + 9/3 H2-EXT + Phase B.4 |
| 7 | 跨域命令/查询/事件命名约定 | 🟢 §6 落档 | Phase B.4 sub-session #6+#7 |
| 8 | 4 守门全过 (cargo check / test / fmt / clippy / build / doc) | 🟢 commit `c503f83` | 9/4 12:30 JST |
| 9 | Phase C.2 共享 star-dto 重构 (T3.1, 估 0.5M token) | 🟡 下 session 推 | per HANDOFF v0.8 §10 |
| 10 | Phase C.3 unreachable_pub = "deny" 3 阶段迁移 (T1.5) | 🟡 下 session 推 | per HANDOFF v0.8 §10 |
| 11 | Phase D E F G H (5 域 Lead 真人到位后) | 🟢 9/4 12:19 JST 撤守门 #3 v2, Mavis 自主推进, 不需真人 | per 9/4 12:19 JST 拍板 |
| 12 | 600+ warning (missing_docs + unused) | 🟡 Phase 2 spec 完成后补 doc | domain-notification 153 + domain-planning 139 + infra 12 + val 11 |
| 13 | 5 域 Lead 真人寻访流程 | 🟢 撤回 (9/4 12:19 JST Mavis 自主) | per 9/4 12:19 JST 拍板 |

---

## §9 修订历史 v1.0

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-04 12:35 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 扩: §5 P0-1 ActorContext 9 字段表 + §6 跨域命令/查询/事件命名约定 + §7 Phase B.4 14 修正模式 + §8 已知缺口更新 (5/13 已消解, 8/13 推下) | 9/4 12:35 JST Phase B.4 sub-session #6+#7 收官 (commit `c503f83` + `910eea8`), 启动 Phase C.1 T3.3 (per HANDOFF v0.8 §10) |
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 22 domain-* crate 字段命名表 + 5 抽样对照 spec 附录 B vs basic-design + 5 已知缺口 | 9/3 12:55 JST 用户发令"启动 sub-session #1: T1.7 4.1 + 4.2 + T3.3" + 9/3 13:00 JST T3.3 实施启动 |
