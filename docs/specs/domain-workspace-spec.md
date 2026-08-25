# domain-workspace 实施 spec

> **状态**: Draft v0.1 (2026-08-25)
> **上游依赖**:
> - 《Requirements》§7 (Workspace)
> - 《Basic Design》§2.1(表 19), §5.7
> - 《API Design》§3.3
> - 《Data Design》§4.2 (`workspace` schema)
> - 《Security Design》§3.4
> **下游交付**: Implementation team — Rust crate 路径 `crates/domain-workspace/`
> **最后审稿**: 待 RFC 化时

---

## 1. 职责与边界

`domain-workspace` 承载 Tenant 内的**协作单位**(§7),Workspace → 多个 Project 的二级层级。负责 Workspace 的创建 / 成员邀请 / 跨 Project 模板,本身**不**持有 WorkItem / Worktree 等业务数据。

**属于本 crate 的**:
- Workspace 实体的生命周期与级联删除检查(防止 Project 存在时删除)
- Workspace 成员邀请(轻量,实际 User 在 `domain-identity`)
- Workspace 级安全策略(继承 Tenant SecurityPolicy)

**不属于本 crate 的**:
- User / Device / Credential 实体(`domain-identity` 拥有)
- Project 实体(`domain-project` 拥有)
- WorkItem / Worktree(`domain-work-item` / `domain-worktree`)

## 2. 关键实体

引用 data-design §4.2 (`workspace` schema):

**Workspace**(聚合根)
- 标识: `workspace_id`, `tenant_id`, `slug`
- 元数据: `display_name`, `description`, `status`
- 关联: `created_by_user_id`, `default_project_template_id`
- 容量: `max_projects`, `max_members`(可空,继承 Tenant 限制)

**WorkspaceMember**(实体)
- `workspace_id`, `user_id`, `role_workspace`(WorkspaceAdmin / WorkspaceMember)
- `invited_at`, `joined_at`, `status`(Invited / Active / Removed)

## 3. 关键不变量

| ID | 不变量 | 上游依据 |
|---|---|---|
| INV-W-01 | Workspace 必须属于一个 Tenant(`tenant_id` 必带,继承 §16 REQ-SEC-001) | basic-design §6.1 |
| INV-W-02 | 删除 Workspace 前必须无 Project 引用(级联检查,Application 层强制) | basic-design §5.7 |
| INV-W-03 | Workspace Member 角色变更不跨 Workspace 生效(独立 role_workspace) | security-design §3.1 |
| INV-W-04 | Workspace 创建时必带 `created_by_user_id`,且该 User 必属同一 Tenant | basic-design §5.7 |

## 4. 接口签名

继承 api-design §3.3。

```rust
// crates/domain-workspace/src/port.rs

pub trait WorkspaceCommandPort {
    async fn create_workspace(
        &self,
        cmd: CreateWorkspaceCommand,  // 含 tenant_id (from JWT), slug, display_name
        actor: ActorContext,
    ) -> Result<WorkspaceId, WorkspaceError>;

    async fn update_workspace(
        &self,
        cmd: UpdateWorkspaceCommand,  // 不可改 tenant_id
        actor: ActorContext,
    ) -> Result<Workspace, WorkspaceError>;

    async fn delete_workspace(
        &self,
        id: WorkspaceId,
        actor: ActorContext,          // 需 workspace:delete
    ) -> Result<(), WorkspaceError>;  // 需级联检查 Project 引用

    async fn invite_member(
        &self,
        cmd: InviteMemberCommand,     // 含 workspace_id, user_id, role_workspace
        actor: ActorContext,
    ) -> Result<WorkspaceMember, WorkspaceError>;

    async fn remove_member(
        &self,
        cmd: RemoveMemberCommand,     // workspace_id, user_id
        actor: ActorContext,
    ) -> Result<(), WorkspaceError>;
}

pub trait WorkspaceQueryPort {
    async fn list_by_tenant(&self, actor: ActorContext) -> Result<Vec<Workspace>, WorkspaceError>;
    async fn get_by_id(&self, id: WorkspaceId, actor: ActorContext) -> Result<Workspace, WorkspaceError>;
    async fn list_members(&self, id: WorkspaceId, actor: ActorContext) -> Result<Vec<WorkspaceMember>, WorkspaceError>;
}
```

