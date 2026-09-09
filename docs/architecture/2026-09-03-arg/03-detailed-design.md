# ARG-ARCH-001 Agent Relationship Graph (ARG) 詳細設計書

> **Status**: 🟡 Draft v0.1 (per 2026-09-09 21:18 JST 派发 brief v0.45 §14.11 ARG.1 文档完整化)
> **Created**: 2026-09-09
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md)
> **承接**: `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1 + `docs/design/BD-AGENT-RELATIONSHIP-001.md` v0.1 + `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 + 9/4 19:00 JST ARG.1 `crates/arg` 6 子模块骨架落地 (per `docs/briefs/arg-01-arg-crate-skeleton.md` §2)

---

## 0. 目的 (Objective)

承接基本設計書 §0 入口, 本詳細設計書定义 ARG 阶段 5 module 体系 (ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime) 的 **module interface 详细化** (Rust 草案类型 + Python 协议 + 关键算法 + 错误处理 + 5 张表 schema DDL + 测试用例), 跨 5 文档严格一致.

---

## 1. ArgCrate 详细接口 (per 基本 §3.1)

### 1.1 Rust 类型草案 (per crates/arg/src/models/agent.rs)

```rust
// crates/arg/src/models/agent.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::ARGError;

/// Agent ID 强类型 (per §6.1 v18 强类型重构)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self { Self(Uuid::now_v7()) }
}

/// 5 域业务类型 (跟 9 SA 解耦, SA 是任务类型)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    // player
    PlayerAccount,
    PlayerRole,
    // economy
    Wallet,
    Transaction,
    // match
    MatchEngine,
    Leaderboard,
    // social
    FriendGraph,
    ChatRoom,
    // admin
    AuditAgent,
    ComplianceAgent,
    // 9 SA (per 要件 §6.1)
    SACodeReview,        // SA-01
    SATestGen,           // SA-02
    SA5DomainLeadAudit,  // SA-03
    SAGitOps,            // SA-04
    SADocSync,           // SA-05
    SARefactor,          // SA-06
    SADBMigration,       // SA-07
    SADomainDev,         // SA-08
    SAAchievementEval,   // SA-09
}

/// Agent 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Standby,
    Archived,
}

/// 信任分 5 段
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrustScoreTier {
    Untrusted,   // 0.0 - 0.2
    Low,         // 0.2 - 0.4
    Medium,      // 0.4 - 0.7
    High,        // 0.7 - 0.9
    VeryHigh,    // 0.9 - 1.0
}

impl TrustScoreTier {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.2 => Self::Untrusted,
            s if s < 0.4 => Self::Low,
            s if s < 0.7 => Self::Medium,
            s if s < 0.9 => Self::High,
            _ => Self::VeryHigh,
        }
    }
}

/// Agent 16 字段 (per 要件 §3.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub agent_type: AgentType,
    pub tenant_id: TenantId,
    pub label: String,
    pub status: AgentStatus,
    pub trust_score: f64,            // 0.0 - 1.0
    pub metadata: serde_json::Value, // 业务 metadata (e.g. 玩家 ID / 钱包 ID)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,               // SCD Type 2 version
    pub is_deleted: bool,           // soft delete (Master 派生规)
    pub created_by: AgentId,
    pub updated_by: AgentId,
    pub rls_class: RLSClass,        // 13 类 RLS
    pub previous_version_id: Option<AgentId>, // SCD Type 2 chain
}

/// RLS 13 类 (per 守门 #13 派生规 (d))
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RLSClass {
    TenantAdmin,
    TenantUser,
    CrossTenantPublic,
    CrossTenantRestricted,
    AgentOwner,
    AgentDelegate,
    AuditReadOnly,
    AuditWriteOnly,
    ComplianceReadOnly,
    ComplianceWriteOnly,
    PublicReadOnly,
    InternalService,
    SystemRoot,
}

impl RLSClass {
    pub fn can_read(&self, actor: &Agent, target: &Agent) -> bool {
        // 13 类 RLS 检查逻辑, 跟 domain-credential RLS 13 类一致
        // per 守门 #13 派生规 (d): Master 100% RLS
        todo!() // P3-C W1 落地
    }
}

/// Tenant ID 强类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub Uuid);
```

### 1.2 关键算法

#### 1.2.1 Agent 创建 (with SCD Type 2)

```rust
// crates/arg/src/ops/agent_node.rs
impl ArgCrate {
    pub async fn create_agent(&self, new: NewAgent) -> Result<Agent, ARGError> {
        // 1. RLS 检查 (per 守门 #13 派生规 (d))
        let actor = self.actor_context.current();
        if !actor.rls_class.can_create_agent() {
            return Err(ARGError::RLSViolation);
        }

        // 2. 验证 agent_type (9 SA + 5 域 + 业务)
        validate_agent_type(&new.agent_type)?;

        // 3. 生成 ID
        let id = AgentId::new();
        let now = Utc::now();

        // 4. SCD Type 2: 写入新版本
        let agent = Agent {
            id,
            agent_type: new.agent_type,
            tenant_id: new.tenant_id,
            label: new.label,
            status: AgentStatus::Active,
            trust_score: 0.5,  // 默认 MEDIUM
            metadata: new.metadata,
            created_at: now,
            updated_at: now,
            version: 1,
            is_deleted: false,
            created_by: actor.id,
            updated_by: actor.id,
            rls_class: derive_rls_class(&new.agent_type, actor.tenant_id),
            previous_version_id: None,
        };

        // 5. Memgraph 持久化
        let cypher = format!(
            "CREATE (a:Agent {{ id: '{}', agent_type: '{}', tenant_id: '{}', \
             label: '{}', status: 'active', trust_score: 0.5, \
             created_at: '{}', version: 1, rls_class: '{}' }}) RETURN a",
            agent.id.0, agent.agent_type, agent.tenant_id.0, agent.label,
            now.to_rfc3339(), agent.rls_class,
        );
        self.memgraph.execute_cypher(&cypher).await?;

        // 6. Cypher 缓存失效
        self.cypher_cache.invalidate_agent(id).await;

        // 7. 事件写入 (per AgentLease)
        self.event_writer.write_event(ARGEvent {
            id: EventId::new(),
            event_type: EventType::AgentCreated,
            source_id: id,
            target_id: id,
            payload: serde_json::to_value(&agent)?,
            timestamp: now,
        }).await?;

        Ok(agent)
    }
}
```

