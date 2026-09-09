# ARG-ARCH-001 Agent Relationship Graph (ARG) Skill Diagrams 关系图

> **Status**: 🟡 Draft v0.1 (per 2026-09-09 21:18 JST 派发 brief v0.45 §14.11 ARG.1 文档完整化)
> **Created**: 2026-09-09
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md)
> **注**: 本文档使用 text-based 关系图 (无 mermaid 依赖), 跨 5 文档严格一致.

---

## 0. 目的 (Objective)

定义 ARG 阶段 **5 module × 9 SA × 5 域** 的 text-based 关系图, 跨 5 文档严格一致. 关系图覆盖:
1. 5 module 跨模块调用关系
2. 9 SA 跨 SA 协作关系
3. 5 域 × 5 module 跨域 RACI 关系
4. 4 effect 维度真实落地关系
5. 5 协议跨 module 推送关系

---

## 1. 5 Module 跨模块调用关系图 (per 基本 §1.1 + 詳細 §5.2)

```
                ┌──────────────────────────┐
                │  User (UI Browser)       │
                └────────────┬─────────────┘
                             │ HTTPS REST + WSS
                             ▼
        ┌──────────────────────────────────────────┐
        │  AgentRuntime                            │
        │  (UI Tier: 5 组件 + zustand 5 channel)   │
        │  (API Tier: 13 REST + 1 WebSocket)       │
        └────────────┬─────────────────────────────┘
                     │ HTTP POST /api/arg/edges
                     │ HTTP GET  /api/arg/agents
                     │ WS        /ws/arg/events
                     ▼
        ┌──────────────────────────────────────────┐
        │  ArgCrate (Data Tier 核心)               │
        │  6 子模块:                               │
        │  - models 10 (Agent/Edge/Template/...)  │
        │  - client 3  (MemgraphClient/Cache/...)  │
        │  - ops 5     (AgentNode/EdgeOps/...)    │
        │  - query 3   (Behavior/Output/Topology)  │
        │  - llm       (5 LLM 用途, per LLMService)│
        │  - error     (10 类错误枚举)             │
        └────┬──────────────────────────┬─────────┘
             │                          │
             │ Bolt 7687                │ in-process 推
             │                          │
             ▼                          ▼
    ┌────────────────┐         ┌──────────────────────┐
    │  Memgraph      │         │  AgentLease          │
    │  (Bolt 7687)   │◀────────│  (Data + Bridge)     │
    │  5 表:         │ Bolt    │  4 Bridge 子模块:    │
    │  - agents (M)  │ subs    │  - MemgraphEventList │
    │  - edges (M)   │         │  - LangGraphStateUpd │
    │  - audit (T)   │         │  - PeriodFlushWorker │
    │  - template_   │         │  - OfflineQueue      │
    │    instances   │         │  (sled 离线降级)     │
    │    (W TTL 30d) │         │  AgentLease 11 字段  │
    │  - unlocks (T) │         │  (per ADR-0030)      │
    └────────────────┘         └──────────┬───────────┘
                                         │ in-process 推
                                         │ + 周期 flush 30s
                                         ▼
                          ┌──────────────────────────┐
                          │  SubAgentOrchestrator   │
                          │  (Effect Tier)          │
                          │  5 子模块:              │
                          │  - ARGDispatchRouter    │
                          │    (effect #1)          │
                          │  - ARGContextInjector   │
                          │    (effect #2)          │
                          │  - ARGTrustEngine       │
                          │    (effect #3)          │
                          │  - ARGOutputEvaluator   │
                          │    (effect #4)          │
                          │  - ARGAchievementEngine │
                          │    (8 拓扑成就)         │
                          └──────────┬──────────────┘
                                     │ LLM 调用
                                     ▼
                          ┌──────────────────────────┐
                          │  LLMService (Data 子模块)│
                          │  5 LLM 用途:            │
                          │  - 挑战双向论证 prompt  │
                          │  - peer review 摘要     │
                          │  - 8 拓扑 Cypher 模板   │
                          │  - 解锁文案生成         │
                          │  - ARG 事件摘要         │
                          │  跟 L0 LLM Pool 共享    │
                          │  (per ADR-0045)         │
                          └──────────────────────────┘
```

