# Spec-01: 跨域 Saga 协调

> **状态**：Draft v0.1
> **日期**：2026-08-27
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per ADR-0035 §8 Phase G / 2026-08-27 21:59 JST 用户授权

## §1 目的

定义 Star 跨域 Saga 协调契约（22 domain bounded context per [basic-design §6](../../../basic-design.md) + SagaCoordinationRole 5 类抽象功能角色 per §2），实现 Worktree Orchestration 跨 12 domain 8 步编排（per §4）。RGS 仓 Q-003 5 域（player/economy/match/social/admin）业务子域场景作为兼容性 footnote（per §4 Q-003 兼容性 footnote），不通过本 spec 推断 22 domain 归属。本 spec 是 Phase G 落地 `crates/star-saga` 的依据。

**为何需要 Saga 抽象层**（per [ADR-0027 §2 STAR IDE Gateway](../../adr/0027-star-ide-gateway.md) 5 通道 + Fallback Ladder 4 级 + [ADR-0035 §8.2 Phase G 方向](../../adr/0035-phase-f-architecture.md) "跨域 Saga 协调：22 domain 跨域事务" line 265 + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../adr/0039-worktree-orchestration-cross-domain.md)）：

1. **跨域事务一致性**：单次用户操作可能跨 22 domain 中多个（如 Worktree Orchestration 8 步 per §4），任一 step 失败需逆向补偿，否则系统状态不一致
2. **5 域独立 Lead 决策边界**（per 8/21 JST 用户偏好"不接受兼任"）：Q-003 经济域决策点必须 Economy Lead 独立拍板，不能由 player 域 Lead 兼任决策
3. **可观测性**：Saga 状态机 + 补偿日志为 ops 提供"半成品事务"恢复入口
4. **测试可替换**：Saga trait 让 conformance test 用 mock provider 跑通（per [spec/services/01 §0](../services/01-service-adapter-spec.md) 反污染原则 + ADR-0025）
5. **不死锁 + 不丢失**：持久化 + 补偿保证 saga 实例即使跨进程崩溃也能恢复

**Saga 与现有契约关系**：
- 与 [spec/agents/01 §2 Lease 协议](../agents/01-agent-runtime-spec.md) 关系：Saga 状态持久化复用 Lease 30s heartbeat 周期
- 与 [spec/agents/02 §2 22 domain 数据源清单](../agents/02-data-sources-spec.md) 关系：SagaStep 通过 `agent://{crate}/{id}` Resource URI 调用 22 domain crate
- 与 [spec/vcs/05 §2 4 Git Provider 接入规范](../vcs/05-real-providers-spec.md) 关系：MR/PR 创建作为典型 saga 触发场景（Worktree Orchestration 7 步 `LinkPullRequest` step,per §4）
- 与 [spec/cache/01 §3 缓存契约](../cache/01-cache-contract-spec.md)（Phase G）关系：Saga 状态机持久化复用 star-cache
- 与 [spec/services/07 审计模型](../flows/07-audit-model.md)（Phase G）关系：`AuditLogging` coordination_role 的 step (per §2) 落 audit log (responsible_crate = `domain-audit`)

## §2 Saga 抽象