#### 1.2.2 Edge 创建 (with cycle detection)

```rust
// crates/arg/src/ops/edge_ops.rs
impl ArgCrate {
    pub async fn create_edge(&self, new: NewEdge) -> Result<Edge, ARGError> {
        // 1. RLS 检查
        let actor = self.actor_context.current();
        if !actor.rls_class.can_create_edge(&new) {
            return Err(ARGError::RLSViolation);
        }

        // 2. 跨域约束检查 (per 守门 #3 v2: 5 域之间强制 consults 而非 delegates_to)
        let source = self.get_agent(new.source_id).await?;
        let target = self.get_agent(new.target_id).await?;
        if source.tenant_id != target.tenant_id
            && new.relationship_type == RelationshipType::DelegatesTo {
            return Err(ARGError::InvalidEdge(
                "跨域边必须用 consults 而非 delegates_to (per 守门 #3 v2)".into()
            ));
        }

        // 3. 循环检测 (delegates_to / reports_to 不允许成环)
        if matches!(new.relationship_type,
            RelationshipType::DelegatesTo | RelationshipType::ReportsTo) {
            if self.detect_cycle(new.source_id, new.target_id, new.relationship_type).await? {
                return Err(ARGError::InvalidEdge("检测到环, 不允许".into()));
            }
        }

        // 4. 权重范围检查
        if !(0.0..=1.0).contains(&new.weight) {
            return Err(ARGError::InvalidEdge(
                format!("weight 必须在 [0.0, 1.0] 范围, 实际 {}", new.weight)
            ));
        }

        // 5. Memgraph 持久化
        let id = EdgeId::new();
        let now = Utc::now();
        let cypher = format!(
            "MATCH (s:Agent {{ id: '{}' }}), (t:Agent {{ id: '{}' }}) \
             CREATE (s)-[r:EDGE {{ id: '{}', relationship_type: '{}', weight: {}, \
             created_at: '{}', version: 1 }}]->(t) RETURN r",
            new.source_id.0, new.target_id.0, id.0, new.relationship_type,
            new.weight, now.to_rfc3339(),
        );
        self.memgraph.execute_cypher(&cypher).await?;

        // 6. 事件写入
        self.event_writer.write_event(ARGEvent {
            id: EventId::new(),
            event_type: EventType::EdgeCreated,
            source_id: new.source_id,
            target_id: new.target_id,
            payload: serde_json::json!({"edge_id": id, "relationship_type": new.relationship_type}),
            timestamp: now,
        }).await?;

        Ok(Edge { id, ..new.into() })
    }
}
```

### 1.3 错误处理 (per crates/arg/src/error.rs)

```rust
// crates/arg/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ARGError {
    #[error("agent not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("invalid edge: {0}")]
    InvalidEdge(String),

    #[error("invalid template: {0}")]
    InvalidTemplate(String),

    #[error("Memgraph down: {0}")]
    MemgraphDown(String),

    #[error("Bolt timeout: {0}")]
    BoltTimeout(String),

    #[error("cache miss: {0}")]
    CacheMiss(String),

    #[error("schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("RLS violation: {0}")]
    RLSViolation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl ARGError {
    pub fn http_status(&self) -> u16 {
        match self {
            Self::NotFound(_) => 404,
            Self::Unauthorized(_) | Self::RLSViolation(_) => 403,
            Self::InvalidEdge(_) | Self::InvalidTemplate(_) => 400,
            Self::MemgraphDown(_) | Self::BoltTimeout(_) => 503,
            _ => 500,
        }
    }
}
```

---

## 2. SubAgentOrchestrator 详细接口 (per 基本 §3.2)

### 2.1 Python 协议 (per crates/arg-effect/src/protocol.py)

