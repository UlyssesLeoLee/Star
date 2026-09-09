# ARG-ARCH-001 ARG.7 ARG Saga 跨 5 Module 詳細 (per brief v0.50 §14.11)

> **Status**: 🟡 Draft v0.1 (per 2026-09-10 01:36 JST 派发 brief v0.50 §14.11 ARG.2-11 docs 阶段)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md) · [arg-bridge §0 入口](./06-arg-02-arg-bridge.md) · [SA 实施 §0 入口](./07-arg-03-5-sa-impl.md) · [13 REST + 1 WS §0 入口](./08-arg-04-13rest-1ws.md) · [frontend E2E §0 入口](./09-arg-05-frontend-e2e.md) · [PG persistence §0 入口](./10-arg-06-pg-persistence.md)
> **承接**: 守门 #14 v2 拍板 D + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 + v0.45 ARG.1 docs (`c678db7`) + v0.42 §14.10.4 5/6 Repository 收官

---

## 0. 目的 (Objective)

承接基本設計書 §1.1 5-tier 架构 + DD §3.5 + DD §3.4 跨模块协调, 本詳細定義 **ARG.7 ARG Saga 跨 5 module** (ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime) 的 **Saga 协调器 + 7 步标准 Saga + 5 域跨域 Saga 实例**, 包含:

1. **Saga 协调器架构** (per SagaStep + SagaContext + SagaOrchestrator)
2. **7 步标准 Saga** (init / validate / dispatch / execute / evaluate / persist / compensate)
3. **5 域跨域 Saga 实例** (player → economy 玩家代币 / match → social 赛季好友 / social → admin 社交合规 / admin 跨 5 域 / economy → match 比赛奖励)
4. **SagaStep idempotency_key** (per WBS v0.18 §6 SagaStep idempotency_key 字段, 跨 5 module 幂等)
5. **补偿事务** (compensate step, 5 module 跨进程 rollback)
6. **5 Reducer 跟 Saga 集成** (per ARG.2 5 Reducer + LangGraph 9/3 集成)
7. **跨 session 续做边界** (per WBS v0.18 line 663 估 ~22M tokens / 5-7 session, 落档后 v0.51+)

**Token 估算**: ARG.7 docs 阶段 ~1.5M (per v0.45 brief §1 估 22M / 9 docs 均分); Saga 实装跨 session 续 (估 5-7M / 4-5 session).

---

## 1. Saga 协调器架构 (per P3-A Saga 派生 + 5 module 跨域)

### 1.1 Saga 跟 P3-A Saga 派生关系 (per WBS v0.18 §6)

| 维度 | P3-A Saga (per WBS v0.18 §6) | ARG Saga (本 doc) |
|---|---|---|
| SagaStep | `idempotency_key` 字段 (per WBS v0.18 §6 实证) | (跨 ARG 5 module 幂等) |
| SagaContext | (per P3-A 6 字段) | (扩展 ARG 5 module + 5 域) |
| SagaOrchestrator | (per P3-A 7 步) | 7 步 (init/validate/dispatch/execute/evaluate/persist/compensate) |
| 跨 module | (per P3-A 跨 3 module) | 跨 5 module |
| 跨域 | (per P3-A 跨 5 域) | 跨 5 域 + 9 SA |

### 1.2 SagaStep 草案 (per WBS v0.18 §6)

```rust
// crates/saga/src/step.rs (per P3-A §6 + 5 module 跨域)
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    pub step_id: Uuid,
    pub saga_id: Uuid,
    pub step_name: String,          // 7 步之一
    pub module: String,             // 5 module 之一
    pub domain: String,             // 5 域之一
    pub sa_type: Option<String>,    // 9 SA 之一 (if applicable)
    pub idempotency_key: String,    // per WBS v0.18 §6 实证
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub status: SagaStepStatus,     // Pending / Running / Completed / Failed / Compensated
    pub retry_count: u32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaStepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Compensated,
}
```

### 1.3 SagaContext 草案 (per P3-A + ARG 5 module 扩展)

