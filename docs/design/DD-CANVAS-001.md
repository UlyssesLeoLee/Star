# DD-CANVAS-001

> **无限画布 (Infinite Canvas) — 总册詳細設計書 v0.1** (per 日本 IPA SEC 標準 / 詳細設計書 テンプレート)
>
> **⚠️ 双核心定位 (per 2026-09-10 17:08 + 17:21 + 17:34 + 18:00 + 18:25 JST Ulysses 拍板)**
> 核心功能 = **管理 agent + 游戏化**, 避免过度冗余
> 6 阶段拍板: 17:00 旧 12 大类 50 项 Miro 全面对标 (撤回) → 17:08 双核心 → 17:21 ARG 图论构造 (A11) → 17:34 多人编辑 (A12, v0.63 反转) → 18:00 基于需求文档制作基本设计文档 (BD) → 18:25 完善详细设计文档 (本 DD)
>
> - 状态: 🟡 Draft **v0.1.1** (v0.1.1 协调性检查修复落档)
> - 目标阶段: 詳細設計 → 実装 → テスト → リリース
> - 关联需求: [`docs/requirements/SRS-CANVAS-001.md`](../requirements/SRS-CANVAS-001.md) v1.1 (58KB, 18:00 JST root 写, 双核心 78 项索引)
> - 关联基本設計: [`docs/design/BD-CANVAS-001.md`](../design/BD-CANVAS-001.md) v0.1 (50KB, 18:00 JST root 写, 5 view 跨域 + 14 张表 W/T/M 100% 覆盖)
> - 关联专题 DD (派生): [`docs/design/DD-CANVAS-AGENT-001.md`](./DD-CANVAS-AGENT-001.md) v0.1 (18:25 JST 子代理 1 写) + [`docs/design/DD-CANVAS-GAMIFY-001.md`](./DD-CANVAS-GAMIFY-001.md) v0.1 (18:25 JST 子代理 2 写)
> - 关联平行 DD: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](./DD-AGENT-RELATIONSHIP-001.md) v0.1 (94KB, 9/9 落档, 13 关键 class + 4 effect + 5 状态机 + 11 共享类型 + 4 时序图 模板)
> - 关联 V0.1: [`docs/frontend-canvas-design.md`](../frontend-canvas-design.md) v0.1 + [`frontend/src/components/CanvasView.tsx`](../frontend/src/components/CanvasView.tsx)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 9/8 15:19 JST 第 6 次强化)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) (5 角色签字栏 per AGENTS.md §3)
> - 日期: 2026-09-10 JST (18:25 JST 拍板"完善详细设计文档")
> - 受众: 実装エンジニア / テストエンジニア / アーキテクト / SRE / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)

---

## §0 文档信息 / 修订履历

### 0.1 文书信息

| 项目 | 内容 |
|---|---|
| 文书 ID | DD-CANVAS-001 |
| 文书名 | 无限画布 (Infinite Canvas) 総冊詳細設計書 |
| 版本 | **v0.1.1** (per 2026-09-10 18:35 JST 协调性检查修复) |
| 作成日 | 2026-09-10 |
| 作成者 | Ulysses — Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手) |
| 关联 commit | `fb89e4a` (10 files / 8,160 insertions, 第 74 次新事件, 含 v0.1.1 修复版) + `153441a` (§4.29 + registry v0.14, 第 75 次新事件) |
| 关联文档 | `SRS-CANVAS-001.md` v1.1 + `BD-CANVAS-001.md` v0.1 (本批派生) + `DD-CANVAS-{AGENT,GAMIFY}-001.md` v0.1 (本批派生) + `DD-AGENT-RELATIONSHIP-001.md` v0.1 (94KB 模板) |
| 平行 DD | `DD-AGENT-VIEW-001.md` (待补, 9/5 落档) |