```python
# crates/arg-effect/src/protocol.py
from dataclasses import dataclass, field
from typing import Literal, Optional
from enum import Enum
import time

class TrustScoreTier(str, Enum):
    UNTRUSTED = "untrusted"   # 0.0 - 0.2
    LOW = "low"               # 0.2 - 0.4
    MEDIUM = "medium"         # 0.4 - 0.7
    HIGH = "high"             # 0.7 - 0.9
    VERY_HIGH = "very_high"   # 0.9 - 1.0

@dataclass
class ARGDispatchRoute:
    """arg_dispatch_route 协议 (F-15)"""
    task_id: str
    lead_id: str
    worker_id: str
    relationship_type: Literal["delegates_to", "stand_in_for", "consults", "collaborates"]
    weight: float  # 0.0 - 1.0
    reason: str
    timestamp: int = field(default_factory=lambda: int(time.time() * 1000))

@dataclass
class ARGContextInject:
    """arg_context_inject 协议 (F-16)"""
    mentee_id: str
    mentor_ids: list[str]
    shadow_ids: list[str]
    history_decisions: list[dict]  # 拉 mentors 历史
    token_estimate: int  # 20-40% 节省
    timestamp: int = field(default_factory=lambda: int(time.time() * 1000))

@dataclass
class ARGTrustScoreUpdate:
    """arg_trust_score_update 协议 (F-12)"""
    agent_id: str
    old_score: float
    new_score: float
    reason: Literal["success", "failure", "peer_review", "challenge_pass", "challenge_fail"]
    timestamp: int = field(default_factory=lambda: int(time.time() * 1000))

@dataclass
class ARGContextInject:
    """arg_context_inject 协议 (F-16)"""
    mentee_id: str
    mentor_ids: list[str]
    shadow_ids: list[str]
    history_decisions: list[dict]  # 拉 mentors 历史
    token_estimate: int  # 20-40% 节省
    timestamp: int = field(default_factory=lambda: int(time.time() * 1000))

@dataclass
class ARGAchievementUnlocked:
    """arg_achievement_unlocked 协议 (F-14)"""
    agent_id: str
    achievement_code: str  # e.g. "TOP-001" (Triangle)
    unlocked_at: int
    notification: str  # 解锁文案 (per LLMService 生成)
    timestamp: int = field(default_factory=lambda: int(time.time() * 1000))

@dataclass
class ARGEdgeChanged:
    """arg_edge_changed 协议 (F-13)"""
    edge_id: str
    change_type: Literal["created", "updated", "archived"]
    source_id: str
    target_id: str
    relationship_type: str
    new_weight: float
    timestamp: int = field(default_factory=lambda: int(time.time() * 1000))
```

### 2.2 关键算法: 8 拓扑成就 Cypher (per G-6 已闭环)

```python
# crates/arg-effect/src/achievement/topology.py
TOPOLOGY_CYPHERS = {
    # TOP-001 Triangle: 3 节点成环 (8C)
    "TOP-001": """
        MATCH (a:Agent)-[r1:EDGE]->(b:Agent)-[r2:EDGE]->(c:Agent)-[r3:EDGE]->(a)
        WHERE a.tenant_id = $tenant_id
        WITH a, b, c, count(DISTINCT r1) + count(DISTINCT r2) + count(DISTINCT r3) AS edges
        WHERE edges = 3
        RETURN DISTINCT a.id AS unlocked_for
    """,

    # TOP-002 K-Core (K=3): 每个节点至少 3 邻居 (8C)
    "TOP-002": """
        MATCH (a:Agent)-[r:EDGE]-()
        WHERE a.tenant_id = $tenant_id
        WITH a, count(DISTINCT r) AS degree
        WHERE degree >= 6
        RETURN a.id AS unlocked_for
    """,

    # TOP-003 Star: 1 中心 + ≥ 4 叶子 (8C)
    "TOP-003": """
        MATCH (center:Agent)-[r:EDGE]->(leaf:Agent)
        WHERE center.tenant_id = $tenant_id
        WITH center, count(DISTINCT leaf) AS leaf_count
        WHERE leaf_count >= 4
        RETURN center.id AS unlocked_for
    """,

    # TOP-004 Path: 长度 ≥ 5 链 (8C)
    "TOP-004": """
        MATCH p = (start:Agent)-[r:EDGE*5..10]->(end:Agent)
        WHERE start.tenant_id = $tenant_id
        RETURN DISTINCT start.id AS unlocked_for
        LIMIT 100
    """,

    # TOP-005 Cycle: 长度 ≥ 4 环 (7R)
    "TOP-005": """
        MATCH p = (a:Agent)-[r:EDGE*4..8]->(a)
        WHERE a.tenant_id = $tenant_id
        RETURN DISTINCT a.id AS unlocked_for
        LIMIT 50
    """,

    # TOP-006 Complete: N 节点全连接 (7R)
    "TOP-006": """
        MATCH (a:Agent)-[r:EDGE]-(b:Agent)
        WHERE a.tenant_id = $tenant_id
        WITH a, count(DISTINCT b) AS degree
        WHERE degree >= 9
        RETURN a.id AS unlocked_for
    """,

    # TOP-007 Bridge: 2 个 cluster 通过 1 节点连接 (3E)
    "TOP-007": """
        MATCH (a:Agent)-[r:EDGE]-(b:Agent)-[r2:EDGE]-(c:Agent)
        WHERE a.tenant_id = $tenant_id AND a.id <> c.id
        WITH b, count(DISTINCT a) + count(DISTINCT c) AS bridge_degree
        WHERE bridge_degree >= 8
        RETURN b.id AS unlocked_for
    """,

    # TOP-008 Hub: 高度数节点 (≥ 8 边) (2L)
    "TOP-008": """
        MATCH (a:Agent)-[r:EDGE]-()
        WHERE a.tenant_id = $tenant_id
        WITH a, count(DISTINCT r) AS degree
        WHERE degree >= 16
        RETURN a.id AS unlocked_for
    """,
}
```

### 2.3 关键算法: 4 effect 维度真实落地 (per 要件 §1.1)