```rust
// crates/star-saga/src/lib.rs（计划位置，本 spec 不实装 Rust 代码）
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Saga 协调能力分类：5 类抽象功能角色
///
/// **dual-use 警告（per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板）**：
/// 5 域（player/economy/match/social/admin）是 RGS 仓**历史治理命名**（5 位真人 Lead
/// 问责结构），Star 仓**不建立业务子域↔DDD bounded context 映射**。本 enum 的
/// 5 值命名为 saga 跨 step 时的 **5 类抽象功能角色**，与 5 域业务子域**完全脱钩**。
/// Step 实际归属到哪个 22 domain crate 由 `responsible_crate: &str` 字段指明，
/// 不通过本 enum 推断。Saga trait 不再以"5 域"为单位承载 Lead 责任分工。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaCoordinationRole {
    /// 身份验证类 step（agent identity / 租户/device 三重绑定校验）
    IdentityValidation,
    /// 资源变更类 step（balance / order / state mutation 等事务性写）
    ResourceMutation,
    /// 状态观察类 step（worktree 状态、agent session 进度、observed state 上报）
    StateObservation,
    /// 决策授权类 step（需 Lead 显式拍板的决策 gate，类似 Q-003 EconomicDecision 关键路径人工 gate）
    DecisionAuthorization,
    /// 审计日志类 step（COC / audit log / compliance record，不可由 SRE 兼任）
    AuditLogging,
}

/// Saga 步骤 trait：归属 22 domain crate 之一，按协调能力分类标注
#[async_trait]
pub trait SagaStep: Send + Sync {
    /// step 名称（如 "ValidateAgentSession" / "CreateWorktree"）
    fn name(&self) -> &str;

    /// 协调能力分类（per SagaCoordinationRole，用于编排器调度 / 可视化 / SLA 分级）
    fn coordination_role(&self) -> SagaCoordinationRole;

    /// 负责的 22 domain crate 路径（如 "domain-worktree" / "domain-agent" / "domain-feedback"）
    /// per basic-design §6 22 bounded context 列表
    fn responsible_crate(&self) -> &str;

    /// 前向执行（commit 路径）
    async fn execute(&self, ctx: &mut SagaContext) -> Result<StepResult, SagaError>;

    /// 逆向补偿（rollback 路径）—— execute 失败后被调用
    async fn compensate(&self, ctx: &mut SagaContext) -> Result<(), SagaError>;
}

/// Saga 定义：有序 step 列表 + 超时
pub struct Saga {
    pub name: String,
    pub steps: Vec<Box<dyn SagaStep>>,
    pub timeout_sec: u32,
}

/// Saga 编排器 trait：执行 / 补偿 / 状态查询
#[async_trait]
pub trait SagaOrchestrator: Send + Sync {
    /// 执行 saga：按 steps 顺序 execute，任一失败 → 自动 compensate 已完成 step
    async fn execute(&self, saga: &Saga) -> Result<SagaResult, SagaError>;

    /// 手动补偿（用于 ops 介入或 saga 持久化恢复）
    async fn compensate(&self, saga: &Saga) -> Result<(), SagaError>;

    /// 状态查询（用于死信排查 / 半成品恢复）
    async fn state(&self, saga_id: &str) -> Result<SagaState, SagaError>;
}

/// Saga 上下文：跨 step 共享的状态（如 transaction_id / 补偿日志）
pub struct SagaContext {
    pub saga_id: String,
    pub tx_id: String,
    /// 跨 step 状态：key 为 responsible_crate 字符串（如 "domain-worktree"），
    /// 不再以 SagaCoordinationRole 枚举为 key（per dual-use 警告，5 域脱钩后
    /// coordination_role 不再承载 step 间共享状态归属语义）
    pub crate_state: std::collections::HashMap<String, serde_json::Value>,
    pub coordination_log: Vec<CoordinationEntry>,
}

/// Saga 状态机（per §5 状态机定义）
pub enum SagaState {
    Pending,
    Running { completed_steps: Vec<String> },
    Completed,
    Compensating { pending_compensations: Vec<String> },
    Compensated,
    Failed { error: String, failed_step: String },
}

/// Saga 执行结果
pub struct SagaResult {
    pub saga_id: String,
    pub final_state: SagaState,
    pub step_results: Vec<StepResult>,
}
```

**关键设计决策**（per v0.1 风格 "凭据注入由 trait 内部完成，调用方不接触" 同样原则 + ADR-0025 反污染）：

