# domain-tenant 实施 spec

> **状态**: Draft v0.1 (2026-08-25)
> **上游依赖**:
> - 《Requirements》§7, §16 (REQ-SEC-001, REQ-SEC-002, REQ-SEC-003)
> - 《Basic Design》§2.1(表 18), §4.10.2, §5.7, §6.1(13 类对象 1-2)
> - 《API Design》§3.2 (domain-tenant 端点), §5.5, §8.3.7
> - 《Data Design》§4.1 (`tenant` schema), §7 (RLS)
> - 《Security Design》§2 (鉴权), §3.1-3.4, §4.1-4.5
> - 《Internal Design》§X (本 spec 主要为接口稳定承诺,实施细节见内部设计)
> **下游交付**: Implementation team — Rust crate 路径 `crates/domain-tenant/`
> **最后审稿**: 待 RFC 化时(PoC-016 / RISK-016 关联)

---

## 1. 职责与边界

`domain-tenant` 是 Star 平台的**最高安全边界**(§16,REQ-SEC-001),承载 Tenant / TenantPolicy / SecurityPolicy / ProviderDataBoundary 4 类核心聚合,负责 tenant_id 的生成、跨租户隔离的第一道闸门、与第三方 AI Provider 的数据契约绑定。**严禁**任何 `domain-*` crate 在未携带 tenant_id 的情况下被 Application Service 组合调用。

**属于本 crate 的**:
- Tenant / TenantPolicy / SecurityPolicy / ProviderDataBoundary 实体的领域不变量与生命周期
- tenant_id 的生成算法 (UUIDv7 派生,§3.1 Security Design) 与跨租户校验
- Tenant Usage 统计(WorkItem 数 / AgentSession 数 / Object Storage 字节)
- SecurityPolicy 与 ProviderDataBoundary 的强一致写入

**不属于本 crate 的**:
- 用户登录态与 JWT 颁发(`domain-identity` 持有 User / Device / Credential)
- Workspace / Project 的逻辑树(`domain-workspace` / `domain-project`)
- 跨域事务(由 `crates/application` 编排,本 crate 仅是聚合根拥有者)

## 2. 关键实体

引用 data-design §4.1 (`tenant` schema),本 crate 拥有 4 个聚合根:

**Tenant**(聚合根)
- 标识: `tenant_id`, `slug`(URL 友好,唯一)
- 元数据: `display_name`, `status`(Active / Suspended / Deleted)
- 计费: `plan_tier`, `created_at`, `trial_ends_at`
- 隔离边界: `tenant_id` 由本 crate 颁发,所有 13 类对象强制携带(§6.1)

**TenantPolicy**(聚合根)
- AI 边界: `cloud_ai_allowed`, `cloud_ai_restricted`, `local_ai_only`
- 上传限制: `no_code_upload`, `metadata_only`, `specific_provider_allowed[]`
- 地域: `allowed_regions[]`, `data_residency_zone`

**SecurityPolicy**(聚合根)
- 二要素: `require_mfa`, `mfa_grace_period_seconds`
- Session: `session_max_age_seconds`, `refresh_token_ttl_seconds`
- Local Runtime: `device_max_per_user`, `device_ttl_seconds`

**ProviderDataBoundary**(聚合根)
- 标识: `provider_id`, `model_id`, `region`
- 数据分类: `data_sent[]`(Prompt / Code / Diff / Symbol / Test / BuildLog)
- 保留: `retention_policy`(Zero / N_Days / UntilTaskEnd)
- 凭据: `credential_ref`(引用 Credential Broker,不存明文,§5.4 Security Design)

## 3. 关键不变量

| ID | 不变量 | 上游依据 |
|---|---|---|
| INV-T-01 | 任何聚合根的 `INSERT` / `UPDATE` 必须携带 `tenant_id`,由本 crate 在 Port 实现层校验,Domain 层不信任调用方 | REQ-SEC-001, basic-design §5.7 |
| INV-T-02 | `tenant_id` 由本 crate 颁发 (UUIDv7),**不可**由调用方传入;Application 层从 JWT 提取 claim 注入 | basic-design §5.7, security-design §4.1 |
| INV-T-03 | 跨 tenant 访问必须返回 403 `SEC-007` + Audit 记录,**绝不**静默放行 | security-design §3.5.4 |
| INV-T-04 | `ProviderDataBoundary.credential_ref` 永不可被 Domain 层明文化,仅可传递 `CredentialRef` Value Object | security-design §5.4 |
| INV-T-05 | `SecurityPolicy` 修改需走 7 天冷却窗口(由 Worker 强制,生产环境租户),防越权篡改 | REQ-SEC-002, security-design §3.1 |
| INV-T-06 | Tenant 状态变为 `Suspended` / `Deleted` 时,**所有** 13 类对象的 RLS 应自动拒绝,本 crate 仅负责状态字段,Application 层负责广播 | basic-design §6.1 |