---

## 2. 9 SA 跨 SA 协作关系图 (per 要件 §6.1.10 矩阵)

```
                 SA-01 code-review
                       │
        ┌──────────────┼──────────────┐
        │ (1 触发)     │ (2 强协作)   │ (2 强协作)
        ▼              ▼              ▼
   SA-02 test-gen  SA-06 refactor  SA-04 git-ops
        │              │              │
        │ (1)          │ (1)          │ (2)
        ▼              ▼              ▼
   SA-08 domain-dev  SA-01 code-review  SA-05 doc-sync
        │              ▲              ▲
        │ (1)          │ (1)          │ (1)
        ▼              │              │
   SA-07 db-migration │              │
        │              │              │
        │ (1)          │              │
        ▼              │              │
   SA-01 code-review  │              │
                       │              │
                       │              │
   SA-03 5-域-lead-audit ◀──────────┘
        │
        │ (2 强协作: 审计 fix)
        ▼
   SA-04 git-ops
        │
        │ (2 强协作: DB 审计)
        ▼
   SA-07 db-migration
        │
        │ (2 强协作: 产出审计)
        ▼
   SA-09 achievement-eval
        │
        │ (1 触发: 行为 review)
        ▼
   SA-01 code-review

   跨 SA-10 task-orchestrator (per ADR-0046 v0.2):
       SA-09 ── (1 触发) ──▶ SA-10 (TMO 7 节点 M-N7 metadata)
```

---

## 3. 5 域 × 5 Module 跨域 RACI 关系图 (per RACI §1.1)

```
                ┌─────────────┐
                │  player     │
                │  Lead       │
                │  (Mavis 代签)│
                └──────┬──────┘
                       │
       ┌───────────────┼───────────────┐
       │ R+A           │ C             │ I
       ▼               ▼               ▼
   ┌────────┐      ┌────────┐      ┌────────┐
   │ArgCrate│      │SubAgent│      │Agent   │
   │        │      │Orchestr│      │Lease   │
   │R+A持久化│     │C业务咨询│     │I事件触发│
   └────┬───┘      └───┬────┘      └────────┘
        │              │
        │              │
        ▼              ▼
   ┌────────────────────────┐
   │  AgentRuntime          │
   │  (R+A, UI/API)         │
   │  player + economy 主要 │
   │  入口                   │
   └────────────────────────┘

   ════════════════════════════════════════

                ┌─────────────┐
                │  match      │
                │  Lead       │
                │  (Mavis 代签)│
                └──────┬──────┘
                       │
       ┌───────────────┼───────────────┐
       │ R+A           │ R+A           │ I
       ▼               ▼               ▼
   ┌────────┐      ┌────────┐      ┌────────┐
   │ArgCrate│      │SubAgent│      │Agent   │
   │        │      │Orchestr│      │Lease   │
   │R+A持久化│     │R+A     │      │I事件触发│
   │        │      │Dispatch│      │        │
   │        │      │路由主战场│     │        │
   └────────┘      └────────┘      └────────┘

   ════════════════════════════════════════

                ┌─────────────┐
                │  social     │
                │  Lead       │
                │  (Mavis 代签)│
                └──────┬──────┘
                       │
       ┌───────────────┼───────────────┐
       │ R+A           │ R+A           │ I
       ▼               ▼               ▼
   ┌────────┐      ┌────────┐      ┌────────┐
   │ArgCrate│      │SubAgent│      │Agent   │
   │        │      │Orchestr│      │Lease   │
   │R+A持久化│     │R+A     │      │I事件触发│
   │        │      │Context │      │        │
   │        │      │注入主战场│     │        │
   └────────┘      └────────┘      └────────┘

   ════════════════════════════════════════

                ┌─────────────┐
                │  admin      │
                │  Lead       │
                │  (Mavis 代签)│
                └──────┬──────┘
                       │
       ┌───────────────┼───────────────┬───────────────┐
       │ R+A           │ R+A           │ R+A           │ R+A
       ▼               ▼               ▼               ▼
   ┌────────┐      ┌────────┐      ┌────────┐      ┌────────┐
   │ArgCrate│      │SubAgent│      │LLM     │      │Agent   │
   │        │      │Orchestr│      │Service │      │Lease   │
   │R+A持久化│     │R+A     │      │R+A     │      │R+A     │
   │audit   │      │Trust + │      │审计摘要│      │audit   │
   │trail   │      │Output  │      │        │      │trail   │
   │        │      │评估主战场│     │        │      │        │
   └────────┘      └────────┘      └────────┘      └────────┘
                                                       │
                                                       ▼
                                                  ┌────────┐
                                                  │Agent   │
                                                  │Runtime │
                                                  │R+A     │
                                                  │admin UI│
                                                  └────────┘
```