- `coordination_role()` 字段**强制**每 step 标注协调能力分类（5 类抽象功能角色，用于编排器调度 / 可视化 / SLA 分级）
- `responsible_crate()` 字段**强制**每 step 标注负责的 22 domain crate（绑定实际业务实现位置）
- `compensate()` 失败不阻断其他 step 补偿（best-effort compensation，per §6 已知缺口 G-05 死信处理）
- `SagaContext.crate_state` 允许 step 跨 step 传值（key 为 responsible_crate 字符串，如 `domain-worktree` step 的结果供 `domain-agent` step 消费）
- 编排器**不**直接调 step trait，step 通过 Application Service 注册（per [arch/01 §4 职责分层](../../arch/01-current-architecture-analysis.md)）

## §3 Saga 协调能力分类映射

per §2 `SagaCoordinationRole` 5 类抽象功能角色 + 22 domain crate 实际归属（per basic-design §6 + [spec/agents/02 §2](../agents/02-data-sources-spec.md) 22 domain crate 清单）：

| 协调能力 | 职责 | 实际归属 22 domain crate（示例） | 必填字段 |
|---|---|---|---|
| `IdentityValidation` | 身份 / 租户 / 设备 / session 三重绑定校验 | `domain-identity` / `domain-tenant` / `domain-agent`（含 `domain-local-runtime` Device 校验） | `responsible_crate` 必填其中之一 |
| `ResourceMutation` | balance / order / state mutation 等事务性写 | `domain-work-item` / `domain-worktree` / `domain-feedback` / `domain-validation` | 同上 |
| `StateObservation` | worktree 状态、agent session 进度、observed state 上报 | `domain-worktree` / `domain-agent` / `domain-local-runtime` / `domain-context` | 同上 |
| `DecisionAuthorization` | 需 Lead 显式拍板的决策 gate（关键路径人工 gate 原则 per [arch/06 §3 NFR-OP-015](../../arch/06-threat-model-nfr.md)） | `domain-permission` / `domain-workflow`（Guard）/ `domain-automation`（审批节点）| 同上 |
| `AuditLogging` | COC / audit log / compliance record | `domain-audit`（唯一归属，不可跨 crate） | `responsible_crate` 必须 = `domain-audit` |

**重要说明（per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板）**：

- ❌ 本 spec **不再**以 5 域（player/economy/match/social/admin）作为 step 归属单位。5 域是 RGS 仓历史治理命名（5 位真人 Lead 问责结构），Star 仓不建立业务子域↔DDD 映射。
- ✅ Step 实际归属通过 `responsible_crate` 字符串字段显式声明到 22 domain crate 之一。
- ✅ `coordination_role` 仅用于编排器调度（如"audit_logging step 必在最后"）、可视化（按角色分组）、SLA 分级（`DecisionAuthorization` 走慢路径）。
- ✅ Lead 责任分工不通过本 enum 表达，通过 §6 "22 domain 各自 lead 与 RACI 矩阵" ADR 化（per AGENTS.md §4 #3 守门，5 域独立 Lead 是真人问责结构，star 仓 22 domain 各有真实 lead 待 DDD Review 阶段补）。

**Lead 兼任约束（per 8/21 JST 用户拍板 + AGENTS.md §4 #3 守门，仅适用于 5 域历史治理命名）**：

- ❌ **架构师不得兼任 Player / Economy 域 Lead**（Q-003 决策点冲突）
- ❌ **SRE 不得兼任 Admin 域 Lead**（COC 控制面与 SRE 责任重叠）
- ❌ **Match 域 Lead 不得兼任 Economy 域 Lead**（防止对局经济利益输送）
- ✅ 5 域 Lead 各自独立签名 saga 契约（DDD Review 阶段补签）

> 上述 Lead 兼任约束在 Star 仓**仅作历史命名兼容性 footnote**。Star 仓 saga 的实际 Lead 责任以 22 domain crate 的真实 lead 为准（per §6 G-12 待决项），不通过 5 域绑定推导。

## §4 Worktree Orchestration Saga 示例（Star 主场景）

完整流程（per [requirements §22 Worktree Orchestration 要件](../../../../requirements.md) + [spec/agents/01 §2 Lease 协议](../agents/01-agent-runtime-spec.md) + [spec/vcs/05 §2 4 Git Provider 接入](../vcs/05-real-providers-spec.md) MR 触发场景）：

