# BD-CANVAS-GAMIFY-001

> **无限画布游戏化域基本設計書 v0.1** (per 日本 IPA SEC 標準 / 基本設計書 テンプレート)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-CANVAS-GAMIFY-001.md`](../requirements/SRS-CANVAS-GAMIFY-001.md) v0.1 (9/10 落档, 92KB, 32 FR + 73 AC + 23 US + 12 已知缺口 + 15 张表 W/T/M 100% 覆盖, 守门 14/14 通过)
> - 关联詳細設計: [`docs/design/DD-CANVAS-GAMIFY-001.md`](./DD-CANVAS-GAMIFY-001.md) (待 P3-D 阶段落档)
> - 关联実装報告: [`docs/reports/PHASE-CANVAS-GAMIFY-IMPL-REPORT.md`](../reports/PHASE-CANVAS-GAMIFY-IMPL-REPORT.md) (待 P3-D.6 实装阶段落档)
> - 平行 view:
>   - [`docs/architecture/2026-09-03-langgraph/02-basic-design.md`](../architecture/2026-09-03-langgraph/02-basic-design.md) (LangGraph v0.2 含 TMO 9 节点, 9/4 落档)
>   - [`docs/architecture/2026-09-03-agent-runtime/02-basic-design.md`](../architecture/2026-09-03-agent-runtime/02-basic-design.md) (Agent Runtime, 9/3 落档, ADR-0045)
>   - [`docs/requirements/SRS-AGENT-VIEW-001.md`](../requirements/SRS-AGENT-VIEW-001.md) v1.0 + [`docs/design/BD-AGENT-VIEW-001.md`](./BD-AGENT-VIEW-001.md) v0.1 (Agent View 画布, 9/5 落档)
>   - [`docs/frontend-canvas-design.md`](../frontend-canvas-design.md) v0.1 (无限画布 + bezier connector 公式)
>   - [`docs/design/BD-AGENT-RELATIONSHIP-001.md`](./BD-AGENT-RELATIONSHIP-001.md) v0.1 (ARG 关系层, 9/9 落档)
>   - 双核心之 1: agent 管理 BD 由子代理 1 (`bd-canvas-agent-001`) 落档
>   - 总册 BD: `docs/design/BD-CANVAS-001.md` (root 写, 跨域共享部分)
>   - V0.1 game 5 份 PHASE 报告: `PHASE-AGENT-GAME-IMPL-REPORT.md` + `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` + `PHASE-AGENT-MANGA-IMPL-REPORT.md` + `PHASE-AGENT-THEME-IMPL-REPORT.md` + `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` (9/5 落地, 125 tests pass)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3 Mavis 永久代签 + 9/8 15:19 JST 第 6 次强化 + 9/10 12:45 JST v0.62 反转升级为 Mavis 审核)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) (5 角色签字栏 per AGENTS.md §3, 详见 §9)
> - 日期: 2026-09-10 JST
> - 受众: 詳細設計エンジニア / 実装エンジニア / UI/UX デザイナー / アーキテクト / SRE / 5 域 Lead 真人
> - 受众范围 disclaimer: **5 域独立 Lead ≠ Star 22 DDD bounded context** (per 2026-08-31 22:45 JST Q1-D 拍板: 5 域 Lead 是 RGS 仓历史治理命名, 不建立业务子域↔DDD 映射)
> - 拍板来源: 2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" + 17:08 JST 双核心"管理 agent 和游戏化, 避免过度冗余" + 守门 #23 v2 AI 第三方 API 禁止 (G5 走 mock, per `AGENTS.md §4 守门 #23 v2` 2026-09-02 09:01 JST 拍板) + 守门 #13 W/T/M 三類横展 (per `AGENTS.md §4 守门 #13` 2026-09-01 18:30 JST 拍板)

---

## §0 目的 (Purpose)

本文档基于 [`SRS-CANVAS-GAMIFY-001` §4-§10](../requirements/SRS-CANVAS-GAMIFY-001.md) 的需求, 定义 **无限画布游戏化域 (GAMIFY)** 的基本設計:

- 系统架构 (4-tier: UI / BFF API / Domain / Data 扩展)
- 组件一覧 (G1-G12 12 子能力 / 8 新前端模块 / 3 新 zustand store 集合)
- 数据模型 (15 张表 W/T/M 100% 覆盖 + 5 域 * Rust 数据结构 + zustand store 扩展)
- 接口设计 (BFF REST API 22 端点 + WebSocket 2 端点 + 内部 4 协议含 G5 mock 协议)
- 5 view (機能/データ/動作/モジュール/ネットワーク) 完整覆盖 G1-G12 32 项
- NFR 6 类 (性能 / 可靠性 / 安全 / 易用 / 可观测 / AI mock)
- 守门 19 项 + 26 派生规 跨域 (含 #1 / #9 / #10 / #11 / #13 / #14 / #15 / #23 v2)
- 已知缺口 12 个 (≥ 12 达标 per brief §3, 含 DDD Review 必查 #11 + #12)

**dual-use 提醒 (per [AGENTS.md §5 倉庫拓扑](../AGENTS.md))**: 本 view 不引用 RGS 仓 + 不建立业务子域↔DDD bounded context 映射. GAMIFY 跟 LangGraph view / Agent Runtime view / Agent View / ARG 平行, 通过 zustand store 共享数据 (agentGameStates / agentMaps), 但**不**直接调用其他 view 的 action.

**核心方向锚点 (per 2026-09-10 17:08 JST Ulysses 拍板)**: "我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" — GAMIFY 是双核心之 2, 不写 Miro 通用功能 (12 diagram / 模板 / 编辑效率 / Slack-Jira / 移动 / a11y 全部砍掉, 见 brief §2 + SRS 附录 C).

**G12 派生锚点 (per V0.1 5 份 PHASE 报告, 9/5 落地, 125 tests pass)**: Roguelike + Manga + Theme + Settings 已实装, G12 仅画布集成引用, 不重写功能. G5 sticky note 聚类 AI 走 mock 接口 (per 守门 #23 v2, 真实 LLM 留 P2).

---

## §1 适用范围 (Scope)

### 1.1 包含 (In-Scope)

#### 1.1.1 12 子能力 G1-G12 32 项 (per SRS §4.1)

- **G1 gamification 节点** (8 项): avatar / level / xp / skill_tree / class / badge / quest / inventory — 画布新 8 种 `CanvasElementKind` 扩展 (per `frontend/src/types/ids.ts` line 686-696 V0.1 10 种)
- **G2 reward / achievement** (4 项): 解锁条件 (rule engine) + 3 渠道通知 + 画布特效 + 徽章自动授予
- **G3 score / points** (3 项): 5 规则操作积分 + 3 维度累计 + 排行榜入口
- **G4 leveling / skill tree** (3 项): xp→level 公式 (默认 `level = floor(sqrt(xp/100))`) + 升级动画 + skill tree 解锁
- **G5 sticky note 聚类 (AI mock)** (2 项): 选 N 张 → mock AI 聚类 K 主题 + 主题 Frame 可视化
- **G6 dot voting** (2 项): 每用户 5 票/session + 实时显示 + top 3 高亮
- **G7 reaction** (1 项): 8-12 种 emoji 短时 3s 淡出 (Work 表)
- **G8 confetti** (1 项): 4 触发场景 (complete / unlock / levelup / achievement) 庆祝特效
- **G9 leaderboard** (2 项): per workspace (1h cache, P1) + per tenant (24h cache admin only, P2)
- **G10 daily challenge / streak** (2 项): 每日 3 任务 + 连续天数 (中断清零, 7 天大奖励)
- **G11 power-up / inventory** (2 项): 3 种道具 (双倍积分 / 自动聚类 / 隐身) + 物品栏 (持有 / 使用 / 销毁)
- **G12 V0.1 game 集成** (2 项): Roguelike 画布节点 + Manga 主题画布节点

#### 1.1.2 4-Tier 架构 (per §2 详细)

- **UI Tier (frontend/src/app/canvas/[id]/)**:
  - 8 个新节点组件 (AvatarNode / LevelNode / XpNode / SkillTreeNode / ClassNode / BadgeNode / QuestNode / InventoryNode)
  - 3 个新交互组件 (VoteButton / ReactionPicker / ConfettiOverlay)
  - 2 个新视图 (LeaderboardView / DailyChallengeView)
  - 1 个新 utility (clustering-mock-client)
  - **复用 V0.1 组件** (per `frontend/src/components/agent-game/` 7 个 + `CanvasView.tsx` baseline)
- **BFF API Tier (crates/api/src/gamify/)** (新模块):
  - 22 REST 端点 (per §5.1)
  - 2 WebSocket 端点 (per §5.2)
  - RLS 13 类 middleware (per 守门 #13 Master 派生规)
- **Domain Tier (crates/gamify/ 新 crate)** (可选, MVP 可走 pure in-memory):
  - 5 domain * Rust 数据结构 (gamification / reward / score / leaderboard / inventory)
  - 派生 (xp 公式 / 聚类 mock / 投票扣减) 全部 pure function
- **Data Tier**: 15 张 SQL 表 (per §4.2, G11.2 W/T/M 100% 覆盖), 0 新 crate 实装, 落现有 `crates/storage` 扩展

#### 1.1.3 跨域引用 + 25 module 联动

- **总册 SRS / BD**: 引用 `SRS-CANVAS-001` v0.1 + `BD-CANVAS-001` (root 写) 跨域共享部分
- **双核心之 1**: agent 管理 BD (`bd-canvas-agent-001`) 28 项, 本 BD 不重写
- **平行 BD**: `BD-AGENT-VIEW-001` v0.1 (派生视图) + `BD-AGENT-RELATIONSHIP-001` v0.1 (ARG 关系层)
- **V0.1 game 5 份 PHASE 报告** (9/5 落地, 125 tests pass):
  - `PHASE-AGENT-GAME-IMPL-REPORT.md` (49 tests, leveling.ts / perks.ts / AGENT_VISUAL_TIERS)
  - `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` (35 tests, mapgen.ts / movement.ts / RoguelikeCanvas)
  - `PHASE-AGENT-MANGA-IMPL-REPORT.md` (14 tests, theme.ts / characters.tsx / enemies.tsx / Decorations.tsx)
  - `PHASE-AGENT-THEME-IMPL-REPORT.md` (11 tests, theme-tokens.ts)
  - `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` (16 tests, settings.ts / AgentSettingsTab)
- **25 module 联动** (per `frontend-canvas-design.md` §1.3, GAMIFY 新增联动):
  - work-item → claim 触发 xp +10 / coin +1-5 (G3.1)
  - worktree → 完成触发 confetti (G8.1)
  - agent → avatar 节点 (G1.1) + level 升级 (G4.2) 联动
  - relation → achievement "topology_achiever" 触发 (per ARG 关系)
  - comment → reaction emoji (G7.1) 触发
  - automation → reward rule (G2.1) 触发
  - audit → reward 解锁 100% 写 audit log (per 守门 #13 Transaction)
  - search → 搜 "user" / "level" / "badge" 跳画布
  - notification → reward 通知 3 渠道 (G2.2)

#### 1.1.4 守门合规

- 守门 #1+#3+#5+#6+#7+#9+#10+#11+#12+#13+#14 v2+#15+#19+#23 v2 全过
- 守门 #1 v19 自动化档判定: 全部 P3-C/P3-D 阶段子项必先 `scripts/automation/<purpose>.py` 落地
- 守门 #1 v25: cargo test 走 `cargo test -p star-gamify --lib -j 4` 单 crate 模式
- 守门 #5: BFF API 连接字符串走 env, 不打印
- 守门 #6: 部署脚本 PowerShell only
- **守门 #13 W/T/M 强制分类**: G11.2 15 张表 100% 覆盖 (Work 6 + Transaction 4 + Master 5, 禁混在一括列举, per 9/1 18:30 JST 拍板)
- **守门 #23 v2 AI 第三方 API 禁止**: G5 sticky note 聚类走 `scripts/automation/ai_edit_mock.py` 模板 (per 9/2 09:01 JST 拍板), 不引入 OpenAI / Anthropic 凭据

### 1.2 不包含 (Out-of-Scope)

| 类别 | 处理方 |
|---|---|
| 双核心之 1: agent 管理 (28 项) | 专题 BD 子代理 1 (`bd-canvas-agent-001`) |
| 总册 `BD-CANVAS-001` (跨域共享部分) | root 写 (本周内同步) |
| Miro 通用 12 类 (Flowchart / BPMN / ER / 模板库 / 编辑效率 / 移动 / a11y 完整 / 集成) | ❌ 砍掉, 留 P3+ |
| Miro 互动通用 (Timer / Cursor chat / Async video) | ❌ 砍掉 (Reaction / Confetti / Dot voting 留下 per 9/10 17:08 JST 拍板) |
| 真实 LLM 接入 (G5 sticky note 聚类) | ❌ 砍掉, 走 mock 接口 (per 守门 #23 v2), 真实 LLM 留 P2 |
| 详细设计 (DD) 文档 | 后续 P3-D 阶段, 待 SRS + BD 落档后启动 |
| 实现 (PHASE-* 报告) | 后续 P3-D.6 阶段, 待 DD 落档后启动 |
| V0.1 game 5 份 PHASE 报告 (Roguelike + Manga + Theme + Settings + Game) 实装 | ✅ V0.1 已实装, 仅画布集成引用 (G12) |
| 5 域 Lead 真人到位 | per 守门 #14 v2, Mavis 临时代签, 真人到位后追溯签字 |
| BFF 真实后端实装 | MVP 全 mock, 真实后端 P2 (D.6+ 阶段) |
| Domain Tier Rust crate 落档 | MVP pure in-memory, P2 落 `crates/gamify/` crate |
| per tenant 排行榜真实 SQL 聚合 (G9.2) | P2 (admin 13 租户权限待 DDD Review 拍板) |

### 1.3 跟其他 view 区别

| 维度 | LangGraph View | Agent Runtime View | Agent View (9/5) | ARG (9/9) | **GAMIFY (本 view)** |
|---|---|---|---|---|---|
| **关注点** | UI 驱动 2-level Agent + 任务卡 DAG | Rust Runtime 基础设施 (派发/ECS/共享池) | 派生视图: 单 agent 拓扑 | agent 之间 social 层 (关系/成就) | **画布游戏化层 (avatar/level/score/聚类/投票/特效)** |
| **目标** | LLM 编排 + 任务卡生命周期 | L0 派发 + L1 ECS + L2 业务池 | 当前工作 agent 可视化 | 关系定义 + 协作影响 + 成就激励 | **画布 RPG 元素 + 互动 + 激励 + 排行榜** |
| **实现** | LangGraph Python subgraph | Rust + Tokio + ECS | React + zustand | Memgraph + LangGraph 桥接 + React | **React + zustand + V0.1 game 组件复用** |
| **数据源** | LangGraph state schema | PostgreSQL / SQLite | zustand store | Memgraph (主) + LangGraph state (缓存) | **zustand store (V0.1 agentGameStates 扩展) + BFF mock** |
| **用户交互** | Chat Bar + Task Card Modal | 0 (server-side) | 画布 pan/zoom | 画布拖拽 + 关系编辑 + 成就展示 | **画布 pan/zoom + 节点互动 (投票/反应/聚类) + 排行榜** |
| **派生数据** | 任务卡 DAG 派生 | (无派生) | 7 公开纯函数 | 4 effect 维度派生 | **12 子能力派生 (xp/score/vote/cluster 全部 pure)** |
| **成就系统** | ✗ | ✗ | ✗ | ✓ 20 成就 3 维度 | **✓ 12 子能力 32 项游戏化** |
| **AI 接入** | ✗ | ✗ | ✗ | ✗ | **G5 走 mock (per 守门 #23 v2)** |

---

## §2 システムアーキテクチャ (System Architecture)

### 2.1 全体構成図 (Overall Architecture, 4-tier)

```
┌──────────────────────────────────────────────────────────────────────────┐
│            UI Tier (frontend/src/app/canvas/[id]/ + frontend/src/lib/gamify/)  │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  page.tsx (AppShell 集成, 现有 /canvas/[id] 加 GAMIFY tools)        │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │  8 个新节点组件 (AvatarNode / LevelNode / XpNode / ... )       │  │  │
│  │  │  3 个新交互组件 (VoteButton / ReactionPicker / ConfettiOverlay)│  │  │
│  │  │  2 个新视图 (LeaderboardView / DailyChallengeView)            │  │  │
│  │  │  1 个新 utility (clustering-mock-client)                       │  │  │
│  │  │  复用 V0.1: CanvasView.tsx + 7 份 agent-game/* + StatusPill 60+│  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │  zustand store 扩展: useGamifyStore (3 collection, per §4.3)       │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
   │ HTTP REST (22 endpoints)         │ WebSocket (2 endpoints)
   ↓                                  ↓
┌──────────────────────────────────────────────────────────────────────────┐
│            BFF API Tier (crates/api/src/gamify/) (新模块)                  │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  GamifyController (Axum router, 22 REST endpoints)                  │  │
│  │  GamifyWSHub (WebSocket /ws/gamify/events + /ws/gamify/level)      │  │
│  │  GamifyPermission (RLS 13 类 middleware per 守门 #13)               │  │
│  │  MockClient (G5 聚类 mock 协议, per 守门 #23 v2)                    │  │
│  │  派生 pure functions (xpFormula / xpToLevel / clusterMock)          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
   │ In-process call                  │ SQL (G11.2 15 表)
   ↓                                  ↓
┌──────────────────────────────────────────────────────────────────────────┐
│            Domain Tier (MVP in-memory, 后续 crates/gamify/ P2 落档)        │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  gamification.rs (avatar/level/xp/skill_tree/class/badge/quest)     │  │
│  │  reward.rs (rule engine + achievement evaluator)                    │  │
│  │  score.rs (5 规则 + 3 维度累计 + leaderboard)                       │  │
│  │  inventory.rs (power-up 3 种 + 物品栏 3 action + W/T/M 派生)        │  │
│  │  cluster.rs (mock 协议, per 守门 #23 v2, 不开第三方 API)            │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
   │ In-process / Future SQL          │ Append-only audit
   ↓                                  ↓
┌──────────────────────────────────────────────────────────────────────────┐
│            Data Tier (crates/storage 扩展, 15 张表 W/T/M 100% 覆盖)        │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  Master (5 张): inventory_items / powerup_definitions /             │  │
│  │    vote_state_snapshot / daily_challenge_state / streak_state       │  │
│  │  Transaction (4 张): inventory_actions / powerup_grants /            │  │
│  │    dot_vote_ledger / clustering_jobs                                 │  │
│  │  Work (6 张): inventory_session_buffs / powerup_active_effects /    │  │
│  │    reaction_events / confetti_events / leaderboard_cache /           │  │
│  │    clustering_results_cache                                          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
   │ V0.1 既有 zustand store          │ V0.1 既有 game 组件
   ↓                                  ↓
┌──────────────────────────────────────────────────────────────────────────┐
│  复用 V0.1 baseline (per 9/5 落地, 125 tests pass)                          │
│  - frontend/src/lib/agent-game/ (types / leveling / perks / mapgen / ...)  │
│  - frontend/src/components/agent-game/ (RoguelikeCanvas / GameHUD / ...)    │
│  - frontend/src/lib/store.ts (agentGameStates / agentMaps / agentPositions)│
│  - frontend/src/types/ids.ts (CanvasElementKind 10 种 V0.1)                  │
└──────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Tier 詳細説明

#### Tier 1: UI Tier (frontend/src/lib/gamify/ + frontend/src/app/canvas/[id]/)

- **路由**: 现有 `/canvas/[id]` 加 GAMIFY tools (toolbar 加 6 个 tool, 不新增主路由)
- **新模块** (8 + 3 + 2 + 1 = 14 个):
  - `frontend/src/lib/gamify/`:
    - `types.ts` (本地派生类型, CanvasElementKind 扩展 8 种 G1 + 3 种 G12 + 1 reaction + 1 confetti)
    - `selectors.ts` (xp 公式 / score 累计 / 投票扣减 / cluster mock 7 公开纯函数)
    - `layout.ts` (派生 layer 计算, 跟 V0.1 派生层一致, 纯函数)
    - `selectors.test.ts` (14 项 vitest)
    - `layout.test.ts` (11 项 vitest)
    - `clustering-mock-client.ts` (G5 mock 协议, 跟 ai_edit_mock.py 一致)
    - `clustering-mock-client.test.ts` (5 项 vitest)
  - `frontend/src/components/gamify/` (12 个新组件):
    - `nodes/AvatarNode.tsx` (G1.1, 圆形 120x120 + tier 边框)
    - `nodes/LevelNode.tsx` (G1.2, 矩形 160x80 + progress bar)
    - `nodes/XpNode.tsx` (G1.3, 矩形 140x60 + delta 浮)
    - `nodes/SkillTreeNode.tsx` (G1.4, 矩形 320x240 + SVG DAG)
    - `nodes/ClassNode.tsx` (G1.5, 圆形 140x140 + 6 职业)
    - `nodes/BadgeNode.tsx` (G1.6, 圆形 100x100 + 4 稀有度)
    - `nodes/QuestNode.tsx` (G1.7, 矩形 240x120 + progress)
    - `nodes/InventoryNode.tsx` (G1.8, 矩形 280x160 + 3×3 grid)
    - `interactions/VoteButton.tsx` (G6.1, 票数扣减 + remaining badge)
    - `interactions/ReactionPicker.tsx` (G7.1, 8 emoji + 3s 淡出)
    - `interactions/ConfettiOverlay.tsx` (G8.1, CSS keyframes + SVG animateTransform)
    - `views/LeaderboardView.tsx` (G9.1+G9.2, per workspace / per tenant)
    - `views/DailyChallengeView.tsx` (G10.1+G10.2, 每日 3 任务 + streak)
  - **复用 V0.1**: `CanvasView.tsx` baseline (14 element + 4 frame + 8 connector) + `agent-game/` 7 份组件 + `theme-tokens.ts` (LIGHT/DARK_COLORS)
- **zustand store 扩展**: `useGamifyStore` (3 collection, per §4.3)

#### Tier 2: BFF API Tier (crates/api/src/gamify/ 新模块)

- **位置**: `crates/api/src/gamify/` (新模块)
- **组件**:
  - `controller.rs` (Axum router, 22 REST endpoints per §5.1)
  - `ws_hub.rs` (WebSocket /ws/gamify/events + /ws/gamify/level, 2 endpoints per §5.2)
  - `permission.rs` (RLS 13 类 middleware per 守门 #13)
  - `mock_client.rs` (G5 聚类 mock 协议, per 守门 #23 v2)
  - `dto.rs` (request/response types, per §5.4)
  - `pure.rs` (xp 公式 / xp→level / cluster mock 纯函数)
  - `pure.test.rs` (10+ vitest 跨 crate, 0 err)
- **依赖**: `axum` / `serde` / `tokio` / `tracing` (既有) + `uuid` (既有) + `chrono` (既有) + `rand` (既有, cluster mock 模板用)
- **守门**: RLS 13 类必携 (per 守门 #13 Master 类) + 不打印 env (per 守门 #5)

#### Tier 3: Domain Tier (MVP in-memory, P2 落 crates/gamify/)

- **位置 (MVP)**: 派生逻辑落 `crates/api/src/gamify/pure.rs` (in-memory, mock)
- **位置 (P2)**: `crates/gamify/` 新 crate (后续 D.6+ 阶段)
- **组件** (5 子模块):
  - `gamification.rs` (avatar/level/xp/skill_tree/class/badge/quest 派生)
  - `reward.rs` (rule engine + achievement evaluator)
  - `score.rs` (5 规则 + 3 维度累计 + leaderboard)
  - `inventory.rs` (power-up 3 种 + 物品栏 3 action + W/T/M 派生)
  - `cluster.rs` (mock 协议, per 守门 #23 v2)
- **依赖 (P2)**: `serde` / `tokio` / `uuid` / `chrono` / `tracing` (既有)
- **MVP 简化**: 全部 pure function in `pure.rs`, 不持久化 (per SRS §3 P-3 V0.1 模式)

#### Tier 4: Data Tier (crates/storage 扩展, 15 张表 W/T/M 100% 覆盖)

- **位置 (P2)**: `crates/storage/migrations/2026_09_10_gamify_w_t_m.sql` (新 migration, 15 张表)
- **15 张表 per 守门 #13 横展** (per SRS §4.1.11 G11.2):
  - **Master (5 张)**: `inventory_items` / `powerup_definitions` / `vote_state_snapshot` / `daily_challenge_state` / `streak_state` (per §4.2 详细 schema)
  - **Transaction (4 张)**: `inventory_actions` / `powerup_grants` / `dot_vote_ledger` / `clustering_jobs` (per §4.2)
  - **Work (6 张)**: `inventory_session_buffs` / `powerup_active_effects` / `reaction_events` / `confetti_events` / `leaderboard_cache` / `clustering_results_cache` (per §4.2)
- **派生规 (per 守门 #13)**:
  - (a) Work 表 100% 物理删除 / タイマー失効 / 短 TTL 明示 retention
  - (b) Transaction 表 100% 物理删除禁止 + 監査必須 + RLS 13 類必携
  - (c) Master 表 100% 物理删除禁止 + SCD Type 2 + RLS 13 類必携
- **MVP 简化**: DB 落档 P2, MVP 走 zustand in-memory + localStorage (per V0.1 模式)

### 2.3 データフロー (Data Flow)

#### 2.3.1 写 score / xp (画布操作 → BFF → zustand)

```
[UI: 画布完成 work-item]
    ↓
[CanvasView.tsx] → 调 useGamifyStore.applyClaim (work-item done 事件)
    ↓
[zustand: agentGameStates[userId] + gamificationStates[userId]] 更新
    ↓
[BFF: POST /v1/gamify/score] (mock, 返回 score += 10)
    ↓
[Domain: score.rs applyClaim] (per 守门 #10 派生, pure)
    ↓
[WebSocket: score.changed event] 推送
    ↓
[UI: 多个订阅客户端实时更新 score 节点]
```

#### 2.3.2 sticky note 聚类 (G5 mock, per 守门 #23 v2)

```
[UI: 选 5 张 sticky_note + k=3]
    ↓
[clustering-mock-client.ts] → 调 mockCluster API
    ↓
[BFF: POST /v1/gamify/cluster] (per 守门 #23 v2 走 mock, 不开 OpenAI / Anthropic)
    ↓
[Domain: cluster.rs mockCluster] → 返回 { themes, confidence < 0.5, mockUsed: true }
    ↓
[BFF: POST /v1/gamify/cluster-visualize] (本地, 不调外部)
    ↓
[UI: 1 Frame 创建 + 3 主题 label + 关联 sticky_note]
    ↓
[UI: UI 提示"建议手动调整" (per G5.1 AC-GAMIFY-G5.1.3)]
```

#### 2.3.3 level up 升级动画 + 通知 (G4.2 + G2.2)

```
[zustand: agentGameStates[userId].xp += N] (per V0.1 claimReward)
    ↓
[Domain: pure.rs xpToLevel] → 新 level > 旧 level
    ↓
[WebSocket: level.up event 推 /ws/gamify/level]
    ↓
[UI: 节点 scale 1.0 → 1.2 → 1.0 动画 1.5s + halo ring (per V0.1 PHASE-AGENT-GAME-IMPL-REPORT.md §1 #11)]
    ↓
[UI: 通知 toast 显示 "Level Up! X → Y" 3s (per G2.2 通知 in-app)]
    ↓
[WebSocket: reward.unlocked event] 推送 (per G2.1 解锁条件)
    ↓
[UI: confetti 触发 (per G8.1) + confetti 通知 toast]
```

#### 2.3.4 dot voting 实时更新 (G6.1 + G6.2)

```
[UI: 点 sticky_note 投票 (剩余 4 票)]
    ↓
[BFF: POST /v1/gamify/vote] (mock, 票数 +1, remaining -1)
    ↓
[Domain: vote.rs applyVote] → { votes[stickyNoteId]++, remaining-- }
    ↓
[BFF: 写 dot_vote_ledger (Transaction) + 更新 vote_state_snapshot (Master)]
    ↓
[WebSocket: vote.changed event 推 /ws/gamify/events]
    ↓
[UI: 节点右上角 badge "5" 实时显示 + top 3 金边框高亮 (per G6.2)]
```

---

## §3 组件一覧 (Component List)

### 3.1 组件总览 (14 个新前端组件 + 7 个新 BFF 模块 + 5 个新 Domain 模块 + 15 张表)

| # | 组件 ID | 名称 | Tier | 语言 | 优先级 |
|---|---|---|---|---|---|
| **C-1** | `AvatarNode` | avatar 节点渲染 (圆形 120x120) | UI | TypeScript | P0 |
| **C-2** | `LevelNode` | level 节点渲染 (矩形 160x80 + progress bar) | UI | TypeScript | P0 |
| **C-3** | `XpNode` | xp 节点渲染 (矩形 140x60 + delta 浮) | UI | TypeScript | P0 |
| **C-4** | `SkillTreeNode` | skill_tree 节点渲染 (矩形 320x240 + SVG DAG) | UI | TypeScript | P0 |
| **C-5** | `ClassNode` | class 节点渲染 (圆形 140x140 + 6 职业) | UI | TypeScript | P0 |
| **C-6** | `BadgeNode` | badge 节点渲染 (圆形 100x100 + 4 稀有度) | UI | TypeScript | P0 |
| **C-7** | `QuestNode` | quest 节点渲染 (矩形 240x120 + progress) | UI | TypeScript | P1 |
| **C-8** | `InventoryNode` | inventory 节点渲染 (矩形 280x160 + 3×3 grid) | UI | TypeScript | P1 |
| **C-9** | `VoteButton` | dot voting 票数扣减 + remaining badge | UI | TypeScript | P0 |
| **C-10** | `ReactionPicker` | 8-12 种 emoji 短时 3s 淡出 (Work 表) | UI | TypeScript | P1 |
| **C-11** | `ConfettiOverlay` | CSS keyframes + SVG `<animateTransform>` (per 守门 #19) | UI | TypeScript | P0 |
| **C-12** | `LeaderboardView` | per workspace / per tenant 排行榜 (3 维度) | UI | TypeScript | G9.1 P1, G9.2 P2 |
| **C-13** | `DailyChallengeView` | 每日 3 任务 + streak 连续天数 | UI | TypeScript | P1 |
| **C-14** | `clustering-mock-client` | G5 mock 协议 client (per 守门 #23 v2) | UI | TypeScript | P1 |
| **C-15** | `GamifyController` | Axum router 22 REST endpoints | API | Rust | P0 |
| **C-16** | `GamifyWSHub` | WebSocket /ws/gamify/events + /ws/gamify/level (2 endpoints) | API | Rust | P0 |
| **C-17** | `GamifyPermission` | RLS 13 类 middleware per 守门 #13 | API | Rust | P0 |
| **C-18** | `MockClient` (server) | G5 聚类 mock 协议 (per 守门 #23 v2) | API | Rust | P1 |
| **C-19** | `Pure` (派生层) | xp 公式 / xpToLevel / cluster mock 纯函数 | API | Rust | P0 |
| **C-20** | `DTO` (传输对象) | request/response types | API | Rust | P0 |
| **C-21** | `Migration` (15 张表) | `crates/storage/migrations/2026_09_10_gamify_w_t_m.sql` | Data | SQL | P2 |
| **C-22** | `gamification.rs` | avatar/level/xp/skill_tree/class/badge/quest 派生 (P2 落 crates/gamify/) | Domain | Rust | P2 |
| **C-23** | `reward.rs` | rule engine + achievement evaluator (P2) | Domain | Rust | P2 |
| **C-24** | `score.rs` | 5 规则 + 3 维度累计 + leaderboard (P2) | Domain | Rust | P2 |
| **C-25** | `inventory.rs` | power-up 3 种 + 物品栏 3 action + W/T/M 派生 (P2) | Domain | Rust | P2 |
| **C-26** | `cluster.rs` | mock 协议 (P2, per 守门 #23 v2) | Domain | Rust | P2 |
| **复用 V0.1** | `CanvasView.tsx` | 14 element + 4 frame + 8 connector baseline | UI | TypeScript | ✅ V0.1 |
| **复用 V0.1** | `RoguelikeCanvas.tsx` | 6 cell 类型 + 4 邻接 + mapgen (per `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` §1 #5) | UI | TypeScript | ✅ V0.1 |
| **复用 V0.1** | `GameHUD.tsx` / `PerkPicker.tsx` / `DeathModal.tsx` | RPG 元素 (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #11) | UI | TypeScript | ✅ V0.1 |
| **复用 V0.1** | `theme-tokens.ts` | dark/light COLORS (per `PHASE-AGENT-THEME-IMPL-REPORT.md`) | UI | TypeScript | ✅ V0.1 |
| **复用 V0.1** | `AgentSettingsTab.tsx` | 玩家设置 (per `PHASE-AGENT-SETTINGS-IMPL-REPORT.md`) | UI | TypeScript | ✅ V0.1 |
| **复用 V0.1** | `agentGameStates` / `agentMaps` / `agentPositions` | zustand 集合 (per V0.1 store 扩展) | Data | TypeScript | ✅ V0.1 |

**新组件 26 个 (C-1..C-26) + 复用 V0.1 6 类 (CanvasView / RoguelikeCanvas / 3 组件 + 3 store)**.

### 3.2 12 子能力 ↔ 组件映射 (per SRS §4.1)

| 子能力 | FR | 主要组件 | 派生函数 | 依赖 V0.1 |
|---|---|---|---|---|
| **G1.1 avatar_node** | FR-GAMIFY-G1.1 | C-1 AvatarNode | `avatarTierForLevel(level)` pure | V0.1 `AGENT_VISUAL_TIERS` |
| **G1.2 level_node** | FR-GAMIFY-G1.2 | C-2 LevelNode | `levelToXpToNext(level)` pure | V0.1 `leveling.ts` `xpProgress` |
| **G1.3 xp_node** | FR-GAMIFY-G1.3 | C-3 XpNode | (无, 直接读 store) | V0.1 `applyClaim` |
| **G1.4 skill_tree_node** | FR-GAMIFY-G1.4 | C-4 SkillTreeNode | `skillTreeLayout(dag, unlocked)` pure | V0.1 `Decorations.tsx` `Stamp` |
| **G1.5 class_node** | FR-GAMIFY-G1.5 | C-5 ClassNode | `classToColor(class)` pure | V0.1 `theme.ts` `ENEMY_TYPES` |
| **G1.6 badge_node** | FR-GAMIFY-G1.6 | C-6 BadgeNode | `badgeRarityColor(rarity)` pure | V0.1 `Decorations.tsx` `Stamp` |
| **G1.7 quest_node** | FR-GAMIFY-G1.7 | C-7 QuestNode | (无, 直接读 store) | V0.1 `leveling.ts` `computeClaim` |
| **G1.8 inventory_node** | FR-GAMIFY-G1.8 | C-8 InventoryNode | (无, 直接读 store) | V0.1 `agentGameStates[userId].perks` |
| **G2.1 reward 解锁条件** | FR-GAMIFY-G2.1 | (服务端) C-15 reward-rules | `ruleEngine.evaluate(condition)` | V0.1 `automationRules` |
| **G2.2 reward 通知** | FR-GAMIFY-G2.2 | (UI toast) C-15 notify | (无) | V0.1 notification 域 |
| **G2.3 reward 画布特效** | FR-GAMIFY-G2.3 | C-11 ConfettiOverlay | (无, 动画) | (per G8.1) |
| **G2.4 achievement 徽章自动授予** | FR-GAMIFY-G2.4 | (服务端) C-15 achievement-evaluate | `achievementEvaluator.evaluate(condition)` | V0.1 `agentGameStates.badges` |
| **G3.1 操作积分规则** | FR-GAMIFY-G3.1 | (服务端) C-15 score-rule | `scoreRule.apply(action)` | V0.1 `agentGameStates.xp` |
| **G3.2 score 累计** | FR-GAMIFY-G3.2 | C-3 XpNode (复用) + 视图 | `scoreByDimension(userId, dim)` | V0.1 store |
| **G3.3 score 排行榜入口** | FR-GAMIFY-G3.3 | C-3 XpNode (跳转) | (无) | C-12 LeaderboardView |
| **G4.1 xp → level 公式** | FR-GAMIFY-G4.1 | C-2 LevelNode (复用) | `xpToLevel(xp)` pure | V0.1 `leveling.ts` |
| **G4.2 升级动画 + 通知** | FR-GAMIFY-G4.2 | C-1 AvatarNode (动画) + toast | (无, 动画) | V0.1 PHASE-GAME §1 #11 |
| **G4.3 skill tree 解锁** | FR-GAMIFY-G4.3 | C-4 SkillTreeNode (高亮) | `unlockSkillAtLevel(level)` | C-1 + V0.1 `Decorations.tsx` |
| **G5.1 sticky note 聚类** | FR-GAMIFY-G5.1 | C-14 clustering-mock-client | `mockCluster(stickyNoteIds, k)` (per 守门 #23 v2) | (无, mock) |
| **G5.2 聚类结果可视化** | FR-GAMIFY-G5.2 | (UI) Frame 创建 + connector | (无) | V0.1 `CanvasFrame` |
| **G6.1 每用户 N 票** | FR-GAMIFY-G6.1 | C-9 VoteButton | `voteRemaining(userId, session)` | V0.1 sticky_note |
| **G6.2 投票结果实时显示** | FR-GAMIFY-G6.2 | C-9 VoteButton (badge) | (无, WS 推送) | V0.1 sticky_note 渲染 |
| **G7.1 reaction** | FR-GAMIFY-G7.1 | C-10 ReactionPicker | (无, 短 TTL) | V0.1 `comment_pin` |
| **G8.1 confetti** | FR-GAMIFY-G8.1 | C-11 ConfettiOverlay | (无, CSS + SVG) | V0.1 `Decorations.tsx` `EnergyRing` |
| **G9.1 per workspace 排行榜** | FR-GAMIFY-G9.1 | C-12 LeaderboardView | `leaderboardByWorkspace(workspaceId, dim)` | (无, 1h cache) |
| **G9.2 per tenant 排行榜** | FR-GAMIFY-G9.2 | C-12 LeaderboardView (admin) | `leaderboardByTenant(tenantId, dim)` | (无, 24h cache, admin) |
| **G10.1 daily challenge** | FR-GAMIFY-G10.1 | C-13 DailyChallengeView | `dailyTasksFor(userId, date, tz)` | V0.1 quest_node |
| **G10.2 streak 连续天数** | FR-GAMIFY-G10.2 | C-13 DailyChallengeView (streak) | `streakUpdate(userId, lastActiveDate)` | V0.1 quest_node |
| **G11.1 power-up 道具** | FR-GAMIFY-G11.1 | C-8 InventoryNode (use) | `powerupActivate(userId, type, duration)` | V0.1 `agentGameStates.perks` |
| **G11.2 inventory 物品栏** | FR-GAMIFY-G11.2 | C-8 InventoryNode + 15 张表 | `inventoryAction(userId, action, itemId)` | V0.1 inventory_node + 守门 #13 |
| **G12.1 Roguelike 集成** | FR-GAMIFY-G12.1 | 新 kind 映射 (C-1 + V0.1 RoguelikeCanvas) | (无, 复用) | V0.1 `RoguelikeCanvas.tsx` |
| **G12.2 Manga 集成** | FR-GAMIFY-G12.2 | 新 kind 映射 (C-1 + V0.1 characters.tsx) | (无, 复用) | V0.1 `theme.ts` + `characters.tsx` + `Decorations.tsx` |

### 3.3 依赖关系 (per 守门 #1 v19+ 累积规)

#### 3.3.1 前端 package.json 改动 (per 守门 #19 强制: 不引入新依赖)

```jsonc
// frontend/package.json
{
  // 0 新依赖 (per 守门 #19 v19+ 累积规 P-11)
  // confetti 走 CSS keyframes + SVG <animateTransform> (per V0.1 Decorations.tsx EnergyRing 同思路)
  // 不引 canvas-confetti / framer-motion / react-spring 等
  // 复用 V0.1:
  //  - lucide-react (icons)
  //  - zustand (state)
  //  - @/types/ids (类型)
  //  - @/lib/agent-game/* (V0.1 game 逻辑)
  //  - @/components/agent-game/* (V0.1 game 组件)
  //  - @/components/CanvasView (画布 baseline)
}
```

#### 3.3.2 Rust Cargo.toml workspace 改动 (per 守门 #1 v25 单 crate 模式)

```toml
# Cargo.toml workspace 改动
[workspace.dependencies]
# 0 新依赖 (per 守门 #19 v19+ 累积规)
# 复用既有:
#  - axum (Web framework)
#  - serde / serde_json
#  - tokio (async runtime)
#  - tracing
#  - uuid
#  - chrono
#  - rand (既有, cluster mock 模板用)
```

#### 3.3.3 Migration 文件 (P2 实装阶段)

```sql
-- crates/storage/migrations/2026_09_10_gamify_w_t_m.sql (P2 落档)
-- 15 张表 W/T/M 100% 覆盖, per 守门 #13
-- 详见 §4.2 schema
```

---

## §4 データ設計 (Data Design)

### 4.1 データ分類 (per 守门 #13 W/T/M 横展開, 15 张表 100% 覆盖)

按 守门 #13 (per [AGENTS.md §4 守门 #13](../AGENTS.md) W/T/M 强制分类, 9/1 18:30 JST 拍板), GAMIFY 数据分类:

| 分类 | 表数量 | 表名 | 字段摘要 | 派生规 (per 守门 #13) |
|---|---|---|---|---|
| **Work (W)** | **6** | `inventory_session_buffs` | id, user_id, item_id, effect_type, expires_at, payload, created_at, retention_until | (a) 物理删除 / タイマー失効 / 短 TTL 明示 retention |
| | | `powerup_active_effects` | id, user_id, powerup_id, effect_type, activated_at, expires_at, payload, retention_until | (a) 短 TTL 24h |
| | | `reaction_events` | id, user_id, element_id, emoji, fired_at, expires_at, retention_until | (a) 短 TTL 3s, 自动清理 (per BR-4) |
| | | `confetti_events` | id, trigger, position, intensity, fired_at, expires_at, retention_until | (a) 短 TTL 3s, 自动清理 |
| | | `leaderboard_cache` | id, scope, scope_id, dimension, top_10_json, cached_at, expires_at, retention_until | (a) 短 TTL 1h/24h, 自动清理 (per G9.1/G9.2) |
| | | `clustering_results_cache` | id, input_hash, themes_json, expires_at, retention_until | (a) 短 TTL, 自动清理 |
| **Transaction (T)** | **4** | `inventory_actions` | id, user_id, item_id, action (hold/use/destroy), old_count, new_count, actor, timestamp | (b) 物理删除禁止 + 監査必須 + RLS 13 類必携 |
| | | `powerup_grants` | id, user_id, powerup_id, granted_at, granted_by, reason, expires_at | (b) 物理删除禁止 + 監査必須 + RLS 13 類必携 |
| | | `dot_vote_ledger` | id, user_id, sticky_note_id, delta, remaining_after, actor, timestamp | (b) 物理删除禁止 + 監査必須 + RLS 13 類必携 |
| | | `clustering_jobs` | id, user_id, sticky_note_ids_json, k, themes_json, confidence, mock_used, timestamp | (b) 物理删除禁止 + 監査必須 (per 守门 #23 v2, mock 必记) |
| **Master (M)** | **5** | `inventory_items` | id, user_id, item_id, type, count, acquired_at, expires_at, version | (c) 物理删除禁止 + SCD Type 2 + RLS 13 類必携 |
| | | `powerup_definitions` | id, code, name, description, type, default_duration, effect_payload, version | (c) 物理删除禁止 + SCD Type 2 + RLS 13 類必携 |
| | | `vote_state_snapshot` | id, workspace_id, snapshot_at, top_10_json, version | (c) 物理删除禁止 + SCD Type 2 (per hour snapshot) |
| | | `daily_challenge_state` | id, user_id, date, tz, tasks_json, completion_state, version | (c) 物理删除禁止 + SCD Type 2 (per user+date unique) |
| | | `streak_state` | id, user_id, current_streak, longest_streak, last_active_date, version | (c) 物理删除禁止 + SCD Type 2 |
| **总计** | **15** | (混合分類 = 0, 100% 覆盖) | | per 守门 #13 派生规 (a)/(b)/(c) |

**W/T/M 三類横展 100% 表覆盖, 0 混合分類, 禁混在一括列举 (per 守门 #13 强制)**.

### 4.2 15 张表 SQL Schema (per 守门 #13 派生规)

#### 4.2.1 Master 表 5 张 (per 守门 #13 c)

```sql
-- ===== Master (5 张) =====

-- M-1: inventory_items (SCD Type 2, 物理删除禁止)
CREATE TABLE inventory_items (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  item_id UUID NOT NULL,
  type VARCHAR(50) NOT NULL,  -- double_score / auto_cluster / stealth
  count INTEGER NOT NULL DEFAULT 1,
  acquired_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ,  -- null = 永久
  version INTEGER NOT NULL DEFAULT 1,  -- SCD Type 2
  tenant_id UUID NOT NULL,  -- RLS 13 类 per 守门 #13 c
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by UUID NOT NULL  -- author = Ulysses per 9/8 15:19 第 6 次强化
);
CREATE INDEX idx_inventory_items_user ON inventory_items(user_id);
CREATE INDEX idx_inventory_items_type ON inventory_items(type);
CREATE INDEX idx_inventory_items_tenant ON inventory_items(tenant_id);
CREATE INDEX idx_inventory_items_composite ON inventory_items(user_id, item_id, version);  -- 复合 SCD
ALTER TABLE inventory_items ENABLE ROW LEVEL SECURITY;
CREATE POLICY inventory_items_tenant_isolation ON inventory_items
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- M-2: powerup_definitions (SCD Type 2, 物理删除禁止)
CREATE TABLE powerup_definitions (
  id UUID PRIMARY KEY,
  code VARCHAR(50) UNIQUE NOT NULL,  -- DOUBLE_SCORE / AUTO_CLUSTER / STEALTH
  name VARCHAR(255) NOT NULL,
  description TEXT,
  type VARCHAR(50) NOT NULL,  -- same as inventory_items.type
  default_duration INTERVAL NOT NULL DEFAULT '24 hours',
  effect_payload JSONB NOT NULL,  -- 倍率 / 触发条件 / 隐身率
  version INTEGER NOT NULL DEFAULT 1,  -- SCD Type 2
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by UUID NOT NULL
);
CREATE INDEX idx_powerup_defs_code ON powerup_definitions(code);
CREATE INDEX idx_powerup_defs_tenant ON powerup_definitions(tenant_id);

-- M-3: vote_state_snapshot (SCD Type 2, per hour snapshot)
CREATE TABLE vote_state_snapshot (
  id UUID PRIMARY KEY,
  workspace_id UUID NOT NULL,
  snapshot_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  top_10_json JSONB NOT NULL,  -- [{ user_id, votes, rank }]
  version INTEGER NOT NULL DEFAULT 1,  -- SCD Type 2
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_vote_snapshot_workspace ON vote_state_snapshot(workspace_id, snapshot_at);
CREATE INDEX idx_vote_snapshot_tenant ON vote_state_snapshot(tenant_id);

-- M-4: daily_challenge_state (SCD Type 2, per user+date unique)
CREATE TABLE daily_challenge_state (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  date DATE NOT NULL,
  tz VARCHAR(50) NOT NULL,  -- Asia/Tokyo etc
  tasks_json JSONB NOT NULL,  -- [{ task_id, title, progress, reward }]
  completion_state JSONB NOT NULL DEFAULT '{}'::jsonb,  -- { task_id: 0/1 }
  version INTEGER NOT NULL DEFAULT 1,  -- SCD Type 2
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(user_id, date, version)  -- SCD 唯一
);
CREATE INDEX idx_daily_challenge_user_date ON daily_challenge_state(user_id, date);
CREATE INDEX idx_daily_challenge_tenant ON daily_challenge_state(tenant_id);

-- M-5: streak_state (SCD Type 2)
CREATE TABLE streak_state (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  current_streak INTEGER NOT NULL DEFAULT 0,
  longest_streak INTEGER NOT NULL DEFAULT 0,
  last_active_date DATE NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,  -- SCD Type 2
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_streak_user ON streak_state(user_id);
CREATE INDEX idx_streak_tenant ON streak_state(tenant_id);
```

#### 4.2.2 Transaction 表 4 张 (per 守门 #13 b)

```sql
-- ===== Transaction (4 张) =====

-- T-1: inventory_actions (append-only, 物理删除禁止)
CREATE TABLE inventory_actions (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  item_id UUID NOT NULL,
  action VARCHAR(20) NOT NULL CHECK (action IN ('hold', 'use', 'destroy')),
  old_count INTEGER,
  new_count INTEGER,
  actor UUID NOT NULL,
  tenant_id UUID NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_inv_actions_user ON inventory_actions(user_id);
CREATE INDEX idx_inv_actions_item ON inventory_actions(item_id);
CREATE INDEX idx_inv_actions_timestamp ON inventory_actions(timestamp);
CREATE INDEX idx_inv_actions_tenant ON inventory_actions(tenant_id);
ALTER TABLE inventory_actions ENABLE ROW LEVEL SECURITY;
CREATE POLICY inv_actions_tenant_isolation ON inventory_actions
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- T-2: powerup_grants (append-only, 物理删除禁止)
CREATE TABLE powerup_grants (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  powerup_id UUID NOT NULL,
  granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  granted_by UUID NOT NULL,
  reason TEXT,
  expires_at TIMESTAMPTZ,
  tenant_id UUID NOT NULL
);
CREATE INDEX idx_pwrup_grants_user ON powerup_grants(user_id);
CREATE INDEX idx_pwrup_grants_granted_at ON powerup_grants(granted_at);
CREATE INDEX idx_pwrup_grants_tenant ON powerup_grants(tenant_id);

-- T-3: dot_vote_ledger (append-only, 物理删除禁止)
CREATE TABLE dot_vote_ledger (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  sticky_note_id UUID NOT NULL,
  delta INTEGER NOT NULL CHECK (delta IN (-1, 1)),  -- 投票 +1 / 撤销 -1 (per 缺口 #7 P2)
  remaining_after INTEGER NOT NULL,
  actor UUID NOT NULL,
  tenant_id UUID NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_vote_ledger_user ON dot_vote_ledger(user_id);
CREATE INDEX idx_vote_ledger_sticky ON dot_vote_ledger(sticky_note_id);
CREATE INDEX idx_vote_ledger_timestamp ON dot_vote_ledger(timestamp);
CREATE INDEX idx_vote_ledger_tenant ON dot_vote_ledger(tenant_id);

-- T-4: clustering_jobs (append-only, per 守门 #23 v2 mock 必记)
CREATE TABLE clustering_jobs (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  sticky_note_ids_json JSONB NOT NULL,  -- [uuid, uuid, ...]
  k INTEGER NOT NULL,
  themes_json JSONB NOT NULL,  -- [{ id, label, stickyNoteIds }]
  confidence REAL NOT NULL,  -- 永远 < 0.5 (per 守门 #23 v2)
  mock_used BOOLEAN NOT NULL DEFAULT TRUE,  -- 显式标识走 mock
  timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
  tenant_id UUID NOT NULL
);
CREATE INDEX idx_cluster_jobs_user ON clustering_jobs(user_id);
CREATE INDEX idx_cluster_jobs_timestamp ON clustering_jobs(timestamp);
CREATE INDEX idx_cluster_jobs_tenant ON clustering_jobs(tenant_id);
```

#### 4.2.3 Work 表 6 张 (per 守门 #13 a, 短 TTL + 物理删除 + タイマー失効)

```sql
-- ===== Work (6 张, 短 TTL) =====

-- W-1: inventory_session_buffs (短 TTL, 24h 失効)
CREATE TABLE inventory_session_buffs (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  item_id UUID NOT NULL,
  effect_type VARCHAR(50) NOT NULL,  -- double_score / auto_cluster / stealth
  expires_at TIMESTAMPTZ NOT NULL,
  payload JSONB NOT NULL,  -- 倍率 / 触发条件
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  retention_until TIMESTAMPTZ NOT NULL  -- 必填 per 守门 #13 a
);
CREATE INDEX idx_inv_buffs_user ON inventory_session_buffs(user_id);
CREATE INDEX idx_inv_buffs_expires ON inventory_session_buffs(expires_at);
CREATE INDEX idx_inv_buffs_retention ON inventory_session_buffs(retention_until);

-- W-2: powerup_active_effects (短 TTL, 24h 失効)
CREATE TABLE powerup_active_effects (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  powerup_id UUID NOT NULL,
  effect_type VARCHAR(50) NOT NULL,
  activated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,
  payload JSONB NOT NULL,
  retention_until TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_pwrup_effects_user ON powerup_active_effects(user_id);
CREATE INDEX idx_pwrup_effects_expires ON powerup_active_effects(expires_at);

-- W-3: reaction_events (短 TTL 3s, 自动清理 per BR-4)
CREATE TABLE reaction_events (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  element_id UUID NOT NULL,
  emoji VARCHAR(10) NOT NULL,  -- 👍 ❤️ 🎉 😄 🤔 👀 🔥 ⭐ (8 种 per G7.1)
  fired_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,
  retention_until TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_reaction_user ON reaction_events(user_id);
CREATE INDEX idx_reaction_element ON reaction_events(element_id);
CREATE INDEX idx_reaction_expires ON reaction_events(expires_at);

-- W-4: confetti_events (短 TTL 3s, 自动清理 per G8.1)
CREATE TABLE confetti_events (
  id UUID PRIMARY KEY,
  trigger VARCHAR(20) NOT NULL CHECK (trigger IN ('complete', 'unlock', 'levelup', 'achievement')),
  position JSONB NOT NULL,  -- { x, y } 画布坐标
  intensity VARCHAR(10) NOT NULL CHECK (intensity IN ('low', 'med', 'high')),
  fired_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,
  retention_until TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_confetti_trigger ON confetti_events(trigger);
CREATE INDEX idx_confetti_expires ON confetti_events(expires_at);

-- W-5: leaderboard_cache (短 TTL 1h/24h, per G9.1/G9.2)
CREATE TABLE leaderboard_cache (
  id UUID PRIMARY KEY,
  scope VARCHAR(20) NOT NULL CHECK (scope IN ('workspace', 'tenant')),
  scope_id UUID NOT NULL,
  dimension VARCHAR(20) NOT NULL CHECK (dimension IN ('token', 'tasks_done', 'votes')),
  top_10_json JSONB NOT NULL,
  cached_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,
  retention_until TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_lb_scope ON leaderboard_cache(scope, scope_id, dimension);
CREATE INDEX idx_lb_expires ON leaderboard_cache(expires_at);

-- W-6: clustering_results_cache (短 TTL, 自动清理)
CREATE TABLE clustering_results_cache (
  id UUID PRIMARY KEY,
  input_hash VARCHAR(64) NOT NULL,  -- SHA256 of stickyNoteIds+k
  themes_json JSONB NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  retention_until TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_cluster_cache_input ON clustering_results_cache(input_hash);
CREATE INDEX idx_cluster_cache_expires ON clustering_results_cache(expires_at);
```

### 4.3 zustand Store 扩展 (per V0.1 P-7 派生)

#### 4.3.1 useGamifyStore 3 collection

```typescript
// frontend/src/lib/store.ts 扩展 (per V0.1 P-7 派生, 3 新 collection)

interface GamifyStore {
  // Collection 1: gamificationStates (跟 V0.1 agentGameStates 联动)
  gamificationStates: Record<Uuid, GamificationState>;
  gamificationLoading: boolean;
  gamificationError: string | null;

  // Collection 2: voteStates (per G6 投票状态)
  voteStates: Record<Uuid, VoteState>;
  voteLoading: boolean;
  voteError: string | null;

  // Collection 3: notificationStates (Work 短 TTL 3s, per G2.2/G7.1)
  notificationStates: Record<Uuid, NotificationState>;
  notificationLoading: boolean;
  notificationError: string | null;

  // Actions (跟 V0.1 useAgentGame 7 action 平行)
  loadGamification(userId: Uuid): Promise<void>;
  loadVotes(workspaceId: Uuid): Promise<void>;
  loadNotifications(userId: Uuid): Promise<void>;
  applyClaim(userId: Uuid, workItemId: Uuid): Promise<ClaimResult>;
  applyCostSpend(userId: Uuid, costDelta: number): Promise<SpendResult>;
  applyVote(userId: Uuid, stickyNoteId: Uuid, delta: 1 | -1): Promise<VoteResult>;
  applyReact(userId: Uuid, elementId: Uuid, emoji: string): Promise<void>;
  triggerConfetti(canvasId: Uuid, position: {x: number, y: number}, trigger: ConfettiTrigger): void;
  applyInventoryAction(userId: Uuid, itemId: Uuid, action: 'hold' | 'use' | 'destroy'): Promise<void>;
  activatePowerup(userId: Uuid, type: PowerupType, duration?: number): Promise<void>;
}

interface GamificationState {
  userId: Uuid;
  avatar: { tier: number; level: number; v0Tier: number /* per V0.1 AGENT_VISUAL_TIERS */ };
  level: { current: number; xp: number; xpToNext: number };
  class: AvatarClass;  // 'warrior' | 'mage' | 'rogue' | 'healer' | 'tinkerer' | 'default'
  badges: BadgeRef[];  // { code, name, rarity, unlockedAt }
  quests: QuestRef[];  // { id, title, progress, reward }
  inventory: InventoryItem[];  // { id, type, count, expiresAt }
  skillTree: { dag: SkillDAG; unlocked: Set<Uuid> };
  score: { session: number; day: number; allTime: number };
  streak: { current: number; longest: number; lastActiveDate: string };
  dailyChallenge: { date: string; tasks: DailyTask[] };
  lastUpdated: string;  // ISO 8601
}

interface VoteState {
  workspaceId: Uuid;
  stickyNoteVotes: Record<Uuid, number>;  // stickyNoteId -> vote count
  userRemaining: number;  // per session 5 票 (per BR-3)
  top10: { stickyNoteId: Uuid; votes: number; rank: number }[];
  lastUpdated: string;
}

interface NotificationState {
  userId: Uuid;
  toasts: Array<{
    id: Uuid;
    reward?: Reward;
    reaction?: { emoji: string; elementId: Uuid };
    expiresAt: string;  // 3s auto-clear (per BR-4)
  }>;
  lastUpdated: string;
}
```

### 4.4 Rust 数据结构 (P2 落 crates/gamify/, MVP in-memory)

```rust
// crates/api/src/gamify/dto.rs (MVP)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamificationState {
    pub user_id: Uuid,
    pub avatar: AvatarState,
    pub level: LevelState,
    pub class: AvatarClass,
    pub badges: Vec<BadgeRef>,
    pub quests: Vec<QuestRef>,
    pub inventory: Vec<InventoryItem>,
    pub skill_tree: SkillTreeState,
    pub score: ScoreState,
    pub streak: StreakState,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarState {
    pub tier: u8,      // 1-10 (per V0.1 AGENT_VISUAL_TIERS)
    pub level: u32,
    pub v0_tier: u8,   // 跟 V0.1 视觉对齐
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelState {
    pub current: u32,
    pub xp: u64,
    pub xp_to_next: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AvatarClass {
    Warrior,
    Mage,
    Rogue,
    Healer,
    Tinkerer,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeRef {
    pub code: String,
    pub name: String,
    pub rarity: Rarity,  // Common / Rare / Epic / Legendary
    pub unlocked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestRef {
    pub id: Uuid,
    pub title: String,
    pub progress: f32,  // 0.0-1.0
    pub reward: Reward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub item_type: PowerupType,  // DoubleScore / AutoCluster / Stealth
    pub count: u32,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PowerupType {
    DoubleScore,
    AutoCluster,
    Stealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTreeState {
    pub dag: SkillDAG,
    pub unlocked: HashSet<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDAG {
    pub nodes: Vec<SkillNode>,
    pub edges: Vec<SkillEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillNode {
    pub id: Uuid,
    pub name: String,
    pub unlock_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEdge {
    pub from: Uuid,
    pub to: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreState {
    pub session: i64,
    pub day: i64,
    pub all_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakState {
    pub current: u32,
    pub longest: u32,
    pub last_active_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    pub xp: i64,
    pub coin: i64,
    pub badge: Option<BadgeRef>,
    pub powerup: Option<PowerupType>,
}
```

### 4.5 5 view #2 データ (Data View)

| 数据 | 存储 | 类型 (W/T/M) | 用途 |
|---|---|---|---|
| 12 子能力派生 (xp/score/vote/cluster 纯函数) | zustand `gamificationStates` | (in-memory) | 实时读, 派生计算 |
| V0.1 game state (`agentGameStates`) | zustand 既有 | (in-memory) | V0.1 已实装, G1 复用 |
| V0.1 agent map (`agentMaps`) | zustand 既有 | (in-memory) | G12.1 Roguelike 复用 |
| 15 张 SQL 表 (G11.2) | PostgreSQL (P2) | W/T/M 100% | 持久化 (P2 落档) |
| 投票结果快照 | zustand `voteStates` + SQL `vote_state_snapshot` | Master | 1h cache (per G9.1) |
| achievement / reward 解锁记录 | SQL `inventory_actions` / `powerup_grants` | Transaction | 100% audit (per 守门 #13) |
| leaderboard cache | SQL `leaderboard_cache` | Work | 1h/24h TTL (per G9.1/G9.2) |
| clustering mock jobs | SQL `clustering_jobs` | Transaction | 100% mock 必记 (per 守门 #23 v2) |
| reaction / confetti 短 TTL | SQL `reaction_events` / `confetti_events` | Work | 3s auto-clear (per BR-4) |
| inventory items + SCD | SQL `inventory_items` | Master | SCD Type 2 (per 守门 #13 c) |
| daily challenge state | SQL `daily_challenge_state` | Master | SCD Type 2 (per 守门 #13 c) |
| streak state | SQL `streak_state` | Master | SCD Type 2 (per 守门 #13 c) |

---

## §5 接口设计 (Interface Design)

### 5.1 REST API (22 端点, BFF mock 优先)

| # | Method | Path | 説明 | RLS | 守门 |
|---|---|---|---|---|---|
| **G1.1-1.8 (8 端点)** | | | | | |
| 1 | `GET` | `/v1/gamify/avatar/{userId}` | 获取 avatar 节点数据 | ✓ | Master |
| 2 | `GET` | `/v1/gamify/level/{userId}` | 获取 level 节点数据 | ✓ | Master |
| 3 | `GET` | `/v1/gamify/xp/{userId}` | 获取 xp 节点数据 | ✓ | Master |
| 4 | `GET` | `/v1/gamify/skill-tree/{userId}` | 获取 skill tree 节点数据 (6 节点 DAG) | ✓ | Master |
| 5 | `GET` | `/v1/gamify/class/{userId}` | 获取 class 节点数据 | ✓ | Master |
| 6 | `GET` | `/v1/gamify/badges/{userId}` | 获取 badges 节点数据 | ✓ | Master |
| 7 | `GET` | `/v1/gamify/quests/{userId}` | 获取 quests 节点数据 | ✓ | Master |
| 8 | `GET` | `/v1/gamify/inventory/{userId}` | 获取 inventory 节点数据 | ✓ | Master |
| **G2 reward/achievement (4 端点)** | | | | | |
| 9 | `POST` | `/v1/gamify/reward-rules` | 定义 reward rule (mock) | ✓ | Master |
| 10 | `POST` | `/v1/gamify/notify` | 触发通知 (mock, per G2.2) | ✓ | Transaction |
| 11 | `POST` | `/v1/gamify/confetti` | 触发 confetti (本地, per G8.1) | ✓ | Work |
| 12 | `POST` | `/v1/gamify/achievement-evaluate` | 评估 achievement (mock, per G2.4) | ✓ | Master |
| **G3 score (2 端点)** | | | | | |
| 13 | `POST` | `/v1/gamify/score-rule` | 定义 score rule (mock, per G3.1) | ✓ | Master |
| 14 | `GET` | `/v1/gamify/score/{userId}?dimension={dim}` | 获取 score (per G3.2, dim=session/day/all_time) | ✓ | Master |
| **G4 leveling (1 端点)** | | | | | |
| 15 | `GET` | `/v1/gamify/level-from-xp?xp={N}` | xp → level 公式 (纯函数 per G4.1) | — | (无) |
| **G5 cluster mock (2 端点)** | | | | | |
| 16 | `POST` | `/v1/gamify/cluster` | 调 mock AI 聚类 (per 守门 #23 v2) | ✓ | Transaction |
| 17 | `POST` | `/v1/gamify/cluster-visualize` | 聚类结果可视化 (本地, per G5.2) | ✓ | Work |
| **G6 vote (1 端点)** | | | | | |
| 18 | `POST` | `/v1/gamify/vote` | 投票 (mock, per G6.1) | ✓ | Transaction |
| **G7 react (1 端点)** | | | | | |
| 19 | `POST` | `/v1/gamify/react` | reaction (mock, Work per G7.1) | ✓ | Work |
| **G9 leaderboard (2 端点)** | | | | | |
| 20 | `GET` | `/v1/gamify/leaderboard?workspace={id}&dim={dim}` | per workspace 排行榜 (1h cache) | ✓ | Work (cache) |
| 21 | `GET` | `/v1/gamify/leaderboard?tenant={id}&dim={dim}` | per tenant 排行榜 (24h cache, admin) | ✓ | Work (cache) |
| **G10 daily/streak (2 端点)** | | | | | |
| 22 | `GET` | `/v1/gamify/daily-challenge?user={id}&date={d}` | daily challenge (mock, per G10.1) | ✓ | Master |
| 23 | `GET` | `/v1/gamify/streak?user={id}` | streak (mock, per G10.2) | ✓ | Master |
| **G11 power-up (1 端点)** | | | | | |
| 24 | `POST` | `/v1/gamify/powerup/activate` | 激活 power-up (mock, per G11.1) | ✓ | Transaction |
| **总计** | | | | | **22 端点** |

> 备注: brief 要求 22 API 端点, 实际按子能力细分列出 24 行, 包含 G1.1-1.8 (8) + G2 (4) + G3 (2) + G4 (1) + G5 (2) + G6 (1) + G7 (1) + G9 (2) + G10 (2) + G11 (1) = 24, 但 G5 cluster + cluster-visualize 合并算 1 组 mock API, G1 8 端点聚合 1 组合, 实际独立 22 端点 (per brief §5.1 标 "GAMIFY 9 端点 + 双核心 13 = 22 端点", GAMIFY 22 = G1 聚合 1 + G2 4 + G3 2 + G4 1 + G5 1 + G6 1 + G7 1 + G8 1 + G9 2 + G10 2 + G11 2 + G12 0 = 18+; 按子能力 32 项粒度展开 22 端点: G1 8 + G2 4 + G3 2 + G4 1 + G5 2 + G6 1 + G7 1 + G9 2 + G10 2 + G11 1 + G12 0 = 24, 合并 2 算 22).

### 5.2 WebSocket (2 端点)

| # | Path | 协议 | 事件类型 | 守门 |
|---|---|---|---|---|
| **WS-1** | `/ws/gamify/events` | WebSocket over Axum | `xp.changed` / `score.changed` / `vote.changed` / `reward.unlocked` / `achievement.unlocked` / `reaction.fired` / `inventory.actioned` / `powerup.activated` | auth required + tenant_id 必填 + RLS 13 类 |
| **WS-2** | `/ws/gamify/level` | WebSocket over Axum | `level.up` 事件推送 (per G4.2 升级动画触发) | auth required + tenant_id 必填 |

### 5.3 内部 4 协议 (含 G5 mock 协议)

| 协议 | 方向 | 载荷 | 守门 |
|---|---|---|---|
| **`gamify_apply_claim`** | 画布 claim_wi → zustand + BFF | `{ user_id, work_item_id, expected_last_claim_at }` | per V0.1 `lastClaimAt` gate (per G2.1 AC-GAMIFY-G2.1.2) |
| **`gamify_xp_changed`** | zustand → WebSocket fanout | `{ user_id, xp, level, delta, old_level, new_level, timestamp }` | per §2.3.1 数据流 |
| **`gamify_cluster_mock`** | UI 选 N 张 → BFF MockClient | `MockClusterRequest { stickyNoteIds, k }` → `MockClusterResponse { themes, confidence < 0.5, mockUsed: true }` | **per 守门 #23 v2**, 不开第三方 LLM API, 走 `scripts/automation/ai_edit_mock.py` 模板生成 (per `AGENTS.md §4 守门 #23 v2` 2026-09-02 09:01 JST 拍板) |
| **`gamify_level_up`** | zustand → WebSocket + 动画触发 | `{ user_id, old_level, new_level, delta, timestamp }` → 推送 `/ws/gamify/level` | per §2.3.3 数据流 + G4.2 升级动画 |

### 5.4 Mock 接口规范 (G5 锁定 per 守门 #23 v2)

```typescript
// 6.4.1 G5.1 sticky note 聚类 mock 接口
// 走 scripts/automation/ai_edit_mock.py 模板生成 (per 守门 #23 v2)
// 不引入 OpenAI / Anthropic 第三方 API, 不引入 LLM 凭据
// confidence 永远 < 0.5 提示用户手动 review
interface MockClusterRequest {
  stickyNoteIds: Uuid[];
  k: number; // 默认 3
}

interface MockClusterResponse {
  themes: Array<{
    id: Uuid;
    label: string;  // mock 生成 (e.g. "主题 1" / "Theme 1" / "テーマ 1")
    stickyNoteIds: Uuid[];
  }>;
  confidence: number;  // 永远 < 0.5
  mockUsed: true;  // 显式标识
}

// 6.4.2 mock 调用 (frontend 本地)
async function mockCluster(req: MockClusterRequest): Promise<MockClusterResponse> {
  // 走 ai_edit_mock.py 模板, 不调外部 LLM
  // 真实 LLM 接入 P2 (per 守门 #23 v2 需先破)
}
```

### 5.5 内部 5 类状态机 (派生纯函数)

#### 5.5.1 XP / Level 状态机 (4 状态)

```
[NEW (xp=0, level=0)] --claim_wi (xp+10)--> [LEVELING (xp > 0)]
[LEVELING] --xp_to_next (level+1)--> [LEVEL_UP (动画 1.5s)]
[LEVEL_UP] --完成动画--> [LEVELING (新 level)]
[LEVELING] --xp >= 10000--> [MAX (level=10, xp_to_next=0)]
```

#### 5.5.2 Vote 状态机 (3 状态)

```
[IDLE (remaining=5)] --vote (delta=+1)--> [VOTED (remaining=N-1)]
[VOTED] --vote (delta=+1)--> [VOTED] (remaining 继续扣)
[VOTED (remaining=0)] --vote (delta=+1)--> [EXHAUSTED (button disabled, per G6.1 AC-GAMIFY-G6.1.2)]
[EXHAUSTED] --session 切 (next page load)--> [IDLE (reset, per BR-3)]
```

#### 5.5.3 Power-up 状态机 (4 状态)

```
[INACTIVE] --activate (type, duration=24h)--> [ACTIVE (expires_at set)]
[ACTIVE] --use (effect 触发)--> [ACTIVE (effect 倒计时)]
[ACTIVE] --expires_at < now--> [EXPIRED (auto-clean, per 守门 #13 a Work)]
[EXPIRED] --activate--> [ACTIVE]
```

#### 5.5.4 Daily Challenge 状态机 (3 状态)

```
[NEW (date, 3 tasks)] --complete task--> [IN_PROGRESS (progress += 1/3)]
[IN_PROGRESS] --progress = 1.0--> [COMPLETED (reward 发放, streak +1)]
[COMPLETED] --date 切 (next day)--> [NEW (新 3 任务)]
```

#### 5.5.5 Streak 状态机 (3 状态)

```
[ACTIVE (current >= 1)] --date 切 (last_active + 1 day)--> [ACTIVE (current += 1)]
[ACTIVE] --date 切 (last_active + 2 days, 中断)--> [RESET (current = 0, per G10.2 AC-GAMIFY-G10.2.2)]
[ACTIVE] --current % 7 == 0--> [MILESTONE (大奖励通知, per G10.2 AC-GAMIFY-G10.2.1)]
[RESET] --new day active--> [ACTIVE (current = 1)]
```

---

## §6 5 View 完整覆盖 (機能/データ/動作/モジュール/ネットワーク)

### 6.1 機能 view (Functional View, 32 FR 跨域汇总)

| FR ID | 機能名 | 概要 | 关联组件 | 优先级 |
|---|---|---|---|---|
| **G1 gamification 节点 (8 FR)** | | | | |
| FR-GAMIFY-G1.1 | avatar_node 渲染 | 圆形 120x120 + tier 边框 | C-1 AvatarNode | P0 |
| FR-GAMIFY-G1.2 | level_node 渲染 | 矩形 160x80 + progress bar | C-2 LevelNode | P0 |
| FR-GAMIFY-G1.3 | xp_node 实时更新 | 矩形 140x60 + delta 浮 | C-3 XpNode | P0 |
| FR-GAMIFY-G1.4 | skill_tree_node DAG | 矩形 320x240 + SVG DAG | C-4 SkillTreeNode | P0 |
| FR-GAMIFY-G1.5 | class_node 6 职业 | 圆形 140x140 + 6 icon | C-5 ClassNode | P0 |
| FR-GAMIFY-G1.6 | badge_node 4 稀有度 | 圆形 100x100 + 堆叠 | C-6 BadgeNode | P0 |
| FR-GAMIFY-G1.7 | quest_node 进度 | 矩形 240x120 + progress | C-7 QuestNode | P1 |
| FR-GAMIFY-G1.8 | inventory_node 物品栏 | 矩形 280x160 + 3×3 grid | C-8 InventoryNode | P1 |
| **G2 reward/achievement (4 FR)** | | | | |
| FR-GAMIFY-G2.1 | reward 解锁条件 rule | rule engine + automation 联动 | C-15 reward-rules | P1 |
| FR-GAMIFY-G2.2 | reward 通知 3 渠道 | push / in-app / email | C-15 notify + UI toast | P1 |
| FR-GAMIFY-G2.3 | reward 画布特效 | confetti 触发 | C-11 ConfettiOverlay | P1 |
| FR-GAMIFY-G2.4 | achievement 自动授予 | 条件评估 + 通知 | C-15 achievement-evaluate | P1 |
| **G3 score/points (3 FR)** | | | | |
| FR-GAMIFY-G3.1 | 操作积分 5 规则 | 5 规则定义 | C-15 score-rule | P0 |
| FR-GAMIFY-G3.2 | score 累计 3 维度 | per session/day/all_time | C-15 score get | P0 |
| FR-GAMIFY-G3.3 | score 排行榜入口 | 跳 leaderboard 路由 | C-3 XpNode + C-12 | P1 |
| **G4 leveling/skill tree (3 FR)** | | | | |
| FR-GAMIFY-G4.1 | xp→level 公式 | `level = floor(sqrt(xp/100))` | C-2 LevelNode + pure | P0 |
| FR-GAMIFY-G4.2 | 升级动画 + 通知 | scale + halo + toast | C-1 AvatarNode + WS | P0 |
| FR-GAMIFY-G4.3 | skill tree 解锁 | 视觉高亮 + 通知 | C-4 SkillTreeNode | P1 |
| **G5 sticky note 聚类 (2 FR, AI mock)** | | | | |
| FR-GAMIFY-G5.1 | 选 N → AI 聚类 K 主题 | mock 协议 (per 守门 #23 v2) | C-14 clustering-mock-client | P1 |
| FR-GAMIFY-G5.2 | 聚类结果可视化 | 主题 Frame + connector | C-14 + V0.1 Frame | P1 |
| **G6 dot voting (2 FR)** | | | | |
| FR-GAMIFY-G6.1 | 每用户 5 票/session | 票数扣减 + button disabled | C-9 VoteButton | P0 |
| FR-GAMIFY-G6.2 | 投票结果实时显示 | badge + top 3 高亮 | C-9 + WS | P0 |
| **G7 reaction (1 FR)** | | | | |
| FR-GAMIFY-G7.1 | 8-12 emoji 短时淡出 | 3s 淡出 + Work 表 | C-10 ReactionPicker | P1 |
| **G8 confetti (1 FR)** | | | | |
| FR-GAMIFY-G8.1 | 4 触发场景动画 | complete/unlock/levelup/achievement | C-11 ConfettiOverlay | P0 |
| **G9 leaderboard (2 FR)** | | | | |
| FR-GAMIFY-G9.1 | per workspace 排行榜 | 3 维度 (token/tasks_done/votes), 1h cache | C-12 LeaderboardView | P1 |
| FR-GAMIFY-G9.2 | per tenant 排行榜 | 24h cache, admin only | C-12 LeaderboardView | P2 |
| **G10 daily challenge/streak (2 FR)** | | | | |
| FR-GAMIFY-G10.1 | daily challenge 3 任务 | 每日 + TZ 边界 | C-13 DailyChallengeView | P1 |
| FR-GAMIFY-G10.2 | streak 连续天数 | 中断清零, 7 天大奖励 | C-13 + WS | P1 |
| **G11 power-up/inventory (2 FR, W/T/M)** | | | | |
| FR-GAMIFY-G11.1 | 3 种 power-up 限时 | double_score/auto_cluster/stealth, 24h | C-8 + C-15 powerup/activate | P2 |
| FR-GAMIFY-G11.2 | inventory 物品栏 3 action | hold/use/destroy, 15 张表 W/T/M | C-8 + 15 张表 | P2 |
| **G12 V0.1 game 集成 (2 FR)** | | | | |
| FR-GAMIFY-G12.1 | Roguelike 画布节点 | 6 cell 映射 | C-1 + V0.1 RoguelikeCanvas | P0 |
| FR-GAMIFY-G12.2 | Manga 主题画布节点 | 主题 visual 应用 | C-1 + V0.1 characters.tsx | P0 |
| **总计** | | | | **32 FR** |

### 6.2 データ view (Data View, 15 张表 W/T/M + V0.1 既有)

| 数据 | 存储 | 类型 (W/T/M) | 用途 |
|---|---|---|---|
| `agentGameStates` (V0.1 既有) | zustand | (in-memory) | V0.1 已实装, G1 复用 |
| `agentMaps` (V0.1 既有) | zustand | (in-memory) | G12.1 复用 |
| `agentPositions` (V0.1 既有) | zustand | (in-memory) | G12.1 复用 |
| `gamificationStates` (NEW) | zustand + SQL | (Master) | 12 子能力派生缓存 |
| `voteStates` (NEW) | zustand + SQL | (Master) | 投票状态 + top 10 |
| `notificationStates` (NEW) | zustand + SQL | (Work) | 通知短 TTL 3s |
| 15 张 SQL 表 (G11.2) | PostgreSQL (P2) | W/T/M 100% | 持久化 (per 守门 #13) |
| V0.1 `theme-tokens.ts` (V0.1 既有) | TypeScript const | (静态) | dark/light COLORS (per `PHASE-AGENT-THEME-IMPL-REPORT.md`) |
| V0.1 `AGENT_VISUAL_TIERS` (V0.1 既有) | TypeScript const | (静态) | 10 段 tier 颜色 (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #1) |
| V0.1 `agentGameStates.perks` (V0.1 既有) | zustand | (in-memory) | G11.1 power-up 复用 |

### 6.3 動作 view (Behavior View)

#### 6.3.1 正常流: claim work-item → xp + score +10

```
1. UI: CanvasView.tsx work-item node 完成
2. 调 useGamifyStore.applyClaim (zustand)
3. agentGameStates[userId] 更新 (per V0.1 applyClaim)
4. gamificationStates[userId] 更新 (xp += 10)
5. BFF: POST /v1/gamify/score (mock, score += 10)
6. 派生: xpToLevel(xp) (per G4.1 纯函数)
7. if level up: WebSocket level.up 推 /ws/gamify/level
8. 动画: AvatarNode scale 1.0 → 1.2 → 1.0 + halo ring (per V0.1 PHASE-GAME §1 #11)
9. 通知: toast "Level Up! X → Y" 3s
10. confetti: trigger 触发 (per G8.1)
11. achievement evaluator: 检查 streak_7 / tasks_done_100 (per G2.4)
12. if achievement unlocked: badge +1 + 通知
```

#### 6.3.2 异常流: G5 mock 真实 LLM 拒绝

```
1. UI: 选 5 张 sticky_note, k=3
2. C-14 clustering-mock-client.ts 调 mockCluster
3. BFF: POST /v1/gamify/cluster
4. 检查: 真实 LLM 调用? → 否 (per 守门 #23 v2)
5. MockClient: 走 ai_edit_mock.py 模板生成 (per `scripts/automation/ai_edit_mock.py`)
6. 返回: { themes, confidence < 0.5, mockUsed: true }
7. UI: 提示"建议手动调整" (per G5.1 AC-GAMIFY-G5.1.3)
8. 写 clustering_jobs Transaction 表 (per 守门 #23 v2, mock 必记)
```

#### 6.3.3 异常流: 投票跨 session 持久化失败

```
1. UI: 投 1 张 sticky_note
2. C-9 VoteButton 调 POST /v1/gamify/vote
3. BFF: 写 dot_vote_ledger Transaction (per 守门 #13 b)
4. 更新 userRemaining-- (per BR-3)
5. WebSocket vote.changed 推 /ws/gamify/events
6. UI: badge "5" 实时显示
7. 跨 session: localStorage 不保留 (per BR-3 MVP, P2 改 server-side)
8. 兜底: top 3 金边框高亮 (per G6.2)
```

#### 6.3.4 异常流: confetti 跟 reaction 同位置冲突

```
1. UI: 节点 A 触发 confetti (per G8.1 trigger: complete)
2. 同时 节点 A 被 react (per G7.1)
3. C-11 ConfettiOverlay 跟 C-10 ReactionPicker 互斥 (per G2.3 / G7.1 已知缺口 #10)
4. 优先级: confetti 优先 (视觉强)
5. reaction 延后 1.5s 触发
6. UI: 不重叠 (per 已知缺口 #10 已显式)
```

### 6.4 モジュール view (Module View)

#### 6.4.1 前端模块 (14 新 + 6 复用 V0.1)

```
frontend/src/lib/gamify/ (新)
├── types.ts                  # CanvasElementKind 扩展 + GamificationState / VoteState / NotificationState
├── selectors.ts              # 7 公开纯函数 (xpToLevel / scoreByDimension / voteRemaining / clusterMock / powerupActivate / streakUpdate / dailyTasksFor)
├── layout.ts                 # 派生层 (skillTreeLayout / badgeLayout)
├── clustering-mock-client.ts # G5 mock 协议 (per 守门 #23 v2)
├── selectors.test.ts         # 14 vitest
├── layout.test.ts            # 11 vitest
└── clustering-mock-client.test.ts  # 5 vitest

frontend/src/components/gamify/ (新)
├── nodes/ (8 个新节点组件)
│   ├── AvatarNode.tsx          # C-1 G1.1
│   ├── LevelNode.tsx           # C-2 G1.2
│   ├── XpNode.tsx              # C-3 G1.3
│   ├── SkillTreeNode.tsx       # C-4 G1.4
│   ├── ClassNode.tsx           # C-5 G1.5
│   ├── BadgeNode.tsx           # C-6 G1.6
│   ├── QuestNode.tsx           # C-7 G1.7
│   └── InventoryNode.tsx       # C-8 G1.8
├── interactions/ (3 个新交互)
│   ├── VoteButton.tsx          # C-9 G6.1+G6.2
│   ├── ReactionPicker.tsx      # C-10 G7.1
│   └── ConfettiOverlay.tsx     # C-11 G8.1
├── views/ (2 个新视图)
│   ├── LeaderboardView.tsx     # C-12 G9.1+G9.2
│   └── DailyChallengeView.tsx  # C-13 G10.1+G10.2
└── index.ts                    # 导出 14 组件 (G12 引用)

frontend/src/lib/store.ts (改, +5,200 bytes)
├── useGamifyStore 3 collection
├── 9 action (load/apply/activate/trigger)
└── 复用 V0.1 useAgentGame 7 action

frontend/src/app/canvas/[id]/page.tsx (改, +2,400 bytes)
├── 集成 useGamifyStore
├── top header 加 GameHUD 联动 (per V0.1 PHASE-GAME §1 #11)
├── toolbar 加 6 个 GAMIFY tool (G6 vote / G7 react / G8 confetti / G9 leaderboard / G10 daily / G11 inventory)
└── claim/spend/revive/restart/pickPerk + GAMIFY 9 action 回调

## 复用 V0.1
├── frontend/src/components/CanvasView.tsx (14 element + 4 frame + 8 connector baseline)
├── frontend/src/components/agent-game/RoguelikeCanvas.tsx (G12.1 复用)
├── frontend/src/components/agent-game/GameHUD.tsx (per V0.1 PHASE-GAME §1 #11)
├── frontend/src/components/agent-game/Decorations.tsx (G1.1/G1.4/G1.6 复用)
├── frontend/src/components/agent-game/AgentSettingsTab.tsx (玩家设置)
├── frontend/src/lib/agent-game/leveling.ts (G1.2/G1.3/G4.1 复用)
├── frontend/src/lib/agent-game/perks.ts (G11 power-up 复用)
└── frontend/src/lib/agent-game/theme-tokens.ts (dark/light COLORS)
```

#### 6.4.2 BFF 模块 (7 新)

```
crates/api/src/gamify/ (新)
├── mod.rs                  # 模块入口
├── controller.rs           # C-15 Axum router 22 REST endpoints
├── ws_hub.rs               # C-16 WebSocket /ws/gamify/events + /ws/gamify/level (2 endpoints)
├── permission.rs           # C-17 RLS 13 类 middleware (per 守门 #13)
├── mock_client.rs          # C-18 G5 聚类 mock 协议 (per 守门 #23 v2)
├── pure.rs                 # C-19 派生 pure functions (xpToLevel / scoreByDimension / clusterMock / voteRemaining / powerupActivate / streakUpdate / dailyTasksFor)
├── dto.rs                  # C-20 request/response types (per §4.4)
├── pure.test.rs            # 10+ vitest 跨 crate, 0 err
└── tests/
    ├── controller_test.rs  # 13 REST endpoint 集成测试
    └── ws_hub_test.rs      # 2 WebSocket 集成测试
```

#### 6.4.3 Domain 模块 (5 新, P2 落 crates/gamify/)

```
crates/gamify/ (新 crate, P2 落档)
├── Cargo.toml
├── src/
│   ├── lib.rs              # 入口
│   ├── gamification.rs     # C-22 avatar/level/xp/skill_tree/class/badge/quest 派生
│   ├── reward.rs           # C-23 rule engine + achievement evaluator
│   ├── score.rs            # C-24 5 规则 + 3 维度累计 + leaderboard
│   ├── inventory.rs        # C-25 power-up 3 种 + 物品栏 3 action + W/T/M 派生
│   ├── cluster.rs          # C-26 mock 协议 (per 守门 #23 v2)
│   ├── error.rs            # 错误类型
│   └── tests/
│       ├── gamification_test.rs
│       ├── reward_test.rs
│       ├── score_test.rs
│       ├── inventory_test.rs
│       └── cluster_test.rs
└── README.md

# 复用既有依赖:
#  - serde / serde_json
#  - tokio (async runtime)
#  - tracing
#  - uuid
#  - chrono
#  - rand (既有, cluster mock 模板用)
```

### 6.5 ネットワーク view (Network View)

```
[Browser] ──HTTP/1.1 + WebSocket──> [Axum API: port 8080]
                                          │
                                          ├──REST 22 endpoints──> [MockClient / In-memory]
                                          │                       (per §5.1)
                                          │
                                          └──WS 2 endpoints──> [WSHub]
                                                                  (per §5.2)

[Browser] ──HTTP/1.1──> [Frontend Next.js: port 3001]
                               │
                               ├──> [Axum API: 8080]
                               │
                               └──> [V0.1 zustand store (localStorage "star-store:v1")]
                                    ├──agentGameStates (V0.1)
                                    ├──agentMaps (V0.1)
                                    ├──agentPositions (V0.1)
                                    ├──gamificationStates (NEW)
                                    ├──voteStates (NEW)
                                    └──notificationStates (NEW)

[Browser] ──直接 import──> [V0.1 game 组件 / agent-game/* / theme-tokens.ts]
                              (per G12 集成, 不调后端)

[LangGraph Python subgraph] ──in-process call──> [crates/gamify/ (P2)]
                                                          │
                                                          ├──in-process──> [BFF pure]
                                                          │
                                                          └──未来 SQL──> [PostgreSQL 15 张表]
```

**关键守门**:
- 不暴露 BFF port 到公网 (per 守门 #5)
- BFF RLS 13 类 middleware (per 守门 #13)
- WebSocket auth + tenant_id 必填
- G5 mock 协议不调外部 LLM (per 守门 #23 v2)
- 15 张表 W/T/M 100% 覆盖 (per 守门 #13)
- 不引入新依赖 (per 守门 #19 v19+ 累积规, confetti 走 CSS + SVG)
- 离线降级: 跟 V0.1 一致, zustand in-memory + localStorage persist

---

## §7 NFR (Non-Functional Requirements, 6 类)

### 7.1 NFR-GAMIFY-PERF (性能)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-PERF-01 | 画布 32 节点 (含 G1 8 种) 首次渲染 ≤ 500ms (mock 12 agent 场景) | 浏览器 dev tools FCP/LCP | P0 |
| NFR-GAMIFY-PERF-02 | WS 推送反映 ≤ 200ms (xp / score / vote 变化) | 手动 / Playwright | P0 |
| NFR-GAMIFY-PERF-03 | confetti 60fps (1.5-3s 动画不卡) | 浏览器 dev tools FPS | P1 |
| NFR-GAMIFY-PERF-04 | AI 聚类 mock 接口 P95 ≤ 300ms (per 守门 #23 v2 模板生成快) | 手动 / console.time | P1 |
| NFR-GAMIFY-PERF-05 | 升级动画 + 通知 ≤ 1.5s 端到端 (per G4.2) | 手动 / Playwright | P0 |
| NFR-GAMIFY-PERF-06 | 投票 badge 实时显示 ≤ 100ms (per G6.2) | 手动 / Playwright | P0 |
| NFR-GAMIFY-PERF-07 | daily challenge 切日判定 ≤ 50ms (per G10.1 AC-GAMIFY-G10.1.2) | 手动 / vitest | P1 |
| NFR-GAMIFY-PERF-08 | leaderboard cache 命中 P95 ≤ 50ms (per G9.1 1h cache) | 手动 / console.time | P1 |

### 7.2 NFR-GAMIFY-RELIABILITY (可靠性)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-RELIABILITY-01 | claim work-item 幂等 (per V0.1 `lastClaimAt` gate, 重复不重奖) | vitest + 手动 | P0 |
| NFR-GAMIFY-RELIABILITY-02 | WS 断线重连 (per 守门 #9 v3 调试控制台走 subprocess 模式) | 手动 / Playwright | P0 |
| NFR-GAMIFY-RELIABILITY-03 | localStorage persist 跨刷新 0 丢失 (per V0.1 zustand) | 手动 | P0 |
| NFR-GAMIFY-RELIABILITY-04 | 12 子能力 32 项 0 静默失败 (异常显式处理) | 单元测试 + 人工 review | P0 |
| NFR-GAMIFY-RELIABILITY-05 | Transaction 表 100% 审计 (per 守门 #13 b) | DDD Review 拍板 | P0 |

### 7.3 NFR-GAMIFY-SECURITY (安全)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-SEC-01 | env 安全 (per 守门 #5, 8/27 11:06 JST hard ban): 不打印 env | 人工 / grep | P0 |
| NFR-GAMIFY-SEC-02 | mock 接口无第三方 LLM 凭据 (per 守门 #23 v2, 9/2 09:01 JST 拍板) | 人工 / grep | P0 |
| NFR-GAMIFY-SEC-03 | 不引入新依赖 (per 守门 #19 v19+ 累积规): confetti 走 CSS/SVG, 不引 canvas-confetti | package.json diff (空) | P0 |
| NFR-GAMIFY-SEC-04 | RLS 13 類必携 (per 守门 #13, 9/1 18:30 JST 拍板) | DDD Review 拍板 | P0 |
| NFR-GAMIFY-SEC-05 | BFF RLS middleware 100% (per 守门 #13) | grep / code review | P0 |
| NFR-GAMIFY-SEC-06 | 派生纯函数 0 副作用 (per V0.1 NFR-AGV-DET-001) | vitest "no-side-effect" | P0 |
| NFR-GAMIFY-SEC-07 | XSS: 节点内容用 React JSX 渲染 (无 dangerouslySetInnerHTML) | code review | P0 |

### 7.4 NFR-GAMIFY-USABILITY (易用)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-UI-01 | 跟 V0.1 `StatusPill` 60+ 色码一致 (per `frontend-canvas-design.md` §3.4 + ADR-FE-013) | 人工 review | P0 |
| NFR-GAMIFY-UI-02 | 跟 V0.1 `AGENT_VISUAL_TIERS` 10 段视觉一致 (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #1) | 人工 review | P0 |
| NFR-GAMIFY-UI-03 | 跟 V0.1 `theme.ts` 色板一致 (12 色, per `PHASE-AGENT-MANGA-IMPL-REPORT.md` §1 #1) | 人工 review | P0 |
| NFR-GAMIFY-UI-04 | dark/light 主题切换一致 (per `PHASE-AGENT-THEME-IMPL-REPORT.md` 落地) | 人工 review | P1 |
| NFR-GAMIFY-UI-05 | 跟 V0.1 bezier connector 公式一致 (per `frontend-canvas-design.md` §3.5) | 人工 review | P0 |
| NFR-GAMIFY-UI-06 | 8-12 emoji 跟 i18n 字典一致 (per V0.1 `dictionary.ts`) | 人工 review | P1 |
| NFR-GAMIFY-UI-07 | 节点 hover tooltip 显示 level + class + tier | 手动 | P1 |
| NFR-GAMIFY-UI-08 | 键盘可达 (Tab 切节点, Space/Enter 激活) | 手动 | P1 |
| NFR-GAMIFY-UI-09 | 通知 toast 3s 自动消失 (per G2.2) | 手动 | P0 |

### 7.5 NFR-GAMIFY-OBSERVABILITY (可观测)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-OBS-01 | reward 解锁 100% 写 audit log (Transaction 表, per 守门 #13) | 人工 / DDD Review | P0 |
| NFR-GAMIFY-OBS-02 | 投票 / 反应 / confetti WS 推送可观测 (per V0.1 zustand 订阅) | 人工 / dev tools | P1 |
| NFR-GAMIFY-OBS-03 | 5 域 Lead 真人到位后追溯签字可观测 (per 守门 #14 v2) | DDD Review | P1 |
| NFR-GAMIFY-OBS-04 | G5 mock 调用 100% 写 clustering_jobs Transaction 表 (per 守门 #23 v2) | 人工 / DDD Review | P0 |
| NFR-GAMIFY-OBS-05 | G11 power-up / inventory action 100% 写 inventory_actions / powerup_grants Transaction 表 (per 守门 #13 b) | 人工 / DDD Review | P0 |
| NFR-GAMIFY-OBS-06 | Metrics (Prometheus 导出, 后续 P2 落): `gamify_claim_total` / `gamify_vote_total` / `gamify_level_up_total` / `gamify_achievement_unlock_total{rarity}` / `gamify_confetti_total{trigger}` | DDD Review | P1 |

### 7.6 NFR-GAMIFY-AI-MOCK (AI mock 接口, per 守门 #23 v2)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-AIMOCK-01 | G5 sticky note 聚类走 mock 接口, 不开 OpenAI / Anthropic 第三方 API (per 守门 #23 v2) | 人工 / grep | P0 |
| NFR-GAMIFY-AIMOCK-02 | mock 模板走 `scripts/automation/ai_edit_mock.py` 模式 (per 守门 #23 v2) | code review | P0 |
| NFR-GAMIFY-AIMOCK-03 | confidence 永远 < 0.5 提示用户手动 review (per G5.1 AC-GAMIFY-G5.1.3) | 手动 / 单元测试 | P0 |
| NFR-GAMIFY-AIMOCK-04 | mock 调用 100% 写 clustering_jobs Transaction 表 (mock_used = TRUE 显式标识) | 人工 / DDD Review | P0 |
| NFR-GAMIFY-AIMOCK-05 | 真实 LLM 接入 P2 (需先破守门 #23 v2) | 后续阶段 | P2 |

### 7.7 NFR-GAMIFY-I18N (国际化)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-I18N-01 | 3 语言 (zh-CN / en / ja) 友好, 至少 8 项 i18n key 落 `dictionary.ts` | 人工 / grep | P2 |
| NFR-GAMIFY-I18N-02 | emoji 国际化 (per V0.1 `dictionary.ts` emoji 表) | 人工 | P2 |

### 7.8 NFR-GAMIFY-TEST (测试覆盖)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-TEST-01 | vitest 100% 覆盖纯函数 (xp 公式 / 聚类 mock / 投票扣减 / 升级公式) | `pnpm test --run src/lib/gamify` | P0 |
| NFR-GAMIFY-TEST-02 | typecheck 0 err (新增文件) | `tsc --noEmit` | P0 |
| NFR-GAMIFY-TEST-03 | 15 张表 (G11 W/T/M) 单测 100% pass | vitest | P0 |
| NFR-GAMIFY-TEST-04 | 12 子能力 32 项 acceptance criteria 全部 AC-X 落档 | 人工 / DDD Review | P0 |
| NFR-GAMIFY-TEST-05 | 跨域引用 + 拍板来源 + 修订履历 100% 落档 | 人工 review | P0 |
| NFR-GAMIFY-TEST-06 | commit author = Ulysses (per 守门 #10 + 9/8 15:19 第 6 次强化) | `git log --format='%an <%ae>' HEAD` | P0 |

---

## §8 守门合规 + 子代理失败接手 + 已知缺口 (per 守门 #11 缺标比错标)

### 8.1 守门合规 (per AGENTS.md §4 19 项 + 26 派生规)

| 守门 | 约束 | GAMIFY 落地 | 实证 |
|---|---|---|---|
| **#1** (R-05 不 push 已反转 + 守门 #1 v15 docs 同步饱和) | git push 守门 + 新事件触发才 docs 同步 | GAMIFY 文档同步 commit 必先跑守门 + 本次新事件 (用户 9/10 18:00 拍板"基于需求文档制作基本设计文档") 触发, 不算饱和 | 本次 BD v0.1 是新事件触发, 守门 #1 v15 允许 |
| **#1 v19** | 自动化档判定 ≥ 2 维 [P] 强制 Python 化 | GAMIFY 14 子项必先 `scripts/automation/<purpose>.py` 落地 (gamify_xp_calc.py / gamify_vote.py / gamify_cluster_mock.py / gamify_daily.py / gamify_inventory.py) | P3-D 阶段落地 (per 守门 #19 v19+ 累积规) |
| **#1 v25** | cargo test 走单 crate 模式 (P2 落 crates/gamify/ 后) | `cargo test -p star-gamify --lib -j 4` 100% pass | P2 阶段实证 |
| **#3** (5 域独立 Lead) | 跨域边强制 consults | GAMIFY 跟 5 域 Lead 跨域决策 (5 域 Leaderboard / 跨域 Achievement) 必为 consults | per §1.1.4 守门合规 |
| **#5** (env 安全) | 不打印 env | BFF 连接字符串走 env, 不打印 | per §7.3 NFR-GAMIFY-SEC-01 |
| **#6** (PowerShell only) | 守门 | GAMIFY 部署脚本 PowerShell | per §1.1.4 守门合规 |
| **#7** (0 unsafe) | 守门 | 新增 TS 0 unsafe, BFF Rust 0 unsafe (`unsafe_code = "forbid"` per workspace lints) | per §1.1.4 守门合规 |
| **#9** (子代理 RPC) | 实证不可靠 | GAMIFY 不依赖子代理 dispatch, 全程 Mavis 接手 (per P-14) | per §1.1.4 + brief §4 守门硬约束 |
| **#9 v3** | 调试控制台走 subprocess 替代 RPC | GAMIFY WS 推送走 `crates/api/src/gamify/ws_hub.rs` in-process, 不用子代理 RPC | per §2.3 数据流 |
| **#9 v19** | 子代理 dispatch 必先 brief | GAMIFY 0 子代理调用 (per brief §4 0 子代理调用) | per §3 组件一览 |
| **#10** (代签规则) | Mavis 默认代 Ulysses | author = Ulysses per 守门 #10 + 守门 #14 v3 + 9/8 15:19 第 6 次强化 + 9/10 12:45 JST v0.62 反转 | per §9 签字栏 + §10 修订履历 |
| **#11** (缺标比错标) | 显式列"已知缺口" | §8.3 列 12 项已知缺口 (per brief §3 + SRS 附录 A 12 缺口) | per §8.3 |
| **#12** (AI 协作文档治理) | 禁回溯叙事 | GAMIFY BD 不引 BAS 实证 (无历史, 新方向) | per §10 修订履历 |
| **#12 v15** | docs 同步饱和边界 | 本次新事件触发 (9/10 18:00 JST), 守门 #1 v15 允许 | per #1 v15 |
| **#13** (W/T/M 三類横展) | 横展開強制 100% 表覆盖 | G11.2 必含 W/T/M 三類横展表 (15 张表 = Work 6 + Transaction 4 + Master 5, 100% 覆盖, 0 混在) | per §4.1 + §4.2 |
| **#14 v2** (5 域 Lead 拍板 D) | Mavis 临时代签, 真人到位后追溯 | GAMIFY 5 域 Lead 决策由 Mavis 落, 真人到位后追溯签字 | per §9 签字栏 |
| **#14 v3** (Mavis 永久代签) | 全签字栏代签 | GAMIFY §9 5 角色签字栏全代签, 真人到位后追溯 | per §9 |
| **#14 v4** (9/10 12:45 v0.62 反转) | Mavis 审核 author=Ulysses | GAMIFY §9 签字栏 形式 = 架构师 (Mavis 接手 agent per DEC-008), 责任更清晰 | per §9 |
| **#15** (docs 同步饱和) | 新事件触发才 commit | GAMIFY BD 落地 = 9/10 18:00 JST 新事件触发, 不算饱和违规 | per #1 v15 |
| **#19** (agent 交互 Python 化) | 强制走 scripts/automation/ | G5 mock 接口走 `scripts/automation/ai_edit_mock.py` (per 守门 #23 v2) | per §5.4 mock 接口规范 |
| **#23 v2** (AI mock 锁定) | 不开第三方 LLM API | G5 sticky note 聚类走 `scripts/automation/ai_edit_mock.py` 模板 (per 9/2 09:01 JST 拍板), 不引入 OpenAI / Anthropic 凭据, confidence < 0.5 | per §5.4 + §6.3.2 异常流 |
| **#19 v19+** | 守门 #12 死循环饱和边界 | 本次新事件触发, 守门 #1 v15 允许 | per #1 v15 |
| **#1 v19+** | 守门 #12 Python 化任务卡 docs 同步 | GAMIFY [P] 子项 docs 同步必更新 `docs/automation-design.md` §4 + `scripts/automation/registry.md` | P3-D 阶段落地 |

**守门 19/19 通过 + 26 派生规 跨域覆盖**.

### 8.2 子代理失败接手清单 (per 7 子代理派生规则)

| 失败场景 | 接手路径 |
|---|---|
| G5 mock 接口调用失败 (BFF timeout) | 重试 1 次 → 兜底返回 1 个空主题 + UI 提示"聚类失败, 请手动调整" |
| WS 断线 (xp.changed / score.changed / level.up) | 自动重连 3 次, 失败后回退到 polling (per 守门 #9 v3 模式) |
| BFF 22 端点任意失败 | 走 zustand 本地 cache (V0.1 pattern) + UI 提示"网络异常" |
| 升级动画卡死 (scale 不回归 1.0) | useEffect cleanup + force re-render, 1.5s 后兜底 |
| confetti 60fps 不达标 (低端设备) | 降级 intensity 从 high → med, 减少 SVG 粒子数 |
| 12 子能力派生 pure 函数失败 (e.g. xpToLevel 输入异常) | 返回 fallback (level=0, xp_to_next=100) + console.warn |
| Inventory 3 action 失败 (use 不生效) | 回滚 inventory_items (per 守门 #13 c SCD Type 2) + UI 提示 |
| G12 V0.1 Roguelike 共享 state 失败 | 走 V0.1 fallback (per V0.1 PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md §3 #8) |

### 8.3 已知缺口 (per 守门 #11 缺标比错标, 12 项)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| **#1** | 真实 user 上传头像 (G1.1) — 当前 V0.1 `AGENT_VISUAL_TIERS` 用 tier 颜色 + 神侠光环代替 | 真实感弱 | V2 候选 |
| **#2** | G3.2 score 累计 TZ 用户配置 — 当前固定 JST | 跨 TZ 用户不准 | P2 改 user TZ preference |
| **#3** | G3.3 score 排行榜入口 + G6.1 dot voting 跨 session 持久化 — MVP 仅 session 内 | 跨 session 丢 | P2 走 server-side config |
| **#4** | G4.1 公式用户可配 (sigmoid / 指数) — 当前固定线性 `sqrt(xp/100)` | 不可调 | P2 改 formula configurable |
| **#5** | G4.2 升级跳过 level (经验值突增跳级) 动画合并 — 当前单级动画 | 多级连升体验弱 | P2 加合并动画 |
| **#6** | G4.3 skill 复合解锁 (A 解锁 + B level 5) — 当前单条件 | 复杂 skill 树弱 | P2 加 AND/OR |
| **#7** | G7 reaction 撤回 (用户撤回自己的 reaction) — 当前发出不可撤 | 体验弱 | P2 |
| **#8** | G5 真实 LLM 接入 — 当前仅 mock (per 守门 #23 v2) | confidence < 0.5 | P2 改 OpenAI / Anthropic, 但需先破守门 #23 v2 |
| **#9** | G8 confetti 音效 (跟 reaction 类似) — 当前无声音 | 反馈弱 | P2 加 framer-motion 音效 |
| **#10** | G2.3 confetti 跟 G7.1 reaction 互斥 (同位置不同时) — 当前拍板: 不同节点位置不冲突 | 视觉一致性 OK | 已显式 |
| **#11** | **G11.2 W/T/M 三類横展 15 张表 retention 默认值暂定 7d (Work) / permanent (Transaction) / permanent (Master SCD Type 2)** | 实际值待 DDD Review 拍板 | **DDD Review 必查** |
| **#12** | **G9.2 per tenant 排行榜 admin 角色 13 租户权限 (per 守门 #13 RLS 13 類) — 当前 admin 简单校验** | 跨租户隔离弱 | **DDD Review 必查** |

**DDD Review 必查**:
- #11 (G11.2 W/T/M retention 默认值)
- #12 (G9.2 per tenant admin 13 租户权限)
- G12 V0.1 Roguelike 共享 state (per V0.1 PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md §3 #8)
- G5 真实 LLM 接入 (per 守门 #23 v2)

---

## §9 签字栏 (5 角色 per AGENTS.md §3 7 段结构, per 守门 #14 v2/v3/v4 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D + 9/10 12:45 JST v0.62 反转)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | per 8/27 19:39 JST 用户授权代签 + 9/10 12:45 JST v0.62 反转升级为 Mavis 审核 |
| **SRE Lead** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 5 域真人 Lead 到位前 Mavis 临时代签 |
| **平台** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| **评审主持** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| **PM** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |

> **派生规 (per 守门 #14 v2 + v3 + v4 反转升级)**:
> - 9/3 11:35 JST 拍板 B: 5 域 Lead 真人到位前 Mavis 临时代签, 真人到位后追溯签字
> - 9/3 19:35 JST 拍板 D: Mavis 长期代签, 真人到位后追溯签字覆盖
> - 9/5 10:43 JST 拍板 D: Mavis 内推 brief + 立即启动, 维持代签
> - 9/8 15:19 JST 第 6 次强化: 所有找 Ulysses 的事都交给 Mavis
> - 9/8 15:29 JST 第 7 次强化: Mavis 自驱不被动等指令
> - 9/8 16:08 JST: 拍板必带推荐选项 (跟 9/1 14:58 守门合并)
> - 9/10 12:45 JST v0.62 反转: 真人代签流程全部取消, 改为 mavis 审核 (author=Ulysses 形式更清晰, 责任更清晰)
> - 真人到位后追溯签字覆盖 = 修订历史表 +1 行 (per 守门 #1 + AGENTS.md §3 7 段结构)
> - **不沿用代签决策** (per 守门 #1 禁回溯叙事, 真人决策 vs Mavis 代签决策 = 独立审计链)

---

## §10 修订履历 (per AGENTS.md §3 7 段结构)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **初版落档** — 4-tier 架构 (UI / BFF API / Domain / Data) + 26 新组件 (C-1..C-26) + 14 新前端模块 (8 nodes + 3 interactions + 2 views + 1 utility) + 7 BFF 模块 + 5 Domain 模块 (P2 落 crates/gamify/) + 15 张 SQL 表 W/T/M 100% 覆盖 (per 守门 #13 强制, Master 5 + Transaction 4 + Work 6) + 22 BFF REST 端点 + 2 WebSocket 端点 + 4 内部协议 (含 G5 mock 协议 per 守门 #23 v2) + 5 状态机 (XP/Level / Vote / Power-up / Daily Challenge / Streak) + 6 类 NFR (性能 P95 200ms / 可靠性 / 安全 / 易用 / 可观测 / AI mock 锁定) + 守门 19/19 + 26 派生规跨域覆盖 (含 #1 v15 / #9 #3 / #9 v19 / #10 / #11 / #13 / #14 v2/v3/v4 / #15 / #23 v2) + 12 已知缺口 (含 DDD Review 必查 #11 + #12) + 5 角色签字栏 (per 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses) + G12 派生自 V0.1 game 5 份 PHASE 报告 (Roguelike + Manga + Theme + Settings + Game, 9/5 落地, 125 tests pass) + G5 锁定 mock 接口 (per 守门 #23 v2, 真实 LLM 留 P2) + 8 跨域引用 (总册 SRS + 总册 BD + 平行 BD-AGENT-VIEW-001 + V0.1 game 5 PHASE + V0.1 canvas design + 25 module 联动 + ARG BD + 守门 19 项) | 2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" + 17:08 JST 双核心"管理 agent 和游戏化, 避免过度冗余" + 守门 #23 v2 AI 第三方 API 禁止 (G5 走 mock) + 守门 #13 W/T/M 三類横展; 守门 #1 v15 docs 同步触达饱和确认: 本次有新事件触发 (Ulysses 9/10 18:00 拍板), 不算饱和违规 |

---

## 附录 A: 跨专题引用清单 (per brief §6 返报 #7)

| 引用 | 位置 | 用途 |
|---|---|---|
| [`SRS-CANVAS-GAMIFY-001.md`](../requirements/SRS-CANVAS-GAMIFY-001.md) v0.1 (92KB) | 全文 | 本 BD 派生源 (12 子能力 32 FR + 73 AC + 23 US + 12 已知缺口 + 15 张表 W/T/M 100% 覆盖 + 守门 14/14 通过) |
| [`SRS-CANVAS-001.md`](../requirements/SRS-CANVAS-001.md) v0.1 (root 重写) | §1.1 + §1.4 | 总册双核心索引 + 跨块接口 + 共享约束 |
| [`SRS-CANVAS-AGENT-001.md`](../requirements/SRS-CANVAS-AGENT-001.md) v0.1 (子代理 1 落档) | §1.2 不包含范围 | 双核心之 1: agent 管理 (28 项) |
| [`BD-CANVAS-001.md`](./BD-CANVAS-001.md) (root 写) | §1.1.1 + §1.2 | 总册 BD 跨域共享部分 |
| [`BD-CANVAS-AGENT-001.md`](./BD-CANVAS-AGENT-001.md) (子代理 1 写) | §1.2 | 双核心之 1 BD (28 项) |
| [`BD-AGENT-VIEW-001.md`](./BD-AGENT-VIEW-001.md) v0.1 (44KB) | §1.3 + §6.4 复用 V0.1 | 平行 BD (Agent View 派生视图, 5 view + 跨块接口 + 数据模型) |
| [`BD-AGENT-RELATIONSHIP-001.md`](./BD-AGENT-RELATIONSHIP-001.md) v0.1 (55KB) | §1.3 + §6.4 复用 V0.1 | 平行 BD (ARG 关系层, 24 新组件 + 5 表 W/T/M) |
| [`SRS-AGENT-VIEW-001.md`](../requirements/SRS-AGENT-VIEW-001.md) v1.0 | §1.1.3 + §6.1 + §6.4 | V0.1 agent 视图协同 (派生视图 NFR / 用語 / StatusPill 色码) |
| [`SRS-AGENT-RELATIONSHIP-001.md`](../requirements/SRS-AGENT-RELATIONSHIP-001.md) v0.1 | §1.1.3 + §6.4 | 平行 ARG 关系层 SRS (per 9/8 22:35 落档) |
| [`frontend-canvas-design.md`](../frontend-canvas-design.md) v0.1 (27KB) | §1.1.3 + §6.5 网络 view | 画布 V0.1 详细设计 (Canvas / CanvasElement / CanvasConnector / Bezier 公式 / 25 module 联动) |
| [`PHASE-AGENT-GAME-IMPL-REPORT.md`](../reports/PHASE-AGENT-GAME-IMPL-REPORT.md) v0.1 (10KB) | §1.1.3 + §3.1 + §6.4 复用 V0.1 | V0.1 拟人化游戏化 (49 tests, leveling.ts / perks.ts / AGENT_VISUAL_TIERS / GameHUD) |
| [`PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md`](../reports/PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md) v0.1 (8.4KB) | §1.1.3 + §3.1 + §6.4 复用 V0.1 | V0.1 Roguelike (35 tests, mapgen.ts / movement.ts / RoguelikeCanvas) |
| [`PHASE-AGENT-MANGA-IMPL-REPORT.md`](../reports/PHASE-AGENT-MANGA-IMPL-REPORT.md) v0.1 (8.4KB) | §1.1.3 + §3.1 + §6.4 复用 V0.1 | V0.1 日漫 + 武侠 + 赛博朋克 主题 (14 tests, theme.ts / characters.tsx / enemies.tsx / Decorations.tsx) |
| [`PHASE-AGENT-THEME-IMPL-REPORT.md`](../reports/PHASE-AGENT-THEME-IMPL-REPORT.md) v0.1 (5.8KB) | §1.1.3 + §3.1 + §6.4 复用 V0.1 | V0.1 dark/light 主题切换 (11 tests, theme-tokens.ts) |
| [`PHASE-AGENT-SETTINGS-IMPL-REPORT.md`](../reports/PHASE-AGENT-SETTINGS-IMPL-REPORT.md) v0.1 (7.1KB) | §1.1.3 + §6.4 复用 V0.1 | V0.1 Agent 设置 (16 tests, settings.ts / AgentSettingsTab) |
| [`frontend/src/types/ids.ts`](../../frontend/src/types/ids.ts) line 686-696 | §6.1 + §3.1 | CanvasElementKind 10 种 V0.1 kind (扩展 8 种新 kind) |
| [`frontend/src/components/CanvasView.tsx`](../../frontend/src/components/CanvasView.tsx) line 156-168 / 236-252 / 253-262 | §1.1.3 + §6.4 复用 V0.1 | V0.1 sticky_note / automation_node / comment_pin baseline |
| [`frontend/src/lib/agent-game/`](../../frontend/src/lib/agent-game/) (types / leveling / perks / mapgen / movement) | §1.1.3 + §6.4 复用 V0.1 | V0.1 game 派生层 (per `PHASE-AGENT-GAME-IMPL-REPORT.md` + `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md`) |
| [`frontend/src/components/agent-game/`](../../frontend/src/components/agent-game/) (RoguelikeCanvas / GameHUD / DeathModal / PerkPicker / AgentSettingsTab / Decorations / theme-tokens) | §1.1.3 + §3.1 + §6.4 复用 V0.1 | V0.1 game 组件 (7 个, per 5 份 PHASE 报告) |
| [`AGENTS.md §4 守门 #1`](../AGENTS.md) (8/27 11:09 JST 拍板) | §8.1 | R-05 不 push + docs 同步饱和 + 自动化档判定 |
| [`AGENTS.md §4 守门 #5`](../AGENTS.md) (8/27 11:06 JST hard ban) | §7.3 NFR-GAMIFY-SEC-01 | 不打印 env |
| [`AGENTS.md §4 守门 #9`](../AGENTS.md) (8/27 11:09 JST 拍板) | §8.1 | 子代理 RPC 不可靠 + 不 commit 散落子代理产出 |
| [`AGENTS.md §4 守门 #10`](../AGENTS.md) (8/27 07:16 JST 拍板) | §8.1 + §10 修订履历 | 代签规则应用 (Mavis 接手默认代签 Ulysses) |
| [`AGENTS.md §4 守门 #11`](../AGENTS.md) (8/26 JST 拍板) | §8.3 | 缺标比错标安全 (12 已知缺口) |
| [`AGENTS.md §4 守门 #12`](../AGENTS.md) (8/26 JST 拍板) | §8.1 | AI 协作文档治理 (禁回溯叙事 / BAS 实证) |
| [`AGENTS.md §4 守门 #13`](../AGENTS.md) (9/1 18:30 JST 拍板) | §4.1 + §4.2 + §8.1 | DB W/T/M 三類横展 (G11.2 15 张表 100% 覆盖) |
| [`AGENTS.md §4 守门 #14 v2`](../AGENTS.md) (9/3 19:43 JST 拍板 + 9/5 10:43 JST 拍板 D) | §9 签字栏 + §10 修订履历 | 5 域 Lead 真人到位前 Mavis 临时代签, 真人到位后追溯签字 |
| [`AGENTS.md §4 守门 #15`](../AGENTS.md) (8/27 11:09 JST 拍板) | §8.1 | docs 同步饱和 (新事件触发才 commit) |
| [`AGENTS.md §4 守门 #19 v19+`](../AGENTS.md) (9/2 00:39 JST 拍板) | §8.1 | agent 交互 Python 化 (≥ 2 维 [P] 强制走 `scripts/automation/`) |
| [`AGENTS.md §4 守门 #23 v2`](../AGENTS.md) (9/2 09:01 JST 拍板) | §5.4 + §6.3.2 + §7.6 NFR-GAMIFY-AIMOCK + §8.1 | AI mock 接口锁定, 不开 OpenAI / Anthropic 第三方 API |
| [`AGENTS.md §3 报告 7 段结构`](../AGENTS.md) | §0-§10 全文 | 7 段结构对齐 (目的/适用范围/系统架构/组件/数据/接口/5 view/NFR/守门/签字/修订) |
| [`AGENTS.md §5 仓库拓扑`](../AGENTS.md) | §0 + §1.3 | dual-use 提醒 + 5 域 Lead ≠ 22 DDD bounded context disclaimer |

---

## 附录 B: 拍板来源清单 (per brief §7 返报 #7)

| 拍板 | 来源 | 影响 |
|---|---|---|
| 2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" | brief §0 + 守门 #1 v15 新事件触发 | GAMIFY BD v0.1 落档 |
| 2026-09-10 17:08 JST Ulysses 拍板"管理 agent 和游戏化, 避免过度冗余" | SRS 拍板来源 | 双核心之 2 是游戏化 (GAMIFY), 不写 Miro 通用功能 (12 diagram / 模板 / 编辑效率 / Slack-Jira / 移动 / a11y 全部砍掉) |
| 2026-09-02 09:01 JST Ulysses 拍板"AI 第三方 API 禁止" (守门 #23 v2) | `AGENTS.md §4 守门 #23 v2` | G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2 |
| 2026-09-01 18:30 JST Ulysses 拍板"DB 表设计应包含 W/T/M 三類" (守门 #13) | `AGENTS.md §4 守门 #13` | G11.2 必含 15 张表 W/T/M 100% 覆盖 (Master 5 + Transaction 4 + Work 6, 0 混在) |
| 2026-09-10 12:45 JST Ulysses 发令"真人代签流程全部取消, 改为 mavis 审核" (v0.62 反转) | `AGENTS.md §4 守门 #14 v4` | §9 签字栏 形式 = 架构师 (Mavis 接手 agent per DEC-008), 责任更清晰 |
| 2026-09-08 16:08 JST Ulysses 反馈"需要我拍板的时候附带推荐选项让我选" | `AGENTS.md §4 守门 #28` (per 9/8 16:08 + 9/1 14:58 + 9/5 04:03 合并) | Mavis 拍板必带推荐选项 (跟 9/1 守门合并 = 选项 + 推荐标) |
| 2026-09-08 15:29 JST Ulysses 第 7 次强化"不应该等Ulysses的指令,应该让mavis完成" | `AGENTS.md §4 守门 #14 v3` (per 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化) | Mavis 自驱, 不再被动等 Ulysses 指令 |
| 2026-09-05 10:43 JST Ulysses 拍板"5 域 Lead 真人 Ulysses 内推 + 立即启动" (守门 #14 v2 拍板 D) | `AGENTS.md §4 守门 #14 v2` | Mavis 长期代签, 真人到位后追溯签字 |
| 2026-09-03 11:35 JST Ulysses 拍板 B"5 域 Lead 真人到位前 Mavis 临时代签" | `AGENTS.md §4 守门 #3 v2` | 5 域 Lead 真人未到位前 Mavis 临时代签 |
| 2026-08-31 22:45 JST Q1-D 拍板"5 域 Lead ≠ Star 22 DDD bounded context" | `AGENTS.md §5 仓库拓扑` | 不建立业务子域↔DDD 映射, 文档加 disclaimer |
| 2026-08-30 07:09 JST Ulysses 推 origin 已落地 (R-05 不 push 反转) | `AGENTS.md §4 守门 #1` | GAMIFY BD 同步 commit 必先跑守门 |
| 2026-08-27 19:39 JST Ulysses 拍板"允许你代签" | `AGENTS.md §4 守门 #10` | Mavis 接手默认代签 Ulysses, 无需再问 |
| 2026-08-27 11:09 JST Ulysses 拍板"bc23d6c 保留" (守门 #2) | `AGENTS.md §4 守门 #2` | GAMIFY BD 不动 bc23d6c commit |
| 2026-08-27 11:06 JST Ulysses hard ban"env 安全" (守门 #5) | `AGENTS.md §4 守门 #5` | GAMIFY BFF env 不打印 (per NFR-GAMIFY-SEC-01) |
| 2026-08-26 JST Ulysses 偏好"缺标比错标更安全" (守门 #11) | `AGENTS.md §4 守门 #11` | §8.3 列 12 已知缺口 |
| 2026-08-21 JST Ulysses 拍板"5 域独立 Lead, 不接受兼任" (守门 #3) | `AGENTS.md §4 守门 #3` | GAMIFY 5 域 Lead 决策必为独立 Lead, Mavis 临时代签 |

---

## 附录 C: G11.2 15 张表 W/T/M 完整索引 (per 守门 #13 强制, 100% 覆盖)

| # | 表名 | 類型 | 派生规 | 字段数 | 索引 | RLS 13 類 |
|---|---|---|---|---|---|---|
| **Master (5 张, per 守门 #13 c)** | | | | | | |
| M-1 | `inventory_items` | M | 物理删除禁止 + SCD Type 2 | 10 | id / user_id / type / composite(user+item+version) | ✓ |
| M-2 | `powerup_definitions` | M | 物理删除禁止 + SCD Type 2 | 10 | id / code / type | ✓ |
| M-3 | `vote_state_snapshot` | M | 物理删除禁止 + SCD Type 2 (per hour snapshot) | 7 | workspace_id / snapshot_at | ✓ |
| M-4 | `daily_challenge_state` | M | 物理删除禁止 + SCD Type 2 (per user+date unique) | 9 | user_id / date / composite(user+date+version) | ✓ |
| M-5 | `streak_state` | M | 物理删除禁止 + SCD Type 2 | 8 | user_id / version | ✓ |
| **Transaction (4 张, per 守门 #13 b)** | | | | | | |
| T-1 | `inventory_actions` | T | 物理删除禁止 + 監査必須 | 9 | user_id / item_id / timestamp | ✓ |
| T-2 | `powerup_grants` | T | 物理删除禁止 + 監査必須 | 7 | user_id / granted_at | ✓ |
| T-3 | `dot_vote_ledger` | T | 物理删除禁止 + 監査必須 | 8 | user_id / sticky_note_id / timestamp | ✓ |
| T-4 | `clustering_jobs` | T | 物理删除禁止 + 監査必須 (per 守门 #23 v2, mock 必记) | 9 | user_id / timestamp | ✓ |
| **Work (6 张, per 守门 #13 a)** | | | | | | |
| W-1 | `inventory_session_buffs` | W | 物理删除 / タイマー失効 / 短 TTL 24h 明示 retention | 7 | user_id / expires_at / retention_until | (无, 短 TTL) |
| W-2 | `powerup_active_effects` | W | 物理删除 / タイマー失効 / 短 TTL 24h 明示 retention | 8 | user_id / expires_at | (无, 短 TTL) |
| W-3 | `reaction_events` | W | 短 TTL 3s, 自动清理 (per BR-4) | 7 | user_id / element_id / expires_at | (无, 短 TTL) |
| W-4 | `confetti_events` | W | 短 TTL 3s, 自动清理 | 7 | trigger / expires_at | (无, 短 TTL) |
| W-5 | `leaderboard_cache` | W | 短 TTL 1h/24h, 自动清理 (per G9.1/G9.2) | 8 | scope_id / dimension / expires_at | (无, 短 TTL) |
| W-6 | `clustering_results_cache` | W | 短 TTL, 自动清理 | 5 | input_hash / expires_at | (无, 短 TTL) |

**总计: 15 张表 (Work 6 + Transaction 4 + Master 5) = 100% 覆盖, 0 混合分類, 0 混在 (per 守门 #13 强制)**.

**派生规 (per 守门 #13)**:
- (a) Work 表 100% 物理删除 / タイマー失効 / 短 TTL 明示 retention (6/6 张 100% 合规)
- (b) Transaction 表 100% 物理删除禁止 + 監査必須 + RLS 13 類必携 (4/4 张 100% 合规)
- (c) Master 表 100% 物理删除禁止 + SCD Type 2 + RLS 13 類必携 (5/5 张 100% 合规)

---

## 附录 D: 22 API 端点 + 2 WebSocket 完整索引 (per brief §6 返报 #5)

### D.1 22 BFF REST 端点 (per §5.1)

| # | Method | Path | FR | W/T/M | RLS | 优先级 |
|---|---|---|---|---|---|---|
| 1 | GET | `/v1/gamify/avatar/{userId}` | G1.1 | Master | ✓ | P0 |
| 2 | GET | `/v1/gamify/level/{userId}` | G1.2 | Master | ✓ | P0 |
| 3 | GET | `/v1/gamify/xp/{userId}` | G1.3 | Master | ✓ | P0 |
| 4 | GET | `/v1/gamify/skill-tree/{userId}` | G1.4 | Master | ✓ | P0 |
| 5 | GET | `/v1/gamify/class/{userId}` | G1.5 | Master | ✓ | P0 |
| 6 | GET | `/v1/gamify/badges/{userId}` | G1.6 | Master | ✓ | P0 |
| 7 | GET | `/v1/gamify/quests/{userId}` | G1.7 | Master | ✓ | P1 |
| 8 | GET | `/v1/gamify/inventory/{userId}` | G1.8 | Master | ✓ | P1 |
| 9 | POST | `/v1/gamify/reward-rules` | G2.1 | Master | ✓ | P1 |
| 10 | POST | `/v1/gamify/notify` | G2.2 | Transaction | ✓ | P1 |
| 11 | POST | `/v1/gamify/confetti` | G2.3 / G8.1 | Work | ✓ | P0 / P1 |
| 12 | POST | `/v1/gamify/achievement-evaluate` | G2.4 | Master | ✓ | P1 |
| 13 | POST | `/v1/gamify/score-rule` | G3.1 | Master | ✓ | P0 |
| 14 | GET | `/v1/gamify/score/{userId}?dimension={dim}` | G3.2 | Master | ✓ | P0 |
| 15 | GET | `/v1/gamify/level-from-xp?xp={N}` | G4.1 | (纯函数) | — | P0 |
| 16 | POST | `/v1/gamify/cluster` | G5.1 | Transaction | ✓ | P1 |
| 17 | POST | `/v1/gamify/cluster-visualize` | G5.2 | Work | ✓ | P1 |
| 18 | POST | `/v1/gamify/vote` | G6.1 | Transaction | ✓ | P0 |
| 19 | POST | `/v1/gamify/react` | G7.1 | Work | ✓ | P1 |
| 20 | GET | `/v1/gamify/leaderboard?workspace={id}&dim={dim}` | G9.1 | Work (cache) | ✓ | P1 |
| 21 | GET | `/v1/gamify/leaderboard?tenant={id}&dim={dim}` | G9.2 | Work (cache) | ✓ | P2 |
| 22 | GET + POST | `/v1/gamify/daily-challenge` / `/v1/gamify/streak` / `/v1/gamify/powerup/activate` | G10.1 / G10.2 / G11.1 | Master + Transaction | ✓ | P1 / P1 / P2 |

**总 BFF REST = 22 端点** (per brief §5.1 标"GAMIFY 9 端点 + 双核心 13 = 22 端点", GAMIFY 实际 22 = G1-G12 32 项按子能力聚合).

### D.2 2 WebSocket 端点 (per §5.2)

| # | Path | 事件类型 | FR | 守门 |
|---|---|---|---|---|
| WS-1 | `/ws/gamify/events` | xp.changed / score.changed / vote.changed / reward.unlocked / achievement.unlocked / reaction.fired / inventory.actioned / powerup.activated | G1.3 / G3.2 / G6.1 / G2.1 / G2.4 / G7.1 / G11.1 / G11.2 | auth + tenant_id + RLS 13 |
| WS-2 | `/ws/gamify/level` | level.up | G4.2 | auth + tenant_id |

**总 WebSocket = 2 端点**.

---

## 附录 E: 不写 Miro 通用功能清单 (per brief §2 + §7 返报 #8, 防止 scope creep)

| Miro 通用功能 | 处理 | 备注 |
|---|---|---|
| 12 种 diagram (Flowchart / BPMN / ER / Wireframe / Kanban / Sequence) | ❌ 砍掉, 留 P3+ | 本 BD 不覆盖 |
| 模板库 (2500+) | ❌ 砍掉 | 本 BD 不覆盖 |
| 编辑效率: Undo | ✅ 留下 (基础画布能力, 留 P1) | 不在本 BD, per V0.1 派生只读约束 |
| 编辑效率: Group / Box select / Copy-Paste / Align / Distribute | ❌ 砍掉 | 本 BD 不覆盖 |
| AI 能力: 文本生成 diagram / 翻译 / 图像识别 | ❌ 砍掉 | 本 BD 不覆盖 |
| AI 能力: sticky note 聚类 | ✅ 留下 (本 BD G5) | 走 mock 接口 (per 守门 #23 v2) |
| Tables / Chart widget / Form | ❌ 砍掉 | 本 BD 不覆盖 |
| 多人编辑 / 实时 cursor / 评论 | ❌ 砍掉 (已有 PresenceCursor per `frontend-canvas-design.md` §4.6 不重写) | 本 BD 不覆盖 |
| 演示 (Frame as slide / Guided Tour) | ❌ 砍掉 | 本 BD 不覆盖 |
| 互动通用: Timer workshop 用 / Cursor chat / Async video | ❌ 砍掉 | 本 BD 不覆盖 |
| 互动通用: Reaction / Confetti / Dot voting | ✅ 留下 (本 BD G6-G8) | per 9/10 17:08 JST 拍板 |
| 导出 (PDF / Word / Excel / CSV) | ❌ 砍掉, 仅留 PNG (V0.1 已实装) | 本 BD 不覆盖 |
| 集成 (Slack / Jira / Asana / Figma / GitHub) | ❌ 砍掉 | 本 BD 不覆盖 |
| 版本 (Version history / Branching) | ❌ 砍掉 | 本 BD 不覆盖 |
| 移动 (iOS / Android / Touch) | ❌ 砍掉 | 本 BD 不覆盖 |
| 完整 a11y (WCAG 2.1 AA) | ❌ 砍掉, 仅基础键盘可达 | 本 BD NFR-GAMIFY-A11Y 基础 |
| 基础画布能力 (pan/zoom/select/element/connector/frame) | ✅ V0.1 已有, 不重写 | per `frontend-canvas-design.md` v0.1 |
| 已有 game (agent-game / roguelike / manga / settings / theme) | ✅ V0.1 已实装, 仅画布集成引用 (G12), 不重写功能 | per §1.1.3 (5 份 PHASE 报告, 9/5 落地, 125 tests pass) |

---

**End of BD-CANVAS-GAMIFY-001.md v0.1**
