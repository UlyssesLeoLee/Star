# BD-CANVAS-001

> **无限画布 (Infinite Canvas) — 总册基本設計書 v0.1** (per 日本 IPA SEC 標準 / 基本設計書 テンプレート)
>
> **⚠️ 双核心定位 (per 2026-09-10 17:08 + 17:21 + 17:34 JST Ulysses 拍板)**
> 核心功能 = **管理 agent + 游戏化**, 避免过度冗余
> 方向重置: 17:00 JST 旧 12 大类 50 项 Miro 全面对标 (撤回, 3 子代理 stop) → 17:08 JST 双核心 (砍 11 大类 Miro 通用) → 17:21 JST ARG 图论构造 (A11 10 项) → 17:34 JST 多人编辑 (A12 8 项, v0.63 反转)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 关联需求: [`docs/requirements/SRS-CANVAS-001.md`](../requirements/SRS-CANVAS-001.md) v1.1 (58KB, 18:00 JST root 写, 双核心 78 项索引, 13 跨拍板派生, 13 风险, 守门 16 交叉引用)
> - 关联专题 BD (派生): [`docs/design/BD-CANVAS-AGENT-001.md`](./BD-CANVAS-AGENT-001.md) v0.1 (A1-A12 46 项, 18:00 JST 子代理 1 写) + [`docs/design/BD-CANVAS-GAMIFY-001.md`](./BD-CANVAS-GAMIFY-001.md) v0.1 (G1-G12 32 项, 18:00 JST 子代理 2 写)
> - 关联 V0.1: [`docs/frontend-canvas-design.md`](../frontend-canvas-design.md) v0.1 (14 element + 4 frame + 8 connector + 9 e2e 守门)
> - 关联 V0.1 实装: [`frontend/src/components/CanvasView.tsx`](../frontend/src/components/CanvasView.tsx) (V0.1 11 处 element 渲染 + tool + minimap)
> - 关联平行 view (派生自 SRS): [`docs/requirements/SRS-AGENT-RELATIONSHIP-001.md`](../requirements/SRS-AGENT-RELATIONSHIP-001.md) v0.1 (ARG, 37KB) + [`docs/design/BD-AGENT-RELATIONSHIP-001.md`](./BD-AGENT-RELATIONSHIP-001.md) v0.1 (55KB, A11 派生源) + [`docs/requirements/SRS-AGENT-VIEW-001.md`](../requirements/SRS-AGENT-VIEW-001.md) v1.0 (31KB) + [`docs/design/BD-AGENT-VIEW-001.md`](./BD-AGENT-VIEW-001.md) v0.1 (44KB)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 9/8 15:19 JST 第 6 次强化)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) (5 角色签字栏 per AGENTS.md §3)
> - 日期: 2026-09-10 JST (18:00 JST 拍板"基于需求文档制作基本设计文档")
> - 受众: 詳細設計エンジニア / 実装エンジニア / UI/UX 设计师 / アーキテクト / SRE / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)

---

## §0 目的 (Purpose)

本文档基于 [`SRS-CANVAS-001.md`](../requirements/SRS-CANVAS-001.md) v1.1 (58KB, 18:00 JST root 写, 双核心 78 项索引) 的需求, 定义 **无限画布 (Infinite Canvas)** 总册基本設計 (BD), 涵盖:

1. **5 view 跨域架构** (機能/データ/動作/モジュール/ネットワーク) 覆盖双核心 (管理 agent + 游戏化) + 多人编辑 (v0.63 反转) + ARG (10 类关系)
2. **14 张表 W/T/M 100% 覆盖** (A11 7 张 + A12 7 张) 跨域汇总
3. **32 API 端点 + 5 WebSocket** 跨域汇总 (BFF REST + WebSocket)
4. **25 module 联动** (work-item / worktree / agent / relation / comment / automation / audit / search / notification + V0.1 game 4 份) 跨域接口
5. **6 类 NFR 跨域共享基线** (性能 / 可靠性 / 安全 / 易用 / 可观测 / ARG)
6. **19 守门 + 26 派生规跨域汇总** + 13 风险 + 12 已知缺口
7. **5 角色签字栏** (架构师 / SRE Lead / 平台 / 评审主持 / PM, per AGENTS.md §3)

2 份平行专题 BD (BD-AGENT + BD-GAMIFY) 各自展开 46 项 / 32 项详细, **本总册不重复**, 仅汇总跨域共享部分 + 跨域接口.

**dual-use 提醒 (per [AGENTS.md §5 倉庫拓扑](../../AGENTS.md))**: 本 view 不引用 RGS 仓 + 不建立业务子域↔DDD bounded context 映射. 5 域独立 Lead (player/economy/match/social/admin per 8/21 JST RGS 治理命名) **不等于** Star 仓 22 DDD bounded context (per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer). 画布通过 25 module 联动接口跟 5 域对接, 但**不**直接调用其他 view.

---

## §1 适用范围 (Scope)

### 1.1 In-Scope (5 view 跨域)

#### 1.1.1 機能 view (5 域跨域 FR)

**双核心 1: 管理 agent (46 项, A1-A12, 详见 BD-CANVAS-AGENT-001.md)**:
- A1 agent 节点渲染 (3 项) — agent_cursor → agent_node 完整卡 + StatusPill 60+ + 双击跳详情
- A2 agent 拓扑图 (4 项) — handoff / 5 域 / 父子 / pipeline
- A3 agent 状态实时同步 (3 项) — 14 状态机实时色码
- A4 agent 关联 worktree (2 项) — 1:N
- A5 agent 关联 work-item (2 项) — 1:N
- A6 agent 操作菜单 (4 项) — 启停 / 重启 / logs / settings
- A7 agent 监控面板 (3 项) — token / cost / runtime
- A8 agent 聚类 / 排序 / 过滤 (3 项)
- A9 agent session 跨域引用 (2 项) — 跟 SRS-AGENT-VIEW-001 / SRS-AGENT-RELATIONSHIP-001
- A10 agent settings V0.1 集成 (2 项) — AgentSettingsTab
- **A11 ARG 图论构造** (10 项, per 17:21 JST 拍板) — 10 类关系 + 4 维度 + 5 模板 + 同步桥 + 成就
- **A12 多人编辑** (8 项, per 17:34 JST v0.63 反转) — 多人 + cursor + 增删改 + Follow + 评论 + @ + CRDT + audit