```rust
// crates/saga/src/context.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaContext {
    pub saga_id: Uuid,
    pub saga_name: String,         // 5 域跨域 Saga 实例名
    pub from_domain: String,       // 5 域之一
    pub to_domain: String,         // 5 域之一
    pub actor_id: Uuid,            // 跨 5 域 Lead 真人 (Mavis 临时代签 per 守门 #14 v3)
    pub actor_domain: String,      // 5 域之一
    pub sa_types: Vec<String>,     // 跨 9 SA 之一
    pub steps: Vec<SagaStep>,
    pub status: SagaStatus,        // Active / Completed / Failed / Compensated
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaStatus {
    Active,
    Completed,
    Failed,
    Compensated,
}
```

### 1.4 SagaOrchestrator 草案 (per 7 步)

```rust
// crates/saga/src/orchestrator.rs
pub struct SagaOrchestrator {
    steps: Vec<SagaStep>,
    context: SagaContext,
}

impl SagaOrchestrator {
    pub async fn run(&mut self) -> Result<SagaStatus, SagaError> {
        // 1. init
        self.init().await?;
        // 2. validate
        self.validate().await?;
        // 3. dispatch (per 5 module)
        self.dispatch().await?;
        // 4. execute (per 9 SA)
        self.execute().await?;
        // 5. evaluate (per 4 effect 维度)
        self.evaluate().await?;
        // 6. persist (per PG 5 张表)
        self.persist().await?;
        // 7. compensate (if any step failed)
        if self.context.status == SagaStatus::Failed {
            self.compensate().await?;
        }
        Ok(self.context.status)
    }

    async fn init(&mut self) -> Result<(), SagaError> { /* ... */ }
    async fn validate(&mut self) -> Result<(), SagaError> { /* ... */ }
    async fn dispatch(&mut self) -> Result<(), SagaError> { /* ... */ }
    async fn execute(&mut self) -> Result<(), SagaError> { /* ... */ }
    async fn evaluate(&mut self) -> Result<(), SagaError> { /* ... */ }
    async fn persist(&mut self) -> Result<(), SagaError> { /* ... */ }
    async fn compensate(&mut self) -> Result<(), SagaError> { /* ... */ }
}
```

---

## 2. 7 步标准 Saga 详细

### 2.1 step 1: init (Saga 启动)

```rust
async fn init(&mut self) -> Result<(), SagaError> {
    self.context.saga_id = Uuid::now_v7();
    self.context.started_at = Utc::now();
    self.context.status = SagaStatus::Active;

    // 写 audit 表 (per doc 10 §2.3)
    self.pg_pool.execute(
        "INSERT INTO audit (id, agent_id, sa_type, action, payload, actor_id, trace_id, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())",
        &[
            &Uuid::now_v7(),
            &self.context.actor_id,
            &"saga-init",
            &self.context.saga_name,
            &serde_json::to_value(&self.context)?,
            &self.context.actor_id,
            &self.context.saga_id,
        ],
    ).await?;

    Ok(())
}
```

### 2.2 step 2: validate (5 module 跨域校验)

```rust
async fn validate(&mut self) -> Result<(), SagaError> {
    // 1. 5 域 Lead RACI 校验 (per RACI §1.2 跨域 consults 派生)
    if !self.rbac_can_invoke(self.context.from_domain.as_str(), self.context.to_domain.as_str()).await? {
        return Err(SagaError::RLSViolation);
    }
    // 2. 9 SA Type 校验
    for sa in &self.context.sa_types {
        if !["SA-01","SA-02","SA-03","SA-04","SA-05","SA-06","SA-07","SA-08","SA-09"].contains(&sa.as_str()) {
            return Err(SagaError::InvalidSAType);
        }
    }
    // 3. 跨域 consults 校验 (per 守门 #3 v2 派生, 5 域之间禁止 delegates_to)
    if !self.validate_consults_edge().await? {
        return Err(SagaError::InvalidCrossDomain);
    }
    Ok(())
}
```

### 2.3 step 3: dispatch (5 module 跨 module)