---

## 4. 4 Effect 维度真实落地关系图 (per 要件 §1.1 + 詳細 §2.3)

```
   ════════════════════ Effect #1: Dispatch 路由 ════════════════════

   User (UI)
      │
      │ 创建 Lead-Worker delegates_to 关系
      ▼
   AgentRuntime (UI)
      │
      │ HTTP POST /api/arg/edges
      ▼
   ArgCrate.create_edge()
      │
      │ MemgraphClient.execute_cypher()
      ▼
   Memgraph (Bolt)
      │
      │ event
      ▼
   MemgraphEventListener (AgentLease)
      │
      │ EventBus
      ▼
   PeriodFlushWorker (AgentLease)
      │
      │ 30s 周期 flush
      ▼
   LangGraphStateUpdater (AgentLease)
      │
      │ 5 Reducer
      ▼
   LangGraph TopAgentState
      │
      │ dispatch event
      ▼
   ARGDispatchRouter (SubAgentOrchestrator)
      │
      │ 派发到 worker_id (按 weight DESC)
      ▼
   Worker (SA-08 domain-dev 等)
      │
      │ 失败?
      ├── 是 ──▶ ARGDispatchRouter 触发 stand_in_for fallback
      │              │
      │              ▼
      │           stand_in worker
      │              │
      │              ▼
      │           Worker 继续
      │
      └── 否 ──▶ Worker 完成任务
                     │
                     ▼
                  arg_dispatch_route 协议
                  (per 要件 §4.1 F-15)
                     │
                     ▼
                  AgentLease 事件推送
                     │
                     ▼
                  AgentRuntime WS 推送
                     │
                     ▼
                  User 看到任务完成

   ════════════════════ Effect #2: 上下文共享 ════════════════════

   mentee 任务卡创建
      │
      ▼
   ARGContextInjector (SubAgentOrchestrator)
      │
      │ 1. 拉 mentors 历史决策 (Cypher 查询)
      ▼
   ArgCrate.query_cypher()
      │
      │ 返回 history_decisions
      ▼
   ARGContextInjector
      │
      │ 2. LLMService 摘要
      ▼
   LLMService.summarize_peer_review()
      │
      │ 返回 summary (100 字)
      ▼
   ARGContextInjector
      │
      │ 3. token 节省 20-40% (per G-14 实证)
      ▼
   arg_context_inject 协议 (F-16)
      │
      ▼
   AgentLease 推送
      │
      ▼
   mentee 看到 mentors 历史摘要 (省 token 20-40%)

   ════════════════════ Effect #3: 信任度加权 ════════════════════

   trust_score 评估触发
      │
      ▼
   ARGTrustEngine (SubAgentOrchestrator)
      │
      │ 1. 读 trust_score + trusts 边 weight
      ▼
   ArgCrate.get_agent() + query_cypher()
      │
      │ 2. 评估
      ▼
   if trust_score ≥ 0.8 AND trusts.weight ≥ 0.7
      │
      ├── 是 ──▶ 跳过 verify (省 15% token)
      │
      └── 否 ──▶ 走正常 verify 流程
                     │
                     ▼
                  arg_trust_score_update 协议 (F-12)
                     │
                     ▼
                  AgentLease 推送
                     │
                     ▼
                  User 看到 trust_score 变更

   ════════════════════ Effect #4: 产出评估 ════════════════════

   Worker 产出 (代码 / 决策 / 测试 / 迁移 / refactor)
      │
      ▼
   ARGOutputEvaluator (SubAgentOrchestrator)
      │
      │ 1. challenges 强制双向论证
      ▼
   LLMService.generate_challenge_prompt()
      │
      │ 5 decision_type × 2 trust_tier 组合 (10 套)
      ▼
   双向论证 prompt
      │
      │ 2. peer_reviews 阈值 ≥ 0.8
      ▼
   LLMService.summarize_peer_review()
      │
      │ 3. 评估结果
      ▼
   if peer_review_score ≥ 0.8
      │
      ├── 是 ──▶ 产出通过
      │
      └── 否 ──▶ 产出回退 + challenge_fail 扣 trust_score
                     │
                     ▼
                  arg_trust_score_update 协议 (F-12)
```

