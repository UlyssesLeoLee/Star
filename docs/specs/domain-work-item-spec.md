# domain-work-item 实施 spec

> **状态**: Draft v0.1 (2026-08-25)
> **上游依赖**:
> - 《Requirements》§8, §9, §36
> - 《Basic Design》§2.1(表 1), §4.9, §4.10.4, §5.7, §7.2 (WorkItem 默认三态), §7.6
> - 《API Design》§3.5 (CRUD + 状态机 + AC)
> - 《Data Design》§4.4 (`work_item` schema)
> - 《Security Design》§3.4, §3.7
> - 《Internal Design》§X
> **下游交付**: Implementation team — Rust crate 路径 `crates/domain-work-item/`
> **最后审稿**: 待 RFC 化时

---

## 1. 职责与边界

`domain-work-item` 是 Star 平台的**业务核心**,承载 WorkItem 聚合根与 Requirement / AcceptanceCriterion / BusinessGoal 三类支撑实体(§8, §36)。负责 WorkItem 的创建 / 状态迁移 / 关联 Repository / Worktree。

**属于本 crate 的**:
- WorkItem 聚合根(Epic / Story / Task / Bug / Subtask / AITask 6 类)
- Requirement / AcceptanceCriterion / BusinessGoal 实体
- WorkItem 默认三态状态机 + Project Policy 扩展状态机(§7.2 REQ-WF-001)
- AITask 子类型的 Repository Scope / Allowed Files / Policy 引用

**不属于本 crate 的**:
- Workflow / State / Transition 定义(`domain-workflow` 拥有)
- Worktree 实体(`domain-worktree` 拥有,WorkItem 仅以 `worktree_ids[]` 引用)
- ChangeSet / DevelopmentExecution(`domain-development` 拥有)
- Comment / Mention / Attachment(`domain-comment` 拥有,WorkItem 触发)

## 2. 关键实体

引用 data-design §4.4 (`work_item` schema):

**WorkItem**(聚合根,§8.1)
- 标识: `work_item_id`, `tenant_id`, `workspace_id`, `project_id`
- 类型: `type`(Epic / Story / Task / Bug / Subtask / AITask)
- 内容: `title`, `description`
- 状态: `status`(由 Workflow 决定,默认三态 TODO/IN_PROGRESS/DONE)
- 责任: `assignee_user_id`, `assignee_agent_id`, `reporter_user_id`
- 优先级: `priority`, `severity`
- 计划: `story_points`, `sprint_id`, `parent_work_item_id`
- 关联: `requirement_ids[]`, `acceptance_criterion_ids[]`, `repository_ids[]`, `worktree_ids[]`
- 元数据: `labels[]`, `components[]`, `created_at`, `updated_at`, `due_date`

**AITask 子类型字段**(§8.1, §27)
- `objective`, `repository_scope`, `allowed_files[]`, `forbidden_files[]`
- `agent_policy_id`, `validation_policy_id`, `context_policy_id`

**Requirement**(§39 Traceability)
- `requirement_id`, `tenant_id`, `business_goal_id`
- `statement`, `rationale`, `linked_work_item_ids[]`

**AcceptanceCriterion**
- `acceptance_criterion_id`, `requirement_id`, `work_item_id`
- `statement`, `coverage_status`, `covered_by_validation_ids[]`

**BusinessGoal**
- `business_goal_id`, `tenant_id`
- `goal`, `description`, `linked_requirement_ids[]`

## 3. 关键不变量

| ID | 不变量 | 上游依据 |
|---|---|---|
| INV-WI-01 | WorkItem 状态机**默认三态** TODO → IN_PROGRESS → DONE,扩展由 ProjectPolicy 自定义 | REQ-WF-001, basic-design §7.2 |
| INV-WI-02 | WorkItem ≠ Git Branch(WorkItem 是业务概念,不是代码分支) | basic-design §44.3 |
| INV-WI-03 | 1 WorkItem → 0/1/N Repository;**不**强制 1:1 | basic-design §5.7 |
| INV-WI-04 | 1 WorkItem → 0/1/N Worktree;Worktree Status 独立于 WorkItem Status | REQ-DEV-001, REQ-WF-002 |
| INV-WI-05 | AITask 创建必须先有 Repository Link + Agent Policy + Validation Policy | basic-design §4.9.5 |
| INV-WI-06 | WorkItem 删除前需级联检查 Worktree(`worktree_ids` 非空 → 拒绝) | basic-design §5.7 |
| INV-WI-07 | 任何 WorkItem INSERT / UPDATE 必须带 `tenant_id` | basic-design §6.1, REQ-SEC-001 |
| INV-WI-08 | 子任务 (Subtask) 必带 `parent_work_item_id`,父必是 Story 或 Epic | data-design §4.4 |
| INV-WI-09 | WorkItem.status 合法迁移由 WorkflowDefinition 决定,本 crate 仅执行判定 | basic-design §4.9.3 |