```python
# crates/arg-effect/src/router/dispatch.py
class ARGDispatchRouter:
    """4 effect 维度 #1: Dispatch 路由"""

    async def dispatch_task(self, lead_id: str, task: dict) -> str:
        # 1. 拉 lead 的 outgoing delegates_to edges
        cypher = """
            MATCH (lead:Agent {id: $lead_id})-[r:EDGE {relationship_type: 'delegates_to'}]->(worker:Agent)
            WHERE r.weight >= 0.5
            RETURN worker.id AS worker_id, r.weight AS weight
            ORDER BY r.weight DESC
        """
        workers = await self.arg_crate.query_cypher(cypher, {"lead_id": lead_id})

        if not workers:
            # 2. fallback 到 stand_in_for
            cypher = """
                MATCH (lead:Agent {id: $lead_id})-[r:EDGE {relationship_type: 'stand_in_for'}]->(worker:Agent)
                WHERE r.weight >= 0.3
                RETURN worker.id AS worker_id, r.weight AS weight
                ORDER BY r.weight DESC
                LIMIT 1
            """
            stand_ins = await self.arg_crate.query_cypher(cypher, {"lead_id": lead_id})
            if stand_ins:
                worker_id = stand_ins[0]["worker_id"]
                await self._emit_dispatch_route(lead_id, worker_id, "stand_in_for", stand_ins[0]["weight"])
                return worker_id

            # 3. 最后 fallback: 5 域 Lead (per 守门 #3 v2, Mavis 临时代签)
            worker_id = await self._fallback_to_5_domain_lead(lead_id, task)
            return worker_id

        # 4. 正常派发: 选 weight 最高的 worker
        worker_id = workers[0]["worker_id"]
        await self._emit_dispatch_route(lead_id, worker_id, "delegates_to", workers[0]["weight"])
        return worker_id
```

```python
# crates/arg-effect/src/injector/context.py
class ARGContextInjector:
    """4 effect 维度 #2: 上下文共享 (token 节省 20-40%)"""

    async def inject_context(self, mentee_id: str, task: dict) -> dict:
        # 1. 拉 mentors 历史决策
        cypher = """
            MATCH (mentee:Agent {id: $mentee_id})-[r:EDGE {relationship_type: 'mentors'}]->(mentor:Agent)
            MATCH (mentor)-[r2:EDGE]->(decision:Decision)
            WHERE decision.timestamp > $since
            RETURN decision.payload AS payload, decision.timestamp AS ts
            ORDER BY decision.timestamp DESC
            LIMIT 10
        """
        since = int((time.time() - 86400 * 7) * 1000)  # 7 天
        decisions = await self.arg_crate.query_cypher(cypher, {
            "mentee_id": mentee_id, "since": since,
        })

        # 2. LLMService 摘要 (per LLMService.summarize_peer_review)
        summary = await self.llm_service.summarize_peer_review(
            PeerReview(payloads=[d["payload"] for d in decisions]),
            length=ReviewLength.SHORT,  # 100 字
        )

        # 3. token 估算 (per G-14 实证 20-40%)
        token_estimate = max(50, len(decisions) * 80 - len(summary) * 4)

        # 4. shadow 静默订阅
        shadow_ids = await self._get_shadow_ids(mentee_id)

        return ARGContextInject(
            mentee_id=mentee_id,
            mentor_ids=[d["mentor_id"] for d in decisions],
            shadow_ids=shadow_ids,
            history_decisions=[d["payload"] for d in decisions],
            token_estimate=token_estimate,
        )
```

---

## 3. LLMService 详细接口 (per 基本 §3.3)

### 3.1 Rust 草案 (per crates/arg/src/llm.rs)

```rust
// crates/arg/src/llm.rs
use crate::error::ARGError;
use crate::models::{Agent, Achievement, ARGEvent, DecisionType, TrustScoreTier};
use std::sync::Arc;
use star_llm_pool::LLMPool;  // 共享 L0 派发层 LLM Pool (per ADR-0045)

pub struct LLMService {
    pool: Arc<LLMPool>,
    mock_mode: bool,  // per 守门 #23 AI mock 不开外部 API
}

impl LLMService {
    /// 1. 挑战双向论证 prompt 模板 (per DD §7: 5 decision_type × 2 trust_tier)
    pub async fn generate_challenge_prompt(
        &self,
        decision_type: DecisionType,
        trust_tier: TrustScoreTier,
    ) -> Result<String, ARGError> {
        if self.mock_mode {
            return Ok(MOCK_CHALLENGE_PROMPTS
                .get(&(decision_type, trust_tier))
                .cloned()
                .unwrap_or_default());
        }

        let prompt = CHALLENGE_PROMPT_TEMPLATE
            .replace("{decision_type}", &format!("{:?}", decision_type))
            .replace("{trust_tier}", &format!("{:?}", trust_tier));

        self.pool.complete(&prompt, "gpt-4o-mini").await
            .map_err(ARGError::from)
    }

    /// 2. peer review 摘要
    pub async fn summarize_peer_review(
        &self,
        review: PeerReview,
        length: ReviewLength,
    ) -> Result<String, ARGError> {
        if self.mock_mode {
            let target_len = match length {
                ReviewLength::Short => 100,
                ReviewLength::Long => 500,
            };
            return Ok(format!("[MOCK] peer review summary ({} chars)", target_len));
        }

        let prompt = format!(
            "请将以下 peer review 摘要成 {} 字: {}",
            match length { ReviewLength::Short => "100", ReviewLength::Long => "500" },
            serde_json::to_string(&review).unwrap_or_default(),
        );

        self.pool.complete(&prompt, "gpt-4o-mini").await
            .map_err(ARGError::from)
    }

    /// 3. 8 拓扑成就 Cypher 模板生成 (per G-6 已闭环)
    pub async fn generate_topology_cypher(
        &self,
        topology: TopologyType,
    ) -> Result<String, ARGError> {
        if self.mock_mode {
            return Ok(TOPOLOGY_CYPHERS.get(&topology).cloned().unwrap_or_default());
        }
        // 真实 LLM 生成: 已闭环, 直接返回静态模板
        Ok(TOPOLOGY_CYPHERS.get(&topology).cloned().unwrap_or_default())
    }

    /// 4. 解锁文案生成
    pub async fn generate_unlock_message(
        &self,
        achievement: Achievement,
    ) -> Result<String, ARGError> {
        if self.mock_mode {
            return Ok(format!(
                "🎉 解锁成就: {} ({})\n  条件: {}\n  稀有度: {:?}",
                achievement.name, achievement.code, achievement.description, achievement.rarity
            ));
        }

        let prompt = format!(
            "请为以下成就生成 3 段解锁文案 (庆祝/条件/稀有度):\n{}",
            serde_json::to_string(&achievement).unwrap_or_default(),
        );

        self.pool.complete(&prompt, "gpt-4o-mini").await
            .map_err(ARGError::from)
    }

    /// 5. ARG 事件摘要
    pub async fn summarize_arg_event(
        &self,
        event: ARGEvent,
    ) -> Result<String, ARGError> {
        if self.mock_mode {
            return Ok(format!("[MOCK] {} event at {}",
                event.event_type, event.timestamp));
        }

        let prompt = format!(
            "请将以下 ARG 事件摘要成 1 句话: {}",
            serde_json::to_string(&event).unwrap_or_default(),
        );

        self.pool.complete(&prompt, "gpt-4o-mini").await
            .map_err(ARGError::from)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReviewLength { Short, Long }
```

