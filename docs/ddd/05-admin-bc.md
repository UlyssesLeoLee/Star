# 05-admin-bc 域 DDD BoundedContext 边界 (Admin BoundedContext)

> **Status**: 🟡 占位 (P3-E.7 DDD 边界验证 docs 阶段, 5 域 Lead 真人到位后签字覆盖架构师代签)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **承接**: STAR-P3-5-DOMAIN-LEAD-PROC.md admin 域 + P3-C.7/C.8 + P3-E.4 KMS + cross-domain-5b-mermaid.md §1 admin 域

本文件是 **admin 域** 的 DDD BoundedContext 边界文档, 配合 `docs/architecture/cross-domain-5b-mermaid.md` 5 域 DDD 边界图.

---

## §1 BoundedContext 定义

**admin 域** = RBAC / permission / tenant (per STAR-P3-5-DOMAIN-LEAD-PROC.md admin 域)

- **业务子域**: 多租户 (Tenant) + RBAC 权限 (Permission / Role) + KMS 凭证 (KMS) + 审计治理 (Audit Governance)
- **Aggregate Root**: `Tenant` + `Permission` + `Role` + `KmsKey`
- **核心职责**: 5 域业务子域的多租户隔离 / RBAC 权限 / KMS 凭证 (per P3-E.4 mock 备选) / 跨域 audit 必填

---

## §2 Aggregate 详情

### §2.1 Tenant Aggregate (per P3-C.8 拍板)

**聚合根**: `Tenant` (per `crates/domain-tenant` lib.rs)

- **字段**:
  - `tenant_id: TenantId`
  - `name: String`
  - `slug: String` (URL-friendly 唯一标识)
  - `plan: TenantPlan` (Free / Pro / Enterprise, 值对象)
  - `isolation_mode: IsolationMode` (Schema / Database / Row, per P3-C.7 Postgres)
  - `settings: TenantSettings` (值对象)
  - `status: TenantStatus` (Provisioning / Active / Suspended / Deleted)
  - `created_at / updated_at: DateTime<Utc>`
- **不变量**:
  - **INV-TN-01** Tenant.slug 全局唯一
  - **INV-TN-02** isolation_mode 决定 Postgres schema/database 隔离级别
  - **INV-TN-03** Deleted 状态必填 soft_delete_at + 30 天后 hard delete
- **命令**: `CreateTenant` / `UpdateTenant` / `SuspendTenant` / `SoftDeleteTenant`
- **事件**:
  - `TenantProvisioned` (pub) → player 域 (创建 owner User + Workspace) + economy 域 (创建 BillingAccount) + social 域 (通知 admin) + audit
  - `TenantSuspended` (pub) → player 域 (暂停所有 user) + economy 域 (暂停 BillingAccount) + social 域 (通知 admin) + audit
  - `TenantDeleted` (pub) → player 域 (cleanup) + economy 域 (cleanup) + audit

### §2.2 Permission Aggregate

**聚合根**: `Permission`

- **字段**:
  - `permission_id: PermissionId`
  - `tenant_id: TenantId`
  - `scope: Scope` (Global / Tenant / Project / Workspace, 值对象)
  - `action: Action` (Create / Read / Update / Delete / Admin, 值对象)
  - `resource: Resource` (User / Project / Workflow / Billing / ..., 值对象)
  - `conditions: Vec<Condition>` (per ABAC: 时间 / 地理 / 角色 ... )
  - `created_at: DateTime<Utc>`
- **不变量**:
  - **INV-PRM-01** Permission 必带 tenant_id + scope + action + resource
  - **INV-PRM-02** scope 决定跨租户 / 跨项目 / 跨 Workspace 边界
  - **INV-PRM-03** conditions (per ABAC) 必填后, 授权校验需执行条件检查
- **命令**: `GrantPermission` / `RevokePermission` / `UpdateConditions`
- **事件**:
  - `PermissionGranted` (pub) → audit
  - `PermissionRevoked` (pub) → audit

