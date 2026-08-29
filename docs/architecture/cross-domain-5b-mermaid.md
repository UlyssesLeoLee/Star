# 5 域 DDD 边界 + 跨域 Saga 流程 mermaid 架构图

> **Status**: 🟡 占位 (P3-F.4 拍板, 等 5 域 Lead 真人到位后补真实架构)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **承接**: STAR-P3-F-DECISION-PACK.md F.4 拍板 / STAR-P3-E-F-SELECTION-RESULT.md 选项 1

本文件是 5 域 DDD 边界 + 跨域 Saga 流程的 mermaid 架构图, 配套 CHANGELOG.md §2 流程.

---

## §1 5 域 DDD 边界图 (BoundedContext + Aggregate + 跨域事件)

```mermaid
graph TB
    subgraph player[player 域 - 用户/identity/workspace]
        User[User Aggregate]
        Workspace[Workspace Aggregate]
        Session[Session Entity]
        User -.->|publish| UserCreated[UserCreated Event]
        Workspace -.->|publish| WorkspaceProvisioned[WorkspaceProvisioned Event]
    end

    subgraph economy[economy 域 - billing/pricing/cost]
        BillingAccount[BillingAccount Aggregate]
        Subscription[Subscription Aggregate]
        Invoice[Invoice Entity]
        BillingAccount -.->|publish| InvoiceIssued[InvoiceIssued Event]
        Subscription -.->|publish| PaymentFailed[PaymentFailed Event]
    end

    subgraph match[match 域 - workflow/状态机/saga]
        Workflow[Workflow Aggregate]
        SagaInstance[SagaInstance Aggregate]
        StateTransition[StateTransition Entity]
        Workflow -.->|publish| WorkflowStarted[WorkflowStarted Event]
        SagaInstance -.->|publish| SagaCompleted[SagaCompleted Event]
    end

    subgraph social[social 域 - collaboration/通知]
        Notification[Notification Aggregate]
        Comment[Comment Aggregate]
        Mention[Mention Entity]
        Notification -.->|publish| NotificationDispatched[NotificationDispatched Event]
        Comment -.->|publish| CommentPosted[CommentPosted Event]
    end

    subgraph admin[admin 域 - RBAC/permission/tenant]
        Tenant[Tenant Aggregate]
        Permission[Permission Aggregate]
        Role[Role Entity]
        Tenant -.->|publish| TenantProvisioned[TenantProvisioned Event]
        Role -.->|publish| RoleAssigned[RoleAssigned Event]
    end

    UserCreated -.->|subscribe| Notification
    WorkspaceProvisioned -.->|subscribe| Role
    InvoiceIssued -.->|subscribe| Notification
    PaymentFailed -.->|subscribe| Role
    WorkflowStarted -.->|subscribe| BillingAccount
    WorkflowStarted -.->|subscribe| Role
    WorkflowStarted -.->|subscribe| Notification
    SagaCompleted -.->|subscribe| Permission
    NotificationDispatched -.->|subscribe| User
    CommentPosted -.->|subscribe| Workflow
    TenantProvisioned -.->|subscribe| Workspace
    RoleAssigned -.->|subscribe| Session

    classDef player fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    classDef economy fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef match fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef social fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    classDef admin fill:#fce4ec,stroke:#880e4f,stroke-width:2px

    class User,Workspace,Session player
    class BillingAccount,Subscription,Invoice economy
    class Workflow,SagaInstance,StateTransition match
    class Notification,Comment,Mention social
    class Tenant,Permission,Role admin
```

---

## §2 跨域 Saga 流程 (创建项目 + 跨域协同)

