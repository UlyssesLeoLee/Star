# 01-player-bc 域 DDD BoundedContext 边界 (Player BoundedContext)

> **Status**: 🟡 占位 (P3-E.7 DDD 边界验证 docs 阶段, 5 域 Lead 真人到位后签字覆盖架构师代签)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签 (per 8/27 19:39 JST 用户授权)
> **承接**: STAR-P3-5-DOMAIN-LEAD-PROC.md player 域 + P3-C.2/C.3/C.8 拍板 + cross-domain-5b-mermaid.md §1 player 域

本文件是 **player 域** 的 DDD BoundedContext 边界文档, 配合 `docs/architecture/cross-domain-5b-mermaid.md` 5 域 DDD 边界图.

---

## §1 BoundedContext 定义

**player 域** = 用户 / 身份 / 工作空间 (per STAR-P3-5-DOMAIN-LEAD-PROC.md player 域)

- **业务子域**: 用户管理 (User Management) + 身份认证 (Identity Auth) + 工作空间 (Workspace) + 设备绑定 (Device Binding)
- **Aggregate Root**: `User` + `Workspace` + `Device`
- **核心职责**: 5 域业务子域 (player / economy / match / social / admin) 的用户身份 / 多租户 / 工作空间生命周期

---

## §2 Aggregate 详情

### §2.1 User Aggregate

**聚合根**: `User` (per `crates/domain-identity` lib.rs INV-ID-01)

- **字段**:
  - `user_id: UserId` (UUID v7)
  - `tenant_id: TenantId` (per INV-ID-01: User 必带 tenant_id)
  - `email: Email` (值对象)
  - `display_name: String`
  - `status: UserStatus` (Active / Suspended / Deleted)
  - `created_at / updated_at: DateTime<Utc>`
- **不变量** (per domain-identity INV-ID-01~04):
  - **INV-ID-01** User 必带 tenant_id (跨租户隔离)
  - **INV-ID-02** Device 三重绑定 tenant+user+project (LRT-001/002)
  - **INV-ID-03** Credential 仅存 hash / ref, 绝不存明文
  - **INV-ID-04** Device 状态机 Active / Revoked / Pending
- **命令**: `CreateUser` / `UpdateUser` / `SuspendUser` / `DeleteUser`
- **事件**:
  - `UserCreated` (pub) → social 域 Notification + admin 域 Role + audit
  - `UserUpdated` (pub) → social 域 (mention / display name)
  - `UserSuspended` (pub) → admin 域 (RBAC 暂停) + audit

### §2.2 Workspace Aggregate

**聚合根**: `Workspace` (per `crates/domain-workspace` lib.rs)

- **字段**:
  - `workspace_id: WorkspaceId`
  - `tenant_id: TenantId`
  - `name: String`
  - `owner_user_id: UserId`
  - `members: Vec<WorkspaceMember>`
  - `settings: WorkspaceSettings` (值对象)
  - `status: WorkspaceStatus` (Provisioning / Active / Archived)
  - `created_at / updated_at: DateTime<Utc>`
- **不变量**:
  - **INV-WS-01** Workspace 必带 tenant_id + owner
  - **INV-WS-02** Workspace 跨 5 域共享 (player / economy / match / social / admin 全部监听 WorkspaceProvisioned 事件)
  - **INV-WS-03** members 列表 owner 不可移除 (除非 owner 转让)
- **命令**: `ProvisionWorkspace` / `AddMember` / `RemoveMember` / `ArchiveWorkspace` / `TransferOwnership`
- **事件**:
  - `WorkspaceProvisioned` (pub) → admin 域 RBAC 初始化 + economy 域 BillingAccount 创建
  - `WorkspaceMemberAdded` (pub) → social 域通知
  - `WorkspaceArchived` (pub) → admin 域 (soft delete)

### §2.3 Device Aggregate

**聚合根**: `Device` (per `crates/domain-identity` DeviceBinding)

