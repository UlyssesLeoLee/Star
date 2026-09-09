# ARG-ARCH-001 ARG.2 arg-bridge 詳細実装設計 (per brief v0.50 §14.11)

> **Status**: 🟡 Draft v0.1 (per 2026-09-10 01:36 JST 派发 brief v0.50 §14.11 ARG.2-11 docs 阶段)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md)
> **承接**: 守门 #14 v2 拍板 D + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 + v0.45 ARG.1 docs (`c678db7` 5 doc 完整化基础)

---

## 0. 目的 (Objective)

本詳細実装設計承接基本設計書 §1.1 5-tier 架构 Bridge Tier + DD §3.4 AgentLease 接口, 定义 **ARG.2 `crates/arg-bridge` 4 子模块** (MemgraphEventListener / LangGraphStateUpdater / PeriodFlushWorker / OfflineQueue) 的 **5 module 跟 Memgraph Bolt 7687 + LangGraph TopAgentState 5 Reducer 跨进程同步协议** (5 内部协议) 的实施路径, 包含:

1. **4 子模块物理布局** (per BD-AGENT-RELATIONSHIP-001 v0.1 §3 + DD §3.4)
2. **5 内部协议 schema** (arg_edge_changed / arg_dispatch_route / arg_context_inject / arg_trust_score_update / arg_achievement_unlocked)
3. **Memgraph Bolt subscription 协议** (Bolt 4.0+ SubscriptionRun 包装, 走 `bolt://localhost:7687`)
4. **30s 周期 flush 协议** (PeriodFlushWorker 单线程, 跨 arg-bridge 共享)
5. **sled 离线降级 + 5min 重试协议** (OfflineQueue 嵌入 5 module 跨进程 pub/sub)
6. **5 Reducer 跟 LangGraph TopAgentState 集成** (5 channel: arg_agents / arg_edges / arg_trust_scores / arg_dispatch_overrides / arg_achievements_unlocked)
7. **跨 session 续做边界** (per WBS v0.18 line 663 估 ~22M tokens / 5-7 session, 落档后 v0.51+)

