# domain-project 实施 spec

> **状态**: Draft v0.1 (2026-08-25)
> **上游依赖**:
> - 《Requirements》§7 (Project)
> - 《Basic Design》§2.1(表 20), §4.10.2, §5.7
> - 《API Design》§3.4
> - 《Data Design》§4.3 (`project` schema)
> - 《Security Design》§3.1-3.4
> **下游交付**: Implementation team — Rust crate 路径 `crates/domain-project/`
> **最后审稿**: 待 RFC 化时

---

## 1. 职责与边界

`domain-project` 承载 Project 模板 / 配置 / Policy(REQ-TWP-003),是 WorkItem / Worktree / Agent 等核心域的"配置平面"。负责 Project 的创建、ProjectPolicy(workflow / permission / notification / agent policy 模板)的整体替换。

**属于本 crate 的**:
- Project 聚合根与 ProjectTemplate(平台级,只读模板)
- ProjectPolicy 的强一致替换(整体 PUT,partial PATCH)
- Project 与 Workspace 的归属关系(tenant_id + workspace_id 必带)

**不属于本 crate 的**:
- Project 内的 WorkItem / Worktree 数据(`domain-work-item` / `domain-worktree` 拥有)
- User 实体(`domain-identity` 拥有)
- Workflow / Board / Planning 的领域逻辑(由 `domain-workflow` / `domain-board` / `domain-planning` 拥有,Project 仅存 Policy 引用)

## 2. 关键实体

引用 data-design §4.3 (`project` schema):

**Project**(聚合根)
- 标识: `project_id`, `tenant_id`, `workspace_id`, `slug`
- 元数据: `display_name`, `description`, `status`
- 模板: `project_template_id`(可空)
- Policy 引用: `default_workflow_id`, `default_permission_scheme_id`, `default_agent_policy_id`, `default_validation_policy_id`, `default_context_policy_id`
- 容量: `max_worktrees`, `max_agent_sessions`(可空)

**ProjectPolicy**(聚合根,与 Project 1:1)
- Workflow 配置: `custom_workflow_id`(可覆盖 default)
- Permission: `permission_scheme_id`
- Notification: `notification_template_id`
- Agent Policy: `agent_policy_id`, `max_runtime_seconds`, `max_context_tokens`
- Validation: `validation_policy_id`, `required_test_passes`
- SCM: `default_repository_id`(可选)
- Commit / PR / Merge Gate: `commit_requires_user`, `pr_creation_requires_user`, `merge_gate`(必须人类)

**ProjectTemplate**(平台级只读聚合)
- `template_id`, `name`, `category`(software_development / devops / research)
- 预置: `default_workflow_id`, `default_permission_scheme_id`, 等
- `created_at`, `version`

## 3. 关键不变量

| ID | 不变量 | 上游依据 |
|---|---|---|
| INV-P-01 | Project 必须属一个 Workspace,Workspace 必须属一个 Tenant(必带 tenant_id + workspace_id) | basic-design §6.1, §7 |
| INV-P-02 | ProjectPolicy 与 Project 1:1 强一致(整体替换,不允许 partial 写入绕过校验) | REQ-TWP-003, basic-design §4.10.2 |
| INV-P-03 | `merge_gate=true` 时,Merge 操作必须人类触发,Agent 不能直接 Merge | basic-design §4.2.6 (Human-in-the-loop) |
| INV-P-04 | `default_workflow_id` 引用必须存在(可空 → 走 system default 三态) | basic-design §7.2 (REQ-WF-001) |
| INV-P-05 | ProjectTemplate 是平台级只读,不允许 Tenant 自定义 Template(防越权) | basic-design §0.3 |
| INV-P-06 | 删除 Project 前需级联检查 WorkItem / Worktree / AgentSession(由 Application 编排) | basic-design §5.7 |

## 4. 接口签名

继承 api-design §3.4。

