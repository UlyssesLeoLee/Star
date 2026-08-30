# 03-match-bc 域 DDD BoundedContext 边界 (Match BoundedContext)

> **Status**: 🟡 占位 (P3-E.7 DDD 边界验证 docs 阶段, 5 域 Lead 真人到位后签字覆盖架构师代签)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **承接**: STAR-P3-5-DOMAIN-LEAD-PROC.md match 域 + P3-C.4/C.5/C.6 + P3-B.8 fallback + cross-domain-5b-mermaid.md §1 match 域

本文件是 **match 域** 的 DDD BoundedContext 边界文档, 配合 `docs/architecture/cross-domain-5b-mermaid.md` 5 域 DDD 边界图.

---

## §1 BoundedContext 定义

**match 域** = workflow / 状态机 / saga (per STAR-P3-5-DOMAIN-LEAD-PROC.md match 域)

- **业务子域**: 工作流定义 (Workflow Definition) + 工作流执行 (Workflow Execution) + Saga 编排 (Saga Orchestration) + 状态机 (State Machine) + 跨域补偿 (Cross-domain Compensation)
- **Aggregate Root**: `Workflow` + `WorkflowInstance` + `SagaInstance`
- **核心职责**: 5 域业务子域的状态机 / Saga 跨域编排 / 跨域补偿 / 失败回滚 (per Q-003 跨域协调)

---

## §2 Aggregate 详情

### §2.1 Workflow Aggregate (Definition)

**聚合根**: `Workflow` (per `crates/domain-workflow` lib.rs)

- **字段**:
  - `workflow_id: WorkflowId`
  - `tenant_id: TenantId`
  - `name: String`
  - `states: Vec<State>` (定义状态机节点)
  - `transitions: Vec<Transition>` (定义状态机边)
  - `triggers: Vec<Trigger>` (外部触发器: UserCreated / WorkspaceProvisioned / ...)
  - `version: u32` (per 工作流版本控制)
  - `status: WorkflowStatus` (Draft / Published / Archived)
- **不变量**:
  - **INV-WF-01** Workflow 必带 tenant_id
  - **INV-WF-02** states + transitions 构成有向无环图 (DAG, 避免循环死锁)
  - **INV-WF-03** 一旦 Published, states 不可改, 只能新版本
- **命令**: `CreateWorkflow` / `PublishWorkflow` / `ArchiveWorkflow` / `NewVersion`
- **事件**:
  - `WorkflowPublished` (pub) → social 域 通知 admin + audit
  - `WorkflowArchived` (pub) → audit

### §2.2 WorkflowInstance Aggregate (Execution)

**聚合根**: `WorkflowInstance` (per `crates/domain-work-item` + `crates/domain-workflow`)

- **字段**:
  - `instance_id: WorkflowInstanceId`
  - `workflow_id: WorkflowId` (引用 §2.1 聚合根)
  - `tenant_id: TenantId`
  - `current_state: State` (per 状态机当前节点)
  - `state_history: Vec<StateTransition>` (审计追溯)
  - `context: WorkflowContext` (跨 step 共享数据)
  - `status: InstanceStatus` (Running / Completed / Failed / Cancelled)
  - `started_at / completed_at: DateTime<Utc>`
- **不变量**:
  - **INV-WFI-01** WorkflowInstance 必带 workflow_id + tenant_id
  - **INV-WFI-02** state_history 不可改 (Append-only, per audit INV-AU-01)
  - **INV-WFI-03** Failed 状态必填 failure_reason
- **命令**: `StartInstance` / `TransitionState` / `CancelInstance` / `FailInstance`
- **事件**:
  - `WorkflowStarted` (pub) → player / economy / social / admin 5 域协同 (per F.4 §2 跨域 Saga 流程)
  - `StateTransitioned` (pub) → social 域 (实时更新) + audit
  - `WorkflowCompleted` (pub) → player 域 (last-activity) + audit
  - `WorkflowFailed` (pub) → Saga 补偿 + audit (per B.9 监控审计 必填)

### §2.3 SagaInstance Aggregate (per star-saga + Q-003)

**聚合根**: `SagaInstance` (per `crates/star-saga` lib.rs)

- **字段**:
  - `saga_id: SagaId`
  - `tenant_id: TenantId`
  - `saga_type: SagaType` (CreateProject / ProvisionWorkspace / UpgradePlan ...)
  - `steps: Vec<SagaStep>` (per step 调用方 + 补偿)
  - `current_step: usize`
  - `status: SagaStatus` (Running / Compensating / Completed / Failed)
  - `started_at / completed_at: DateTime<Utc>`
- **不变量**:
  - **INV-SG-01** SagaInstance 必带 tenant_id
  - **INV-SG-02** steps 必填 compensation (失败回滚调用方)
  - **INV-SG-03** 同一 SagaInstance 不能跨 step 失败重复补偿 (per Q-003 idempotency)
