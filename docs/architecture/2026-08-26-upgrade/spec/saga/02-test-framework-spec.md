# Spec-02: Saga 测试框架

> **状态**：Draft v0.1
> **日期**：2026-08-28
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per ADR-0036 §8 Phase H / 2026-08-27 21:59 JST 用户授权第三次强化

## §1 目的

定义 Star Saga 测试框架完整规范，Phase H 落地到 `crates/star-saga` 的 test 模块（per [spec/saga/01](../saga/01-saga-coordination-spec.md) Saga 抽象 + [ADR-0036 §8.2 Phase H 方向](../../adr/0036-phase-g-architecture.md) D14 Saga 测试维度）。框架需支持三件套：

1. **Time-travel debugging**（事件重放）— 基于 spec/saga/01 §5 状态机持久化
2. **Chaos test**（节点失败注入）— 验证 5 域 Lead 边界 + 补偿路径
3. **Property-based testing**（状态机全路径覆盖）— 验证不变量（termination / 终态收敛 / 5 域必经 Economy）

**为何需要独立测试框架**（per ADR-0036 §8.2 Phase H 方向 + spec/saga/01 §6 G-07 + G-09）：

1. **状态机 8 步枚举 + 5 域 Lead 决策路径爆炸**（per spec/saga/01 §4 Q-003）：手工写测试用例覆盖不全
2. **跨进程崩溃恢复**（per spec/saga/01 §1 #5 不死锁 + 不丢失）：需要注入节点崩溃 + 持久化恢复测试
3. **5 域 Lead 决策边界**（per 8/21 JST 拒绝兼任硬约束）：需要验证 Lead 兼任违规被检测
4. **Property 不变量回归**（per spec/saga/01 §5 状态机 5 终态）：防止未来演化破坏不变量
5. **可观测性**（per spec/saga/01 §1 #3）：时间线视图 + 事件审计支撑 ops 半成品事务恢复

**与现有契约关系**：