| # | Step 名称 | coordination_role | responsible_crate | 触发 | 失败补偿 |
|---|---|---|---|---|---|
| 1 | `ValidateWorkItemOwnership` | `IdentityValidation` | `domain-work-item` | WorkItem 状态变更为 `IN_PROGRESS` 且用户触发 AI Task | no-op |
| 2 | `CreateWorktree` | `ResourceMutation` | `domain-worktree` | step 1 通过 | 删除已创建 worktree + 清 observed state |
| 3 | `RegisterAgentSession` | `ResourceMutation` | `domain-agent` | step 2 通过 | 撤销 session（释放 lease） |
| 4 | `StartContextBuild` | `StateObservation` | `domain-context` | step 3 通过 | 清空 ContextPacket 元数据 |
| 5 | `AuthorizeFeedbackGate` | `DecisionAuthorization` | `domain-feedback` | Agent 第一次完成 ChangeSet 上报 | 回退 ChangeSet 状态为 DRAFT |
| 6 | `TriggerValidation` | `ResourceMutation` | `domain-validation` | step 5 通过 | 标记 ValidationResult = `Aborted` |
| 7 | `LinkPullRequest` | `ResourceMutation` | `domain-scm` | step 6 通过（validation passed） | 关闭 PR（如已创建） |
| 8 | `WriteAuditLog` | `AuditLogging` | `domain-audit` | 必填，saga 必含此收尾 step | no-op（append-only 幂等） |

**Step 编排语义**（per §5 状态机 Running 路径）：

- 严格串行：每 step 必须在上一 step `execute()` 返回 `StepResult::Ok` 后才能开始
- `AuditLogging` step **必填且只能放在最后**（编排器强制，per `coordination_role` 调度规则）
- `DecisionAuthorization` step 走慢路径（人工 gate / 长 SLA，per [arch/06 §3 NFR-OP-015](../../arch/06-threat-model-nfr.md)）

**逆向补偿**（任意 step 失败触发，per §5 状态机 Compensating 路径）：

| Step | compensate 行为 | 失败处理 |
|---|---|---|
| `LinkPullRequest` | 关闭 PR（如已创建） | best-effort，失败入死信（per G-05）|
| `TriggerValidation` | 标记 ValidationResult = `Aborted` | best-effort，失败入死信 |
| `AuthorizeFeedbackGate` | 回退 ChangeSet 状态为 DRAFT | best-effort |
| `StartContextBuild` | 清空 ContextPacket 元数据 | best-effort |
| `RegisterAgentSession` | 撤销 session（释放 lease，per ADR-0030） | best-effort |
| `CreateWorktree` | 删除已创建 worktree + 清 observed state | best-effort |
| `ValidateWorkItemOwnership` | no-op | 必然成功 |
| `WriteAuditLog` | no-op | 必然成功（append-only 幂等）|

**Q-003 兼容性 footnote**：RGS 仓的 Q-003 经济交易 Saga（ValidatePlayerIdentity → EconomicDecision → ExecuteTransaction → UpdatePlayerBalance → AuditLog）可映射到本 spec 的 5 类 coordination_role 模式，**但不通过 5 域绑定**实现——RGS 仓与 Star 仓的 saga 实现细节各自独立（per AGENTS.md §5 v0.6 仓库拓扑硬约束）。

## §5 状态机

```
Pending ──execute()──> Running ──all steps OK──> Completed
   │                      │
   │                      ├──any step 失败──> Compensating ──all 补偿 OK──> Compensated
   │                      │                      │
   │                      │                      └──补偿失败──> Failed (死信, per G-05)
   │                      │
   │                      └──timeout──> Failed
   │
   └──init 失败──> Failed
```

**状态转换触发条件**：

