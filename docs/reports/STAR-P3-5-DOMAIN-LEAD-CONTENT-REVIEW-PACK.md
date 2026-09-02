# STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK 5 域 Lead 真人 review 完整内容确认包 (一站式操作手册)

> **Status**: 🟡 Draft v0.1 (本文件是 5 域 Lead 真人到位后**直接可执行**的"操作手册", 整合 13 docs 摘要 + 5 域 DDD docs 详情 + 5 角色 + 5 域 Lead 10 真人 checklist + 36 commits 11.3 小时预算 + 4 阻塞跨 session 续做列表)
> **Created**: 2026-08-30 11:13 JST
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签 (per 8/27 19:39 JST 用户授权)
> **承接**: STAR-P3-5-DOMAIN-LEAD-PROC.md v0.2 (177 lines) + REGISTRY v0.1 + REVIEW-CHECKLIST v0.1 (124 lines) + REVIEW-PROTOCOL v0.1 (200 lines) + DDD-REVIEW-PHASE v0.1 (112 lines) + E7-SIGN-OFF-TEMPLATE v0.1 (106 lines) + F5-5OF5-CHECKLIST v0.1 (147 lines) + F5-5OF5-EMPIRICAL v0.1 (134 lines) + E6-SAGA-IMPL-REPORT v0.1 (157 lines) + 5 域 DDD docs (44.6KB markdown) + 6 份 P3 报告 (34.3KB) + 2 份 INC-SESSION (23.8KB) = **13 份 docs + 6 份报告 + 2 份 INC-SESSION = 21 份 review 目标**

---

## §0 一句话硬约束 (per 2026-08-30 09:08 JST Ulysses 反馈)

> **可以代签 Ulysses, 不可以编造历史. 5 域 Lead 真人到位后, 签字栏 #1 追溯签字必须用真人真实身份 (email + 到岗日 + 评审结论), 不再代签. (per 8/27 19:39 JST 用户授权反转)**

---

## §1 真人到位流程 (5 步, per `STAR-P3-5-DOMAIN-LEAD-PROC.md` v0.2)

### 步骤 1: Ulysses 找 5 个真人

- **方法 A (推荐)**: Ulysses 个人网络 / 公司内部 5 个工程师, 每人认领 1 域, 签署 DDD Review 协议
- **方法 B (备选)**: freelance 平台 (Toptal / Upwork) 找 5 个 Rust 工程师
- **方法 C (备选)**: 开源社区招募

**硬约束 (per 8/21 JST Ulysses 拍板)**: 不接受兼任 — 架构师不能兼任 player Lead, SRE 不能兼任 admin Lead. 5 个真人 = 5 个独立个体.

### 步骤 2: 5 域 Lead 注册

文件: `STAR-P3-5-DOMAIN-LEAD-REGISTRY.md` §1 表, 5 行:

| 域 | Lead 姓名 | 邮箱 | 角色 | 域边界 docs | 到岗日期 | 状态 |
|---|---|---|---|---|---|---|
| **player** (用户/identity/workspace) | `<待填>` | `<待填>` | Player Lead | `docs/ddd/01-player-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |
| **economy** (billing/pricing/cost) | `<待填>` | `<待填>` | Economy Lead | `docs/ddd/02-economy-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |
| **match** (workflow/状态机/saga) | `<待填>` | `<待填>` | Match Lead | `docs/ddd/03-match-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |
| **social** (collaboration/通知) | `<待填>` | `<待填>` | Social Lead | `docs/ddd/04-social-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |
| **admin** (RBAC/permission/tenant) | `<待填>` | `<待填>` | Admin Lead | `docs/ddd/05-admin-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |

落地 1 commit: `docs(governance): STAR-P3-5-DOMAIN-LEAD-REGISTRY.md 5 域 Lead 真人到位 (5 行填入)`.

### 步骤 3: 5 域 Lead review 域边界 docs (6 章节 × 5 份 = 30 项)

每域真人 30-60 分钟, 5 域 = 2.5-5 小时 (per STAR-OLU-001 §6 质量门 5 维). 6 章节 review 速查 (per `STAR-P3-5-DOMAIN-LEAD-REVIEW-CHECKLIST.md` §0.3):

```
[ ] §1 BoundedContext 业务子域 + Aggregate Root 划分合理
[ ] §2 Aggregate 字段类型 / 索引 / 约束 (per §5 已知缺口)
[ ] §3 跨域事件 schema + at-least-once / exactly-once 投递
[ ] §4 Cargo crate 引用 (散落 vs 独立 crate 拍板)
[ ] §5 已知缺口完整 (含 #1-#6 域内细节)
[ ] §6 签字栏 #1 域 Lead 真人签字 (覆盖架构师代签)
```

5 域 Lead 落地 5 commits (1 域 1 commit per docs).

### 步骤 4: 5 域 Lead review 6 份 P3 报告 (6 章节 × 6 份 = 36 项)

6 章节 review 速查 (per `STAR-P3-5-DOMAIN-LEAD-REVIEW-CHECKLIST.md` §0.2):

```
[ ] §0 目的: 拍板包承接 + 触发事件 + 范围清晰
[ ] §1 改动: 文件路径 + 行数 + commit short hash 实证
[ ] §2 验证: 守门 #1+#9+#12+#8+#15 跨 stage 全过
[ ] §3 缺口: 真人到位 / 凭证 / 集成测试 完整列
[ ] §4 子代理: 0 子代理调用 (守门 #9 RPC 不可靠实证)
[ ] §6 签字栏: 5 角色 + 架构师代签 (per ec6dee0)
```

6 份 P3 报告 review 落地 6 commits (1 报告 1 commit per docs).

### 步骤 5: 跨 session 续做 + 真人到位验收 (7 项 checklist)

```markdown
1. [ ] 5 域 Lead 真人到位, 每人 1 域 (per §1 拒绝兼任硬约束)
2. [ ] 5 域 Lead review 5 份 DDD 边界 docs (per §3 已实装), 签字栏 #1 追溯签字
3. [ ] 5 域 Lead review 6 份 P3 阶段收官报告, 签字栏 #1 追溯签字覆盖架构师代签
4. [ ] 5 域 Lead review PHASE-P3-CROSS-STAGE-INC-SESSION-003.md / -004.md, 签字栏 5 角色全部追溯签字
5. [ ] E.6 Saga 跨域编排 (per match 域 Lead 真人补详细补偿机制) phase 2 启动
6. [ ] DDD Review 阶段 5 域 Lead 真人 + SRE Lead + 平台 + 评审 + PM 5 角色真人到位 (per AGENTS §3 模板)
7. [ ] 质量门 5/5 (per STAR-OLU-001 §6) 实证: 功能完整 / 测试覆盖 / 守门 0 违反 / 文档同步 / git 证据
```

**总 commits 落地**: 1 (REGISTRY) + 5 (5 域 DDD docs) + 6 (6 份 P3 报告) + 2 (2 份 INC-SESSION) = **14 commits** (per `STAR-P3-E7-SIGN-OFF-TEMPLATE.md` §3).

---

## §2 5 域 DDD 边界 docs review 详情 (5 域 × 6 章节 = 30 项)

### §2.1 Player 域 (`docs/ddd/01-player-bc.md`, 7.4KB)

**业务子域**: 用户管理 (User Management) + 身份认证 (Identity Auth) + 工作空间 (Workspace) + 设备绑定 (Device Binding)
**Aggregate Root**: `User` + `Workspace` + `Device` (3 Aggregate)
**跨域事件**: 7 pub + 3 sub
**Cargo crate 引用**: `domain-identity` + `domain-workspace` + `domain-tenant` (跨 player/admin) + `domain-audit` (只读, 调 Port)

**5 项 review (player Lead 真人)**:

1. [ ] §1 BoundedContext 业务子域 + Aggregate Root 划分合理 (用户 / 工作空间 / 设备 3 域核心边界清晰)
2. [ ] §2 3 Aggregate 字段类型 / 索引 / 约束
   - User: 6 字段 (user_id/tenant_id/email/display_name/status/timestamps) + 4 不变量 (INV-ID-01~04)
   - Workspace: 8 字段 + 3 不变量 (INV-WS-01~03)
   - Device: 6 字段 + 2 不变量 (INV-ID-02 三重绑定 + INV-ID-04 状态机)
3. [ ] §3 7 pub 事件 (UserCreated / UserUpdated / UserSuspended / WorkspaceProvisioned / WorkspaceMemberAdded / WorkspaceArchived / DeviceRegistered/DeviceRevoked) + 3 sub 事件 (TenantProvisioned / RoleAssigned / WorkflowCompleted)
4. [ ] §4 Cargo crate 引用: player 域不直接拥有 `domain-audit` 写权限, 仅调 Port 写 (per domain-audit §1 INV-AU-01 唯一 Append-only Domain). 是否需独立 `domain-player` crate phase 2 拍板.
5. [ ] §5 已知缺口 4 项完整: #1 真人到位 + #2 INV-ID-02 3-tuple 详细字段 + #3 Device 三重绑定 (LRT-001/002) + #4 跨域事件投递可靠性 (at-least-once vs exactly-once)

**签字栏 #1**: `**player 域 Lead**: <player Lead 姓名> | <签字日 2026-XX-XX> | 🟢 player 域 review pass; 3 Aggregate (User / Workspace / Device) + 7 pub + 3 sub 跨域事件 + 6 章节全过; 签字栏 #1 追溯`

### §2.2 Economy 域 (`docs/ddd/02-economy-bc.md`, 9.2KB)

**业务子域**: 计费账户 + 订阅 + 发票 + API 凭证 + 成本核算
**Aggregate Root**: `BillingAccount` + `Subscription` + `Invoice` + `ApiKey` (4 Aggregate)
**跨域事件**: 7 pub + 4 sub
**Cargo crate 引用**: `domain-project` (含 BillingAccount/Subscription) + `domain-cli` (含 ApiKey 双模式) + `domain-audit` (只读) + `domain-kms` (Phase 2 真凭证)

**5 项 review (economy Lead 真人)**:

1. [ ] §1 BoundedContext 业务子域 + Aggregate Root 划分 (BillingAccount / Subscription / Invoice / ApiKey 4 域核心边界清晰)
2. [ ] §2 4 Aggregate 字段类型 / 索引 / 约束
   - BillingAccount: 7 字段 + 3 不变量 (INV-BL-01~03)
   - Subscription: 8 字段 + 3 不变量 (INV-SUB-01~03)
   - Invoice: 8 字段 + 3 不变量 (INV-INV-01~03)
   - ApiKey: 7 字段 + 3 不变量 (INV-AK-01~03, 双模式 storage)
3. [ ] §3 7 pub 事件 (BillingAccountCreated / InvoiceIssued / PaymentFailed / SubscriptionCreated / SubscriptionCancelled / ApiKeyRotated / ApiKeyRevoked) + 4 sub 事件 (WorkspaceProvisioned / UserSuspended / TenantProvisioned / RoleAssigned)
4. [ ] §4 Cargo crate 引用: economy 域**没有**专属 `domain-economy` crate, 4 Aggregate 散在 `domain-project` + `domain-cli`. 是否拆分独立 crate phase 2 拍板.
5. [ ] §5 已知缺口 6 项完整: #1 真人到位 + #2 拆分 crate 拍板 + #3 Invoice line items 详细 schema + #4 cross-currency 汇率表数据源 + #5 ApiKey 默认 mode (per tenant 配置?) + #6 Invoice 合规 (各国税法)

**签字栏 #1**: `**economy 域 Lead**: <economy Lead 姓名> | <签字日> | 🟢 economy 域 review pass; 4 Aggregate + 7 pub + 4 sub 6 章节全过; 签字栏 #1 追溯 (含 INV-BL-01~03 不变量)`

### §2.3 Match 域 (`docs/ddd/03-match-bc.md`, 8.8KB)

**业务子域**: 工作流定义 + 工作流执行 + Saga 编排 + 状态机 + 跨域补偿
**Aggregate Root**: `Workflow` + `WorkflowInstance` + `SagaInstance` (3 Aggregate)
**跨域事件**: 7 pub + 5 sub
**Cargo crate 引用**: `domain-workflow` + `domain-work-item` + `star-saga` + `domain-audit` (只读) + `domain-cli` (API→CLI fallback)

**5 项 review (match Lead 真人)**:

1. [ ] §1 BoundedContext 业务子域 + Aggregate Root 划分 (Workflow / WorkflowInstance / SagaInstance 3 域核心边界清晰)
2. [ ] §2 3 Aggregate 字段类型 / 索引 / 约束
   - Workflow: 8 字段 + 3 不变量 (INV-WF-01~03, 含 DAG 校验)
   - WorkflowInstance: 8 字段 + 3 不变量 (INV-WFI-01~03, 含 Append-only state_history)
   - SagaInstance: 7 字段 + 3 不变量 (INV-SG-01~03, 含 idempotency)
3. [ ] §3 7 pub 事件 (WorkflowPublished / WorkflowStarted / StateTransitioned / WorkflowCompleted / WorkflowFailed / SagaCompleted / SagaFailed / SagaCompensated) + 5 sub 事件 (UserCreated / WorkspaceProvisioned / PaymentFailed / NotificationDispatched / TenantProvisioned)
4. [ ] §4 Cargo crate 引用: match 域**没有**专属 `domain-match` crate, 3 Aggregate 散在 `domain-workflow` + `domain-work-item` + `star-saga`. 是否整合 phase 2 拍板.
5. [ ] §5 已知缺口 5 项完整: #1 真人到位 + #2 **E.6 Saga 跨域编排详细补偿机制 (at-least-once / exactly-once / idempotency key / 补偿链顺序, per 跨域 Saga 流程 F.4 §2 alt 路径, match 域 Lead 必补)** + #3 WorkflowInstance 状态机版本控制详细 schema + #4 Saga idempotency 详细策略 + #5 状态机 DAG 校验实现细节

**签字栏 #1**: `**match 域 Lead**: <match Lead 姓名> | <签字日> | 🟢 match 域 review pass; 3 Aggregate + 7 pub + 5 sub 6 章节全过; 签字栏 #1 追溯 (含 E.6 Saga 详细补偿机制 + 跨域 Saga 流程 F.4 §2)`

### §2.4 Social 域 (`docs/ddd/04-social-bc.md`, 8.9KB)

**业务子域**: 通知 (Notification) + 评论 (Comment) + mention + 协作 (Collaboration) + 跨域事件总线
**Aggregate Root**: `Notification` + `Comment` + `Mention` (3 Aggregate, Mention 值对象-like)
**跨域事件**: 7 pub + 12 sub (5 域全部监听 NotificationDispatched)
**Cargo crate 引用**: `domain-notification` (42KB) + `domain-comment` + `domain-collaboration` + `domain-audit` (只读) + `domain-search` (P3-E.3)

**5 项 review (social Lead 真人)**:

1. [ ] §1 BoundedContext 业务子域 + Aggregate Root 划分 (Notification / Comment / Mention 3 域核心边界清晰)
2. [ ] §2 3 Aggregate 字段类型 / 索引 / 约束
   - Notification: 12 字段 + 4 不变量 (INV-NOT-01~04, 含 30 天 retention)
   - Comment: 11 字段 + 3 不变量 (INV-CMT-01~03, 含 Append-only 编辑)
   - Mention: 3 字段 (值对象-like, 不独立成 Aggregate)
3. [ ] §3 7 pub 事件 (NotificationDispatched 5 域全部监听 / NotificationDelivered / NotificationFailed / CommentPosted / CommentEdited / CommentDeleted / MentionTriggered) + 12 sub 事件 (UserCreated / WorkspaceProvisioned / WorkspaceMemberAdded / UserSuspended / InvoiceIssued / PaymentFailed / WorkflowStarted/Completed/Failed / SagaCompleted/Failed / TenantProvisioned / RoleAssigned)
4. [ ] §4 Cargo crate 引用: social 域**没有**专属 `domain-social` crate, 3 Aggregate 散在 3 个 domain crate. 是否整合 phase 2 拍板.
5. [ ] §5 已知缺口 5 项完整: #1 真人到位 + #2 **5 域 notification template (per §3.2 12 订阅事件) 详细文案 + i18n (social 域 Lead 必补)** + #3 notification 投递可靠性 (at-least-once vs exactly-once, 跨 match 域 Lead 拍板) + #4 30 天 retention 策略实现 (cron? lazy expire?) + #5 跨域事件总线架构 (in-process channel? external broker?)

**签字栏 #1**: `**social 域 Lead**: <social Lead 姓名> | <签字日> | 🟢 social 域 review pass; 3 Aggregate + 7 pub + 12 sub 6 章节全过; 签字栏 #1 追溯 (含 5 域 notification template 12 订阅事件)`

### §2.5 Admin 域 (`docs/ddd/05-admin-bc.md`, 10.3KB)

**业务子域**: 多租户 + RBAC 权限 + KMS 凭证 + 审计治理
**Aggregate Root**: `Tenant` + `Permission` + `Role` + `KmsKey` (4 Aggregate)
**跨域事件**: 8 pub + 8 sub
**Cargo crate 引用**: `domain-tenant` + `domain-permission` + `domain-kms` (LocalMockKms) + `domain-audit` (跨域拥有) + `infrastructure` (Postgres)

**5 项 review (admin Lead 真人)**:

1. [ ] §1 BoundedContext 业务子域 + Aggregate Root 划分 (Tenant / Permission / Role / KmsKey 4 域核心边界清晰)
2. [ ] §2 4 Aggregate 字段类型 / 索引 / 约束
   - Tenant: 8 字段 + 3 不变量 (INV-TN-01~03, 含 isolation_mode Schema/Database/Row)
   - Permission: 7 字段 + 3 不变量 (INV-PRM-01~03, 含 ABAC conditions)
   - Role: 6 字段 + 3 不变量 (INV-RL-01~03, 含 Owner 包含全部 permission)
   - KmsKey: 7 字段 + 5 不变量 (INV-KMS-01~05, 含 Envelope encryption + 90/30 天轮换)
3. [ ] §3 8 pub 事件 (TenantProvisioned / TenantSuspended / TenantDeleted / PermissionGranted/Revoked / RoleAssigned/Revoked / KmsKeyCreated/Rotated / KmsAccessDenied) + 8 sub 事件 (UserCreated / WorkspaceProvisioned / PaymentFailed / ApiKeyRevoked / WorkflowStarted / SagaCompleted / WorkflowFailed / NotificationFailed)
4. [ ] §4 Cargo crate 引用: admin 域**没有**专属 `domain-admin` crate, 4 Aggregate 散在 4 个 domain crate. `domain-audit` 是 admin 域**跨域拥有**的 Aggregate (per 9 AI Audit 必填字段), 5 域都调 AuditRecorder Port 写, 但 admin 域拥有唯一 Append-only 写权限. 是否整合 phase 2 拍板.
5. [ ] §5 已知缺口 6 项完整: #1 真人到位 + #2 **E.4 KMS 真凭证路径 (Vault / AWS KMS, per §2.4 INV-KMS-05, 等 Ulysses 凭证到位切真, admin 域 Lead 必补)** + #3 ABAC conditions 详细 schema + #4 Tenant isolation_mode 详细 schema/database/row 切换 + #5 Role 继承/复合机制 + #6 KMS 轮换策略 90/30 天周期 + audit 必填详细实现

**签字栏 #1**: `**admin 域 Lead**: <admin Lead 姓名> | <签字日> | 🟢 admin 域 review pass; 4 Aggregate + 8 pub + 8 sub 6 章节全过; 签字栏 #1 追溯 (含 E.4 KMS 真凭证 + ABAC conditions + KMS 轮换策略)`

---

## §3 6 份 P3 阶段收官报告 review 详情 (6 报告 × 6 章节 = 36 项)

### §3.1 `PHASE-P3-C1-IMPL-REPORT.md` (5.3KB, C.1 Workspace 域 收官, commit `f93d909`)

- [ ] §0 目的: 承接 STAR-P3-C-DECISION-PACK.md C.1 拍板, 触发 2026-08-30 08:18 JST, C.1 Workspace 域 范围
- [ ] §1 改动: `crates/domain-workspace/src/lib.rs` 增强, commit `f93d909` 实证
- [ ] §2 验证: cargo check 0 err, tsc 0 错, release cargo test 41/41 crate 0 fail
- [ ] §3 缺口: 列 5 域 Lead 真人到位 / Postgres 持久层 / 跨域 Saga 等缺口
- [ ] §4 子代理: 0 子代理调用, root 直实装
- [ ] §6 签字栏: 5 角色签字栏 + 架构师代签 (per `ec6dee0`), player 域 Lead 跨域 review 签字

### §3.2 `PHASE-P3-C2-C5-IMPL-REPORT.md` (5.7KB, C.2-C.5 4 子项 batch, commit `81de99a`)

- [ ] §0 目的: 承接 C.2-C.5 拍板, 触发 2026-08-30 08:27 JST
- [ ] §1 改动: `domain-project` + `domain-identity` + `domain-work-item` + `domain-workflow` 4 crate 增强, commit `81de99a`
- [ ] §2 验证: cargo check 0 err (9.95s 缓存命中)
- [ ] §3 缺口: 列 5 域 Lead / 4 域实装跨 crate 集成 / Saga 跨域编排 等缺口
- [ ] §4 子代理: 0 子代理调用
- [ ] §6 签字栏: 5 角色 + 架构师代签, 5 域 Lead 跨域 review 签字

### §3.3 `PHASE-P3-C6-C8-IMPL-REPORT.md` (5.4KB, C.6-C.8 3 子项 batch, commit `25d086e`)

- [ ] §0 目的: 承接 C.6-C.8 拍板, 触发 2026-08-30 08:30 JST
- [ ] §1 改动: `star-saga` + `infrastructure` + `domain-tenant` 3 crate 增强, commit `25d086e`
- [ ] §2 验证: cargo check 0 err
- [ ] §3 缺口: 列 5 域 Lead / 跨域 Saga / DDD 边界 等缺口
- [ ] §4 子代理: 0 子代理调用
- [ ] §6 签字栏: 5 角色 + 架构师代签, 5 域 Lead 跨域 review 签字

### §3.4 `PHASE-P3-D1-D7-IMPL-REPORT.md` (5.2KB, D.1-D.7 7 子项 batch, commit `8ace1d5` + merge `55006a0`)

- [ ] §0 目的: 承接 D.1-D.7 拍板, 触发 2026-08-30 08:32 JST
- [ ] §1 改动: w28 切 HubCliRuntime 入口 + 跨平台 e2e + Playwright + realFetch wrapper + 3 handler real-mode + markdownlint+cargo doc CI + UserMenu 状态条 7 子项, commit `8ace1d5` + merge `55006a0`
- [ ] §2 验证: cargo check 0 err (8.38s, 19 warning pre-existing)
- [ ] §3 缺口: D.2 跨平台 e2e + D.6 markdownlint + cargo doc CI 真实 runner 配置 stub
- [ ] §4 子代理: 0 子代理调用
- [ ] §6 签字栏: 5 角色 + 架构师代签, 5 域 Lead 跨域 review 签字

### §3.5 `PHASE-P3-E1-E4-IMPL-REPORT.md` (6.1KB, E.1-E.4 4 子项 batch, commit `5ea9611` + merge `d2e2a99`)

- [ ] §0 目的: 承接 E.1-E.4 拍板, 触发 2026-08-30 08:36 JST
- [ ] §1 改动: `domain-audit` + `domain-notification` + `domain-search` 3 域 + `crates/domain-kms` 新建 (LocalMockKms + 5 不变量 + 3 单测), commit `5ea9611` + merge `d2e2a99`
- [ ] §2 验证: cargo check 0 err (0.80s cache 命中, 42/42 crate)
- [ ] §3 缺口: E.4 KMS 真凭证路径 / E.5 真人 / E.6 Saga / E.7 DDD 边界 等缺口
- [ ] §4 子代理: 0 子代理调用
- [ ] §6 签字栏: 5 角色 + 架构师代签, 5 域 Lead 跨域 review 签字 (含 E.4 KMS mock 备选)

### §3.6 `PHASE-P3-F1-F5-IMPL-REPORT.md` (6.6KB, F.2-F.5 4 子项 batch, commit `6c1bd6c` + merge `93512a9`)

- [ ] §0 目的: 承接 F.2-F.5 拍板, 触发 2026-08-30 08:55 JST
- [ ] §1 改动: `frontend/e2e/cross-domain-5b.spec.ts` 3 Playwright test + `CHANGELOG.md` + `docs/architecture/cross-domain-5b-mermaid.md` + `docs/governance/P3-quality-gate-5d.md` 4 deliverable, commit `6c1bd6c` + merge `93512a9`
- [ ] §2 验证: cargo check 0 err (0.48s cache 命中, P3-F 不增新 crate)
- [ ] §3 缺口: F.1 5 域 Lead 真人 / F.2 真实 e2e 需 5 域 Lead 真人 + dev server / F.5 质量门 5/5 实证 等缺口
- [ ] §4 子代理: 0 子代理调用
- [ ] §6 签字栏: 5 角色 + 架构师代签, 5 域 Lead 跨域 review 签字 (含 4 deliverable)

### §3.7 `PHASE-P3-CROSS-STAGE-INC-SESSION-003.md` (11.1KB, 18 commits + 15 deliverable 收编)

- [ ] §0 目的: 跨阶段收编报告, 承接 2026-08-29 跨 session 续做
- [ ] §1 改动: 18 commits + 15 deliverable (per `adb5f4f` 收编)
- [ ] §2 验证: 守门 #1+#9+#12+#8+#15 全过
- [ ] §3 缺口: 列跨 session 续做项
- [ ] §4 子代理: 0 子代理调用
- [ ] §6 签字栏: 5 角色 + 5 域 Lead 跨域 review 签字

### §3.8 `PHASE-P3-CROSS-STAGE-INC-SESSION-004.md` (12.7KB, 12 deliverable + 8 commits 收编)

- [ ] §0 目的: 跨阶段收编报告, 承接 2026-08-30 "全做" 5 套推进
- [ ] §1 改动: 12 deliverable + 8 commits (per `64b3885` 收编)
- [ ] §2 验证: 守门 #1+#9+#12+#8+#15 全过
- [ ] §3 缺口: 列跨 session 续做项
- [ ] §4 子代理: 0 子代理调用
- [ ] §6 签字栏: 5 角色 + 5 域 Lead 跨域 review 签字

---

## §4 E.6 Saga 详细补偿机制 (match 域 Lead 真人必补, 跨 session 续)

文件: `crates/star-saga/src/saga_step.rs` + `saga_5b_call.rs` + `compensation_strategy.rs` (3 新模块, 13.3KB Rust 源码, per `64b3885` 落地)

**match 域 Lead 真人到位后需补**:

1. [ ] **at-least-once vs exactly-once 投递**: match 域 Lead 拍板 (per `STAR-P3-5-DOMAIN-LEAD-REVIEW-CHECKLIST.md` §0.3 + `03-match-bc.md` §5 #2 缺口 + `04-social-bc.md` §5 #3 缺口)
2. [ ] **idempotency key 详细策略**: per `03-match-bc.md` §2.3 INV-SG-03 (同一 SagaInstance 不能跨 step 失败重复补偿)
3. [ ] **补偿链顺序**: per `crates/star-saga/src/compensation_strategy.rs` CompensationStrategy trait 骨架, match 域 Lead 补 DefaultCompensationStrategy 实际实现
4. [ ] **5 域跨域调用 stub 待 5 域 Lead 真人补详细业务逻辑** (per `PHASE-P3-E6-SAGA-IMPL-REPORT.md` §3 #3 缺口)
5. [ ] **crates/star-saga 单测**: 5 域跨域调用 + Compensation at-least-once 测试, match 域 Lead 写

**落地 1 commit**: `feat(star-saga): E.6 详细补偿机制 (at-least-once / exactly-once / idempotency key / 补偿链顺序) (per match 域 Lead)` + merge --no-ff main + 推 origin + 守门 #1+#9+#12+#8+#15 全过.

---

## §5 4 阻塞跨 session 续做 (5 域 Lead / 5 角色 真人到位后必推)

| # | 阻塞 | 等待方 | 续做 commit 数 | 文档 |
|---|---|---|---|---|
| 1 | **5 域 Lead 真人到位** | Ulysses 找 5 个真人, 每人 1 域, 追溯签字覆盖应急代签 (per `ec6dee0` 选项 4) | 1 (REGISTRY) + 5 (5 域 DDD docs) + 6 (6 份 P3 报告) + 2 (2 份 INC-SESSION) = **14 commits** | `STAR-P3-5-DOMAIN-LEAD-REGISTRY.md` v0.1 + `STAR-P3-5-DOMAIN-LEAD-PROC.md` v0.2 + `STAR-P3-5-DOMAIN-LEAD-REVIEW-CHECKLIST.md` v0.1 + `STAR-P3-5-DOMAIN-LEAD-REVIEW-PROTOCOL.md` v0.1 + `STAR-P3-DDD-REVIEW-PHASE.md` v0.1 + `STAR-P3-E7-SIGN-OFF-TEMPLATE.md` v0.1 + `STAR-P3-F5-5OF5-CHECKLIST.md` v0.1 + `STAR-P3-F5-5OF5-EMPIRICAL.md` v0.1 |
| 2 | **E.6 Saga 详细补偿机制** | match 域 Lead 真人补 (at-least-once / exactly-once / idempotency key / 补偿链顺序) | 1 commit + 1 域 Lead 签字 + 1 docs 同步 | `crates/star-saga/src/saga_step.rs` + `saga_5b_call.rs` + `compensation_strategy.rs` + `PHASE-P3-E6-SAGA-IMPL-REPORT.md` v0.2 |
| 3 | **B.5/B.6 + E.4 KMS 真凭证路径** | Ulysses 提供 OpenClaw/Hermes endpoint + API key + Vault/AWS KMS 凭证; economy/admin 域 Lead 切真替换 LocalMockKms | 2 commits (B.5 + B.6) + 1 commit (E.4) | `crates/star-saga/src/saga_step.rs` + `crates/domain-kms/src/lib.rs` LocalMockKms → Vault / AwsKms 切换 |
| 4 | **D.2/D.6 真实 GitHub Actions runner 配置** | SRE 配 (markdownlint + cargo doc CI 真实 runner, 替换 stub) | 2 commits (D.2 + D.6) | `.github/workflows/markdownlint.yml` + `cargo doc --no-deps` workflow |

**总 commits 落地**: 14 (5 域 Lead) + 1 (E.6) + 3 (B.5/B.6/E.4) + 2 (D.2/D.6) + 1 (F.5 5/5 实证总收口) = **21 commits**, 推 origin, P3 全 5 阶段 64/64 收官 + 5/5 质量门.

---

## §6 跨域 review 矩阵 (5 域 × 6 域 = 30 跨 review 项)

| 5 域 Lead \\ 6 域 (5 DDD docs + 1 跨域) | 01 player | 02 economy | 03 match | 04 social | 05 admin |
|---|---|---|---|---|---|
| player Lead | ✅ 主管 | ⚠️ 跨域 review | ⚠️ 跨域 review | ⚠️ 跨域 review | ⚠️ 跨域 review |
| economy Lead | ⚠️ 跨域 review | ✅ 主管 | ⚠️ 跨域 review | ⚠️ 跨域 review | ⚠️ 跨域 review |
| match Lead | ⚠️ 跨域 review | ⚠️ 跨域 review | ✅ 主管 | ⚠️ 跨域 review | ⚠️ 跨域 review |
| social Lead | ⚠️ 跨域 review | ⚠️ 跨域 review | ⚠️ 跨域 review | ✅ 主管 | ⚠️ 跨域 review |
| admin Lead | ⚠️ 跨域 review | ⚠️ 跨域 review | ⚠️ 跨域 review | ⚠️ 跨域 review | ✅ 主管 |

**矩阵说明**: 行 = 5 域 Lead, 列 = 5 域 docs + 1 跨域. 每行 1 域 Lead 主管自己的 1 域 (✅) + 跨域 review 4 域 (⚠️). 5 行 × 5 列 = 25 review 项, 加上 6 份 P3 报告每域 1 份 = 30 review 项.

---

## §7 时间预算 (5 域 Lead 协作, 11.3 小时)

| 步骤 | 时间预算 | 5 域 Lead 协作 |
|---|---|---|
| 步骤 1: REGISTRY 填 5 行 | 10 分钟 | Ulysses 独立 |
| 步骤 2: 5 域 Lead 各自 review 1 域 DDD docs | 60 分钟/域 = 5 小时 | 5 域 Lead 可并行 |
| 步骤 3: 5 域 Lead review 6 份 P3 报告 | 30 分钟/域 = 2.5 小时 | 5 域 Lead 可并行 |
| 步骤 4: 5 域 Lead 跨域 review | 30 分钟/域 = 2.5 小时 | 5 域 Lead 可并行 |
| 步骤 5: 5 域 Lead 签字栏 #1 追溯签字 | 10 分钟/域 = 50 分钟 | 5 域 Lead 可并行 |
| **总时间** | **11.3 小时** | **5 域 Lead 并行, 实际 ~2.3 小时/域** |

**总 commits**: 14 commits (per §1 步骤 5, 详细 plan per `STAR-P3-E7-SIGN-OFF-TEMPLATE.md` §3).

**DDD Review 阶段 (10 真人到位后)**: 7 步流程, P3 阶段从 4/5 升到 5/5, per `STAR-P3-DDD-REVIEW-PHASE.md` §2.

---

## §8 5 维度质量门 5/5 实证表 (DDD Review 阶段后, 38 commits 实证)

| 维度 | 4/5 (当前) | 5/5 (DDD Review 后) | 实证 commits |
|---|---|---|---|
| 1. 功能完整 | 56/64 (87.5%) | 64/64 (100%) | 17 + 8 = 25 |
| 2. 测试覆盖 | 44/44 crate 100% | 44/44 crate 100% + 5 docs | 17 |
| 3. 守门 0 违反 | 12 项 0 违反 | 12 项 0 违反 + 5 域 DDD review | 17 |
| 4. 文档同步 | 6 维度闭环 | 6 维度 + 13 docs 5 域 Lead 签字 | 17 + 14 |
| 5. git 证据 | 17 commits | 38 commits | 38 |
| **总计** | **4/5 (P3-A 25/25 已 5/5)** | **5/5 (P3 全 5 阶段 64/64 100%)** | **38 commits** |

**5/5 实证 = P3 阶段正式收官**. 落地 1 commit `docs(governance): P3 质量门 5/5 实证 (5 维度 + 5 角色 + 5 域 Lead 签字)`, 包含本文件 §8 + 5 维度 5/5 实证表.

---

## §9 签字栏 (5 角色 + 5 域 Lead 10 真人)

| # | 角色 | 姓名 | 签字日 | 5 维度实证 |
|---|---|---|---|---|
| 1 | 架构负责人 | `<待到岗>` | `<待签>` | 维度 1 + 3 + 5 |
| 2 | SRE Lead | `<待到岗>` | `<待签>` | 维度 2 + 3 |
| 3 | 平台工程师 | `<待到岗>` | `<待签>` | 维度 2 + 4 |
| 4 | 评审主持人 | `<待到岗>` | `<待签>` | 维度 3 + 5 |
| 5 | 项目负责人（PM）| `<待到岗>` | `<待签>` | 维度 1 + 4 |
| 6 | player 域 Lead | `<待到岗>` | `<待签>` | 维度 4 + 6 docs |
| 7 | economy 域 Lead | `<待到岗>` | `<待签>` | 维度 1 + 4 docs (B.5/B.6 切真) |
| 8 | match 域 Lead | `<待到岗>` | `<待签>` | 维度 1 + 4 docs (E.6 Saga 详细) + 维度 3+5 (评审跨域) |
| 9 | social 域 Lead | `<待到岗>` | `<待签>` | 维度 4 + 4 docs (5 域 template 12 订阅) |
| 10 | admin 域 Lead | `<待到岗>` | `<待签>` | 维度 1 + 4 docs (E.4 KMS 真凭证) + 维度 2+3 (SRE 跨域) |

---

## §10 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 5 域 Lead 真人 review 完整内容确认包 (5 步骤 + 5 域 DDD docs 30 项 review + 8 份报告 36 项 + E.6 Saga 详补 + 4 阻塞 + 跨域矩阵 + 11.3 小时 + 38 commits 5/5 实证) | 2026-08-30 11:13 JST Ulysses 指令"你替我把真人的内容全部确认好"触发 |