```mermaid
sequenceDiagram
    autonumber
    participant U as User (player 域)
    participant P as player.Workspace
    participant M as match.Workflow
    participant E as economy.BillingAccount
    participant A as admin.Role
    participant S as social.Notification
    participant Aud as admin.Audit (per B.9)

    U->>P: 创建项目 workspace
    P->>M: WorkflowStarted (触发 match 域)
    M->>E: ProvisionBillingAccount (跨域调用)
    E-->>M: BillingAccountCreated
    M->>A: RoleAssigned (project owner)
    A-->>M: RoleAssignedOK
    M->>S: NotificationDispatched (项目就绪)
    S-->>U: NotificationDelivered (in-app + email)
    M->>Aud: Audit (WorkflowCompleted, 9 AI Audit 必填)

    Note over U,Aud: 失败回滚 (Saga Compensation per star-saga + Q-003)

    alt Economy ProvisionBillingAccount 失败
        E-->>M: BillingProvisionFailed
        M->>P: WorkflowFailed (回滚状态)
        M->>A: 取消 RoleAssigned
        M->>Aud: Audit (WorkflowFailed + 补偿原因)
        Aud-->>U: 通知失败 (per social 域)
    end
```

---

## §3 5 域 Lead 真人 + Aggregate Owner 责任矩阵

| 域 | BoundedContext | Aggregate | Lead (待真人到位) | 责任 | RACI |
|---|---|---|---|---|---|
| **player** | Identity / Workspace | User / Workspace / Session | 架构师代签 (per ec6dee0) | user CRUD + workspace 生命周期 | R/A: player Lead / C: 全 5 域 / I: admin |
| **economy** | Billing / Subscription | BillingAccount / Subscription | 架构师代签 | 计费 / 订阅 / 发票 / 退款 | R: economy Lead / A: 财务 / C: admin / I: player |
| **match** | Workflow / Saga | Workflow / SagaInstance | 架构师代签 | 状态机 / Saga 跨域编排 / 补偿 | R: match Lead / A: PM / C: 全 5 域 / I: admin |
| **social** | Notification / Comment | Notification / Comment | 架构师代签 | 跨域通知 / 协作 / mention | R: social Lead / A: PM / C: 全 5 域 / I: admin |
| **admin** | Tenant / RBAC | Tenant / Permission / Role | 架构师代签 | 多租户 / RBAC / permission / audit | R: admin Lead / A: 安全 / C: 全 5 域 / I: 真人 (per 8/21 拒绝兼任) |

---

## §4 5 域 Lead 真人到位流程 (per STAR-P3-5-DOMAIN-LEAD-PROC.md)

1. **Ulysses 找 5 个真人**, 每人认领 1 域 (player / economy / match / social / admin)
2. **每域 Lead 独立签字** (per 8/21 JST 拒绝兼任硬约束), 覆盖架构师代签 (per ec6dee0 选项 4 应急)
3. **BoundedContext 边界验证** (per P3-E.7), 真人签字 + Aggregate Owner 责任矩阵签字
4. **跨域 Saga 流程图真人补完** (per §2), match 域 Lead 真人补补偿机制
5. **DDD Review 阶段 Lead 真人全程参与** (per 守门 #5 5 域独立 Lead)

---

## §5 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 5 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束), 当前 §3 RACI 全部架构师代签 | 跨 session 续 |
| 2 | mermaid 渲染需 GitHub / obsidian / VSCode mermaid 插件支持, CI runner 渲染需 GitHub Actions 配置 | P3-D.6 runner 配置 stub (per PHASE-P3-D1-D7-IMPL-REPORT.md) |
| 3 | 跨域 Saga 详细补偿机制待 match 域 Lead 真人补 (per §2 alt 路径) | P3-E.6 Saga 实装 |
| 4 | 5 域 BoundedContext 边界图 (per §1) Aggregate 内 Entity 完整字段待 5 域 Lead 真人补 | P3-E.7 DDD 边界验证 |

---

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 5 域 DDD 边界图 (mermaid graph) + 跨域 Saga 流程图 (mermaid sequence) + 5 域 Lead 责任矩阵 + 真人到位流程 + 已知缺口 4 项 | 2026-08-30 08:46 JST P3-F.4 拍板 + 跨 session 续做触发 |