---

## 5. 5 协议跨 Module 推送关系图 (per 要件 §4.1 F-12..F-16)

```
   ┌────────────────────────────────────────────────────────────┐
   │              5 协议跨 Module 推送拓扑                       │
   └────────────────────────────────────────────────────────────┘

   ┌────────────────────┐
   │  ArgCrate          │
   │  (Data Tier)       │
   │  event:            │
   │  - agent.created   │
   │  - agent.updated   │
   │  - edge.created    │
   │  - edge.updated    │
   │  - edge.archived   │
   │  - achievement.    │
   │    unlocked        │
   └─────────┬──────────┘
             │
             │ 5 类事件
             ▼
   ┌────────────────────────────────────────────┐
   │  AgentLease (Data + Bridge Tier)            │
   │  4 Bridge 子模块:                           │
   │  - MemgraphEventListener (Bolt subs)        │
   │  - LangGraphStateUpdater (5 Reducer)        │
   │  - PeriodFlushWorker (30s)                  │
   │  - OfflineQueue (sled 5min 重试)             │
   │  AgentLease 11 字段 (per ADR-0030)          │
   └──────┬─────────────────┬──────────────────┬─┘
          │                 │                  │
          │ 推送            │ in-process 推    │ 周期 flush
          │                 │                  │
          ▼                 ▼                  ▼
   ┌────────────┐    ┌──────────────┐    ┌──────────────┐
   │SubAgentOr- │    │ LangGraph    │    │ 5 协议 (5 类)│
   │chestrator  │    │ TopAgent     │    │ - arg_edge_  │
   │            │    │ State        │    │   changed   │
   │ 监听 + 处理│    │ 5 channel +  │    │ - arg_dispa- │
   │ 4 effect   │    │ 5 Reducer    │    │   tch_route │
   │ 维度       │    │              │    │ - arg_conte- │
   │            │    │              │    │   xt_inject │
   └─────┬──────┘    └──────┬───────┘    │ - arg_trust_ │
         │                  │            │   score_upd │
         │                  │            │ - arg_achie- │
         │                  │            │   vement_   │
         │                  │            │   unlocked  │
         │                  │            └──────┬──────┘
         │                  │                   │
         │                  │                   │ 5 协议推送
         │                  │                   ▼
         │                  │            ┌──────────────┐
         │                  │            │ AgentLease   │
         │                  │            │ 推送         │
         │                  │            └──────┬───────┘
         │                  │                   │
         │                  │                   │ WS 推送
         │                  │                   ▼
         │                  │            ┌──────────────┐
         │                  │            │ AgentRuntime │
         │                  │            │ (UI Tier)    │
         │                  │            │ - 5 组件     │
         │                  │            │ - zustand    │
         │                  │            │   5 channel  │
         │                  │            │ - WebSocket  │
         │                  │            │   /ws/arg/   │
         │                  │            │   events     │
         │                  │            └──────┬───────┘
         │                  │                   │
         │                  │                   │ UI 通知
         │                  │                   ▼
         │                  │            ┌──────────────┐
         │                  │            │ User (UI)    │
         │                  │            │ 看到:        │
         │                  │            │ - 关系变更   │
         │                  │            │ - 派发路由   │
         │                  │            │ - 上下文注入 │
         │                  │            │ - 信任度变更 │
         │                  │            │ - 成就解锁   │
         │                  │            └──────────────┘
         │                  │
         │                  │
         │ 派发 LLM 摘要   │ TopAgent State 扩展
         ▼                  ▼
   ┌────────────┐    ┌──────────────┐
   │ LLMService │    │ LangGraph    │
   │ 5 用途:    │    │ TopAgent     │
   │ - 挑战     │    │ State:       │
   │   双向论证 │    │ + arg_agents │
   │ - peer     │    │ + arg_edges  │
   │   review   │    │ + arg_trust_ │
   │   摘要     │    │   scores     │
   │ - 8 拓扑   │    │ + arg_dispa- │
   │   Cypher   │    │   tch_overri │
   │ - 解锁文案 │    │   des        │
   │ - ARG 事件 │    │ + arg_achie- │
   │   摘要     │    │   vements_   │
   │            │    │   unlocked   │
   └────────────┘    └──────────────┘
```

