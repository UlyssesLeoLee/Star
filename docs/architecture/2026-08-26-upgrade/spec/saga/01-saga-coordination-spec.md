# Spec-01: 跨域 Saga 协调

> **状态**：Draft v0.1
> **日期**：2026-08-27
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per ADR-0035 §8 Phase G / 2026-08-27 21:59 JST 用户授权

## §1 目的

定义 Star 跨域 Saga 协调契约（5 域独立 Lead per 8/21 JST），实现 Q-003 经济域决策场景：player 域发起交易 → economy 域决策 → 通知 player 域结果。本 spec 是 Phase G 落地 `crates/star-saga` 的依据。

**为何需要 Saga 抽象层**（per [ADR-0027 §2 STAR IDE Gateway](../../adr/0027-star-ide-gateway.md) 5 通道 + Fallback Ladder 4 级 + 5 域独立 Lead 派生需求 + [ADR-0035 §8.2 Phase G 方向](../../adr/0035-phase-f-architecture.md) "跨域 Saga 协调：22 domain 跨域事务" line 265）：

1. **跨域事务一致性**：单次用户操作可能跨 5 域（如 player 域发起交易 → economy 域决策 → admin 域审计），任一域失败需逆向补偿，否则系统状态不一致
2. **5 域独立 Lead 决策边界**（per 8/21 JST 用户偏好"不接受兼任"）：Q-003 经济域决策点必须 Economy Lead 独立拍板，不能由 player 域 Lead 兼任决策
3. **可观测性**：Saga 状态机 + 补偿日志为 ops 提供"半成品事务"恢复入口
4. **测试可替换**：Saga trait 让 conformance test 用 mock provider 跑通（per [spec/services/01 §0](../services/01-service-adapter-spec.md) 反污染原则 + ADR-0025）
5. **不死锁 + 不丢失**：持久化 + 补偿保证 saga 实例即使跨进程崩溃也能恢复

**Saga 与现有契约关系**：
- 与 [spec/agents/01 §2 Lease 协议](../agents/01-agent-runtime-spec.md) 关系：Saga 状态持久化复用 Lease 30s heartbeat 周期
- 与 [spec/agents/02 §2 22 domain 数据源清单](../agents/02-data-sources-spec.md) 关系：SagaStep 通过 `agent://{crate}/{id}` Resource URI 调用 22 domain crate
- 与 [spec/vcs/05 §2 4 Git Provider 接入规范](../vcs/05-real-providers-spec.md) 关系：MR/PR 创建作为典型 saga 触发场景（player 域发起 → economy 域记账 → admin 域审计）
- 与 [spec/cache/01 §3 缓存契约](../cache/01-cache-contract-spec.md)（Phase G）关系：Saga 状态机持久化复用 star-cache
- 与 [spec/services/07 审计模型](../services/07-audit-model.md)（Phase G）关系：Admin 域 AuditLog step 落 audit log

## §2 Saga 抽象