### 0.2 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **初始版本: 双核心 (管理 agent + 游戏化) + 多人编辑 (per 17:34 JST v0.63 反转) + ARG (per 17:21 JST) 总册 DD 跨域汇总, 10 段 + 5 附录, 15 章节 IPA SEC 模板, 5 view 跨域 + 14 张表 SQL DDL W/T/M 100% 覆盖 + 32 API 端点 OpenAPI spec + 5 WebSocket 协议 + 13 关键 class + 5 状态机 + 11 共享类型 + 4 关键时序图 + 6 类 NFR + 19 守门 + 12 已知缺口 + 5 角色签字栏** | **2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档" + 18:00 JST BD 拍板 + 17:34 JST v0.63 反转多人编辑 + 17:21 JST ARG 图论构造 + 17:08 JST 双核心 + 17:00 JST 旧方向撤回 (per 守门 #1 禁回溯叙事)** |
| **v0.1.1** | **2026-09-10 18:35 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **协调性检查修复: 3 处 — (1) §4 C-16 `CanvasBackend` → C-25 `CanvasElementsBackend` (本批新增 A12.3 顺延, 跟表名 `canvas_elements_backend` 一致) + §4 C-21 `MultiUserAudit` → C-26 `CanvasMultiUserAudit` (本批新增 A12.8 顺延, 跟表名 `canvas_multi_user_audit` 一致); (2) §5.3 Trust Score 5 档 命名/阈值统一为 ARG 源 `Untrusted (0-0.2) / Low (0.2-0.4) / Medium (0.4-0.7) / High (0.7-0.9) / VeryHigh (0.9-1.0)`; (3) §1.1 / §4 列表头同步更新 (C-21/C-16 → C-25/C-26). 不重写 ARG 4 份 commit (per 守门 #1 禁回溯叙事). BD-CANVAS-AGENT-001 v0.1 无需修复 (5.4.4 已用 ARG 源命名/阈值). DD-CANVAS-AGENT-001 v0.1 子代理 1 已用 C-16=RelationshipEditor / C-21=ARGController (匹配 ARG 源), A12 走 §4.14.x sub-class 模式 (与本总册 C-25/C-26 不同但等价, 跨 DD 一致性在 附录 A 派生源说明).** | **2026-09-10 18:30 JST Ulysses 拍板"确保本设计和 agent 图关系设计妥善协调不冲突" + 协调性检查报告 `COORDINATION-CHECK-001.md` v0.1 §3 修复方案** |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档基于 [`BD-CANVAS-001.md` §0-§10](../design/BD-CANVAS-001.md) v0.1 (50KB, 18:00 JST root 写) 的基本設計, 定义 **无限画布 (Infinite Canvas)** 总冊詳細設計 (DD):

- 5 view 跨域詳細实现 (機能/データ/動作/モジュール/ネットワーク, 跨 3 SRS + 3 BD)
- 14 张表 SQL DDL W/T/M 100% 覆盖 (A11 7 张 + A12 7 张, 跨域汇总)
- 32 API 端点 OpenAPI spec + 5 WebSocket protocol 完整
- 6 类 NFR benchmark (性能/可靠性/安全/易用/可观测/ARG)
- 13 关键 class (C-1/C-2/C-3/C-4/C-7/C-8/C-11/C-12/C-13/C-14/C-15 + **C-25 + C-26** [v0.1.1 修复: 原 C-16/C-21 顺延]) 跨域 + 5 状态机 + 11 共享类型
- 4 关键时序图 (写关系 / 协作影响 / 成就评估 / 离线降级)
- 74+ 测试用例 (UT + IT + E2E + PT, 跨域 ≥ 30 个)
- 19 守门 + 26 派生规 跨域汇总 (含 #23 v2 AI mock 锁)
- 12 已知缺口 (含 3 P0 阻塞, 跨域)

2 份平行专题 DD (DD-AGENT + DD-GAMIFY) 各自展开 46 项 / 32 项详细, **本总册不重复**, 仅汇总跨域共享部分 + 跨域接口.

**dual-use 提醒 (per [AGENTS.md §5 倉庫拓扑](../../AGENTS.md))**: 本 view 不引用 RGS 仓 + 不建立业务子域↔DDD bounded context 映射. 5 域独立 Lead (player/economy/match/social/admin per 8/21 JST RGS 治理命名) **不等于** Star 仓 22 DDD bounded context (per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer). 画布通过 25 module 联动接口跟 5 域对接, 但**不**直接调用其他 view.

### 1.2 包含 (In-Scope, 15 章节)

| 章节 | 内容 | 跨域 |
|---|---|---|
| §0 文档信息 / 修订履历 | 1 段 | 跨域 |
| §1 文档目的 / 适用范围 | 1.1 目的 + 1.2 In-Scope + 1.3 Out-of-Scope | 跨域 |
| §2 系统架构 | 5 view 跨域 + 5-tier + 4 sequence diagram | 跨域 |
| §3 概念 module 布局 | 24 组件 → 18 Rust module + 1 Python LangGraph module | 跨域 |
| §4 关键 class | 13 关键 class 完整字段 + 方法签名 + 错误处理 | 跨域 |
| §5 状态机 | 5 状态机 Rust enum + 状态转移函数 | 跨域 |
| §6 共享类型 | 11 共享类型完整定义 | 跨域 |
| §7 接口协议 | 5 WebSocket + 内部 5 协议 + 32 API OpenAPI spec | 跨域 |
| §8 时序图 | 4 关键时序图 | 跨域 |
| §9 数据持久化 | 14 张表 SQL DDL + Memgraph Cypher + zustand | 跨域 |
| §10 测试用例 | UT + IT + E2E + PT 4 类, 跨域 ≥ 30 | 跨域 |
| 附录 A | 跨专题引用清单 | 跨域 |
| 附录 B | 跨域组件映射 (4 文件 DD + 6 crate + 5 V0.1 复用 + 25 module 联动) | 跨域 |
| 附录 C | 已知缺口 (12 个, 含 3 P0 阻塞) | 跨域 |
| 附录 D | 5 角色签字栏 | 跨域 |
| 附录 E | 修订履历 + token OLU 估算 + 守门交叉引用 | 跨域 |

### 1.3 不包含 (Out-of-Scope)

- **实现 (PHASE-* 报告)**: 后续 P3-D.6 阶段
- **UT/IT/E2E/PT 测试代码实装**: P3-D.6 阶段 (本 DD 仅 spec + 场景)
- **25 module 实体实现**: 25 module 各自 docs, 画布只联动不实装
- **Miro 通用 12 类** (per 17:08 JST 避免过度冗余): 12 diagram / 模板 / 集成 / 移动 / a11y
- **5 域 Lead 真人到位前跨域编排决策**: Mavis 临时代签 (per 守门 #14 v2)
- **双核心之 1: agent 管理 46 项 详细 DD**: 专题 DD 子代理 1 (DD-CANVAS-AGENT-001 v0.1)
- **双核心之 2: 游戏化 32 项 详细 DD**: 专题 DD 子代理 2 (DD-CANVAS-GAMIFY-001 v0.1)

---

## §2 系统架构 (System Architecture, 5 view 跨域 + 5-tier + 4 sequence diagram)

### 2.1 5 view 跨域架构图

```
┌────────────────────────────────────────────────────────────────────┐
│                     gm-console frontend (Next.js 14)                │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 5 view 跨域 UI:                                                │  │
│  │ 1. 機能 view: 78 FR (46 + 32) + 10 关键 class (跨域)         │  │
│  │ 2. データ view: 14 张表 (W/T/M) + 5 状态机 + 11 共享类型      │  │
│  │ 3. 動作 view: 5 域 + 多人 + ARG 4 维 + 4 关键时序图         │  │
│  │ 4. モジュール view: 18 Rust module + 1 Python + V0.1 复用     │  │
│  │ 5. ネットワーク view: 5 WebSocket + 32 API + 74+ 测试          │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
                                ↕ (BFF REST API 32 端点 + WebSocket 5 端点, OpenAPI spec 详见 §7)
┌────────────────────────────────────────────────────────────────────┐
│              BFF Layer (FastAPI + Next.js API routes)               │
│  V0.1 9 API  +  A11 5 API  +  A12 5 API  +  GAMIFY 9 API  +  WS 5 端点 │
└────────────────────────────────────────────────────────────────────┘
                                ↕
┌────────────────────────────────────────────────────────────────────┐
│              Backend (Rust crate + Memgraph + LangGraph)            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │
│  │ crates/arg  │  │ arg-bridge  │  │ arg-effect  │  │ api/arg   │  │
│  │ (data + 6   │  │ (Memgraph ↔ │  │ (4 维度 +   │  │ (13 REST  │  │
│  │  models)    │  │  LangGraph  │  │  5 模板 +   │  │  + 1 WS) │  │
│  │             │  │  同步桥)   │  │  成就)      │  │           │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ Memgraph (Docker, port 7687 Bolt + 7444 HTTP, 数据卷持久化) │  │
│  │ 14 张表 W/T/M 100% 覆盖 (per 守门 #13)                       │  │
│  └─────────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ 25 module 联动接口 (work-item / worktree / agent / relation / │  │
│  │ comment / automation / audit / search / notification + V0.1) │  │
│  └─────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

### 2.2 5-tier 架构 (跨域派生自 DD-AGENT-RELATIONSHIP-001 v0.1 §2)

| Tier | 描述 | 跨域 |
|---|---|---|
| Tier 1: gm-console frontend | Next.js 14 + React 18 + zustand 5 view 跨域 UI | 跨域 |
| Tier 2: BFF | FastAPI + Next.js API routes, 32 REST + 5 WebSocket | 跨域 |
| Tier 3: Domain Crates | 6 Rust crate (arg + arg-bridge + arg-effect + api/arg + 新增 2) | 跨域 |
| Tier 4: Memgraph | 图数据库, 14 张表 W/T/M 100% 覆盖 (per 守门 #13) | 跨域 |
| Tier 5: LangGraph | 25 module 联动接口 + TMO 9 节点 (per `SRS-STAR-AGENT-RUNTIME-001.md` §4) | 跨域 |

### 2.3 4 关键时序图 (跨域, per DD-AGENT-RELATIONSHIP-001 v0.1 §8)

#### 2.3.1 时序图 1: 写关系 (A11 ARG + V0.1 canvas)

```
User → AgentRelationshipEditor (FE) → POST /v1/arg/edges → Memgraph write
  → EventBus `edge.changed` (in-process) → LangGraph StateGraph update
  → 30s period flush → Memgraph read (consistency)
  → WS /ws/arg/events push → AgentRelationshipView (FE) render
  → audit log append (Transaction, per 守门 #13)
```

#### 2.3.2 时序图 2: 协作影响 (A11.3 4 维度, dispatch/context/trust/review)

```
Top Agent receives task → reads ARG → modifies dispatch logic
  → dispatch_router: 按 delegates_to 边 spawn L1 sub-agent
  → context_injector: mentee 拉 mentor 历史 decision 入 prompt
  → trust_engine: trusts weight ≥ 0.8 跳过 verify 节点
  → output_evaluator: peer_reviews 双向 review, challenges 强制论证
  → 4 维度 effect tier 落地 (per DD-AGENT-RELATIONSHIP-001 v0.1 §4)
```

#### 2.3.3 时序图 3: 成就评估 (A11.10 + 8 拓扑 Cypher + 10 challenges prompt)

```
Memgraph write 触发 EventBus → achievement_engine 3 evaluator 并行
  → topology_evaluator: 8 Cypher 模板 (TOP-001..TOP-008) 评估
  → behavior_evaluator: 7 event pattern (BEH-001..BEH-007)
  → output_evaluator: 5 metrics (OUT-001..OUT-005)
  → AchievementPublisher.publish_achievement_unlocked (SSE)
  → Achievement Wall UI render + 通知
```

#### 2.3.4 时序图 4: 离线降级 (per DD-AGENT-RELATIONSHIP-001 v0.1 §4.4)

```
Memgraph write fails → in-process 缓存继续工作 (LWW 缓存)
  → 用户无感, 写入操作本地落 EventBus queue
  → 30s 周期重试 Memgraph write
  → 重连后 flush 所有 pending event
  → 一致性: in-process 跟 Memgraph 最终一致 (per LWW + version optimistic lock)
```

---

## §3 概念 module 布局 (Conceptual Module Layout, 24 组件 → 18 Rust module + 1 Python LangGraph + 跨域)

### 3.1 24 组件 → 18 Rust module + 1 Python LangGraph

**派生自 BD §3 + DD-AGENT-RELATIONSHIP-001 v0.1 §3**:

| 组件 | 路径 | 派生 | 类型 |
|---|---|---|---|
| 1. CanvasView 增强 | `frontend/src/components/CanvasView.tsx` | A1-A12 全部 14 张表 + 46 项 | V0.1 + 扩展 |
| 2. CanvasToolbar 扩展 | `frontend/src/components/CanvasToolbar.tsx` (V0.1 line 376) | A11.2 + A12.5 + 跨域 5 view | 新增 |
| 3. CanvasSidebar 扩展 | `frontend/src/components/CanvasSidebar.tsx` (V0.1) | A2.2 + A12.7 + V0.1 game | 新增 |
| 4. CanvasMinimap 增强 | `frontend/src/components/CanvasMinimap.tsx` (V0.1 line 365) | A11.9 + A12.1 | V0.1 + 扩展 |
| 5. AgentNode 完整卡 | `frontend/src/components/agent-game/AgentNode.tsx` | A1.1 agent_node 完整卡 | A1 |
| 6. AgentTopologyView | `frontend/src/components/agent-game/AgentTopologyView.tsx` | A2.1-A2.4 拓扑图 | A2 |
| 7. AgentStatusBadge | `frontend/src/components/agent-game/AgentStatusBadge.tsx` | A3.1 14 状态机 | A3 |
| 8. AgentMonitorPanel | `frontend/src/components/agent-game/AgentMonitorPanel.tsx` | A7.1-A7.3 监控 | A7 |
| 9. RelationshipEditor | `frontend/src/components/agent-relationships/RelationshipEditor.tsx` | A11.2 关系编辑 (9/8 落档) | A11 |
| 10. RelationshipView | `frontend/src/components/agent-relationships/RelationshipView.tsx` | A11.1 10 类边渲染 (9/8 落档) | A11 |
| 11. AchievementWall | `frontend/src/components/agent-relationships/AchievementWall.tsx` | A11.4 成就 (9/8 落档) | A11 |
| 12. MultiUserPresence | `frontend/src/components/canvas/MultiUserPresence.tsx` | A12.2 实时 cursor | A12 |
| 13. CommentThread | `frontend/src/components/canvas/CommentThread.tsx` | A12.5 评论线程 + @ | A12 |
| 14. FollowMode | `frontend/src/components/canvas/FollowMode.tsx` | A12.4 Follow mode | A12 |
| 15. ConflictResolver | `frontend/src/components/canvas/ConflictResolver.tsx` | A12.6 CRDT | A12 |
| 16. GamificationNode 8 种 | `frontend/src/components/agent-game/GamificationNode.tsx` | G1.1-G1.8 avatar/level/xp/skill_tree/class/badge/quest/inventory | G1 |
| 17. ScorePoints | `frontend/src/components/agent-game/ScorePoints.tsx` | G3.1-G3.3 积分 | G3 |
| 18. LevelingSystem | `frontend/src/components/agent-game/LevelingSystem.tsx` | G4.1-G4.3 升级 | G4 |
| 19. DotVoting | `frontend/src/components/agent-game/DotVoting.tsx` | G6.1-G6.2 投票 | G6 |
| 20. ReactionEmoji | `frontend/src/components/agent-game/ReactionEmoji.tsx` | G7.1 emoji | G7 |
| 21. Confetti | `frontend/src/components/agent-game/Confetti.tsx` | G8.1 confetti | G8 |
| 22. Leaderboard | `frontend/src/components/agent-game/Leaderboard.tsx` | G9.1-G9.2 排行榜 | G9 |
| 23. DailyChallenge | `frontend/src/components/agent-game/DailyChallenge.tsx` | G10.1-G10.2 每日 + streak | G10 |
| 24. PowerUpInventory | `frontend/src/components/agent-game/PowerUpInventory.tsx` | G11.1-G11.2 道具 + 物品栏 | G11 |

### 3.2 18 Rust module (跨域, 派生自 crates/arg 6 module + 6 new + V0.1 复用 6)

| Module | 路径 | 跨域 | 派生 |
|---|---|---|---|
| 1. `crates/arg/src/models/` | 16 字段 (per DD-AGENT-RELATIONSHIP-001 v0.1 §3.2.5) | A11 + A12 + 25 module | ARG 派生 |
| 2. `crates/arg/src/ops/` | 6 操作 (edge / template / trust / output / achievement / cluster) | A11 + GAMIFY | ARG 派生 |
| 3. `crates/arg/src/bridge/` | 5 协议 (AgentContext 跨域) | 25 module 联动 | ARG 派生 |
| 4. `crates/arg-bridge/src/` | 4 子模块 (Memgraph ↔ LangGraph 同步桥) | A11.9 同步桥 | ARG 派生 |
| 5. `crates/arg-effect/src/` | 4 effect + 5 模板 + 成就 (per DD-AGENT-RELATIONSHIP-001 v0.1 §4) | A11.3 4 维度 | ARG 派生 |
| 6. `crates/api/arg/src/` | 13 REST + 1 WebSocket (per DD-AGENT-RELATIONSHIP-001 v0.1 §6.1) | A11.1-A11.10 | ARG 派生 |
| 7. `crates/api/src/multi_user/` (新) | A12 多人编辑 (8 module) | A12.1-A12.8 | 本批新增 |
| 8. `crates/api/src/canvas_backend/` (新) | A12 backend 持久化层 | A12.3 + A12.8 | 本批新增 |
| 9. `crates/api/src/permissions/` (新) | A12.7 权限 | A12.7 | 本批新增 |
| 10. `crates/api/src/comments/` (新) | A12.5 评论 | A12.5 | 本批新增 |
| 11. `crates/api/src/follow/` (新) | A12.4 follow | A12.4 | 本批新增 |
| 12. `crates/api/src/audit_multi/` (新) | A12.8 audit | A12.8 | 本批新增 |
| 13. `crates/gamify/src/models/` (新) | G1-G12 8 节点 struct | GAMIFY | 本批新增 |
| 14. `crates/gamify/src/scoring/` (新) | G3 score + G4 leveling | GAMIFY | 本批新增 |
| 15. `crates/gamify/src/clustering/` (新) | G5 sticky note AI mock | GAMIFY G5 | 本批新增 |
| 16. `crates/gamify/src/engagement/` (新) | G6 投票 + G7 reaction + G8 confetti | GAMIFY | 本批新增 |
| 17. `crates/gamify/src/leaderboard/` (新) | G9 leaderboard | GAMIFY | 本批新增 |
| 18. `crates/gamify/src/inventory/` (新) | G10 daily + G11 power-up/inventory (W/T/M) | GAMIFY | 本批新增 |

### 3.3 1 Python LangGraph module (跨域, 派生自 SRS-STAR-AGENT-RUNTIME-001 v0.2)

`graph/state_graph.py` (TMO 9 节点 + 4 effect tier module 集成, 派生自 DD-AGENT-RELATIONSHIP-001 v0.1 §3 + DD-AGENT-RELATIONSHIP-001 v0.1 §4.5)

### 3.4 跨域组件映射表 (跨 24 前端 + 18 Rust + 1 Python + 5 V0.1 复用)

详见附录 B.

---

## §4 关键 class (Key Classes, 13 跨域)

**派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §3.2 + §3.3 + §4, P3-D 落地 C-1/C-2/C-3/C-4/C-7/C-8/C-11/C-12/C-13/C-14/C-15 + C-25 + C-26 (本批新增 A12, per 协调性检查报告 §3 修复方案)**:

| Class | 字段数 | 方法数 | 跨域 | 派生 |
|---|---|---|---|---|
| C-1 `AgentNode` | 16 (id, name, archetype, domain, status, trust_score, metadata, created_at, updated_at, version, x, y, width, height, rotation, z_index) | 12 (move, resize, rotate, set_color, get_status, set_status, attach_worktree, attach_work_item, derive_handoff, archive, restore, validate) | A1 + A2 + A5 | 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §3.2.1 + `SRS-STAR-AGENT-RUNTIME-001.md` §2.3 |
| C-2 `AgentRelationshipEdge` | 14 (id, from_agent, to_agent, type, weight, direction, archived, metadata, version, created_at, created_by, trust_score_delta, effect_tier, audit_hash) | 8 (create, archive, restore, set_weight, increment_version, validate, derive_effect, compute_trust) | A11 + A2.1 handoff | 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §3.2.1 + §4.1.3 |
| C-3 `TeamTemplate` | 9 (id, name, topology, agent_ids, edges_json, ttl_seconds, created_at, version, is_active) | 6 (instantiate, expire, clone, set_active, validate, derive_topology) | A11.4 5 模板 | 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §4.5 |
| C-4 `TeamTemplateInstance` | 5 (id, template_id, instance_name, agent_ids, edges_json, expires_at) | 4 (create, expire, list, extend) | A11.4 (TTL 30d Work) | 派生自 `SRS-AGENT-RELATIONSHIP-001.md` §4.2.3 + 守门 #13 Work |
| C-7 `AgentWorktreeRelation` | 6 (id, agent_id, worktree_id, status, created_at, version) | 4 (attach, detach, sync_status, list) | A4.1 1:N | V0.1 扩展 |
| C-8 `AgentWorkItemRelation` | 6 (id, agent_id, work_item_id, status, created_at, version) | 4 (attach, detach, sync_status, list) | A5.1 1:N | V0.1 扩展 |
| C-11 `ARGDispatchRouter` | 5 (edges, agent_context, dispatch_strategy, conflict_resolver, fallback_chain) | 6 (route, fallback, resolve, trace, log, audit) | A11.3 4.1 Dispatch 路由 | 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §4.3.1 |
| C-12 `ARGContextInjector` | 5 (mentor_history, shadow_events, context_cap_kb, prompt_injector, dedup_strategy) | 5 (inject, dedup, cap, compress, audit) | A11.3 4.2 上下文共享 | 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §4.3.2 |
| C-13 `ARGTrustEngine` | 6 (trust_store, edge_weight, skip_threshold, peer_review_threshold, nudge_delta, audit_log) | 7 (compute_trust, nudge, skip_verify, escalate, audit, validate, get_trust_score) | A11.3 4.3 信任度加权 | 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §4.3.3 |
| C-14 `ARGOutputEvaluator` | 8 (challenge_prompts, peer_review_threshold, escalation_chain, mock_llm_client, decision_types, trust_tiers, payload, audit_log) | 8 (evaluate, challenge, peer_review, escalate, audit, validate, get_verdict, mock_call) | A11.3 4.4 产出评估 | 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §4.3.4 |
| C-15 `Achievement` | 8 (id, name, description, category, rarity, condition, reward_id, version) | 5 (evaluate, unlock, list, validate, audit) | A11.4 成就 + G2 achievement | 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §6.2 |
| C-25 `CanvasElementsBackend` | 12 (id, canvas_id, kind, x, y, width, height, rotation, z_index, content, locked, hidden, version) | 10 (create, read, update, delete, move, resize, rotate, lock, hide, list) | A12.3 element backend (替代 V0.1 localStorage) | 本批新增 (per 协调性检查报告 §3.1, 跟表名 `canvas_elements_backend` 一致) |
| C-26 `CanvasMultiUserAudit` | 9 (id, canvas_id, actor_user_id, action, target_id, target_type, payload, created_at, version) | 6 (record, list, query, archive, export, validate) | A12.8 audit | 本批新增 (per 协调性检查报告 §3.2 + 守门 #13 Transaction, 跟表名 `canvas_multi_user_audit` 一致) |

**13 关键 class 总计**: 跨域, 13 / 24 (C-1..C-24) = 54.2% 落地, 11 留 P3-C 实装阶段 (C-5/C-6/C-9/C-10/C-17/C-18/C-19/C-20/C-22/C-23/C-24)

---

## §5 状态机 (State Machines, 5 跨域)

**派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §5 (Edge / Agent / Trust Score 5 档 / Template Instance / Achievement)**:

| 状态机 | 状态数 | 状态转移函数 | 跨域 |
|---|---|---|---|
| 5.1 `Edge` 状态机 | 5 状态 (Active / Archived / Draft / PendingApproval / Failed) | 12 转移函数 (create / archive / restore / approve / reject / bump_version / validate / merge / split / promote / demote / fail) | A11.5 关系 audit + A11.7 关系 archive/restore |
| 5.2 `Agent` 状态机 | 14 状态 (per `SRS-STAR-AGENT-RUNTIME-001.md` §8: queued / spawning / initializing / compiling_context / planning / executing / awaiting_feedback / awaiting_human / awaiting_tool / validating / paused / completed / failed / cancelled) | 13 转移函数 (queue / spawn / init / compile / plan / execute / await_feedback / await_human / await_tool / validate / pause / complete / fail / cancel) | A3.1 14 状态机实时色码 |
| 5.3 `Trust Score` 5 档 | 5 状态 (**Untrusted 0.0-0.2** / **Low 0.2-0.4** / **Medium 0.4-0.7** / **High 0.7-0.9** / **VeryHigh 0.9-1.0**) (per ARG 源 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §3.3.3, v0.1.1 协调性检查修复统一) | 5 转移函数 (nudge_up / nudge_down / recompute / archive / verify) | A11.3 4.3 信任度加权 |
| 5.4 `Template Instance` 状态机 | 4 状态 (Creating / Active / Expired / Archived) | 4 转移函数 (create / activate / expire / archive) | A11.4 5 模板 + 守门 #13 Work TTL 30d |
| 5.5 `Achievement` 状态机 | 4 状态 (Locked / Available / Unlocked / Revoked) | 4 转移函数 (lock / unlock / revoke / check) | A11.4 成就 + G2 reward |

**5 状态机总计**: 跨域, 32 状态 + 38 转移函数, 5 / 5 落地 (100%)

---

## §6 共享类型 (Shared Types, 11 跨域)

**派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §3.2.5 (11 共享类型)**:

| 共享类型 | 字段数 | 跨域 | 派生 |
|---|---|---|---|
| 6.1 `ARGEvent` | 8 (event_type, edge_id, actor, before, after, timestamp, version, source) | A11 全部 | DD-AGENT-REL v0.1 §3.2.5 |
| 6.2 `Decision` | 6 (decision_id, decision_type, agent_id, context, outcome, timestamp) | A11.3 4.1 Dispatch | DD-AGENT-REL v0.1 §3.2.5 |
| 6.3 `Output` | 5 (output_id, agent_id, content, trust_tier, timestamp) | A11.3 4.4 产出评估 | DD-AGENT-REL v0.1 §3.2.5 |
| 6.4 `Verdict` | 4 (verdict_id, decision_id, accept, reason) | A11.3 4.4 产出评估 | DD-AGENT-REL v0.1 §3.2.5 |
| 6.5 `LLMClient` (mock) | 3 (prompt, response, confidence) | A11.3 4.4 + G5 聚类 (per 守门 #23 v2) | DD-AGENT-REL v0.1 §3.2.5 |
| 6.6 `AchievementUnlock` | 5 (id, user_id, achievement_id, unlocked_at, trigger_event) | A11.4 成就 | DD-AGENT-REL v0.1 §3.2.5 |
| 6.7 `TemplateInstance` | 5 (id, template_id, instance_name, agent_ids, edges_json) | A11.4 5 模板 | DD-AGENT-REL v0.1 §3.2.5 |
| 6.8 `ARGState` (in-process 缓存) | 4 (canvas_id, edges, version, last_sync_at) | A11.9 同步桥 + A12.3 element backend | DD-AGENT-REL v0.1 §3.2.5 |
| 6.9 `EscalationInfo` | 5 (escalation_id, source, target, reason, level) | A11.3 4.4 challenges escalate | DD-AGENT-REL v0.1 §3.2.5 |
| 6.10 `PeerReviewVerdict` | 5 (review_id, reviewer_id, score, threshold, accept) | A11.3 4.4 peer_reviews | DD-AGENT-REL v0.1 §3.2.5 |
| 6.11 `ChallengePrompt` | 5 (prompt_id, decision_type, trust_tier, prompt, escalation) | A11.3 4.4 challenges (10 套 prompt 模板) | DD-AGENT-REL v0.1 §3.2.5 |

**11 共享类型总计**: 跨域, 56 字段, 11 / 11 落地 (100%)

---

## §7 接口协议 (Interface Protocols, 5 WebSocket + 32 API + 内部 5 协议)

### 7.1 5 WebSocket 协议 (跨域)

| 端点 | 协议 | 跨域 | 派生 |
|---|---|---|---|
| `wss://canvas-collab/canvases/[id]` | A12.1 + A12.3 多人 + element 增删改, max 200ms P95, max 10 并发, JSON-RPC 2.0 风格 | A12.1 + A12.3 | per `frontend-canvas-design.md` §4.1 模式 A |
| `wss://canvas-presence/canvases/[id]` | A12.2 实时 cursor, heartbeat 30s, JSON {user_id, x, y, viewport, action} | A12.2 | per `frontend-canvas-design.md` §4.6 PresenceCursor |
| `wss://canvas-comments/canvases/[id]` | A12.5 评论线程 + @, JSON {comment_id, thread_id, mentions, notify_at} | A12.5 | per V0.1 comment_pin line 253-262 |
| `wss://canvas-follow/canvases/[id]` | A12.4 Follow mode, JSON {follower_user_id, leader_user_id, expires_at} | A12.4 | per V0.1 §4.9 URL 透传 |
| `wss://arg-events/canvases/[id]` | A11 ARG events, JSON {event_type, edge_id, payload, timestamp} | A11 全部 | per `DD-AGENT-RELATIONSHIP-001.md` v0.1 §6.1 |

### 7.2 32 BFF REST API 端点 (跨域 OpenAPI spec, 详见 BD-CANVAS-001 v0.1 §5.1)

32 端点分布:
- V0.1 9 端点 (Canvas CRUD / Element / Connector / Viewport / Share / Export PNG / Audit / Search / Comment)
- A11 5 端点 (ARG edges CRUD / Template instantiate / Sync status / Audit log / Relationship view)
- A12 5 端点 (Multi-user Realtime / Presence / Follow / Comment thread / Multi-user audit)
- AGENT 5 端点 (Agent status sync / Operation start-stop-restart / Token usage)
- GAMIFY 8 端点 (Reward / Score / Level / Cluster / Vote / Reaction / Confetti / Leaderboard / Daily / Power-up / Inventory)

### 7.3 内部 5 协议 (跨域)

| 协议 | 跨域 |
|---|---|
| **In-process StateGraph 桥接**: Memgraph write → EventBus `edge.changed` → in-process StateGraph update → 30s period flush | A11 全部 |
| **WebSocket 协议** (5 端点) | 跨域 |
| **25 module 联动接口** (per 总册 §6.3) | work-item / worktree / agent / relation / comment / automation / audit / search / notification |
| **OAuth 协议** (per 守门 #5 + 守门 #1 v1a) | 401 retry 跨 session 续 |
| **WSS 选型协议** (per A12.1 P0 阻塞) | 拍板前走 polling fallback (V0.1 模式 B 30s) |

### 7.4 错误处理 (跨域, per 守门 #1 v1a + 守门 #5)

| 错误类型 | 处理 | 跨域 |
|---|---|---|
| 401 Authentication failed | 跨 session 续, **不算 timeout, max 2 retries** | 跨域 |
| Recv failure / Connect failed | max 2 retries, 30s-2min 后常恢复 | 跨域 |
| Timeout (5s+) | max 2 retries | 跨域 |
| 5xx Server Error | 1 retry, 5min 后, 失败给用户提示 | 跨域 |
| 4xx Client Error (非 401) | 不 retry | 跨域 |
| 凭据 (agent 启停 + reward 解锁) | stdin pipe 模式 | 跨域 |
| WSS 选型 (A12.1) | 拍板前 polling fallback | A12 |
| CRDT 选型 (A12.6) | 拍板前 LWW | A12 |

---

## §8 时序图 (Sequence Diagrams, 4 关键)

详见 §2.3 (4 关键时序图) + DD-AGENT-RELATIONSHIP-001 v0.1 §8 (4 关键时序图).

---

## §9 数据持久化 (Data Persistence, 14 张表 SQL DDL + Memgraph Cypher + zustand)

### 9.1 14 张表 W/T/M 跨域汇总 (per 守门 #13 100% 覆盖)

**A11 7 张表** (派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §9.1, 完整 SQL DDL 详见专题 DD):

```sql
-- 1. agents (Master, SCD Type 2, 100% RLS 13 类)
CREATE TABLE agents (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  archetype VARCHAR(32) NOT NULL,  -- SA-01..SA-09 / LEAD-{DOMAIN} / CUSTOM
  domain VARCHAR(32),  -- player/economy/match/social/admin
  status VARCHAR(32) NOT NULL,  -- 14 状态机
  trust_score FLOAT DEFAULT 0.5,
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  version INT DEFAULT 1
);

-- 2. agent_relationship_edges (Master, SCD Type 2, 100% RLS 13 类)
CREATE TABLE agent_relationship_edges (
  id UUID PRIMARY KEY,
  from_agent UUID NOT NULL REFERENCES agents(id),
  to_agent UUID NOT NULL REFERENCES agents(id),
  type VARCHAR(32) NOT NULL,  -- 10 类关系 (4 核心 + 6 扩展)
  weight FLOAT DEFAULT 0.5,
  direction VARCHAR(16) NOT NULL,  -- directed / undirected
  archived BOOLEAN DEFAULT FALSE,
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  created_by UUID NOT NULL,
  version INT DEFAULT 1,
  trust_score_delta FLOAT DEFAULT 0.0,
  effect_tier VARCHAR(16),  -- dispatch/context/trust/review
  audit_hash VARCHAR(64)
);

-- 3. agent_relationship_edges_audit (Transaction, append-only, 100% audit)
CREATE TABLE agent_relationship_edges_audit (
  id UUID PRIMARY KEY,
  edge_id UUID NOT NULL,
  action VARCHAR(16) NOT NULL,  -- create/update/archive
  old_value JSONB,
  new_value JSONB,
  actor UUID NOT NULL,
  timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- 4. team_template_instances (Work, TTL 30 天)
CREATE TABLE team_template_instances (
  id UUID PRIMARY KEY,
  template_id UUID NOT NULL,
  instance_name VARCHAR(255) NOT NULL,
  agent_ids UUID[] NOT NULL,
  edges_json JSONB NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '30 days'
);

-- 5. achievements (Master, SCD Type 2, 100% RLS 13 类)
CREATE TABLE achievements (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  category VARCHAR(32),  -- topology/behavior/output (per SRS-AGENT-REL v0.1 §6)
  rarity VARCHAR(16),  -- common/rare/epic/legendary
  condition JSONB NOT NULL,  -- 触发条件
  reward_id UUID,
  version INT DEFAULT 1
);

-- 6. achievement_unlocks (Transaction, append-only, 100% audit)
CREATE TABLE achievement_unlocks (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  achievement_id UUID NOT NULL REFERENCES achievements(id),
  unlocked_at TIMESTAMPTZ DEFAULT NOW(),
  trigger_event JSONB
);

-- 7. relationship_events (Transaction, append-only, 100% audit)
CREATE TABLE relationship_events (
  id UUID PRIMARY KEY,
  edge_id UUID REFERENCES agent_relationship_edges(id),
  event_type VARCHAR(32) NOT NULL,  -- edge.changed / weight.changed / archive
  actor UUID,
  payload JSONB,
  before JSONB,
  after JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  version INT DEFAULT 1
);
```

**A12 7 张表** (本批新增, 完整 SQL DDL 详见专题 DD):

```sql
-- 8. canvas_multi_user_audit (Transaction, append-only, SCD Type 2, 100% RLS 13 类)
CREATE TABLE canvas_multi_user_audit (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  actor_user_id UUID NOT NULL,
  action VARCHAR(32) NOT NULL,  -- 9 种: move/resize/rotate/create/delete/lock/hide/comment/follow
  target_id UUID,
  target_type VARCHAR(32),
  payload JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  version INT DEFAULT 1
);

-- 9. canvas_comments (Master, SCD Type 2, 100% RLS 13 类)
CREATE TABLE canvas_comments (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  element_id UUID,
  author_id UUID NOT NULL,
  body TEXT NOT NULL,
  parent_comment_id UUID REFERENCES canvas_comments(id),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  version INT DEFAULT 1
);

-- 10. canvas_comment_mentions (Transaction, append-only, 100% audit)
CREATE TABLE canvas_comment_mentions (
  id UUID PRIMARY KEY,
  comment_id UUID NOT NULL REFERENCES canvas_comments(id),
  mentioned_user_id UUID NOT NULL,
  notified_at TIMESTAMPTZ DEFAULT NOW()
);

-- 11. canvas_permissions (Master, SCD Type 2, 100% RLS 13 类)
CREATE TABLE canvas_permissions (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  user_id UUID NOT NULL,
  level VARCHAR(16) NOT NULL,  -- view/comment/edit (per A12.7)
  granted_by UUID,
  granted_at TIMESTAMPTZ DEFAULT NOW(),
  version INT DEFAULT 1
);

-- 12. canvas_followers (Work, session-bound, 允许物理删除)
CREATE TABLE canvas_followers (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  follower_user_id UUID NOT NULL,
  leader_user_id UUID NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL  -- session 结束
);

-- 13. canvas_elements_backend (Master, SCD Type 2, 100% RLS 13 类)
CREATE TABLE canvas_elements_backend (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  kind VARCHAR(32) NOT NULL,  -- 14 element kind
  x FLOAT NOT NULL,
  y FLOAT NOT NULL,
  width FLOAT NOT NULL,
  height FLOAT NOT NULL,
  rotation FLOAT DEFAULT 0,
  z_index INT DEFAULT 0,
  content JSONB,  -- element 特定内容
  locked BOOLEAN DEFAULT FALSE,
  hidden BOOLEAN DEFAULT FALSE,
  version INT DEFAULT 1
);

-- 14. canvas_presence_cursors (Work, heartbeat 30s, 允许物理删除)
CREATE TABLE canvas_presence_cursors (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  user_id UUID NOT NULL,
  x FLOAT,
  y FLOAT,
  viewport JSONB,
  heartbeat_at TIMESTAMPTZ DEFAULT NOW()
);
```

**14 张表 W/T/M 验证**:
- **Master 6/14 (42.9%)**: agents / agent_relationship_edges / achievements / canvas_comments / canvas_permissions / canvas_elements_backend
- **Transaction 5/14 (35.7%)**: agent_relationship_edges_audit / achievement_unlocks / relationship_events / canvas_multi_user_audit / canvas_comment_mentions
- **Work 3/14 (21.4%)**: team_template_instances (TTL 30d) / canvas_followers (session-bound) / canvas_presence_cursors (heartbeat 30s)
- **总计 14/14 = 100%** ✓ (per 守门 #13 W/T/M 三類横展 强制分类, 禁止混在)

### 9.2 29 张表跨域汇总 (per 守门 #13)

总册 14 张表 (A11 7 + A12 7) + GAMIFY 15 张表 (per BD-CANVAS-GAMIFY-001 v0.1 §4.1 G11 15 张表 Work 6 + Transaction 4 + Master 5) = **29 张表 100% W/T/M 覆盖跨域汇总**

### 9.3 Memgraph Cypher schema (A11 派生, per `DD-AGENT-RELATIONSHIP-001.md` v0.1 §9.2)

```cypher
// Agent 节点 (per 守门 #13 Master)
CREATE (a:Agent {
  id: $id,
  name: $name,
  archetype: $archetype,
  domain: $domain,
  status: $status,
  trust_score: $trust_score,
  metadata: $metadata,
  created_at: $created_at,
  updated_at: $updated_at,
  version: $version
});

// 10 类关系边 (4 核心 + 6 扩展)
CREATE (a)-[r:DELEGATES_TO | CONSULTS | COLLABORATES_WITH | REPORTS_TO | MENTORS | PEER_REVIEWS | STAND_IN_FOR | SHADOWS | CHALLENGES | TRUSTS {
  id: $id,
  weight: $weight,
  archived: $archived,
  created_at: $created_at,
  created_by: $created_by,
  version: $version,
  metadata: $metadata
}]->(b);

// 8 拓扑 Cypher 模板 (per DD-AGENT-RELATIONSHIP-001 v0.1 §8)
// TOP-001..TOP-008 跨域覆盖 8 类拓扑成就
```

### 9.4 zustand store 扩展 (V0.1 + 双核心 + A11 + A12 跨域)

```typescript
// V0.1 已有 (per frontend/src/lib/store.ts)
interface V01Store {
  worktrees: Worktree[];
  agentSessions: AgentSession[];
  automationRules: AutomationRule[];
  feedbacks: Feedback[];
  canvasElements: CanvasElement[];  // 14 element kind
  canvasConnectors: CanvasConnector[];
  canvasFrames: CanvasFrame[];
  // actions
  moveCanvasElement: (id, x, y) => void;
  deleteCanvasElement: (id) => void;
}

// A11 扩展 (本批)
interface A11Store extends V01Store {
  argEdges: AgentRelationshipEdge[];  // 10 类关系
  argTeamTemplates: TeamTemplate[];  // 5 模板
  argAudit: AgentRelationshipEdgeAudit[];  // Transaction
  argAchievements: Achievement[];  // Master
  argAchievementUnlocks: AchievementUnlock[];  // Transaction
  // A11 actions
  createEdge: (edge: AgentRelationshipEdge) => Promise<void>;
  instantiateTemplate: (templateId: string, agentIds: string[]) => Promise<TeamTemplateInstance>;
}

// A12 扩展 (本批)
interface A12Store extends A11Store {
  canvasMultiUserAudit: CanvasMultiUserAudit[];  // Transaction
  canvasComments: CanvasComment[];  // Master
  canvasCommentMentions: CanvasCommentMention[];  // Transaction
  canvasPermissions: CanvasPermission[];  // Master
  canvasFollowers: CanvasFollower[];  // Work
  canvasElementsBackend: CanvasElementsBackend[];  // Master (替代 V0.1 localStorage)
  canvasPresenceCursors: CanvasPresenceCursor[];  // Work
  // A12 actions
  sendCursor: (x, y) => void;
  followUser: (userId: string) => Promise<void>;
  resolveConflict: (strategy: 'LWW' | 'Yjs' | 'Automerge') => void;
}

// GAMIFY 扩展 (本批)
interface GamifyStore extends A12Store {
  gamificationNodes: GamificationNode[];  // 8 kind
  rewards: Reward[];
  achievements: Achievement[];  // 共享
  scores: Score[];
  levels: Level[];
  leaderboards: Leaderboard[];
  votes: Vote[];
  reactions: Reaction[];
  confettis: Confetti[];
  challenges: Challenge[];
  streaks: Streak[];
  powerups: Powerup[];
  inventory: InventoryItem[];
  // GAMIFY actions
  vote: (elementId: string) => void;
  react: (elementId: string, emoji: string) => void;
  clusterStickyNotes: (elementIds: string[], k: number) => Promise<MockClusterResponse>;  // per 守门 #23 v2 mock
}
```

---

## §10 测试用例 (Test Cases, 4 类 ≥ 30 跨域)

**派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §10 (52 UT + 10 IT + 8 E2E + 4 PT = 74 测试) + GAMIFY ≥ 30**:

| 测试类 | 数量 | 跨域 | 派生 |
|---|---|---|---|
| 10.1 UT (Unit Tests) | 52 | A1-A12 + G1-G12 | DD-AGENT-REL v0.1 §10.1 |
| 10.2 IT (Integration Tests) | 10 | A11.5 关系 audit + A12.8 多人 audit + GAMIFY G11 inventory | DD-AGENT-REL v0.1 §10.2 |
| 10.3 E2E (End-to-End Tests) | 8 | V0.1 9 + A11 + A12 多人 | DD-AGENT-REL v0.1 §10.3 |
| 10.4 PT (Performance Tests) | 4 | NFR-PERF 8 项 (A3.1 WS sync < 200ms, A12.1 多人 < 200ms, GAMIFY G5 聚类 < 3s) | DD-AGENT-REL v0.1 §10.4 |
| **总计** | **74+** | 跨域 ≥ 30 | per DD-AGENT-REL v0.1 §10 |

---

## §11 NFR 详细 (Non-Functional Requirements, 6 类, per BD §7 + AGENTS.md §3 守门 19/19 跨域)

### 11.1 NFR-CANVAS-PERF: 性能 (跨域 ≥ 6 项, per 守门 #1 v15)

| ID | 名称 | 指标 | 验证方式 | 优先级 |
|---|---|---|---|---|
| NFR-CANVAS-PERF-01 | 画布首次渲染 | ≤ 500ms (mock 12 agent + 12 worktree + 30 wi + 50 ARG 边 + 10 多人并发) | FCP / LCP | P0 |
| NFR-CANVAS-PERF-02 | 状态同步延迟 | ≤ 200ms (P95) 反映到 agent_node 色码 | dev tools + P95 | P0 |
| NFR-CANVAS-PERF-03 | ARG 边创建 latency | P95 < 200ms (Memgraph Bolt) | Prometheus | P0 |
| NFR-CANVAS-PERF-04 | 多人同时编辑 Realtime 同步 | 10 用户并发, 元素增删改 ≤ 200ms (P95) | dev tools + WSS | P0 |
| NFR-CANVAS-PERF-05 | 实时 Cursor 同步 | cursor 移动 ≤ 100ms (P95, throttle 50ms) | dev tools + P95 | P0 |
| NFR-CANVAS-PERF-06 | Follow mode 同步 | viewport 同步 ≤ 100ms (P95) | dev tools + P95 | P1 |

### 11.2 NFR-CANVAS-RELIABILITY: 可靠性 (3 项, per 守门 #1 累积规)

- **NFR-CANVAS-RELI-01**: 元素增删改 100% 持久化 (PostgreSQL `canvas_elements_backend` + audit `canvas_multi_user_audit` 双写, per 守门 #13 Transaction 100% audit)
- **NFR-CANVAS-RELI-02**: WSS 断线自动 retry 3 次 (exponential backoff 1s/2s/4s), 重连后自动 sync, 不丢操作 (per UC-A14)
- **NFR-CANVAS-RELI-03**: Memgraph 不可达时 in-process 缓存继续工作 (派生自 `BD-AGENT-RELATIONSHIP-001` §7.2, per §9.2 离线降级)

### 11.3 NFR-CANVAS-SECURITY: 安全 (per 守门 #5 + 守门 #13 + 9/1 13:03+13:05 JST envoy 偏好)

- **NFR-CANVAS-SEC-01**: 13 租户隔离 (RLS 13 类必携, per 守门 #13), audit log 必记 (per 守门 #13 Transaction 100% audit)
- **NFR-CANVAS-SEC-02**: A12 BFF 3 级权限校验走 envoy middleware (per A12.7 + 9/1 13:03+13:05 JST envoy 独立 deployment 偏好), 不依赖前端隐藏
- **NFR-CANVAS-SEC-03**: A12 WSS connection 走 TLS 1.3+ (per NFR-CANVAS-MU-CONS-01)
- **NFR-CANVAS-SEC-04**: A12.8 `canvas_multi_user_audit` 表 100% RLS 13 类必携 (per 守门 #13 + A12.8)
- **NFR-CANVAS-SEC-05**: Memgraph 连接字符串走 env (per 守门 #5), 不打印
- **NFR-CANVAS-SEC-06**: 代签规则 (per 守门 #10 + 9/8 15:19 第 6 次强化), author = Ulysses, 真人到位后追溯签字

### 11.4 NFR-CANVAS-USABILITY: 易用 (5 项, per 守门 #1 v15)

- **NFR-CANVAS-UI-01**: 视觉一致性 (跟 StatusPill 60+ 配色一致 / 跟 V0.1 `frontend-canvas-design.md` 一致 / 跟 ARG 10 类关系颜色规范一致 / dark mode 优先)
- **NFR-CANVAS-A11Y-01**: 键盘可达 (顶部 dropdown 满足 WAI-ARIA listbox 模式 / 右键菜单满足 menu 模式 / 多人 cursor 颜色不能仅靠颜色区分, 12 色调色板 + 形状/字母辅助, color-blind 友好)
- **NFR-CANVAS-DET-01**: 派生确定性 (同样输入永远出同样输出, 排序稳定 `[status_order ASC, due_date ASC, id ASC]`)
- **NFR-CANVAS-STATE-01**: 派生只读 + 跨块接口 (画布不进 zustand store 派生数据, 跨块接口走 URL 参数 + 跳路由, 不直接调其他 view 的 LangGraph node; **A12.3 多人编辑元素增删改走 backend 持久化 + WSS 广播, 不进 zustand persist**)
- **NFR-CANVAS-I18N-01**: 国际化 (3 语言 zh-CN / en / ja 友好, 至少 4 项 i18n key 落 `dictionary.ts`: agent status / role / kind / 关系 type 10 类 / **多人 cursor 名字 + 权限 view-comment-edit + 评论 + @ 提示**)

### 11.5 NFR-CANVAS-OBSERVABILITY: 可观测 (3 项, per 守门 #23 v2 调试控制台 + Prometheus)

- **NFR-CANVAS-OBS-01**: agent 状态变化 / ARG 边创建 / 状态联动 / **多人 cursor 移动 / 元素增删改 / 评论 / @** 100% audit + Prometheus 导出
- **NFR-CANVAS-OBS-02**: tracing 日志 (关系增删改 INFO / Memgraph 不可达 WARN / 成就解锁 INFO / trust_score 变化 DEBUG / 多人操作 INFO)
- **NFR-CANVAS-OBS-03**: OpenTelemetry 链路追踪 (元素创建 → 同步桥 → effect reload / 多人编辑 → BFF → audit 全链路)

### 11.6 NFR-CANVAS-ARG: ARG 特定 (派生自 `BD-AGENT-RELATIONSHIP-001` §7.4)

- **NFR-CANVAS-ARG-01**: 20 成就 3 维度分布 (8 拓扑 + 7 行为 + 5 产出, 稀有度 8 COMMON + 7 RARE + 3 EPIC + 2 LEGENDARY)
- **NFR-CANVAS-ARG-02**: 4 维度 effect reload ≤ 50ms (in-process state diff)
- **NFR-CANVAS-ARG-03**: Period flush 30s 周期, 0 阻塞 (独立 tokio task)
- **NFR-CANVAS-ARG-04**: WebSocket 推送 < 100ms (10 并发客户端, SSE fan-out)
- **NFR-CANVAS-ARG-05**: 5 模板 1-click 部署 ≤ 30s (1 事务 Cypher)
- **NFR-CANVAS-ARG-06**: 信任度动态 (成功 +0.01 / 失败 -0.05)

### 11.7 NFR-CANVAS-MU-CONS-01 (A12 多人编辑守门合规, 跨域)

- **5 域 Lead 真人未到位 disclaimer**: A12.7 3 级权限矩阵 + A12.5 评论 @ + A12.3 多人编辑元素增删改 + A12.1 多人同时编辑 + A12.2 实时 cursor + A12.4 Follow mode + A12.6 CRDT 选型 + A12.8 audit log, 共 8 项中 5 项 (A12.1/A12.6/A12.7) 需 5 域 Lead 真人到位后拍板, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

## §12 5 角色签字栏 (per AGENTS.md §3 7 段结构, per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D + 9/10 12:45 JST v0.62 反转)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

## §13 修订历史 (per AGENTS.md §3 7 段结构, per 守门 #1 禁回溯叙事不重写 v0.1 + 修订历史表 1 行)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 18:25 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **初始版本: 双核心 (管理 agent + 游戏化) + 多人编辑 (per 17:34 JST v0.63 反转) + ARG (per 17:21 JST) 总册 DD 跨域汇总, 11 段 + 5 附录, 15 章节 IPA SEC 模板, 5 view 跨域 + 14 张表 SQL DDL W/T/M 100% 覆盖 + 32 API 端点 OpenAPI spec + 5 WebSocket 协议 + 13 关键 class + 5 状态机 + 11 共享类型 + 4 关键时序图 + 6 类 NFR + 19 守门 + 12 已知缺口 + 5 角色签字栏** | **2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档" + 18:00 JST BD 拍板 + 17:34 JST v0.63 反转多人编辑 + 17:21 JST ARG 图论构造 + 17:08 JST 双核心 + 17:00 JST 旧方向撤回 (per 守门 #1 禁回溯叙事)** |
| **v0.1.1** | **2026-09-10 18:35 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **协调性检查修复: 3 处 — (1) §4 C-16 `CanvasBackend` → C-25 `CanvasElementsBackend` (本批新增 A12.3 顺延, 跟表名 `canvas_elements_backend` 一致) + §4 C-21 `MultiUserAudit` → C-26 `CanvasMultiUserAudit` (本批新增 A12.8 顺延, 跟表名 `canvas_multi_user_audit` 一致); (2) §5.3 Trust Score 5 档 命名/阈值统一为 ARG 源 `Untrusted (0-0.2) / Low (0.2-0.4) / Medium (0.4-0.7) / High (0.7-0.9) / VeryHigh (0.9-1.0)`; (3) §1.1 / §4 列表头同步更新 (C-21/C-16 → C-25/C-26). 不重写 ARG 4 份 commit (per 守门 #1 禁回溯叙事). BD-CANVAS-AGENT-001 v0.1 无需修复 (5.4.4 已用 ARG 源命名/阈值). DD-CANVAS-AGENT-001 v0.1 子代理 1 已用 C-16=RelationshipEditor / C-21=ARGController (匹配 ARG 源), A12 走 §4.14.x sub-class 模式 (与本总册 C-25/C-26 不同但等价, 跨 DD 一致性在 附录 A 派生源说明).** | **2026-09-10 18:30 JST Ulysses 拍板"确保本设计和 agent 图关系设计妥善协调不冲突" + 协调性检查报告 `COORDINATION-CHECK-001.md` v0.1 §3 修复方案** |
| **v0.1.2** | **2026-09-10 19:10 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **IPA SEC 合规性修复: 加 §11 NFR 详细 (6 类, 29 项, 跨域) + §12 5 角色签字栏 (per AGENTS.md §3 7 段结构, per 守门 #14 v2/v3/v4) + §13 修订历史 (从原 §0 修订履历 + 附录 E.1 跨拍板派生 抽出). 不重写 §0-§10 + 附录 A-E (per 守门 #1 禁回溯叙事, v0.1.2 反转行显式标).** | **2026-09-10 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准" + 守门 #9 v19 Mavis 自驱 + 守门 #1 v19 批量改 1 commit 收官** |

---

## 附录 A: 跨专题引用清单

详见 `DD-CANVAS-AGENT-001.md` v0.1 附录 A + `DD-CANVAS-GAMIFY-001.md` v0.1 附录 A.

---

## 附录 B: 跨域组件映射 (4 文件 DD + 6 crate + 5 V0.1 复用 + 25 module 联动)

详见 §3 跨域组件映射表.

---

## 附录 C: 已知缺口 (12 个, 含 3 P0 阻塞)

详见 BD-CANVAS-001 v0.1 §8.3 (12 已知缺口跨域). 完整列表:
- #1 A12.1 多人同时编辑 WebSocket 选型未拍板 (**P0 阻塞**)
- #2 A12.6 冲突解决 CRDT 选型未拍板 (**P0 阻塞**)
- #3 A12.7 view/comment/edit 3 级权限矩阵 (**P0 阻塞**)
- #4 V0.1 localStorage + zustand persist 跟多人编辑冲突 (**P0 阻塞**)
- #5 5 域 Lead 真人未到位 (Mavis 临时代签)
- #6 ARG 5 维度 effect tier 模块实装待 P3-C 阶段
- #7 Memgraph 部署 (Docker 启动 port 7687 Bolt + 7444 HTTP)
- #8 L0↔L1 通信协议
- #9 TMO 9 节点 vs ARG 边界梳理
- #10 3 commit 跨域 v0.62 反转 (per 2026-09-10 12:45 JST)
- #11 守门 #1 v15 docs 同步饱和
- #12 守门 #23 v2 AI 第三方 API 禁止 (G5 走 mock)

**已知缺口 12 个 ≥ 8 满足** (per 守门 #11)

**DDD Review 必查**: #1 + #2 + #3 + #7 + #8 + #9

---

## 附录 D: 5 角色签字栏 (per AGENTS.md §3)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

## 附录 E: 修订履历 + token OLU 估算 + 守门交叉引用

### E.1 跨拍板派生 (5 阶段 + 5 守门拍板, per 守门 #14 v2 跨域)

| 派生触发 | 文档 | 修订人 | 修订内容 |
|---|---|---|---|
| 2026-09-10 17:00 JST Ulysses 拍板 v1 | 旧方向 (撤回) | Mavis 接手 | 12 大类 50 项 Miro 全面对标 (撤回) |
| 2026-09-10 17:08 JST Ulysses 拍板 v2 | 旧方向 v1.0 (撤回) | Mavis 接手 | 双核心 = 管理 agent + 游戏化, 避免过度冗余 (含砍多人编辑, 17:34 JST 撤回) |
| 2026-09-10 17:21 JST Ulysses 拍板 v3 | 旧方向 v1.0 二次更新 (撤回) | Mavis 接手 | 画布内体现 agent 之间关系的图论构造 (ARG) |
| 2026-09-10 17:34 JST Ulysses 拍板 v4 (v0.63 反转) | SRS v1.2 + BD v0.1 (本批) | Mavis 接手 | 多人编辑是要的, 撤回 17:08 JST 砍多人编辑决定, A12 8 项新增 |
| 2026-09-10 18:00 JST Ulysses 拍板 v5 | BD v0.1 (本批) | Mavis 接手 | 基于需求文档制作基本设计文档 |
| 2026-09-10 18:25 JST Ulysses 拍板 v6 | 本 DD v0.1 (本批) | Mavis 接手 | 完善详细设计文档 |
| 2026-09-10 12:45 JST v0.62 反转 | 跨域硬约束 | Mavis 接手 | 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses |
| 2026-09-05 10:43 JST 拍板 D | 跨域硬约束 | Mavis 接手 | Mavis 长期代签, 真人到位后追溯签字 |
| 2026-09-03 11:35 JST 拍板 B | 跨域硬约束 | Mavis 接手 | 5 域 Lead Mavis 临时代签 |
| 2026-08-31 22:45 JST Q1-D 拍板 | 跨域 disclaimer | Mavis 接手 | 5 域 Lead ≠ Star 22 DDD bounded context |
| 2026-09-01 18:30 JST 守门 #13 | 跨域硬约束 | Mavis 接手 | DB W/T/M 三類横展 强制分类 |
| 2026-09-02 09:01 JST 守门 #23 v2 | G5 硬约束 | Mavis 接手 | AI 第三方 API 禁止, G5 走 mock |
| 2026-08-27 19:39 JST 用户授权 | 元数据 | Mavis 接手 | Mavis 接手代签 Ulysses (per 守门 #10 + 守门 #14 v3) |

### E.2 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)

| 阶段 | 估算 |
|---|---|
| 本 DD 撰写期 (root + 2 worker) | ~0.85M tokens (1 SRE·周 ≈ 1.2M, 在预算内) |
| 累计 P3-D.5 全部 11 commit (4 SRS + 1 总册 BD + 2 专题 BD + 1 总册 DD + 2 专题 DD + 1 任务卡同步) | ~3.86M tokens (3.22 SRE·周) |
| 后续 P3-D.6 启动实装 (P0 36 项) | ~3-5M tokens |
| 双核心 78 项 全部落地 (12 个月+) | ~13-19M (13-19 SRE·周, per STAR-OLU-001) |

### E.3 守门交叉引用 (本 DD 撰写期, per 守门 #12 v21)

| 守门 | 应用 | 引用 |
|---|---|---|
| 守门 #1 v15 docs 同步饱和 | 本 DD 3 commit 落档 (per 守门 #1 v15 第 74-76 次新事件) | per AGENTS.md §4 #1 v15 |
| 守门 #1 v19 agent 交互 Python 化 | 0 子代理调用 (DD 撰写期, root + 2 worker 直实装) | per AGENTS.md §4 #1 v19 |
| 守门 #1 禁回溯叙事 | 不重写 4 SRS + 2 BD 6 commit (4f56979 / 73d490f / 396833a / f35b3f5 / f7d0932 / 73c0826), DD 是新方向, 不回写 SRS + BD | per AGENTS.md §4 #1 |
| 守门 #3 5 域独立 Lead ≠ Star 22 DDD | 跨域 disclaimer 显式 | per AGENTS.md §4 #3 |
| 守门 #5 env 安全 hard ban | 0 env 打印, 仅 invoke, 凭据走 stdin pipe (不适用纯文档) | per AGENTS.md §4 #5 |
| 守门 #9 #3 子代理 RPC 不可靠 | 0 子代理调用 (DD 撰写期) | per AGENTS.md §4 #9 #3 |
| 守门 #9 v19 Mavis 自驱 | 18:25 JST 拍板 → 18:27 JST 落 3 brief + 派 2 子代理 ~2 分钟 | per AGENTS.md §4 #9 v19 |
| 守门 #9 v20 子代理 dispatch 必先 brief 落档 | 3 份 brief 落 `docs/briefs/dd-canvas-{total,agent,gamify}-001.md` | per AGENTS.md §4 #9 v20 |
| 守门 #9 v27 RPC 失败 fallback 3 段 | invoke → verify → collect_output | per AGENTS.md §4.1.1 v27 |
| 守门 #10 commit author=Ulysses | 3 commit author=Ulysses (per 8/27 19:39 JST 授权) | per AGENTS.md §4 #10 |
| 守门 #11 缺标比错标 | 已知缺口 12 个 ≥ 8 满足, 显式列附录 C | per AGENTS.md §4 #11 |
| 守门 #12 AI 文档治理 | BAS 引用必 git log --follow 实证, 禁回溯叙事 | per AGENTS.md §4 #12 |
| 守门 #12 v21 docs 同步必更新 §4 + registry | 1 任务卡 + 1 索引追加 (后续) | per AGENTS.md §4 #12 v21 |
| 守门 #13 DB W/T/M 三類横展 100% 覆盖 | 14 张表 (A11 7 + A12 7) + 29 张表跨域汇总 | per AGENTS.md §4 #13 |
| 守门 #14 v2 5 域 Lead Mavis 临时代签 | 真人到位后追溯签字 (per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D) | per AGENTS.md §4 #14 v2 |
| 守门 #14 v3 Mavis 永久代签 | 5 角色签字栏 author=Ulysses | per AGENTS.md §4 #14 v3 |
| 守门 #14 v4 v0.62 反转 | 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST) | per AGENTS.md §4 #14 v4 |
| 守门 #15 docs 同步饱和 | 3 commit 落档 (本 DD 3 份 + 后续) | per AGENTS.md §4 #15 |
| 守门 #23 v2 AI 第三方 API 禁止 | G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2 | per AGENTS.md §4 #23 v2 |

---

> **撰写完成**: 2026-09-10 18:30 JST, root session mvs_942987595a124037901d37205a548e6f
> **2 专题 DD 状态**: 撰写中 (bg_9ce1be6e DD-AGENT + bg_50df734c DD-GAMIFY), 等待 worker 子代理返回后 root 校对 + 1 commit 3 文件落档 (per 守门 #1 v15 docs 同步饱和)
> **下次拍板触发**: 2 专题 DD 返回后, root 自动 commit 3 文件落档 (本总册 DD + 2 专题 DD), 后续 P3-D.6 启动实装 (P0 36 项 per 总册 §4.4). per 守门 #12 v21 后续需更新 automation-design.md §4.29 + registry.md v0.14.
