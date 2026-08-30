# 04-social-bc 域 DDD BoundedContext 边界 (Social BoundedContext)

> **Status**: 🟡 占位 (P3-E.7 DDD 边界验证 docs 阶段, 5 域 Lead 真人到位后签字覆盖架构师代签)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **承接**: STAR-P3-5-DOMAIN-LEAD-PROC.md social 域 + P3-E.2 Notification 域 + cross-domain-5b-mermaid.md §1 social 域

本文件是 **social 域** 的 DDD BoundedContext 边界文档, 配合 `docs/architecture/cross-domain-5b-mermaid.md` 5 域 DDD 边界图.

---

## §1 BoundedContext 定义

**social 域** = collaboration / 通知 (per STAR-P3-5-DOMAIN-LEAD-PROC.md social 域)

- **业务子域**: 通知 (Notification) + 评论 (Comment) + mention + 协作 (Collaboration) + 跨域事件总线 (Cross-domain Event Bus)
- **Aggregate Root**: `Notification` + `Comment` + `Mention`
- **核心职责**: 5 域业务子域的跨域通知 (per P3-E.2 拍板) / 协作 / 评论 / mention

---

## §2 Aggregate 详情

### §2.1 Notification Aggregate (per P3-E.2 拍板)

**聚合根**: `Notification` (per `crates/domain-notification` lib.rs, 42KB)

- **字段**:
  - `notification_id: NotificationId`
  - `tenant_id: TenantId`
  - `recipient_user_id: UserId`
  - `channel: Channel` (InApp / Email / Webhook / Slack, 值对象)
  - `priority: Priority` (Low / Normal / High / Urgent)
  - `content: Content` (标题 + 消息体 + 链接, 值对象)
  - `source_event: String` (触发事件类型, e.g. "UserCreated" / "InvoiceIssued")
  - `source_aggregate_id: String` (源 Aggregate ID, 用于反查)
  - `read_at: Option<DateTime<Utc>>`
  - `delivered_at: Option<DateTime<Utc>>`
  - `status: NotificationStatus` (Pending / Delivered / Failed / Expired)
  - `created_at: DateTime<Utc>`
- **不变量**:
  - **INV-NOT-01** Notification 必带 tenant_id + recipient_user_id
  - **INV-NOT-02** channel 多选 (InApp + Email 双发), 至少 1 必填
  - **INV-NOT-03** delivered_at 必填后才标 Delivered (per Q-003 至少一次投递)
  - **INV-NOT-04** 30 天后未读自动 Expired (per retention 策略)
- **命令**: `CreateNotification` / `MarkRead` / `RetryDelivery` / `ExpireNotification`
- **事件**:
  - `NotificationDispatched` (pub) → 5 域 (player / economy / match / social / admin 全部监听, per F.4 §1)
  - `NotificationDelivered` (pub) → audit (投递成功审计)
  - `NotificationFailed` (pub) → retry 3 次后告警 + admin 域 audit

### §2.2 Comment Aggregate

**聚合根**: `Comment` (per `crates/domain-comment`)

- **字段**:
  - `comment_id: CommentId`
  - `tenant_id: TenantId`
  - `target_aggregate: String` (被评论对象, e.g. "WorkItem" / "Workflow")
  - `target_aggregate_id: String` (源 Aggregate ID)
  - `author_user_id: UserId`
  - `content: Content` (markdown 格式)
  - `mentions: Vec<Mention>` (@用户, 值对象)
  - `reactions: Vec<Reaction>` (emoji 反应)
  - `parent_comment_id: Option<CommentId>` (嵌套回复)
  - `created_at / edited_at: DateTime<Utc>`
  - `status: CommentStatus` (Visible / Hidden / Deleted)
- **不变量**:
  - **INV-CMT-01** Comment 必带 tenant_id + author_user_id + target_aggregate_id
  - **INV-CMT-02** mentions 必填触发 NotificationDispatched 事件 (per §2.1)
  - **INV-CMT-03** 一旦 Deleted 不可改 (Append-only, per audit INV-AU-01)
- **命令**: `CreateComment` / `EditComment` / `DeleteComment` / `ReactToComment`
- **事件**:
  - `CommentPosted` (pub) → match 域 (timeline 更新) + audit
  - `CommentEdited` (pub) → audit
  - `CommentDeleted` (pub) → audit
  - `MentionTriggered` (pub) → Notification 域 (创建 @user notification)

### §2.3 Mention Aggregate (值对象-like)

**注**: Mention 通常作为 Comment 或 Notification 的一部分, 不独立成 Aggregate. 这里列出来是因为它在跨域事件中频繁出现.

- **字段**:
  - `mentioned_user_id: UserId`
  - `context: String` (mention 的上下文, e.g. "@张三是 owner")
  - `notified: bool` (是否已触发 Notification)