```rust
// crates/domain-project/src/port.rs

pub trait ProjectCommandPort {
    async fn create_project(
        &self,
        cmd: CreateProjectCommand,  // 含 workspace_id, slug, display_name, template_id?
        actor: ActorContext,
    ) -> Result<ProjectId, ProjectError>;

    async fn update_project(
        &self,
        cmd: UpdateProjectCommand,  // 不可改 workspace_id / tenant_id
        actor: ActorContext,
    ) -> Result<Project, ProjectError>;

    async fn delete_project(
        &self,
        id: ProjectId,
        actor: ActorContext,
    ) -> Result<(), ProjectError>;  // 级联检查

    async fn replace_project_policy(
        &self,
        cmd: ReplaceProjectPolicyCommand,  // 整体替换
        actor: ActorContext,
    ) -> Result<ProjectPolicy, ProjectError>;

    async fn patch_project_policy(
        &self,
        cmd: PatchProjectPolicyCommand,    // 部分更新(受限字段)
        actor: ActorContext,
    ) -> Result<ProjectPolicy, ProjectError>;
}

pub trait ProjectQueryPort {
    async fn list_by_workspace(&self, workspace_id: WorkspaceId, actor: ActorContext) -> Result<Vec<Project>, ProjectError>;
    async fn get_by_id(&self, id: ProjectId, actor: ActorContext) -> Result<Project, ProjectError>;
    async fn get_policy(&self, id: ProjectId, actor: ActorContext) -> Result<ProjectPolicy, ProjectError>;
    async fn list_templates(&self, category: Option<String>, actor: ActorContext) -> Result<Vec<ProjectTemplate>, ProjectError>;
    async fn get_template(&self, id: ProjectTemplateId, actor: ActorContext) -> Result<ProjectTemplate, ProjectError>;
}
```

## 5. Domain Events

| Subject (NATS) | 触发条件 | Payload |
|---|---|---|
| `star.events.project.project.created.v1` | `create_project` 成功 | `project_id, workspace_id, tenant_id, slug, template_id?` |
| `star.events.project.project.policy_replaced.v1` | `replace_project_policy` 成功 | `project_id, policy_version, replaced_at` |
| `star.events.project.project.policy_patched.v1` | `patch_project_policy` 成功 | `project_id, patched_fields[], new_version` |
| `star.events.project.project.deleted.v1` | `delete_project` 成功(级联检查通过) | `project_id, deleted_at` |

**订阅者**:
- `domain-audit`(Append-only)
- `domain-search`(投影更新)
- `domain-notification`(`policy_replaced` 通知 Project Member)

## 6. 数据所有权

引用 data-design §4.3(`project` schema):

- `project.project`(聚合根)
- `project.project_policy`(聚合根)
- `project.project_template`(平台级只读)

**RLS 策略**:
- `project.project` + `project.project_policy`:`USING (current_setting('app.current_tenant_id') = tenant_id)`
- `project.project_template`:**禁用 RLS**(平台级,所有 Tenant 可读)

**索引策略**:
- `project.project(workspace_id, slug)` UNIQUE
- `project.project_policy(project_id)` UNIQUE
- `project.project_template(category, name)` UNIQUE

## 7. 鉴权与授权

引用 security-design §3.1-3.4:

**Permission 字符串**:
- `project:read`, `project:create`, `project:update`, `project:delete`
- `project_policy:read`, `project_policy:update`
- `project_template:read`(平台级,所有认证用户)

**内置 Role 覆盖**:
- `tenant_admin` — 全部
- `project_admin` — 全部(本 Project 范围)
- `developer` / `viewer` — 仅 `project:read` + `project_policy:read`

## 8. 错误码

| 错误码 | HTTP | 触发条件 |
|---|---|---|
| `SEC-001` / `SEC-002` / `SEC-007` | 401/403/403 | 鉴权类 |
| `PRJ-001` | 422 | slug 格式非法 |
| `PRJ-002` | 409 | slug 在同 Workspace 已存在 |
| `PRJ-003` | 409 | Project 仍有 WorkItem / Worktree 引用 |
| `PRJ-004` | 422 | ProjectPolicy 引用不存在的 Workflow / PermissionScheme |
| `PRJ-005` | 422 | `merge_gate=false`(防越权) |
| `PRJ-006` | 404 | ProjectTemplate 不存在 |

## 9. 实施任务分解