## 4. 接口签名

继承 api-design §3.2。Port trait 锁定如下,**实现细节留待 internal-design**:

```rust
// crates/domain-tenant/src/port.rs

pub trait TenantCommandPort {
    /// 申请新 tenant_id 并创建 Tenant(平台 Platform Admin 触发)
    async fn create_tenant(
        &self,
        cmd: CreateTenantCommand,   // 含 slug, display_name, plan_tier
        actor: ActorContext,         // 仅 Platform Admin
    ) -> Result<TenantId, TenantError>;

    /// 修改 Tenant 元数据(不可改 tenant_id 本身)
    async fn update_tenant(
        &self,
        cmd: UpdateTenantCommand,
        actor: ActorContext,
    ) -> Result<Tenant, TenantError>;

    /// 整体替换 SecurityPolicy(需 Protected 鉴权 + 7 天冷却)
    async fn replace_security_policy(
        &self,
        cmd: ReplaceSecurityPolicyCommand,
        actor: ActorContext,
    ) -> Result<SecurityPolicy, TenantError>;

    /// 注册 / 更新 ProviderDataBoundary
    async fn upsert_provider_boundary(
        &self,
        cmd: UpsertProviderBoundaryCommand,
        actor: ActorContext,
    ) -> Result<ProviderDataBoundary, TenantError>;

    /// 暂停 / 恢复 Tenant
    async fn transition_status(
        &self,
        cmd: TransitionTenantStatusCommand,  // 含 target: Suspended / Active / Deleted
        actor: ActorContext,
    ) -> Result<Tenant, TenantError>;
}

pub trait TenantQueryPort {
    /// 读取当前 JWT 绑定的 Tenant
    async fn get_current(&self, actor: ActorContext) -> Result<Tenant, TenantError>;
    /// 读取 Tenant 详情(JWT claim tenant_id 必须与路径一致)
    async fn get_by_id(&self, id: TenantId, actor: ActorContext) -> Result<Tenant, TenantError>;
    /// 读取 SecurityPolicy
    async fn get_security_policy(&self, id: TenantId, actor: ActorContext) -> Result<SecurityPolicy, TenantError>;
    /// 读取 ProviderDataBoundary 列表
    async fn list_provider_boundaries(&self, id: TenantId, actor: ActorContext) -> Result<Vec<ProviderDataBoundary>, TenantError>;
    /// 资源使用统计
    async fn get_usage_report(&self, id: TenantId, actor: ActorContext) -> Result<TenantUsageReport, TenantError>;
}
```

## 5. Domain Events

引用 api-design §5.5 (CloudEvents 1.0 命名空间),本 Module 发布的事件:

| Subject (NATS) | 触发条件 | Payload 关键字段 |
|---|---|---|
| `star.events.tenant.tenant.created.v1` | `create_tenant` 成功 | `tenant_id, slug, plan_tier, created_at` |
| `star.events.tenant.tenant.security_policy_replaced.v1` | `replace_security_policy` 成功 | `tenant_id, policy_id, version, applied_at` |
| `star.events.tenant.tenant.provider_boundary_upserted.v1` | `upsert_provider_boundary` 成功 | `tenant_id, provider_id, model_id, data_sent[]` |
| `star.events.tenant.tenant.status_changed.v1` | `transition_status` 成功 | `tenant_id, from_status, to_status, reason` |

**订阅者**(由 application 编排):
- `domain-audit` (Append-only) — 全部事件
- `domain-notification` — `status_changed` (Tenant Suspended 时通知 Platform Admin)
- `worker.projection.role` — `status_changed` 触发 Search Index 更新

## 6. 数据所有权

引用 data-design §4.1(`tenant` schema),本 Module 拥有的表 / 视图:

- `tenant.tenant`(聚合根)
- `tenant.tenant_policy`(聚合根)
- `tenant.security_policy`(聚合根,7 天冷却字段)
- `tenant.provider_data_boundary`(聚合根,JSONB data_sent)
- `tenant.tenant_usage_snapshot`(Projection,Worker 周期刷新)

**RLS 策略**(引用 data-design §7.1 通用模板):
- 所有表启用 RLS,USING 子句强制 `current_setting('app.current_tenant_id') = tenant_id`
- 跨 tenant SELECT 0 行;INSERT/UPDATE 阻断 + Audit
- Service-Internal 走 `BYPASSRLS` 角色(参考 data-design §7.5)

**索引策略**(引用 data-design §8):
- `tenant.tenant(slug)` UNIQUE
- `tenant.provider_data_boundary(tenant_id, provider_id, model_id)` 复合 UNIQUE
- `tenant.tenant_usage_snapshot(tenant_id, snapshot_at DESC)` 用于 Usage 趋势查询

## 7. 鉴权与授权

引用 security-design §3.1-3.4:

**Permission 字符串**:
- `tenant:read`, `tenant:update`
- `tenant_policy:read`, `tenant_policy:update`
- `provider_boundary:read`, `provider_boundary:create`, `provider_boundary:update`, `provider_boundary:delete`

**内置 Role 覆盖**(security-design §3.2):
- `tenant_admin` — 全部 tenant:* / tenant_policy:* / provider_boundary:* 操作
- `project_admin` — 仅 `tenant:read` + `tenant_policy:read` + `provider_boundary:read`
- `developer` / `viewer` — 仅 `tenant:read`

**强制**:Tenant 边界变更 (Suspended / Deleted) 需 Protected 鉴权 + Tenant Admin 角色 + 二次确认 (§3.6.1 security-design 的"7 天冷却"约束,本 crate 仅暴露冷却查询接口,Worker 强制执行)。

## 8. 错误码

引用 api-design §8.3.7 (SEC- 系列),本 Module 涉及:

| 错误码 | HTTP | 触发条件 |
|---|---|---|
| `SEC-001` | 401 | JWT 缺失 / 失效 / `alg=none` |
| `SEC-002` | 403 | `X-Tenant-Id` Header 与 JWT `tenant_id` claim 不一致 |
| `SEC-007` | 403 | Cross-Tenant Access(actor.tenant_id != resource.tenant_id) |
| `SRV-001` | 503 | Tenant Service 不可用(健康检查失败) |
| `T-001` | 422 | slug 格式非法(URL 不友好) |
| `T-002` | 409 | slug 已存在 |
| `T-003` | 422 | ProviderDataBoundary 数据分类与 `no_code_upload` 冲突 |
| `T-004` | 422 | SecurityPolicy 7 天冷却未到 |
| `T-005` | 409 | Tenant 状态非法迁移(例如 Deleted → Active) |

## 9. 实施任务分解

> **依赖**:无(本 crate 是 25 Module 依赖图的最底层,§2.3 basic-design)。任务粒度按 AI token 估算(1 人·天 ≈ 100K-300K tokens,参考 RGS-TS-001 v0.4 §6.2)。

| 任务 | 描述 | 依赖 | TBD-MEASURE | 估算 |
|---|---|---|---|---|
| T1 | 实体 + Value Object 定义 + 4 个聚合根构造器(无 IO) | 无 | — | 80K tokens |
| T2 | `TenantCommandPort` 5 个方法签名 + 错误码 + Domain Event 定义 | T1 | — | 120K tokens |
| T3 | `TenantQueryPort` 5 个方法签名 + DTO 转换 | T1, T2 | — | 80K tokens |
| T4 | SecurityPolicy 7 天冷却查询接口(`PolicyCoolingQuery`) | T2 | data-design §4.1 security_policy 表 | 60K tokens |
| T5 | ProviderDataBoundary 校验器(数据分类 ↔ no_code_upload) | T2 | security-design §3.4 表第 7-8 行 | 100K tokens |
| T6 | TenantUsageReport 聚合查询(读 WorkItem / AgentSession / Object Storage 字节,需 Service-Internal 跨域查询) | T3 | data-design §11 性能预算 | 180K tokens |
| T7 | 单元测试覆盖(13 类 tenant_id 对象的 RLS 行为矩阵 + 4 个不变量) | T1-T5 | test-design §X 跨租户测试矩阵 | 220K tokens |
| T8 | 集成测试:Platform Admin 端到端(创建 Tenant → 注册 Provider → 替换 SecurityPolicy → Suspend) | T6, T7 | api-design §3.2 | 150K tokens |