---

## 6. 5 Module 落地路径关系图 (per 基本 §8)

```
   ┌──────────────────────────────────────────────────────────────┐
   │  ARG 阶段 11 子项 (ARG.1..ARG.11) 落地路径                    │
   └──────────────────────────────────────────────────────────────┘

   ARG.1 (2026-09-09 05:00 JST 收官, commit 43c1f0c)
   ┌──────────────────────────────────────────────────────────┐
   │  crates/arg/ 6 子模块骨架 + 32 UT + 2 脚本               │
   │  → ArgCrate + LLMService + AgentLease (Data 部分)         │
   │  5 守门 0 err + 33 UT 100% pass + 44 文件 / 4206 行       │
   │  2 脚本: memgraph_setup.py + arg_seed.py                  │
   │  1 报告: PHASE-ARG-01-IMPL-REPORT.md 19.4KB                │
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.2 (P3-C W2 落地, per DD §3.1 + §4.10-4.11)
   ┌──────────────────────────────────────────────────────────┐
   │  crates/arg-bridge/ 4 子模块                              │
   │  → AgentLease (Bridge 部分)                                │
   │  同步桥协议: Bolt subs + EventBus + 30s flush + sled 降级 │
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.3 (P3-C W3 落地, per DD §3.1 + §4.5-4.9)
   ┌──────────────────────────────────────────────────────────┐
   │  crates/arg-effect/ 5 子模块                              │
   │  → SubAgentOrchestrator                                    │
   │  4 effect 维度真实落地 (Dispatch/Context/Trust/Output)    │
   │  8 拓扑成就 Cypher 模板 + 10 套 challenges prompt         │
   │  L0↔L1 PyO3 协议                                          │
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.4 (2026-09-09 05:31 JST 收官, commit 6e2cda6)
   ┌──────────────────────────────────────────────────────────┐
   │  crates/api/src/arg/ 13 REST + 1 WebSocket + RLS 13 类   │
   │  → AgentRuntime (REST/WS 部分)                            │
   │  5 守门 0 err + 24/24 cargo test + 10/10 Python IT        │
   │  14 routes 13 REST + 1 WS 严格按 axum 0.8 (ADR-0048)      │
   │  1 脚本: arg_api_test.py 580 行                            │
   │  1 报告: PHASE-ARG-04-IMPL-REPORT.md 20.2KB                │
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.5 (P3-C W4 落地, per DD §4.13)
   ┌──────────────────────────────────────────────────────────┐
   │  frontend/src/app/agent-relationships/ 5 UI 组件          │
   │  → AgentRuntime (UI 部分)                                 │
   │  RelationshipEditor / RelationshipView / AchievementWall  │
   │  EdgeTypeSelector / TemplateGallery                       │
   │  zustand useARGStore 5 channel                            │
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.6 (P3-D W1 落地, per DD §10.1)
   ┌──────────────────────────────────────────────────────────┐
   │  30 UT 完整落地                                            │
   │  crates/arg 24 + crates/arg-bridge 10 + crates/arg-effect │
   │  18, 但去重后 = 52 UT per DD §10.1                        │
   │  cargo test -p star-arg --lib -j 4 100% pass              │
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.7 (P3-D W2 落地, per DD §10.2-§10.4)
   ┌──────────────────────────────────────────────────────────┐
   │  10 IT + 8 E2E + 4 PT 端到端实装                            │
   │  拖拽建边 / Dispatch / Consults / 协作並行 / Stand-in    │
   │  fallback / Trust skip verify / 成就解锁 / 离线重连        │
   │  4 PT: 边 P95<200ms / Cypher<500ms / 事件<100ms / 成就<1s│
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.8 (P3-E W1 落地, per BD §7.4 + DD §3.2.4)
   ┌──────────────────────────────────────────────────────────┐
   │  7 行为 + 5 产出成就 evaluator 落地 (8 拓扑已在 ARG.3)      │
   │  3 evaluator 并行 + 异步触发 + SSE 推送                   │
   │  20 成就完整闭环 (8 拓扑 + 7 行为 + 5 产出)               │
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.9 (P3-E 收官时落地, per AGENTS.md §3 7 段结构)
   ┌──────────────────────────────────────────────────────────┐
   │  PHASE-ARG-IMPL-REPORT.md v0.1 实施报告                    │
   │  5 守门维度实证 + 跨 session 续做清单                     │
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.10 (5 域 Lead 真人到位后 DDD Review, per DD §13 G-9/G-4/G-10)
   ┌──────────────────────────────────────────────────────────┐
   │  DDD Review (G-9 跟 TMO 9 节点边界 + G-4 trusts 安全审计 │
   │  + G-10 Schema V2 迁移路径)                                │
   │  5 域 Lead 真人到位 (per 守门 #14 v3 暂时不追踪) 后       │
   │  DDD Review 拍板, 缺口 G-9 关键 (ARG 跟 TMO 任务卡 DAG 边界)│
   └──────────────────────────────────────────────────────────┘
                           │
                           ▼
   ARG.11 (真人到位流程, per 守门 #14 v3 暂时不追踪)
   ┌──────────────────────────────────────────────────────────┐
   │  5 域 Lead 真人到位 (追溯签字覆盖修订历史)                  │
   │  跨 session 续, 真人到位后追溯签字覆盖 Mavis 临时代签     │
   │  (per 守门 #1 禁回溯叙事)                                  │
   └──────────────────────────────────────────────────────────┘
```