- **命令**: `StartSaga` / `AdvanceStep` / `CompensateSaga` / `CompleteSaga`
- **事件**:
  - `SagaStarted` (pub) → audit
  - `SagaStepCompleted` (pub) → social 域 (实时更新) + audit
  - `SagaCompleted` (pub) → admin 域 (SagaCompleted 事件订阅) + audit
  - `SagaFailed` (pub) → 触发 compensation 链 + audit
  - `SagaCompensated` (pub) → 通知 owner + audit (compensation 必填 reason)

---

## §3 跨域事件 (Match 域作为发布者 / 订阅者)

### §3.1 Match 域发布 (pub) 事件

| 事件 | 订阅域 | 订阅方职责 |
|---|---|---|
| `WorkflowPublished` | social / admin / audit | social 域 通知 admin; admin 域 audit; audit 必填 |
| `WorkflowStarted` | player / economy / social / admin | 5 域协同 (per F.4 §2 跨域 Saga 流程); player 域 last-activity; economy 域 BillingAccount; social 域通知; admin 域 audit |
| `StateTransitioned` | social / audit | social 域实时更新; audit 必填 |
| `WorkflowCompleted` | player / admin / audit | player 域 last-activity; admin 域 audit; audit 必填 |
| `WorkflowFailed` | player / economy / admin / audit | 触发 Saga 补偿; player 域状态回滚; economy 域 BillingAccount 取消; admin 域 audit |
| `SagaCompleted` | admin / audit | admin 域 audit 必填 |
| `SagaFailed` | player / economy / admin / audit | 触发补偿链; admin 域 audit; 通知 owner |

### §3.2 Match 域订阅 (sub) 事件

| 事件 | 发布域 | Match 域职责 |
|---|---|---|
| `UserCreated` (player 域) | player | 触发 OnboardingWorkflow |
| `WorkspaceProvisioned` (player 域) | player | 触发 ProvisionWorkspaceSaga (per F.4 §2 跨域 Saga 流程) |
| `PaymentFailed` (economy 域) | economy | 触发 SuspendAccountWorkflow |
| `NotificationDispatched` (social 域) | social | 触发 NotificationDeliveryWorkflow |
| `TenantProvisioned` (admin 域) | admin | 触发 TenantBootstrapWorkflow |

---

## §4 Cargo Crate 引用 (per main HEAD `ccf27fc`)

| 域 | Cargo Crate | 路径 | Lead 域 |
|---|---|---|---|
| match | `domain-workflow` | `crates/domain-workflow/` (per P3-C.5 收官, commit `81de99a`) | workflow Lead |
| match | `domain-work-item` | `crates/domain-work-item/` (per P3-C.4 收官) | work-item Lead |
| match | `star-saga` | `crates/star-saga/` (per P3-C.6 收官, commit `25d086e`) | saga Lead |
| match | `domain-audit` (只读) | `crates/domain-audit/` (调用 AuditRecorder Port) | audit Lead |
| match | `domain-cli` (API→CLI fallback, per P3-B.8) | `crates/domain-cli/` (含 fallback 链路) | cli Lead (含 API fallback) |

**注**: match 域**没有**专属 `domain-match` crate, workflow / work-item / saga 3 Aggregate 散在 `domain-workflow` + `domain-work-item` + `star-saga`. 5 域 Lead 真人到位后, 可考虑整合 (per P3-E.7 DDD 边界验证 phase 2).

---

## §5 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | match 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束), 本 doc 由架构师代签 | 跨 session 续, match 域 Lead 真人追溯签字 |
| 2 | E.6 Saga 跨域编排详细补偿机制 (per 跨域 Saga 流程 F.4 §2 alt 路径) 待 match 域 Lead 真人补 | match 域 Lead 真人到位后 |
| 3 | WorkflowInstance 状态机版本控制 (per §2.2 INV-WF-03) 详细 schema 待 match 域 Lead 真人补 | match 域 Lead 真人到位后 |
| 4 | Saga idempotency 详细策略 (per §2.3 INV-SG-03) 待 match 域 Lead 真人补 | match 域 Lead 真人到位后 |
| 5 | 状态机 DAG 校验 (per §2.1 INV-WF-02) 实现细节待 match 域 Lead 真人拍板 | match 域 Lead 真人到位后 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | match 域 Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟡 应急代签; match 域 BoundedContext + 3 Aggregate (Workflow / WorkflowInstance / SagaInstance) + 跨域事件 7 pub + 5 sub + Cargo crate 引用 (散在 domain-workflow + domain-work-item + star-saga) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: match 域 BoundedContext + 3 Aggregate (Workflow / WorkflowInstance / SagaInstance) + 跨域事件 7 pub + 5 sub + Cargo crate 引用 + 已知缺口 5 项 | 2026-08-30 08:55 JST 5 域 DDD 边界 docs 落地触发 |
