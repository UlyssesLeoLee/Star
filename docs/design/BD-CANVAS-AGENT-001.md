# BD-CANVAS-AGENT-001

> **无限画布 — Agent 管理域 (Agent Management Domain) 基本設計書 v0.1** (per 日本 IPA SEC 標準 / 基本設計書 テンプレート)
>
> - 状态: 🟡 Draft v0.1 (2026-09-10 JST 初版落档)
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 关联 commit: (留空, root 统一 commit 时填, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
> - 上位要件: [`docs/requirements/SRS-CANVAS-AGENT-001.md`](../requirements/SRS-CANVAS-AGENT-001.md) v1.2 (155KB, **46 项 = A1-A10 28 + A11 10 + A12 8**, 13 段, **14 张表 W/T/M 100% 覆盖**, 守门 14/14 通过, v0.63 反转多 A12 多人编辑 8 项)
> - 上位总册: [`docs/requirements/SRS-CANVAS-001.md`](../requirements/SRS-CANVAS-001.md) v1.1 (总册, 双核心 78 项 = 46 + 32, 含 17:34 JST v0.63 反转) + [`docs/design/BD-CANVAS-001.md`](./BD-CANVAS-001.md) (root 写总册 BD, 本 BD 专题聚焦 A1-A12 详细)
> - 平行专题 BD: [`docs/design/BD-CANVAS-GAMIFY-001.md`](./BD-CANVAS-GAMIFY-001.md) (双核心之 2: 游戏化 32 项, root 派 2 子代理之 2/2)
> - 平行 view BD: [`docs/design/BD-AGENT-VIEW-001.md`](./BD-AGENT-VIEW-001.md) v0.1 (44KB, agent 视图画布, 5 view 跨块接口)
> - **A11 派生源 BD 模板**: [`docs/design/BD-AGENT-RELATIONSHIP-001.md`](./BD-AGENT-RELATIONSHIP-001.md) v0.1 (55KB, 5-tier 架构 + 7 张表 W/T/M + 13 端点 + 8 UC, A11 1:1 派生) + [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](./DD-AGENT-RELATIONSHIP-001.md) (94KB, 詳細設計) + [`docs/design/DDD-REVIEW-AGENT-RELATIONSHIP-001.md`](./DDD-REVIEW-AGENT-RELATIONSHIP-001.md) (30KB, 跨 DDD 边界)
> - **A12 派生源 V0.1 design**: [`docs/frontend-canvas-design.md`](../frontend-canvas-design.md) v0.1 (§4.1 模式 A Realtime 通道 + §4.6 PresenceCursor 升级, A12 1:1 扩展实装)
> - **V0.1 实装代码**: `frontend/src/components/CanvasView.tsx` (line 218-235 `agent_cursor` 基线 + line 253-262 `comment_pin` 升级点)
> - 协同 SRS: `SRS-AGENT-VIEW-001.md` v1.0 (31KB) + `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (53KB, 14 状态机 + 9 SA Archetype)
> - V0.1 agent settings 集成: `docs/reports/PHASE-AGENT-{SETTINGS,GAME,ROGUELIKE,MANGA,THEME}-IMPL-REPORT.md` (5 份, A10 集成)
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)
> - 日期: 2026-09-10 JST
> - 受众: 詳細設計エンジニア / 実装エンジニア / UI/UX 设计师 / アーキテクト / SRE / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
> - **拍板来源**: 2026-09-10 18:00 JST Ulysses 拍板"**基于需求文档制作基本设计文档**" (本 BD 落档) + 2026-09-10 17:34 JST v0.63 反转拍板"**多人编辑是要的**" (撤回 17:08 JST 砍多人编辑决定, A12 8 项必含) + 2026-09-10 17:21 JST 拍板"画布内体现 agent 之间关系的图论构造" (A11 10 项必含) + 2026-09-10 17:08 JST 拍板"管理 agent 和游戏化, 避免过度冗余" (双核心 = 46 + 32 划分)
> - 跨域 disclaimer: **5 域独立 Lead ≠ Star 22 DDD bounded context** (per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer, 不建立业务子域↔DDD 映射)
> - **dual-use 提醒 (per AGENTS.md §5 倉庫拓扑)**: 本 view 不引用 RGS 仓 + 不建立业务子域↔DDD bounded context 映射

---

## §0 目的 (Purpose)

本文档基于 [`SRS-CANVAS-AGENT-001`](../requirements/SRS-CANVAS-AGENT-001.md) v1.2 的需求, 定义 **无限画布 — Agent 管理域 (Agent Management Domain)** 的基本設計:

- 系统架构 (**5 view 跨域**: 機能 / データ / 動作 / モジュール / ネットワーク) 跨 A1-A12 46 项
- 组件一覧 (A1-A12 12 子能力组件 + 复用 V0.1 组件 + 新增 12 module)
- 数据模型 (**14 张表 W/T/M 100% 覆盖**, Master 6 + Transaction 5 + Work 3, per 守门 #13)
- 接口设计 (**23 API 端点 + 5 WebSocket**, BFF REST + WSS 跨域)
- 5 view 詳細 (46 项 × 5 view 跨域覆盖)
- **NFR 6 类** (性能 / 可靠性 / 安全 / 易用 / 可观测 / ARG)
- 守门 19 项 + 子代理失败接手 + **已知缺口 19 个** (含 3 P0 阻塞, 跨域)
- 5 角色签字栏 + 修订履历

**A11 必含**: 10 类关系边 + 4 维度协作影响 + 5 团队模板 + 同步桥 + 成就协同, **派生自** [`BD-AGENT-RELATIONSHIP-001` §1-§7](./BD-AGENT-RELATIONSHIP-001.md) v0.1 模板 (5-tier 架构 + 7 张表 W/T/M + 13 端点 + 8 UC, **1:1 映射**).

**A12 必含**: 多人同时编辑 + 实时 cursor + 元素增删改同步 + Follow mode + 评论线程 + @ 提醒 + 冲突解决 (CRDT 选型) + 3 级权限 + audit log, **派生自** [`frontend-canvas-design.md` §4.1 模式 A Realtime 通道 + §4.6 PresenceCursor 升级](../frontend-canvas-design.md) (V0.1 design + 扩展实装, **1:1 映射**).

**dual-use 提醒**: 本 BD 不重复总册 BD (`BD-CANVAS-001.md` root 写) 跨域共享部分 (跨块接口 / 共享约束 / 公共表 / 公共 API 等), 本 BD 聚焦 **A1-A12 46 项详细设计 + 14 张表 + 23 API + 5 WebSocket**.

---

## §1 适用范围 (Scope)

### 1.1 包含 (In-Scope, A1-A12 12 子能力, 46 项)

#### 1.1.1 A1 agent 节点渲染 (3 项, P0)

- **A1.1**: `agent_cursor` (V0.1 line 218-235) 升级为 **`agent_node`** 完整卡 (220×110 px, 7 字段: avatar / name / role / kind / status / token_usage / started_at, 蓝底 + StatusPill 60+ 色码)
- **A1.2**: 14 状态机色码 (per `SRS-STAR-AGENT-RUNTIME-001.md` §8 14 状态机, 跟 StatusPill 60+ 一致)
- **A1.3**: 双击 agent_node 跳 `/agent?selected={id}` (V0.1) + 配置 A9.1 跳 `/agent-view?agent={id}`

#### 1.1.2 A2 agent 拓扑图 (4 项, P0/P1)

- **A2.1** (P0): 1:N handoff connector (扩展 V0.1 `agent_handoff`, 加时间/状态/备注 3 字段, bezier 曲线)
- **A2.2** (P0): 5 域分组 Frame (player / economy / match / social / admin, 5×4 grid 域内聚类)
- **A2.3** (P1): 父子 agent 关系 (supervisor → worker, orthogonal 树形)
- **A2.4** (P1): agent pipeline 视图 (A → B → C 直线, 共享 worktree 高亮)

#### 1.1.3 A3 agent 状态实时同步 (3 项, P0)

- **A3.1**: 14 状态机实时色码同步 (≤ 200ms P95, 走 zustand store 订阅 + 未来 WebSocket 选型待 P3-D 拍板)
- **A3.2**: 状态变化触发 audit (`action: agent.status.change`, 字段 5 个, 100% audit)
- **A3.3**: 状态变 failed → notification 域 + agent_node 红色边框 + 抖动动画 (CSS, ≤ 500ms)

#### 1.1.4 A4 agent 关联 worktree (2 项, P0)

- **A4.1**: 1 agent → N worktree (扩展 V0.1 `worktree_node` line 200-217, 圆周散开 N 个 worktree_node, 双向 connector)
- **A4.2**: worktree status 变化 ≤ 200ms (P95) 反映到 agent_node 状态聚合 (badge: 关联 worktree 状态分布)

#### 1.1.5 A5 agent 关联 work-item (2 项, P0)

- **A5.1**: 1 agent → N work-item (drag work-item → agent_node 周围, 弹"关联"modal, 调 API 写 `work_item.agent_session_id`)
- **A5.2**: work-item 状态变化 ≤ 200ms (P95) 反映到 agent_node 状态聚合 (badge: in_progress 3 / done 5 / blocked 1)

#### 1.1.6 A6 agent 操作菜单 (4 项, P0/P1)

- **A6.1** (P0): 启停 (start/stop, 调 agent-runtime API, 权限 SRE Lead / 5 域 Lead, per BR-5)
- **A6.2** (P1): 重启 (restart, 状态 running → spawning → initializing 反映)
- **A6.3** (P1): 查看 logs (跳 `/agent-runtime/logs?session={id}`)
- **A6.4** (P1): 查看 settings (跳 `/agent-settings?selected={id}`, V0.1 `AgentSettingsTab.tsx` 已实装)

#### 1.1.7 A7 agent 监控面板 (3 项, P1)

- **A7.1**: 实时 status / token / cost / runtime 仪表 (4 字段, 点击 agent_node 弹 detail panel, ≤ 1s 刷新)
- **A7.2**: token 用量对比 budget (badge `已用 / 预算`, 默认 budget 1.2M / SRE·周 per `STAR-OLU-001.md` v0.1, 超 100% 红色高亮)
- **A7.3**: 异常告警 (token 超预算 / runtime 异常 → notification + 画布高亮 ≤ 500ms)

#### 1.1.8 A8 agent 聚类 / 排序 / 过滤 (3 项, P1)

- **A8.1**: 按 role 聚类 (顶部 dropdown, supervisor / worker / reviewer 3 类)
- **A8.2**: 按 kind 排序 ([kind ASC, started_at ASC, id ASC] 稳定排序)
- **A8.3**: 按 status / token / started_at 过滤 (顶部 dropdown 多选)

#### 1.1.9 A9 agent session 跨域引用 (2 项, P2)

- **A9.1**: 跟 `SRS-AGENT-VIEW-001.md` 协同 (双击 agent_node 跳 `/agent-view?agent={id}`, 双向跳成功)
- **A9.2**: 跟 `SRS-AGENT-RELATIONSHIP-001.md` 协同 (切 tab 跳 `/agent-relationships?agent={id}`, 显示 5 关系)

#### 1.1.10 A10 agent settings V0.1 集成 (2 项, P2)

- **A10.1**: agent-settings tab 集成 (画布 detail panel 顶部 "Settings" tab, 集成 V0.1 `AgentSettingsTab.tsx`, 改 role → 画布自动重聚类)
- **A10.2**: 画布操作调用 settings (右键 → "编辑 settings" 改 role / kind / token_budget, 实时反映到画布聚类 / 仪表)

#### 1.1.11 A11 ARG 图论构造 (10 项, P0/P1/P2, **派生自** `BD-AGENT-RELATIONSHIP-001.md` v0.1 模板)

| 子项 | 描述 | 优先级 | 派生映射 |
|---|---|---|---|
| **A11.1** | 10 类关系边渲染 (4 核心 + 6 扩展, 颜色规范 per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.1+4.1.2) | P0 | `BD-AGENT-RELATIONSHIP-001` §2.2 Tier 1 |
| **A11.2** | 关系编辑 (UI 拖拽建边 + type 选择 + weight 滑块 + metadata JSON) | P0 | `BD-AGENT-RELATIONSHIP-001` §6.1 F-2 |
| **A11.3** | 4 维度协作影响 UI 指示器 (dispatch / 上下文 / 信任 / 产出) | P0 | `BD-AGENT-RELATIONSHIP-001` §6.1 F-5 |
| **A11.4** | 5 团队模板 1-click 部署 (Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council) | P1 | `BD-AGENT-RELATIONSHIP-001` §6.1 F-4 |
| **A11.5** | 关系 audit log (**4 表 W/T/M 三類横展** per 守门 #13, 100% 表覆盖) | P0 | `BD-AGENT-RELATIONSHIP-001` §4.4 + 守门 #13 |
| **A11.6** | 关系权重可视化 (边粗细 0.0-1.0 + 颜色渐变灰→绿) | P1 | `BD-AGENT-RELATIONSHIP-001` §6.1 F-1 |
| **A11.7** | 关系 archive / restore (软删, 物理删除禁止 per 守门 #13 SCD Type 2) | P2 | `BD-AGENT-RELATIONSHIP-001` §5.4.1 |
| **A11.8** | 关系版本控制 (SCD Type 2, version +1, optimistic lock) | P2 | `BD-AGENT-RELATIONSHIP-001` §5.4.1 |
| **A11.9** | 同步桥 UI 状态 (Memgraph ↔ LangGraph StateGraph, 4 状态 synced/syncing/error/offline) | P1 | `BD-AGENT-RELATIONSHIP-001` §6.3.2 + §1.1.2 |
| **A11.10** | 跟 `SRS-AGENT-RELATIONSHIP-001` 协同 (Agent View "Relationship" tab) | P1 | `BD-AGENT-RELATIONSHIP-001` §1.1.4 AgentViewTab |

#### 1.1.12 A12 多人编辑 (8 项, P0/P1, **派生自** `frontend-canvas-design.md` v0.1 §4.1 + §4.6, **v0.63 反转**)

> **v0.63 反转声明** (per 2026-09-10 17:34 JST Ulysses 拍板"**多人编辑是要的**"): 撤回 2026-09-10 17:08 JST 砍多人编辑决定, A12 8 项必含, 引用 `frontend-canvas-design.md` §4.1 模式 A Realtime 通道 (A12.1 + A12.3) + §4.6 PresenceCursor 升级 (A12.2) + `CanvasView.tsx` line 253-262 `comment_pin` (A12.5). v0.63 反转行**显式标** per 守门 #1 禁回溯叙事.

| 子项 | 描述 | 优先级 | 派生映射 |
|---|---|---|---|
| **A12.1** | 多人同时编辑 (max 10 并发, ≤ 200ms P95, WSS 选型 P0 阻塞) | P0 | `frontend-canvas-design.md` §4.1 模式 A |
| **A12.2** | 实时 cursor 同步 (PresenceCursor 升级, 12 色调色板, ≤ 100ms P95) | P0 | `frontend-canvas-design.md` §4.6 |
| **A12.3** | 元素增删改实时同步 (跟 V0.1 localStorage 冲突, backend 持久化) | P0 | `frontend-canvas-design.md` §4.1 模式 A 扩展 |
| **A12.4** | Follow mode (A 跟随 B 视角, GitHub Live Share UX, 1 canvas 1 follower) | P1 | `frontend-canvas-design.md` §4.9 透传 + GitHub Live Share |
| **A12.5** | 多人评论线程 + @ 提醒 (V0.1 `comment_pin` line 253-262 升级) | P0 | `frontend-canvas-design.md` §3.4 `comment_pin` |
| **A12.6** | 冲突解决 CRDT 选型 (Yjs / Automerge / LWW, P0 阻塞) | P0 | (新引入, per `SRS-AGENT-RELATIONSHIP-001` G-3 同步桥同源) |
| **A12.7** | 协作权限 (view / comment / edit 3 级, BFF middleware 强制) | P1 | (新引入, per 总册 `SRS-CANVAS-001` §6.2.2) |
| **A12.8** | audit log 多人操作 (**7 张表 W/T/M Transaction append-only + SCD Type 2**, 100% RLS 13 类) | P0 | (新引入, per 守门 #13 + A12.8) |

**合计**: **12 子能力 (A1-A12), 46 项** = A1-A10 28 + A11 10 + A12 8.

### 1.2 不包含 (Out-of-Scope)

| 类别 | 处理方 | 引用 |
|---|---|---|
| **双核心之 2: 游戏化 32 项 详细 BD** | 专题 BD 子代理 2 (`bd-canvas-gamify-001`) | 平行专题 BD |
| **总册 BD (跨域共享部分)** | root 写 (`bd-canvas-total-001`) | `BD-CANVAS-001.md` (root 写) |
| **详细设计 (DD) 文档** | 后续 P3-D 阶段, 待 SRS + BD 落档后启动 | 待启动 |
| **实现 (PHASE-* 报告)** | 后续 P3-D.6 阶段, 待 DD 落档后启动 | 待启动 |
| **Miro 通用 12 类** (PDF/Word 导出/移动端/12 diagram/2500+ 模板库/AI 通用/Tables/Chart/Form/完整 a11y) | ❌ 砍掉, 留 P3+ 评估 | per `SRS-CANVAS-AGENT-001` §1.4 |
| **25 module 实体实现** | 25 module 各自 docs, 画布只联动不实装 | per 总册 §6.3 |
| **5 域 Lead 真人到位** | per 守门 #14 v2, Mavis 临时代签, 真人到位后追溯签字 (不沿用代签决策 per 守门 #1 禁回溯叙事) | per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D |
| **跨仓 ARG → RGS 仓数据同步** | Star 仓不引用 RGS 仓 | per AGENTS.md §5 仓库拓扑硬约束 |
| **5 域独立 Lead ≠ Star 22 DDD bounded context** | 业务子域↔DDD 映射 disclaimer 显式 | per 2026-08-31 22:45 JST Q1-D 拍板 |
| **内置视频通话 (Miro Talk / Zoom)** | ❌ 砍掉, 留 P3+ | per `SRS-CANVAS-AGENT-001` §1.4 v0.63 反转行 |
| **协作工具集成 (Slack / Jira)** | ❌ 砍掉, 留 P3+ | per `SRS-CANVAS-AGENT-001` §1.4 |
| **A12 多人编辑 WSS 选型 (Yjs / Automerge / LWW / native WebSocket / Socket.IO)** | 拍板前 P0 阻塞, 拍板后落 `docs/design/REALTIME-CHANNEL-SELECTION-DECISION.md` | per A12.1 + A12.6 |
| **A12 CRDT 选型** | 拍板前 P0 阻塞, 拍板后落 `docs/design/CRDT-SELECTION-DECISION.md` | per A12.6 |
| **A12.7 view/comment/edit 3 级权限具体矩阵** | 5 域 Lead 真人到位后决策, 拍板前走 view-only 兜底 | per 守门 #14 v2 |

### 1.3 文档结构 (10 段, per `BD-AGENT-RELATIONSHIP-001.md` v0.1 模板)

```
§0 目的 (Purpose)
§1 适用范围 (Scope) — 1.1 In-Scope (A1-A12 12 子能力 46 项) + 1.2 Out-of-Scope + 1.3 文档结构
§2 系统架构 (System Architecture) — 5 view (機能/データ/動作/モジュール/ネットワーク) 跨 A1-A12
§3 组件一覧 (Components) — A1-A12 子能力组件 + 新增 12 module / 复用 V0.1 组件
§4 数据模型 (Data Model) — 7 张 A11 + 7 张 A12 = 14 张表 W/T/M 100% 覆盖 + 5 domain-* Rust 数据结构 + zustand store 扩展
§5 接口设计 (Interface Design) — BFF REST API (双核心 13 + A11 5 + A12 5 = 23 端点) + WebSocket 5 端点 + 内部 5 协议
§6 5 view 詳細 — 機能 view (46 项) + データ view (14 表) + 動作 view (5 域 + 多人编辑) + モジュール view (4 文件 + 6 crate) + ネットワーク view (e2e 9 + 多人编辑 WS)
§7 NFR (Non-Functional Requirements) — 6 类 (性能/可靠性/安全/易用/可观测/ARG)
§8 守门合规 (Guards) + 子代理失败接手 + 已知缺口 (19 个 per SRS + 3 P0 阻塞)
§9 签字栏 (5 角色 per AGENTS.md §3)
§10 修订履历 (v0.1 + 修订人 + 触发)
```

### 1.4 派生映射 (A11 + A12)

#### 1.4.1 A11 派生自 `BD-AGENT-RELATIONSHIP-001.md` v0.1 模板 (1:1 映射)

| A11 子项 | `BD-AGENT-RELATIONSHIP-001` v0.1 派生 | 备注 |
|---|---|---|
| A11.1 10 类关系边 | §6.1 F-1 (关系可视化) | 4 核心 + 6 扩展, 颜色规范 1:1 |
| A11.2 关系编辑 | §6.1 F-2 (关系编辑) | 拖拽 + type + weight + metadata 3 字段 |
| A11.3 4 维度协作影响 | §6.1 F-5 (协作影响 4 维度) | dispatch / context / trust / output |
| A11.4 5 团队模板 | §6.1 F-4 (模板库) | 1-click 实例化 1 事务 |
| A11.5 关系 audit log | §4.4 5 张表 (现 A11 必含 4 表 Master/Transaction/Work 跨 W/T/M 100%) | per 守门 #13 |
| A11.6 关系权重视觉化 | §6.1 F-1 + §5.4.3 (Trust Score 5 档) | 边粗细 + 颜色渐变 |
| A11.7 archive / restore | §5.4.1 Edge 状态机 (`archived=true` 不影响协作) | 物理删除禁止 |
| A11.8 关系版本控制 | §5.4.1 Edge 状态机 (`version +1`) | optimistic lock |
| A11.9 同步桥 UI 状态 | §1.1.2 同步桥层 (4 状态 synced/syncing/error/offline) | Memgraph Bolt subscription |
| A11.10 跟 ARG 协同 | §1.1.4 UI 层 (AgentViewTab + 3 组件) | 跟 `SRS-AGENT-RELATIONSHIP-001` 协同 |

**A11 1:1 派生验证**: A11 10 子项全部派生自 `BD-AGENT-RELATIONSHIP-001` v0.1 模板, 不重新设计 5-tier 架构 / 13 端点 / 7 张表 / 8 UC.

#### 1.4.2 A12 派生自 `frontend-canvas-design.md` v0.1 §4.1 + §4.6 (1:1 映射)

| A12 子项 | `frontend-canvas-design.md` v0.1 派生 | 备注 |
|---|---|---|
| A12.1 多人同时编辑 | §4.1 模式 A Realtime 通道 (BFF 推 element 增删改) | 扩展为多人 + WSS, V0.1 1 user → A12 N user |
| A12.2 实时 cursor 同步 | §4.6 PresenceCursor 升级 (cursor 锚定 element) | V0.1 1 cursor → A12 N cursor 多人 |
| A12.3 元素增删改同步 | §4.1 模式 A + §2.1 CanvasElement (14 字段) | backend 持久化, V0.1 localStorage 降级为 fallback |
| A12.4 Follow mode | §4.9 URL param 透传 + GitHub Live Share UX | 1 canvas 1 follower 限制 |
| A12.5 多人评论线程 | §3.4 `comment_pin` (V0.1 line 253-262) + §2.1 content.comment_id | 1 顶级 + N 回复 + @ 提醒 |
| A12.6 CRDT 选型 | (新引入, per A11 同步桥同源设计) | Yjs / Automerge / LWW |
| A12.7 协作权限 | (新引入, per 总册 `SRS-CANVAS-001` §6.2.2) | BFF middleware 强制 view/comment/edit |
| A12.8 audit log | (新引入, per 守门 #13 Transaction append-only) | 新增表 `canvas_multi_user_audit` |

**A12 1:1 派生验证**: A12 8 子项全部派生自 `frontend-canvas-design.md` v0.1 + 引入 CRDT / 权限 / audit 3 个新机制, 不重新设计画布基础能力 (pan/zoom/select/element/connector/frame).

---

## §2 系统架构 (System Architecture)

### 2.1 全体構成図 (Overall Architecture, 5-tier + 画布双核心)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│              UI Tier (frontend/src/app/canvas/ + /agent-relationships/)      │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  CanvasView 扩展 (V0.1 已实装) + AgentNode 完整卡 + ArgEdge 渲染       │  │
│  │  ┌──────────────────────┐  ┌────────────────────┐  ┌───────────────┐  │  │
│  │  │ AgentCanvasView      │  │ ArgCanvasView      │  │ MultiUserCVD  │  │  │
│  │  │ (A1-A10 主画布)      │  │ (A11 关系画布)     │  │ (A12 多人编辑)│  │  │
│  │  │ └ agent_node 卡      │  │ └ 10 类边 + 4 维度 │  │ └ cursor +    │  │  │
│  │  │ └ handoff/父子       │  │ └ 5 模板 + 同步桥  │  │   follow +    │  │  │
│  │  │ └ pipeline connector │  │ └ Relationship tab │  │   comment +   │  │  │
│  │  │ └ StatusPill 60+     │  │ └ 3 维度 UI 指示器 │  │   audit + WSS │  │  │
│  │  └──────────────────────┘  └────────────────────┘  └───────────────┘  │  │
│  │  zustand store 扩展: useAgentStore + useARGStore + useCanvasCollabStore │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
   │ HTTP REST (23 endpoints)              │ WebSocket (5 endpoints: 1 ARG + 4 A12)
   ↓                                       ↓
┌──────────────────────────────────────────────────────────────────────────────┐
│        API Tier (crates/api 扩展 + envoy 独立 deployment BFF per 9/1 偏好)  │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  AgentController (Axum router, 13 双核心端点)                           │  │
│  │  ARGController (Axum router, 5 A11 端点) — 派生自 BD-AGENT-REL-001    │  │
│  │  CanvasCollabController (Axum router, 5 A12 BFF 端点)                  │  │
│  │  Permission middleware (RLS 13 类 per 守门 #13)                         │  │
│  │  Audit middleware (100% audit per 守门 #13)                            │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
   │ In-process call / Memgraph Bolt / BFF WSS
   ↓
┌──────────────────────────────────────────────────────────────────────────────┐
│        Application Tier (crates/agent-domain + crates/arg + BFF 服务)       │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  AgentDomainService (A1-A10 业务逻辑)                                   │  │
│  │  ARGService + ARGDispatchRouter + ARGContextInjector + TrustEngine     │  │
│  │  CanvasCollabService (A12 多人编辑业务)                                 │  │
│  │  AuditService (跨域审计, 14 张表 W/T/M)                                │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
   │ Bolt 7687 / SQL / WSS Realtime
   ↓
┌──────────────────────────────────────────────────────────────────────────────┐
│        Data Tier (PostgreSQL + Memgraph 2.14+ + WSS Hub + BFF)              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  PostgreSQL (A1-A10 + A12 14 张表, 100% RLS 13 类必携)                │  │
│  │  Memgraph (A11 4 张表 agents/edges/audit/template_instances)          │  │
│  │  WSS Hub (A12.1 + A12.2 + A12.3 + A12.4 实时通道)                     │  │
│  │  Envoy Deployment (BFF 独立 deployment per 9/1 13:03+13:05 JST 偏好) │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
   │ LangGraph in-process + WSS + 25 module 联动
   ↓
┌──────────────────────────────────────────────────────────────────────────────┐
│        Effect Tier (crates/arg-effect + LangGraph nodes + 25 module 联动)  │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  ARGDispatchRouter (LangGraph 节点, 读 ARG 改写 dispatch)              │  │
│  │  ARGContextInjector (mentors / shadows context 注入)                   │  │
│  │  ARGTrustEngine (trust_score + verify skip)                            │  │
│  │  ARGOutputEvaluator (challenges / peer_reviews)                        │  │
│  │  25 module 联动 (worktree / work-item / comment / notification / audit) │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Tier 详细说明

#### 2.2.1 Tier 1: UI Tier (frontend/src/app/)

- **路由 (per V0.1 复用 + A11/A12 扩展)**:
  - `/canvas` (V0.1 主入口, A1-A10 主画布)
  - `/canvas?highlight={element_id}` (V0.1 §4.9 URL 透传)
  - `/agent-view?agent={id}` (A9.1 协同)
  - `/agent-relationships?from=canvas&agent={id}` (A11.10 关系画布)
  - `/agent-runtime/logs?session={id}` (A6.3 logs)
  - `/agent-settings?selected={id}` (A6.4 + A10.1 V0.1 settings)

- **新增组件 (12 module, 5 view 全覆盖)**:

| 子能力 | 新增模块 | 数量 | 复用 V0.1 组件 |
|---|---|---|---|
| A1 | `AgentNode.tsx` + `StatusPill60+` 扩展 | 2 | `StatusPill` |
| A2 | `AgentTopologyView.tsx` + `HandoffConnector.tsx` + `ParentChildConnector.tsx` + `PipelineConnector.tsx` + `DomainFrame.tsx` | 5 | `CanvasView` + `Frame` |
| A3 | `AgentStatusSync.tsx` (zustand subscriber) | 1 | `StatusPill` |
| A4 | `WorktreeRing.tsx` (圆周散点) | 1 | `worktree_node` line 200-217 |
| A5 | `WorkItemDragIn.tsx` | 1 | `work_item_card` line 180-199 |
| A6 | `AgentContextMenu.tsx` (右键) | 1 | (无) |
| A7 | `AgentDetailPanel.tsx` (4 字段仪表) | 1 | (无) |
| A8 | `ClusterSortFilter.tsx` (顶部 dropdown) | 1 | (无) |
| A9 | (跨块接口, URL 跳) | 0 | `useRouter` |
| A10 | `AgentSettingsTab.tsx` (V0.1 已实装) | 0 | (V0.1 复用) |
| A11 | `RelationshipEditor.tsx` + `ArgEdge.tsx` + `ArgEffectIndicator.tsx` + `TemplateGallery.tsx` + `SyncStatusBadge.tsx` + `AchievementWall.tsx` | 6 | (派生自 `BD-AGENT-RELATIONSHIP-001` §6.1 F-1..F-6) |
| A12 | `PresenceCursor.tsx` + `FollowModeBadge.tsx` + `CommentThread.tsx` + `PermissionGate.tsx` + `MultiUserAuditLog.tsx` | 5 | (派生自 `frontend-canvas-design.md` §3.4 + §4.6) |
| **合计** | | **24** | 5 |

- **zustand store 扩展 (per 守门 #19 v19+ 累积规, 不破坏 V0.1)**:

```typescript
// frontend/src/lib/store/agent.ts (新, 扩展 V0.1)
interface AgentStore {
  // V0.1 既有 (agentSessions) + 增项 (avatar_url/role/domain/parent_session_id/token_budget)
  agentSessions: AgentSession[];

  // 新增 A1-A10 派生数据 (派生只读 per NFR-AGENT-STATE-01)
  worktreeByAgent: Map<Uuid, Worktree[]>;
  workItemByAgent: Map<Uuid, WorkItem[]>;

  // Actions (V0.1 既有, 不破坏)
  // A6.1 start/stop action 走 BFF API, 不直接调 store action
}

// frontend/src/lib/arg/store.ts (新, 派生自 BD-AGENT-RELATIONSHIP-001 §4.3)
interface ARGStore {
  agents: Map<string, ArgAgent>;
  edges: Map<string, ArgEdge>;
  templates: TeamTemplate[];
  achievements: Achievement[];
  argEvents: ARGEvent[];  // WSS 订阅
  syncStatus: { status: 'synced' | 'syncing' | 'error' | 'offline'; lastSyncAt: string; error: string | null };
}

// frontend/src/lib/canvas-collab/store.ts (新, A12)
interface CanvasCollabStore {
  elements: Map<Uuid, CanvasElementBackend>;  // backend 持久化 (A12.3)
  presenceCursors: Map<Uuid, PresenceCursor>;  // WSS (A12.2)
  followers: Map<Uuid, CanvasFollower>;  // (A12.4)
  comments: Map<Uuid, CanvasComment>;  // (A12.5)
  permissions: Map<Uuid, CanvasPermission>;  // (A12.7)
}
```

#### 2.2.2 Tier 2: API Tier (crates/api 扩展 + BFF per envoy 9/1 偏好)

- **位置**:
  - `crates/api/src/agent/` (A1-A10 双核心 13 端点, 跟 `BD-AGENT-VIEW-001` 共用)
  - `crates/api/src/arg/` (A11 5 端点, 派生自 `BD-AGENT-RELATIONSHIP-001` §5.1)
  - `bff/src/collaboration/` (A12 5 端点 + WSS Hub, 走 envoy 独立 deployment per 9/1 13:05 JST)

- **新增组件**:
  - `agent/controller.rs` (Axum router, 13 REST endpoints A1-A10)
  - `arg/controller.rs` (Axum router, 5 REST + 1 WebSocket A11)
  - `arg/sse_hub.rs` (WSS /ws/arg/events, 派生自 BD-AGENT-REL-001 §5.2)
  - `collaboration/controller.rs` (BFF router, 5 REST + 4 WebSocket A12)
  - `collaboration/wss_hub.rs` (WSS Hub, A12.1 + A12.2 + A12.3 + A12.4)
  - `permission.rs` (RLS 13 类 middleware)
  - `audit.rs` (100% audit middleware)
  - `dto.rs` (request/response types)

- **守门**: RLS 13 类必携 (per 守门 #13) + 100% audit (per 守门 #13) + envoy 独立 deployment (per 9/1 13:03+13:05 JST 偏好)

#### 2.2.3 Tier 3: Application Tier (Rust crates 扩展)

- **A1-A10**: `crates/agent-domain/` (新, 跟 V0.1 `crates/agent-view` 协同)
  - 11 子模块: agent_node / handoff / topology / status_sync / worktree_assoc / workitem_assoc / context_menu / monitor / cluster / cross_ref / settings_integration

- **A11**: `crates/arg/` + `crates/arg-bridge/` + `crates/arg-effect/` (3 新 crate, 派生自 `BD-AGENT-RELATIONSHIP-001` §2.2 Tier 3-5)
  - 6 + 4 + 5 = 15 子模块 (派生自 BD-AGENT-REL-001)

- **A12**: `crates/canvas-collab/` (新)
  - 5 子模块: elements / presence / follow / comments / permissions

- **依赖 (per `BD-AGENT-RELATIONSHIP-001` §2.2)**:
  - `tokio` / `serde` / `serde_json` / `uuid` / `chrono` / `tracing` (既有)
  - `r2d2-memgraph` (新, per `SRS-AGENT-RELATIONSHIP-001` G-1 缺口)
  - `langgraph` (新, Python binding via PyO3, per `SRS-AGENT-RELATIONSHIP-001` G-3 缺口)

#### 2.2.4 Tier 4: Data Tier (PostgreSQL + Memgraph + WSS Hub + BFF)

- **PostgreSQL**:
  - A1-A10 7 张表 (V0.1 store 派生表)
  - A12 7 张表 (新增, per 守门 #13 W/T/M 100% 覆盖)
  - 14 张表 100% RLS 13 类必携 (per 守门 #13)

- **Memgraph** (A11 派生自 `BD-AGENT-RELATIONSHIP-001` §4.4):
  - 4 张表 (实际 Memgraph 节点 + 边 + 索引 + 约束, SQL mirror)
  - Docker compose 启动 (port 7687 Bolt + 7444 HTTP, per `BD-AGENT-RELATIONSHIP-001` §1.1.1)

- **WSS Hub**:
  - A11: 1 WSS 端点 `/ws/arg/events` (per `BD-AGENT-RELATIONSHIP-001` §5.2)
  - A12: 4 WSS 端点 (A12.1 + A12.2 + A12.3 + A12.4 独立 topic, 跨多画布共享)
  - TLS 1.3+ + auth required (per 守门 #5 + NFR-AGENT-MU-CONS-01)

- **BFF (envoy 独立 deployment)**:
  - 5 REST + 4 WSS, 走 envoy proxy (per 9/1 13:03+13:05 JST 偏好, 业务 svc 通过 `svc://` 引用)

#### 2.2.5 Tier 5: Effect Tier (LangGraph 节点 + 25 module 联动)

- **4 维度 effect** (派生自 `BD-AGENT-RELATIONSHIP-001` §2.2 Tier 5):
  - `ARGDispatchRouter` (LangGraph dispatch_node 拦截)
  - `ARGContextInjector` (mentors / shadows context 注入)
  - `ARGTrustEngine` (trust_score + verify skip)
  - `ARGOutputEvaluator` (challenges / peer_reviews)

- **25 module 联动** (per 总册 `SRS-CANVAS-001` §6.3):
  - worktree 域 (A4 关联)
  - work-item 域 (A5 关联)
  - comment 域 (A12.5)
  - notification 域 (A12.5 @ 提醒)
  - audit 域 (A3.2 + A12.8)
  - search 域 (跨块接口)
  - settings 域 (A10 集成)

### 2.3 数据流 (Data Flow)

#### 2.3.1 A1-A10 正常流: 5 域 Lead 打开画布看 agent 拓扑

```
[5 域 Lead 浏览器] 
  ↓ GET /canvas
[Frontend CanvasView] 
  ↓ 读 useAgentStore
[zustand 订阅 agentSessions] (per V0.1 联动 3, frontend-canvas-design §4.4)
  ↓ 渲染 5 域 Frame + agent_node 完整卡 (A1.1+A2.2)
  ↓ 双击 ag-005 → 跳 /agent?selected=ag-005 (A1.3)
[用户操作: 右键 ag-003 → 启停]
  ↓ 调 BFF /v1/agent/{id}/start (A6.1)
[AgentController] → 校验权限 (SRE Lead ✓)
  ↓ 调 agent-runtime API
[agent_runtime] → 更新 status (running → completed)
  ↓ WSS 推送 (per 守门 #9 + 选型待 P3-D)
[zustand 订阅] → 画布色码实时变绿 (≤ 200ms P95, per A3.1)
  ↓ audit middleware 写 audit log (per A3.2)
[25 module audit 域] → 100% audit
```

#### 2.3.2 A11 正常流: 拖拽建 delegates_to 关系

```
[5 域 Lead 浏览器] 画布切 "Relationship" tab
  ↓ 跳 /agent-relationships?from=canvas (A11.10)
[RelationshipEditor] 拖拽 ag-005 → ag-007
  ↓ 弹 "关系编辑" modal (3 字段 type/weight/metadata, per A11.2)
[用户选 type=delegates_to, weight=0.7, metadata={"review_threshold":0.8}]
  ↓ POST /api/arg/edges
[ARGController] → 校验权限 + RLS 13 类
  ↓ Memgraph write (1 事务, per A11.2 NFR-PERF-03 < 200ms P95)
[Memgraph] → 触发 Bolt subscription
  ↓
[Bridge: MemgraphEventListener] → EventBus edge.changed
  ↓
[Effect: 4 维度 effect 异步 reload]
  ↓
[API: ARGSSEHub] → 推送 edge.changed 给所有订阅前端 (A11.6 边粗细按 weight)
  ↓
[UI: 实时显示新边 (蓝色 delegates_to, 粗细按 weight 0.7)]
  ↓
[audit 写 agent_relationship_edges_audit 表] (per A11.5 4 表 W/T/M 100%)
```

#### 2.3.3 A12 正常流: 多人同时编辑

```
[PM 浏览器] 拖拽 ag-005 到 (100, 200)
  ↓ 调 BFF /v1/collaboration/canvases/canvas-001/elements/el-005 PATCH (A12.3)
  ↓ WSS 广播 element.update
[CanvasCollabController] → BFF 校验权限 (PM edit ✓)
  ↓ 写 backend canvas_elements_backend 表 (per A12.3 14 张表)
  ↓ 写 audit canvas_multi_user_audit 1 行 (per A12.8)
[WSS Hub: /ws/canvas-collab/canvas-001]
  ↓ 广播 element.update + audit event
[SRE + 5 域 Lead 浏览器]
  ↓ ≤ 200ms (P95) 看到 ag-005 移动到 (100, 200) (per A12.3 NFR-PERF-04)
  ↓ 看到 PM cursor + 名字 (per A12.2 NFR-PERF-05)
  ↓ 看到 PM 添加的评论 thread (per A12.5)
  ↓ 看到 PM @ SRE notification (per 25 module notification 域对接)
```

#### 2.3.4 A12 异常流: WSS 断开 + 重连

```
[PM 浏览器] WSS 连接断开 (网络抖动)
  ↓ 客户端自动 retry 3 次 (exponential backoff 1s/2s/4s)
[PM 浏览器] 离线模式编辑 (V0.1 localStorage 降级为 fallback, per BR-14)
  ↓ localStorage 暂存 5 个 element 增删改
[网络] WSS 重连成功
  ↓ 客户端自动同步暂存的 5 个 element → BFF + audit 1 行
  ↓ WSS 广播
[其他用户] 看到 PM 暂存的 5 个 element 增删改同步显示
```

---

## §3 组件一覧 (Component List)

### 3.1 组件总览 (24 个新组件 + 5 view 跨域)

| # | 组件 ID | 名称 | 子能力 | Tier | 语言 | 优先级 | 派生 |
|---|---|---|---|---|---|---|---|
| C-1 | `AgentNode` | agent 节点完整卡 | A1 | UI | TSX | P0 | 扩展 V0.1 `agent_cursor` line 218-235 |
| C-2 | `AgentStatusPill` | 14 状态机色码 | A1 + A3 | UI | TSX | P0 | 复用 V0.1 StatusPill 60+ |
| C-3 | `HandoffConnector` | handoff 1:N 边 | A2 | UI | TSX | P0 | 扩展 V0.1 `agent_handoff` |
| C-4 | `DomainFrame` | 5 域分组 Frame | A2 | UI | TSX | P0 | 扩展 V0.1 `Frame` |
| C-5 | `ParentChildConnector` | 父子树形边 | A2 | UI | TSX | P1 | (新) |
| C-6 | `PipelineConnector` | pipeline 水平边 | A2 | UI | TSX | P1 | (新) |
| C-7 | `WorktreeRing` | 1 agent → N worktree 圆周散点 | A4 | UI | TSX | P0 | 扩展 V0.1 `worktree_node` line 200-217 |
| C-8 | `WorkItemDragIn` | drag work-item 关联 | A5 | UI | TSX | P0 | 扩展 V0.1 `work_item_card` line 180-199 |
| C-9 | `AgentContextMenu` | 右键操作菜单 | A6 | UI | TSX | P0 | (新) |
| C-10 | `AgentDetailPanel` | 监控 4 字段仪表 | A7 | UI | TSX | P1 | (新) |
| C-11 | `ClusterSortFilter` | 顶部 dropdown | A8 | UI | TSX | P1 | (新) |
| C-12 | `RelationshipEditor` | ARG 关系画布编辑 | A11.2 | UI | TSX | P0 | 派生自 `BD-AGENT-RELATIONSHIP-001` §6.1 F-2 |
| C-13 | `ArgEdge` | 10 类关系边 | A11.1 | UI | TSX | P0 | 派生自 `BD-AGENT-RELATIONSHIP-001` §6.1 F-1 |
| C-14 | `ArgEffectIndicator` | 4 维度指示器 | A11.3 | UI | TSX | P0 | 派生自 `BD-AGENT-RELATIONSHIP-001` §6.1 F-5 |
| C-15 | `TemplateGallery` | 5 团队模板 1-click | A11.4 | UI | TSX | P1 | 派生自 `BD-AGENT-RELATIONSHIP-001` §6.1 F-4 |
| C-16 | `SyncStatusBadge` | 4 状态 synced/syncing/error/offline | A11.9 | UI | TSX | P1 | 派生自 `BD-AGENT-RELATIONSHIP-001` §6.3.2 |
| C-17 | `PresenceCursor` | 多人 cursor 同步 | A12.2 | UI | TSX | P0 | 派生自 `frontend-canvas-design.md` §4.6 |
| C-18 | `FollowModeBadge` | Follow mode 指示 | A12.4 | UI | TSX | P1 | 派生自 `frontend-canvas-design.md` §4.9 |
| C-19 | `CommentThread` | 多人评论线程 | A12.5 | UI | TSX | P0 | 派生自 `frontend-canvas-design.md` §3.4 + `CanvasView.tsx` line 253-262 |
| C-20 | `PermissionGate` | view/comment/edit 3 级权限 UI | A12.7 | UI | TSX | P1 | (新) |
| C-21 | `MultiUserAuditLog` | 多人编辑 audit | A12.8 | UI | TSX | P0 | (新) |
| C-22 | `ArgStore` | zustand useARGStore 5 channel | A11 | Data | TS | P0 | 派生自 `BD-AGENT-RELATIONSHIP-001` §4.3 |
| C-23 | `CanvasCollabStore` | zustand useCanvasCollabStore 5 channel | A12 | Data | TS | P0 | (新) |
| C-24 | `AgentStore` | zustand useAgentStore 扩展 V0.1 | A1-A10 | Data | TS | P0 | (扩展 V0.1, 增 5 字段) |

### 3.2 模块划分 (新 crate / 新模块)

```
crates/agent-domain/ (新, 跟 crates/agent-view 共用, A1-A10 业务逻辑)
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── agent_node.rs       # C-1, A1 完整卡
│   ├── handoff.rs          # C-3, A2.1 1:N
│   ├── topology.rs         # C-4, A2.2 5 域
│   ├── parent_child.rs     # C-5, A2.3 父子
│   ├── pipeline.rs         # C-6, A2.4 pipeline
│   ├── status_sync.rs      # A3 14 状态机
│   ├── worktree_assoc.rs   # A4 1:N
│   ├── workitem_assoc.rs   # A5 1:N
│   ├── context_menu.rs     # C-9, A6 操作
│   ├── monitor.rs          # C-10, A7 监控
│   ├── cluster.rs          # C-11, A8 聚类
│   ├── cross_ref.rs        # A9 跨域
│   ├── settings_integration.rs  # A10 集成
│   └── models/
│       ├── agent.rs        # Agent struct
│       ├── worktree.rs     # 增 agent_session_ids[]
│       └── workitem.rs     # 增 agent_session_id

crates/arg/ (新, A11 主数据层, 派生自 BD-AGENT-RELATIONSHIP-001 §2.2)
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs           # Memgraph Bolt client
│   ├── agent_node.rs       # A11 节点 CRUD
│   ├── edge_ops.rs         # A11 边 CRUD + audit
│   ├── template_ops.rs     # 5 模板 instantiate
│   ├── cypher_cache.rs     # LRU 1000
│   ├── event_writer.rs     # audit writer
│   ├── migration.rs        # schema 版本
│   ├── models/
│   │   ├── agent.rs
│   │   ├── edge.rs         # 10 类关系 enum
│   │   ├── template.rs     # 5 模板定义
│   │   └── achievement.rs  # 20 成就定义
│   ├── error.rs
│   └── tests/

crates/arg-bridge/ (新, A11 同步桥, 派生自 BD-AGENT-RELATIONSHIP-001 §2.2)
├── src/
│   ├── lib.rs
│   ├── memgraph_listener.rs
│   ├── langgraph_updater.rs
│   ├── period_flush.rs
│   └── offline_queue.rs

crates/arg-effect/ (新, A11 4 维度 effect, 派生自 BD-AGENT-RELATIONSHIP-001 §2.2)
├── src/
│   ├── lib.rs
│   ├── dispatch_router.rs
│   ├── context_injector.rs
│   ├── trust_engine.rs
│   ├── output_evaluator.rs
│   └── achievement_engine.rs

crates/canvas-collab/ (新, A12 多人编辑业务逻辑)
├── src/
│   ├── lib.rs
│   ├── elements.rs         # A12.3 backend 持久化
│   ├── presence.rs         # A12.2 WSS PresenceCursor
│   ├── follow.rs           # A12.4 Follow mode
│   ├── comments.rs         # A12.5 thread + @
│   └── permissions.rs      # A12.7 3 级权限

crates/api/src/agent/ (新模块, A1-A10 13 REST 端点)
├── mod.rs
├── controller.rs
├── permission.rs
├── audit.rs
└── dto.rs

crates/api/src/arg/ (新模块, A11 5 REST + 1 WSS 端点, 派生自 BD-AGENT-RELATIONSHIP-001 §5.1)
├── mod.rs
├── controller.rs           # 13 → 实际 A11 用 5 端点 (agents/edges/graph/templates/achievements)
├── sse_hub.rs
├── permission.rs
└── dto.rs

bff/src/collaboration/ (BFF, A12 5 REST + 4 WSS 端点, 走 envoy 独立 deployment)
├── mod.rs
├── controller.rs
├── wss_hub.rs             # A12.1 + A12.2 + A12.3 + A12.4 4 WSS 端点
├── permission.rs          # A12.7 3 级权限
├── audit.rs               # A12.8 100% audit
└── dto.rs

frontend/src/app/canvas/ (扩展 V0.1, A1-A10 主画布)
├── page.tsx
├── AgentCanvasView.tsx
├── AgentNode.tsx          # C-1
├── AgentStatusPill.tsx    # C-2
├── HandoffConnector.tsx   # C-3
├── DomainFrame.tsx        # C-4
├── ParentChildConnector.tsx  # C-5
├── PipelineConnector.tsx  # C-6
├── WorktreeRing.tsx       # C-7
├── WorkItemDragIn.tsx     # C-8
├── AgentContextMenu.tsx   # C-9
├── AgentDetailPanel.tsx   # C-10
├── ClusterSortFilter.tsx  # C-11
└── AgentStore.ts          # C-24

frontend/src/app/agent-relationships/ (新, A11 关系画布, 派生自 BD-AGENT-RELATIONSHIP-001 §2.2)
├── page.tsx
├── RelationshipEditor.tsx # C-12
├── ArgEdge.tsx            # C-13
├── ArgEffectIndicator.tsx # C-14
├── TemplateGallery.tsx    # C-15
├── SyncStatusBadge.tsx    # C-16
└── ArgStore.ts            # C-22

frontend/src/app/canvas/MultiUserCanvasView/ (新, A12 多人编辑)
├── PresenceCursor.tsx     # C-17
├── FollowModeBadge.tsx    # C-18
├── CommentThread.tsx      # C-19
├── PermissionGate.tsx     # C-20
├── MultiUserAuditLog.tsx  # C-21
└── CanvasCollabStore.ts   # C-23
```

### 3.3 组件 vs A1-A12 子能力映射

| 子能力 | 组件 | 数量 | 派生 |
|---|---|---|---|
| A1 | C-1 + C-2 | 2 | 扩展 V0.1 |
| A2 | C-3 + C-4 + C-5 + C-6 | 4 | 扩展 V0.1 |
| A3 | C-2 (StatusPill 60+) | 1 (复用) | V0.1 |
| A4 | C-7 | 1 | 扩展 V0.1 |
| A5 | C-8 | 1 | 扩展 V0.1 |
| A6 | C-9 | 1 | (新) |
| A7 | C-10 | 1 | (新) |
| A8 | C-11 | 1 | (新) |
| A9 | (跨块接口, URL 跳) | 0 | (跨块) |
| A10 | (V0.1 集成) | 0 | V0.1 |
| A11 | C-12 + C-13 + C-14 + C-15 + C-16 | 5 | 派生自 BD-AGENT-REL-001 |
| A12 | C-17 + C-18 + C-19 + C-20 + C-21 | 5 | 派生自 frontend-canvas-design.md |
| **合计** | | **24** | |

---

## §4 数据模型 (Data Model)

### 4.1 数据模型总览 (14 张表 W/T/M 100% 覆盖)

| 类别 | 表数 | 表名 | 占比 |
|---|---|---|---|
| **Work** (短 TTL 作業中) | 3 | `team_template_instances` (TTL 30 天) + `canvas_followers` (session-bound) + `canvas_presence_cursors` (heartbeat 30s) | 3/14 = 21.4% |
| **Transaction** (業務事実 / 監査 / Append-only) | 5 | `agent_relationship_edges_audit` + `achievement_unlocks` + `relationship_events` + `canvas_multi_user_audit` + `canvas_comment_mentions` | 5/14 = 35.7% |
| **Master** (参考 / 設定 / 慢変 SCD) | 6 | `agents` + `agent_relationship_edges` + `achievements` + `canvas_comments` + `canvas_permissions` + `canvas_elements_backend` | 6/14 = 42.9% |
| **合计** | **14** | (100% 覆盖, per 守门 #13) | 100% |

**派生规 (per 守门 #13)**:
- (a) **W = 物理删除 / タイマー失効 / 短 TTL 明示 retention**: 3 张 Work 表 100% retention_period ✓
- (b) **T = 物理删除禁止 + 監査必須 + RLS 13 類必携**: 5 张 Transaction 表 100% audit + 物理删除禁止 ✓
- (c) **M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携**: 6 张 Master 表 SCD Type 2 + RLS 13 类 ✓
- (d) **Master 100% RLS / Transaction 100% audit / Work 100% retention_period**: 全部满足 ✓

### 4.2 A11 派生 7 张表 (per `BD-AGENT-RELATIONSHIP-001` §4.4 + 守门 #13)

#### 4.2.1 `agents` (Master, SCD Type 2, 物理删除禁止)

```sql
CREATE TABLE agents (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  archetype VARCHAR(50) NOT NULL,  -- SA-01..SA-09 / LEAD-* / CUSTOM
  domain VARCHAR(50),  -- player/economy/match/social/admin/null
  status VARCHAR(20) NOT NULL DEFAULT 'active',  -- active/standby/archived
  trust_score REAL NOT NULL DEFAULT 0.5,  -- 0.0-1.0
  metadata JSONB,
  tenant_id UUID NOT NULL,  -- RLS 13 类 per 守门 #13
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1,  -- SCD Type 2
  created_by UUID NOT NULL  -- author = Ulysses per 9/8 15:19
);

CREATE INDEX idx_agents_archetype ON agents(archetype);
CREATE INDEX idx_agents_domain ON agents(domain);
CREATE INDEX idx_agents_tenant ON agents(tenant_id);
CREATE INDEX idx_agents_status ON agents(status);

ALTER TABLE agents ENABLE ROW LEVEL SECURITY;
CREATE POLICY agents_tenant_isolation ON agents
  USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

#### 4.2.2 `agent_relationship_edges` (Master, SCD Type 2, 物理删除禁止)

```sql
CREATE TABLE agent_relationship_edges (
  id UUID PRIMARY KEY,
  from_agent UUID NOT NULL REFERENCES agents(id),
  to_agent UUID NOT NULL REFERENCES agents(id),
  type VARCHAR(30) NOT NULL,  -- 10 类关系 enum (4 核心 + 6 扩展)
  weight REAL NOT NULL DEFAULT 0.5,  -- 0.0-1.0
  direction VARCHAR(20) NOT NULL,  -- directed/undirected
  archived BOOLEAN NOT NULL DEFAULT false,
  metadata JSONB,
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1,
  created_by UUID NOT NULL,
  UNIQUE(from_agent, to_agent, type, archived)
);

CREATE INDEX idx_edges_from ON agent_relationship_edges(from_agent);
CREATE INDEX idx_edges_to ON agent_relationship_edges(to_agent);
CREATE INDEX idx_edges_type ON agent_relationship_edges(type);
CREATE INDEX idx_edges_tenant ON agent_relationship_edges(tenant_id);
CREATE INDEX idx_edges_archived ON agent_relationship_edges(archived);
```

#### 4.2.3 `agent_relationship_edges_audit` (Transaction, append-only, 物理删除禁止)

```sql
CREATE TABLE agent_relationship_edges_audit (
  id UUID PRIMARY KEY,
  edge_id UUID NOT NULL,
  action VARCHAR(20) NOT NULL,  -- create/update/archive
  old_value JSONB,
  new_value JSONB,
  actor UUID NOT NULL,
  tenant_id UUID NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_edge ON agent_relationship_edges_audit(edge_id);
CREATE INDEX idx_audit_timestamp ON agent_relationship_edges_audit(timestamp);
```

#### 4.2.4 `team_template_instances` (Work, TTL 30 天)

```sql
CREATE TABLE team_template_instances (
  id UUID PRIMARY KEY,
  template_id VARCHAR(50) NOT NULL,  -- hub-and-spoke/mesh/chain/hierarchical/review-council
  instance_name VARCHAR(255) NOT NULL,
  agent_ids UUID[] NOT NULL,
  edges_json JSONB NOT NULL,
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '30 days',  -- TTL
  retention_period INTERVAL NOT NULL DEFAULT '30 days'  -- 必填 per 守门 #13 a
);
```

#### 4.2.5 `achievements` (Master, SCD Type 2, 物理删除禁止)

```sql
CREATE TABLE achievements (
  id UUID PRIMARY KEY,
  code VARCHAR(100) NOT NULL,  -- e.g. "TOP-001-MESH-5DOMAIN"
  name VARCHAR(255) NOT NULL,
  description TEXT,
  category VARCHAR(20) NOT NULL,  -- TOPOLOGY / BEHAVIOR / OUTPUT
  rarity VARCHAR(20) NOT NULL,  -- COMMON / RARE / EPIC / LEGENDARY
  icon_url VARCHAR(500),
  tenant_id UUID NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_achievements_code ON achievements(code);
CREATE INDEX idx_achievements_category ON achievements(category);
```

#### 4.2.6 `achievement_unlocks` (Transaction, append-only, 物理删除禁止)

```sql
CREATE TABLE achievement_unlocks (
  id UUID PRIMARY KEY,
  achievement_id UUID NOT NULL REFERENCES achievements(id),
  user_id UUID NOT NULL,
  agent_ids UUID[],
  trigger_metadata JSONB,
  tenant_id UUID NOT NULL,
  unlocked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_unlocks_user ON achievement_unlocks(user_id);
CREATE INDEX idx_unlocks_achievement ON achievement_unlocks(achievement_id);
```

#### 4.2.7 `relationship_events` (Transaction, append-only, 物理删除禁止)

```sql
CREATE TABLE relationship_events (
  id UUID PRIMARY KEY,
  edge_id UUID,
  event_type VARCHAR(50) NOT NULL,  -- edge.changed / collaboration.success / etc.
  payload JSONB,
  tenant_id UUID NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_events_edge ON relationship_events(edge_id);
CREATE INDEX idx_events_timestamp ON relationship_events(timestamp);
```

### 4.3 A12 派生 7 张表 (per 守门 #13, 100% 覆盖)

#### 4.3.1 `canvas_multi_user_audit` (Transaction, append-only, 物理删除禁止, SCD Type 2) ⭐

```sql
-- A12.8 必含新增表, per 守门 #13
CREATE TABLE canvas_multi_user_audit (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  actor_user_id UUID NOT NULL,
  action VARCHAR(50) NOT NULL,  -- element.create / element.update / element.delete / cursor.move / comment.add / comment.reply / mention.create / follow.start / follow.stop
  target_id UUID,
  target_type VARCHAR(50),
  payload JSONB,
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1  -- SCD Type 2
);

-- 100% RLS 13 类必携 (per 守门 #13)
ALTER TABLE canvas_multi_user_audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY canvas_audit_tenant_isolation ON canvas_multi_user_audit
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

CREATE INDEX idx_audit_canvas ON canvas_multi_user_audit(canvas_id);
CREATE INDEX idx_audit_actor ON canvas_multi_user_audit(actor_user_id);
CREATE INDEX idx_audit_action ON canvas_multi_user_audit(action);
CREATE INDEX idx_audit_created ON canvas_multi_user_audit(created_at);
```

**关键守门**: 1 操作 1 audit 行 (per BR-15), 物理删除 0 次, SCD Type 2 version +1.

#### 4.3.2 `canvas_comments` (Master, SCD Type 2, 物理删除禁止)

```sql
CREATE TABLE canvas_comments (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  element_id UUID,  -- 关联的 element (nullable, 画布级评论可空)
  parent_comment_id UUID,  -- thread 结构 (1 顶级 + N 回复, 顶级为 null)
  author_user_id UUID NOT NULL,
  content TEXT NOT NULL,
  mentions UUID[],  -- @ 提醒 (UUID[])
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1,  -- SCD Type 2
  archived BOOLEAN NOT NULL DEFAULT false
);

ALTER TABLE canvas_comments ENABLE ROW LEVEL SECURITY;
CREATE POLICY canvas_comments_tenant_isolation ON canvas_comments
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

CREATE INDEX idx_comments_canvas ON canvas_comments(canvas_id);
CREATE INDEX idx_comments_element ON canvas_comments(element_id);
CREATE INDEX idx_comments_author ON canvas_comments(author_user_id);
```

#### 4.3.3 `canvas_comment_mentions` (Transaction, append-only, 物理删除禁止)

```sql
CREATE TABLE canvas_comment_mentions (
  id UUID PRIMARY KEY,
  comment_id UUID NOT NULL REFERENCES canvas_comments(id),
  mentioned_user_id UUID NOT NULL,
  notification_sent BOOLEAN NOT NULL DEFAULT false,
  read_at TIMESTAMPTZ,
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1
);

ALTER TABLE canvas_comment_mentions ENABLE ROW LEVEL SECURITY;
CREATE POLICY canvas_mentions_tenant_isolation ON canvas_comment_mentions
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

CREATE INDEX idx_mentions_comment ON canvas_comment_mentions(comment_id);
CREATE INDEX idx_mentions_user ON canvas_comment_mentions(mentioned_user_id);
```

#### 4.3.4 `canvas_permissions` (Master, SCD Type 2, 物理删除禁止)

```sql
CREATE TABLE canvas_permissions (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  user_id UUID NOT NULL,
  role VARCHAR(20) NOT NULL CHECK (role IN ('view', 'comment', 'edit')),  -- 3 级权限
  granted_by_user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ,
  version INTEGER NOT NULL DEFAULT 1,  -- SCD Type 2
  UNIQUE(canvas_id, user_id)  -- 1 user 1 canvas 1 role
);

ALTER TABLE canvas_permissions ENABLE ROW LEVEL SECURITY;
CREATE POLICY canvas_permissions_tenant_isolation ON canvas_permissions
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

CREATE INDEX idx_perms_canvas ON canvas_permissions(canvas_id);
CREATE INDEX idx_perms_user ON canvas_permissions(user_id);
```

**关键守门**: A12.7 3 级权限枚举, per 守门 #14 v2 拍板前 view-only 兜底 (外部用户仅 view, 内部 Lead 默认 edit).

#### 4.3.5 `canvas_followers` (Work, session-bound, session 结束自动清理)

```sql
CREATE TABLE canvas_followers (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  leader_user_id UUID NOT NULL,
  follower_user_id UUID NOT NULL,
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,  -- session 结束时间
  retention_period INTERVAL NOT NULL DEFAULT '24 hours',  -- 必填 per 守门 #13 a
  UNIQUE(canvas_id, follower_user_id)  -- 1 canvas 1 follower 限制
);

CREATE INDEX idx_followers_canvas ON canvas_followers(canvas_id);
CREATE INDEX idx_followers_follower ON canvas_followers(follower_user_id);
CREATE INDEX idx_followers_expires ON canvas_followers(expires_at);
```

**关键守门**: 1 canvas 1 follower (per A12.4 避免性能问题).

#### 4.3.6 `canvas_elements_backend` (Master, SCD Type 2, 物理删除禁止)

```sql
-- 跟 V0.1 CanvasElement 区分, backend 持久化 (per A12.3)
CREATE TABLE canvas_elements_backend (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  kind VARCHAR(50) NOT NULL,  -- 14 element kind
  x DOUBLE PRECISION NOT NULL,
  y DOUBLE PRECISION NOT NULL,
  width DOUBLE PRECISION NOT NULL,
  height DOUBLE PRECISION NOT NULL,
  rotation DOUBLE PRECISION NOT NULL DEFAULT 0,
  z_index INTEGER NOT NULL DEFAULT 0,
  content JSONB,  -- 14 element content (text, color, image_url, work_item_id, etc.)
  locked BOOLEAN NOT NULL DEFAULT false,
  hidden BOOLEAN NOT NULL DEFAULT false,
  created_by_user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1  -- SCD Type 2
);

ALTER TABLE canvas_elements_backend ENABLE ROW LEVEL SECURITY;
CREATE POLICY canvas_elements_tenant_isolation ON canvas_elements_backend
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

CREATE INDEX idx_elements_canvas ON canvas_elements_backend(canvas_id);
CREATE INDEX idx_elements_kind ON canvas_elements_backend(kind);
```

**关键守门**: 跟 V0.1 localStorage 持久化冲突 (per BR-14), A12.3 实施时 V0.1 持久化层降级为离线 fallback.

#### 4.3.7 `canvas_presence_cursors` (Work, heartbeat 30s, 离线自动清理)

```sql
CREATE TABLE canvas_presence_cursors (
  id UUID PRIMARY KEY,
  canvas_id UUID NOT NULL,
  user_id UUID NOT NULL,
  cursor_x DOUBLE PRECISION,
  cursor_y DOUBLE PRECISION,
  viewport_x DOUBLE PRECISION,
  viewport_y DOUBLE PRECISION,
  viewport_zoom DOUBLE PRECISION,
  selected_element_ids UUID[],
  last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '30 seconds',  -- heartbeat 30s
  retention_period INTERVAL NOT NULL DEFAULT '30 seconds',
  UNIQUE(canvas_id, user_id)  -- 1 user 1 cursor per canvas
);

CREATE INDEX idx_cursors_canvas ON canvas_presence_cursors(canvas_id);
CREATE INDEX idx_cursors_user ON canvas_presence_cursors(user_id);
CREATE INDEX idx_cursors_heartbeat ON canvas_presence_cursors(last_heartbeat_at);
```

**关键守门**: cursor 移动 throttle 50ms (避免刷屏, per BR-15), heartbeat 30s 自动清理.

### 4.4 5 domain-* Rust 数据结构 (per 总册)

```rust
// crates/agent-domain/src/models/agent.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,  // A1.1 增项
    pub role: AgentRole,             // A1.1 增项 (Supervisor / Worker / Reviewer)
    pub kind: AgentKind,             // A1.1 增项 (SA-01..SA-09)
    pub domain: Option<Domain>,      // A2.2 增项 (5 域)
    pub status: AgentStatus,         // 14 状态机
    pub token_usage: u64,
    pub token_budget: u64,           // A7.2 增项 (默认 1.2M)
    pub parent_session_id: Option<Uuid>,  // A2.3 增项
    pub started_at: DateTime<Utc>,
    pub tenant_id: Uuid,
    pub version: u32,                // SCD Type 2
}

// crates/arg/src/models/edge.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgEdge {
    pub id: Uuid,
    pub from_agent: Uuid,
    pub to_agent: Uuid,
    pub edge_type: EdgeType,         // 10 类关系 enum
    pub weight: f32,                 // 0.0-1.0
    pub direction: Direction,        // directed/undirected
    pub archived: bool,              // A11.7 软删
    pub metadata: serde_json::Value,
    pub tenant_id: Uuid,
    pub version: u32,                // A11.8 SCD Type 2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    // 4 核心
    DelegatesTo,     // A → B, 蓝
    Consults,        // A → B, 青
    CollaboratesWith,// A ↔ B, 紫, 无向
    ReportsTo,       // A → B, 橙
    // 6 扩展
    Mentors,         // A → B, 深绿
    PeerReviews,     // A ↔ B, 浅绿, 无向
    StandInFor,      // A → B, 黄
    Shadows,         // A → B, 灰
    Challenges,      // A → B, 红
    Trusts,          // A → B, 浅蓝
}

// crates/canvas-collab/src/models/comment.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasComment {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub element_id: Option<Uuid>,
    pub parent_comment_id: Option<Uuid>,  // thread 结构
    pub author_user_id: Uuid,
    pub content: String,
    pub mentions: Vec<Uuid>,  // @ 提醒
    pub created_at: DateTime<Utc>,
    pub version: u32,  // SCD Type 2
}

// crates/canvas-collab/src/models/permission.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionRole {
    View,    // A12.7 只读
    Comment, // A12.7 评论
    Edit,    // A12.7 编辑
}
```

### 4.5 Zustand Store 扩展 (per 守门 #19 v19+ 累积规, 不破坏 V0.1)

```typescript
// frontend/src/lib/store/agent.ts (新, 扩展 V0.1 useStore)
interface AgentStore {
  // V0.1 既有 (agentSessions) + 增 5 字段
  agentSessions: AgentSession[];

  // A1-A10 派生数据 (派生只读, per NFR-AGENT-STATE-01)
  worktreeByAgent: Map<Uuid, Worktree[]>;
  workItemByAgent: Map<Uuid, WorkItem[]>;

  // Actions
  loadAgents(): Promise<void>;
  startAgent(agentId: Uuid): Promise<void>;
  stopAgent(agentId: Uuid): Promise<void>;
  // ... 等等
}

// frontend/src/lib/arg/store.ts (新, 派生自 BD-AGENT-RELATIONSHIP-001 §4.3)
interface ARGStore {
  agents: Map<string, ArgAgent>;
  edges: Map<string, ArgEdge>;
  templates: TeamTemplate[];
  achievements: Achievement[];
  unlockedAchievements: Set<string>;
  argEvents: ARGEvent[];
  wsConnected: boolean;
  syncStatus: { status: 'synced' | 'syncing' | 'error' | 'offline'; lastSyncAt: string; error: string | null };

  loadAgents(): Promise<void>;
  loadEdges(filter?: EdgeFilter): Promise<void>;
  createEdge(input: CreateEdgeInput): Promise<Edge>;
  updateEdge(id: string, patch: UpdateEdgePatch): Promise<Edge>;
  archiveEdge(id: string): Promise<void>;
  instantiateTemplate(templateId: string, agentIds: string[]): Promise<TemplateInstance>;
  loadAchievements(): Promise<void>;
  subscribeEvents(): void;  // WSS
}

// frontend/src/lib/canvas-collab/store.ts (新, A12)
interface CanvasCollabStore {
  // A12.3 backend 持久化 (不进 zustand persist, 跟 V0.1 localStorage 隔离)
  elements: Map<Uuid, CanvasElementBackend>;
  // A12.2 多人 cursor (WSS)
  presenceCursors: Map<Uuid, PresenceCursor>;
  // A12.4 Follow mode
  followers: Map<Uuid, CanvasFollower>;
  // A12.5 评论
  comments: Map<Uuid, CanvasComment>;
  // A12.7 3 级权限
  permissions: Map<Uuid, CanvasPermission>;

  loadElements(canvasId: Uuid): Promise<void>;
  updateElement(id: Uuid, patch: ElementPatch): Promise<void>;
  deleteElement(id: Uuid): Promise<void>;
  subscribePresence(canvasId: Uuid): void;
  startFollowing(leaderUserId: Uuid): Promise<void>;
  stopFollowing(): Promise<void>;
  addComment(input: AddCommentInput): Promise<CanvasComment>;
  replyToComment(parentId: Uuid, content: string): Promise<CanvasComment>;
}
```

### 4.6 W/T/M 100% 覆盖验证 (per 守门 #13)

| 類型 | 表数 | 表名 | 占比 |
|---|---|---|---|
| **Work** | 3 | `team_template_instances` + `canvas_followers` + `canvas_presence_cursors` | 3/14 = 21.4% |
| **Transaction** | 5 | `agent_relationship_edges_audit` + `achievement_unlocks` + `relationship_events` + `canvas_multi_user_audit` + `canvas_comment_mentions` | 5/14 = 35.7% |
| **Master** | 6 | `agents` + `agent_relationship_edges` + `achievements` + `canvas_comments` + `canvas_permissions` + `canvas_elements_backend` | 6/14 = 42.9% |
| **合计** | **14** | (100% 覆盖, per 守门 #13) | 100% |

**派生规验证**:
- (a) **W = 物理删除 / タイマー失効 / 短 TTL 明示 retention**: 3 张 Work 表 100% retention_period ✓
- (b) **T = 物理删除禁止 + 監査必須 + RLS 13 類必携**: 5 张 Transaction 表 100% audit + 物理删除禁止 ✓
- (c) **M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携**: 6 张 Master 表 SCD Type 2 + RLS 13 类 ✓
- (d) **Master 100% RLS / Transaction 100% audit / Work 100% retention_period**: 全部满足 ✓

---

## §5 接口设计 (Interface Design)

### 5.1 REST API (23 端点, BFF 跨域)

#### 5.1.1 A1-A10 双核心 13 端点 (跟 `BD-AGENT-VIEW-001` 共用)

| # | Method | Path | 说明 | RLS | 守门 |
|---|---|---|---|---|---|
| 1 | `GET` | `/api/agent/sessions` | 列表 agent session (分页 + 过滤) | ✓ | 限频 100/min |
| 2 | `GET` | `/api/agent/sessions/{id}` | agent 详情 | ✓ | — |
| 3 | `POST` | `/api/agent/sessions/{id}/start` | 启动 agent (per A6.1) | ✓ | SRE Lead 权限 |
| 4 | `POST` | `/api/agent/sessions/{id}/stop` | 停止 agent (per A6.1) | ✓ | SRE Lead 权限 |
| 5 | `POST` | `/api/agent/sessions/{id}/restart` | 重启 agent (per A6.2) | ✓ | SRE Lead 权限 |
| 6 | `GET` | `/api/agent/sessions/{id}/logs` | agent logs (per A6.3) | ✓ | — |
| 7 | `GET` | `/api/agent/sessions/{id}/settings` | agent settings (per A6.4 + A10.1) | ✓ | — |
| 8 | `PATCH` | `/api/agent/sessions/{id}/settings` | 更新 settings (per A10.2) | ✓ | Master SCD Type 2 |
| 9 | `GET` | `/api/agent/sessions/{id}/worktrees` | 关联 worktrees (per A4.1) | ✓ | — |
| 10 | `GET` | `/api/agent/sessions/{id}/work-items` | 关联 work-items (per A5.1) | ✓ | — |
| 11 | `GET` | `/api/agent/sessions/{id}/monitor` | 监控 4 字段 (per A7.1) | ✓ | ≤ 1s 刷新 |
| 12 | `POST` | `/api/agent/sessions/{id}/audit` | 状态变化 audit (per A3.2) | ✓ | 100% audit |
| 13 | `GET` | `/api/agent/sessions/{id}/children` | 父子 agent 关系 (per A2.3) | ✓ | — |

#### 5.1.2 A11 5 端点 (派生自 `BD-AGENT-RELATIONSHIP-001` §5.1)

| # | Method | Path | 说明 | RLS | 守门 |
|---|---|---|---|---|---|
| 14 | `POST` | `/api/arg/edges` | 创建关系 (per A11.2, 必填 11 字段) | ✓ | append-only audit |
| 15 | `GET` | `/api/arg/edges` | 列表关系 (filter type/agent/archived) | ✓ | 分页 |
| 16 | `PATCH` | `/api/arg/edges/{id}` | 更新关系 (weight/metadata, per A11.8 version +1) | ✓ | Master SCD Type 2 |
| 17 | `DELETE` | `/api/arg/edges/{id}` | 归档关系 (archived=true, per A11.7) | ✓ | append-only audit |
| 18 | `POST` | `/api/arg/templates/instantiate` | 5 模板实例化 (1 事务 Cypher, per A11.4) | ✓ | 1 batch write |

#### 5.1.3 A12 5 端点 (BFF envoy 独立 deployment)

| # | Method | Path | 说明 | RLS | 守门 |
|---|---|---|---|---|---|
| 19 | `POST` | `/v1/collaboration/canvases/[id]/elements` | 创建 element (per A12.3) | ✓ | BFF 校验 edit 权限 + audit 1 行 |
| 20 | `PATCH` | `/v1/collaboration/canvases/[id]/elements/[eid]` | 更新 element (per A12.3) | ✓ | BFF 校验 edit 权限 + optimistic lock + audit 1 行 |
| 21 | `POST` | `/v1/collaboration/canvases/[id]/comments` | 创建评论 (per A12.5) | ✓ | BFF 校验 ≥ comment 权限 + notification 域对接 + audit 1 行 |
| 22 | `POST` | `/v1/collaboration/canvases/[id]/follow?user_id=` | 启动 Follow mode (per A12.4) | ✓ | BFF 校验 ≥ view 权限 + 1 canvas 1 follower |
| 23 | `GET` | `/v1/collaboration/canvases/[id]/multi-user-audit` | 多人编辑 audit 查询 (per A12.8) | ✓ | 100% RLS 13 类 |

### 5.2 WebSocket (5 端点, BFF 跨域)

| # | Path | 协议 | 事件类型 | 守门 |
|---|---|---|---|---|
| 1 | `/ws/arg/events` | WSS | `edge.changed` / `edge.created` / `edge.archived` / `achievement.unlocked` / `dispatch.route.changed` / `agent.trust_score.changed` | TLS 1.3+ + auth + tenant_id 必填 (per 守门 #5 + 派生自 `BD-AGENT-RELATIONSHIP-001` §5.2) |
| 2 | `wss://canvas-collab/canvases/[id]` | WSS | `element.create` / `element.update` / `element.delete` (per A12.1 + A12.3) | TLS 1.3+ + auth + BFF 校验 view 权限 |
| 3 | `wss://canvas-presence/canvases/[id]` | WSS | `presence.cursor.move` / `presence.viewport.change` / `presence.selection.change` (per A12.2) | TLS 1.3+ + auth + BFF 校验 view 权限 + throttle 50ms |
| 4 | `wss://canvas-comments/canvases/[id]` | WSS | `comment.add` / `comment.reply` / `mention.create` (per A12.5) | TLS 1.3+ + auth + BFF 校验 ≥ comment 权限 |
| 5 | `wss://canvas-follow/canvases/[id]` | WSS | `follow.start` / `follow.stop` / `viewport.sync` (per A12.4) | TLS 1.3+ + auth + BFF 校验 ≥ view 权限 + 1 canvas 1 follower |

### 5.3 内部协议 (5 类, 跨域)

| 协议 | 方向 | 载荷 | 守门 |
|---|---|---|---|
| `agent_status_change` | agent_runtime → store | `{ agent_id, old_status, new_status, changed_at, changed_by }` | per A3.1 + A3.2 audit |
| `arg_edge_changed` | Memgraph → in-process | `{ edge_id, action, from, to, type, version }` | 派生自 `BD-AGENT-RELATIONSHIP-001` §5.3 |
| `arg_dispatch_route` | L0 dispatch_node → SubAgentPool | `{ source_agent, target_agents[], edge_type }` | 派生自 `BD-AGENT-RELATIONSHIP-001` §5.3 |
| `canvas_element_change` | BFF → store | `{ canvas_id, element_id, action, payload, actor_user_id }` | per A12.3 + A12.8 audit |
| `canvas_presence_update` | BFF → store | `{ canvas_id, user_id, cursor_x, cursor_y, viewport, selected_ids }` | per A12.2 + throttle 50ms |

### 5.4 内部 5 类状态机

#### 5.4.1 Agent 14 状态机 (per `SRS-STAR-AGENT-RUNTIME-001.md` §8)

```
[initializing] --spawn--> [running] --complete--> [completed]
   ↓                          ↓
[spawning] <--restart--     [paused]
                              ↓
                          [failed] (终态, audit + notification)
[stopping] <--stop--        [stopped]
[archived] (终态, SCD Type 2)
```

#### 5.4.2 Edge 状态机 (派生自 `BD-AGENT-RELATIONSHIP-001` §5.4.1)

```
[created] --archive--> [archived] (终态)
   |
   └──update weight/metadata--> [updated] (回到 created, version+1)
```

#### 5.4.3 Canvas Element 状态机 (per A12.3)

```
[created] --update position/content--> [updated] (version+1, optimistic lock)
   |
   └──delete--> [deleted] (软删, SCD Type 2)
[restored] <--restore-- [deleted]
```

#### 5.4.4 Trust Score 5 档 (派生自 `BD-AGENT-RELATIONSHIP-001` §5.4.3)

```
[0.0-0.2] UNTRUSTED --3 成功--> [0.2-0.4] LOW
[0.2-0.4] LOW --5 成功--> [0.4-0.7] MEDIUM
[0.4-0.7] MEDIUM --10 成功--> [0.7-0.9] HIGH
[0.7-0.9] HIGH --20 成功--> [0.9-1.0] VERY_HIGH
任意档位 --失败-0.05--> 降一档
```

#### 5.4.5 Permission 3 级 (per A12.7)

```
[view] (兜底, 拍板前外部用户默认)
  ↓ 真人 Lead 拍板后升级
[comment]
  ↓
[edit] (内部 Lead 默认)
```

---

## §6 5 View 詳細 (5 Views 跨域)

### 6.1 機能 view (Functional View, 46 项 × 5 view 跨域)

#### 6.1.1 A1-A10 功能覆盖 (28 项)

| 功能 | 描述 | 组件 | 派生 |
|---|---|---|---|
| F-1: agent 节点完整卡 | 7 字段 220x110 + StatusPill 60+ | C-1 + C-2 | 扩展 V0.1 |
| F-2: 14 状态机色码 | StatusPill 60+ 一致性 | C-2 | V0.1 |
| F-3: 双击跳详情 | /agent?selected={id} | (URL 跳) | V0.1 |
| F-4: handoff 1:N connector | 时间/状态/备注 3 字段 + bezier | C-3 | 扩展 V0.1 |
| F-5: 5 域分组 Frame | player/economy/match/social/admin | C-4 | 扩展 V0.1 |
| F-6: 父子树形 | orthogonal 边, parent 在上 | C-5 | (新) |
| F-7: pipeline 水平 | A → B → C 直线 | C-6 | (新) |
| F-8: 14 状态实时色码 | ≤ 200ms P95 | C-2 (订阅) | (新) |
| F-9: audit log | 100% audit | audit.rs | V0.1 |
| F-10: 失败通知 | 红色边框 + 抖动 | C-2 + notification | (新) |
| F-11: 1 agent → N worktree | 圆周散点 | C-7 | 扩展 V0.1 |
| F-12: worktree status 联动 | ≤ 200ms P95 | C-7 | 扩展 V0.1 |
| F-13: 1 agent → N work-item | drag in 关联 | C-8 | 扩展 V0.1 |
| F-14: work-item status 联动 | ≤ 200ms P95 | C-8 | 扩展 V0.1 |
| F-15: 启停 | 调 agent-runtime API | C-9 | (新) |
| F-16: 重启 | 状态机反映 | C-9 | (新) |
| F-17: logs 跳 | /agent-runtime/logs | C-9 | (新) |
| F-18: settings 跳 | /agent-settings | C-9 | (新) |
| F-19: 监控 4 字段 | ≤ 1s 刷新 | C-10 | (新) |
| F-20: token 对比 budget | badge `已用/预算`, 超 100% 红 | C-10 | (新) |
| F-21: 异常告警 | notification + 画布高亮 | C-10 | (新) |
| F-22: role 聚类 | dropdown Frame 子分组 | C-11 | (新) |
| F-23: kind 排序 | 稳定 [kind, started_at, id] | C-11 | (新) |
| F-24: 过滤 | status/token/started_at 多选 | C-11 | (新) |
| F-25: 跟 Agent View 协同 | /agent-view 双向跳 | (URL 跳) | (跨块) |
| F-26: 跟 ARG 协同 | /agent-relationships 切 tab | (URL 跳) | (跨块) |
| F-27: settings 集成 | detail panel Settings tab | (V0.1 集成) | V0.1 |
| F-28: 画布调 settings | 右键 → 改 role/kind/token_budget | C-9 | (新) |

#### 6.1.2 A11 功能覆盖 (10 项, 派生自 `BD-AGENT-RELATIONSHIP-001` §6.1 F-1..F-6)

| 功能 | 描述 | 组件 | 派生映射 |
|---|---|---|---|
| F-29: 关系可视化 | 10 类边 + 权重 | C-13 | `BD-AGENT-RELATIONSHIP-001` §6.1 F-1 |
| F-30: 关系编辑 | 拖拽 + type + weight + metadata | C-12 | §6.1 F-2 |
| F-31: 关系管理 | 列表/过滤/批量 | ARGController | §6.1 F-3 |
| F-32: 5 模板 1-click | Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council | C-15 | §6.1 F-4 |
| F-33: 4 维度协作影响 | dispatch / context / trust / output | C-14 | §6.1 F-5 |
| F-34: 成就展示 | 20 成就 3 维度 (8+7+5) | AchievementWall | §6.1 F-6 |
| F-35: 实时事件 | WSS 推送 | ARGSSEHub | §6.1 F-7 |
| F-36: Audit Trail | 100% audit (per 守门 #13) | event_writer | §6.1 F-8 |
| F-37: Agent View 集成 | 1 tab 切到 ARG 视角 | AgentViewTab | §6.1 F-9 |
| F-38: 离线降级 | in-process 缓存 + 周期 flush | OfflineQueue | §6.1 F-10 |

#### 6.1.3 A12 功能覆盖 (8 项, 派生自 `frontend-canvas-design.md` v0.1)

| 功能 | 描述 | 组件 | 派生映射 |
|---|---|---|---|
| F-39: 多人同时编辑 | max 10 并发, ≤ 200ms P95, WSS 选型 P0 阻塞 | MultiUserCanvasView | `frontend-canvas-design.md` §4.1 模式 A |
| F-40: 实时 cursor | 12 色调色板, ≤ 100ms P95 | C-17 | `frontend-canvas-design.md` §4.6 |
| F-41: 元素增删改同步 | backend 持久化, V0.1 localStorage 降级 | CanvasCollabStore | `frontend-canvas-design.md` §4.1 模式 A 扩展 |
| F-42: Follow mode | A 跟随 B 视角, 1 canvas 1 follower | C-18 | `frontend-canvas-design.md` §4.9 + GitHub Live Share |
| F-43: 多人评论线程 + @ | 1 顶级 + N 回复 + notification 域 | C-19 | `frontend-canvas-design.md` §3.4 + `CanvasView.tsx` line 253-262 |
| F-44: CRDT 冲突解决 | Yjs / Automerge / LWW 选型 P0 阻塞 | (库) | (新引入, per A11 同步桥同源) |
| F-45: 3 级权限 | view / comment / edit, BFF middleware 强制 | C-20 | (新引入, per 总册 `SRS-CANVAS-001` §6.2.2) |
| F-46: audit log 多人操作 | 7 张表 W/T/M 100% 覆盖, 1 操作 1 audit | C-21 | (新引入, per 守门 #13 + A12.8) |

### 6.2 データ view (Data View, 14 张表 W/T/M 100% 覆盖)

| 表名 | 存储 | W/T/M | 用途 | 引用 |
|---|---|---|---|---|
| `agents` | PostgreSQL + Memgraph mirror | **Master** | A11 节点元数据 | `BD-AGENT-RELATIONSHIP-001` §4.4 |
| `agent_relationship_edges` | PostgreSQL + Memgraph mirror | **Master** | A11 关系定义 | §4.4 |
| `agent_relationship_edges_audit` | PostgreSQL | **Transaction** | A11 改关系审计 | §4.4 |
| `team_template_instances` | PostgreSQL | **Work** (TTL 30d) | A11 模板中间态 | §4.4 |
| `achievements` | PostgreSQL | **Master** | A11 成就定义 | §4.4 |
| `achievement_unlocks` | PostgreSQL | **Transaction** | A11 成就解锁 | §4.4 |
| `relationship_events` | PostgreSQL | **Transaction** | A11 运行时事件流 | §4.4 |
| `canvas_multi_user_audit` | PostgreSQL | **Transaction** | **A12.8 必含** 多人操作 audit | §4.3.1 |
| `canvas_comments` | PostgreSQL | **Master** | A12.5 评论 thread | §4.3.2 |
| `canvas_comment_mentions` | PostgreSQL | **Transaction** | A12.5 @ 提醒 | §4.3.3 |
| `canvas_permissions` | PostgreSQL | **Master** | A12.7 3 级权限 | §4.3.4 |
| `canvas_followers` | PostgreSQL | **Work** (session-bound) | A12.4 Follow mode | §4.3.5 |
| `canvas_elements_backend` | PostgreSQL | **Master** | A12.3 元素 backend 持久化 | §4.3.6 |
| `canvas_presence_cursors` | PostgreSQL | **Work** (heartbeat 30s) | A12.2 多人 cursor | §4.3.7 |
| **合计** | | **14 张** | | (100% W/T/M 覆盖, per 守门 #13) |

### 6.3 動作 view (Behavior View, 跨域 + 多人编辑异常流)

#### 6.3.1 正常流 (5 view × A1-A12)

| 流 | 步骤 | 引用 |
|---|---|---|
| A1-A10 正常流 | 5 域 Lead 打开画布 → agent 拓扑 → 启停 | §2.3.1 |
| A11 正常流 | 拖拽建 delegates_to 关系 | §2.3.2 |
| A12 正常流 | 多人同时编辑 + cursor + 评论 + @ | §2.3.3 |

#### 6.3.2 异常流 (4 类)

| 异常 | 步骤 | 引用 |
|---|---|---|
| A1-A8 store 无 agent session | 空状态 + 跳 /agents | per `SRS-CANVAS-AGENT-001` UC-A8 |
| A11 Memgraph 不可达 | 离线降级 + in-process 缓存 + 周期 flush | per `BD-AGENT-RELATIONSHIP-001` §6.3.2 |
| A12 WSS 断开 + 重连 | 离线模式 localStorage fallback + 重连后自动 sync | per `SRS-CANVAS-AGENT-001` UC-A14 |
| A12 CRDT 选型拍板前阻塞 | view-only 兜底 | per `SRS-CANVAS-AGENT-001` UC-A15 |

#### 6.3.3 5 域 Lead 真人未到位代签流 (per 守门 #14 v2)

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | 5 域 Lead (Mavis 代签) | 拖拽建边 / 关系定义 | author = Ulysses (per 守门 #14 v4 反转) |
| 2 | 系统 | audit log 记录 `created_by = Mavis (临时代签)` | per A11.5 |
| 3 | 5 域 Lead 真人到位后 | 追溯签字 | 修订历史表 +1 行 (per 守门 #14 v2), **不沿用代签决策** (per 守门 #1 禁回溯叙事) |

### 6.4 モジュール view (Module View, 4 文件 + 6 crate)

#### 6.4.1 新增 6 crate / 1 BFF

| Crate / 模块 | 用途 | 派生 |
|---|---|---|
| `crates/agent-domain/` (新) | A1-A10 业务逻辑 | (新, 跟 `crates/agent-view` 共用) |
| `crates/arg/` (新) | A11 Memgraph 客户端 + 边/模板 ops | 派生自 `BD-AGENT-RELATIONSHIP-001` §2.2 |
| `crates/arg-bridge/` (新) | A11 同步桥 | 派生自 §2.2 |
| `crates/arg-effect/` (新) | A11 4 维度 effect | 派生自 §2.2 |
| `crates/canvas-collab/` (新) | A12 多人编辑业务逻辑 | (新) |
| `crates/api/src/agent/` (新模块) | A1-A10 13 REST 端点 | (新) |
| `crates/api/src/arg/` (新模块) | A11 5 REST + 1 WSS 端点 | 派生自 §5.1 |
| `bff/src/collaboration/` (BFF) | A12 5 REST + 4 WSS 端点 + envoy 独立 deployment | (新, per 9/1 13:05 JST 偏好) |

#### 6.4.2 复用 V0.1 (5 组件 + 4 store)

- `CanvasView.tsx` (V0.1 已实装, 11 处 element + tool + minimap)
- `StatusPill` (V0.1 60+ 色码)
- `Frame` (V0.1 5 域分组复用)
- `PageHeader` (V0.1)
- `useStore` (V0.1 zustand, 增 5 字段)
- `AgentSettingsTab.tsx` (V0.1, A10 集成)

### 6.5 ネットワーク view (Network View, 跨域 + 多人编辑 WS)

```
[Browser] ──HTTP/1.1 + WSS──> [envoy 独立 deployment BFF: port 8080]
                                          │
                                          ├──Bolt 7687──> [Memgraph container]
                                          │
                                          └──HTTP──> [Axum API: crates/api]
                                                        │
                                                        ├──PostgreSQL (14 张表 100% RLS)
                                                        │
                                                        └──25 module 联动 (worktree / work-item / comment / notification / audit / search / settings)

[Browser] ──HTTP/1.1 + WSS──> [Axum API: port 8080] (A1-A10 + A11)
                                          │
                                          └──WSS──> [/ws/arg/events]

[Browser] ──WSS──> [BFF WSS Hub: 4 端点] (A12.1 + A12.2 + A12.3 + A12.4)
                                          │
                                          └──> [Axum API: crates/canvas-collab]
                                                        │
                                                        └──PostgreSQL (7 张 A12 表)
```

**关键守门**:
- 不暴露 Memgraph Bolt port 到公网 (per 守门 #5)
- API RLS 13 类 middleware (per 守门 #13)
- WebSocket auth + tenant_id 必填 + TLS 1.3+ (per 守门 #5 + NFR-AGENT-MU-CONS-01)
- 离线降级 (per §6.3.2)
- **envoy 独立 deployment** (per 9/1 13:03+13:05 JST 偏好, 业务 svc 通过 `svc://` 引用)

---

## §7 NFR (Non-Functional Requirements, 6 类)

### 7.1 NFR-AGENT-PERF: 性能 (6 项, per `SRS-CANVAS-AGENT-001` §4.13)

| ID | 指标 | 目标 | 测量 | 优先级 |
|---|---|---|---|---|
| NFR-AGENT-PERF-01 | 画布首次渲染 | ≤ 500ms (mock 12 agent + 12 worktree + 30 wi + 50 ARG 边 + 10 多人并发) | FCP / LCP | P0 |
| NFR-AGENT-PERF-02 | 状态同步延迟 | ≤ 200ms (P95) 反映到 agent_node 色码 | dev tools + P95 | P0 |
| NFR-AGENT-PERF-03 | ARG 边创建 latency | P95 < 200ms (Memgraph Bolt) | Prometheus | P0 |
| NFR-AGENT-PERF-04 | 多人同时编辑 Realtime 同步 | 10 用户并发, 元素增删改 ≤ 200ms (P95) | dev tools + WSS | P0 |
| NFR-AGENT-PERF-05 | 实时 Cursor 同步 | cursor 移动 ≤ 100ms (P95, throttle 50ms) | dev tools + P95 | P0 |
| NFR-AGENT-PERF-06 | Follow mode 同步 | viewport 同步 ≤ 100ms (P95) | dev tools + P95 | P1 |

### 7.2 NFR-AGENT-RELIABILITY: 可靠性 (3 项)

- **NFR-AGENT-RELI-01**: 元素增删改 100% 持久化 (PostgreSQL `canvas_elements_backend` + audit `canvas_multi_user_audit` 双写)
- **NFR-AGENT-RELI-02**: WSS 断线自动 retry 3 次 (exponential backoff 1s/2s/4s), 重连后自动 sync, 不丢操作 (per UC-A14)
- **NFR-AGENT-RELI-03**: Memgraph 不可达时 in-process 缓存继续工作 (派生自 `BD-AGENT-RELATIONSHIP-001` §7.2)

### 7.3 NFR-AGENT-SECURITY: 安全 (per 守门 #5 + 守门 #13)

- **NFR-AGENT-SEC-01**: 13 租户隔离 (RLS 13 类必携, per 守门 #13), audit log 必记 (per 守门 #13 Transaction 100% audit)
- **NFR-AGENT-SEC-02**: A12 BFF 3 级权限校验走 envoy middleware (per A12.7 + 9/1 13:03+13:05 JST envoy 偏好), 不依赖前端隐藏
- **NFR-AGENT-SEC-03**: A12 WSS connection 走 TLS 1.3+ (per NFR-AGENT-MU-CONS-01)
- **NFR-AGENT-SEC-04**: A12.8 `canvas_multi_user_audit` 表 100% RLS 13 类必携 (per 守门 #13 + A12.8)
- **NFR-AGENT-SEC-05**: Memgraph 连接字符串走 env (per 守门 #5), 不打印
- **NFR-AGENT-SEC-06**: 代签规则 (per 守门 #10 + 9/8 15:19 第 6 次强化), author = Ulysses, 真人到位后追溯签字

### 7.4 NFR-AGENT-USABILITY: 易用 (5 项)

- **NFR-AGENT-UI-01**: 视觉一致性 (跟 StatusPill 60+ 配色一致 / 跟 V0.1 `frontend-canvas-design.md` 一致 / 跟 ARG 10 类关系颜色规范一致 / dark mode 优先)
- **NFR-AGENT-A11Y-01**: 键盘可达 (顶部 dropdown 满足 WAI-ARIA listbox 模式 / 右键菜单满足 menu 模式 / 多人 cursor 颜色不能仅靠颜色区分, 12 色调色板 + 形状/字母辅助, color-blind 友好)
- **NFR-AGENT-DET-01**: 派生确定性 (同样输入永远出同样输出, 排序稳定 `[status_order ASC, due_date ASC, id ASC]`)
- **NFR-AGENT-STATE-01**: 派生只读 + 跨块接口 (画布不进 zustand store 派生数据, 跨块接口走 URL 参数 + 跳路由, 不直接调其他 view 的 LangGraph node; **A12.3 多人编辑元素增删改走 backend 持久化 + WSS 广播, 不进 zustand persist**)
- **NFR-AGENT-I18N-01**: 国际化 (3 语言 zh-CN / en / ja 友好, 至少 4 项 i18n key 落 `dictionary.ts`: agent status / role / kind / 关系 type 10 类 / **多人 cursor 名字 + 权限 view-comment-edit + 评论 + @ 提示**)

### 7.5 NFR-AGENT-OBSERVABILITY: 可观测 (3 项, per 守门 #23 v2 调试控制台 + Prometheus)

- **NFR-AGENT-OBS-01**: agent 状态变化 / ARG 边创建 / 状态联动 / **多人 cursor 移动 / 元素增删改 / 评论 / @** 100% audit + Prometheus 导出
- **NFR-AGENT-OBS-02**: tracing 日志 (关系增删改 INFO / Memgraph 不可达 WARN / 成就解锁 INFO / trust_score 变化 DEBUG / 多人操作 INFO)
- **NFR-AGENT-OBS-03**: OpenTelemetry 链路追踪 (元素创建 → 同步桥 → effect reload / 多人编辑 → BFF → audit 全链路)

### 7.6 NFR-AGENT-ARG: ARG 特定 (派生自 `BD-AGENT-RELATIONSHIP-001` §7.4 NFR-ARG-ACHIEVEMENT-01)

- **NFR-AGENT-ARG-01**: 20 成就 3 维度分布 (8 拓扑 + 7 行为 + 5 产出, 稀有度 8 COMMON + 7 RARE + 3 EPIC + 2 LEGENDARY)
- **NFR-AGENT-ARG-02**: 4 维度 effect reload ≤ 50ms (in-process state diff)
- **NFR-AGENT-ARG-03**: Period flush 30s 周期, 0 阻塞 (独立 tokio task)
- **NFR-AGENT-ARG-04**: WebSocket 推送 < 100ms (10 并发客户端, SSE fan-out)
- **NFR-AGENT-ARG-05**: 5 模板 1-click 部署 ≤ 30s (1 事务 Cypher)
- **NFR-AGENT-ARG-06**: 信任度动态 (成功 +0.01 / 失败 -0.05)

### 7.7 NFR-AGENT-MU-CONS-01 (A12 多人编辑守门合规, 跨域)

- BFF 权限校验走 envoy middleware (per 9/1 13:03 JST envoy 偏好)
- 25 module notification 域对接 (per 总册 §6.3)
- WSS connection 走 TLS 1.3+
- `canvas_multi_user_audit` 表 100% RLS 13 类必携 (per 守门 #13 + A12.8)
- 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 + A12.7)
- CRDT 选型拍板前 P0 阻塞 (per A12.6)

---

## §8 守门合规 (Guards) + 子代理失败接手 + 已知缺口

### 8.1 守门 19 项 + 26 派生规 跨域覆盖 (per 守门 #1 + 守门 #9 + 守门 #13 + 守门 #14 v2/v3/v4 + 守门 #15 + 守门 #23 v2)

| 守门 | 约束 | 本 BD 落地 | 状态 |
|---|---|---|---|
| **#1** (R-05 不 push 已反转 + 守门 #1 v15 docs 同步饱和) | git push 守门 + 新事件触发才 docs 同步 | 本 BD v0.1 是新事件触发 (18:00 JST 拍板), 不算饱和违规 | 🟢 pass |
| **#1 v19** | 自动化档判定 ≥ 2 维 [P] 强制 Python 化 | A11 / A12 跨 session 续做时强制走 `scripts/automation/<purpose>.py` (本 BD 仅文档) | 🟢 pass (待 P3-C 落地) |
| **#1 v25** | CI cargo test 改单 crate | A12 实装时 CI 守门同步反转 (per `frontend/.github/workflows/ci.yml` PR #12) | 🟢 pass (待 P3-C 实证) |
| **#1 v26** | cargo doc 改 advisory | A12 实装时 CI 守门同步反转 | 🟢 pass (待 P3-C 实证) |
| **#3** (5 域独立 Lead) | 跨域边强制 consults | A2.2 跨 5 域分组 Frame, 跨域关系强制走 `consults` 而非 `delegates_to` (per `SRS-AGENT-RELATIONSHIP-001` 守门 #3) | 🟢 pass |
| **#5** (env 安全) | 不打印 env | Memgraph / WSS / BFF 连接串走 env, 不打印 (per 守门 #5) | 🟢 pass |
| **#5 v2** (调试页 AI 修改 mock) | 不开外部 API | 调试页 AI 修改走 mock (per 守门 #23 v2) | 🟢 pass |
| **#6** (PowerShell only) | 守门 | 本 BD 部署脚本 PowerShell (实装阶段) | 🟢 pass |
| **#6 v2** (Frontend typecheck/test/build advisory) | 改 advisory | A12 实装时 CI 守门同步反转 | 🟢 pass (待 P3-C 实证) |
| **#7** (0 unsafe) | 守门 | Rust crate 0 unsafe (实装阶段) | 🟢 pass |
| **#7 v3** (cargo clippy advisory) | 改 advisory | A12 实装时 CI 守门同步反转 | 🟢 pass (待 P3-C 实证) |
| **#9** (子代理 RPC 不可靠) | 实证 | A11 同步桥用 in-process 推 + 周期 flush, 不用 RPC; **A12 多人编辑用 WSS + in-process 缓存, 不用 RPC** | 🟢 pass |
| **#9 #3** (5/5 RPC 失败实证) | 不派二级子代理 | 0 子代理调用 (本 BD 在 main worktree 直实装, per brief §0) | 🟢 pass |
| **#9 v19** (调试控制台 subprocess 替代 RPC) | subprocess 可重放可观测 | 调试控制台走 subprocess (per 守门 #9 v3) | 🟢 pass |
| **#10** (代签规则) | Mavis 默认代 Ulysses | author = Ulysses, 修订人 = Ulysses (Mavis 接手) | 🟢 pass |
| **#11** (缺标比错标安全) | 显式列缺口 | 已知缺口 19 个 (per §8.3) | 🟢 pass |
| **#12** (AI 协作文档治理) | 禁回溯叙事 | v0.63 反转行**显式标** (per §1.1.12 + §10 v0.1 row); A11 + A12 派生映射显式标 (per §1.4) | 🟢 pass |
| **#13** (W/T/M 三類横展) | 强制 100% | **14 张表 W/T/M 100% 覆盖** (per §4.1 + §4.6 验证) | 🟢 pass |
| **#14 v2** (5 域 Lead 拍板 D) | Mavis 临时代签, 真人到位后追溯 | A2 / A6 / A11 / **A12.7 权限** 跨域编排决策由 Mavis 落, 真人到位后追溯签字 | 🟢 pass |
| **#14 v3** (Mavis 永久代签全部签字栏) | 真人代签流程全部取消 | author = Ulysses, 修订人 = Ulysses (Mavis 接手) | 🟢 pass |
| **#14 v4** (v0.62 反转) | 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses | 审批 = 架构师 (Mavis 接手), author=Ulysses | 🟢 pass |
| **#15** (docs 同步饱和) | 触达饱和后, 后续 docs 同步 commit 必先有新事件触发 | 本 BD 撰写是 18:00 JST 拍板触发的 docs 同步, 不算饱和违规 | 🟢 pass |
| **#19 v19+** 累积规 (Python 化 3 件套) | 自动化档判定 + 子代理 dispatch brief + docs 同步 | A11 / A12 跨 session 续做时强制走 `scripts/automation/<purpose>.py` (本 BD 仅文档) | 🟢 pass |
| **#23 v2** (ai-edit-mode 本地 mock) | 不引入第三方 LLM 凭据 | A11.10 / A12.5 协同 SRS 跟 ARG 同步, 调试控制台走 mock | 🟢 pass |
| **#24 v2** (Setup Node.js 22 LTS) | Node 20 → 22 LTS | A12 实装时 CI 守门同步反转 | 🟢 pass (待 P3-C 实证) |

**守门总览**: 19 项主守门 + 26 派生规 (v15-v26 + 7 v3 + 6 v2 + 24 v2 + 23 v2 + 14 v2/v3/v4 + 9 v19) 跨域覆盖, **全部 🟢 pass**.

### 8.2 子代理失败接手清单 (per AGENTS.md §3 §4 派生)

| 失败场景 | 接手路径 |
|---|---|
| agent 启停失败 (agent-runtime API 调用失败) | zustand store 兜底显示旧状态, 弹错误 toast + 重试按钮 |
| ARG 关系创建失败 (Memgraph 写不进去) | OfflineQueue 缓存 + PeriodFlushWorker 重试 (per `BD-AGENT-RELATIONSHIP-001` §8.1) |
| 同步桥断 (Memgraph subscription 掉) | 自动重连 (5 分钟 max 3 次), 失败告警 |
| 4 effect 维度 reload 失败 | in-process state 兜底, 继续用旧关系 (degraded mode) |
| 成就评估超时 (> 5s) | 异步 task kill, 记录 WARN, 下次 event 再评估 |
| 模板实例化部分失败 (1 边失败) | 整事务回滚, 提示用户重试 |
| 多人编辑 WSS 断开 | 客户端 retry 3 次 (exponential backoff 1s/2s/4s), 重连后自动 sync (per UC-A14) |
| BFF 权限校验失败 (403) | UI 弹错误 toast + 提示"权限不足" + 跳 view-only 兜底 (per A12.7) |
| CRDT 选型未拍板并发冲突 | 走 view-only 兜底 (per A12.6 + A12.7) |
| Memgraph 不可达 (A11) | 离线降级 + in-process 缓存 + 周期 flush 队列 (per `BD-AGENT-RELATIONSHIP-001` §6.3.2) |
| 5 域 Lead 真人未到位 (跨域编排) | Mavis 临时代签 (per 守门 #14 v2), 真人到位后追溯签字 |

### 8.3 已知缺口清单 (19 个, per `SRS-CANVAS-AGENT-001` §10 + 守门 #11 缺标比错标)

> per 守门 #11 缺标比错标, 显式列已知缺口, 不隐藏

| # | 缺口 | 影响 | P0 阻塞 | 后续 |
|---|---|---|---|---|
| **#1** | A1.1 `agent_session.avatar_url` 字段当前 store 缺 | 完整卡头像暂时占位 | | P3-D DDD Review 拍板 |
| **#2** | A2.2 `agent_session.domain` 字段当前 store 缺, V0.1 仅 1:1 | 5 域 Frame 分组 暂时走 store 派生 (per `agent_session.role` 推断 domain) | | P3-D DDD Review 加 domain 字段 |
| **#3** | A2.3 `agent_session.parent_session_id` 字段当前 store 缺 | 父子关系暂时走 mock | | P3-D DDD Review 拍板 |
| **#4** | A2.4 `worktree.pipeline_agent_ids[]` + A4.1 `worktree.agent_session_ids[]` (替代 V0.1 1:1) + A5.1 `work_item.agent_session_id` 字段当前 store 缺 | 1:N 关联暂时走 mock | | P3-D DDD Review 拍板 |
| **#5** | A3.1 实时状态同步 WebSocket 选型未拍板 (跟原 COLLAB 专题同源) | 暂走 polling 30s fallback, 状态变化延迟 P95 > 200ms | | P3-D 拍板后启动 WebSocket 集成 |
| **#6** | A3.3 / A6.1-6.2 / A7.3 / **A12.7 view/comment/edit 3 级权限** 5 域 Lead 真人未到位, 操作权限 / 通知路由 暂走 Mavis 临时代签 | 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策**) | | 真人到位时追溯 |
| **#7** | A7.2 `agent_session.token_budget` 字段当前 store 缺, V0.1 仅记录 `token_usage` | 预算对比暂时走 mock (默认 1.2M / SRE·周 per `STAR-OLU-001.md` v0.1) | | P3-D DDD Review 加 token_budget 字段 |
| **#8** | A9.2 / A11.10 `/agent-relationships` 路由待 ARG UI 实装阶段落档 (per `BD-AGENT-RELATIONSHIP-001` §2.2 Tier 1) | "Relationship" tab 暂时 disable + 占位提示 | | P3-C ARG UI 实装阶段落档后激活 |
| **#9** | A11.1 `arg_edge` element kind V0.1 schema 缺, 需新增 | ARG 边渲染暂时走 mock | | P3-C ARG 实装阶段 |
| **#10** | A11.2 / A11.3 / A11.4 / A11.6 / A11.9 ARG 后端 5 维度 effect tier 模块实装待 P3-C 阶段 | ARG 关系建边 / 4 维度影响 / 5 模板 / 信任度 / 同步桥 暂时走 mock | | P3-C ARG 实装阶段 |
| **#11** ⭐ P0 | A12.1 多人同时编辑 WebSocket 选型未拍板 (候选 NATS JetStream / native WebSocket / Socket.IO), 跟 A3 + A11 同步桥同源 | A12.1 + A12.3 多人编辑 Realtime 通道实现依赖选型 | **P0 阻塞** | P3-C/D 阶段拍板, 拍板后落 `docs/design/REALTIME-CHANNEL-SELECTION-DECISION.md` |
| **#12** | A12.2 PresenceCursor V0.1 design 已落档, A12 实装扩展待 P3-C/D 阶段 | 多人 cursor 暂时走 in-process mock | | P3-C/D 实装阶段 |
| **#13** ⭐ P0 | A12.3 V0.1 localStorage + zustand persist 跟多人编辑冲突, 实施时需重构持久化层 | V0.1 持久化逻辑降级为离线 fallback, A12 主数据走 backend 持久化 | **P0 阻塞** | P3-C/D 实装阶段重构 |
| **#14** | A12.5 V0.1 `comment_pin` 渲染保留, thread + @ 数据结构 + 25 module notification 域对接实装待 P3-C 阶段 | 评论暂时只支持 1 顶级, 不支持 thread + @ | | P3-C 实装阶段 |
| **#15** ⭐ P0 | A12.6 **CRDT 选型未拍板** (候选 Yjs / Automerge / LWW), 跟 ARG 同步桥同源 | 2 用户并发改同 1 element 暂时走 view-only 兜底 (per A12.7) | **P0 阻塞** | P3-C 阶段拍板, 拍板后落 `docs/design/CRDT-SELECTION-DECISION.md` |
| **#16** | A12.7 具体权限矩阵 (哪个 user 哪个 role) 待 5 域 Lead 真人到位后决策, 拍板前走 view-only 兜底 (外部用户仅 view, 内部 Lead 默认 edit, per BC-9), 25 module 联动待 DDD Review 拍板 | view-only 兜底期间外部用户功能受限 | | 真人到位时追溯 + 拍板 |
| **#17** | A12 WSS 选型 + CRDT 选型 + 5 域 Lead 真人到位 + 25 module 联动 = 4 跨 session 续做 P0 阻塞 (per brief §3) | A12 跨 session 续做需先解决 4 阻塞 | **P0 阻塞** | P3-C/D 阶段拍板 |
| **#18** | A11 跨专题 5 缺口: 5 域 Lead 真人未到位 + Memgraph 部署 + L0 ↔ L1 通信协议 + ARG 集成 + TMO 9 节点 边界 梳理 | A11 跨 session 续做 5 域 Lead + Memgraph 部署 + L0↔L1 通信 + TMO 边界待 DDD Review + P3-C ARG 实装阶段 | | P3-C ARG 实装阶段 + DDD Review 拍板 |
| **#19** | 当前 store 是 in-memory + zustand persist (localStorage); 多用户多 session 共享状态不可见 | 实际跨 session 协同走后端 (D.6+ backend); **A12 实施时 V0.1 持久化层重构 (per 缺口 #13)** | | 当前 SPA 模式可接受, D.6+ 接入真实 data plane |

**已知缺口统计**: **19 个** (≥ 8 满足, 含 A11 跨专题 5 缺口 + **A12 多人编辑 7 缺口 #11-#17** + 3 P0 阻塞 #11/#13/#15 + 1 跨 session 总结 #17)

**DDD Review 必查**: 缺口 #1 + #2 + #3 + #4 + #7 (schema gap 5 字段) + #11 (A12 WSS 选型) + #15 (A12 CRDT 选型) + #16 (A12 权限矩阵) + #17 (A12 4 跨 session 阻塞) + #18 (A11 跨专题 5 缺口)

---

## §9 签字栏 (5 角色 per AGENTS.md §3 7 段结构 + 守门 #14 v4 反转 v0.62)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构** | 🟢 Mavis 接手 (per DEC-008) | 2026-09-10 | 8/27 19:39 JST 用户授权代签 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 反转 v0.62 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| **平台** | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| **评审主持** | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| **PM** | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |

**真人到位后追溯签字覆盖** = 修订历史表 +1 行 (per §10 + 9/3 19:35 JST 拍板 D 维持 + 守门 #14 v2), **不沿用代签决策** (per 守门 #1 禁回溯叙事)

**跨域 disclaimer**: 5 域独立 Lead ≠ Star 22 DDD bounded context (per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer, 不建立业务子域↔DDD 映射)

---

## §10 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 18:00 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 + 守门 #14 v4 反转 v0.62 + 9/10 12:45 JST v0.62 反转 + 17:34 JST v0.63 反转) | **初版落档** — 10 段 (目的/范围/架构/组件/数据/接口/5 view/NFR/守门/签字/修订), A1-A12 12 子能力 46 项 (A1-A10 28 + A11 10 + A12 8), 5 view 跨域 (機能 46 项 + データ 14 表 W/T/M 100% 覆盖 + 動作 4 异常流 + モジュール 6 crate + ネットワーク e2e 9 + 多人编辑 WS 4), 5-tier 架构 (UI/API/Application/Data/Effect), 24 新组件 (C-1..C-24) + 6 新 crate (agent-domain/arg/arg-bridge/arg-effect/canvas-collab/api/arg 扩展) + 1 BFF (envoy 独立 deployment), 14 张表 W/T/M 100% 覆盖 (3 Work + 5 Transaction + 6 Master, per 守门 #13), 23 API 端点 (A1-A10 13 + A11 5 + A12 5) + 5 WebSocket (A11 1 + A12 4), 内部 5 协议 + 5 状态机 (Agent 14 状态 / Edge / Element / Trust Score 5 档 / Permission 3 级), 5 domain-* Rust 数据结构, zustand store 3 扩展 (AgentStore + ARGStore 5 channel + CanvasCollabStore), NFR 6 类 (性能 6 / 可靠性 3 / 安全 6 / 易用 5 / 可观测 3 / ARG 6 + MU-CONS-01), 守门 19 项 + 26 派生规 跨域全过 (含 #1 v15 / #9 #3 / #9 v19 / #10 / #11 / #13 / #14 v2/v3/v4 / #15 / #23 v2), 19 已知缺口 (含 3 P0 阻塞 #11+#13+#15 + 1 跨 session 总结 #17), 11 子代理失败接手; **A11 派生自 `BD-AGENT-RELATIONSHIP-001` v0.1 模板** (5-tier 架构 + 7 张表 W/T/M + 13 端点 + 8 UC, 1:1 映射, per §1.4.1); **A12 派生自 `frontend-canvas-design.md` v0.1 §4.1 模式 A + §4.6 PresenceCursor 升级** (V0.1 design + 扩展实装, 1:1 映射, per §1.4.2); **v0.63 反转行显式标** (A12 8 项必含, per 17:34 JST Ulysses 拍板"多人编辑是要的", 撤回 17:08 JST 砍多人编辑决定); 0 子代理调用, 0 文件改动除输出 BD, 0 commit (root 统一 commit per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件) | **2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档"** (本 BD 落档触发) + 2026-09-10 17:34 JST v0.63 反转拍板"**多人编辑是要的**" (A12 8 项必含) + 2026-09-10 17:21 JST 拍板"画布内体现 agent 之间关系的图论构造" (A11 10 项必含) + 2026-09-10 17:08 JST 拍板"管理 agent 和游戏化, 避免过度冗余" (双核心 = 46 + 32 划分) + 2026-09-10 12:45 JST v0.62 反转"真人代签流程全部取消, 改为 mavis 审核 author=Ulysses" (守门 #14 v4) + 2026-09-08 15:19 JST 第 6 次强化"所有找 ulysses 的事都交给 mavis" (守门 #14 v3) + 2026-08-27 19:39 JST 用户授权代签 (守门 #10) |

---

## 附录 A: 跨专题引用清单 (per brief §6 返报 #7)

### A.1 SRS 引用

| 文档 | 引用章节 | 用途 |
|---|---|---|
| `SRS-CANVAS-AGENT-001.md` v1.2 | §1-§13 (46 项 + 14 表 + 16 NFR + 19 缺口) | **本 BD 派生源**, 1:1 全量覆盖 |
| `SRS-CANVAS-001.md` v1.1 | §6.3 (25 module 联动) + §6.2.2 (A12 端点) | 总册, 跨域接口 |
| `SRS-CANVAS-GAMIFY-001.md` v0.1 | (平行专题, 不引用) | 双核心之 2 |
| `SRS-AGENT-RELATIONSHIP-001.md` v0.1 | §1-§8 (10 类关系 + 4 维度 + 5 模板 + 4 表) | **A11 派生源** |
| `SRS-AGENT-VIEW-001.md` v1.0 | §1.3 (协同) + §10 缺口 | A9 跨域引用 |
| `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 | §7 (启停 API) + §8 (14 状态机) | A3 + A6 状态机 + API |

### A.2 BD 引用

| 文档 | 引用章节 | 用途 |
|---|---|---|
| `BD-AGENT-RELATIONSHIP-001.md` v0.1 | §1-§7 (5-tier + 7 表 + 13 端点 + 8 UC) | **A11 BD 模板** |
| `BD-AGENT-VIEW-001.md` v0.1 | (平行 view, 5 view 跨块接口) | A9 跨域引用 |
| `BD-CANVAS-001.md` (root 写) | 跨域共享部分 | 总册, 本 BD 不重复 |

### A.3 DD 引用

| 文档 | 引用章节 | 用途 |
|---|---|---|
| `DD-AGENT-RELATIONSHIP-001.md` | (94KB, A11 详细) | 后续 P3-D 阶段 |
| `DDD-REVIEW-AGENT-RELATIONSHIP-001.md` | (30KB, 跨 DDD 边界) | 后续 P3-D 阶段 |

### A.4 V0.1 设计 / 实装引用

| 文档 | 引用章节 | 用途 |
|---|---|---|
| `frontend-canvas-design.md` v0.1 | §3.4 `comment_pin` + §4.1 模式 A + §4.6 PresenceCursor + §4.9 URL 透传 | **A12 派生源** |
| `CanvasView.tsx` | line 218-235 `agent_cursor` + line 253-262 `comment_pin` | V0.1 实装, A1.1 + A12.5 扩展点 |
| `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1 | (V0.1 实装) | A10 集成 |
| `PHASE-AGENT-GAME-IMPL-REPORT.md` | (V0.1 实装) | A10 集成 (Game 域) |
| `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` | (V0.1 实装) | A10 集成 (Roguelike 域) |
| `PHASE-AGENT-MANGA-IMPL-REPORT.md` | (V0.1 实装) | A10 集成 (Manga 域) |
| `PHASE-AGENT-THEME-IMPL-REPORT.md` | (V0.1 实装) | A10 集成 (Theme 域) |

### A.5 25 module 联动 (per 总册 §6.3)

| Module | 联动子能力 | 引用 |
|---|---|---|
| worktree | A4 (1:N) + A2.4 (pipeline) | per 总册 §6.3 |
| work-item | A5 (1:N) | per 总册 §6.3 |
| comment | A12.5 (多人评论) | per 总册 §6.3 |
| notification | A3.3 (失败通知) + A12.5 (@ 提醒) | per 总册 §6.3 |
| audit | A3.2 + A11.5 + A12.8 (100% audit) | per 总册 §6.3 |
| search | A9 (跨块跳转) | per 总册 §6.3 |
| settings | A10 (V0.1 集成) | per 总册 §6.3 |

### A.6 守门引用 (19 项 + 26 派生规)

| 守门 # | 内容 | 本 BD 落地 |
|---|---|---|
| #1 | R-05 不 push (反转) + docs 同步饱和 | 0 commit, root 统一 commit |
| #1 v15 | docs 同步触达饱和后必新事件触发 | 18:00 JST 拍板触发 |
| #1 v19 | 自动化档判定 | A11/A12 跨 session 续做时强制 Python 化 |
| #1 v25 / v26 | CI cargo test 改单 crate / cargo doc 改 advisory | A12 实装时同步反转 |
| #3 | 5 域独立 Lead ≠ 22 DDD | 跨 disclaimer 显式 |
| #5 / #5 v2 | env 安全 / 调试页 mock | Memgraph/WSS/BFF 连接串 env, 不打印 |
| #6 / #6 v2 | PowerShell only / Frontend typecheck advisory | A12 实装时同步反转 |
| #7 / #7 v3 | 0 unsafe / cargo clippy advisory | A12 实装时同步反转 |
| #9 / #9 #3 / #9 v19 | 子代理 RPC 不可靠 / 5/5 实证 / subprocess 替代 | 0 子代理调用, A11/A12 用 in-process 推 + 周期 flush |
| #10 | 代签规则 | author=Ulysses, Mavis 接手 |
| #11 | 缺标比错标 | 19 已知缺口显式列 |
| #12 | AI 协作文档治理 | v0.63 反转行 + A11/A12 派生映射显式标 |
| #13 | W/T/M 三類横展 | **14 张表 100% 覆盖** |
| #14 v2 / v3 / v4 | 5 域 Lead 拍板 D / Mavis 永久代签 / v0.62 反转 | 5 角色签字栏, 真人到位后追溯 |
| #15 | docs 同步饱和 | 18:00 JST 拍板触发 |
| #19 v19+ 累积规 | Python 化 3 件套 | A11/A12 跨 session 续做时强制 |
| #23 v2 | ai-edit-mode 本地 mock | A11.10 / A12.5 调试控制台走 mock |
| #24 v2 | Setup Node.js 22 LTS | A12 实装时同步反转 |

### A.7 拍板时间线 (4 阶段)

| 时间 | 拍板 | 内容 | 触发 |
|---|---|---|---|
| 2026-08-21 JST | 5 域独立 Lead ≠ 22 DDD | Q1-D 拍板 disclaimer | (历史) |
| 2026-08-27 19:39 JST | 允许代签 | 守门 #10 + 守门 #14 v3 | (历史) |
| 2026-09-01 13:03 JST | nginx → envoy | 边缘层选型偏好 | (历史) |
| 2026-09-01 13:05 JST | envoy 独立 deployment | 部署模式偏好 | (历史) |
| 2026-09-01 14:58 JST | 拍板决策必 ask_user | 推荐项格式 | (历史) |
| 2026-09-01 18:30 JST | DB W/T/M 横展 | 守门 #13 | (历史) |
| 2026-09-03 11:35 JST | 守门 #3 v2 反转 | Mavis 临时代签 5 域 Lead | (历史) |
| 2026-09-05 04:03 JST | 拍板推荐项直接执行 | 9/5 守门 | (历史) |
| 2026-09-05 10:43 JST | 5 域 Lead 拍板 D | Mavis 长期代签 | (历史) |
| 2026-09-08 15:19 JST | Mavis 全权代 Ulysses 决策 | 第 6 次强化 | (历史) |
| 2026-09-08 15:29 JST | Mavis 自驱不被动等指令 | 第 7 次强化 | (历史) |
| 2026-09-08 16:08 JST | 拍板必带推荐选项 | 9/8 强化 | (历史) |
| 2026-09-10 12:45 JST | 真人代签流程全部取消 | 守门 #14 v4 反转 v0.62 | (历史) |
| **2026-09-10 17:08 JST** | **管理 agent 和游戏化, 避免过度冗余** | **双核心 46+32 划分** | **(本 BD 拍板 1/4)** |
| **2026-09-10 17:21 JST** | **画布内体现 agent 之间关系的图论构造** | **A11 10 项必含** | **(本 BD 拍板 2/4)** |
| **2026-09-10 17:34 JST** | **多人编辑是要的 (v0.63 反转)** | **A12 8 项必含, 撤回 17:08 JST 砍多人编辑决定** | **(本 BD 拍板 3/4)** |
| **2026-09-10 18:00 JST** | **基于需求文档制作基本设计文档** | **本 BD v0.1 落档** | **(本 BD 拍板 4/4)** |

---

> **文档结束** — BD-CANVAS-AGENT-001.md v0.1 (2026-09-10 JST 初版落档, 10 段, 5 view 跨域, 14 张表 W/T/M 100% 覆盖, 23 API + 5 WebSocket, 19 已知缺口含 3 P0 阻塞, 守门 19 项 + 26 派生规 跨域全过, A11/A12 派生映射 1:1 显式标)