## 4. 接口签名

继承 api-design §3.5。

```rust
// crates/domain-work-item/src/port.rs

pub trait WorkItemCommandPort {
    async fn create_work_item(
        &self,
        cmd: CreateWorkItemCommand,  // 含 type, title, project_id, assignee?, parent?
        actor: ActorContext,
    ) -> Result<WorkItemId, WorkItemError>;

    async fn update_work_item(
        &self,
        cmd: UpdateWorkItemCommand,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError>;  // If-Match 乐观并发

    async fn delete_work_item(
        &self,
        id: WorkItemId,
        actor: ActorContext,
    ) -> Result<(), WorkItemError>;

    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,  // 含 from, to, reason
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError>;  // 必须 WorkflowDefinition 允许

    async fn bulk_update(
        &self,
        cmd: WorkItemBulkUpdate,
        actor: ActorContext,
    ) -> Result<BulkResult, WorkItemError>;

    async fn link_repository(
        &self,
        cmd: LinkRepositoryCommand,   // work_item_id, repository_id
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError>;

    async fn create_requirement(
        &self,
        cmd: CreateRequirementCommand,
        actor: ActorContext,
    ) -> Result<RequirementId, WorkItemError>;

    async fn create_acceptance_criterion(
        &self,
        cmd: CreateAcceptanceCriterionCommand,
        actor: ActorContext,
    ) -> Result<AcceptanceCriterionId, WorkItemError>;
}

pub trait WorkItemQueryPort {
    async fn list_by_project(&self, q: ListWorkItemQuery, viewer: ActorContext) -> Result<Vec<WorkItem>, WorkItemError>;
    async fn get_by_id(&self, id: WorkItemId, viewer: ActorContext) -> Result<WorkItem, WorkItemError>;
    async fn list_transitions(&self, id: WorkItemId, viewer: ActorContext) -> Result<Vec<Transition>, WorkItemError>;
    async fn list_requirements(&self, work_item_id: WorkItemId, viewer: ActorContext) -> Result<Vec<Requirement>, WorkItemError>;
    async fn list_acceptance_criteria(&self, work_item_id: WorkItemId, viewer: ActorContext) -> Result<Vec<AcceptanceCriterion>, WorkItemError>;
    async fn list_business_goals(&self, q: ListBusinessGoalQuery, viewer: ActorContext) -> Result<Vec<BusinessGoal>, WorkItemError>;
}
```

## 5. Domain Events

| Subject (NATS) | 触发条件 | Payload |
|---|---|---|
| `star.events.work_item.work_item.created.v1` | `create_work_item` 成功 | `work_item_id, project_id, type, status` |
| `star.events.work_item.work_item.status_changed.v1` | `transition_status` 成功 | `work_item_id, from, to, reason, actor` |
| `star.events.work_item.work_item.worktree_linked.v1` | Worktree 创建并 link 到本 WorkItem | `work_item_id, worktree_id` |
| `star.events.work_item.work_item.deleted.v1` | `delete_work_item` 成功 | `work_item_id, deleted_at` |
| `star.events.work_item.acceptance_criterion.created.v1` | `create_acceptance_criterion` 成功 | `ac_id, work_item_id, statement` |
| `star.events.work_item.acceptance_criterion.covered.v1` | Validation 写入 coverage | `ac_id, validation_result_id` |

**订阅者**:
- `domain-audit`(Append)
- `domain-search`(投影)
- `domain-collaboration`(Realtime 推送)
- `domain-board` / `domain-planning`(Sprint / Board 视图更新)

## 6. 数据所有权

引用 data-design §4.4(`work_item` schema),本 Module 拥有的表:

- `work_item.work_item`(聚合根,**核心聚合根**)
- `work_item.requirement`(实体)
- `work_item.acceptance_criterion`(实体)
- `work_item.business_goal`(实体)

