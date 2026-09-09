# ARG-ARCH-001 ARG.3 SA-01..SA-09 実装経路 (per brief v0.50 §14.11)

> **Status**: 🟡 Draft v0.1 (per 2026-09-10 01:36 JST 派发 brief v0.50 §14.11 ARG.2-11 docs 阶段)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md) · [arg-bridge §0 入口](./06-arg-02-arg-bridge.md)
> **承接**: 守门 #14 v2 拍板 D + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 + v0.45 ARG.1 docs (`c678db7` 5 doc) + ARG.2 (本批 doc 06)

---

## 0. 目的 (Objective)

承接基本設計書 §1.1 5-tier 架构 Effect Tier + DD §3.5 SubAgentOrchestrator 接口, 本詳細実装設計定义 **ARG.3 `crates/arg-effect` 5 子模块** (ARGDispatchRouter / ARGContextInjector / ARGTrustEngine / ARGOutputEvaluator / ARGAchievementEngine) 的 **9 SA (SA-01..SA-09) 実装経路**, 包含:

1. **5 子模块物理布局** (per BD-AGENT-RELATIONSHIP-001 v0.1 §3 + DD §3.5)
2. **9 SA Type 跨 5 子模块调用关系** (per 要件 §6.1.10 矩阵 + §6.1.11 跨 SA 关系)
3. **8 拓扑成就 Cypher 查询** (per ARGAchievementEngine, 跟 20 成就映射)
4. **10 类 challenges prompt** (per LLMService, 跨 5 域)
5. **L0↔L1 PyO3 桥** (per 9/3 LangGraph 9/3 L0 全体代理 + L1 任务卡子代理)
6. **5 域 Lead 真人到位前 Mavis 临时代签** (per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D 维持)
7. **跨 session 续做边界** (per WBS v0.18 line 663 估 ~22M tokens / 5-7 session, 落档后 v0.51+)

**Token 估算**: ARG.3 docs 阶段 ~2.0M (per v0.45 brief §1 估 22M / 9 docs 均分); Rust 实装跨 session 续 (估 8M / 5-7 session).

---

## 1. 5 子模块物理布局

### 1.1 crates/arg-effect 目录结构

```
crates/arg-effect/
├── Cargo.toml                  # workspace 47 成员 (per AGENTS.md §4.2)
├── src/
│   ├── lib.rs                  # pub mod 5 + 5 协议导出
│   ├── dispatch.rs             # C-17 ARGDispatchRouter (4 effect 维度 #1)
│   ├── context.rs              # C-18 ARGContextInjector (4 effect 维度 #2)
│   ├── trust.rs                # C-19 ARGTrustEngine (4 effect 维度 #3)
│   ├── output.rs               # C-20 ARGOutputEvaluator (4 effect 维度 #4)
│   ├── achievement.rs          # C-21 ARGAchievementEngine (8 拓扑成就)
│   ├── prompts.rs              # 10 类 challenges prompt (跨 5 域)
│   ├── topology_cypher.rs      # 8 拓扑成就 Cypher 查询
│   ├── error.rs                # EffectError 8 类 (DispatchNoRoute / ContextTooLarge / TrustScoreOutOfRange / OutputQualityLow / AchievementNotUnlocked / LLMServiceDown / PyO3BridgeFailed / InternalError)
│   └── tests/                  # 30 IT 嵌入 (per WBS v0.18 §14.11 ARG.6 30 UT 端到端)
│       ├── dispatch_it.rs      # 7 IT (4 effect 维度 #1 跨 9 SA)
│       ├── context_it.rs       # 6 IT (4 effect 维度 #2 跨 9 SA)
│       ├── trust_it.rs         # 6 IT (4 effect 维度 #3 跨 9 SA)
│       ├── output_it.rs        # 6 IT (4 effect 维度 #4 跨 9 SA)
│       └── achievement_it.rs   # 5 IT (8 拓扑 Cypher + 20 成就解锁)
```

**5 子模块依赖关系** (per 基本 §1.1 Effect Tier):