```rust
async fn dispatch(&mut self) -> Result<(), SagaError> {
    for module in ["ArgCrate", "SubAgentOrchestrator", "LLMService", "AgentLease", "AgentRuntime"] {
        let step = SagaStep {
            step_id: Uuid::now_v7(),
            saga_id: self.context.saga_id,
            step_name: "dispatch".to_string(),
            module: module.to_string(),
            domain: self.context.from_domain.clone(),
            sa_type: None,
            idempotency_key: format!("{}:{}:dispatch", self.context.saga_id, module),
            input: serde_json::json!({"from_domain": self.context.from_domain, "to_domain": self.context.to_domain}),
            output: None,
            status: SagaStepStatus::Running,
            retry_count: 0,
            created_at: Utc::now(),
            completed_at: None,
        };
        // 跨 5 module 派发 (per 守门 #9 v3 subprocess 隔离)
        let result = self.dispatch_to_module(module, &step).await;
        self.record_step_result(step, result).await?;
    }
    Ok(())
}
```

### 2.4 step 4: execute (9 SA 跨 SA)

```rust
async fn execute(&mut self) -> Result<(), SagaError> {
    for sa_type in &self.context.sa_types.clone() {
        let step = SagaStep {
            step_id: Uuid::now_v7(),
            saga_id: self.context.saga_id,
            step_name: "execute".to_string(),
            module: "SubAgentOrchestrator".to_string(),
            domain: self.context.from_domain.clone(),
            sa_type: Some(sa_type.clone()),
            idempotency_key: format!("{}:execute:{}", self.context.saga_id, sa_type),
            input: serde_json::json!({"sa_type": sa_type, "context": self.context}),
            output: None,
            status: SagaStepStatus::Running,
            retry_count: 0,
            created_at: Utc::now(),
            completed_at: None,
        };
        // 跑 SA (per doc 07 §6 SA-01..SA-09 实施)
        let result = self.execute_sa(sa_type, &step).await;
        self.record_step_result(step, result).await?;
    }
    Ok(())
}
```

### 2.5 step 5: evaluate (4 effect 维度)

```rust
async fn evaluate(&mut self) -> Result<(), SagaError> {
    for effect_dim in ["dispatch", "context", "trust", "output"] {
        let step = SagaStep {
            step_id: Uuid::now_v7(),
            saga_id: self.context.saga_id,
            step_name: "evaluate".to_string(),
            module: "SubAgentOrchestrator".to_string(),
            domain: self.context.to_domain.clone(),
            sa_type: None,
            idempotency_key: format!("{}:evaluate:{}", self.context.saga_id, effect_dim),
            input: serde_json::json!({"effect_dim": effect_dim}),
            output: None,
            status: SagaStepStatus::Running,
            retry_count: 0,
            created_at: Utc::now(),
            completed_at: None,
        };
        let result = self.evaluate_effect(effect_dim).await;
        self.record_step_result(step, result).await?;
    }
    Ok(())
}
```

### 2.6 step 6: persist (PG 5 张表)

```rust
async fn persist(&mut self) -> Result<(), SagaError> {
    for table in ["agents", "edges", "audit", "template_instances", "unlocks"] {
        let step = SagaStep {
            step_id: Uuid::now_v7(),
            saga_id: self.context.saga_id,
            step_name: "persist".to_string(),
            module: "AgentLease".to_string(),
            domain: self.context.to_domain.clone(),
            sa_type: None,
            idempotency_key: format!("{}:persist:{}", self.context.saga_id, table),
            input: serde_json::json!({"table": table}),
            output: None,
            status: SagaStepStatus::Running,
            retry_count: 0,
            created_at: Utc::now(),
            completed_at: None,
        };
        let result = self.persist_to_table(table).await;
        self.record_step_result(step, result).await?;
    }
    Ok(())
}
```

### 2.7 step 7: compensate (失败回滚)

```rust
async fn compensate(&mut self) -> Result<(), SagaError> {
    // 逆序跑补偿 (per Saga 派生)
    for step in self.context.steps.iter().rev() {
        if step.status == SagaStepStatus::Completed {
            // 跑补偿 (per step.module 派生)
            let result = self.compensate_step(step).await;
            if result.is_err() {
                // 写 audit failed 事件, Mavis 接手人工 review
                self.write_failed_event(step).await?;
            }
        }
    }
    self.context.status = SagaStatus::Compensated;
    Ok(())
}
```

---

## 3. 5 域跨域 Saga 实例

### 3.1 player → economy 玩家代币改动 Saga