---

## 7. 5 Module 跨 Module 集成关系 (per 基本 §1.3 + 詳細 §1.2.1-§1.2.2)

```
   ┌──────────────────────────────────────────────────────────┐
   │  创建关系 (S-01) 跨 module 集成                           │
   └──────────────────────────────────────────────────────────┘

   User
     │
     │ 1 click 拖拽 source
     │ 2 click 选 relationship_type
     │ 3 click 提交
     ▼
   AgentRuntime.UI.RelationshipEditor
     │
     │ HTTP POST /api/arg/edges
     ▼
   AgentRuntime.REST.create_edge
     │
     │ ArgCrate.create_edge()
     ▼
   ArgCrate.1 RLS 检查
     │ - actor.rls_class.can_create_edge()
     │ - 跨域约束: 5 域之间禁止 delegates_to (per 守门 #3 v2)
     │ - 循环检测: delegates_to / reports_to 不允许成环
     │ - 权重范围: 0.0 ≤ weight ≤ 1.0
     ▼
   ArgCrate.2 Memgraph 持久化
     │ - Bolt 7687 execute_cypher
     │ - 5 表 W/T/M 严格 (per 守门 #13)
     ▼
   ArgCrate.3 Cypher 缓存失效
     │ - 5min TTL + 256 MB LRU
     ▼
   ArgCrate.4 事件写入 (per AgentLease)
     │ - ARGEvent { id, event_type, source_id, target_id, payload, timestamp }
     │ - 物理写入 audit 表 (append-only, per 守门 #13 (b))
     ▼
   AgentLease.MemgraphEventListener
     │ - Bolt subscription 监听
     │ - EventBus 推送
     ▼
   AgentLease.PeriodFlushWorker
     │ - 30s 周期 flush (Memgraph → ArgCrate 缓存)
     │ - 离线降级: sled 5min 重试
     ▼
   AgentLease.LangGraphStateUpdater
     │ - 5 Reducer 跨 LangGraph TopAgentState
     │ - + arg_agents / + arg_edges / + arg_trust_scores
     │ - + arg_dispatch_overrides / + arg_achievements_unlocked
     ▼
   LangGraph TopAgentState
     │ - dispatch event
     ▼
   SubAgentOrchestrator.ARGDispatchRouter
     │ - 监听 arg_dispatch_route 协议
     │ - 派发到 worker_id (按 weight DESC)
     ▼
   SubAgentOrchestrator.ARGAchievementEngine (if applicable)
     │ - 8 拓扑成就评估 (per G-6)
     │ - if 拓扑满足 -> unlock
     ▼
   SubAgentOrchestrator.ARGTrustEngine (if applicable)
     │ - trust_score 更新 (per ADR-0030)
     │ - if trust_score ≥ 0.8 + trusts.weight ≥ 0.7 -> 跳过 verify
     ▼
   SubAgentOrchestrator.ARGContextInjector (if applicable)
     │ - mentee 启动拉 mentors 历史
     │ - LLMService.summarize_peer_review() (省 token 20-40%)
     ▼
   AgentLease.事件推送
     │ - 5 协议 (per 要件 §4.1 F-12..F-16)
     │ - WebSocket 推送 /ws/arg/events
     ▼
   AgentRuntime.UI.AchievementWall (if applicable)
     │ - zustand 5 channel (events)
     │ - User 看到:
     │   - 关系创建成功
     │   - 成就解锁通知 (e.g. "🎉 Triangle 拓扑成就!")
     │   - 信任度变更
     │   - 上下文注入 (省 token 提示)
     ▼
   User 看到完整结果
```