| 子模块 | 上游依赖 | 下游消费者 | 跨 LangGraph 集成 |
|---|---|---|---|
| **C-17 ARGDispatchRouter** | ArgCrate (读 Agent + Edge) + LLMService | SubAgentOrchestrator (派发) | TMO arg_dispatch_overrides |
| **C-18 ARGContextInjector** | ArgCrate (读 Edge.shared_context) | SubAgentOrchestrator (注入) | TMO arg_context_inject_audit |
| **C-19 ARGTrustEngine** | ArgCrate (读 trust_score) + LLMService | SubAgentOrchestrator (加权) | TMO arg_trust_scores |
| **C-20 ARGOutputEvaluator** | LLMService (5 产出评估) | SubAgentOrchestrator (评估) | (TMO 内部) |
| **C-21 ARGAchievementEngine** | ArgCrate (8 拓扑 Cypher) + LLMService (解锁文案) | SubAgentOrchestrator (解锁) | TMO arg_achievements_unlocked |

### 1.2 5 子模块跟 ARG.1 + ARG.2 + ARG.4 集成边界

| 维度 | ARG.1 crates/arg | ARG.2 crates/arg-bridge | ARG.3 crates/arg-effect (本 doc) | ARG.4 crates/api/src/arg |
|---|---|---|---|---|
| 物理 Tier | Data | Bridge | Effect | API + UI |
| 主功能 | Memgraph 持久化 | 5 协议订阅 + 跨 LangGraph | 5 子模块 + 9 SA 实施 | 14 routes (13 REST + 1 WS) |
| 5 module 命中 | ArgCrate + LLMService + AgentLease | AgentLease | SubAgentOrchestrator | AgentRuntime |
| 跨进程 | (in-process) | Memgraph + LangGraph | LangGraph + LLMService | (HTTP REST + WS) |
| Token 估 | 6M | ~7M | ~8M (本 doc 估) | 3M (per ARG.4 实测 1.8-2.2M) |

---

## 2. 9 SA Type 跨 5 子模块调用关系

### 2.1 9 SA 总览 (per 要件 §6.1)

| SA # | SA Type | 4 effect 维度 | 5 子模块命中 | 5 域主战场 | LLMService 参与 |
|---|---|---|---|---|---|
| SA-01 | code-review | #1 dispatch + #3 trust + #4 output | C-17 + C-19 + C-20 | admin | ✅ review 摘要 |
| SA-02 | test-gen | #2 context | C-18 | admin | ❌ |
| SA-03 | 5-域-lead-audit | #1 dispatch + #3 trust + #4 output | C-17 + C-19 + C-20 | admin (主) | ✅ 审计摘要 |
| SA-04 | git-ops | #1 dispatch | C-17 | admin | ❌ |
| SA-05 | doc-sync | (无 4 维度, 仅事件) | (无, 仅 EventWriter) | (跨域) | ❌ |
| SA-06 | refactor | #2 context | C-18 | admin | ✅ 重构建议 |
| SA-07 | db-migration | #1 dispatch | C-17 | admin | ❌ |
| SA-08 | domain-dev | #1 dispatch | C-17 | (5 域) | ✅ 业务代码 |
| SA-09 | achievement-eval | #1 dispatch + #3 trust + 8 拓扑 | C-17 + C-19 + C-21 | social (主) | ✅ 解锁文案 |

### 2.2 9 SA 跟 4 effect 维度矩阵 (per 要件 §6.1.11)