- **字段**:
  - `device_id: DeviceId`
  - `tenant_id + user_id + project_id: 3-tuple` (per INV-ID-02)
  - `platform: Platform` (Local Runtime / CLI / Web)
  - `binding: DeviceBinding`
  - `status: DeviceStatus` (Active / Revoked / Pending)
  - `last_seen_at: DateTime<Utc>`
- **不变量**:
  - **INV-ID-02** 3-tuple 必填, 跨租户/跨用户/跨项目读取 = 拒绝
  - **INV-ID-04** 状态机 Active / Revoked / Pending (per `crates/domain-identity` INV-ID-04)
- **命令**: `RegisterDevice` / `RevokeDevice` / `UpdateLastSeen`
- **事件**:
  - `DeviceRegistered` (pub) → audit
  - `DeviceRevoked` (pub) → admin 域 (强制登出) + audit

---

## §3 跨域事件 (Player 域作为发布者 / 订阅者)

### §3.1 Player 域发布 (pub) 事件

| 事件 | 订阅域 | 订阅方职责 |
|---|---|---|
| `UserCreated` | social / admin / audit | social 域 通知 owner; admin 域 初始化 RBAC role; audit 必填 |
| `UserUpdated` | social | mention 列表更新 |
| `UserSuspended` | admin / audit | admin 域 RBAC 暂停; audit 必填 |
| `WorkspaceProvisioned` | admin / economy / audit | admin 域 RBAC 初始化 (owner role); economy 域 创建 BillingAccount; audit 必填 |
| `WorkspaceMemberAdded` | social | 通知新成员 |
| `WorkspaceArchived` | admin | 软删除 RBAC + 数据 |
| `DeviceRegistered` / `DeviceRevoked` | admin / audit | admin 域 强制登出 / 凭证刷新; audit 必填 |

### §3.2 Player 域订阅 (sub) 事件

| 事件 | 发布域 | Player 域职责 |
|---|---|---|
| `TenantProvisioned` (admin 域) | admin | 创建 owner User + Workspace |
| `RoleAssigned` (admin 域) | admin | 更新 user session 权限 (per `crates/domain-identity` Session entity) |
| `WorkflowCompleted` (match 域) | match | 更新 user last-activity timestamp |

---

## §4 Cargo Crate 引用 (per main HEAD `ccf27fc`)

| 域 | Cargo Crate | 路径 | Lead 域 |
|---|---|---|---|
| player | `domain-identity` | `crates/domain-identity/` | identity Lead |
| player | `domain-workspace` | `crates/domain-workspace/` | workspace Lead |
| player | `domain-tenant` | `crates/domain-tenant/` (跨 player/admin) | tenant Lead |
| player | `domain-audit` (只读) | `crates/domain-audit/` (player 域调用 `AuditRecorder` Port) | audit Lead |

**注**: player 域不直接拥有 `domain-audit` 写权限, 仅调用 Port 写事件 (per domain-audit §1 唯一 Append-only Domain).

---

## §5 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | player 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束), 本 doc 由架构师代签 | 跨 session 续, player Lead 真人追溯签字 |
| 2 | INV-ID-02 3-tuple 详细字段 schema 待 5 域 Lead 真人补 | player 域 Lead 真人到位后 |
| 3 | Device 三重绑定 tenant+user+project (LRT-001/002) 详细生命周期 5 域 Lead 真人补 | player 域 Lead 真人到位后 |
| 4 | 跨域事件投递可靠性 (at-least-once vs exactly-once) 待 5 域 Lead 真人拍板 | match 域 Lead (跨域编排) 真人到位后 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | player 域 Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟡 应急代签; player 域 BoundedContext + 3 Aggregate + 跨域事件 7 pub + 3 sub |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: player 域 BoundedContext + 3 Aggregate (User / Workspace / Device) + 跨域事件 7 pub + 3 sub + Cargo crate 引用 + 已知缺口 4 项 | 2026-08-30 08:55 JST 5 域 DDD 边界 docs 落地触发 (per 守门 #12 v15 派生饱和: P3 跨阶段 INC-SESSION-004 收官是新事件) |