| From | To | 触发条件 |
|---|---|---|
| Pending | Running | `Orchestrator.execute()` 调用 |
| Running | Completed | 所有 step `execute()` 成功 |
| Running | Compensating | 任一 step `execute()` 失败（自动触发）|
| Compensating | Compensated | 所有已执行 step `compensate()` 成功 |
| Compensating | Failed | 任一 `compensate()` 失败（入死信）|
| Running | Failed | saga 整体超时（per Saga.timeout_sec）|
| Pending | Failed | saga 初始化失败（如 step 注册缺失）|

**持久化**（per [spec/agents/02 §2](../agents/02-data-sources-spec.md) domain 数据存储 + [spec/cache/01 §3](../cache/01-cache-contract-spec.md) Phase G 缓存契约）：

- SagaState 写入 `agent://saga/{saga_id}` Resource URI
- 持久化时机：每 step 状态变更后立即落盘（避免跨进程崩溃丢失）
- 恢复机制：saga orchestrator 启动时扫描 `state=Running | Compensating` 的 saga 实例，自动恢复（per ADR-0030 §2 Lease 30s heartbeat 模式）

**跨进程协调**：

- 多 star-saga 实例通过 Lease + Heartbeat 抢锁（per [ADR-0030 §3](../../adr/0030-agent-lease-heartbeat-resume.md) 11 字段）
- 同一 saga_id 仅一个实例持有 lease，其他实例等待或只读
- Lease 过期 → 自动触发 compensate 路径（防 saga 半成品）

## §6 已知缺口（per 缺标比错标安全）

| # | 缺口 | 状态 | 触发 |
|---|---|---|---|
| G-01 | Saga 嵌套（saga 触发子 saga）未涉及 | 🟡 Phase G 评估 | v0.1 初版未展开 |
| G-02 | Saga 版本管理（saga definition 演进）未设计 | 🟡 Phase G 评估 | v0.1 初版未展开 |
| G-03 | 跨 region Saga 协调（multi-region 部署）未涉及 | 🟡 Phase 2+ | 多 region 部署规划 |
| G-04 | Saga 测试框架（time-travel debugging）未设计 | 🟡 Phase G 评估 | v0.1 初版未展开 |
| G-05 | Saga 监控 + 死信（per [spec/services/03 §7](../services/03-webhook-adapter-spec.md) dead letter 模式） | 🟡 Phase G 评估 | 死信模式参考 webhook adapter |
| G-06 | Saga 跨 22 domain crate 端到端时延 P99 SLA（per [ADR-0027 §3 SRE NFR](../../adr/0027-star-ide-gateway.md)）未量化 | 🟡 SRE Lead 校准 | 8-step Worktree Orchestration Saga 端到端 SLA 待 SRE Lead 拍板 |
| G-07 | Saga 与 Lease 协议集成点（per [spec/agents/01 §2 Lease 30s heartbeat](../agents/01-agent-runtime-spec.md)）未细化 | 🟡 Phase G 评估 | 复用 Lease 心跳还是独立 heartbeat |
| G-08 | 22 domain crate 的 SagaStep trait 实现边界（per [spec/agents/02 §2 22 domain 数据源清单](../agents/02-data-sources-spec.md)）未展开 | 🟡 Phase G 实现 | 22 crate 接入时定义 |
| G-09 | spec/cache/01 缓存契约 + spec/services/07 审计模型（均为 Phase G 规划目标）尚未存在 | 🟡 Phase G 同步起草 | cross-ref 引用目标待落 |
| G-10 | 补偿幂等性（同一 compensate 多次调用结果一致）未明确约束 | 🟡 Phase G 评估 | 崩溃恢复可能触发重复补偿 |
| G-11 | 22 domain crate 各自 SagaStep trait 接入清单（per [spec/agents/02 §2](../agents/02-data-sources-spec.md)）未展开 | 🟡 Phase G 实现 | 22 crate 接入时定义 responsible_crate 字段 |
| G-12 | Star 仓 22 domain 真实 lead 与 RACI 矩阵（5 域历史治理命名脱钩后，22 crate 实际 lead 责任分工） | 🟡 DDD Review 阶段 | 不通过 5 域推导，22 crate 各自 lead 待 DDD Review 补 |
| G-13 | saga spec v0.2 dual-use 警告后，5 域字符串硬编码检索（`grep "Player\|Economy\|Match\|Social\|Admin"`）残留清理 | 🟡 P3 阶段 | 仅 saga spec v0.2 改写，code base / 其他 spec 需 P3 阶段 sweep |