| 任务 | 描述 | 依赖 | TBD-MEASURE | 估算 |
|---|---|---|---|---|
| T1 | Project + ProjectPolicy + ProjectTemplate 实体 + Value Object | 无 | — | 80K tokens |
| T2 | `ProjectCommandPort` 5 个方法 + 错误码 | T1 | — | 120K tokens |
| T3 | `ProjectQueryPort` 5 个方法 | T1, T2 | — | 80K tokens |
| T4 | ProjectPolicy 引用完整性校验(workflow / permission_scheme 等存在) | T2 | data-design §4.3 | 100K tokens |
| T5 | 级联删除检查(WorkItem / Worktree / AgentSession) | T2 | basic-design §5.7 | 120K tokens |
| T6 | 单元测试 + RLS 测试矩阵(13 类对象覆盖) | T1-T5 | security-design §3.5.4 | 150K tokens |
| T7 | 集成测试:Template → 创建 Project → 替换 Policy → 删除 | T6 | api-design §3.4 | 100K tokens |

**合计估算**: ~750K tokens ≈ 3 人·天(AI 协作模式)

## 10. 验收标准(AC)

```gherkin
Feature: Project 模板与 Policy

  Scenario: 从 Template 创建 Project
    Given 用户 U 是 WorkspaceAdmin
    And Platform 提供 ProjectTemplate T1 (category=software_development)
    When U POST /v1/projects {workspace_id, slug, template_id: T1}
    Then 201 Created {project_id, default_workflow_id: T1.default_workflow_id}
    And  AuditEvent 记录 template 来源

  Scenario: replace_project_policy 强制引用完整性
    Given 用户 U 是 project_admin
    When U PUT /v1/projects/{P}/policy {default_workflow_id: "non_existent"}
    Then 422 PRJ-004 (Workflow 引用不存在)
    And  ProjectPolicy 未被修改

  Scenario: 删除仍有 WorkItem 引用的 Project 拒绝
    Given Project P 有 5 个 WorkItem
    When DELETE /v1/projects/{P}
    Then 409 PRJ-003 (WorkItem 仍存在)
    And  Project 未被删除

  Scenario: merge_gate 强制人类 Merge
    Given Project P 的 policy.merge_gate=true
    When AgentSession 尝试直接调用 merge API
    Then 403 AGT-010 (必须人类)
    And  AuditEvent 记录 attempt=denied

  Scenario: ProjectTemplate 跨 Tenant 可见
    Given User U1 (Tenant X) 和 U2 (Tenant Y) 都已认证
    When 两者都 GET /v1/project-templates
    Then 返回相同模板列表(平台级共享)
    And  不可 PATCH /v1/project-templates/{id}(只读)
```

## 11. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| ProjectPolicy 越权篡改 | High | project_admin 鉴权 + Audit + 7 天冷却(SecurityPolicy 复刻) | security-design §3.1 |
| Template 篡改 | Critical | 平台级只读,DB 角色无 UPDATE 权限 | basic-design §0.3 |
| 级联删除遗漏 | High | Application 编排显式调用 `domain-work-item` / `domain-worktree` 列表查询 | basic-design §5.7 |
| Workflow 引用悬空 | Medium | `replace_project_policy` 强制引用完整性校验 | data-design §4.5 |

## 12. Open Issues

- J-PRJ-01: ProjectPolicy 整体替换 vs 部分更新是否同时支持?(目前两者都支持,部分更新字段白名单待定)
- J-PRJ-02: ProjectTemplate 是否允许 Tenant 复制后自定义?(目前禁止)
- J-PRJ-03: `merge_gate` 是否支持更细粒度(per-PR 级别)?(目前 per-Project)
- J-PRJ-04: `max_worktrees` / `max_agent_sessions` 配额是硬限还是软限?(待 RFC)

## 附录 A:关键流程时序图 — Project 创建与 Policy 替换

