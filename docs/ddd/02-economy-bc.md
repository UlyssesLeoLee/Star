# 02-economy-bc 域 DDD BoundedContext 边界 (Economy BoundedContext)

> **Status**: 🟡 占位 (P3-E.7 DDD 边界验证 docs 阶段, 5 域 Lead 真人到位后签字覆盖架构师代签)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **承接**: STAR-P3-5-DOMAIN-LEAD-PROC.md economy 域 + P3-C.2/C.4 计费集成 + P3-B.3 API Key 双模式 + cross-domain-5b-mermaid.md §1 economy 域

本文件是 **economy 域** 的 DDD BoundedContext 边界文档, 配合 `docs/architecture/cross-domain-5b-mermaid.md` 5 域 DDD 边界图.

---

## §1 BoundedContext 定义

**economy 域** = 计费 / 定价 / 成本 (per STAR-P3-5-DOMAIN-LEAD-PROC.md economy 域)

- **业务子域**: 计费账户 (Billing Account) + 订阅 (Subscription) + 发票 (Invoice) + API 凭证 (API Key) + 成本核算 (Cost Breakdown)
- **Aggregate Root**: `BillingAccount` + `Subscription` + `Invoice` + `ApiKey`
- **核心职责**: 5 域业务子域的计费 / 订阅 / 发票 / API 凭证 (encrypted + env_var) / 成本核算

---

## §2 Aggregate 详情

### §2.1 BillingAccount Aggregate

**聚合根**: `BillingAccount` (per `crates/domain-project` 计费集成 + per `crates/domain-cli` 凭证)

- **字段**:
  - `account_id: BillingAccountId`
  - `tenant_id: TenantId` (per INV-BL-01)
  - `plan: PricingTier` (Free / Pro / Enterprise, 值对象)
  - `payment_method: PaymentMethod` (信用卡 / 银行转账 / 平台账户, 值对象)
  - `balance: Money` (值对象, 含币种 + 金额)
  - `status: BillingStatus` (Active / PastDue / Suspended)
  - `created_at / updated_at: DateTime<Utc>`
- **不变量**:
  - **INV-BL-01** BillingAccount 必带 tenant_id (跨租户隔离)
  - **INV-BL-02** balance 不可为负 (debt 走 credit line 而非负 balance)
  - **INV-BL-03** 跨币种转换走 per-day 汇率表 (audit 必填)
- **命令**: `CreateBillingAccount` / `ChargeBillingAccount` / `RefundBillingAccount` / `SuspendBillingAccount`
- **事件**:
  - `BillingAccountCreated` (pub) → audit + social 域通知 owner
  - `InvoiceIssued` (pub) → social 域通知 owner + admin 域 audit
  - `PaymentFailed` (pub) → admin 域 (RBAC 暂停) + social 域通知

### §2.2 Subscription Aggregate

**聚合根**: `Subscription` (per `crates/domain-project`)

- **字段**:
  - `subscription_id: SubscriptionId`
  - `tenant_id: TenantId`
  - `plan: PricingTier`
  - `billing_cycle: BillingCycle` (Monthly / Yearly)
  - `started_at: DateTime<Utc>`
  - `expires_at: DateTime<Utc>`
  - `auto_renew: bool`
  - `status: SubscriptionStatus` (Active / Cancelled / Expired)
- **不变量**:
  - **INV-SUB-01** Subscription 必带 tenant_id
  - **INV-SUB-02** 同一 tenant 同一 plan 只能有 1 个 Active Subscription
  - **INV-SUB-03** expires_at 必填, 过期前 7 天发 RenewalReminder 事件
- **命令**: `CreateSubscription` / `CancelSubscription` / `RenewSubscription` / `UpgradePlan` / `DowngradePlan`
- **事件**:
  - `SubscriptionCreated` (pub) → player 域 Workspace 通知 + audit
  - `SubscriptionCancelled` (pub) → player 域 (Workspace 降级) + admin 域 (RBAC 调整)
  - `SubscriptionRenewed` (pub) → audit

### §2.3 Invoice Aggregate

**聚合根**: `Invoice`

- **字段**:
  - `invoice_id: InvoiceId`
  - `tenant_id: TenantId`
  - `subscription_id: SubscriptionId`
  - `line_items: Vec<LineItem>` (per-resource cost breakdown)
  - `subtotal / tax / total: Money`
  - `due_date: DateTime<Utc>`
  - `status: InvoiceStatus` (Draft / Issued / Paid / Overdue / Void)
- **不变量**:
  - **INV-INV-01** Invoice 一旦 Issued 不可改 line items (per SaaS 计费合规)
  - **INV-INV-02** total = subtotal + tax (校验)
  - **INV-INV-03** Paid 状态必填 payment_received_at
- **命令**: `DraftInvoice` / `IssueInvoice` / `MarkPaid` / `VoidInvoice`
- **事件**:
  - `InvoiceIssued` (pub) → 同 BillingAccount
  - `InvoicePaid` (pub) → audit
  - `InvoiceOverdue` (pub) → admin 域 (RBAC 暂停) + social 域通知

### §2.4 ApiKey Aggregate (per P3-B.3 双模式)

**聚合根**: `ApiKey` (per `crates/domain-cli` ApiKey 双模式存储)