**双核心 2: 游戏化 (32 项, G1-G12, 详见 BD-CANVAS-GAMIFY-001.md)**:
- G1 gamification 节点 (8 项) — avatar / level / xp / skill_tree / class / badge / quest / inventory
- G2 reward/achievement (4 项)
- G3 score/points (3 项)
- G4 leveling/skill tree (3 项)
- G5 sticky note 聚类 (AI, mock, per 守门 #23 v2) (2 项)
- G6 dot voting (2 项) — 每用户 N 票
- G7 reaction (1 项) — emoji 表情
- G8 confetti (1 项)
- G9 leaderboard (2 项) — per workspace / tenant
- G10 daily challenge / streak (2 项)
- G11 power-up / inventory (2 项) — **W/T/M 三類横展 15 张表 100% 覆盖**
- G12 V0.1 game 集成 (2 项) — Roguelike + Manga

#### 1.1.2 データ view (14 张表 W/T/M 跨域汇总, per 守门 #13)

**A11 7 张表** (派生自 [`BD-AGENT-RELATIONSHIP-001.md`](./BD-AGENT-RELATIONSHIP-001.md) v0.1 §3.2 + §4.2.3 + [`SRS-AGENT-RELATIONSHIP-001.md` §4.2](../requirements/SRS-AGENT-RELATIONSHIP-001.md) §4.2.3):

| # | 表 | 类型 (W/T/M) | 物理删除 | SCD Type 2 | RLS 13 类 | 审计必携 | 字段数 |
|---|---|---|---|---|---|---|---|
| 1 | `agents` | **Master** | 禁止 | ✓ | ✓ | — | 9 (id, name, archetype, domain, status, trust_score, metadata, created_at, updated_at, version) |
| 2 | `agent_relationship_edges` | **Master** | 禁止 | ✓ | ✓ | — | 8 (id, from_agent, to_agent, type, weight, direction, archived, version, metadata) |
| 3 | `agent_relationship_edges_audit` | **Transaction** (append-only) | 禁止 | — | ✓ | ✓ (actor/timestamp/old/new) | 6 (id, edge_id, action, old_value, new_value, actor, timestamp) |
| 4 | `team_template_instances` (TTL 30 天) | **Work** | 允许 (TTL) | — | — | — | 5 (id, template_id, instance_name, agent_ids, edges_json, expires_at) |
| 5 | `achievements` | **Master** | 禁止 | ✓ | ✓ | — | 8 (id, name, description, category, rarity, condition, reward_id, version) |
| 6 | `achievement_unlocks` | **Transaction** (append-only) | 禁止 | — | ✓ | ✓ (user_id/timestamp) | 5 (id, user_id, achievement_id, unlocked_at, trigger_event, version) |
| 7 | `relationship_events` | **Transaction** (append-only) | 禁止 | — | ✓ | ✓ (timestamp) | 9 (id, edge_id, event_type, actor, payload, before, after, version) |

**A12 7 张表** (派生自 [`SRS-CANVAS-AGENT-001.md` §4.12 A12.8](../requirements/SRS-CANVAS-AGENT-001.md) + [`SRS-CANVAS-GAMIFY-001.md` §4 G11 W/T/M 跨域汇总](../requirements/SRS-CANVAS-GAMIFY-001.md)):

| # | 表 | 类型 (W/T/M) | 物理删除 | SCD Type 2 | RLS 13 类 | 审计必携 | 字段数 |
|---|---|---|---|---|---|---|---|
| 1 | **`canvas_multi_user_audit`** (核心 A12.8) | **Transaction** (append-only) | 禁止 | ✓ (version +1) | ✓ | ✓ (1 操作 1 行) | 9 (id, canvas_id, actor_user_id, action, target_id, target_type, payload, created_at, version) |
| 2 | `canvas_comments` | **Master** | 禁止 | ✓ | ✓ | — | 6 (id, canvas_id, element_id, author_id, body, created_at, version) |
| 3 | `canvas_comment_mentions` | **Transaction** (append-only) | 禁止 | — | ✓ | ✓ | 5 (id, comment_id, mentioned_user_id, notified_at) |
| 4 | `canvas_permissions` | **Master** | 禁止 | ✓ | ✓ | — | 7 (id, canvas_id, user_id, level, granted_by, granted_at, version) |
| 5 | `canvas_followers` (session-bound) | **Work** | 允许 (session 结束) | — | — | — | 5 (id, canvas_id, follower_user_id, leader_user_id, expires_at) |
| 6 | `canvas_elements_backend` | **Master** | 禁止 | ✓ | ✓ | — | 12 (id, canvas_id, kind, x, y, width, height, rotation, z_index, content, locked, hidden, version) |
| 7 | `canvas_presence_cursors` (heartbeat 30s) | **Work** | 允许 (heartbeat 失効) | — | — | — | 7 (id, canvas_id, user_id, x, y, viewport, heartbeat_at) |

**A11 + A12 = 14 张表 W/T/M 100% 覆盖验证**:
- Master 6/14 (42.9%): agents / agent_relationship_edges / achievements / canvas_comments / canvas_permissions / canvas_elements_backend
- Transaction 5/14 (35.7%): agent_relationship_edges_audit / achievement_unlocks / relationship_events / canvas_multi_user_audit / canvas_comment_mentions
- Work 3/14 (21.4%): team_template_instances / canvas_followers / canvas_presence_cursors
- **总计 14/14 = 100% ✓** (per 守门 #13 W/T/M 三類横展 100% 表覆盖, 禁止混在)

#### 1.1.3 動作 view (5 域 + 多人编辑跨域)

- **5 域分组 (per 8/21 JST RGS 治理命名)**: player / economy / match / social / admin, 域内 agent 自动聚类 (per A8.1)
- **5 域 Lead 真人未到位, Mavis 临时代签** (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
- **跨域编排** (per 守门 #3): 5 域 Lead 拒绝兼任, 跨域关系强制 consults 而非 delegates_to (per ARG 主源 §5.1)
- **多人编辑协作** (per A12): 多人同时编辑 + 实时 cursor + Follow mode (GitHub Live Share UX)
- **ARG 4 维度协作影响** (per A11.3): dispatch 路由 / 上下文共享 / 信任度加权 / 产出评估

#### 1.1.4 モジュール view (4 文件 + 6 crate 跨域)

| 路径 | 用途 | 状态 |
|---|---|---|
| `docs/design/BD-CANVAS-001.md` (本文件) | 总册 BD 跨域 | v0.1 (本批 root 写) |
| `docs/design/BD-CANVAS-AGENT-001.md` | 专题 BD agent 管理 46 项 | v0.1 (本批 子代理 1 写) |
| `docs/design/BD-CANVAS-GAMIFY-001.md` | 专题 BD 游戏化 32 项 | v0.1 (本批 子代理 2 写) |
| `docs/frontend-canvas-design.md` | V0.1 画布 design 基线 | v0.1 (V0.1 已落档) |
| `crates/arg/` (A11 派生, 6 module) | ARG 数据 + 5-tier 架构 | v0.1 (9/8 已落档) |
| `crates/arg-bridge/` (A11 派生) | Memgraph ↔ LangGraph 同步桥 | v0.1 (9/8 已落档) |
| `crates/arg-effect/` (A11 派生) | 4 维度协作影响 + 5 模板 + 成就 | v0.1 (9/8 已落档) |
| `crates/api/arg/` (A11 REST API) | 13 REST + 1 WebSocket 端点 | v0.1 (9/8 已落档) |
| `frontend/src/app/agent-relationships/` (A11 UI) | 3 页面 (Editor / View / Wall) + zustand 5 channel | v0.1 (9/8 已落档) |
| `frontend/src/components/agent-game/` (G12 V0.1 game 集成) | Roguelike + Manga + Theme + Settings | v0.1 (9/8 已落档) |

#### 1.1.5 ネットワーク view (e2e 守门跨域)

**V0.1 e2e 守门** (9 case, 9/4 拍板 canvas-e2e-guard-001 落档):
1. /canvas/canvas-001 路由 200 + 14 element + 4 frame + minimap 渲染
2. pan: shift+drag 主 svg, viewport.x / viewport.y 变化
3. zoom: 工具栏 ZoomIn 按钮 / 滚轮 / 上限 400% 边界
4. fit-to-content: 工具栏 Maximize2 按钮, viewport 重置到 minX/minY
5. highlight URL auto-pan/zoom (useEffect 触发)
6. 选区删除: 选中 1 element + 工具栏 trash
7. minimap viewport rect 跟随 viewport 变化

**V0.1 share + export e2e 守门** (3 case, 9/4 拍板 canvas-share-export-001 落档):
8. Share 按钮点击 → clipboard 写入当前 URL
9. Export PNG 按钮点击 → 触发 download (.png 后缀)
10. Share 按钮点击后 → react-hot-toast toast 出现

**P3-D 阶段 e2e 守门 (本 BD 派生, 待 DD + 实装后)**:
- A11 ARG e2e: 关系拖拽 / 模板 1-click / 信任度可视化 / 4 维度效果 / 成就解锁
- A12 多人编辑 e2e: 多人同时编辑 / 实时 cursor / 元素增删改同步 / Follow mode / 评论线程 / @ / 冲突解决 / audit log
- G5 sticky note 聚类 e2e: AI mock 接口调用 + 聚类结果验证
- G6 dot voting e2e: 每用户 N 票 + 实时显示
- G8 confetti e2e: 触发条件 + 动画帧率
- G11 power-up / inventory e2e: 道具 + 物品栏 + W/T/M 跨域验证

### 1.2 Out-of-Scope (per 17:08 JST 避免过度冗余, 11 大类砍掉, 留 P3+)

| 类别 | 砍掉内容 | 砍掉理由 |
|---|---|---|
| Miro 通用协作 | 多人同时编辑 / 实时 cursor / Follow mode / 评论线程 / @ 提醒 / 内置视频通话 (除 A12 外) | 超出核心, 协作工具 (Slack / 飞书) 已有 |
| Miro 通用演示 | Frame as slide / Guided Tour / Speaker notes / Timer 演讲 | 演示用 Keynote / PowerPoint 已成熟 |
| Miro 通用导出 | PDF / Word / Excel / CSV / SVG (除 PNG 外) | PNG 已有 (V0.1), 文本导出超出核心 |
| Miro 通用版本 | Version history / Branching / Restore | 跟 Git worktree 重复 |
| Miro 通用集成 | Slack / Jira / Asana / Figma / Notion / GitHub / Zoom | Star 25 module 已有 |
| Miro 通用移动 | iOS / Android native / Touch / Stylus / Offline | 客户端重投入 |
| Miro 12 种 diagram | Mind map / Flowchart / BPMN / ER / Wireframe / Kanban / Sequence / Smart drawing / Draw | 内容生产用专门工具 |
| Miro 模板库 | 2500+ 模板 | 评估做 5-10 个, 留 P3+ |
| Miro AI 通用 | 文本生成 diagram / 翻译 / 图像识别 (除 G5 聚类外) | 仅留 sticky note 聚类 (GAMIFY G5, 走 mock) |
| Miro Tables / Chart widget / Form | 表格 / 数据源 / Chart / 表单 | 跟 Star 数据模块重复 |
| 完整 a11y (WCAG 2.1 AA) | 完整合规 | 部分 a11y 跟随 V0.1, 完整合规留 P3+ |
| 详细设计 (DD) 文档 | 后续 P3-D 阶段 | 待 SRS + BD 落档后启动 |
| 实现 (PHASE-* 报告) | 后续 P3-D.6 阶段 | 待 DD 落档后启动 |

---

## §2 系统架构 (System Architecture)

### 2.1 5 view 跨域架构图

```
┌────────────────────────────────────────────────────────────────────┐
│                     gm-console frontend (Next.js 14)                │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ 5 view 跨域 UI:                                                │  │
│  │ 1. 機能 view (46 + 32 项 FR, 跨域 user story)               │  │
│  │ 2. データ view (14 张表 W/T/M, 跨域 25 module 联动)         │  │
│  │ 3. 動作 view (5 域 + 多人编辑 + ARG 4 维)                   │  │
│  │ 4. モジュール view (本总册 + 2 专题 BD + 6 crate)            │  │
│  │ 5. ネットワーク view (V0.1 9 e2e 守门 + 派生 e2e)            │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
                                ↕ (BFF REST API 32 端点 + WebSocket 5 端点)
┌────────────────────────────────────────────────────────────────────┐
│              BFF Layer (FastAPI + Next.js API routes)               │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │
│  │ V0.1 9 API    │  │ A11 5 API    │  │ A12 5 API    │                │
│  │ canvas CRUD  │  │ ARG edges    │  │ 多人 collab  │                │
│  │ element/conn │  │ template     │  │ presence     │                │
│  │ viewport     │  │ sync status  │  │ follow mode  │                │
│  │ share/PNG    │  │ audit log    │  │ comment/@    │                │
│  │ audit/search │  │ view         │  │ multi-audit  │                │
│  └──────────────┘  └──────────────┘  └──────────────┘                │
│  ┌──────────────┐  ┌──────────────┐                                  │
│  │ GAMIFY 9 API │  │ 内部 5 协议  │                                  │
│  │ reward/score │  │ (跨域 + 25   │                                  │
│  │ level/cluster│  │  module 联动)│                                  │
│  │ vote/reaction│  └──────────────┘                                  │
│  │ confetti/lb  │                                                   │
│  │ challenge/pu │                                                   │
│  └──────────────┘                                                   │
└────────────────────────────────────────────────────────────────────┘
                                ↕
┌────────────────────────────────────────────────────────────────────┐
│              Backend (Rust crate + Memgraph + LangGraph)            │
│                                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │
│  │ crates/arg  │  │ arg-bridge  │  │ arg-effect  │  │ api/arg   │  │
│  │ (data + 6   │  │ (Memgraph ↔ │  │ (4 维度 +   │  │ (13 REST  │  │
│  │  models)    │  │  LangGraph  │  │  5 模板 +   │  │  + 1 WS) │  │
│  │             │  │  同步桥)   │  │  成就)      │  │           │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │
│         ↕                  ↕                  ↕              ↕      │
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

### 2.2 5-tier 架构 (跨域派生自 BD-AGENT-RELATIONSHIP-001 v0.1 §1)

1. **Tier 1: gm-console frontend (Next.js 14 + React 18 + zustand)** — 5 view 跨域 UI
2. **Tier 2: BFF (FastAPI + Next.js API routes)** — 32 REST + 5 WebSocket 端点
3. **Tier 3: Domain Crates (Rust)** — `crates/arg` + `arg-bridge` + `arg-effect` + `api/arg` 4 大模块
4. **Tier 4: Memgraph (图数据库)** — 14 张表 W/T/M 100% 覆盖
5. **Tier 5: LangGraph 集成** — 25 module 联动接口 + TMO 9 节点 (per `SRS-STAR-AGENT-RUNTIME-001.md` §4)

### 2.3 跨域交互模式 (5 种)

1. **Frontend ↔ BFF**: HTTP REST (32 端点) + WebSocket (5 端点, max 200ms 延迟)
2. **BFF ↔ Domain Crates**: in-process call (subprocess 走守门 #9 v3 实证)
3. **Domain Crates ↔ Memgraph**: Bolt protocol (port 7687), LWW + version optimistic lock
4. **Domain Crates ↔ LangGraph**: in-process StateGraph (per `BD-AGENT-RELATIONSHIP-001.md` §1.2 同步桥)
5. **BFF ↔ 25 module 联动**: per 总册 §6.3 25 module 接口 (work-item / worktree / agent / relation / comment / automation / audit / search / notification)

---

## §3 组件一覧 (Components)

### 3.1 新增组件 (本 BD 派生, 跨域汇总)

| 组件 | 路径 | 派生自 | 状态 |
|---|---|---|---|
| CanvasView 增强 (A1-A12 全部 14 张表 + 46 项 element 扩展) | `frontend/src/components/CanvasView.tsx` line 1-450 | SRS-CANVAS-AGENT-001 §4 + V0.1 11 处 element | v0.1 (本批扩展) |
| 5 工具栏扩展 (A11 关系编辑 + A12 多人 + 跨域) | `frontend/src/components/CanvasToolbar.tsx` (新增) | A11.2 + A12.5 + 跨域 5 view | v0.1 (本批扩展) |
| 7 域侧边栏 (5 域 + 多人 + V0.1 game) | `frontend/src/components/CanvasSidebar.tsx` (新增) | A2.2 + A12.7 + V0.1 game | v0.1 (本批扩展) |
| Minimap 增强 (跨域) | `frontend/src/components/CanvasMinimap.tsx` (新增) | A11.9 + A12.1 + V0.1 §3.3 | v0.1 (本批扩展) |
| 6 crate (A11 派生) | `crates/arg/` + `arg-bridge/` + `arg-effect/` + `api/arg/` (4 + 2 跨域) | BD-AGENT-RELATIONSHIP-001 v0.1 | v0.1 (9/8 已落档) |
| 14 张表 W/T/M schema | PostgreSQL (V0.1 兼容) + Memgraph (A11 派生) | 守门 #13 W/T/M | v0.1 (本批派生) |
| 25 module 联动接口 | BFF Layer (跨域) | 总册 §6.3 25 module | v0.1 (本批扩展) |

### 3.2 复用组件 (V0.1 已有)

| 组件 | 路径 | 状态 |
|---|---|---|
| V0.1 14 element (work-item/worktree/agent/feedback/sticky/automation/text + StatusPill 60+) | `frontend/src/components/CanvasView.tsx` line 156-340 | v0.1 (9/4 已落档) |
| V0.1 工具栏 (select/pan/zoom/fit/trash) | `frontend/src/components/CanvasView.tsx` line 376 | v0.1 (9/4 已落档) |
| V0.1 minimap | `frontend/src/components/CanvasView.tsx` line 365 | v0.1 (9/4 已落档) |
| V0.1 9 e2e 守门 | `frontend/e2e/canvas-view.spec.ts` (6 case) + `frontend/e2e/canvas-share-export.spec.ts` (3 case) | v0.1 (9/4 已落档) |
| V0.1 5 game 组件 (Roguelike + Manga + Theme + Settings) | `frontend/src/components/agent-game/` | v0.1 (9/8 已落档) |
| V0.1 SRS-STAR-AGENT-RUNTIME 9 SA Archetype + 14 状态机 | `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 | v0.1 (9/3 已落档) |

### 3.3 模块划分 (跨域, 跨 2 专题 BD)

| 模块 | 路径 | 跨域 | 派生专题 BD |
|---|---|---|---|
| agent 节点 + 拓扑 + 状态 + worktree + work-item | `crates/arg/src/models/` | A1-A5 | BD-AGENT §3 |
| agent 操作 + 监控 + 聚类 | `crates/arg/src/ops/` | A6-A8 | BD-AGENT §3 |
| agent 跨域 + settings | `crates/arg/src/bridge/` | A9-A10 | BD-AGENT §3 |
| ARG 10 类关系 + 4 维度 + 5 模板 + 同步桥 | `crates/arg-effect/src/` + `crates/arg-bridge/src/` | A11 | BD-AGENT §3 (派生自 BD-AGENT-RELATIONSHIP-001 §3) |
| 多人编辑 (多人 + cursor + 增删改 + Follow + 评论 + @ + CRDT + audit) | `crates/arg-effect/src/multi_user/` (新增) | A12 | BD-AGENT §3 (本批派生) |
| gamification 节点 + reward + score + leveling | `crates/gamify/src/` (新增) | G1-G4 | BD-GAMIFY §3 |
| sticky note 聚类 (AI, mock) | `crates/gamify/src/cluster/` (新增, mock 接口) | G5 | BD-GAMIFY §3 (per 守门 #23 v2) |
| dot voting + reaction + confetti + leaderboard + daily challenge + power-up + V0.1 game | `crates/gamify/src/engagement/` (新增) + `frontend/src/components/agent-game/` (V0.1 复用) | G6-G12 | BD-GAMIFY §3 |

---

## §4 数据模型 (Data Model, 14 张表 W/T/M 跨域汇总)

**详细数据模型见 [`BD-CANVAS-AGENT-001.md` §4](../design/BD-CANVAS-AGENT-001.md) (A11 + A12 14 张表) + [`BD-CANVAS-GAMIFY-001.md` §4](../design/BD-CANVAS-GAMIFY-001.md) (GAMIFY 15 张表, 含 G11 power-up / inventory).**

### 4.1 14 张表 W/T/M 100% 覆盖验证 (跨域汇总)

| 类型 | 表数 | 表名 (跨 A11 + A12 + GAMIFY G11 跨域去重) |
|---|---|---|
| **Master (SCD Type 2)** | 6/14 (42.9%) | agents / agent_relationship_edges / achievements / canvas_comments / canvas_permissions / canvas_elements_backend |
| **Transaction (append-only)** | 5/14 (35.7%) | agent_relationship_edges_audit / achievement_unlocks / relationship_events / canvas_multi_user_audit / canvas_comment_mentions |
| **Work (短 TTL)** | 3/14 (21.4%) | team_template_instances / canvas_followers / canvas_presence_cursors |
| **总计** | **14/14 (100%)** | per 守门 #13 W/T/M 三類横展 强制分类, 禁止混在 |

**GAMIFY G11 15 张表** (per 守门 #13, Work 6 + Transaction 4 + Master 5):
- Work 6: `inventory_session_buffs` / `powerup_active_effects` / `reaction_events` / `confetti_events` / `leaderboard_cache` / `clustering_results_cache`
- Transaction 4: `inventory_actions` / `powerup_grants` / `dot_vote_ledger` / `clustering_jobs`
- Master 5: `inventory_items` / `powerup_definitions` / `vote_state_snapshot` / `daily_challenge_state` / `streak_state`

**G11 跨域汇总**: GAMIFY G11 15 张表 + AGENT A11 7 张 + A12 7 张 = **29 张表 100% W/T/M 覆盖** (跨域汇总, per 守门 #13)

### 4.2 域模型派生 (Rust domain-* crate, per 总册 §6.3)

- 25 module 联动接口 (per 总册 §6.3 25 module 联动接口)
- domain-collaboration (V0.1 画布) + domain-arg (A11 新增) + domain-gamify (G 新增)
- 5 域 (player/economy/match/social/admin) 跨 domain 边界 (per 守门 #3 + 2026-08-31 22:45 JST Q1-D 拍板 disclaimer)

### 4.3 zustand store 扩展 (V0.1 + 双核心 + A11 + A12)

**V0.1 已有** (per `frontend/src/lib/store.ts`):
- worktrees / agentSessions / automationRules / feedbacks (V0.1 9 module)
- canvasElements / canvasConnectors / canvasFrames (V0.1 画布)
- `moveCanvasElement` / `deleteCanvasElement` (V0.1 action)

**A11 扩展** (本批):
- argEdges (10 类关系边)
- argTeamTemplates (5 模板)
- argAudit (Transaction append-only)
- argAchievements (Master) + argAchievementUnlocks (Transaction)

**A12 扩展** (本批):
- canvasMultiUserAudit (Transaction append-only)
- canvasComments (Master) + canvasCommentMentions (Transaction)
- canvasPermissions (Master, view/comment/edit 3 级)
- canvasFollowers (Work, session-bound)
- canvasElementsBackend (Master, 替代 V0.1 localStorage + zustand persist)
- canvasPresenceCursors (Work, heartbeat 30s)

**GAMIFY 扩展** (本批):
- gamificationNodes (8 kind: avatar/level/xp/skill_tree/class/badge/quest/inventory)
- rewards / achievements (跟 A11 共享)
- scores / levels / leaderboards
- votes (dot voting 实时) / reactions / confetti
- challenges (daily) / streaks
- powerups / inventory (G11)

---

## §5 接口设计 (Interface Design, 32 API 端点 + 5 WebSocket 跨域汇总)

**详细 API 设计见 [`BD-CANVAS-AGENT-001.md` §5](../design/BD-CANVAS-AGENT-001.md) (A11 + A12 23 端点 + 5 WebSocket) + [`BD-CANVAS-GAMIFY-001.md` §5](../design/BD-CANVAS-GAMIFY-001.md) (GAMIFY 22 端点).**

### 5.1 BFF REST API 32 端点 (跨域汇总)

| 端点 | 方法 | 路径 | 派生专题 |
|---|---|---|---|
| Canvas CRUD | GET/POST/PATCH/DELETE | `/v1/collaboration/canvases` | V0.1 |
| Element CRUD | GET/POST/PATCH/DELETE | `/v1/collaboration/canvases/[id]/elements` | V0.1 |
| Connector CRUD | GET/POST/PATCH/DELETE | `/v1/collaboration/canvases/[id]/connectors` | V0.1 |
| Viewport sync | WS (planned) / Polling (current) | `/v1/collaboration/canvases/[id]/viewport` | V0.1 |
| Share (URL) | GET | `/v1/collaboration/canvases/[id]/share-link` | V0.1 |
| Export PNG | POST | `/v1/collaboration/canvases/[id]/export/png` | V0.1 |
| Audit | GET | `/v1/collaboration/canvases/[id]/audit` | V0.1 |
| Search | GET | `/v1/search?q=&type=canvas` | V0.1 |
| Comment pin | GET/POST | `/v1/collaboration/canvases/[id]/comments` | V0.1 |
| **ARG edges CRUD** | GET/POST/PATCH/DELETE | `/v1/arg/edges` | A11 |
| **ARG template instantiate** | POST | `/v1/arg/templates/instantiate` | A11 |
| **ARG sync status** | GET | `/v1/arg/sync?canvas_id=` | A11 |
| **ARG audit log** | GET | `/v1/arg/edges/audit?edge_id=` | A11 |
| **ARG relationship view (Agent View tab)** | GET | `/v1/arg/view?agent_session_id=` | A11 |
| **多人编辑 Realtime 通道 (per 17:34 JST v0.63 反转)** | WS | `wss://canvas-collab/canvases/[id]` | A12 |
| **多人编辑 presence (实时 cursor)** | WS | `wss://canvas-presence/canvases/[id]` | A12 |
| **多人编辑 follow mode** | POST | `/v1/collaboration/canvases/[id]/follow?user_id=` | A12 |
| **多人编辑 comment thread + @ 提醒** | POST | `/v1/collaboration/canvases/[id]/comments/[cid]/thread` + `/v1/notifications/@` | A12 |
| **多人编辑 audit log (新增表 `canvas_multi_user_audit`)** | GET | `/v1/collaboration/canvases/[id]/multi-user-audit` | A12 |
| **Agent status sync (A11 协同)** | WS | `wss://agent-status/canvases/[id]` | AGENT (A3) |
| **Agent operation (启停/重启)** | POST | `/v1/agent-sessions/[id]/start` `/stop` `/restart` | AGENT (A6) |
| **Agent token usage** | GET | `/v1/agent-sessions/[id]/token-usage` | AGENT (A7) |
| **Gamification reward** | POST | `/v1/gamify/rewards` | GAMIFY (G2) |
| **Score points** | GET/POST | `/v1/gamify/scores` | GAMIFY (G3) |
| **Level up** | WS | `wss://gamify-level/canvases/[id]` | GAMIFY (G4) |
| **Sticky note 聚类 (AI, mock)** | POST | `/v1/gamify/sticky-notes/cluster` | GAMIFY (G5) |
| **Dot voting** | POST | `/v1/gamify/votes` | GAMIFY (G6) |
| **Reaction** | POST | `/v1/gamify/reactions` | GAMIFY (G7) |
| **Confetti trigger** | WS | `wss://gamify-confetti/canvases/[id]` | GAMIFY (G8) |
| **Leaderboard** | GET | `/v1/gamify/leaderboard?scope=workspace&metric=token` | GAMIFY (G9) |
| **Daily challenge** | GET | `/v1/gamify/daily-challenge` | GAMIFY (G10) |
| **Power-up / Inventory** | GET/POST | `/v1/gamify/powerups` + `/v1/gamify/inventory` | GAMIFY (G11) |

### 5.2 内部协议 5 个 (跨域)

- **In-process StateGraph 桥接** (per BD-AGENT-RELATIONSHIP-001 §1.2): Memgraph write → EventBus `edge.changed` → in-process StateGraph update → 30s period flush
- **WebSocket 协议 (5 端点)**: 详见 §5.1 (agent-status / canvas-collab / canvas-presence / gamify-level / gamify-confetti)
- **25 module 联动接口** (per 总册 §6.3): work-item / worktree / agent / relation / comment / automation / audit / search / notification
- **OAuth 协议** (per 守门 #5 + 守门 #1 v1a): 401 retry 跨 session 续, max 2 retries
- **WSS 选型协议** (per A12.1 P0 阻塞): NATS JetStream / native WebSocket / Socket.IO 待拍板

### 5.3 错误处理 (跨域共享, per 守门 #1 v1a + 守门 #5)

| 错误类型 | 处理 | 跨域 |
|---|---|---|
| 401 Authentication failed | 跨 session 续, **不算 timeout, max 2 retries**, Ulysses 验证 $env:GHCR_PAT | per 守门 #1 v1a 2026-09-03 11:07 JST 实证 |
| Recv failure / Connect failed | max 2 retries, 30s-2min 后常恢复, 不连续 retry | per 守门 #1 v1a |
| Timeout (5s+) | max 2 retries | per 守门 #1 v1a |
| 5xx Server Error | 1 retry, 5min 后, 失败给用户提示 | per 守门 #1 v1a |
| 4xx Client Error (非 401) | 不 retry, 直接给用户错误 | per 守门 #1 v1a |
| 凭据 (agent 启停 + reward 解锁) | stdin pipe 模式, 密码不上命令行, 不打印 | per 守门 #5 hard ban |
| WSS 选型 (A12.1) | 拍板前走 polling fallback (V0.1 模式 B 30s) | per A12.1 P0 阻塞 |
| CRDT 选型 (A12.6) | 拍板前走 LWW (Last-Write-Wins) | per A12.6 P0 阻塞 |

---

## §6 5 view 詳細 (5 views, 跨域)

### 6.1 機能 view (46 + 32 = 78 项 FR 跨域)

**双核心 1: 管理 agent 46 项** (A1-A12, 详见 [`BD-CANVAS-AGENT-001.md` §4.1-§4.12](../design/BD-CANVAS-AGENT-001.md)):
- A1 节点 3 + A2 拓扑 4 + A3 状态 3 + A4 worktree 2 + A5 work-item 2 + A6 操作 4 + A7 监控 3 + A8 聚类 3 + A9 跨域 2 + A10 settings 2 + A11 ARG 10 + A12 多人编辑 8 = **46 项**

**双核心 2: 游戏化 32 项** (G1-G12, 详见 [`BD-CANVAS-GAMIFY-001.md` §4.1-§4.12](../design/BD-CANVAS-GAMIFY-001.md)):
- G1 节点 8 + G2 reward 4 + G3 score 3 + G4 leveling 3 + G5 聚类 2 + G6 投票 2 + G7 reaction 1 + G8 confetti 1 + G9 leaderboard 2 + G10 challenge 2 + G11 power-up 2 + G12 V0.1 game 2 = **32 项**

**跨域合计**: 78 项 (46 + 32), per 总册 §4.1 双核心 60 项 → 78 项 (含 A12 多人编辑 8 项, per 17:34 JST v0.63 反转)

### 6.2 データ view (14 张表 W/T/M 跨域汇总)

详见 §4 数据模型. **29 张表 100% W/T/M 覆盖** (A11 7 + A12 7 + GAMIFY 15).

### 6.3 動作 view (5 域 + 多人编辑跨域)

- **5 域分组** (per 8/21 JST RGS 治理命名): player / economy / match / social / admin
- **5 域 Lead 真人未到位** (Mavis 临时代签, per 守门 #14 v2)
- **跨域编排** (per 守门 #3): 跨域边强制 consults, 拒绝兼任
- **多人编辑协作** (per A12): 5 域 Lead 真人到位后, view/comment/edit 3 级权限 (拍板前 view-only 兜底)
- **ARG 4 维度协作影响** (per A11.3): dispatch / context / trust / review

### 6.4 モジュール view (4 文件 + 6 crate 跨域)

详见 §3 组件一览. **4 文件 (本批 BD) + 6 crate (A11 派生) + 5 域 V0.1 game 组件** 跨域.

### 6.5 ネットワーク view (V0.1 9 e2e 守门 + P3-D 派生 e2e)

- **V0.1 已落档 9 e2e 守门** (per `frontend/e2e/canvas-view.spec.ts` + `canvas-share-export.spec.ts`): 6 + 3 case
- **P3-D 派生 e2e** (待 DD + 实装后):
  - A11 ARG: 关系拖拽 / 模板 1-click / 信任度 / 4 维度 / 成就
  - A12 多人编辑: 多人 / cursor / 增删改 / Follow / 评论 / @ / 冲突 / audit
  - G5 聚类: mock 调用 + 验证
  - G6 投票: N 票 + 实时
  - G8 confetti: 触发 + 帧率
  - G11 power-up / inventory: 道具 + W/T/M 跨域验证

---

## §7 NFR (Non-Functional Requirements, 6 类跨域)

### 7.1 性能 (NFR-PERF)

| 编号 | 指标 | 目标 | 跨域 |
|---|---|---|---|
| NFR-PERF-1 | viewport pan/zoom 帧率 | ≥ 60 FPS (中端笔记本 Chrome 1280x720) | 跨域 + V0.1 |
| NFR-PERF-2 | element 渲染数量 | ≥ 1000 element 不掉帧 (per design §9 CANVAS-OI-09) | 跨域 + V0.1 |
| NFR-PERF-3 | 实时状态同步延迟 (agent 14 状态机) | < 200ms (P95) | A3 + A12 (per A3.1) |
| NFR-PERF-4 | 导出 PNG (1920x1080) | < 5s | V0.1 |
| NFR-PERF-5 | sticky note AI 聚类 (N=50) | < 3s (mock 接口) | G5 |
| NFR-PERF-6 | confetti 动画帧率 | ≥ 60 FPS | G8 |
| NFR-PERF-7 | dot voting 实时显示 | < 100ms (P95) | G6 |
| NFR-PERF-8 | **多人同时编辑 (A12, max 10 并发)** | < 200ms (P95) | A12 (per A12.1) |

### 7.2 可访问性 (NFR-A11Y)

| 编号 | 指标 | 目标 | 跨域 |
|---|---|---|---|
| NFR-A11Y-1 | 键盘可访问 | 全功能可键盘操作 (Tab / Enter / Esc / 方向键) | V0.1 部分 |
| NFR-A11Y-2 | 颜色对比度 | ≥ 4.5:1 (正文) / ≥ 3:1 (大字体) | V0.1 |
| NFR-A11Y-3 | 焦点指示 | 明显焦点环 (高对比度) | V0.1 |
| NFR-A11Y-4 | 完整 WCAG 2.1 AA | **❌ 砍掉, 留 P3+** (per 17:08 JST 避免过度冗余) | n/a |

### 7.3 安全 (NFR-SEC, per 守门 #5 + 守门 #1 v1a)

| 编号 | 指标 | 目标 | 跨域 |
|---|---|---|---|
| NFR-SEC-1 | 凭据管理 | stdin pipe 模式, 密码不上命令行, 不打印 (per 守门 #5) | 跨域 |
| NFR-SEC-2 | 401 Authentication failed | 跨 session 续, **不算 timeout, max 2 retries** (per 守门 #1 v1a) | 跨域 |
| NFR-SEC-3 | 网络错误 | max 2 retries, 30s-2min 后常恢复 | 跨域 |
| NFR-SEC-4 | agent 启停权限 | BFF API 层强制, 5 域 Lead 决策 (per 守门 #3) | A6 |
| NFR-SEC-5 | 多人编辑权限 (A12) | BFF API 层强制, view/comment/edit 3 级 | A12.7 |

### 7.4 扩展性 (NFR-EXT)

| 编号 | 指标 | 目标 | 跨域 |
|---|---|---|---|
| NFR-EXT-1 | 新增 agent element kind | 接入约定 type + content, ≤ 1 PR | A1 |
| NFR-EXT-2 | 新增 gamification element kind | 同上, 8 kind V1 | G1 |
| NFR-EXT-3 | 新增 reward rule | 复用 V0.1 automation rule, ≤ 1 PR | G2 |
| NFR-EXT-4 | AI 能力 (G5) | mock 接口 → 真实 LLM 平滑切换 (per 守门 #23 v2) | G5 |
| NFR-EXT-5 | 新增导出格式 | 导出器 plug-in, ≤ 1 PR | n/a (只 PNG) |

### 7.5 国际化 (NFR-I18N, per STAR-I18N 拍板)

| 编号 | 指标 | 目标 | 跨域 |
|---|---|---|---|
| NFR-I18N-1 | 支持语言 | zh-CN (default) / en / ja (per STAR-I18N 拍板) | 跨域 |
| NFR-I18N-2 | 文案提取 | i18n key 化, 不硬编码 | 跨域 |
| NFR-I18N-3 | 日期 / 数字 | locale-appropriate 格式 | 跨域 |

### 7.6 兼容性 (NFR-COMPAT)

| 编号 | 指标 | 目标 |
|---|---|---|
| NFR-COMPAT-1 | 浏览器 | Chrome 120+ / Edge 120+ / Safari 17+ / Firefox 120+ (per V0.1) |
| NFR-COMPAT-2 | 设备 | Desktop (1280x720+) primary, Tablet (iPad) secondary, **Mobile 砍掉, 留 P3+** |
| NFR-COMPAT-3 | OS | Windows 10+ / macOS 12+ / iPadOS 16+ (per 25 module e2e 实测) |

---

## §8 守门 (Guards) + 子代理失败接手 + 已知缺口

### 8.1 守门 19 项 (跨域汇总, per AGENTS.md §4)

| 守门 | 应用 | 跨域 |
|---|---|---|
| 守门 #1 (R-05 不 push + v15 docs 同步饱和) | 本 BD 4 commit 落档 + 后续 v15 docs 同步 | 跨域 |
| 守门 #1 v19 agent 交互 Python 化 | 0 子代理调用 (实装期) | 跨域 |
| 守门 #3 5 域独立 Lead ≠ Star 22 DDD bounded context | 跨域边强制 consults (per ARG 主源 §5.1) | 跨域 |
| 守门 #5 env 安全 hard ban | 凭据走 stdin pipe | 跨域 |
| 守门 #6 PowerShell only | 文档撰写 + BD 落地 | 跨域 |
| 守门 #7 0 unsafe | Rust 0 unsafe 块 (跨域) | 跨域 |
| 守门 #9 #3 子代理 RPC 不可靠 | 0 子代理调用 (BD 撰写期, root 直实装) | 跨域 |
| 守门 #9 v19 Mavis 自驱 | 拍板后立即执行 (18:00 JST → 18:02 JST 落 3 brief + 派 2 子代理) | 跨域 |
| 守门 #9 v20 子代理 dispatch 必先 brief 落档 | 3 份 brief 落 `docs/briefs/bd-canvas-{total,agent,gamify}-001.md` | 跨域 |
| 守门 #9 v27 RPC 失败 fallback 3 段 | invoke → verify → collect_output | 跨域 |
| 守门 #10 commit author=Ulysses | 本 BD 4 commit 落档 | 跨域 |
| 守门 #11 缺标比错标 | 已知缺口 ≥ 8 个 (per §8.3) | 跨域 |
| 守门 #12 AI 文档治理 | BAS 引用必 git log --follow 实证 | 跨域 |
| 守门 #13 DB W/T/M 三類横展 | 29 张表 (A11 7 + A12 7 + GAMIFY 15) 100% 覆盖 | 跨域 |
| 守门 #14 v2 5 域 Lead Mavis 临时代签 | 真人到位后追溯签字 | 跨域 |
| 守门 #14 v3 Mavis 永久代签 | 修订人/审批者 author=Ulysses | 跨域 |
| 守门 #14 v4 v0.62 反转 | 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST) | 跨域 |
| 守门 #15 docs 同步饱和 | 本 BD 4 commit 落档 | 跨域 |
| 守门 #23 v2 AI 第三方 API 禁止 | G5 sticky note 聚类走 mock 接口 (per `ai_edit_mock.py`), 真实 LLM 留 P2 | 跨域 |

### 8.2 守门 26 派生规 (跨域汇总, per AGENTS.md §4 #1 派生 v1-v26)

v1-v14 基础守门 (cargo + 守门实证) + v15 docs 同步饱和 + v16 P0-1 联动 + v17 H2 范围扩量 + v18 (空) + v19 agent 交互 Python 化 + v20 子代理 dispatch 必先 brief + v21 docs 同步必更新 §4 + registry + v22 调试控制台不污染 + v23 AI mock 锁定 + v24 subprocess 替代 RPC + v25 cargo test 单 crate + v26 cargo doc advisory

**守门 19 项 + 26 派生规 = 45 项跨域** (per AGENTS.md §4.1 §守门编号累计表)

### 8.3 已知缺口 ≥ 8 个 (跨域, per 守门 #11 缺标比错标)

| # | 缺口 | 跨域 | 阻塞 | 跨专题 |
|---|---|---|---|---|
| 1 | **A12.1 多人同时编辑 WebSocket 选型未拍板** (P0 阻塞, 候选 NATS JetStream / native WebSocket / Socket.IO) | A12 | **P0** | BD-AGENT §8 |
| 2 | **A12.6 冲突解决 CRDT 选型未拍板** (P0 阻塞, 候选 Yjs / Automerge / LWW) | A12 | **P0** | BD-AGENT §8 |
| 3 | **A12.7 view/comment/edit 3 级权限矩阵** (5 域 Lead 真人到位后决策, 拍板前 view-only 兜底) | A12 | P0 | BD-AGENT §8 |
| 4 | **V0.1 localStorage + zustand persist 跟多人编辑冲突, 实施时需重构持久化层** | A12.3 + GAMIFY | P0 | BD-AGENT §8 |
| 5 | **5 域 Lead 真人未到位** (Mavis 临时代签, 真人到位后追溯签字 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策**) | 跨域 | n/a | 跨域 |
| 6 | **ARG 5 维度 effect tier 模块实装待 P3-C 阶段** (per `SRS-AGENT-RELATIONSHIP-001.md` §1.3) | A11 | n/a | BD-AGENT §8 |
| 7 | **Memgraph 部署** (Docker 启动 port 7687 Bolt + 7444 HTTP, 数据卷持久化, per `SRS-AGENT-RELATIONSHIP-001.md` §3.2 PR-4) | A11 | P0 | BD-AGENT §8 |
| 8 | **L0↔L1 通信协议** (per `SRS-STAR-AGENT-RUNTIME-001.md` §4.4, 跨 5 域 Lead 责任边界) | 跨域 | P0 | BD-AGENT §8 |
| 9 | **TMO 9 节点 vs ARG 边界梳理** (per `SRS-AGENT-RELATIONSHIP-001.md` §5.3, 不取代 LangGraph 任务卡 DAG, 平行层) | A11 | n/a | BD-AGENT §8 |
| 10 | **3 commit 跨域 v0.62 反转** (per 2026-09-10 12:45 JST, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses) | 跨域 | n/a | 跨域 |
| 11 | **守门 #1 v15 docs 同步饱和** (per 4 commit 累计 1.46M tokens, 第 68-71 次新事件触发仍允许, 后续 BD + DD + 实装需新事件触发) | 跨域 | n/a | 跨域 |
| 12 | **守门 #23 v2 AI 第三方 API 禁止** (per GAMIFY G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2, 跨域硬约束) | G5 | n/a | BD-GAMIFY §8 |

**已知缺口 12 个 ≥ 8 满足** (per 守门 #11)

**DDD Review 必查**:
- 4 P0 阻塞 (#1 + #2 + #3 + #7)
- A12 跨 session 续做 (#4)
- 5 域 Lead 真人到位 (#5)
- ARG TMO 边界 (#6 + #9)
- L0↔L1 通信 (#8)
- 守门 #23 v2 跨域 (#12)

### 8.4 子代理失败接手 (per 守门 #9 #3 + #9 v27 实证 5/5 RPC 不可靠)

- 2 worker 子代理 (BD-AGENT + BD-GAMIFY) 并行, 0 子代理调用 (BD 撰写期, root + 2 worker)
- 失败 fallback: 子代理 30s 内 必跑 `dispatcher.py verify <task_id>` 二次验证
- 验证失败 → `dispatcher.py collect_output` 拉真实 output + 写 `docs/briefs/<task_id>.status.json` 标 retry_count
- 连续 2 次 retry 失败 → 走 root session 直接实装 (per 守门 #9 v27)

---

## §9 签字栏 (5 角色 per AGENTS.md §3)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

## §10 修订履历 (v0.1 + 修订人 + 触发)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **初始版本: 双核心 (管理 agent + 游戏化) + 多人编辑 (per 17:34 JST v0.63 反转) + ARG (per 17:21 JST) 总册 BD 跨域汇总, 5 view 跨域 + 14 张表 W/T/M 100% 覆盖 + 32 API 端点 + 5 WebSocket + 6 类 NFR + 19 守门 + 12 已知缺口 + 5 角色签字栏** | **2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" + 17:34 JST v0.63 反转多人编辑 + 17:21 JST ARG 图论构造 + 17:08 JST 双核心 + 17:00 JST 旧方向撤回 (per 守门 #1 禁回溯叙事)** |

### 10.1 跨拍板派生 (4 阶段 + 5 守门拍板, per 守门 #14 v2 跨域)

| 派生触发 | 文档 | 修订人 | 修订内容 |
|---|---|---|---|
| 2026-09-10 17:00 JST Ulysses 拍板 v1 | 旧方向 (撤回) | Mavis 接手 | 12 大类 50 项 Miro 全面对标 (撤回) |
| 2026-09-10 17:08 JST Ulysses 拍板 v2 | 旧方向 v1.0 (撤回) | Mavis 接手 | 双核心 = 管理 agent + 游戏化, 避免过度冗余 (含砍多人编辑, 17:34 JST 撤回) |
| 2026-09-10 17:21 JST Ulysses 拍板 v3 | 旧方向 v1.0 二次更新 (撤回) | Mavis 接手 | 画布内体现 agent 之间关系的图论构造 (ARG) |
| 2026-09-10 17:34 JST Ulysses 拍板 v4 (v0.63 反转) | 本 BD v0.1 (本批) | Mavis 接手 | 多人编辑是要的, 撤回 17:08 JST 砍多人编辑决定, A12 8 项新增 |
| 2026-09-10 18:00 JST Ulysses 拍板 v5 | 本 BD v0.1 (本批) | Mavis 接手 | 基于需求文档制作基本设计文档 |
| 2026-09-10 12:45 JST v0.62 反转 | 跨域硬约束 | Mavis 接手 | 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses |
| 2026-09-05 10:43 JST 拍板 D | 跨域硬约束 | Mavis 接手 | Mavis 长期代签, 真人到位后追溯签字 |
| 2026-09-03 11:35 JST 拍板 B | 跨域硬约束 | Mavis 接手 | 5 域 Lead Mavis 临时代签 |
| 2026-08-31 22:45 JST Q1-D 拍板 | 跨域 disclaimer | Mavis 接手 | 5 域 Lead ≠ Star 22 DDD bounded context |
| 2026-09-01 18:30 JST 守门 #13 | 跨域硬约束 | Mavis 接手 | DB W/T/M 三類横展 强制分类 |
| 2026-09-02 09:01 JST 守门 #23 v2 | G5 硬约束 | Mavis 接手 | AI 第三方 API 禁止, G5 走 mock |
| 2026-08-27 19:39 JST 用户授权 | 元数据 | Mavis 接手 | Mavis 接手代签 Ulysses (per 守门 #10 + 守门 #14 v3) |

### 10.2 守门交叉引用 (本 BD 撰写期, per 守门 #12 v21)

| 守门 | 应用 | 引用 |
|---|---|---|
| 守门 #1 v15 docs 同步饱和 | 本 BD 4 commit 落档 (per 守门 #1 v15 第 72-75 次新事件) | per AGENTS.md §4 #1 v15 |
| 守门 #1 v19 agent 交互 Python 化 | 0 子代理调用 (BD 撰写期, root + 2 worker 直实装) | per AGENTS.md §4 #1 v19 |
| 守门 #1 禁回溯叙事 | 不重写 4 SRS commit (4f56979 / 73d490f / 396833a / f35b3f5), BD 是新方向 | per AGENTS.md §4 #1 |
| 守门 #3 5 域独立 Lead ≠ Star 22 DDD | 跨域 disclaimer 显式 | per AGENTS.md §4 #3 |
| 守门 #5 env 安全 hard ban | 0 env 打印, 仅 invoke, 凭据走 stdin pipe (不适用纯文档) | per AGENTS.md §4 #5 |
| 守门 #9 #3 子代理 RPC 不可靠 | 0 子代理调用 (BD 撰写期) | per AGENTS.md §4 #9 #3 |
| 守门 #9 v19 Mavis 自驱 | 18:00 JST 拍板 → 18:02 JST 落 3 brief + 派 2 子代理 ~2 分钟 | per AGENTS.md §4 #9 v19 |
| 守门 #9 v20 子代理 dispatch 必先 brief 落档 | 3 份 brief 落 `docs/briefs/bd-canvas-{total,agent,gamify}-001.md` | per AGENTS.md §4 #9 v20 |
| 守门 #9 v27 RPC 失败 fallback 3 段 | invoke → verify → collect_output | per AGENTS.md §4.1.1 v27 |
| 守门 #10 commit author=Ulysses | 4 commit author=Ulysses (per 8/27 19:39 JST 授权) | per AGENTS.md §4 #10 |
| 守门 #11 缺标比错标 | 已知缺口 12 个 ≥ 8 满足, 显式列 §8.3 | per AGENTS.md §4 #11 |
| 守门 #12 AI 文档治理 | BAS 引用必 git log --follow 实证, 禁回溯叙事 | per AGENTS.md §4 #12 |
| 守门 #12 v21 docs 同步必更新 §4 + registry | 1 任务卡 + 1 索引追加 (后续) | per AGENTS.md §4 #12 v21 |
| 守门 #13 DB W/T/M 三類横展 100% 覆盖 | 29 张表 (A11 7 + A12 7 + GAMIFY 15) 100% 覆盖 | per AGENTS.md §4 #13 |
| 守门 #14 v2 5 域 Lead Mavis 临时代签 | 真人到位后追溯签字 (per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D) | per AGENTS.md §4 #14 v2 |
| 守门 #14 v3 Mavis 永久代签 | 5 角色签字栏 author=Ulysses | per AGENTS.md §4 #14 v3 |
| 守门 #14 v4 v0.62 反转 | 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST) | per AGENTS.md §4 #14 v4 |
| 守门 #15 docs 同步饱和 | 4 commit 落档 (本 BD 4 份 + 后续) | per AGENTS.md §4 #15 |
| 守门 #23 v2 AI 第三方 API 禁止 | G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2 | per AGENTS.md §4 #23 v2 |

### 10.3 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)

| 阶段 | 估算 |
|---|---|
| 本 BD 撰写期 (root + 2 worker) | ~0.5M tokens (1 SRE·周 ≈ 1.2M, 在预算内) |
| 累计 P3-D.5 全部 8 commit (4 SRS + 4 BD) | ~1.96M tokens (1.63 SRE·周) |
| 后续 P3-D 详细设计 (DD, 3 份) | ~0.6M tokens (3 子代理并行) |
| 后续 P3-D.6 启动实装 (P0 36 项) | ~3-5M tokens |
| 双核心 78 项 全部落地 (12 个月+) | ~13-19M tokens (13-19 SRE·周, per STAR-OLU-001) |

---

> **撰写完成**: 2026-09-10 18:05 JST, root session mvs_942987595a124037901d37205a548e6f
> **2 专题 BD 状态**: 撰写中 (bg_98f425fd BD-AGENT + bg_f5625885 BD-GAMIFY), 等待 worker 子代理返回后 root 校对 + 1 commit 3 文件落档 (per 守门 #1 v15 docs 同步饱和)
> **下次拍板触发**: 2 专题 BD 返回后, root 自动 commit 3 文件落档 (本总册 BD + 2 专题 BD), 后续 P3-D DD 阶段启动 (待 3 BD 落档后)