```rust
// crates/star-saga/src/lib.rs（计划位置，本 spec 不实装 Rust 代码）
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 5 域枚举（per 8/21 JST 用户偏好"5 域独立 Lead，不接受兼任"）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Domain {
    Player,    // 用户身份 + 玩家域
    Economy,   // 交易/经济/决策（Q-003 核心决策点）
    Match,     // 匹配/对局
    Social,    // 社交/好友
    Admin,     // COC/审计/合规（COC 独立控制面）
}

/// Saga 步骤 trait：5 域 Lead 各自实现自己域的 step
#[async_trait]
pub trait SagaStep: Send + Sync {
    /// step 名称（如 "ValidatePlayerIdentity" / "EconomicDecision"）
    fn name(&self) -> &str;

    /// 所属域（per 5 域独立 Lead，每 step 必须归属一个域）
    fn domain(&self) -> Domain;

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
    pub domain_state: std::collections::HashMap<Domain, serde_json::Value>,
    pub compensation_log: Vec<CompensationEntry>,
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

- `domain()` 字段**强制**每 step 标注所属域（5 域独立 Lead 责任边界不可混用）
- `compensate()` 失败不阻断其他 step 补偿（best-effort compensation，per §6 已知缺口 G-05 死信处理）
- `SagaContext.domain_state` 允许 step 跨 step 传值（如 EconomicDecision 结果供 ExecuteTransaction 消费）
- 编排器**不**直接调 step trait，step 通过 Application Service 注册（per [arch/01 §4 职责分层](../arch/01-current-architecture-analysis.md)）

## §3 5 域独立 Lead 映射

per 8/21 JST 用户偏好（不可兼任）+ [ADR-0035 §4 5 决策 D6-D10](../../adr/0035-phase-f-architecture.md) + [ADR-0034 §4 5 域责任矩阵](../../adr/0034-phase-e-architecture.md) + AGENTS.md §4 #3 守门：

| 域 | 职责 | SagaStep 范围 | 独立 Lead 签字 |
|---|---|---|---|
| Player | 用户身份 + 玩家域 | player 域 step（如 ValidatePlayerIdentity / UpdatePlayerBalance） | ⏳ Player Lead (DDD Review 阶段补) |
| Economy | 交易/经济/决策 | economy 域 step（**Q-003 核心决策点** —— EconomicDecision step 需 Economy Lead 独立拍板） | ⏳ Economy Lead |
| Match | 匹配/对局 | match 域 step（如 CreateMatch / JoinMatch） | ⏳ Match Lead |
| Social | 社交/好友 | social 域 step（如 SendFriendRequest / UpdateFriendList） | ⏳ Social Lead |
| Admin | COC/审计/合规 | admin 域 step（**COC 独立控制面** —— AuditLog step 不可由 SRE 域兼任） | ⏳ Admin Lead |

**5 域 Lead 兼任约束**（per 8/21 JST 用户拍板 + AGENTS.md §4 #3 守门）：

- ❌ **架构师不得兼任 Player / Economy Lead**（Q-003 决策点冲突）
- ❌ **SRE 不得兼任 Admin Lead**（COC 控制面与 SRE 责任重叠）
- ❌ **Match Lead 不得兼任 Economy Lead**（防止对局经济利益输送）
- ✅ 5 域 Lead 各自独立签名 saga 契约（DDD Review 阶段补签）

**与现有域分层的对应**：
- 与 [spec/agents/02 §2 22 domain crate 数据源清单](../agents/02-data-sources-spec.md) 的 22 核心 domain crate 对应：5 域是 saga 编排视角，22 crate 是数据访问视角
- 与 [ADR-0027 §2 STAR IDE Gateway 3 通道](../../adr/0027-star-ide-gateway.md) 对应：IDE Gateway 入口触发 saga，5 域 Lead 提供 step 实现

## §4 Q-003 交易 Saga 示例

完整流程（per [spec/agents/01 §2](../agents/01-agent-runtime-spec.md) Lease 协议 + [spec/vcs/05 §2 4 Git Provider 接入](../vcs/05-real-providers-spec.md) MR 触发场景）：

1. **player 域 step**: `ValidatePlayerIdentity` — 检查 player_id 有效（调 domain-agent crate）
2. **economy 域 step**: `EconomicDecision` — 决策（Q-003 核心）— 需 Economy Lead 独立决策（调 domain-economy crate）
3. **economy 域 step**: `ExecuteTransaction` — 执行转账（调 domain-economy crate）
4. **player 域 step**: `UpdatePlayerBalance` — 更新余额（调 domain-player crate）
5. **admin 域 step**: `AuditLog` — 记录审计日志（per [spec/services/07 审计模型](../services/07-audit-model.md) Phase G，调 domain-admin crate）

**逆向补偿**（任意 step 失败触发，per §5 状态机 Compensating 路径）：

| Step | compensate 行为 | 失败处理 |
|---|---|---|
| `UpdatePlayerBalance` | 回滚余额 | best-effort，失败入死信（per G-05）|
| `ExecuteTransaction` | 反向转账 | best-effort，失败入死信 |
| `EconomicDecision` | 通知（no-op，pure decision）| 必然成功 |
| `ValidatePlayerIdentity` | no-op | 必然成功 |
| `AuditLog` | no-op | 必然成功 |

**Q-003 决策点**（per §3 Economy Lead 独立决策边界）：

- EconomicDecision step **不可被自动审批**（per [arch/06 §3 NFR-OP-015](../arch/06-threat-model-nfr.md) 关键路径人工 gate 原则）
- 大额交易（> threshold）需 Economy Lead 显式签字（per ADR-0027 §3 SRE NFR 待 SRE Lead 量化）
- Economy Lead 决策 SLA 未量化（per §6 已知缺口 G-06）

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
| G-06 | Q-003 Economy Lead 决策 SLA（per [ADR-0027 §3 SRE NFR](../../adr/0027-star-ide-gateway.md)）未量化 | 🟡 SRE Lead 校准 | 大额交易 SLA 待 SRE Lead 拍板 |
| G-07 | Saga 与 Lease 协议集成点（per [spec/agents/01 §2 Lease 30s heartbeat](../agents/01-agent-runtime-spec.md)）未细化 | 🟡 Phase G 评估 | 复用 Lease 心跳还是独立 heartbeat |
| G-08 | 22 domain crate 的 SagaStep trait 实现边界（per [spec/agents/02 §2 22 domain 数据源清单](../agents/02-data-sources-spec.md)）未展开 | 🟡 Phase G 实现 | 22 crate 接入时定义 |
| G-09 | spec/cache/01 缓存契约 + spec/services/07 审计模型（均为 Phase G 规划目标）尚未存在 | 🟡 Phase G 同步起草 | cross-ref 引用目标待落 |
| G-10 | 补偿幂等性（同一 compensate 多次调用结果一致）未明确约束 | 🟡 Phase G 评估 | 崩溃恢复可能触发重复补偿 |

## §7 引用文档

- [adr/0027-star-ide-gateway.md](../../adr/0027-star-ide-gateway.md) — STAR IDE Gateway（3 通道 + 5 域责任矩阵 + SRE NFR）
- [adr/0035-phase-f-architecture.md](../../adr/0035-phase-f-architecture.md) — Phase F 整体架构（§8.2 Phase G 方向 = "跨域 Saga 协调：22 domain 跨域事务" line 265）
- [spec/agents/01-agent-runtime-spec.md](../agents/01-agent-runtime-spec.md) — Agent Runtime Spec（§2 Lease 协议，30s heartbeat / 300s TTL）
- [spec/agents/02-data-sources-spec.md](../agents/02-data-sources-spec.md) — 22 domain crate 数据源契约（§2 22 domain 清单 + `agent://{crate}/{id}` URI 模式）
- [spec/vcs/05-real-providers-spec.md](../vcs/05-real-providers-spec.md) — 4 Git Provider 接入规范（MR 触发 saga 场景）
- [spec/cache/01-cache-contract-spec.md](../cache/01-cache-contract-spec.md) — 缓存契约（**Phase G 规划目标，本 spec 引用其 §3 持久化模式**）
- [spec/services/07-audit-model.md](../services/07-audit-model.md) — 审计模型（**Phase G 规划目标，AuditLog step 落 audit log**）
- [spec/services/01-service-adapter-spec.md](../services/01-service-adapter-spec.md) — SA 协议（§0 反污染原则参考）
- [spec/services/03-webhook-adapter-spec.md](../services/03-webhook-adapter-spec.md) — Webhook Adapter（§7 死信模式参考）

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：Saga trait + 5 域 Lead 映射 + Q-003 交易 Saga 5 步流程 + 逆向补偿 5 步 + 状态机 6 转换 + 10 已知缺口 | per ADR-0035 §8.2 Phase G 方向（line 265 "跨域 Saga 协调：22 domain 跨域事务"）+ 2026-08-27 21:59 JST 用户授权"继续, 你可以代签"（per AGENTS.md §1.0 v0.5 三次强化）|