- **字段**:
  - `key_id: ApiKeyId`
  - `tenant_id: TenantId`
  - `provider: Provider` (OpenClaw / Hermes / Custom)
  - `key_ref: ApiKeyRef` (双模式: encrypted hash 或 env_var 引用)
  - `mode: StorageMode` (Encrypted | EnvVar)
  - `created_at / rotated_at: DateTime<Utc>`
- **不变量**:
  - **INV-AK-01** ApiKey 明文绝不落盘 (per P3-B.3 双模式: encrypted hash 或 env_var 引用)
  - **INV-AK-02** 跨租户读取 = 拒绝 (per INV-KMS-03 同模式)
  - **INV-AK-03** rotated_at 必填, 90 天强制轮换 (per P3-B.3 90 天周期)
- **命令**: `CreateApiKey` / `RotateApiKey` / `RevokeApiKey`
- **事件**:
  - `ApiKeyCreated` (pub) → audit
  - `ApiKeyRotated` (pub) → audit (轮换必填 actor)
  - `ApiKeyRevoked` (pub) → admin 域 (凭证失效) + audit

---

## §3 跨域事件 (Economy 域作为发布者 / 订阅者)

### §3.1 Economy 域发布 (pub) 事件

| 事件 | 订阅域 | 订阅方职责 |
|---|---|---|
| `BillingAccountCreated` | social / audit | social 域 通知 owner; audit 必填 |
| `InvoiceIssued` | social / admin / audit | social 域 通知 owner; admin 域 audit 必填; audit 必填 |
| `PaymentFailed` | admin / social / audit | admin 域 RBAC 暂停; social 域 通知; audit 必填 |
| `SubscriptionCreated` | player / audit | player 域 Workspace 通知; audit 必填 |
| `SubscriptionCancelled` | player / admin | player 域 Workspace 降级; admin 域 RBAC 调整 |
| `ApiKeyRotated` | audit | audit 必填 (轮换 actor) |
| `ApiKeyRevoked` | admin / audit | admin 域 凭证失效; audit 必填 |

### §3.2 Economy 域订阅 (sub) 事件

| 事件 | 发布域 | Economy 域职责 |
|---|---|---|
| `WorkspaceProvisioned` (player 域) | player | 创建 BillingAccount (Free tier 默认) + Subscription |
| `UserSuspended` (player 域) | player | 暂停 BillingAccount (per INV-BL-02 走 credit line) |
| `TenantProvisioned` (admin 域) | admin | 创建 root BillingAccount + Subscription |
| `RoleAssigned` (admin 域) | admin | 调整 BillingAccount plan (Pro 升级) |

---

## §4 Cargo Crate 引用 (per main HEAD `ccf27fc`)

| 域 | Cargo Crate | 路径 | Lead 域 |
|---|---|---|---|
| economy | `domain-project` | `crates/domain-project/` (含 BillingAccount / Subscription 字段) | project Lead (含计费集成) |
| economy | `domain-cli` | `crates/domain-cli/` (含 ApiKey 双模式, per P3-B.3) | cli Lead (含凭证) |
| economy | `domain-audit` (只读) | `crates/domain-audit/` (调用 AuditRecorder Port) | audit Lead |
| economy | `domain-kms` (Phase 2 真凭证) | `crates/domain-kms/` (per P3-E.4 mock 备选 + LocalMockKms) | kms Lead (admin 域) |

**注**: economy 域**没有**专属 `domain-economy` crate, 计费 / 订阅 / 发票 / 凭证 4 Aggregate 散在 `domain-project` (per P3-C.2 拍板) + `domain-cli` (per P3-B.3 拍板). 5 域 Lead 真人到位后, 可考虑拆分独立 `domain-economy` crate (per P3-E.7 DDD 边界验证 phase 2).

---

## §5 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | economy 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束), 本 doc 由架构师代签 | 跨 session 续, economy 域 Lead 真人追溯签字 |
| 2 | BillingAccount / Subscription / Invoice / ApiKey 4 Aggregate 散在 domain-project + domain-cli, 是否拆分独立 domain-economy crate 待 5 域 Lead 真人拍板 | economy 域 Lead 真人到位后 |
| 3 | Invoice line items 详细 schema (per-resource cost breakdown) 待 5 域 Lead 真人补 | economy 域 Lead 真人到位后 |
| 4 | cross-currency 汇率表 (per INV-BL-03) 详细数据源待 5 域 Lead 真人拍板 (外部 API? 内部表?) | economy 域 Lead 真人到位后 |
| 5 | ApiKey 双模式 (encrypted vs env_var) 默认 mode 待 5 域 Lead 真人拍板 (per tenant 配置?) | economy 域 Lead 真人到位后 |
| 6 | Invoice 合规 (per SaaS 计费合规, 各国税法差异) 待 5 域 Lead 真人 + 财务咨询 | economy 域 Lead 真人到位后 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | economy 域 Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟡 应急代签; economy 域 BoundedContext + 4 Aggregate + 跨域事件 7 pub + 4 sub + Cargo crate 引用 (散在 domain-project + domain-cli) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: economy 域 BoundedContext + 4 Aggregate (BillingAccount / Subscription / Invoice / ApiKey) + 跨域事件 7 pub + 4 sub + Cargo crate 引用 + 已知缺口 6 项 | 2026-08-30 08:55 JST 5 域 DDD 边界 docs 落地触发 |