**RLS 策略**:
- `USING (current_setting('app.current_tenant_id') = tenant_id)`
- `BYPASSRLS` 仅 Service-Internal

**索引策略**(data-design §8):
- `work_item.work_item(project_id, status, updated_at DESC)` — 列表查询主索引
- `work_item.work_item(assignee_user_id, status)` — 个人工作台
- `work_item.work_item(sprint_id)` — Sprint 视图
- `work_item.work_item(parent_work_item_id)` — 父子关系
- `work_item.acceptance_criterion(work_item_id, requirement_id)` 复合
- `work_item.requirement(business_goal_id)`

## 7. 鉴权与授权

引用 security-design §3.1-3.4, §3.7:

**Permission 字符串**:
- `work_item:read`, `work_item:create`, `work_item:update`, `work_item:delete`, `work_item:transition`, `work_item:bulk_update`
- `requirement:create`, `requirement:read`
- `ac:create`, `ac:read`, `ac:update`, `ac:delete`
- `business_goal:create`, `business_goal:read`

**内置 Role 覆盖**(security-design §3.2):
- `tenant_admin` — 全部
- `project_admin` — 全部(本 Project)
- `developer` — 全部(本 Project,可被 ProjectPolicy 收窄)
- `viewer` — 仅 read

**特殊动作**:
- `work_item:transition` 是 Policy 级别,受 WorkflowDefinition 的 `required_permission` 约束
- `work_item:bulk_update` 是 Protected(防误操作)

## 8. 错误码

引用 api-design §8.3.13(WI- 系列):

| 错误码 | HTTP | 触发条件 |
|---|---|---|
| `SEC-001` / `SEC-002` / `SEC-007` | 401/403/403 | 鉴权类 |
| `WI-001` | 404 | WorkItem 不存在 |
| `WI-002` | 409 | If-Match 乐观并发冲突 |
| `WI-003` | 409 | 非法状态迁移(WorkflowDefinition 不允许) |
| `WI-004` | 422 | AITask 创建缺少 Repository / Agent Policy / Validation Policy 引用 |
| `WI-005` | 409 | 仍有 Worktree 引用,删除拒绝 |
| `WI-006` | 422 | Subtask parent 不是 Story / Epic |
| `WI-007` | 422 | 必填字段缺失(title / type) |
| `WI-008` | 409 | 循环父子关系 |

## 9. 实施任务分解

| 任务 | 描述 | 依赖 | TBD-MEASURE | 估算 |
|---|---|---|---|---|
| T1 | WorkItem + Requirement + AC + BusinessGoal 实体 + Value Object | 无 | — | 120K tokens |
| T2 | `WorkItemCommandPort` 8 个方法 + 错误码 + Domain Event | T1 | — | 180K tokens |
| T3 | `WorkItemQueryPort` 6 个方法 | T1, T2 | — | 100K tokens |
| T4 | 默认三态状态机(状态迁移判定表) | T2 | basic-design §7.2 | 80K tokens |
| T5 | ProjectPolicy 自定义扩展示例支持(WorkflowDefinition 查询 + 扩展状态机) | T4 | data-design §4.5 | 120K tokens |
| T6 | AITask 子类型校验(Repository / Policy 必带) | T2 | basic-design §4.9.5 | 60K tokens |
| T7 | 级联删除检查(Worktree 引用) | T2 | basic-design §5.7 | 80K tokens |
| T8 | 乐观并发(If-Match / version 字段) | T2 | api-design §3.5 | 60K tokens |
| T9 | 单元测试 + RLS 测试矩阵 + 状态机迁移测试 | T1-T8 | security-design §3.5.4 | 220K tokens |
| T10 | 集成测试:CRUD + 状态迁移 + 关联 Repository + AITask 创建 | T9 | api-design §3.5 | 180K tokens |

**合计估算**: ~1.2M tokens ≈ 5-6 人·天(AI 协作模式)

## 10. 验收标准(AC)