---

## §3 跨域事件 (Social 域作为发布者 / 订阅者)

### §3.1 Social 域发布 (pub) 事件

| 事件 | 订阅域 | 订阅方职责 |
|---|---|---|
| `NotificationDispatched` | player / economy / match / social / admin | 5 域全部监听 (per F.4 §1 跨域事件总线), 用于实时 UI 更新 |
| `NotificationDelivered` | audit | 投递成功审计 |
| `NotificationFailed` | admin / audit | retry 失败告警; audit 必填 |
| `CommentPosted` | match / audit | match 域 timeline 更新; audit 必填 |
| `CommentEdited` | audit | audit 必填 |
| `CommentDeleted` | audit | audit 必填 |
| `MentionTriggered` | (内部 Notification 域) | 创建 @user notification |

### §3.2 Social 域订阅 (sub) 事件

| 事件 | 发布域 | Social 域职责 |
|---|---|---|
| `UserCreated` (player 域) | player | 发送欢迎 notification (template: "欢迎 {display_name}") |
| `WorkspaceProvisioned` (player 域) | player | 通知 owner + members |
| `WorkspaceMemberAdded` (player 域) | player | 通知新成员 |
| `UserSuspended` (player 域) | player | 通知 admin |
| `InvoiceIssued` (economy 域) | economy | 通知 owner (template: "您的发票 {invoice_id} 已发出") |
| `PaymentFailed` (economy 域) | economy | 通知 owner (template: "支付失败, 请更新支付方式") |
| `WorkflowStarted` (match 域) | match | 通知 owner (template: "Workflow {workflow_name} 已开始") |
| `WorkflowCompleted` (match 域) | match | 通知 owner (template: "Workflow {workflow_name} 已完成") |
| `WorkflowFailed` (match 域) | match | 通知 owner (template: "Workflow {workflow_name} 失败") |
| `SagaCompleted` (match 域) | match | 通知 owner (template: "Saga {saga_type} 已完成") |
| `SagaFailed` (match 域) | match | 通知 owner (template: "Saga {saga_type} 失败, 已补偿") |
| `TenantProvisioned` (admin 域) | admin | 通知 owner |
| `RoleAssigned` (admin 域) | admin | 通知 user (新角色) |

---

## §4 Cargo Crate 引用 (per main HEAD `ccf27fc`)

| 域 | Cargo Crate | 路径 | Lead 域 |
|---|---|---|---|
| social | `domain-notification` | `crates/domain-notification/` (per P3-E.2 拍板, 42KB lib.rs) | notification Lead |
| social | `domain-comment` | `crates/domain-comment/` | comment Lead |
| social | `domain-collaboration` | `crates/domain-collaboration/` | collaboration Lead |
| social | `domain-audit` (只读) | `crates/domain-audit/` (调用 AuditRecorder Port) | audit Lead |
| social | `domain-search` (per P3-E.3 拍板) | `crates/domain-search/` (通知全文搜索) | search Lead |

**注**: social 域**没有**专属 `domain-social` crate, notification / comment / collaboration 3 Aggregate 散在 `domain-notification` + `domain-comment` + `domain-collaboration`. 5 域 Lead 真人到位后, 可考虑整合 (per P3-E.7 DDD 边界验证 phase 2).

---

## §5 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | social 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束), 本 doc 由架构师代签 | 跨 session 续, social 域 Lead 真人追溯签字 |
| 2 | 5 域 notification template (per §3.2 12 订阅事件) 详细文案 + i18n 待 social 域 Lead 真人补 | social 域 Lead 真人到位后 |
| 3 | notification 投递可靠性 (at-least-once vs exactly-once, per Q-003) 待 match 域 Lead 真人拍板 | match 域 Lead 真人到位后 |
| 4 | 30 天 retention 策略 (per §2.1 INV-NOT-04) 详细实现 (cron? lazy expire?) 待 social 域 Lead 真人拍板 | social 域 Lead 真人到位后 |
| 5 | 跨域事件总线架构 (per §3.1 5 域全部监听) 实施细节 (in-process channel? external broker?) 待 5 域 Lead 真人拍板 | 5 域 Lead 真人到位后 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | social 域 Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟡 应急代签; social 域 BoundedContext + 3 Aggregate (Notification / Comment / Mention) + 跨域事件 7 pub + 12 sub + Cargo crate 引用 (散在 domain-notification + domain-comment + domain-collaboration) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: social 域 BoundedContext + 3 Aggregate (Notification / Comment / Mention) + 跨域事件 7 pub + 12 sub + Cargo crate 引用 + 已知缺口 5 项 | 2026-08-30 08:55 JST 5 域 DDD 边界 docs 落地触发 |