**Token 估算**: ARG.2 docs 阶段 ~1.5M (per v0.45 brief §1 估 22M / 9 docs 均分); Rust 实装跨 session 续 (per 已知缺口 #1, 估 5-7M / 5-7 session).

---

## 1. 4 子模块物理布局

### 1.1 crates/arg-bridge 目录结构

```
crates/arg-bridge/
├── Cargo.toml                  # workspace 47 成员 (per AGENTS.md §4.2 "47 个 package")
├── src/
│   ├── lib.rs                  # pub mod 4 + 5 协议导出
│   ├── listener.rs             # C-13 MemgraphEventListener (订阅 Bolt subscription)
│   ├── updater.rs              # C-14 LangGraphStateUpdater (5 Reducer)
│   ├── flush.rs                # C-15 PeriodFlushWorker (30s 周期)
│   ├── offline.rs              # C-16 OfflineQueue (sled 嵌入 + 5min 重试)
│   ├── protocol.rs             # 5 内部协议 schema (Rust struct + serde_json)
│   ├── error.rs                # BridgeError 6 类 (BoltSubscribeFailed / LangGraphReducerFailed / FlushTimeout / OfflinePersistFailed / MemgraphDown / InternalError)
│   └── tests/                  # 33 IT 嵌入 (per ARG.1 100% UT baseline 模式)
│       ├── listener_it.rs      # 8 IT (订阅协议 + 5 协议 fanout)
│       ├── updater_it.rs       # 8 IT (5 Reducer 集成)
│       ├── flush_it.rs         # 8 IT (30s 周期 + 幂等)
│       └── offline_it.rs       # 9 IT (sled 嵌入 + 5min 重试 + 跨进程 pub/sub)
```

**4 子模块依赖关系** (per 基本 §1.1 Bridge Tier):

| 子模块 | 上游依赖 | 下游消费者 | 跨进程边界 |
|---|---|---|---|
| **C-13 MemgraphEventListener** | Memgraph Bolt 7687 + C-8 MemgraphClient (per ARG.1) | C-14 + C-15 (本地 in-process mpsc) | Bolt subscription (跨进程) |
| **C-14 LangGraphStateUpdater** | C-13 + LangGraph TopAgentState 5 Reducer | TMO 7 节点 (per §6.1 v18 强类型 Reducer) | in-process 推 (同进程) |
| **C-15 PeriodFlushWorker** | C-13 + C-16 | C-14 (周期合并) | 30s 周期 (同进程) |
| **C-16 OfflineQueue** | sled 1.0 embedded DB (per P0-1 actor context 实证) | C-15 (重试) | 跨进程 pub/sub (sled topic) |

### 1.2 跟 ARG.1 crates/arg 6 子模块差异

| 维度 | ARG.1 crates/arg | ARG.2 crates/arg-bridge |
|---|---|---|
| 物理 Tier | Data Tier (主) | Bridge Tier (主) |
| 主功能 | Memgraph 持久化 + Cypher 缓存 | Memgraph 订阅 + 跨 LangGraph 同步 + 离线降级 |
| 5 module 命中 | ArgCrate (主) + LLMService (子) + AgentLease (子) | AgentLease (主) |
| 跨进程协议 | Memgraph Bolt (client side) | Memgraph Bolt subscription (server side) + LangGraph 5 Reducer (in-process) |
| 离线降级 | (无, ARG.1 是 online-only) | sled 嵌入 (per OfflineQueue) |
| Token 估 | 6M (per ARG.1 实测 4.5-5M) | ~7M (per WBS v0.18 估) |
| Session 估 | 4-5 (per ARG.1 实证) | 5-7 (跨 session 续) |

---

## 2. 5 内部协议 schema (per 5 effect 维度)

### 2.1 5 协议总表

| # | 协议 | 触发源 | 5 module 命中 | 持久化层 | 跨 LangGraph 同步 |
|---|---|---|---|---|---|
| 1 | `arg_edge_changed` | C-17 ARGDispatchRouter (4 维度 #1) | SubAgentOrchestrator | Memgraph Edge (M) | TMO arg_edges channel |
| 2 | `arg_dispatch_route` | C-17 + C-18 (4 维度 #1+#2) | SubAgentOrchestrator | Memgraph Edge (M, weight 字段) | TMO arg_dispatch_overrides channel |
| 3 | `arg_context_inject` | C-18 (4 维度 #2) | SubAgentOrchestrator | Memgraph Edge (M, shared_context 字段) | TMO arg_context_inject_audit channel |
| 4 | `arg_trust_score_update` | C-19 (4 维度 #3) | SubAgentOrchestrator | Memgraph Agent (M, trust_score 字段) | TMO arg_trust_scores channel |
| 5 | `arg_achievement_unlocked` | C-21 (8 拓扑成就) | SubAgentOrchestrator | PG unlocks (T) | TMO arg_achievements_unlocked channel |

### 2.2 协议 schema 草案 (Rust struct + serde_json)

```rust
// crates/arg-bridge/src/protocol.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 协议 #1: arg_edge_changed (4 维度 #1 主)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgEdgeChanged {
    pub edge_id: Uuid,
    pub from_agent_id: Uuid,
    pub to_agent_id: Uuid,
    pub relationship_type: String,    // 10 类之一 (per ARG.1 C-3)
    pub weight: f64,                   // 0.0 - 1.0
    pub created_at: DateTime<Utc>,
    pub trigger_sa: String,            // 9 SA 之一
}

/// 协议 #2: arg_dispatch_route (4 维度 #1 dispatch 覆盖)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgDispatchRoute {
    pub route_id: Uuid,
    pub from_agent_id: Uuid,
    pub to_agent_id: Uuid,
    pub sa_type: String,               // 9 SA 之一
    pub weight: f64,                   // 信任分加权
    pub override_reason: String,
    pub created_at: DateTime<Utc>,
}

/// 协议 #3: arg_context_inject (4 维度 #2 上下文共享)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgContextInject {
    pub inject_id: Uuid,
    pub source_agent_id: Uuid,
    pub target_agent_id: Uuid,
    pub shared_context: serde_json::Value,  // 任意 JSON (业务数据)
    pub sa_type: String,
    pub created_at: DateTime<Utc>,
}

/// 协议 #4: arg_trust_score_update (4 维度 #3 信任度)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgTrustScoreUpdate {
    pub agent_id: Uuid,
    pub old_score: f64,                // 0.0 - 1.0
    pub new_score: f64,
    pub delta_reason: String,          // 5 段之一 (per ARG.1 C-6)
    pub trigger_sa: String,
    pub created_at: DateTime<Utc>,
}

/// 协议 #5: arg_achievement_unlocked (8 拓扑成就)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgAchievementUnlocked {
    pub unlock_id: Uuid,
    pub agent_id: Uuid,
    pub achievement_id: String,        // 20 成就之一 (per ARG.1 C-5)
    pub topology_snapshot: serde_json::Value,  // 8 拓扑 Cypher snapshot
    pub unlock_text: String,           // LLMService 生成 (per ARG.1 llm 子模块)
    pub created_at: DateTime<Utc>,
}
```

### 2.3 协议序列化 + Memgraph subscription 协议

5 协议**统一走 JSON 序列化** (serde_json), 落 Memgraph subscription channel (`arg_bridge_events` topic), 跨进程 fanout 到:
- C-14 LangGraphStateUpdater (本进程)
- C-15 PeriodFlushWorker (本进程, 周期合并)
- C-16 OfflineQueue (跨进程, sled 持久化降级)

**Memgraph subscription 协议** (Bolt 4.0+ `SubscriptionRun`):
```cypher
// Memgraph 端订阅语句
SUBSCRIBE COMMIT
MATCH (n) WHERE n.__topic__ = 'arg_bridge_events'
RETURN n.payload AS payload, n.created_at AS created_at
```

---

## 3. MemgraphEventListener 详细 (C-13)

### 3.1 Bolt subscription 协议 (per Memgraph 2.14 文档)

```rust
// crates/arg-bridge/src/listener.rs
use crate::protocol::*;
use memgraph_client::MemgraphClient;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct MemgraphEventListener {
    memgraph: Arc<MemgraphClient>,
    output_tx: mpsc::Sender<ArgBridgeEvent>,
    subscription_query: String,
}

impl MemgraphEventListener {
    pub async fn run(&self) -> Result<(), BridgeError> {
        let mut stream = self.memgraph.subscribe(&self.subscription_query).await?;
        while let Some(event) = stream.next().await {
            let payload: serde_json::Value = event.payload()?;
            let bridge_event = match event.topic().as_str() {
                "arg_edge_changed" => ArgBridgeEvent::EdgeChanged(serde_json::from_value(payload)?),
                "arg_dispatch_route" => ArgBridgeEvent::DispatchRoute(serde_json::from_value(payload)?),
                "arg_context_inject" => ArgBridgeEvent::ContextInject(serde_json::from_value(payload)?),
                "arg_trust_score_update" => ArgBridgeEvent::TrustScoreUpdate(serde_json::from_value(payload)?),
                "arg_achievement_unlocked" => ArgBridgeEvent::AchievementUnlocked(serde_json::from_value(payload)?),
                _ => return Err(BridgeError::UnknownTopic(event.topic())),
            };
            self.output_tx.send(bridge_event).await?;
        }
        Ok(())
    }
}

pub enum ArgBridgeEvent {
    EdgeChanged(ArgEdgeChanged),
    DispatchRoute(ArgDispatchRoute),
    ContextInject(ArgContextInject),
    TrustScoreUpdate(ArgTrustScoreUpdate),
    AchievementUnlocked(ArgAchievementUnlocked),
}
```

### 3.2 失败兜底 (per 守门 #9 v3 subprocess 隔离 + 守门 #5 env 安全)

- **Bolt subscription 断开**: 自动重连 (3 retry, 5s 间隔), 失败 → 落 OfflineQueue (per C-16)
- **协议反序列化失败**: 写 error log, 不阻断后续事件
- **Memgraph 进程崩溃**: PeriodFlushWorker 检测心跳, 30s 内未恢复 → 启 OfflineQueue 5min 重试

---

## 4. LangGraphStateUpdater 详细 (C-14)

### 4.1 5 Reducer 跟 TopAgentState 集成 (per LangGraph 9/3 阶段)

```rust
// crates/arg-bridge/src/updater.rs
use langgraph_runtime::TopAgentState;

pub struct LangGraphStateUpdater {
    state: Arc<TopAgentState>,
    input_rx: mpsc::Receiver<ArgBridgeEvent>,
}

impl LangGraphStateUpdater {
    pub async fn run(&mut self) -> Result<(), BridgeError> {
        while let Some(event) = self.input_rx.recv().await {
            match event {
                ArgBridgeEvent::EdgeChanged(e) => {
                    self.state.dispatch_arg_edge_changed(e).await?;
                }
                ArgBridgeEvent::DispatchRoute(r) => {
                    self.state.dispatch_arg_dispatch_route(r).await?;
                }
                ArgBridgeEvent::ContextInject(c) => {
                    self.state.dispatch_arg_context_inject(c).await?;
                }
                ArgBridgeEvent::TrustScoreUpdate(t) => {
                    self.state.dispatch_arg_trust_score_update(t).await?;
                }
                ArgBridgeEvent::AchievementUnlocked(a) => {
                    self.state.dispatch_arg_achievement_unlocked(a).await?;
                }
            }
        }
        Ok(())
    }
}
```

### 4.2 5 Reducer 派生规 (per 守门 #3 v2 + §1.1 平行关系)

5 Reducer 跟 LangGraph 9/3 阶段 TMO 7 节点**共享** TopAgentState 5 channel, **不重建** 状态层 (per 守门 #3 v2 平行 ≠ 重叠).

| Reducer | TopAgentState channel | 5 module 命中 |
|---|---|---|
| `arg_agents` | agents map<AgentId, AgentSummary> | ArgCrate |
| `arg_edges` | edges map<EdgeId, EdgeSummary> | ArgCrate |
| `arg_trust_scores` | trust_scores map<AgentId, f64> | ArgCrate + LLMService |
| `arg_dispatch_overrides` | dispatch_overrides map<SA, Vec<Route>> | SubAgentOrchestrator |
| `arg_achievements_unlocked` | achievements_unlocked map<AgentId, Vec<Unlock>> | SubAgentOrchestrator + LLMService |

---

## 5. PeriodFlushWorker 详细 (C-15)

### 5.1 30s 周期协议 (单线程 + 幂等)

```rust
// crates/arg-bridge/src/flush.rs
use std::time::Duration;
use tokio::time::interval;

pub struct PeriodFlushWorker {
    input_rx: mpsc::Receiver<ArgBridgeEvent>,
    period: Duration,  // 默认 30s
}

impl PeriodFlushWorker {
    pub async fn run(&mut self) -> Result<(), BridgeError> {
        let mut ticker = interval(self.period);
        let mut buffer: Vec<ArgBridgeEvent> = Vec::with_capacity(256);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if !buffer.is_empty() {
                        self.flush_batch(&buffer).await?;
                        buffer.clear();
                    }
                }
                Some(event) = self.input_rx.recv() => {
                    buffer.push(event);
                    if buffer.len() >= 256 {
                        self.flush_batch(&buffer).await?;
                        buffer.clear();
                    }
                }
            }
        }
    }

    async fn flush_batch(&self, events: &[ArgBridgeEvent]) -> Result<(), BridgeError> {
        // 幂等: 同一 event 多次 flush 仅首次生效 (走 Memgraph upsert + LangGraph idempotency_key)
        for event in events {
            self.flush_single(event).await?;
        }
        Ok(())
    }
}
```

### 5.2 幂等保证 (per W/T/M Transaction 表 audit 派生)

- **Memgraph 端**: upsert 走 Edge.id + Agent.id 主键, 重复写无副作用
- **LangGraph 端**: 5 Reducer 走 idempotency_key (per event.id Uuid), 重复 dispatch 仅首次生效
- **失败兜底**: 5min 重试 3 次后转 OfflineQueue (per C-16)

---

## 6. OfflineQueue 详细 (C-16)

### 6.1 sled 嵌入 + 5min 重试

```rust
// crates/arg-bridge/src/offline.rs
use sled::Db;

pub struct OfflineQueue {
    db: Db,
    retry_interval: Duration,  // 默认 5min
}

impl OfflineQueue {
    pub fn new(path: &str) -> Result<Self, BridgeError> {
        let db = sled::open(path)?;
        Ok(Self { db, retry_interval: Duration::from_secs(300) })
    }

    pub async fn persist(&self, event: &ArgBridgeEvent) -> Result<(), BridgeError> {
        let key = event.id().to_string();
        let value = serde_json::to_vec(event)?;
        self.db.insert(key, value)?;
        self.db.flush()?;
        Ok(())
    }

    pub async fn retry_pending(&self) -> Result<usize, BridgeError> {
        let mut retried = 0;
        for result in self.db.iter() {
            let (key, value) = result?;
            let event: ArgBridgeEvent = serde_json::from_slice(&value)?;
            // 重试走 Memgraph + LangGraph
            self.replay_event(&event).await?;
            self.db.remove(&key)?;
            retried += 1;
        }
        Ok(retried)
    }
}
```

### 6.2 跨进程 pub/sub (sled topic)

`OfflineQueue` 走 sled 1.0 embedded DB 嵌入 arg-bridge 进程, 跨进程通过**文件系统共享** (`/var/lib/star-arg-bridge/offline.sled`) 实现 pub/sub, 5 module 跨进程消费.

---

## 7. 跟 LangGraph 9/3 + Agent Runtime 9/3 集成

### 7.1 5 Reducer 跟 LangGraph TopAgentState 集成边界 (per 守门 #3 v2)

| 维度 | LangGraph 9/3 (主) | ARG 9/3 (辅) |
|---|---|---|
| 5 Reducer 状态 | TopAgentState 5 channel (主) | LangGraphStateUpdater (辅, 5 Reducer fanout) |
| 7 节点 TMO | TMO subgraph | 跨 TMO 7 节点 arg_* 共享 |
| 跨进程 | in-process (LangGraph 协议) | Memgraph Bolt subscription (跨进程) |
| 5 module 命中 | (TMO 7 节点) | AgentLease (主) |

### 7.2 跟 Agent Runtime 9/3 集成边界

| 维度 | Agent Runtime 9/3 (主) | ARG 9/3 (辅) |
|---|---|---|
| 9 Archetype | ECS 9 archetype | (跨 ARG Edge weight 字段) |
| L0 / L1 / L2 | L0 全体 + L1 任务卡 + L2 共享池 | (跨 L1 arg_agents channel) |
| 5 module 命中 | (L0 + L1 ECS) | AgentRuntime (REST/WS 部分) |
| 跨进程 | in-process (ECS 协议) | Memgraph + EventBus |

---

## 8. 验证摘要 (Validation)

### 8.1 docs 阶段 (per v0.45 模式)

- 9 docs 落档 ≥ 5000 字总 (本 doc ~3000 字, 9 doc 总 ~22K 字)
- 跨 5 文档严格一致 (per §1.1 5 module 命名 + 9 SA 命名)
- 引用守门完备 (per §1.1 + §2.1 5 协议 + §3-§6 4 子模块 + §7 集成)

### 8.2 Rust 实装 (跨 session 续, per WBS v0.18 估 ~7M / 5-7 session)

- 33 IT 嵌入 (`crates/arg-bridge/src/tests/{listener,updater,flush,offline}_it.rs` 8+8+8+9=33)
- 跨 4 子模块 100% test coverage (跟 ARG.1 100% UT baseline 一致)
- 5 协议 schema 跨 crates/arg + crates/arg-bridge 100% 一致
- Memgraph Bolt subscription 协议跨 ARG.1 + ARG.2 100% 一致

### 8.3 跟 ARG.1 + ARG.4 集成 (per 守门 #1 v3 跨 stage 累积)

- ARG.1 (`c678db7` 5 doc) + ARG.4 (commit `6e2cda6` 14 routes) + ARG.2 (本 doc) 三者 5 module 命名 / 5 Reducer 命名 / 9 SA 命名 / 5 域命名 100% 一致
- 跨 stage token 实测: ARG.1 ~4.5M / ARG.4 ~1.8M / ARG.2 估 ~5-7M

---

## 9. 已知缺口 (per 缺标比错标安全, 守门 #11)

1. **Rust 实装跨 session 续** (per 已知缺口 #1 + 估 ~7M / 5-7 session, 落档后 v0.51+)
2. **Memgraph subscription 协议细节** (per §3.1 Bolt 4.0+ `SubscriptionRun` 实证需要 Memgraph 2.14 启动后跑 33 IT 实证)
3. **sled 1.0 跨进程 pub/sub 协议** (per §6.2 文件系统共享, 实证需要 9 IT 重试 + 跨进程集成, 跨 session 续)
4. **5 Reducer 跟 TMO 7 节点集成** (per §4.1 + §7.1, 跨 session 续做)
5. **5 协议 schema 跟 ARG.3 crates/arg-effect 集成** (per §2.1 协议 1-3 触发源 C-17/C-18, ARG.3 docs 阶段是 v0.51+ 续做项)
6. **5 协议 schema 跟 ARG.10 frontend 集成** (per v0.51+ 续做项)

---

## 10. 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 永久代签 per 守门 #14 v3 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 5 域 Lead 真人到位前临时代签 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |

---

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.2 arg-bridge 5 module ↔ Memgraph + LangGraph 5 Reducer 同步协议 4 子模块 (C-13/C-14/C-15/C-16) + 5 内部协议 schema + 30s 周期 flush + sled 离线降级 + 5 Reducer 集成 + 跨 session 续做边界 | 2026-09-10 01:36 JST 用户发令"继续" + brief v0.50 §14.11 ARG.2-11 docs 阶段派发 + 守门 #14 v3 Mavis 永久代签 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 43 次新事件触发仍允许 |