```gherkin
Feature: WorkItem 状态机与业务规则

  Scenario: 默认三态状态机
    Given WorkItem W (status=TODO)
    When PATCH /v1/work-items/{W} {status: IN_PROGRESS}
    Then 200 OK, status=IN_PROGRESS
    When PATCH /v1/work-items/{W} {status: DONE}
    Then 200 OK, status=DONE
    And  AuditEvent 记录 from→to 转换

  Scenario: 非法状态迁移被 WorkflowDefinition 拒绝
    Given WorkItem W (status=DONE) 且 WorkflowDefinition 不允许 DONE→TODO
    When PATCH /v1/work-items/{W} {status: TODO}
    Then 409 WI-003 (非法状态迁移)

  Scenario: AITask 缺少 Agent Policy 引用
    Given 用户尝试创建 AITask
    And Request 缺 agent_policy_id
    When POST /v1/work-items
    Then 422 WI-004 (AITask 必带 Policy)

  Scenario: 删除仍有 Worktree 引用
    Given WorkItem W 关联 Worktree WT
    When DELETE /v1/work-items/{W}
    Then 409 WI-005 (Worktree 仍存在)

  Scenario: 跨 Tenant 访问
    Given User U (Tenant X) 访问 WorkItem W (Tenant Y)
    When GET /v1/work-items/{W}
    Then 403 SEC-007

  Scenario: 乐观并发冲突
    Given WorkItem W (version=3) 被 Client A 加载
    When Client A 提交 PATCH 时 W 已被 Client B 修改到 version=4
    Then 409 WI-002 (If-Match 失败)
```

## 11. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| WorkItem 与 Worktree Status 强耦合 | High | REQ-WF-002 强约束,本 crate 仅写 WorkItem.status,不写 Worktree.status | basic-design §7.1 |
| AITask 创建绕过 Policy 校验 | High | T6 强制 Repository / Agent Policy / Validation Policy 三者必带 | basic-design §4.9.5 |
| 跨 Project WorkItem 引用 | Medium | `parent_work_item_id` 必须同 Project 校验 | data-design §4.4 |
| 状态机无限扩展(失去"默认三态"约束) | Medium | T5 扩展示例以 ProjectPolicy 显式声明,默认走 system default | basic-design §7.2 |
| 13 类 tenant_id 漏配 | Critical | RLS + AuthorizationChecker 双重 | basic-design §6.1 |

## 12. Open Issues

- J-WI-01: WorkItem 是否需要软删除(支持恢复)?目前硬删除
- J-WI-02: AITask 是否允许升级为人类 Task(中途接管)?目前不支持(需新建 WorkItem)
- J-WI-03: parent_work_item_id 是否支持跨 Project?目前同 Project
- J-WI-04: 状态机扩展是否需要 Project Admin 二次确认?目前直接走 ProjectPolicy

## 附录 A:关键流程时序图 — AITask 创建 + 状态迁移

```mermaid
sequenceDiagram
    autonumber
    actor U as User
    participant GW as API Gateway
    participant APP as Application Service
    participant WI as domain-work-item
    participant PRJ as domain-project
    participant SC as domain-scm
    participant AUD as domain-audit
    participant NATS as NATS

    U->>GW: POST /v1/work-items {type: AITask, repository_id, agent_policy_id, validation_policy_id}
    GW->>APP: create_work_item(cmd, actor)
    APP->>APP: AuthorizationChecker.check(actor, action=WorkItemCreate)
    APP->>WI: WorkItemCommandPort::create_work_item
    WI->>WI: 校验 AITask 子类型
    par 并行校验
        WI->>PRJ: ProjectPolicy.agent_policy_id 存在?
        PRJ-->>WI: OK
    and
        WI->>PRJ: ProjectPolicy.validation_policy_id 存在?
        PRJ-->>WI: OK
    and
        WI->>SC: Repository 存在?
        SC-->>WI: OK
    end
    alt 校验失败
        WI-->>APP: Err(WI-004)
        APP-->>GW: 422
        GW-->>U: 422
    else 校验通过
        WI->>WI: 生成 work_item_id, status=TODO (default)
        WI->>PRJ: BEGIN; INSERT work_item.work_item
        PRJ-->>WI: OK
        WI->>PRJ: INSERT outbox (WorkItemCreated)
        PRJ->>PRJ: COMMIT
        WI-->>APP: WorkItemId
        APP->>AUD: AuditRecorder
        APP-->>GW: 201 Created
        GW-->>U: 201
    end

    Note over PRJ,NATS: Outbox 推送
    PRJ->>NATS: publish star.events.work_item.work_item.created.v1
    NATS-->>AUD: Append
    NATS-->>worker.projection: Search Index

    U->>GW: POST /v1/work-items/{W}:transition {to: IN_PROGRESS}
    GW->>APP: transition_status(cmd, actor)
    APP->>WI: WorkItemCommandPort::transition_status
    WI->>WI: 查询 WorkflowDefinition (from=TODO, to=IN_PROGRESS 合法?)
    alt 合法
        WI->>PRJ: BEGIN; UPDATE work_item.work_item (status=IN_PROGRESS, version+1)
        WI->>PRJ: INSERT outbox (StatusChanged)
        WI->>PRJ: COMMIT
        WI-->>APP: WorkItem (new status)
        APP->>AUD: Audit
        APP-->>GW: 200 OK
        GW-->>U: 200
    else 非法
        WI-->>APP: Err(WI-003)
        APP-->>GW: 409
        GW-->>U: 409
    end
```