**合计估算**: ~990K tokens ≈ 4-5 人·天(AI 协作模式)

## 10. 验收标准(AC)

```gherkin
Feature: Tenant 边界与 SecurityPolicy

  Scenario: 跨 Tenant 访问被拒绝
    Given 用户 A 属于 Tenant X
    When 用户 A 用 Tenant X 的 JWT 访问 GET /v1/tenants/{tenant_y}
    Then 响应 403 SEC-002 (Header 与 JWT 不一致)
    And  响应 403 SEC-007 (Cross-Tenant Access Forbidden)
    And  AuditEvent 记录 actor=A, resource=tenant_y, result=denied

  Scenario: ProviderDataBoundary 拒绝 Code 上传
    Given Tenant Policy 设定 no_code_upload=true
    When 尝试注册 ProviderDataBoundary 含 data_sent=[Code, Diff]
    Then 响应 422 T-003 (no_code_upload 冲突)
    And  SecurityPolicy 边界未被修改

  Scenario: SecurityPolicy 冷却窗口
    Given Tenant 1 在 T0 替换了 SecurityPolicy
    When 同一 Tenant 在 T0 + 6 天尝试再次替换
    Then 响应 422 T-004 (7 天冷却未到)
    And  AuditEvent 记录 attempt=blocked

  Scenario: Tenant Suspend 后跨域查询返回 0 行
    Given Tenant X 状态从 Active → Suspended
    When 任意 13 类对象 (WorkItem / Worktree / etc.) 查询携带 tenant_id=X
    Then RLS 返回 0 行
    And  跨 Tenant X 的 NATS 订阅被 ACL 拒绝

  Scenario: TenantUsageReport 准确性
    Given Tenant X 有 100 WorkItem, 5 AgentSession (24h), 2.3GB Object Storage
    When GET /v1/tenants/{tenant_x}/usage
    Then response.work_item_count == 100
    And response.agent_session_24h_count == 5
    And response.object_storage_bytes == 2.3GB ± 5%
```

## 11. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-016 Local Runtime Compromise | Critical | tenant_id 三重绑定 (tenant+user+project), mTLS 短时 Credential, Revocation 立即生效 | basic-design §4.6.3 |
| RISK-021 Prompt Injection from Repository | Critical | ProviderDataBoundary 强约束,Untrusted-as-Instruct 检测 | basic-design §4.10.7 |
| Tenant 数据泄漏 | Critical | RLS 强制 + AuthorizationChecker 双重 + Object Storage Key 前缀 | basic-design §6.1, security-design §4.1-4.3 |
| SecurityPolicy 越权篡改 | High | 7 天冷却 + Protected 鉴权 + Audit | security-design §3.1 |
| 冷启动期 13 类对象 RLS 漏配 | High | 季度演练 + 自动化测试矩阵 | security-design §3.5.4 |

## 12. Open Issues

- J-T-01: 7 天冷却是否对 Trial 租户豁免?(basic-design §15 J.11 候选)
- J-T-02: ProviderDataBoundary 的 `data_sent` 是否需要在 UI 渲染为图形化确认?(无现成原型)
- J-T-03: TenantUsageReport 的 Object Storage 字节统计,S3 兼容后端 List API 调用频率限制需 PoC 校准(§API-10.3)
- J-T-04: 多 Region 部署时,`allowed_regions[]` 决策是放 Server 端还是 Worker 端?(本 spec 默认 Server 端,待 RFC)
- J-T-05: SecurityPolicy 修改是否需要双签 (Tenant Admin + Platform Admin)?目前仅 Tenant Admin 即可(需 ADR)

## 附录 A:关键流程时序图 — Tenant 创建 + SecurityPolicy 替换