```mermaid
sequenceDiagram
    autonumber
    actor U as User (WorkspaceAdmin)
    participant GW as API Gateway
    participant APP as Application Service
    participant PRJ as domain-project
    participant WF as domain-workflow
    participant PS as domain-permission
    participant PG as PostgreSQL
    participant AUD as domain-audit
    participant NATS as NATS

    U->>GW: POST /v1/projects {workspace_id, slug, template_id}
    GW->>APP: create_project(cmd, actor)
    APP->>APP: AuthorizationChecker.check(actor, action=ProjectCreate)
    APP->>PRJ: ProjectCommandPort::create_project
    PRJ->>PRJ: 生成 project_id
    PRJ->>PG: BEGIN
    PRJ->>PRJ: 解析 template → 默认 workflow / permission_scheme
    PRJ->>PG: INSERT project.project
    PRJ->>PG: INSERT project.project_policy
    PRJ->>PG: INSERT outbox (ProjectCreated)
    PG-->>PRJ: OK
    PRJ->>PG: COMMIT
    PRJ-->>APP: ProjectId
    APP->>AUD: AuditRecorder.record
    APP-->>GW: 201 Created
    GW-->>U: 201

    Note over PG,NATS: Outbox Worker
    PG->>NATS: publish star.events.project.project.created.v1
    NATS-->>AUD: Append
    NATS-->>worker.projection: Search Index

    U->>GW: PUT /v1/projects/{P}/policy {full policy}
    GW->>APP: replace_project_policy(cmd, actor)
    APP->>PRJ: ProjectCommandPort::replace_project_policy
    PRJ->>PRJ: 引用完整性校验
    par 并行校验
        PRJ->>WF: workflow_id 存在?
        WF-->>PRJ: OK
    and
        PRJ->>PS: permission_scheme_id 存在?
        PS-->>PRJ: OK
    end
    alt 校验失败
        PRJ-->>APP: Err(PRJ-004)
        APP-->>GW: 422
        GW-->>U: 422
    else 校验通过
        PRJ->>PG: BEGIN; UPDATE project.project_policy; INSERT outbox
        PRJ->>PG: COMMIT
        PRJ-->>APP: ProjectPolicy (new version)
        APP->>AUD: Audit (Protected)
        APP-->>GW: 200 OK
        GW-->>U: 200 OK
    end
```

## 附录 B:边界清单

| 边界类型 | 本 Module 行为 |
|---|---|
| 上游依赖 | `domain-tenant` (tenant_id), `domain-workspace` (workspace_id) |
| 下游调用 | `domain-audit`, `domain-notification`, `domain-search` |
| 跨域事务 | `replace_project_policy` 时调用 `domain-workflow` / `domain-permission` 校验(同事务读) |
| RLS 强制 | `project.project` + `project.project_policy` 启用 RLS,`project_template` 禁用 RLS(平台级) |
| 13 类 tenant_id 对象 | 间接覆盖(Project 必带 tenant_id,自身非 13 类) |
| 14 状态 AgentSession 触发 | 间接(通过 ProjectPolicy.agent_policy_id 影响 AgentSession 启动) |
| 17 状态 Worktree 触发 | 间接(通过 ProjectPolicy 影响 Worktree 行为) |
| WorkItem 3 态 | 间接(通过 default_workflow_id 决定 WorkItem 默认状态机) |

**接口稳定承诺**:Port trait 签名 + 6 条错误码 + 6 条不变量在后续 RFC 阶段不会变更。

## 15. 与其他 domain 协作 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md) + [spec/saga/01 v0.2 SagaCoordinationRole](../../architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md),本节定义 `project` 与 22 domain 中 9 个 domain 的显式接触面。

| 源 Domain | 目标 Domain | 接触方式 | 接触点 |
|---|---|---|---|
| tenant | project | Customer-Supplier | Project.tenant_id 引用 (FK) |
| workspace | project | Customer-Supplier | Project.workspace_id + WorkspacePermissionScheme 派生 |
| project | work-item | Customer-Supplier | WorkItem.project_id + ProjectPolicy (Workflow 扩展状态机源) |
| project | workflow | Customer-Supplier | Project.workflow_definition_id 引用 |
| project | board | Customer-Supplier | Project.board_configuration_id 引用 |
| project | planning | Customer-Supplier | Project.sprint_scheme_id 引用 |
| project | automation | Customer-Supplier | Project.automation_rules[] 派生 |
| project | notification | Customer-Supplier | Project.notification_scheme_id 引用 |

**接触面统计**: 8 条 (v0.16 新增,本 spec 由 `scripts/inter_collab_refine.py` 批量生成)

**dual-use 警告** (per AGENTS.md §5 v0.6 + Q1-D 拍板): 5 域 (player/economy/match/social/admin) 是 RGS 仓历史治理命名,Star 仓不建立业务子域↔DDD 映射。本 spec 协作基于 22 domain crate,不通过 5 域绑定推导。