---

## 8. 9 SA 跨 SA + 5 module 关系图 (per 要件 §6.1.11)

```
   ┌──────────────────────────────────────────────────────────┐
   │  9 SA × 5 module 跨模块映射 (per 要件 §6.1.11)            │
   └──────────────────────────────────────────────────────────┘

   SA-01 code-review:
   ┌──────────────────────────────────────────────────────┐
   │  ArgCrate:    ✅ create_edge / patch_agent 持久化    │
   │  SubAgent:    ✅ ContextInjector 拉 mentors review  │
   │  LLMService:  ✅ summarize_peer_review (5 类 × 2 长度)│
   │  AgentLease:  — (I: event: review.completed)        │
   │  AgentRuntime:✅ UI 审查 tab                         │
   └──────────────────────────────────────────────────────┘
                          │
                          ▼ (1 触发)
   SA-02 test-gen → SA-06 refactor → SA-04 git-ops
                          │
                          ▼ (2 强协作)
   SA-08 domain-dev → SA-01 code-review (回环)

   SA-03 5-域-lead-audit:
   ┌──────────────────────────────────────────────────────┐
   │  ArgCrate:    ✅ create_audit_event (audit T 表)     │
   │  SubAgent:    ✅ OutputEvaluator 跨域审计             │
   │  LLMService:  ✅ generate_audit_summary              │
   │  AgentLease:  ✅ audit lease (11 字段 per ADR-0030)  │
   │  AgentRuntime:✅ admin UI 审计报告                    │
   └──────────────────────────────────────────────────────┘
   (唯一全 module R+A 的 SA, per 要件 §6.1.3 特殊约束)

   SA-09 achievement-eval:
   ┌──────────────────────────────────────────────────────┐
   │  ArgCrate:    ✅ unlock_achievement (unlocks T 表)   │
   │  SubAgent:    ✅ AchievementEngine (8 拓扑评估)      │
   │  LLMService:  ✅ generate_unlock_message             │
   │  AgentLease:  — (I: event: achievement.unlocked)    │
   │  AgentRuntime:✅ UI 成就墙推送                        │
   └──────────────────────────────────────────────────────┘
   (次全 module R+A, 4/5 module, AgentLease 是 I)
```

---

## 9. 5 Module 跟 LangGraph + Agent Runtime 平行关系 (per 基本 §1.2 + 守门 #3 v2)