```yaml
saga_name: player_to_economy_token_change
from_domain: player
to_domain: economy
sa_types: [SA-08]  # business code generation
steps:
  - name: init
    module: ArgCrate
    actor: player Lead (Mavis 临时代签)
  - name: validate
    module: SubAgentOrchestrator
    check: rbac_player_to_economy  # 跨域 consults
  - name: dispatch
    module: SubAgentOrchestrator
    target: economy Agent (Wallet + Transaction)
  - name: execute
    sa: SA-08
    module: LLMService
    prompt: C-04  # business code generation
  - name: evaluate
    effect_dim: [trust]  # 仅 trust 维度 (effect #3)
    module: SubAgentOrchestrator
  - name: persist
    table: [agents, audit]  # agents (M) + audit (T)
    module: AgentLease
  - name: compensate
    trigger: failed
    module: AgentLease
```

### 3.2 match → social 赛季好友 Saga

```yaml
saga_name: match_to_social_season_friend
from_domain: match
to_domain: social
sa_types: [SA-04, SA-08]  # git-ops + domain-dev
steps:
  - name: init
    module: ArgCrate
  - name: validate
    module: SubAgentOrchestrator
    check: rbac_match_to_social
  - name: dispatch
    module: SubAgentOrchestrator
    target: social Agent (FriendGraph)
  - name: execute
    sa: [SA-04, SA-08]
    module: LLMService
  - name: evaluate
    effect_dim: [dispatch, context]  # effect #1 + #2
    module: SubAgentOrchestrator
  - name: persist
    table: [edges, unlocks]  # edges (M) + unlocks (T)
    module: AgentLease
  - name: compensate
    trigger: failed
    module: AgentLease
```

### 3.3 social → admin 社交合规 Saga

```yaml
saga_name: social_to_admin_compliance
from_domain: social
to_domain: admin
sa_types: [SA-03]  # 5-域-lead-audit
steps:
  - name: init
    module: ArgCrate
  - name: validate
    module: SubAgentOrchestrator
    check: rbac_social_to_admin  # admin 是 I (per RACI §1.2)
  - name: dispatch
    module: SubAgentOrchestrator
    target: admin Agent (AuditAgent + ComplianceAgent)
  - name: execute
    sa: SA-03
    module: LLMService
    prompt: C-05  # audit summary
  - name: evaluate
    effect_dim: [dispatch, trust, output]  # effect #1 + #3 + #4
    module: SubAgentOrchestrator
  - name: persist
    table: [audit]  # audit (T) WORM
    module: AgentLease
  - name: compensate
    trigger: failed
    module: AgentLease
```

### 3.4 admin 跨 5 域 Saga (SA-03 主)

```yaml
saga_name: admin_cross_5_domain_audit
from_domain: admin
to_domain: 5  # 跨 5 域
sa_types: [SA-03]  # 5-域-lead-audit
steps:
  - name: init
    module: ArgCrate
  - name: validate
    module: SubAgentOrchestrator
    check: rbac_admin_cross_5_domain  # admin role=audit (per doc 08 §3.1)
  - name: dispatch
    module: SubAgentOrchestrator
    target: 5 域 Agent
  - name: execute
    sa: SA-03
    module: LLMService
    prompt: C-05
  - name: evaluate
    effect_dim: [dispatch, trust, output]  # 全部 3 维度
    module: SubAgentOrchestrator
  - name: persist
    table: [audit]  # audit (T) WORM
    module: AgentLease
  - name: compensate
    trigger: failed
    module: AgentLease
```

### 3.5 economy → match 比赛奖励 Saga

```yaml
saga_name: economy_to_match_reward
from_domain: economy
to_domain: match
sa_types: [SA-08]  # business code generation
steps:
  - name: init
    module: ArgCrate
  - name: validate
    module: SubAgentOrchestrator
    check: rbac_economy_to_match
  - name: dispatch
    module: SubAgentOrchestrator
    target: match Agent (MatchEngine + Leaderboard)
  - name: execute
    sa: SA-08
    module: LLMService
    prompt: C-04
  - name: evaluate
    effect_dim: [trust]  # 仅 trust
    module: SubAgentOrchestrator
  - name: persist
    table: [agents, audit]
    module: AgentLease
  - name: compensate
    trigger: failed
    module: AgentLease
```

---

## 4. SagaStep idempotency_key 派生 (per WBS v0.18 §6)