---

## 4. AgentLease 详细接口 (per 基本 §3.4)

### 4.1 Rust 草案 (per crates/arg-bridge/src/lease.rs)

```rust
// crates/arg-bridge/src/lease.rs
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;
use sled::Db;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LeaseId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HolderType {
    Lead,
    Worker,
    Orchestrator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    EdgeCreate,
    EdgeUpdate,
    EdgeArchive,
    TemplateInstantiate,
    AchievementUnlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeaseState {
    Active,
    Expired,
    Released,
    Aborted,
}

/// AgentLease 11 字段 (per ADR-0030 + 基本 §3.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLease {
    pub lease_id: LeaseId,                  // 1. UUID v7
    pub agent_id: AgentId,                  // 2. 租户 agent
    pub tenant_id: TenantId,                // 3. 租户隔离
    pub acquired_at: DateTime<Utc>,         // 4. 获取时间
    pub expires_at: DateTime<Utc>,          // 5. 过期时间 (默认 30s)
    pub heartbeat_at: DateTime<Utc>,        // 6. 心跳时间 (每 10s)
    pub holder: HolderType,                 // 7. Lead / Worker / Orchestrator
    pub operation: OperationType,           // 8. 关系修改 / 模板 instantiate / 成就解锁
    pub target_id: String,                  // 9. edge_id / template_id / achievement_id
    pub state: LeaseState,                  // 10. Active / Expired / Released / Aborted
    pub context: serde_json::Value,         // 11. 上下文 (per LLMService 摘要)
}

/// Lease manager (per ADR-0030 + 守门 #1 v25)
pub struct LeaseManager {
    leases: Arc<Mutex<HashMap<LeaseId, AgentLease>>>,
    sled: Db,  // 离线降级
    heartbeat_interval: Duration,  // 10s
    default_ttl: Duration,         // 30s
}

impl LeaseManager {
    pub async fn acquire(
        &self,
        agent_id: AgentId,
        tenant_id: TenantId,
        holder: HolderType,
        operation: OperationType,
        target_id: String,
        context: serde_json::Value,
    ) -> Result<AgentLease, ARGError> {
        let lease_id = LeaseId(Uuid::now_v7());
        let now = Utc::now();

        let lease = AgentLease {
            lease_id,
            agent_id,
            tenant_id,
            acquired_at: now,
            expires_at: now + self.default_ttl,
            heartbeat_at: now,
            holder,
            operation,
            target_id,
            state: LeaseState::Active,
            context,
        };

        // 1. 内存存储
        self.leases.lock().await.insert(lease_id, lease.clone());

        // 2. sled 持久化 (离线降级)
        self.sled.insert(
            lease_id.0.as_bytes(),
            serde_json::to_vec(&lease).map_err(ARGError::from)?,
        ).map_err(ARGError::from)?;

        // 3. 启动心跳任务
        self.start_heartbeat(lease_id).await;

        Ok(lease)
    }

    pub async fn heartbeat(&self, lease_id: LeaseId) -> Result<(), ARGError> {
        let mut leases = self.leases.lock().await;
        let lease = leases.get_mut(&lease_id).ok_or(ARGError::NotFound("lease".into()))?;
        lease.heartbeat_at = Utc::now();
        lease.expires_at = lease.heartbeat_at + self.default_ttl;
        Ok(())
    }

    pub async fn release(&self, lease_id: LeaseId) -> Result<(), ARGError> {
        let mut leases = self.leases.lock().await;
        let lease = leases.get_mut(&lease_id).ok_or(ARGError::NotFound("lease".into()))?;
        lease.state = LeaseState::Released;
        self.sled.remove(lease_id.0.as_bytes()).map_err(ARGError::from)?;
        Ok(())
    }

    async fn start_heartbeat(&self, lease_id: LeaseId) {
        let leases = self.leases.clone();
        let interval = self.heartbeat_interval;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval.to_std().unwrap()).await;
                let mut leases = leases.lock().await;
                if let Some(lease) = leases.get_mut(&lease_id) {
                    if lease.state != LeaseState::Active {
                        break;
                    }
                    if Utc::now() > lease.expires_at {
                        lease.state = LeaseState::Expired;
                        break;
                    }
                    lease.heartbeat_at = Utc::now();
                } else {
                    break;
                }
            }
        });
    }
}
```

---

## 5. AgentRuntime 详细接口 (per 基本 §3.5)

### 5.1 Rust REST handlers (per crates/api/src/arg/handlers.rs)