### §2.3 Role Aggregate

**聚合根**: `Role`

- **字段**:
  - `role_id: RoleId`
  - `tenant_id: TenantId`
  - `name: String` (Owner / Admin / Member / Guest / Custom)
  - `permissions: Vec<PermissionId>` (引用 §2.2 聚合根)
  - `is_system: bool` (系统角色 vs 自定义)
  - `created_at / updated_at: DateTime<Utc>`
- **不变量**:
  - **INV-RL-01** Role 必带 tenant_id + name
  - **INV-RL-02** 系统角色 (is_system=true) 不可删
  - **INV-RL-03** Owner 角色必含所有 permission (full access)
- **命令**: `CreateRole` / `AddPermission` / `RemovePermission` / `DeleteRole` / `AssignRoleToUser`
- **事件**:
  - `RoleAssigned` (pub) → player 域 (更新 user session) + social 域 (通知 user) + audit
  - `RoleRevoked` (pub) → player 域 (更新 user session) + audit

### §2.4 KmsKey Aggregate (per P3-E.4 拍板)

**聚合根**: `KmsKey` (per `crates/domain-kms` LocalMockKms, mock 备选)

- **字段**:
  - `key_id: KeyId`
  - `tenant_id: TenantId`
  - `mode: KmsMode` (LocalMock / Vault / AwsKms, per P3-E.4 mock 备选)
  - `master_key_id: KeyId` (per LocalMockKms 启动时随机)
  - `deks: HashMap<String, Vec<u8>>` (per tenant DEK, process 内)
  - `created_at / last_rotation: DateTime<Utc>`
- **不变量** (per `crates/domain-kms` INV-KMS-01~05):
  - **INV-KMS-01** 唯一入口: 5 域业务子域禁止直接调用外部 KMS SDK, 必须经 KmsClient Port
  - **INV-KMS-02** Envelope encryption: master key (KMS) + DEK (per-tenant), DEK 从不落明文存储
  - **INV-KMS-03** 5 域凭证隔离: tenant_id 必填, 跨租户读取 = 拒绝
  - **INV-KMS-04** 轮换周期: master key 90 天, DEK 30 天, audit log 必填轮换人
  - **INV-KMS-05** 真凭证路径: Vault / AWS KMS 真实 endpoint + key (需 Ulysses 凭证, 跨 session 阻塞)
- **命令**: `GenerateDek` / `Encrypt` / `Decrypt` / `RotateDek` / `Health`
- **事件**:
  - `KmsKeyCreated` (pub) → audit
  - `KmsKeyRotated` (pub) → audit (轮换 actor 必填)
  - `KmsAccessDenied` (pub) → admin 域告警 + audit (跨租户访问尝试)

---

## §3 跨域事件 (Admin 域作为发布者 / 订阅者)

### §3.1 Admin 域发布 (pub) 事件

| 事件 | 订阅域 | 订阅方职责 |
|---|---|---|
| `TenantProvisioned` | player / economy / social / audit | player 域 创建 owner + workspace; economy 域 BillingAccount; social 域通知; audit 必填 |
| `TenantSuspended` | player / economy / social / audit | player 域 暂停所有 user; economy 域 暂停 BillingAccount; social 域通知; audit 必填 |
| `TenantDeleted` | player / economy / audit | player 域 cleanup; economy 域 cleanup; audit 必填 |
| `PermissionGranted` / `PermissionRevoked` | audit | audit 必填 |
| `RoleAssigned` | player / social / audit | player 域 更新 user session; social 域通知; audit 必填 |
| `RoleRevoked` | player / audit | player 域 更新 user session; audit 必填 |
| `KmsKeyCreated` / `KmsKeyRotated` | audit | audit 必填 |
| `KmsAccessDenied` | (admin 内部告警) + audit | 跨租户访问尝试告警; audit 必填 |

### §3.2 Admin 域订阅 (sub) 事件