```
   ┌──────────────────────────────────────────────────────────┐
   │  3 view 平行关系 (per 守门 #3 v2 拍板 B 反转)             │
   └──────────────────────────────────────────────────────────┘

   LangGraph View (9/3)        Agent Runtime View (9/3)       ARG View (本 view)
   ┌─────────────────┐         ┌──────────────────┐         ┌──────────────────┐
   │ 关注点:         │         │ 关注点:          │         │ 关注点:          │
   │ UI 驱动 2-level │         │ 大规模 AI Agent  │         │ agent 之间 social│
   │ Agent (L0+L1)   │         │ 并发 Runtime 设施│         │ 层 (关系+effect) │
   │                 │         │                  │         │                  │
   │ 9 SA Type:      │         │ 9 Archetype:     │         │ 5 module:        │
   │ LangGraph       │         │ ECS (bevy_ecs/   │         │ ArgCrate /       │
   │ subgraph        │         │ flecs)           │         │ SubAgentOrch /   │
   │ (in-process)    │         │ (in-process)     │         │ LLMService /     │
   │                 │         │                  │         │ AgentLease /     │
   │                 │         │                  │         │ AgentRuntime     │
   │                 │         │                  │         │ (跨 tier)        │
   │                 │         │                  │         │                  │
   │ Checkpoint:     │         │ Checkpoint:      │         │ Checkpoint:      │
   │ 3-tier          │         │ 3-tier           │         │ 共享 PG          │
   │ (RAM/Redis/DB)  │         │ (HOT/WARM/COLD)  │         │ (audit + unlocks)│
   │                 │         │                  │         │                  │
   │ 通信:           │         │ 通信:            │         │ 通信:            │
   │ LangGraph       │         │ Event Bus +      │         │ Memgraph Bolt    │
   │ 状态 + 边 +     │         │ Mailbox +        │         │ subscription +   │
   │ reducer         │         │ Channel          │         │ in-process 推 +  │
   │                 │         │                  │         │ 周期 flush 30s   │
   │                 │         │                  │         │                  │
   │ 路径:           │         │ 路径:            │         │ 路径:            │
   │ docs/arch/      │         │ docs/arch/       │         │ docs/arch/       │
   │ 2026-09-03-     │         │ 2026-09-03-      │         │ 2026-09-03-      │
   │ langgraph/      │         │ agent-runtime/   │         │ arg/             │
   │                 │         │                  │         │ (本 view)        │
   │                 │         │                  │         │                  │
   │ 依赖:           │         │ 依赖:            │         │ 依赖:            │
   │ LangGraph       │         │ bevy_ecs / flecs │         │ Memgraph +       │
   │ Python          │         │ Rust + Tokio     │         │ LangGraph +      │
   │                 │         │                  │         │ Agent Runtime    │
   │                 │         │                  │         │ (parallel)       │
   └─────────────────┘         └──────────────────┘         └──────────────────┘
            │                            │                              │
            └────────────────────────────┼──────────────────────────────┘
                                         │
                                         ▼
                              9 SA Type 是**接口**而不是实现
                              LangGraph subgraph + Agent Runtime ECS + ARG 5 module
                              互不取代, 互不建立业务子域↔DDD bounded context 映射
                              (per 守门 #3 v2 + AGENTS.md §5 仓库拓扑 disclaimer)
```

---

## 10. 守门合规 (per AGENTS.md §4 + §4.1)

- **#1 v15**: 守门 #12 死循环饱和边界
- **#3 v2**: 5 域独立 Lead, 跨域边强制 `consults` 而非 `delegates_to`
- **#5**: env 安全
- **#6**: PowerShell only
- **#9 v19**: 子代理 RPC 不可靠
- **#10**: 代签规则
- **#13**: W/T/M 三类横展开
- **#14 v3**: 5 域 Lead 拍板 D, Mavis 永久代签
- **#19 v19**: 守门 #12 死循环饱和边界

---

## 11. 签字栏 (per 守门 #14 v3 Mavis 永久代签)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| SRE Lead (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| 平台 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| 评审主持 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| PM (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |

---

## 12. 修订历史 (per 守门 #3 禁回溯叙事)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-09 21:18 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 5 module + 9 SA + 5 域 + 4 effect + 5 协议 + 11 子项落地 + 跨 module 集成 + 3 view 平行 关系图 (text-based, 共 10 个关系图) | 2026-09-09 21:16 JST 用户发令"开子代理和 worktree 并行处理" + brief v0.45 §14.11 ARG.1 文档完整化 |