## 5. Domain Events

| Subject (NATS) | 触发条件 | Payload |
|---|---|---|
| `star.events.workspace.workspace.created.v1` | `create_workspace` 成功 | `workspace_id, tenant_id, slug, created_by` |
| `star.events.workspace.workspace.deleted.v1` | `delete_workspace` 成功 | `workspace_id, deleted_at` |
| `star.events.workspace.member.invited.v1` | `invite_member` 成功 | `workspace_id, user_id, role_workspace` |
| `star.events.workspace.member.removed.v1` | `remove_member` 成功 | `workspace_id, user_id` |

**订阅者**:
- `domain-audit`(Append-only) — 全部事件
- `domain-notification` — `member.invited`(通知被邀请人)

## 6. 数据所有权

引用 data-design §4.2(`workspace` schema):

- `workspace.workspace`(聚合根)
- `workspace.workspace_member`(实体)

**RLS 策略**:
- `USING (current_setting('app.current_tenant_id')::uuid = tenant_id)`,跨 tenant 查询返回 0 行
- `BYPASSRLS` 仅 Service-Internal 角色

**索引策略**:
- `workspace.workspace(tenant_id, slug)` UNIQUE
- `workspace.workspace_member(workspace_id, user_id)` UNIQUE

## 7. 鉴权与授权

引用 security-design §3.1-3.4:

**Permission 字符串**:
- `workspace:read`, `workspace:create`, `workspace:update`, `workspace:delete`
- `workspace_member:read`, `workspace_member:invite`, `workspace_member:remove`

**内置 Role 覆盖**:
- `tenant_admin` — 全部
- `project_admin` / `developer` / `viewer` — 仅 `workspace:read` + `workspace_member:read`(自身成员关系)

## 8. 错误码

| 错误码 | HTTP | 触发条件 |
|---|---|---|
| `SEC-001` | 401 | JWT 缺失 |
| `SEC-002` | 403 | tenant_id Header 与 JWT 不一致 |
| `SEC-007` | 403 | Cross-Tenant Workspace 访问 |
| `WS-001` | 422 | slug 格式非法 |
| `WS-002` | 409 | slug 在同 Tenant 内已存在 |
| `WS-003` | 409 | Workspace 仍有 Project 引用,删除拒绝 |
| `WS-004` | 404 | WorkspaceMember 不存在 |
| `WS-005` | 403 | 邀请 User 不属同 Tenant |

## 9. 实施任务分解

| 任务 | 描述 | 依赖 | TBD-MEASURE | 估算 |
|---|---|---|---|---|
| T1 | Workspace + WorkspaceMember 实体 + Value Object | 无 | — | 60K tokens |
| T2 | `WorkspaceCommandPort` 5 个方法 + 错误码 | T1 | — | 100K tokens |
| T3 | `WorkspaceQueryPort` 3 个方法 | T1, T2 | — | 60K tokens |
| T4 | 级联删除检查(Application 层 Service 调用) | T2 | basic-design §5.7 | 80K tokens |
| T5 | 单元测试 + RLS 测试矩阵 | T1-T4 | security-design §3.5.4 | 100K tokens |
| T6 | 集成测试:End-to-End 创建 → 邀请成员 → 删除 | T5 | api-design §3.3 | 80K tokens |

**合计估算**: ~480K tokens ≈ 2 人·天(AI 协作模式)

## 10. 验收标准(AC)

```gherkin
Feature: Workspace 管理

  Scenario: 创建 Workspace 成功
    Given 用户是 Tenant Admin
    When POST /v1/workspaces {slug: "acme", display_name: "Acme Corp"}
    Then 201 Created {workspace_id, slug}
    And  AuditEvent 记录 action=workspace_created

  Scenario: 删除仍有 Project 引用
    Given Workspace W 包含 Project P
    When DELETE /v1/workspaces/{W}
    Then 409 WS-003 (Project 仍存在)
    And  Workspace 未被删除

  Scenario: 跨 Tenant 邀请拒绝
    Given 用户 U 在 Tenant X
    When Workspace W (Tenant Y) 邀请 U
    Then 403 WS-005 (User 不属同 Tenant)
    And  WorkspaceMember 未被创建

  Scenario: 邀请非成员用户到 Workspace
    Given User A 是 Workspace W 的 WorkspaceAdmin
    When A 邀请 User B 到 W
    Then 201 Created {WorkspaceMember, status=Invited}
    And  Notification 发送给 B
```

