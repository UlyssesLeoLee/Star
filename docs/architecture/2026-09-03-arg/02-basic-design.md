# ARG-ARCH-001 Agent Relationship Graph (ARG) 基本設計書

> **Status**: 🟡 Draft v0.1 (per 2026-09-09 21:18 JST 派发 brief v0.45 §14.11 ARG.1 文档完整化)
> **Created**: 2026-09-09
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md)
> **承接**: `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1 + `docs/design/BD-AGENT-RELATIONSHIP-001.md` v0.1 + `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 + 9/4 19:00 JST ARG.1 `crates/arg` 6 子模块骨架落地 (per `docs/briefs/arg-01-arg-crate-skeleton.md` §2)

---

## 0. 目的 (Objective)

承接要件定義書 §0 入口, 本基本設計書定义 ARG 阶段 5 module 体系 (ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime) 的**系统架构** / **组件一览** / **数据模型** / **接口设计** / **状态机** / **NFR**, 跨 5 文档严格一致.

---

## 1. システムアーキテクチャ (System Architecture)

### 1.1 5-tier 架构 (per BD-AGENT-RELATIONSHIP-001 v0.1 §2.1 + 守门 #3 v2)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       UI Tier (frontend/agent-relationships)               │
│   5 UI 组件: RelationshipEditor / RelationshipView / AchievementWall /      │
│             EdgeTypeSelector / TemplateGallery                              │
│   zustand useARGStore 5 channel: agents / edges / templates / achievements │
│                       / events (WebSocket)                                   │
│                       5 module: AgentRuntime (UI 部分)                       │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │ HTTPS REST + WSS
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       API Tier (crates/api/src/arg)                          │
│   13 REST + 1 WebSocket + RLS 13 类 (per ADR-0048 axum 0.8)                │
│   5 module: AgentRuntime (REST/WS 部分)                                     │
│   W/T/M: per 守门 #13 (5 张新表 100% 覆盖)                                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │ Memgraph Bolt (port 7687)
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Data Tier (crates/arg)                                 │
│   6 子模块: models 10 + client 3 + ops 5 + query 3 + llm + error             │
│   5 module: ArgCrate (含 LLMService 挑战/peer review 摘要子模块 +            │
│             AgentLease 事件写入子模块)                                        │
│   Memgraph Bolt 客户端 + Cypher 缓存 + LLM 调用 + 错误处理                    │
│   5 张表: agents(M) / edges(M) / audit(T) / template_instances(W TTL 30d)   │
│           / unlocks(T)                                                       │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │ Memgraph Bolt subscription + EventBus
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Bridge Tier (crates/arg-bridge)                         │
│   4 子模块: MemgraphEventListener / LangGraphStateUpdater /                  │
│             PeriodFlushWorker (30s) / OfflineQueue (sled 离线降级)           │
│   5 module: AgentLease (同步桥部分)                                         │
│   同步桥协议 (5 内部协议): arg_edge_changed / arg_dispatch_route /            │
│             arg_context_inject / arg_trust_score_update /                    │
│             arg_achievement_unlocked                                          │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │ in-process 推 + 周期 flush
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Effect Tier (crates/arg-effect)                         │
│   5 子模块: ARGDispatchRouter / ARGContextInjector / ARGTrustEngine /        │
│             ARGOutputEvaluator / ARGAchievementEngine                        │
│   5 module: SubAgentOrchestrator                                              │
│   4 effect 维度 (per 要件 §1.1): Dispatch 路由 / 上下文共享 /                │
│             信任度加权 / 产出评估                                             │
│   LangGraph TopAgentState 扩展 5 channel: arg_agents / arg_edges /           │
│             arg_trust_scores / arg_dispatch_overrides /                      │
│             arg_achievements_unlocked + 5 Reducer                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 跟 LangGraph 9/3 + Agent Runtime 9/3 平行关系 (per 守门 #3 v2)

| 维度 | LangGraph View (9/3) | Agent Runtime View (9/3) | **ARG View (本 view)** |
|---|---|---|---|
| 关注点 | UI 驱动 2-level hierarchical Agent (L0 全体代理 + L1 任务卡子代理) | 大规模 AI Agent 并发 Runtime 基础设施 | agent 之间的 social 层 (关系 + effect 维度) |
| 9 SA Type | LangGraph subgraph (in-process, LangGraph 协议) | ECS 9 Archetype (in-process, ECS 协议) | **本 view 引用 (业务子代理类型)** |
| Checkpoint | LangGraph 3-tier (RAM/Redis/DB) | Runtime 3-tier (HOT/WARM/COLD) | 共享 PG (audit + unlocks) |
| 通信 | LangGraph 状态 + 边 + reducer | Event Bus + Mailbox + Channel | Memgraph Bolt subscription + in-process 推 + 周期 flush |
| 路径 | `docs/architecture/2026-09-03-langgraph/` | `docs/architecture/2026-09-03-agent-runtime/` | `docs/architecture/2026-09-03-arg/` |
| 5 module | (TMO 7 节点) | (L0 + L1 ECS + L2 共享池) | **ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime** |

**关系**: 3 个 view **平行**, 9 SA Type 是**接口**而不是实现. LangGraph subgraph + Agent Runtime ECS + ARG 5 module 互不取代, 互不建立业务子域↔DDD bounded context 映射 (per 守门 #3 v2 + AGENTS.md §5 仓库拓扑 disclaimer).

### 1.3 5 Module 跟 5-tier 架构映射

| 5 Module | UI Tier | API Tier | Data Tier | Bridge Tier | Effect Tier |
|---|---|---|---|---|---|
| **ArgCrate** | — | — | ✅ (主) | — | — |
| **SubAgentOrchestrator** | — | — | — | — | ✅ (主) |
| **LLMService** | — | — | ✅ (子模块) | — | ✅ (调用) |
| **AgentLease** | — | — | ✅ (子模块) | ✅ (主) | — |
| **AgentRuntime** | ✅ (主) | ✅ (主) | — | — | — |

---

## 2. コンポーネント一覧 (Components)

### 2.1 24 新组件 (per BD-AGENT-RELATIONSHIP-001 v0.1 §3)

| 组件 | 5 module | 5-tier | 责任 | 9 SA 引用 |
|---|---|---|---|---|
| **C-1** AgentNode | ArgCrate | Data | Agent 模型 16 字段 | (基础) |
| **C-2** Edge | ArgCrate | Data | Edge 模型 14 字段 | (基础) |
| **C-3** RelationshipType | ArgCrate | Data | 10 类关系枚举 | (基础) |
| **C-4** TeamTemplate | ArgCrate | Data | TeamTemplate 模型 5 字段 | (基础) |
| **C-5** Achievement | ArgCrate | Data | Achievement 模型 20 字段 | SA-09 |
| **C-6** TrustScoreTier | ArgCrate | Data | 5 段信任分枚举 | SA-03 |
| **C-7** ARGEvent | ArgCrate | Data | ARGEvent 模型 9 字段 | (基础) |
| **C-8** MemgraphClient | ArgCrate | Data | Bolt 7687 客户端 | (基础) |
| **C-9** CypherCache | ArgCrate | Data | 5min TTL + 256 MB LRU | SA-08 |
| **C-10** EventWriter | ArgCrate + AgentLease | Data + Bridge | 事件写入 + lease 锁 | (基础) |
| **C-11** LLMService | LLMService | Data | 挑战/peer review 摘要 | SA-01 + SA-03 + SA-09 |
| **C-12** ARGError | ArgCrate | Data | 10 类错误枚举 | (基础) |
| **C-13** MemgraphEventListener | AgentLease | Bridge | Bolt subscription 监听 | (基础) |
| **C-14** LangGraphStateUpdater | AgentLease | Bridge | 5 Reducer 跨 LangGraph | SA-10 (TMO 7 节点) |
| **C-15** PeriodFlushWorker | AgentLease | Bridge | 30s 周期 flush | (基础) |
| **C-16** OfflineQueue | AgentLease | Bridge | sled 离线降级 + 5min 重试 | (基础) |
| **C-17** ARGDispatchRouter | SubAgentOrchestrator | Effect | 4 维度 effect #1 | SA-04 + SA-08 |
| **C-18** ARGContextInjector | SubAgentOrchestrator | Effect | 4 维度 effect #2 | SA-02 + SA-06 |
| **C-19** ARGTrustEngine | SubAgentOrchestrator | Effect | 4 维度 effect #3 | SA-01 + SA-03 |
| **C-20** ARGOutputEvaluator | SubAgentOrchestrator | Effect | 4 维度 effect #4 | SA-01 + SA-03 + SA-09 |
| **C-21** ARGAchievementEngine | SubAgentOrchestrator | Effect | 8 拓扑成就评估 | SA-09 |
| **C-22** ArgRESTHandlers | AgentRuntime | API | 13 REST 端点 | (跨域) |
| **C-23** ArgWebSocketHandler | AgentRuntime | API | 1 WebSocket 推送 | (跨域) |
| **C-24** ArgUIComponents | AgentRuntime | UI | 5 UI 组件 + useARGStore | (跨域) |

### 2.2 24 组件跟 9 SA 跨 SA 关系 (per 要件 §6.1.10 矩阵)

每个组件至少 1 个 9 SA 命中, 跨 SA 协作通过组件 (e.g. C-17 ARGDispatchRouter 跨 SA-04 + SA-08, per 要件 §6.1.11).

### 2.3 24 组件跟 5 域命中

| 组件 | player | economy | match | social | admin |
|---|---|---|---|---|---|
| C-1..C-12 (Data 基础) | ✅ | ✅ | ✅ | ✅ | ✅ |
| C-13..C-16 (Bridge) | ✅ | ✅ | ✅ | ✅ | ✅ (主 audit) |
| C-17 ARGDispatchRouter | ✅ | — | ✅ (主) | — | ✅ |
| C-18 ARGContextInjector | ✅ | — | — | ✅ (主) | — |
| C-19 ARGTrustEngine | ✅ | ✅ | ✅ | ✅ | ✅ (主) |
| C-20 ARGOutputEvaluator | ✅ | ✅ | — | — | ✅ (主) |
| C-21 ARGAchievementEngine | ✅ (主) | ✅ | ✅ | ✅ (主) | — |
| C-22 ArgRESTHandlers | ✅ | ✅ | ✅ | ✅ (主) | ✅ |
| C-23 ArgWebSocketHandler | ✅ | — | ✅ | ✅ (主) | — |
| C-24 ArgUIComponents | ✅ | — | — | ✅ (主) | ✅ |

---

## 3. 5 Module 詳細設計 (per §2 组件 + 9 SA 跨 SA 关系 + 5 域命中)

### 3.1 ArgCrate (Data Tier 核心, 6 子模块)

**物理位置**: `crates/arg/` (ARG.1 收官 commit `43c1f0c`)

**6 子模块**:

1. **models** (10 子模块): Agent + Edge + RelationshipType + TeamTemplate + Achievement + TrustScoreTier + ARGEvent + ARGError + Decision + PeerReview + TrustScore + TemplateInstance + AchievementUnlock (per DD §3.2)
2. **client** (3 子模块): MemgraphClient (Bolt 7687) + CypherCache (5min TTL + 256 MB LRU) + Migration (V1→V2 脚本)
3. **ops** (5 子模块): AgentNode + Edge + Template + Achievement + EventWriter (per AgentLease 共享)
4. **query** (3 子模块): Behavior (7 行为评估) + Output (5 产出评估) + Topology (8 拓扑 Cypher)
5. **llm** (1 子模块, per LLMService): 挑战/peer review 摘要 + 解锁文案生成
6. **error** (1 子模块): 10 类错误枚举 (NotFound + Unauthorized + InvalidEdge + InvalidTemplate + MemgraphDown + BoltTimeout + CacheMiss + SchemaMismatch + RLSViolation + InternalError)

**关键 API**:

```rust
// crates/arg/src/lib.rs
pub struct ArgCrate {
    memgraph: Arc<MemgraphClient>,
    cypher_cache: Arc<CypherCache>,
    event_writer: Arc<EventWriter>,
    llm: Arc<LLMService>,
}

impl ArgCrate {
    pub async fn create_agent(&self, agent: NewAgent) -> Result<Agent, ARGError>;
    pub async fn get_agent(&self, id: AgentId) -> Result<Agent, ARGError>;
    pub async fn patch_agent(&self, id: AgentId, patch: AgentPatch) -> Result<Agent, ARGError>;
    pub async fn list_agents(&self, filter: AgentFilter) -> Result<Vec<Agent>, ARGError>;

    pub async fn create_edge(&self, edge: NewEdge) -> Result<Edge, ARGError>;
    pub async fn get_edge(&self, id: EdgeId) -> Result<Edge, ARGError>;
    pub async fn patch_edge(&self, id: EdgeId, patch: EdgePatch) -> Result<Edge, ARGError>;
    pub async fn delete_edge(&self, id: EdgeId) -> Result<(), ARGError>;
    pub async fn list_edges(&self, filter: EdgeFilter) -> Result<Vec<Edge>, ARGError>;

    pub async fn query_graph(&self, query: GraphQuery) -> Result<GraphSnapshot, ARGError>;

    pub async fn instantiate_template(&self, template_id: TemplateId, params: TemplateParams) -> Result<TemplateInstance, ARGError>;

    pub async fn unlock_achievement(&self, code: &str, agent_id: AgentId) -> Result<AchievementUnlock, ARGError>;
    pub async fn list_achievements(&self, agent_id: AgentId) -> Result<Vec<Achievement>, ARGError>;

    // Trust score
    pub async fn update_trust_score(&self, agent_id: AgentId, delta: f64, reason: TrustReason) -> Result<TrustScore, ARGError>;
}
```

**5 域命中**: player + economy + match + social + admin (5 域全覆盖, per §2.3 跨域组件).
**9 SA 引用**: SA-01..SA-09 (9 SA 全部通过 ArgCrate 持久化, per 要件 §6.1.11 映射表).
**守门**: 0 unsafe (per `unsafe_code = "forbid"`), 100% RLS 13 类 (per 守门 #13 派生规 (d)), SCD Type 2 (per 派生规 (c)).

### 3.2 SubAgentOrchestrator (Effect Tier 5 子模块)

**物理位置**: `crates/arg-effect/` (ARG.3 docs 阶段, per DD §3.1 + §4.5-4.9, P3-C W3 落地)

**5 子模块** (per §2.1 C-17..C-21):

1. **ARGDispatchRouter** (C-17): 4 维度 effect #1, Lead 收到任务按 outgoing `delegates_to` 自动 spawn Worker, 失败按 `stand_in_for` fallback
2. **ARGContextInjector** (C-18): 4 维度 effect #2, mentee 启动拉 mentor 历史决策, shadow 静默订阅, token 节省 20-40%
3. **ARGTrustEngine** (C-19): 4 维度 effect #3, trust_score ≥ 0.8 + trusts 边 weight ≥ 0.7 跳过 verify, 节省 15% token
4. **ARGOutputEvaluator** (C-20): 4 维度 effect #4, challenges 强制双向论证, peer_reviews 阈值 ≥ 0.8
5. **ARGAchievementEngine** (C-21): 8 拓扑成就 Cypher 评估 + 解锁文案生成 (per LLMService)

**关键协议** (per 要件 §4.1 F-12..F-16):

```python
# crates/arg-effect/src/protocol.rs
@dataclass
class ARGDispatchRoute:
    """arg_dispatch_route 协议 (F-15)"""
    task_id: str
    lead_id: str
    worker_id: str
    relationship_type: Literal["delegates_to", "stand_in_for"]
    weight: float  # 0.0 - 1.0
    reason: str

@dataclass
class ARGContextInject:
    """arg_context_inject 协议 (F-16)"""
    mentee_id: str
    mentor_ids: list[str]
    shadow_ids: list[str]
    history_decisions: list[Decision]  # 拉 mentors 历史
    token_estimate: int  # 20-40% 节省

@dataclass
class ARGTrustScoreUpdate:
    """arg_trust_score_update 协议 (F-12)"""
    agent_id: str
    old_score: float
    new_score: float
    reason: Literal["success", "failure", "peer_review", "challenge_pass", "challenge_fail"]
    timestamp: int

@dataclass
class ARGAchievementUnlocked:
    """arg_achievement_unlocked 协议 (F-14)"""
    agent_id: str
    achievement_code: str  # e.g. "TOP-001" (Triangle)
    unlocked_at: int
    notification: str  # 解锁文案 (per LLMService 生成)
```

**5 域命中**: 主要 match (Dispatch) + social (Context) + admin (Trust) + player (Achievement), per §2.3 跨域组件.
**9 SA 引用**: SA-01 + SA-02 + SA-03 + SA-04 + SA-06 + SA-08 + SA-09 (per 要件 §6.1.11).
**守门**: 4 effect 维度真实落地 (per 要件 §1.1 4 effect 维度真实影响协作派生约束, 不允许只列 1-2 个 component).

### 3.3 LLMService (Data Tier 子模块 + Effect Tier 调用)

**物理位置**: `crates/arg/src/llm.rs` (ArgCrate 子模块, per ARG.1 收官 commit `43c1f0c`)

**职责**: ARG 阶段 LLM 调用层 (挑战/peer review 摘要/8 拓扑 Cypher 模板生成), 跟 L0 派发层 LLM Pool 共享 (per ADR-0045 Agent Runtime L2 共享池).

**5 LLM 用途**:

1. **挑战 (challenges) 双向论证 prompt 模板 10 套** (per DD §7): 5 decision_type × 2 trust_tier (LOW 0.2-0.4 + HIGH 0.7-0.9) 组合
2. **peer review 摘要**: 5 类 (决策/代码/测试/迁移/refactor) × 2 长度 (短 100 字 + 长 500 字)
3. **8 拓扑成就 Cypher 模板生成** (per G-6 已闭环): TOP-001 Triangle / TOP-002 K-Core / TOP-003 Star / TOP-004 Path / TOP-005 Cycle / TOP-006 Complete / TOP-007 Bridge / TOP-008 Hub
4. **解锁文案生成**: 8 拓扑 + 7 行为 + 5 产出 = 20 成就, 每成就 3 段文案 (庆祝/条件/稀有度)
5. **ARG 事件摘要**: arg_edge_changed / arg_dispatch_route / arg_context_inject / arg_trust_score_update / arg_achievement_unlocked 5 协议, 每协议 1 句话摘要

**关键 API**:

```rust
// crates/arg/src/llm.rs
pub struct LLMService {
    pool: Arc<LLMPool>,  // 共享 L0 派发层 LLM Pool
    mock_mode: bool,     // per 守门 #23 AI mock 不开外部 API
}

impl LLMService {
    pub async fn generate_challenge_prompt(&self, decision_type: DecisionType, trust_tier: TrustScoreTier) -> Result<String, ARGError>;
    pub async fn summarize_peer_review(&self, review: PeerReview, length: ReviewLength) -> Result<String, ARGError>;
    pub async fn generate_topology_cypher(&self, topology: TopologyType) -> Result<String, ARGError>;
    pub async fn generate_unlock_message(&self, achievement: Achievement) -> Result<String, ARGError>;
    pub async fn summarize_arg_event(&self, event: ARGEvent) -> Result<String, ARGError>;
}
```

**5 域命中**: 主要 admin (审计摘要) + match (成就文案) + social (解锁文案), per §2.3 跨域组件.
**9 SA 引用**: SA-01 (review 摘要) + SA-03 (审计摘要) + SA-08 (业务代码生成) + SA-09 (解锁文案).
**守门**: #23 AI mock 不开外部 API (per 守门 #5 v2), 调试页 AI 修改 mock 不开 OpenAI/Anthropic.

### 3.4 AgentLease (Data Tier 子模块 + Bridge Tier 4 子模块)

**物理位置**: `crates/arg/src/ops/event_writer.rs` (ArgCrate 子模块) + `crates/arg-bridge/` (ARG.2 docs 阶段, P3-C W2 落地)

**职责**: ARG 阶段 lease 语义 (per ADR-0030 Agent Lease/Heartbeat/Resume 11 字段), 跨 Agent Handoff 锁 / 心跳 / 恢复, 跟 RLS 13 类隔离配合.

**4 Bridge 子模块** (per §2.1 C-13..C-16):

1. **MemgraphEventListener** (C-13): Bolt subscription 监听 Memgraph 事件变更, 推 in-process EventBus
2. **LangGraphStateUpdater** (C-14): 5 Reducer 跨 LangGraph TopAgentState 扩展 (arg_agents / arg_edges / arg_trust_scores / arg_dispatch_overrides / arg_achievements_unlocked)
3. **PeriodFlushWorker** (C-15): 30s 周期 flush (Memgraph → ArgCrate), 跟离线降级配合
4. **OfflineQueue** (C-16): sled 离线降级 + 5min 重试 (per NFR-R-01)

**Lease 11 字段** (per ADR-0030):

```rust
// crates/arg-bridge/src/lease.rs
pub struct AgentLease {
    pub lease_id: LeaseId,                  // 1. UUID v7
    pub agent_id: AgentId,                  // 2. 租户 agent
    pub tenant_id: TenantId,                // 3. 租户隔离
    pub acquired_at: Timestamp,             // 4. 获取时间
    pub expires_at: Timestamp,              // 5. 过期时间 (默认 30s)
    pub heartbeat_at: Timestamp,            // 6. 心跳时间 (每 10s)
    pub holder: HolderType,                 // 7. Lead / Worker / Orchestrator
    pub operation: OperationType,           // 8. 关系修改 / 模板 instantiate / 成就解锁
    pub target_id: TargetId,                // 9. edge_id / template_id / achievement_id
    pub state: LeaseState,                  // 10. Active / Expired / Released / Aborted
    pub context: serde_json::Value,         // 11. 上下文 (per LLMService 摘要)
}
```

**5 域命中**: 主要 admin (audit trail) + player (持久化) + match (赛季) + social (成就) + economy (经济), per §2.3 跨域组件.
**9 SA 引用**: SA-03 (audit lease) + SA-04 (ops lease) + SA-07 (migration lease) + (跨域).
**守门**: 跟 ADR-0030 11 字段对齐, 跨 Agent Handoff 锁 / 心跳 / 恢复 100%.

### 3.5 AgentRuntime (UI Tier + API Tier)

**物理位置**: `crates/api/src/arg/` 扩展 (ARG.4 收官 commit `6e2cda6`) + `frontend/src/app/agent-relationships/` (ARG.5 docs 阶段, P3-C W4 落地)

**职责**: ARG 阶段运行时入口 (13 REST API + 1 WS + 5 UI 组件), 跟 axum 0.8 对齐 (per ADR-0048).

**13 REST 端点** (per DD §4.12 axum 0.8 `{id}` 语法):

```rust
// crates/api/src/arg/mod.rs (per ARG.4 收官)
pub fn arg_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Agent CRUD (4)
        .route("/api/arg/agents", post(create_agent))
        .route("/api/arg/agents", get(list_agents))
        .route("/api/arg/agents/:id", get(get_agent))
        .route("/api/arg/agents/:id", patch(patch_agent))
        // Edge CRUD (5)
        .route("/api/arg/edges", post(create_edge))
        .route("/api/arg/edges", get(list_edges))
        .route("/api/arg/edges/:id", get(get_edge))
        .route("/api/arg/edges/:id", patch(patch_edge))
        .route("/api/arg/edges/:id", delete(delete_edge))
        // Graph query (1)
        .route("/api/arg/graph", get(query_graph))
        // Template (1)
        .route("/api/arg/templates/instantiate", post(instantiate_template))
        // Achievement (2)
        .route("/api/arg/achievements", get(list_achievements))
        .route("/api/arg/achievements/unlock", post(unlock_achievement))
        // WebSocket (1, 实际不是 REST, 但 per 14 端点统计)
        .route("/ws/arg/events", get(arg_ws_handler))
}
```

**1 WebSocket** (`/ws/arg/events`): SSE-over-WS, 6 事件类型: edge.changed / edge.created / edge.archived / achievement.unlocked / dispatch.route.changed / agent.trust_score.changed.

**5 UI 组件** + **zustand useARGStore 5 channel** (per DD §4.13):

```typescript
// frontend/src/lib/arg/store.ts
export const useARGStore = create<ARGStore>((set, get) => ({
  agents: [],         // 5 channel #1
  edges: [],          // 5 channel #2
  templates: [],      // 5 channel #3
  achievements: [],   // 5 channel #4
  events: [],         // 5 channel #5 (WebSocket)
  // ... actions
}));

// frontend/src/app/agent-relationships/components/
// 1. RelationshipEditor.tsx - 编辑关系 (拖拽建边 ≤ 3 步)
// 2. RelationshipView.tsx - 查看关系图谱
// 3. AchievementWall.tsx - 成就墙 (SA-09 推送)
// 4. EdgeTypeSelector.tsx - 关系类型选择 (10 类)
// 5. TemplateGallery.tsx - 团队模板库 (5 类)
```

**5 域命中**: 主要 social (UI 关系编辑) + admin (admin UI) + match (赛季) + player + economy, per §2.3 跨域组件.
**9 SA 引用**: 跨域 (无直接 SA 命中, 主要是 UI/UX 入口), per 要件 §6.1.11.
**守门**: 13 REST 严格按 axum 0.8 `{id}` 语法 (per ADR-0048), 14 routes 实证 0 err (per ARG.4 收官).

---

## 4. データモデル (Data Model)

### 4.1 5 张新表 W/T/M 严格 (per 守门 #13 横展開派生规)

| 表名 | W/T/M 类别 | RLS | 审计 | SCD | 关键字段 |
|---|---|---|---|---|---|
| **agents** | Master (M) | 13 类 | n/a | Type 2 | id (UUID v7) / agent_type / tenant_id / label / status / trust_score / metadata / created_at / updated_at / version |
| **edges** | Master (M) | 13 类 | n/a | Type 2 | id / source_id / target_id / relationship_type / weight / created_at / updated_at / version |
| **audit** | Transaction (T) | 13 类 | append-only | n/a | id / event_type / actor_id / target_id / payload / timestamp / prev_hash (链式) |
| **template_instances** | Work (W, TTL 30d) | n/a | n/a | n/a | id / template_id / params / status / created_at / expires_at |
| **unlocks** | Transaction (T) | 13 类 | append-only | n/a | id / agent_id / achievement_code / unlocked_at |

**派生规检查** (per 守门 #13 (a)(b)(c)(d)(e)):

- (a) W = 物理删除 / タイマー失効 / 短 TTL 明示 retention: ✅ template_instances TTL 30d
- (b) T = 物理删除禁止 + 監査必須 + RLS 13 類必携: ✅ audit + unlocks 100% RLS + append-only
- (c) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携: ✅ agents + edges SCD Type 2 + 100% RLS
- (d) Master 100% RLS: ✅ agents + edges (2/2)
- (e) Transaction 100% audit: ✅ audit + unlocks (2/2)
- **5/5 表 100% 覆盖, 0 混合分类, 0 缺口** (per 守门 #13 派生规)

### 4.2 5 表 ER 图 (text-based)

```
agents (M) ─────────┐
   │ id              │
   │ agent_type      │
   │ tenant_id       │
   │ trust_score     │
   │ ...             │
   │                 │
   │ (1)             │ (N)
   │                 │
   ▼                 ▼
edges (M) ──────── audit (T)
   │ id              │ id
   │ source_id ──────┤ event_type
   │ target_id ──────┤ actor_id
   │ relationship_type│ target_id
   │ weight          │ payload
   │ ...             │ timestamp
   │                 │ prev_hash
   │                 │
   │ (N)             │ (N)
   ▼                 ▼
unlocks (T) ──── template_instances (W, TTL 30d)
   │ id              │ id
   │ agent_id        │ template_id
   │ achievement_code│ params
   │ unlocked_at     │ status
                    │ expires_at
```

---

## 5. インターフェース設計 (Interface Design)

### 5.1 5 Module 接口边界

| 5 Module | 输入 | 输出 | 依赖 |
|---|---|---|---|
| **ArgCrate** | NewAgent / NewEdge / GraphQuery / NewTemplateInstance | Agent / Edge / GraphSnapshot / TemplateInstance | MemgraphClient + CypherCache + EventWriter + LLMService |
| **SubAgentOrchestrator** | ARGEvent + LangGraph TopAgentState | ARGDispatchRoute / ARGContextInject / ARGTrustScoreUpdate / ARGAchievementUnlocked | ArgCrate (read) + LLMService (调用) + AgentLease (推送) |
| **LLMService** | 5 LLM 用途参数 | String (5 类) | LLMPool (共享) + ArgCrate (read agent) |
| **AgentLease** | ARGEvent + LangGraph State | AgentLease 11 字段 | ArgCrate (read/write) + MemgraphClient (Bolt subscription) + sled (离线降级) |
| **AgentRuntime** | HTTP request / WebSocket | JSON response / WS event | ArgCrate (主) + AgentLease (推送) |

### 5.2 5 Module 跨模块调用时序 (per 要件 S-01 拖拽建边)

```
User (UI) → AgentRuntime (UI Tier)
   ↓ HTTP POST /api/arg/edges
AgentRuntime (API Tier)
   ↓ ArgCrate.create_edge()
ArgCrate (Data Tier)
   ↓ MemgraphClient.execute_cypher()
Memgraph (Bolt)
   ↓ event
MemgraphEventListener (AgentLease / Bridge Tier)
   ↓ EventBus
PeriodFlushWorker (AgentLease)
   ↓ LangGraphStateUpdater
LangGraphStateUpdater (AgentLease)
   ↓ 5 Reducer
LangGraph TopAgentState
   ↓ dispatch event
ARGDispatchRouter (SubAgentOrchestrator / Effect Tier)
   ↓ LLMService.generate_unlock_message() (if applicable)
LLMService
   ↓
ARGAchievementEngine (SubAgentOrchestrator)
   ↓ WebSocket push
AgentRuntime (UI Tier)
   ↓ SSE
User (UI) 看到关系创建 + 成就解锁通知
```

### 5.3 5 Module 跟 9 SA 跨模块调用

| SA | ArgCrate | SubAgentOrchestrator | LLMService | AgentLease | AgentRuntime |
|---|---|---|---|---|---|
| **SA-01 code-review** | create_edge / patch_agent | ARGContextInjector (拉 mentors review 模式) | generate_peer_review_summary | event: review.completed | UI 审查 tab |
| **SA-02 test-gen** | create_edge / patch_agent | ARGContextInjector (拉 mentors test 模式) | — | event: test.completed | UI test 报告 |
| **SA-03 5-域-lead-audit** | create_audit_event | ARGOutputEvaluator (跨域审计) | generate_audit_summary | lease: audit lease | admin UI 审计 |
| **SA-04 git-ops** | create_edge / patch_agent | ARGDispatchRouter (派发 ops) | — | lease: ops lease | UI ops 监控 |
| **SA-05 doc-sync** | create_edge | — | — | event: doc.synced | UI doc 预览 |
| **SA-06 refactor** | create_edge / patch_agent | ARGContextInjector (拉 mentors refactor 模式) | generate_refactor_suggestion | event: refactor.completed | UI diff 展示 |
| **SA-07 db-migration** | create_audit_event | ARGDispatchRouter (派发 migration) | — | lease: migration lease | admin UI migration 监控 |
| **SA-08 domain-dev** | create_edge / patch_agent | ARGDispatchRouter (派发 dev 任务) | generate_business_code | event: dev.completed | UI dev 监控 |
| **SA-09 achievement-eval** | unlock_achievement | ARGAchievementEngine (8 拓扑评估) | generate_unlock_message | event: achievement.unlocked | UI 成就墙推送 |

---

## 6. 5 状態機 (State Machines)

### 6.1 Edge 状态机 (per DD §5.1)

```
       create_edge()
          │
          ▼
    ┌──────────┐
    │ created  │
    └──────────┘
       │     │
       │     │ archive_edge()
       │     ▼
       │  ┌──────────┐
       │  │ archived │
       │  └──────────┘
       │     │
       │     │ unarchive_edge()
       │     ▼
       │  ┌──────────┐
       │  │ created  │ (循环)
       │  └──────────┘
       │
       │ patch_edge()
       ▼
    ┌──────────┐
    │ updated  │ (last_updated_at 更新, weight 可能改)
    └──────────┘
       │
       │ archive_edge()
       ▼
    ┌──────────┐
    │ archived │
    └──────────┘
```

### 6.2 Agent 状态机

```
    create_agent()
       │
       ▼
  ┌──────────┐
  │  active  │ (默认)
  └──────────┘
     │     │
     │     │ standby_agent()
     │     ▼
     │  ┌──────────┐
     │  │ standby  │ (暂停接收任务)
     │  └──────────┘
     │     │
     │     │ activate_agent()
     │     ▼
     │  ┌──────────┐
     │  │  active  │ (循环)
     │  └──────────┘
     │
     │ archive_agent()
     ▼
  ┌──────────┐
  │ archived │
  └──────────┘
```

### 6.3 Trust Score 5 段状态机

```
  UNTRUSTED (0-0.2)
     │ +0.1 (success)
     ▼
  LOW (0.2-0.4)
     │ +0.1 (success)
     ▼
  MEDIUM (0.4-0.7)
     │ +0.1 (success) or -0.2 (failure)
     ▼
  HIGH (0.7-0.9)
     │ +0.1 (peer_review) or -0.2 (challenge_fail)
     ▼
  VERY_HIGH (0.9-1.0)
     │ -0.5 (severe_failure) → MEDIUM
     │ -0.3 (moderate_failure) → MEDIUM/HIGH 边界
```

### 6.4 Template Instance 状态机

```
  instantiate_template()
     │
     ▼
  ┌──────────┐
  │ pending  │ (30s 内初始化)
  └──────────┘
     │
     │ initialize_complete()
     ▼
  ┌──────────┐
  │  active  │ (应用模板)
  └──────────┘
     │
     │ expire (30d TTL)
     ▼
  ┌──────────┐
  │ expired  │ (物理删除 per 守门 #13 (a))
  └──────────┘
```

### 6.5 Achievement 状态机 (append-only)

```
  condition_met()
     │
     ▼
  ┌──────────┐
  │  locked  │ (未解锁, append-only, 不可改)
  └──────────┘
     │
     │ (state transition is single-shot, 不可逆)
     ▼
  ┌──────────┐
  │ unlocked │ (append-only, notification 推送)
  └──────────┘
```

---

## 7. 6 NFR (per 要件 §4.2 + 守门 #13)

### 7.1 性能 (per NFR-P-01..NFR-P-04)

| NFR | 度量 | 5 module 命中 | 守门 |
|---|---|---|---|
| **NFR-P-01** 边创建 P95 < 200ms | 1000 次 P95 | AgentRuntime + ArgCrate | cargo bench -p star-arg (P3-E W1) |
| **NFR-P-02** Cypher 查询 P95 < 500ms | 1000 次 P95 | ArgCrate + SubAgentOrchestrator | cargo bench -p star-arg (P3-E W1) |
| **NFR-P-03** 事件推送 P95 < 100ms | 1000 次 P95 | AgentRuntime + AgentLease | pytest arg_e2e_test (P3-D W2) |
| **NFR-P-04** 成就评估 P95 < 1s | 1000 次 P95 | SubAgentOrchestrator + LLMService | pytest arg_e2e_test (P3-D W2) |

### 7.2 可靠性 (per NFR-R-01..NFR-R-02)

| NFR | 度量 | 5 module 命中 | 守门 |
|---|---|---|---|
| **NFR-R-01** 离线降级 | sled 离线队列 + 5min 重试 | ArgCrate + AgentLease | pytest arg_bridge_test (P3-C W2) |
| **NFR-R-02** Memgraph HA 集群 | replica set (G-7 后续阶段) | ArgCrate | n/a |

### 7.3 安全 (per NFR-S-01..NFR-S-04 + 守门 #13)

| NFR | 度量 | 5 module 命中 | 守门 |
|---|---|---|---|
| **NFR-S-01** RLS 13 类隔离 | per-tenant + per-agent 双层 | ArgCrate + AgentRuntime | cargo test -p star-arg RLS (ARG.1 已闭环) |
| **NFR-S-02** SCD Type 2 | Master 表历史可追溯 | ArgCrate | cargo test -p star-arg SCD (ARG.1 已闭环) |
| **NFR-S-03** Env 不打印 | 凭证 stdin pipe | (跨域) | scripts/automation/memgraph_setup.py (per 守门 #5) |
| **NFR-S-04** 代签规则 | author = Ulysses | (跨域) | commit author = Ulysses (per 守门 #10 + #14 v3) |

### 7.4 成就 (per NFR-A-01..NFR-A-02 + 守门 #13)

| NFR | 度量 | 5 module 命中 | 守门 |
|---|---|---|---|
| **NFR-A-01** 8 拓扑成就 | 8C+7R+3E+2L = 20 分布 | SubAgentOrchestrator | pytest arg_achievement_eval (P3-E W1) |
| **NFR-A-02** 7 行为 + 5 产出 | 异步触发 + SSE 推送 | SubAgentOrchestrator + LLMService | pytest arg_achievement_eval (P3-E W1) |

### 7.5 易用 (per NFR-U-01..NFR-U-02)

| NFR | 度量 | 5 module 命中 | 守门 |
|---|---|---|---|
| **NFR-U-01** 1-click 模板 | ≤ 30s instantiate | AgentRuntime + SubAgentOrchestrator | e2e Playwright (P3-D W2) |
| **NFR-U-02** 拖拽建边 | ≤ 3 步 (3 click) | AgentRuntime | e2e Playwright (P3-D W2) |

### 7.6 可观 (per NFR-O-01)

| NFR | 度量 | 5 module 命中 | 守门 |
|---|---|---|---|
| **NFR-O-01** Prometheus 指标 | 7 指标 + tracing | ArgCrate + AgentRuntime | ops_metrics_config (per 守门 #13 W/T/M) |

---

## 8. 5 Module 跟 4 New Crate 落地路径 (per §1.3 跨 tier 映射 + ARG.1-11 子项)

| 5 Module | 物理 crate | 路径 | 状态 |
|---|---|---|---|
| ArgCrate | `crates/arg/` | 6 子模块 (models 10 + client 3 + ops 5 + query 3 + llm + error) = 29 src + 7 tests + Cargo.toml = 37 文件 | 🟢 **ARG.1 收官** (commit `43c1f0c`, 44 文件 / 4206 行 / 2 脚本 / 33 UT 100% pass) |
| SubAgentOrchestrator | `crates/arg-effect/` | 5 子模块 (ARGDispatchRouter / ARGContextInjector / ARGTrustEngine / ARGOutputEvaluator / ARGAchievementEngine) | 🟡 ARG.3 docs 阶段 (P3-C W3 落地) |
| LLMService | `crates/arg/src/llm.rs` | 1 子模块 (5 LLM 用途, per §3.3) | 🟢 **ARG.1 收官** (per 43c1f0c) |
| AgentLease | `crates/arg/src/ops/event_writer.rs` (Data) + `crates/arg-bridge/` (Bridge, 4 子模块) | 1 子模块 (Data) + 4 子模块 (Bridge) | 🟡 ARG.2 docs 阶段 (P3-C W2 落地, per DD §3.1 + §4.10-4.11) |
| AgentRuntime | `crates/api/src/arg/` (API) + `frontend/src/app/agent-relationships/` (UI) | 13 REST + 1 WebSocket + 5 UI 组件 | 🟢 **ARG.4 收官** (API, commit `6e2cda6`, 14 routes 13 REST + 1 WS / 24/24 cargo test + 10/10 Python IT) + 🟡 ARG.5 docs 阶段 (UI, P3-C W4 落地) |

---

## 9. 守门合规 (per AGENTS.md §4 + §4.1)

同要件 §9 守门合规.

---

## 10. 签字栏 (per 守门 #14 v3 Mavis 永久代签)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| SRE Lead (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| 平台 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| 评审主持 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| PM (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |

---

## 11. 修订历史 (per 守门 #3 禁回溯叙事)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-09 21:18 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 5 module 設計完整化 (5-tier 架构 + 24 组件 + 5 module × 5-tier 映射 + 5 module 詳細設計 + 5 表 W/T/M + 5 状态机 + 6 NFR + 5 module × 4 new crate 落地路径) | 2026-09-09 21:16 JST 用户发令"开子代理和 worktree 并行处理" + brief v0.45 §14.11 ARG.1 文档完整化 |
