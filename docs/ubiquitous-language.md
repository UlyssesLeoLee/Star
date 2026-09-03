# Ubiquitous Language (Star 仓统一语言)

| Version | Date | Author | Change |
|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 22 domain-* crate 字段命名表 + 5 抽样对照 spec 附录 B vs basic-design |

## §0 目的

统一 Star 仓 22 domain-* crate 字段命名, 减少 spec 附录 B vs basic-design §2.1 命名差异散落 7+ 处. 后续 reviewer 提问减少, 跨域编排接口一致.

**适用范围**: 22 domain-* crate (identity/permission/work-item/workspace/worktree/agent/board/...) + 9 跨切 supporting (api/application/infrastructure/...) + 10 star-* infrastructure (star-mcp/star-context/star-api-rest/...)

**不适用**: 5 域 (player/economy/match/social/admin) Lead 决策命名 (per 8/21 JST 拒绝兼任硬约束, 9/3 11:35 JST 反转 Mavis 临时代签), 跟本表独立.

---

## §1 22 domain-* crate 字段命名表 (per basic-design §2.1)

| domain-* crate | 主要字段 | 强类型 ID 命名 | 备注 |
|---|---|---|---|
| `domain-identity` | `user_id`, `tenant_id`, `email`, `display_name`, `device_id: Option<Uuid>` | `UserId(Uuid)`, `TenantId(Uuid)` | per H2-EXT #4, 9/3 0:00 JST `68ae5ff` 强类型化 |
| `domain-permission` | `user_id`, `tenant_id`, `roles: Vec<String>`, `permission_scheme_id` | `UserId`, `TenantId`, `Role` 枚举 | per `Role` 5 种: `tenant_admin` / `project_admin` / `developer` / `viewer` / `agent` |
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
| `star-context` | `user_id: Uuid`, `tenant_id: Uuid`, `device_id: Option<Uuid>`, `roles: Vec<String>`, `is_local_runtime: bool` | `ActorContext` (无强类型) | 共享 ActorContext, 跨 crate 唯一权威 |

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

## §3 已知缺口 (per 缺标比错标)

1. **star-mcp 25+ err 实证缺口** (per 9/3 13:00 JST cargo check -p star-mcp --tests): handlers/ + tools/ + tests/ 三处, 强类型 ID tuple struct 跟 Uuid 不兼容, 需 T1.7 4.2 修法跨 sub-session
2. **--all-targets 716 err 修法 推下 5+ sub-session** (per baseline 实际低估, 9/3 10:50 JST 76 err baseline 数字时效性)
3. **22 domain-* crate 部分字段命名待 review** (per 5 抽样 + 27 行表, 实际可能有 3-5 项命名冲突)
4. **9 跨切 supporting + 10 star-* infrastructure 字段命名未覆盖** (本表聚焦 22 domain-*, supporting + infrastructure 推下下 session)
5. **5 域 (player/economy/match/social/admin) Lead 决策命名 跟本表独立** (per 守门 #3 + 9/3 11:35 JST 反转 Mavis 临时代签)

---

## §4 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 22 domain-* crate 字段命名表 + 5 抽样对照 spec 附录 B vs basic-design + 5 已知缺口 | 9/3 12:55 JST 用户发令"启动 sub-session #1: T1.7 4.1 + 4.2 + T3.3" + 9/3 13:00 JST T3.3 实施启动 (per 4 类剩余任务 拍板 B 加快并行 + 守门 #3 v2 反转 + 守门 #1 v3 派生规实证缺口) |