- 与 [spec/saga/01 §5 状态机](../saga/01-saga-coordination-spec.md) 关系：Property-based testing 基于 5 终态 + 6 转换做路径覆盖
- 与 [spec/saga/01 §5 SagaContext + SagaEvent](../saga/01-saga-coordination-spec.md) 关系：EventStore 复用 SagaContext 字段（saga_id / step_name / status）
- 与 [spec/agents/01 §2 Lease 协议](../agents/01-agent-runtime-spec.md) 关系：Chaos test 注入"节点崩溃"模拟 Lease 30s heartbeat 失联
- 与 [ADR-0036 §3 spec/crate 关系](../../adr/0036-phase-g-architecture.md) 关系：Phase H 测试框架对应 `crates/star-saga/tests/` 8 文件
- 与 [ADR-0036 §7 已知缺口 #7](../../adr/0036-phase-g-architecture.md) 关系：Economy Lead 决策 SLA 量化是 Chaos test 决策超时场景的前置

**测试框架与 MVP 不变量**（per ADR-0036 §8.3 Phase F + Phase G 不变量 + spec/saga/01 §0 MVP 不实装）：

- ✅ 0 unsafe
- ✅ 0 新外部依赖（`proptest` 如 workspace 无则选用 `quickcheck`，二者择一）
- ✅ 不引用未发生 commit
- ✅ 代签 per 2026-08-27 21:59 JST 用户授权第三次强化（per AGENTS.md §1.0 v0.5）

## §2 Time-travel Debugging

基于事件持久化（per [spec/saga/01 §5 状态机持久化](../saga/01-saga-coordination-spec.md) + ADR-0036 §3 spec/crate 关系表）：

```rust
// crates/star-saga/src/test/recorder.rs（计划位置，本 spec 不实装 Rust 代码）
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Saga 事件录制器：所有 SagaEvent 落 EventStore 供 replay
pub struct SagaRecorder {
    pub store: Arc<dyn EventStore>,
}

#[async_trait]
pub trait EventStore: Send + Sync {
    /// 追加事件（orchestrator 每次状态转换调用）
    async fn append(&self, event: SagaEvent) -> Result<(), SagaError>;

    /// 从指定 step 重新执行 saga（用于调试 + ops 恢复半成品事务）
    async fn replay(&self, saga_id: &str, from_step: usize) -> Result<SagaContext, SagaError>;

    /// 列所有事件（用于审计 + 时间线视图）
    async fn list_events(&self, saga_id: &str) -> Result<Vec<SagaEvent>, SagaError>;
}

/// Saga 事件：状态机每一步转换一条
pub struct SagaEvent {
    pub saga_id: String,
    pub step_name: String,
    pub event_type: SagaEventType,
    pub timestamp: i64,  // Unix ms
    pub data: serde_json::Value,  // SagaContext 快照
}

pub enum SagaEventType {
    StepStarted,
    StepSucceeded,
    StepFailed,
    CompensateStarted,
    CompensateSucceeded,
    CompensateFailed,
    SagaCompleted,
    SagaAborted,
}
```

**API 契约**：

- `replay(saga_id, from_step)` — 从指定 step 重新执行 saga（用于调试 + ops 恢复半成品事务）
  - from_step 边界：0 = 从头跑；> 当前 step 数 = 错误（EventStore::list_events 校验）
  - 冲突：与正在跑的 saga 同 saga_id 时拒绝（per §5 已知缺口 #2）
- `list_events(saga_id)` — 列所有事件（按 timestamp 升序）
  - 用于审计 + 时间线视图 + property-based 状态机断言

**时间线视图**（CLI 调试输出，per [spec/services/01 §0 反污染原则](../services/01-service-adapter-spec.md) + ADR-0025）：

```
saga_id: q003-trade-7f8a
12:34:56.123  StepStarted       ValidatePlayerIdentity
12:34:56.456  StepSucceeded     ValidatePlayerIdentity (3ms)
12:34:56.789  StepStarted       EconomicDecision
12:34:57.012  StepSucceeded     EconomicDecision (223ms, Economy Lead 拍板)
12:34:57.234  StepStarted       ExecuteTransaction
12:34:57.890  StepFailed        ExecuteTransaction (network timeout)
12:34:57.901  CompensateStarted UpdatePlayerBalance
12:34:58.123  CompensateSucceeded UpdatePlayerBalance (best-effort 成功)
12:34:58.456  SagaAborted       q003-trade-7f8a
```

**实现路径**（per ADR-0036 §3 spec/crate 关系表 D14 `tests/orchestrator.rs`）：

- `crates/star-saga/src/test/recorder.rs`（~80 行）：SagaRecorder + EventStore trait
- `crates/star-saga/src/test/event.rs`（~60 行）：SagaEvent + SagaEventType enum
- 后端：Phase H 内存 InMemory（per ADR-0036 D13 star-cache 模式），Phase H+ 切 SQLite/Postgres

## §3 Chaos Test

**故障注入场景矩阵**（per spec/saga/01 §5 状态机 8 步 + §4 Q-003 5 域 Lead 决策路径 + §6 G-07 Lead 决策 SLA）：

| # | 场景 | 注入点 | 预期行为 | 优先级 |
|---|------|--------|----------|--------|
| C-01 | Step 执行超时 | `StepExecutor.execute_step` 包装 `tokio::time::timeout` | Saga 走 Compensating → Compensated | P0 |
| C-02 | Step 永久失败 | `Step::execute` 返回 `Err(SagaError::StepFailed)` | Saga 走 Compensating → Compensated | P0 |
| C-03 | Compensate 失败 | `Step::compensate` 返回 `Err(SagaError::CompensateFailed)` | Saga 走 Failed + 死信队列（per G-05 死信）| P0 |
| C-04 | 网络瞬断 | Mock 网络层（chaos-mesh / 手动 mock）| 自动重试 3 次后 Compensate（per [spec/services/03 §7 死信](../services/03-event-bus-spec.md) 重试策略）| P1 |
| C-05 | Step 重复执行 | 同 saga_id 重入 | 幂等（去重，per [spec/cache/01 §3 缓存契约](../cache/01-cache-contract-spec.md) key = saga_id+step_name）| P1 |
| C-06 | Saga 节点崩溃 | Orchestrator state 持久化（per spec/saga/01 §5 状态机持久化）| 启动恢复 + 续跑（per [spec/agents/01 §2 Lease 30s heartbeat](../agents/01-agent-runtime-spec.md) 失联检测）| P0 |
| C-07 | 5 域 Lead 决策超时 | Economy Lead 决策 SLA 超（per ADR-0036 §7 #7 决策 SLA 量化）| 走 reject 分支（per [spec/saga/01 §4 Q-003 决策点](../saga/01-saga-coordination-spec.md) 大额交易显式签字）| P0 |

**注入框架选型**（per ADR-0036 §8.3 0 新外部依赖硬约束）：

- ✅ **Phase H 首选：手动 mock**（写 test 专用 Step 实现 + `tokio::time::pause()`）—— 0 外部依赖
- ⏳ **Phase H+ 备选：`chaos-mesh`**（per [arch/06 NFR-OP](../../adr/0035-phase-f-architecture.md) 性能混沌测试）—— 需 SRE Lead 评估引入

**C-06 Saga 节点崩溃恢复详细路径**（per [spec/agents/01 §2 Lease 协议](../agents/01-agent-runtime-spec.md) 30s heartbeat + 300s TTL）：

1. Orchestrator 启动时扫描 EventStore 中 `SagaEventType::StepStarted` 但无后续 `StepSucceeded/StepFailed` 的 saga（孤儿 step）
2. 检查 Lease 是否过期（per [spec/agents/01 §2 TTL 300s](../agents/01-agent-runtime-spec.md)）
3. 未过期 = 续跑；已过期 = 触发 Compensate（per spec/saga/01 §5 状态机 Compensating 路径）
4. 续跑前 `EventStore::replay(saga_id, last_completed_step + 1)` 重建 SagaContext

**C-07 5 域 Lead 决策 SLA 量化待补**（per ADR-0036 §7 #7）：

- 当前缺口：决策响应时间阈值未量化（如 < 100ms）
- Phase H 测试用占位值（如 50ms / 500ms / 5s 三个档位跑 reject 分支）
- 最终值由 SRE Lead + Economy Lead 协同拍板（per ADR-0036 §7 #7 拍板路径）

## §4 Property-based Testing

基于 [spec/saga/01 §5 状态机](../saga/01-saga-coordination-spec.md)（Pending → Running → Completed/Compensating/Compensated/Failed）：

**不变量列表**（5 域独立 Lead per 8/21 JST 拒绝兼任 + spec/saga/01 §1 #5 不死锁 + 不丢失）：

| # | 不变量 | 形式化 | 违反示例 |
|---|--------|--------|----------|
| P-01 | Saga 终止 | 任意 Saga 必终止（无死循环）| Running 状态永远不转换 |
| P-02 | 5 终态收敛 | 任意 Saga 状态机只走 5 个终态之一（Completed / Compensated / Failed / Aborted / Pending）| 进入未定义状态 |
| P-03 | Step 失败触发 Compensate | 任意 Step 失败必触发 Compensating 转换 | Step 失败后直接走 Completed |
| P-04 | Compensate 失败必报警 | 任意 Compensate 失败必入死信（per G-05），不静默吞 | Compensate 失败返回 Ok |
| P-05 | 5 域 Saga 必经 Economy 决策 | 任意跨 player+economy 的 saga 必含 EconomicDecision step（per [ADR-0036 §3 5 域决策矩阵](../../adr/0036-phase-g-architecture.md)）| 跨域 saga 跳过 Economy 域 |
| P-06 | 状态机转换幂等 | 同一 SagaEventType 重复不破坏状态 | StepSucceeded 重复落事件 |

**测试框架选型**（per ADR-0036 §8.3 0 新外部依赖）：

- ✅ **Phase H 首选：`proptest`**（如 workspace 已有，per [spec/vcs/05 测试依赖](../vcs/05-real-providers-spec.md) 历史引入）
- ⏳ **Phase H 备选：`quickcheck`**（proptest 不可用时）

**Property 测试用例形态**（per `proptest` 标准）：

```rust
// crates/star-saga/src/test/property.rs（计划位置）
use proptest::prelude::*;

proptest! {
    /// P-01: 任意 Saga 必终止
    #[test]
    fn saga_terminates(steps in vec(any::<TestStep>(), 1..20)) {
        let saga = Saga { name: "p01".into(), steps, timeout_sec: 60 };
        let result = tokio_test::block_on(orchestrator.execute(&saga));
        prop_assert!(matches!(result, Ok(_) | Err(SagaError::Failed)));
    }

    /// P-05: 5 域 Saga 必经 Economy 决策
    #[test]
    fn cross_domain_saga_must_include_economy(
        steps in vec(any::<TestStep>(), 3..10)
            .prop_map(|mut v| { v.push(TestStep::EconomicDecision); v })
    ) {
        let saga = Saga { name: "p05".into(), steps, timeout_sec: 60 };
        let has_economy = saga.steps.iter().any(|s| s.domain() == Domain::Economy);
        prop_assert!(has_economy);
    }

    // P-02 / P-03 / P-04 / P-06 略
}
```

**覆盖率目标**（per ADR-0036 §5 token-OLU 15-23M Phase G 估算，Phase H 增量）：

- 状态机 8 步 × 5 终态 = 40 路径
- 5 域 Lead 决策矩阵 5×5 = 25 组合（per ADR-0036 §4 5 域决策矩阵）
- 故障注入 7 场景 × 2 注入深度 = 14 路径
- 总目标：~80 路径覆盖（per `proptest` 默认 256 case/run，可达 20K 路径/单 property）

## §5 已知缺口

1. **事件存储后端选型**（Phase H 内存 InMemory / Phase H+ SQLite/Postgres）—— 决策依赖 ops 团队
2. **Saga replay 与当前正在跑的 saga 冲突处理** —— replay 期间是否允许同 saga_id 新事件？需锁定策略
3. **Chaos test 框架选型**（手动 mock vs chaos-mesh）—— per §3 C-04 注入点决策依赖 SRE Lead
4. **5 域 Lead 决策 SLA 量化**（per [ADR-0036 §7 #7](../../adr/0036-phase-g-architecture.md)）—— SRE Lead + Economy Lead 协同拍板
5. **Property-based testing 完整覆盖率**（state machine + 5 域决策组合 + 7 chaos 场景交叉）—— 当前 §4 列出 6 不变量，剩余 5 决策矩阵 × 7 chaos 场景 = 35 组合待补
6. **性能开销**（replay 慢 replay 快 vs 磁盘 IO）—— EventStore append/replay 延迟指标需 SRE Lead 量化
7. **5 域 Lead 真实身份签字**（per ADR-0036 §4 §7 #6）—— DDD Review 阶段补签，per 8/21 JST 5 域独立 Lead 拒绝兼任硬约束
8. **Test fixture 复用**（per spec/saga/01 §4 Q-003 5 步交易场景）—— 是否在 `crates/star-saga/tests/fixtures/` 抽象 fixture 给所有 property + chaos test 共享，待 Phase H 实施时决策

## §6 引用文档

- [spec/saga/01-saga-coordination-spec.md](../saga/01-saga-coordination-spec.md) — Saga 抽象 + 5 域 Lead + 状态机 8 步
- [spec/agents/01-agent-runtime-spec.md](../agents/01-agent-runtime-spec.md) — Lease 协议 30s heartbeat / 300s TTL
- [spec/services/01-service-adapter-spec.md](../services/01-service-adapter-spec.md) — 反污染原则（test mock 边界）
- [spec/services/03-event-bus-spec.md](../services/03-event-bus-spec.md) — 死信 + 重试策略
- [spec/cache/01-cache-contract-spec.md](../cache/01-cache-contract-spec.md) — 缓存契约（EventStore key 复用）
- [ADR-0036 §3 spec/crate 关系](../../adr/0036-phase-g-architecture.md) — Phase H D14 Saga 测试维度映射
- [ADR-0036 §7 已知缺口 #7](../../adr/0036-phase-g-architecture.md) — Economy Lead 决策 SLA 量化
- [ADR-0036 §8.2 Phase H 方向](../../adr/0036-phase-g-architecture.md) — 测试三件套（time-travel + chaos + property）
- [ADR-0025 厂商适配反污染](../../adr/0025-vendor-adapter-anti-contamination.md) — test mock 隔离原则

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：Time-travel + Chaos + Property-based 三件套 + 7 chaos 场景 + 6 不变量 + 8 已知缺口 | ADR-0036 §8.2 Phase H 方向 |

---

**Phase H 测试三件套已就绪：Time-travel debugging（SagaRecorder + EventStore） + Chaos test（7 场景矩阵 + 注入框架选型） + Property-based testing（6 不变量 + proptest/quickcheck 选型）。已知缺口 8 项已列明，待 SRE Lead + 5 域 Lead 协同拍板的 SLA 量化、存储后端选型、chaos 框架选型在 Phase H 实施时收口。**