## §7 引用文档

- [adr/0027-star-ide-gateway.md](../../adr/0027-star-ide-gateway.md) — STAR IDE Gateway（3 通道 + 5 域责任矩阵 + SRE NFR）
- [adr/0035-phase-f-architecture.md](../../adr/0035-phase-f-architecture.md) — Phase F 整体架构（§8.2 Phase G 方向 = "跨域 Saga 协调：22 domain 跨域事务" line 265）
- [spec/agents/01-agent-runtime-spec.md](../agents/01-agent-runtime-spec.md) — Agent Runtime Spec（§2 Lease 协议，30s heartbeat / 300s TTL）
- [spec/agents/02-data-sources-spec.md](../agents/02-data-sources-spec.md) — 22 domain crate 数据源契约（§2 22 domain 清单 + `agent://{crate}/{id}` URI 模式）
- [spec/vcs/05-real-providers-spec.md](../vcs/05-real-providers-spec.md) — 4 Git Provider 接入规范（MR 触发 saga 场景）
- [spec/cache/01-cache-contract-spec.md](../cache/01-cache-contract-spec.md) — 缓存契约（**Phase G 规划目标，本 spec 引用其 §3 持久化模式**）
- [spec/services/07-audit-model.md](../flows/07-audit-model.md) — 审计模型（**Phase G 规划目标，AuditLog step 落 audit log**）
- [spec/services/01-service-adapter-spec.md](../services/01-service-adapter-spec.md) — SA 协议（§0 反污染原则参考）
- [spec/services/03-webhook-adapter-spec.md](../services/03-webhook-adapter-spec.md) — Webhook Adapter（§7 死信模式参考）

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：Saga trait + 5 域 Lead 映射 + Q-003 交易 Saga 5 步流程 + 逆向补偿 5 步 + 状态机 6 转换 + 10 已知缺口 | per ADR-0035 §8.2 Phase G 方向（line 265 "跨域 Saga 协调：22 domain 跨域事务"）+ 2026-08-27 21:59 JST 用户授权"继续, 你可以代签"（per AGENTS.md §1.0 v0.5 三次强化）|
| v0.2 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | **5 域绑定冲突修复**：§2 `Domain` enum 改为 `SagaCoordinationRole`（5 类抽象功能角色：IdentityValidation/ResourceMutation/StateObservation/DecisionAuthorization/AuditLogging）；`SagaStep` 加 `responsible_crate: &str` 字段指明 22 domain crate 之一；`SagaContext.domain_state` 改 `crate_state`（key 改为 crate 字符串）；§3 重写为"Saga 协调能力分类映射"（基于 5 类 role + 22 crate 实际归属），5 域 Lead 兼任约束降为历史命名兼容性 footnote；§4 重写为 Star 主场景"Worktree Orchestration Saga" 8 步示例（ValidateWorkItemOwnership → CreateWorktree → RegisterAgentSession → StartContextBuild → AuthorizeFeedbackGate → TriggerValidation → LinkPullRequest → WriteAuditLog），Q-003 降为 RGS 兼容性 footnote；§6 加 G-11/G-12/G-13（22 crate 接入清单 / 22 crate 真实 lead / 5 域字符串硬编码清理）；增加 dual-use 警告注释（per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板）| 2026-09-01 14:38 JST 任务"模块间协作细化"触发：用户选 A 架构层 22 Domain 协作 + L3 完整覆盖 + doc-only，明确要先解 saga spec 5 域绑定冲突 |