| SA | Dispatch (#1) | Context (#2) | Trust (#3) | Output (#4) |
|---|---|---|---|---|
| SA-01 code-review | ✅ | — | ✅ | ✅ |
| SA-02 test-gen | — | ✅ | — | — |
| SA-03 5-域-lead-audit | ✅ | — | ✅ | ✅ |
| SA-04 git-ops | ✅ | — | — | — |
| SA-05 doc-sync | — | — | — | — |
| SA-06 refactor | — | ✅ | — | — |
| SA-07 db-migration | ✅ | — | — | — |
| SA-08 domain-dev | ✅ | — | — | — |
| SA-09 achievement-eval | ✅ | — | ✅ | — |

### 2.3 9 SA 跟 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v3)

- **player Lead**: SA-08 (业务代码生成) R+A
- **economy Lead**: SA-08 (业务代码生成) R+A
- **match Lead**: SA-04 (git-ops) + SA-08 (业务代码) R+A
- **social Lead**: SA-09 (achievement-eval) R+A
- **admin Lead**: SA-01 + SA-02 + SA-03 + SA-06 + SA-07 R+A (5 SA 全 admin 主战场)
- **跨域协调 (Mavis 接手)**: SA-05 (doc-sync) I (per 5 域 Lead 决策)

---

## 3. 8 拓扑成就 Cypher 查询 (per ARGAchievementEngine C-21)

### 3.1 8 拓扑清单 (per 要件 §6.1.6 + DD §3.5.5)

| # | 拓扑名 | Cypher 模式 | 解锁条件 | 20 成就映射 |
|---|---|---|---|---|
| T-01 | Mentor (师徒) | `(a:Agent {type:'PlayerRole'})-[:MENTORS]->(b:Agent {type:'PlayerRole'})` | 至少 1 mentor edge | A-01 + A-02 |
| T-02 | Peer (平辈) | `(a:Agent)-[:PEER_REVIEW]->(b:Agent)` | 至少 3 peer edge | A-03 + A-04 |
| T-03 | Trust Network (信任网) | `(a:Agent {trust:>=0.7})-[:TRUSTS*2..3]->(b:Agent)` | trust >= 0.7 二跳 | A-05 |
| T-04 | Squad (战队) | `(a:Agent {type:'PlayerRole'})-[:SQUAD_MEMBER_OF]->(team:TeamTemplate)` | 至少 1 squad edge | A-06 + A-07 |
| T-05 | Cross-Domain (跨域) | `(a:Agent {domain:'player'})-[:COLLABORATES]->(b:Agent {domain:'economy'})` | 跨 2+ 域 | A-08 |
| T-06 | Long Chain (长链) | `(a:Agent)-[:MENTORS*4..]->(b:Agent)` | 4+ 跳 mentor 链 | A-09 |
| T-07 | High Trust Cluster (高信任簇) | `(a:Agent {trust:>=0.9})-[:TRUSTS]->(b:Agent {trust:>=0.9})` | trust >= 0.9 簇 | A-10 |
| T-08 | Domain Master (域主) | `(a:Agent {domain:'X'})-[:OWNS]->(b:Agent {domain:'X'})` count >= 10 | 单一域 10+ agent | A-11..A-20 (域特定) |

### 3.2 Cypher 缓存策略 (per ARG.1 C-9 CypherCache)

```rust
// crates/arg-effect/src/topology_cypher.rs
use crate::error::EffectError;
use arg::CypherCache;

pub struct TopologyCypher {
    cache: CypherCache,  // 5min TTL + 256 MB LRU (per ARG.1 C-9)
}

impl TopologyCypher {
    pub async fn check_mentor_topology(&self, agent_id: Uuid) -> Result<bool, EffectError> {
        let cypher = r#"
            MATCH (a:Agent {id: $agent_id})-[:MENTORS]->(b:Agent)
            RETURN COUNT(b) > 0 AS has_mentor
        "#;
        self.cache.query(cypher, &[("agent_id", &agent_id)]).await
    }

    // ... 7 other topology queries 类似
}
```

---

## 4. 10 类 challenges prompt (per LLMService, 跨 5 域)

### 4.1 10 类 challenges 总表 (per 要件 §6.1.5)

| # | Challenge | 5 域命中 | Prompt 模板 | LLMService 调用方 |
|---|---|---|---|---|
| C-01 | code-review 摘要 | admin | "Summarize the following code review..." | C-20 (SA-01) |
| C-02 | peer review 摘要 | admin | "Provide peer review for..." | C-20 (SA-01) |
| C-03 | refactor 建议 | admin | "Suggest refactoring for..." | C-18 (SA-06) |
| C-04 | business code generation | (5 域) | "Generate business code for domain {X}..." | C-17 (SA-08) |
| C-05 | audit summary | admin | "Audit summary for cross-domain ops..." | C-20 (SA-03) |
| C-06 | test case generation | admin | "Generate test cases for..." | C-18 (SA-02) |
| C-07 | db-migration 评估 | admin | "Evaluate db migration impact..." | C-17 (SA-07) |
| C-08 | achievement unlock text | social | "Generate achievement unlock text for..." | C-21 (SA-09) |
| C-09 | trust score 解释 | admin | "Explain trust score change..." | C-19 (SA-03) |
| C-10 | dispatch route 解释 | (5 域) | "Explain dispatch route decision..." | C-17 (SA-04 + SA-08) |

### 4.2 LLMService 集成边界 (per ARG.1 llm 子模块)

```rust
// crates/arg-effect/src/prompts.rs
use arg::LLMService;

pub struct ChallengesPrompt {
    llm: LLMService,
}

impl ChallengesPrompt {
    pub async fn code_review_summary(&self, code: &str) -> Result<String, EffectError> {
        self.llm.complete("C-01", code).await
    }

    pub async fn peer_review(&self, code: &str) -> Result<String, EffectError> {
        self.llm.complete("C-02", code).await
    }

    // ... 8 other challenge prompts 类似
}
```

---

## 5. L0↔L1 PyO3 桥 (per 9/3 LangGraph 9/3)

### 5.1 L0 全体代理 + L1 任务卡子代理 (per 9/3 LangGraph 9/3)

```rust
// crates/arg-effect/src/pyo3_bridge.rs
use pyo3::prelude::*;

pub struct L0L1PyO3Bridge {
    py: Python<'static>,
}

impl L0L1PyO3Bridge {
    /// L0 全体代理 → L1 任务卡子代理 (per 9/3 LangGraph 9/3 2-level hierarchical)
    pub fn spawn_l1_subagent(&self, l0_agent_id: Uuid, task_card_id: Uuid) -> PyResult<L1SubAgent> {
        // 走 Python LangGraph 协议 (per 9/3 LangGraph 9/3)
        let py_l0 = self.py.import("langgraph_runtime")?.getattr("L0Agent")?;
        let l1 = py_l0.call_method1("spawn_l1", (l0_agent_id, task_card_id))?;
        Ok(L1SubAgent::from_py(l1)?)
    }

    /// L1 任务卡子代理 → L0 全体代理 (跨任务汇总)
    pub fn aggregate_to_l0(&self, l1_outputs: Vec<L1Output>) -> PyResult<L0Summary> {
        let py_l0 = self.py.import("langgraph_runtime")?.getattr("L0Agent")?;
        let summary = py_l0.call_method1("aggregate", (l1_outputs,))?;
        Ok(L0Summary::from_py(summary)?)
    }
}
```

### 5.2 跟 LangGraph 9/3 集成边界 (per 守门 #3 v2)

| 维度 | LangGraph 9/3 (主) | ARG 9/3 (辅) |
|---|---|---|
| 2-level hierarchical | TMO 7 节点 (主) | PyO3 桥 (辅, L0↔L1) |
| 9 SA Type | LangGraph subgraph (in-process) | (跨 ARG 5 子模块调用) |
| 5 Reducer | TopAgentState 5 channel | (ARG.2 5 Reducer 共享) |
| 5 module 命中 | (TMO 7 节点) | SubAgentOrchestrator (主) |

---

## 6. 9 SA 实施路径 (per WBS v0.18 §14.11 ARG.3)

### 6.1 SA-01 code-review 实施

```rust
// crates/arg-effect/src/dispatch.rs
pub async fn sa01_code_review(&self, agent_id: AgentId) -> Result<ReviewSummary, EffectError> {
    // 1. Dispatch (effect #1)
    let route = self.dispatch_router.find_route(agent_id, "SA-01").await?;
    // 2. Trust (effect #3)
    let trust = self.trust_engine.get_trust(agent_id).await?;
    // 3. LLMService code-review 摘要 (C-01)
    let review = self.llm.code_review_summary(&route.code_snippet).await?;
    // 4. Output (effect #4)
    self.output_evaluator.evaluate(&review, trust).await?;
    Ok(ReviewSummary { agent_id, review, trust })
}
```

### 6.2 SA-02 test-gen 实施 (effect #2 唯一)

```rust
// crates/arg-effect/src/context.rs
pub async fn sa02_test_gen(&self, agent_id: AgentId) -> Result<TestCases, EffectError> {
    // 1. Context (effect #2)
    let context = self.context_injector.inject(agent_id, "test-gen").await?;
    // 2. 派发 L1 任务卡子代理 (per §5.1 PyO3 桥)
    let l1 = self.l0_l1_bridge.spawn_l1_subagent(agent_id, context.task_card_id).await?;
    // 3. 生成 test cases
    Ok(l1.test_cases().await?)
}
```

### 6.3 SA-03 5-域-lead-audit 实施 (admin 主战场)

```rust
// crates/arg-effect/src/output.rs
pub async fn sa03_5_domain_lead_audit(&self, cross_domain_ops: Vec<CrossDomainOp>) -> Result<AuditSummary, EffectError> {
    // 1. Dispatch (effect #1)
    let route = self.dispatch_router.find_audit_route().await?;
    // 2. Trust (effect #3) - 跨 5 域 Lead 信任分
    let trusts = self.trust_engine.get_cross_domain_trusts().await?;
    // 3. LLMService audit summary (C-05)
    let summary = self.llm.audit_summary(&cross_domain_ops, &trusts).await?;
    // 4. Output (effect #4)
    self.output_evaluator.evaluate_audit(&summary).await?;
    Ok(AuditSummary { summary, trusts })
}
```

### 6.4 SA-04 git-ops 实施 (effect #1 唯一)

```rust
pub async fn sa04_git_ops(&self, ops_request: GitOpsRequest) -> Result<GitOpsResult, EffectError> {
    // 1. Dispatch (effect #1)
    let route = self.dispatch_router.find_git_ops_route(ops_request).await?;
    // 2. 派发 ops sub-agent
    let l1 = self.l0_l1_bridge.spawn_l1_subagent(route.from_agent_id, route.task_card_id).await?;
    // 3. 跑 git ops
    Ok(l1.git_ops(ops_request).await?)
}
```

### 6.5 SA-05 doc-sync 实施 (无 4 维度, 仅事件)

```rust
pub async fn sa05_doc_sync(&self, doc_event: DocSyncEvent) -> Result<(), EffectError> {
    // 1. 走 EventWriter 写 arg_bridge_events topic
    // 2. Memgraph subscription 跨进程 fanout
    // 3. 5 域 Lead 决策 I (per §2.3 跨域协调 Mavis 接手)
    self.event_writer.write_arg_event(&doc_event).await?;
    Ok(())
}
```

### 6.6 SA-06 refactor 实施 (effect #2)

```rust
pub async fn sa06_refactor(&self, code: &str) -> Result<RefactorSuggestion, EffectError> {
    // 1. Context (effect #2)
    let context = self.context_injector.inject(code_agent_id, "refactor").await?;
    // 2. LLMService refactor 建议 (C-03)
    let suggestion = self.llm.refactor_suggestion(&code).await?;
    Ok(RefactorSuggestion { context, suggestion })
}
```

### 6.7 SA-07 db-migration 实施 (effect #1)

```rust
pub async fn sa07_db_migration(&self, migration_request: MigrationRequest) -> Result<MigrationResult, EffectError> {
    // 1. Dispatch (effect #1)
    let route = self.dispatch_router.find_migration_route(migration_request).await?;
    // 2. LLMService db-migration 评估 (C-07)
    let eval = self.llm.db_migration_eval(&migration_request).await?;
    // 3. 派发 migration sub-agent
    let l1 = self.l0_l1_bridge.spawn_l1_subagent(route.from_agent_id, route.task_card_id).await?;
    Ok(l1.db_migration(migration_request, eval).await?)
}
```

### 6.8 SA-08 domain-dev 实施 (effect #1 + 跨 5 域)

```rust
pub async fn sa08_domain_dev(&self, dev_request: DomainDevRequest) -> Result<DomainDevResult, EffectError> {
    // 1. Dispatch (effect #1) - 跨 5 域 Lead
    let route = self.dispatch_router.find_dev_route(&dev_request).await?;
    // 2. LLMService business code generation (C-04)
    let code = self.llm.business_code_generation(&dev_request).await?;
    // 3. 派发 dev sub-agent
    let l1 = self.l0_l1_bridge.spawn_l1_subagent(route.from_agent_id, route.task_card_id).await?;
    Ok(l1.domain_dev(dev_request, code).await?)
}
```

### 6.9 SA-09 achievement-eval 实施 (effect #1 + #3 + 8 拓扑)

```rust
pub async fn sa09_achievement_eval(&self, agent_id: AgentId) -> Result<Vec<AchievementUnlock>, EffectError> {
    // 1. Dispatch (effect #1)
    let route = self.dispatch_router.find_achievement_route(agent_id).await?;
    // 2. Trust (effect #3) - 信任分
    let trust = self.trust_engine.get_trust(agent_id).await?;
    // 3. 8 拓扑 Cypher 检查 (per §3.1)
    let mut unlocks = Vec::new();
    for topology in &topology_set() {  // T-01..T-08
        if self.topology_cypher.check_topology(topology, agent_id).await? {
            let achievement = self.find_achievement_for_topology(topology);
            let text = self.llm.achievement_unlock_text(agent_id, achievement).await?;
            unlocks.push(AchievementUnlock { agent_id, achievement, text });
        }
    }
    Ok(unlocks)
}
```

---

## 7. 验证摘要 (Validation)

### 7.1 docs 阶段

- 5 子模块物理布局明确 (per §1.1 目录结构)
- 9 SA 实施路径完整 (per §6.1-§6.9)
- 8 拓扑成就 Cypher 模式清晰 (per §3.1)
- 10 类 challenges prompt 跨 5 域 (per §4.1)
- L0↔L1 PyO3 桥 (per §5.1)
- 5 域 Lead Mavis 临时代签 (per §2.3)

### 7.2 Rust 实装 (跨 session 续)

- 30 IT 嵌入 (`crates/arg-effect/src/tests/{dispatch,context,trust,output,achievement}_it.rs` 7+6+6+6+5=30)
- 跨 5 子模块 100% test coverage
- 9 SA 实施路径 100% 一致 (per §6.1-§6.9)

### 7.3 跟 ARG.1 + ARG.2 + ARG.4 集成

- 5 module 命名 / 9 SA 命名 / 4 effect 维度命名 / 5 域命名 100% 一致
- 跨 stage token 实测: ARG.1 ~4.5M / ARG.4 ~1.8M / ARG.2 估 ~5-7M / ARG.3 估 ~6-8M

---

## 8. 已知缺口 (per 守门 #11)

1. **Rust 实装跨 session 续** (per 估 ~8M / 5-7 session, 落档后 v0.51+)
2. **8 拓扑成就 Cypher 实测** (per §3.1, 跨 session 续, 需要 Memgraph 启动后跑 5 IT 实证)
3. **10 类 challenges prompt 实证** (per §4.1, 跨 LLMService 集成, 跨 session 续)
4. **L0↔L1 PyO3 桥实测** (per §5.1, 跨 LangGraph 9/3 集成, 跨 session 续)
5. **5 域 Lead 真人到位前 Mavis 临时代签** 维持 (per §2.3, 真人到位后追溯签字 per 守门 #1 禁回溯叙事)
6. **ARG.3 跟 ARG.10 frontend 集成** (per v0.51+ 续做项)

---

## 9. 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 永久代签 per 守门 #14 v3 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 5 域 Lead 真人到位前临时代签 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.3 crates/arg-effect 5 子模块 (C-17/C-18/C-19/C-20/C-21) + 9 SA 实施路径 (SA-01..SA-09) + 8 拓扑成就 Cypher + 10 类 challenges prompt + L0↔L1 PyO3 桥 + 5 域 Lead Mavis 临时代签 | 2026-09-10 01:36 JST 用户发令"继续" + brief v0.50 §14.11 ARG.2-11 docs 阶段派发 + 守门 #14 v3 Mavis 永久代签 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 43 次新事件触发仍允许 |