## 11. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| Workspace 删除级联破坏 Project | High | Application 层强制级联检查 + Worker 周期校验孤儿 Workspace | basic-design §5.7 |
| 跨 Tenant Workspace 越权 | Critical | RLS + AuthorizationChecker 双重 | basic-design §6.1 |
| Workspace Member 数量爆炸 | Medium | `max_members` 字段(继承 Tenant 限制) | security-design §3.1 |

## 12. Open Issues

- J-WS-01: Workspace 级 RBAC 角色是否与 Tenant Role 重叠?目前双层(WorkspaceMember.role_workspace + User.tenant_role)
- J-WS-02: `default_project_template_id` 是否需在 Workspace 创建时强制?(目前可选)
- J-WS-03: Workspace 是否支持"软删除"以保留历史 Project 引用?(目前硬删除)

## 附录 A:关键流程时序图 — Workspace 创建与级联删除保护

```mermaid
sequenceDiagram
    autonumber
    actor U as User (Tenant Admin)
    participant GW as API Gateway
    participant APP as Application Service
    participant WS as domain-workspace
    participant PRJ as domain-project
    participant PG as PostgreSQL
    participant AUD as domain-audit
    participant NATS as NATS

    U->>GW: POST /v1/workspaces {slug, display_name}
    GW->>APP: create_workspace(cmd, actor)
    APP->>APP: AuthorizationChecker.check(actor, action=WorkspaceCreate)
    APP->>WS: WorkspaceCommandPort::create_workspace
    WS->>WS: 生成 workspace_id (UUIDv7)
    WS->>PG: BEGIN; INSERT workspace.workspace
    PG-->>WS: OK
    WS->>PG: INSERT outbox (WorkspaceCreated)
    PG-->>WS: OK
    WS->>PG: COMMIT
    WS-->>APP: WorkspaceId
    APP->>AUD: AuditRecorder.record
    APP-->>GW: 201 Created
    GW-->>U: 201

    Note over PG,NATS: Outbox Worker 异步推送
    PG->>NATS: publish star.events.workspace.workspace.created.v1
    NATS-->>AUD: 订阅

    U->>GW: DELETE /v1/workspaces/{W}
    GW->>APP: delete_workspace(W, actor)
    APP->>WS: WorkspaceCommandPort::delete_workspace
    WS->>PRJ: ProjectQueryPort::list_by_workspace(W)  [跨域只读]
    PRJ-->>WS: [Project P1, P2, ...]
    alt Project 引用 > 0
        WS-->>APP: Err(WS-003)
        APP-->>GW: 409 Conflict
        GW-->>U: 409 WS-003
    else Project 引用 == 0
        WS->>PG: BEGIN; DELETE workspace.workspace
        PG-->>WS: OK
        WS->>PG: COMMIT
        WS-->>APP: OK
        APP->>AUD: Audit (Protected)
        APP-->>GW: 204 No Content
        GW-->>U: 204
    end
```

## 附录 B:边界清单

| 边界类型 | 本 Module 行为 |
|---|---|
| 上游依赖 | `domain-tenant`(tenant_id 注入) |
| 下游调用 | `domain-audit`, `domain-notification`, `domain-project`(级联检查时只读) |
| 跨域事务 | 删除 Workspace 前调用 `domain-project` 列表查询(Application 编排) |
| RLS 强制 | `workspace.workspace` + `workspace.workspace_member` 启用 RLS |
| 13 类 tenant_id 对象 | 间接覆盖(Workspace 必带 tenant_id,自身非 13 类) |
| 14 状态 AgentSession 触发 | 无 |
| 17 状态 Worktree 触发 | 无 |
| WorkItem 3 态 | 无 |

**接口稳定承诺**:Port trait 签名 + 6 条错误码在后续 RFC 阶段不会变更。