| 事件 | 发布域 | Admin 域职责 |
|---|---|---|
| `UserCreated` (player 域) | player | 创建默认 Owner Role + 初始化 RBAC (per §2.3) |
| `WorkspaceProvisioned` (player 域) | player | 创建 Workspace-scoped Permission |
| `PaymentFailed` (economy 域) | economy | 暂停 Tenant (per INV-TN-01) + 触发 SuspendAccountWorkflow |
| `ApiKeyRevoked` (economy 域) | economy | 凭证失效处理 (轮换 + 通知 user) |
| `WorkflowStarted` (match 域) | match | audit 必填 (per 9 AI Audit 必填字段) |
| `SagaCompleted` (match 域) | match | audit 必填 |
| `WorkflowFailed` (match 域) | match | audit 必填 + Saga 失败告警 |
| `NotificationFailed` (social 域) | social | retry 告警 + audit |

---

## §4 Cargo Crate 引用 (per main HEAD `ccf27fc`)

| 域 | Cargo Crate | 路径 | Lead 域 |
|---|---|---|---|
| admin | `domain-tenant` | `crates/domain-tenant/` (per P3-C.8 拍板) | tenant Lead |
| admin | `domain-permission` | `crates/domain-permission/` | permission Lead |
| admin | `domain-kms` | `crates/domain-kms/` (per P3-E.4 拍板, LocalMockKms) | kms Lead |
| admin | `domain-audit` (跨域) | `crates/domain-audit/` (admin 域拥有, 5 域 必填) | audit Lead (admin 域跨域所有权) |
| admin | `infrastructure` (Postgres) | `crates/infrastructure/` (per P3-C.7 Postgres schema/database/row 隔离) | infrastructure Lead |

**注**: admin 域**没有**专属 `domain-admin` crate, tenant / permission / kms / audit 4 Aggregate 散在 `domain-tenant` + `domain-permission` + `domain-kms` + `domain-audit`. 5 域 Lead 真人到位后, 可考虑整合 (per P3-E.7 DDD 边界验证 phase 2).

**特别说明**: `domain-audit` 是 admin 域**跨域拥有**的 Aggregate (per 9 AI Audit 必填字段), 5 域都调用 AuditRecorder Port 写, 但 admin 域拥有唯一 Append-only 写权限 (per domain-audit §1 INV-AU-01).

---

## §5 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | admin 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束), 本 doc 由架构师代签 | 跨 session 续, admin 域 Lead 真人追溯签字 |
| 2 | E.4 KMS 真凭证路径 (Vault / AWS KMS, per §2.4 INV-KMS-05) 等 Ulysses 凭证到位切真 | admin 域 Lead 真人到位后 |
| 3 | ABAC conditions 详细 schema (per §2.2 INV-PRM-03) 待 admin 域 Lead 真人补 | admin 域 Lead 真人到位后 |
| 4 | Tenant isolation_mode 详细 schema/database/row 切换 (per §2.1 INV-TN-02 + P3-C.7) 待 admin 域 Lead 真人拍板 | admin 域 Lead 真人到位后 |
| 5 | Role 继承 / 复合 (per §2.3) 详细机制待 admin 域 Lead 真人拍板 (e.g. Owner 包含 Admin 权限?) | admin 域 Lead 真人到位后 |
| 6 | KMS 轮换策略 (per §2.4 INV-KMS-04) 90/30 天周期 + audit 必填 待 admin 域 Lead 真人补详细实现 | admin 域 Lead 真人到位后 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | admin 域 Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟡 应急代签; admin 域 BoundedContext + 4 Aggregate (Tenant / Permission / Role / KmsKey) + 跨域事件 8 pub + 8 sub + Cargo crate 引用 (散在 domain-tenant + domain-permission + domain-kms + domain-audit) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: admin 域 BoundedContext + 4 Aggregate (Tenant / Permission / Role / KmsKey) + 跨域事件 8 pub + 8 sub + Cargo crate 引用 + 已知缺口 6 项 | 2026-08-30 08:55 JST 5 域 DDD 边界 docs 落地触发 |