### 4.1 idempotency_key 格式 (per WBS v0.18 §6)

`<saga_id>:<module>:<step_name>` 格式:
- `<saga_id>`: SagaContext.saga_id
- `<module>`: 5 module 之一
- `<step_name>`: 7 步之一 (init/validate/dispatch/execute/evaluate/persist/compensate)

### 4.2 幂等保证 (per WBS v0.18 §6 实证)

- 重复跑同一 SagaStep 走 idempotency_key 命中, 跳过 (per P3-A §6 实证)
- 失败重试走 idempotency_key 命中 + retry_count 累加 (per §2.7)
- 补偿事务 (compensate step) 走 reverse idempotency_key, 跟正 step 区分 (per §2.7 派生)

---

## 5. 5 Reducer 跟 Saga 集成 (per ARG.2 + LangGraph 9/3)

### 5.1 Saga 跟 5 Reducer 集成

| Reducer | Saga 集成 | 5 module 命中 |
|---|---|---|
| `arg_agents` | Saga 写 agents 表 → Reducer 同步 (per ARG.2) | ArgCrate |
| `arg_edges` | Saga 写 edges 表 → Reducer 同步 | ArgCrate |
| `arg_trust_scores` | Saga evaluate step (effect #3) → Reducer 同步 | ArgCrate + LLMService |
| `arg_dispatch_overrides` | Saga dispatch step → Reducer 同步 | SubAgentOrchestrator |
| `arg_achievements_unlocked` | Saga execute (SA-09) → Reducer 同步 | SubAgentOrchestrator + LLMService |

### 5.2 Saga 跟 LangGraph 9/3 集成边界 (per 守门 #3 v2)

| 维度 | LangGraph 9/3 (主) | ARG Saga 9/3 (辅) |
|---|---|---|
| 协调协议 | LangGraph subgraph (in-process) | SagaStep (跨 5 module) |
| 状态层 | TopAgentState 5 channel | SagaContext (跨 5 module) |
| 5 module 命中 | (TMO 7 节点) | (跨 5 module) |
| 跨进程 | (in-process) | 跨 5 module |

---

## 6. 验证摘要

### 6.1 docs 阶段

- Saga 协调器架构 (per §1)
- 7 步标准 Saga (per §2)
- 5 域跨域 Saga 实例 (per §3)
- SagaStep idempotency_key (per §4)
- 5 Reducer 集成 (per §5)

### 6.2 Saga 实装 (跨 session 续)

- 7 步 100% 跑通 (per §2.1-§2.7)
- 5 域跨域 Saga 实例 5 个 100% (per §3.1-§3.5)
- SagaStep idempotency_key 100% (per §4)
- 补偿事务 100% (per §2.7)

### 6.3 跨 stage 集成 (per 守门 #1 v3)

- 跟 ARG.1 + ARG.2 + ARG.3 + ARG.4 + ARG.6 命名 100% 一致
- 跨 5 module 协调 (per §1.1)

---

## 7. 已知缺口 (per 守门 #11)

1. **Saga 实装跨 session 续** (per 估 ~5-7M / 4-5 session, 落档后 v0.51+)
2. **5 域跨域 Saga 实例 7 步实证** (per §3, 跨 session 续, 实证需要 18/18 cargo test pass + 跨 session 集成)
3. **SagaStep idempotency_key 实证** (per §4, 跨 session 续, 跟 P3-A §6 实证一致)
4. **补偿事务跨进程 rollback** (per §2.7, 跨 session 续, 实证需要 5 module 跨进程)
5. **Saga 跟 LangGraph 9/3 集成实证** (per §5, 跨 session 续)
6. **ARG.7 跟 ARG.10 frontend 集成** (per v0.51+ 续做项)

---

## 8. 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 永久代签 per 守门 #14 v3 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 5 域 Lead 真人到位前临时代签 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |

---

## 9. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.7 ARG Saga 跨 5 module 协调器 + 7 步标准 Saga + 5 域跨域 Saga 实例 + SagaStep idempotency_key + 补偿事务 + 5 Reducer 集成 | 2026-09-10 01:36 JST 用户发令"继续" + brief v0.50 §14.11 ARG.2-11 docs 阶段派发 + 守门 #14 v3 Mavis 永久代签 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 43 次新事件触发仍允许 |