```rust
// crates/api/src/arg/handlers.rs
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use star_arg::{ArgCrate, NewAgent, NewEdge, GraphQuery, NewTemplateParams};
use star_context::ActorContext;

pub async fn create_agent(
    State(crate_ref): State<Arc<ArgCrate>>,
    actor: ActorContext,
    Json(new): Json<NewAgent>,
) -> impl IntoResponse {
    match crate_ref.create_agent(new).await {
        Ok(agent) => (StatusCode::CREATED, Json(agent)).into_response(),
        Err(e) => (StatusCode::from_u16(e.http_status()).unwrap(), Json(e)).into_response(),
    }
}

pub async fn get_agent(
    State(crate_ref): State<Arc<ArgCrate>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match crate_ref.get_agent(AgentId(id)).await {
        Ok(agent) => (StatusCode::OK, Json(agent)).into_response(),
        Err(e) => (StatusCode::from_u16(e.http_status()).unwrap(), Json(e)).into_response(),
    }
}

pub async fn list_agents(
    State(crate_ref): State<Arc<ArgCrate>>,
    Query(filter): Query<AgentFilter>,
) -> impl IntoResponse {
    match crate_ref.list_agents(filter).await {
        Ok(agents) => (StatusCode::OK, Json(agents)).into_response(),
        Err(e) => (StatusCode::from_u16(e.http_status()).unwrap(), Json(e)).into_response(),
    }
}

// 同样模式: patch_agent / create_edge / get_edge / patch_edge / delete_edge /
// list_edges / query_graph / instantiate_template / list_achievements / unlock_achievement

/// WebSocket handler (per /ws/arg/events)
pub async fn arg_ws_handler(
    ws: WebSocketUpgrade,
    State(crate_ref): State<Arc<ArgCrate>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        // 6 事件类型推送: edge.changed / edge.created / edge.archived /
        // achievement.unlocked / dispatch.route.changed / agent.trust_score.changed
        let _ = socket; // placeholder
    })
}
```

### 5.2 TypeScript UI 组件 (per frontend/src/lib/arg/store.ts)

```typescript
// frontend/src/lib/arg/store.ts
import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';

export interface ARGStore {
  // 5 channel (per 基本 §3.5)
  agents: Agent[];
  edges: Edge[];
  templates: TeamTemplate[];
  achievements: Achievement[];
  events: ARGEvent[];

  // actions
  fetchAgents: (filter?: AgentFilter) => Promise<void>;
  createEdge: (newEdge: NewEdge) => Promise<Edge>;
  queryGraph: (query: GraphQuery) => Promise<GraphSnapshot>;
  instantiateTemplate: (templateId: string, params: TemplateParams) => Promise<TemplateInstance>;
  unlockAchievement: (code: string) => Promise<AchievementUnlock>;
}

export const useARGStore = create<ARGStore>()(immer((set, get) => ({
  agents: [],
  edges: [],
  templates: [],
  achievements: [],
  events: [],

  fetchAgents: async (filter) => {
    const response = await fetch(`/api/arg/agents?${new URLSearchParams(filter as any)}`);
    const agents = await response.json();
    set((state) => { state.agents = agents; });
  },

  createEdge: async (newEdge) => {
    const response = await fetch('/api/arg/edges', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newEdge),
    });
    const edge = await response.json();
    set((state) => { state.edges.push(edge); });
    return edge;
  },

  // ... 同样模式: queryGraph / instantiateTemplate / unlockAchievement
})));
```

### 5.3 React 组件: RelationshipEditor (per 要件 NFR-U-02 ≤ 3 步)

```typescript
// frontend/src/app/agent-relationships/components/RelationshipEditor.tsx
import { useState } from 'react';
import { useARGStore } from '@/lib/arg/store';

export function RelationshipEditor() {
  const [sourceId, setSourceId] = useState<string>('');
  const [targetId, setTargetId] = useState<string>('');
  const [relationshipType, setRelationshipType] = useState<string>('delegates_to');
  const [weight, setWeight] = useState<number>(0.5);

  const createEdge = useARGStore((state) => state.createEdge);

  const handleSubmit = async () => {
    // 1 click: 选 source agent
    // 2 click: 选 target agent
    // 3 click: 提交 (≤ 3 步 per NFR-U-02)
    await createEdge({ source_id: sourceId, target_id: targetId, relationship_type: relationshipType, weight });
  };

  return (
    <div className="relationship-editor">
      <h3>关系编辑器 (3 步完成)</h3>
      <select onChange={(e) => setSourceId(e.target.value)}>
        {/* 1 click: source */}
        {useARGStore.getState().agents.map((a) => (
          <option key={a.id} value={a.id}>{a.label}</option>
        ))}
      </select>
      <select onChange={(e) => setRelationshipType(e.target.value)}>
        {/* 2 click: relationship_type */}
        <option value="delegates_to">delegates_to</option>
        <option value="reports_to">reports_to</option>
        <option value="mentors">mentors</option>
        <option value="shadows">shadows</option>
        <option value="trusts">trusts</option>
        <option value="consults">consults</option>
        <option value="collaborates">collaborates</option>
        <option value="stand_in_for">stand_in_for</option>
        <option value="challenges">challenges</option>
        <option value="peer_reviews">peer_reviews</option>
      </select>
      <input type="number" min="0" max="1" step="0.1" value={weight} onChange={(e) => setWeight(parseFloat(e.target.value))} />
      <button onClick={handleSubmit}>创建 (3 click)</button>
    </div>
  );
}
```

---

## 6. 5 张表 Schema DDL (per 守门 #13 W/T/M 严格 + 基本 §4.1)

### 6.1 agents 表 (Master / SCD Type 2 / 100% RLS)