```mermaid
sequenceDiagram
    autonumber
    actor PA as Platform Admin
    participant GW as API Gateway
    participant APP as Application Service
    participant TEN as domain-tenant (Port)
    participant PG as PostgreSQL (tenant schema)
    participant AUD as domain-audit
    participant NATS as NATS JetStream

    PA->>GW: POST /v1/tenants {slug, display_name, plan_tier}
    GW->>GW: 验证 JWT (Platform Admin role)
    GW->>APP: create_tenant(cmd, actor)
    APP->>APP: AuthorizationChecker.check(actor, action=TenantCreate)
    APP->>TEN: TenantCommandPort::create_tenant
    TEN->>TEN: 生成 tenant_id (UUIDv7)
    TEN->>PG: BEGIN; INSERT tenant.tenant; INSERT tenant.tenant_policy
    PG-->>TEN: OK
    TEN->>PG: INSERT outbox (TenantCreated)
    PG-->>TEN: OK
    TEN->>PG: COMMIT
    TEN-->>APP: Tenant
    APP->>AUD: AuditRecorder.record(actor=PA, action=create_tenant, resource=new_tenant)
    APP-->>GW: 201 Created {tenant_id}
    GW-->>PA: 201 Created

    Note over PG,NATS: 1s 内 Outbox Worker 轮询
    PG->>NATS: publish star.events.tenant.tenant.created.v1
    NATS-->>AUD: 订阅 (Audit Append-only)
    NATS-->>worker.projection: 订阅 (Search Index)

    PA->>GW: PUT /v1/tenants/{id}/policies {policy_id}
    GW->>APP: replace_security_policy(cmd, actor)
    APP->>APP: 检查 7 天冷却 (T_TENANT_LAST_REPLACE)
    alt 冷却未到
        APP-->>GW: 422 T-004
        GW-->>PA: 422
    else 冷却已到
        APP->>TEN: TenantCommandPort::replace_security_policy
        TEN->>PG: UPDATE tenant.security_policy
        TEN-->>APP: SecurityPolicy (新 version)
        APP->>AUD: Audit (Protected 鉴权记录)
        APP-->>GW: 200 OK
        GW-->>PA: 200 OK
    end
```

## 附录 B:边界清单

| 边界类型 | 本 Module 行为 |
|---|---|
| 上游依赖 | 无(本 crate 是 25 Module 依赖图的最底层,§2.3 basic-design) |
| 下游调用 | `domain-audit` (Append), `domain-notification` (Tenant Suspended 时), `worker.projection.role` (Search Index) |
| 跨域事务 | 由 `crates/application` 编排,本 crate 仅返回聚合根 |
| RLS 强制 | 全部 4 个聚合根表 + 1 个 Projection 启用 RLS,USING 子句强制 tenant_id 匹配 |
| 13 类 tenant_id 对象 | 覆盖 #1 Repository Credential(本 crate 是 tenant_id 颁发者,Credential 由 Credential Broker 持有,§5.4 security-design),间接覆盖 **#7 AI Prompt / #8 AI Response**(通过 ProviderDataBoundary.data_sent[] 与 no_code_upload 约束) |
| 14 状态 AgentSession 触发 | 无(本 crate 不管理 AgentSession 状态) |
| 17 状态 Worktree 触发 | 无(本 crate 不管理 Worktree 状态) |
| WorkItem 3 态 | 无(WorkItem 由 `domain-work-item` 拥有) |

**接口稳定承诺**:本 spec 锁定的 Port trait 签名、13 类对象覆盖表、不变量 INV-T-01~06 在后续 RFC 阶段不会变更(除非 §15 basic-design Open Issue 解决)。

## 15. 与其他 domain 协作 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md) + [spec/saga/01 v0.2 SagaCoordinationRole](../../architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md),本节定义 `tenant` 与 22 domain 中 5 个 domain 的显式接触面。

| 源 Domain | 目标 Domain | 接触方式 | 接触点 |
|---|---|---|---|
| tenant | identity | Customer-Supplier | TenantMembership / TenantPolicy 校验 (per requirements §16) |
| tenant | workspace | Customer-Supplier | Workspace.tenant_id 引用 (FK) |
| tenant | project | Customer-Supplier | Project.tenant_id 引用 (FK) |
| tenant | audit | Separate Ways | Tenant 创建 / SecurityPolicy 替换事件全量审计 (LRT-001) |

**接触面统计**: 4 条 (v0.16 新增,本 spec 由 `scripts/inter_collab_refine.py` 批量生成)

**dual-use 警告** (per AGENTS.md §5 v0.6 + Q1-D 拍板): 5 域 (player/economy/match/social/admin) 是 RGS 仓历史治理命名,Star 仓不建立业务子域↔DDD 映射。本 spec 协作基于 22 domain crate,不通过 5 域绑定推导。