## 附录 B:边界清单

| 边界类型 | 本 Module 行为 |
|---|---|
| 上游依赖 | `domain-tenant`, `domain-workspace`, `domain-project`, `domain-workflow` (WorkflowDefinition 查询) |
| 下游调用 | `domain-audit`, `domain-search`, `domain-board`, `domain-planning`, `domain-collaboration` |
| 跨域事务 | `create_work_item` + `replace_project_policy` 校验在同一 Application 事务(`application` 编排) |
| RLS 强制 | 4 个表全部启用 RLS,tenant_id 强制 |
| 13 类 tenant_id 对象 | **覆盖 #1 Repository Credential**(WorkItem → Repository 引用,间接强制)、#3 Worktree(WorkItem.worktree_ids[] 引用)、其他间接 |
| 14 状态 AgentSession 触发 | 无直接,但 AITask 类型与 AgentSession 1:N 关系间接管理 |
| 17 状态 Worktree 触发 | 无直接,但 WorkItem.status 独立(REQ-WF-002);Worktree 状态变更不反向写 WorkItem |
| WorkItem 3 态 | **本 Module 拥有**:TODO / IN_PROGRESS / DONE(REQ-WF-001 强约束);扩展 IN_REVIEW / BLOCKED / CANCELLED 等由 ProjectPolicy 自定义 |

**接口稳定承诺**:Port trait 签名 + 8 条错误码 + 9 条不变量 + 默认三态状态机在后续 RFC 阶段不会变更。

## 15. 与其他 domain 协作 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md) + [spec/saga/01 v0.2 SagaCoordinationRole](../../architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md),本节定义 `work-item` 与 22 domain 中 11 个 domain 的显式接触面。

| 源 Domain | 目标 Domain | 接触方式 | 接触点 |
|---|---|---|---|
| project | work-item | Customer-Supplier | WorkItem.project_id + ProjectPolicy (Workflow 扩展状态机源) |
| workflow | work-item | Customer-Supplier | WorkflowDefinition → state machine (per REQ-WF-001) |
| board | work-item | Customer-Supplier | BoardConfiguration.project_id 投影 WorkItem 列表 |
| planning | work-item | Customer-Supplier | Sprint.contains_work_item_ids[] (只读 FK) |
| comment | work-item | Customer-Supplier | Comment.parent = WorkItem (per REQ-COLLAB-001) |
| relation | work-item | Customer-Supplier | Relation.source/target = WorkItem (blocks/relates/duplicates, per REQ-COLLAB-002) |
| collaboration | work-item | Customer-Supplier | Realtime 状态推送 (per requirements §15) |
| automation | work-item | Customer-Supplier | AutomationRule.action = WorkItem transition (per REQ-AUTO-001) |
| development | work-item | Customer-Supplier | DevelopmentExecution.work_item_id 引用 |
| search | work-item | Published Language | 投影 WorkItem → Search Index (worker projection role) |
| notification | work-item | Separate Ways(异步) | 监听 WorkItem StateChanged 触发 |

**接触面统计**: 11 条 (v0.16 新增,本 spec 由 `scripts/inter_collab_refine.py` 批量生成)

**dual-use 警告** (per AGENTS.md §5 v0.6 + Q1-D 拍板): 5 域 (player/economy/match/social/admin) 是 RGS 仓历史治理命名,Star 仓不建立业务子域↔DDD 映射。本 spec 协作基于 22 domain crate,不通过 5 域绑定推导。