```sql
-- crates/arg/migrations/2026-09-09-001-create-agents.sql
-- per 守门 #13 派生规 (c): M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携

CREATE TABLE agents (
    id UUID PRIMARY KEY,                          -- UUID v7
    agent_type TEXT NOT NULL,                     -- 5 域 + 9 SA + 业务
    tenant_id UUID NOT NULL,                      -- 租户隔离
    label TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',        -- active / standby / archived
    trust_score REAL NOT NULL DEFAULT 0.5,        -- 0.0 - 1.0
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 1,           -- SCD Type 2
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,    -- 软删除
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    rls_class TEXT NOT NULL,                      -- 13 类 RLS
    previous_version_id UUID,                     -- SCD Type 2 chain
    FOREIGN KEY (previous_version_id) REFERENCES agents(id)
);

CREATE INDEX idx_agents_tenant_id ON agents(tenant_id);
CREATE INDEX idx_agents_agent_type ON agents(agent_type);
CREATE INDEX idx_agents_trust_score ON agents(trust_score);
CREATE INDEX idx_agents_rls_class ON agents(rls_class);

-- 100% RLS 13 类 (per 守门 #13 派生规 (d))
ALTER TABLE agents ENABLE ROW LEVEL SECURITY;
CREATE POLICY agents_tenant_isolation ON agents
    USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

### 6.2 edges 表 (Master / SCD Type 2 / 100% RLS)

```sql
-- crates/arg/migrations/2026-09-09-002-create-edges.sql
CREATE TABLE edges (
    id UUID PRIMARY KEY,
    source_id UUID NOT NULL REFERENCES agents(id),
    target_id UUID NOT NULL REFERENCES agents(id),
    relationship_type TEXT NOT NULL,              -- 10 类关系枚举
    weight REAL NOT NULL DEFAULT 0.5 CHECK (weight >= 0.0 AND weight <= 1.0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 1,           -- SCD Type 2
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    rls_class TEXT NOT NULL,
    previous_version_id UUID,
    FOREIGN KEY (previous_version_id) REFERENCES edges(id)
);

CREATE INDEX idx_edges_source_id ON edges(source_id);
CREATE INDEX idx_edges_target_id ON edges(target_id);
CREATE INDEX idx_edges_relationship_type ON edges(relationship_type);

ALTER TABLE edges ENABLE ROW LEVEL SECURITY;
CREATE POLICY edges_tenant_isolation ON edges
    USING (EXISTS (
        SELECT 1 FROM agents
        WHERE agents.id = edges.source_id
        AND agents.tenant_id = current_setting('app.tenant_id')::UUID
    ));
```

### 6.3 audit 表 (Transaction / append-only / 100% RLS)

```sql
-- crates/arg/migrations/2026-09-09-003-create-audit.sql
-- per 守门 #13 派生规 (b): T = 物理删除禁止 + 監査必須 + RLS 13 類必携

CREATE TABLE audit (
    id UUID PRIMARY KEY,
    event_type TEXT NOT NULL,                     -- edge.created / edge.updated / agent.created / etc.
    actor_id UUID NOT NULL,                       -- 触发者
    target_id UUID,                               -- 目标 agent/edge
    payload JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    prev_hash TEXT,                               -- 链式审计
    rls_class TEXT NOT NULL
);

-- 物理删除禁止 (per 守门 #13 派生规 (b))
CREATE OR REPLACE FUNCTION prevent_audit_delete() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'audit 表不允许物理删除 (per 守门 #13 派生规 (b))';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER no_audit_delete BEFORE DELETE ON audit
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_delete();

CREATE INDEX idx_audit_event_type ON audit(event_type);
CREATE INDEX idx_audit_timestamp ON audit(timestamp);

ALTER TABLE audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY audit_audit_readonly ON audit
    FOR SELECT USING (rls_class IN ('AuditReadOnly', 'ComplianceReadOnly', 'SystemRoot'));
CREATE POLICY audit_audit_writeonly ON audit
    FOR INSERT WITH CHECK (rls_class IN ('AuditWriteOnly', 'SystemRoot'));
```

### 6.4 template_instances 表 (Work / TTL 30d / 短 TTL 显式 retention)

```sql
-- crates/arg/migrations/2026-09-09-004-create-template-instances.sql
-- per 守门 #13 派生规 (a): W = 物理删除 / タイマー失効 / 短 TTL 明示 retention

CREATE TABLE template_instances (
    id UUID PRIMARY KEY,
    template_id UUID NOT NULL,
    params JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',       -- pending / active / expired
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days')
);

CREATE INDEX idx_template_instances_template_id ON template_instances(template_id);
CREATE INDEX idx_template_instances_expires_at ON template_instances(expires_at);

-- タイマー失効: 过期后物理删除 (per 守门 #13 派生规 (a))
CREATE OR REPLACE FUNCTION cleanup_expired_template_instances() RETURNS void AS $$
BEGIN
    DELETE FROM template_instances WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

-- 每小时跑一次 cron
-- (per 守门 #13 派生规 (a): 短 TTL 明示 retention)
```

### 6.5 unlocks 表 (Transaction / append-only / 100% RLS)

```sql
-- crates/arg/migrations/2026-09-09-005-create-unlocks.sql
CREATE TABLE unlocks (
    id UUID PRIMARY KEY,
    agent_id UUID NOT NULL REFERENCES agents(id),
    achievement_code TEXT NOT NULL,               -- TOP-001 / BEH-001 / OUT-001
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rls_class TEXT NOT NULL
);

-- 物理删除禁止 (per 守门 #13 派生规 (b))
CREATE OR REPLACE FUNCTION prevent_unlocks_delete() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'unlocks 表不允许物理删除 (per 守门 #13 派生规 (b))';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER no_unlocks_delete BEFORE DELETE ON unlocks
    FOR EACH ROW EXECUTE FUNCTION prevent_unlocks_delete();

CREATE UNIQUE INDEX idx_unlocks_agent_achievement ON unlocks(agent_id, achievement_code);

ALTER TABLE unlocks ENABLE ROW LEVEL SECURITY;
CREATE POLICY unlocks_tenant_isolation ON unlocks
    USING (EXISTS (
        SELECT 1 FROM agents
        WHERE agents.id = unlocks.agent_id
        AND agents.tenant_id = current_setting('app.tenant_id')::UUID
    ));
```

---

## 7. 测试用例 (per ARG.6 30 UT + ARG.7 10 IT + 8 E2E + 4 PT)

### 7.1 30 UT 完整列表 (per ARG.6 + DD §10.1)

| Test ID | 范围 | 5 module 命中 | 9 SA 引用 |
|---|---|---|---|
| UT-01..05 | ArgCrate 5 个核心 API (create_agent / get_agent / patch_agent / list_agents / delete_agent) | ArgCrate | (基础) |
| UT-06..10 | ArgCrate 5 个 edge API (create_edge / get_edge / patch_edge / delete_edge / list_edges) | ArgCrate | (基础) |
| UT-11..13 | ArgCrate 3 个 query API (query_graph / behavior / output) | ArgCrate | (基础) |
| UT-14..15 | ArgCrate 2 个 template API (instantiate / list) | ArgCrate | SA-08 |
| UT-16..17 | ArgCrate 2 个 achievement API (unlock / list) | ArgCrate | SA-09 |
| UT-18..20 | ArgCrate 3 个 model 测试 (RLS / SCD / 5 段 trust score) | ArgCrate | (基础) |
| UT-21..24 | SubAgentOrchestrator 4 effect 维度测试 (Dispatch / Context / Trust / Output) | SubAgentOrchestrator | SA-01 + SA-04 |
| UT-25..26 | LLMService 2 个核心方法 (challenge_prompt / unlock_message) | LLMService | SA-01 + SA-09 |
| UT-27..28 | AgentLease 2 个核心方法 (acquire / release) | AgentLease | (基础) |
| UT-29..30 | AgentRuntime 2 个 UI 组件测试 (RelationshipEditor / AchievementWall) | AgentRuntime | (基础) |

### 7.2 10 IT + 8 E2E + 4 PT (per ARG.7 + DD §10.2-§10.4)

| Test ID | 范围 | 5 module 跨模块 |
|---|---|---|
| **IT-01..03** | 3 IT: 拖拽建边 / Dispatch 路由 / Stand-in fallback | AgentRuntime + ArgCrate + SubAgentOrchestrator + AgentLease |
| **IT-04..06** | 3 IT: 上下文注入 / Trust skip verify / 成就解锁推送 | SubAgentOrchestrator + LLMService + ArgCrate + AgentLease |
| **IT-07..10** | 4 IT: 离线降级 / 周期 flush / 5 域 Lead fallback / SCD 历史追溯 | AgentLease + ArgCrate |
| **E2E-01..03** | 3 E2E (Playwright): RelationshipEditor 拖拽 / AchievementWall 推送 / TemplateGallery 1-click | AgentRuntime |
| **E2E-04..06** | 3 E2E: 跨域 consults / 信任度 5 段变更 / 8 拓扑成就解锁 | SubAgentOrchestrator + LLMService + ArgCrate + AgentLease |
| **E2E-07..08** | 2 E2E: SCD Type 2 历史追溯 / RLS 13 类跨 tenant 隔离 | ArgCrate + AgentRuntime |
| **PT-01** | 边创建 P95 < 200ms (1000 次) | ArgCrate |
| **PT-02** | Cypher 查询 P95 < 500ms (1000 次) | ArgCrate + SubAgentOrchestrator |
| **PT-03** | 事件推送 < 100ms (1000 次) | AgentRuntime + AgentLease |
| **PT-04** | 成就评估 P95 < 1s (1000 次) | SubAgentOrchestrator + LLMService |

---

## 8. 5 Module 跟 4 new crate 落地 (per 基本 §8 + 守门 #19 v19)

| 5 Module | 物理 crate | Python 自动化档 | 状态 |
|---|---|---|---|
| ArgCrate | `crates/arg/` | `memgraph_setup.py` (Docker compose + health probe + .env stub, 守门 #5 不打印密码) + `arg_seed.py` (24 节点 5 域 Lead + 9 SA + 10 demo + 10 边) | 🟢 **ARG.1 收官** (33 UT 100% pass) |
| SubAgentOrchestrator | `crates/arg-effect/` | `arg_dispatch_test.py` (10 IT) + `arg_achievement_eval.py` (8 拓扑) | 🟡 ARG.3 docs 阶段 |
| LLMService | `crates/arg/src/llm.rs` | (ArgCrate 子模块, 共用 memgraph_setup.py) | 🟢 **ARG.1 收官** |
| AgentLease | `crates/arg-bridge/` | `arg_bridge_test.py` (10 IT) | 🟡 ARG.2 docs 阶段 |
| AgentRuntime | `crates/api/src/arg/` + `frontend/src/app/agent-relationships/` | `arg_api_test.py` (10 IT) + `arg_ui_test.py` (Playwright) | 🟢 **ARG.4 收官** (API) + 🟡 ARG.5 docs 阶段 (UI) |

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
| v0.1 | 2026-09-09 21:18 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 5 module interface 詳細化 (Rust 草案类型 + Python 协议 + 关键算法 + 错误处理 + 5 表 DDL + 30 UT + 10 IT + 8 E2E + 4 PT) | 2026-09-09 21:16 JST 用户发令"开子代理和 worktree 并行处理" + brief v0.45 §14.11 ARG.1 文档完整化 |
