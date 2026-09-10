# Brief: bd-canvas-agent-001

**Agent**: worker (root 派发, 2 子代理并行之 1/2 per 守门 #9 v20 + v27 3 段 fallback)
**Phase**: P3-D.5 BD 基本设计 (18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档")
**Created**: 2026-09-10 18:02 JST
**Token 预算**: ~0.4M (守门 #4 / #19 估算, 1 SRE·周 = 1.2M 留 3x 缓冲)
**Worktree**: 在 root 当前 main worktree 直实装 (per 守门 #9 #3 实证 5/5 RPC 不可靠, 不派二级子代理)

---

## 0. 触发

2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" — 基于 SRS-CANVAS-AGENT-001.md v1.2 (155KB, 46 项 = 38 v1.1 + 8 A12 多人编辑, 13 段, 14 张表 W/T/M 100% 覆盖) 制作 agent 管理域 BD.

撤回 17:08 JST 砍多人编辑决定 (per 17:34 JST v0.63 反转) + 17:21 JST 拍板 ARG 图论构造 (A11 10 项).

## 1. 范围 (in-scope)

### 1.1 本专题 BD 覆盖 — Agent 管理域 (双核心之 1)

**核心**: A1-A12 (12 子能力, 46 项) 的基本设计, 包括 agent 节点 + 拓扑 + 状态 + worktree + work-item + 操作 + 监控 + 聚类 + 跨域 + settings + **ARG (10 类关系 + 4 维度 + 5 模板 + 同步桥 + 成就)** + **多人编辑 (8 项: 多人同时编辑 + 实时 cursor + 元素增删改 + Follow mode + 评论线程 + @ + 冲突解决 + audit)**.

| 子能力 | 来源 SRS | 内容 | 项数 |
|---|---|---|---|
| A1 agent 节点 | SRS §4.1 | agent_cursor → agent_node 完整卡 + StatusPill 60+ + 双击跳详情 | 3 |
| A2 agent 拓扑 | SRS §4.2 | handoff / 5 域 / 父子 / pipeline | 4 |
| A3 agent 状态 | SRS §4.3 | 14 状态机实时色码 + audit + notification | 3 |
| A4 worktree | SRS §4.4 | 1 agent → N worktree, status 联动 | 2 |
| A5 work-item | SRS §4.5 | 1 agent → N work-item, status 联动 | 2 |
| A6 操作菜单 | SRS §4.6 | 启停 / 重启 / logs / settings | 4 |
| A7 监控面板 | SRS §4.7 | status / token / cost / runtime 仪表 + budget + 告警 | 3 |
| A8 聚类/排序/过滤 | SRS §4.8 | role / kind / status / token / 启动时间 | 3 |
| A9 跨域引用 | SRS §4.9 | 跟 SRS-AGENT-VIEW-001 / SRS-AGENT-RELATIONSHIP-001 协同 | 2 |
| A10 settings | SRS §4.10 | 跟 V0.1 AgentSettingsTab 集成 | 2 |
| A11 ARG | SRS §4.11 | 10 类关系 + 4 维度 + 5 模板 + 同步桥 + 成就 + audit | 10 |
| A12 多人编辑 | SRS §4.12 (per 17:34 JST v0.63 反转) | 多人 + cursor + 增删改 + Follow + 评论 + @ + CRDT + audit | 8 |
| **合计** | | | **46** |

### 1.2 引用 baseline (必读, 不能编造)

| 文档 | 用途 | 路径 |
|---|---|---|
| **专题 SRS (本 BD 派生源)** | A1-A12 46 项 + 14 张表 W/T/M 需求 | `D:\Star\docs\requirements\SRS-CANVAS-AGENT-001.md` v1.2 (155KB) |
| **BD 模板 (10 段)** | 严格按 10 段 IPA SEC 模板 | `D:\Star\docs\design\BD-AGENT-RELATIONSHIP-001.md` v0.1 (55KB) |
| 平行 BD 参照 (agent view 画布) | 5 view + 跨块接口 + 数据模型 | `D:\Star\docs\design\BD-AGENT-VIEW-001.md` v0.1 (44KB) |
| **ARG 主源 (A11 派生)** | 10 类关系 + 4 维度 + 5 模板 + 4 表 + 8 UC | `D:\Star\docs\requirements\SRS-AGENT-RELATIONSHIP-001.md` v0.1 |
| **ARG 基本设计 (A11 模板)** | 5-tier 架构 + 7 张表 W/T/M + 13 端点 + 8 UC | `D:\Star\docs\design\BD-AGENT-RELATIONSHIP-001.md` v0.1 (55KB) |
| **ARG 详细设计 (A11 DD)** | 13 模板类 + 8 UC 详细实现 | `D:\Star\docs\design\DD-AGENT-RELATIONSHIP-001.md` (94KB) |
| **ARG DDD Review** | 跨 DDD 边界 + 5 派板 | `D:\Star\docs\design\DDD-REVIEW-AGENT-RELATIONSHIP-001.md` (30KB) |
| 总册 SRS + 总册 BD (root 写) | 跨域接口 + 共享约束 | `D:\Star\docs/requirements/SRS-CANVAS-001.md` v1.1 + `D:\Star\docs/design/BD-CANVAS-001.md` (本批 root 写) |
| V0.1 canvas design | 14 element + 4 frame + 8 connector + 9 e2e 守门 | `D:\Star\docs\frontend-canvas-design.md` v0.1 |
| V0.1 实装代码 | CanvasView.tsx 11 处 element + tool + minimap | `D:\Star\frontend/src/components/CanvasView.tsx` |
| SRS-STAR-AGENT-RUNTIME-001.md | 14 状态机 + 9 SA Archetype | `D:\Star\docs\requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 |
| V0.1 agent settings 4 份 PHASE | A10 集成 (Settings + Game + Roguelike + Manga) | `D:\Star\docs/reports/PHASE-AGENT-{SETTINGS,GAME,ROGUELIKE,MANGA,THEME}-IMPL-REPORT.md` |
| 25 module 联动 | work-item / worktree / agent / relation / comment / search / notification | per 总册 §6.3 |

### 1.3 文档结构 (per BD-AGENT-RELATIONSHIP-001 v0.1 模板, 10 段)

```
§0 目的 (Purpose)
§1 适用范围 (Scope) — 1.1 In-Scope (A1-A12 12 子能力) + 1.2 Out-of-Scope (Miro 通用 12 类砍掉 + 详细设计 DD 后续)
§2 系统架构 (System Architecture) — 5 view (機能/データ/動作/モジュール/ネットワーク) 跨 A1-A12
§3 组件一览 (Components) — A1-A12 子能力组件 + 新增 12 module / 复用 V0.1 组件
§4 数据模型 (Data Model) — 7 张表 (Master 3 + Transaction 3 + Work 1) + 5 domain-* Rust 数据结构 + zustand store 扩展 + 14 张表跨域汇总
§5 接口设计 (Interface Design) — BFF REST API (双核心 13 + A11 5 + A12 5 = 23 端点) + WebSocket 5 端点 + 内部 5 协议 + 5 view 跨域 API
§6 5 view 詳細 (5 views) — 機能 view (46 项) + データ view (14 表) + 動作 view (5 域 + 多人编辑) + モジュール view (4 文件 + 6 crate) + ネットワーク view (e2e 9 + 多人编辑 WS)
§7 NFR (Non-Functional Requirements) — 6 类 (性能/可靠性/安全/易用/可观测/ARG)
§8 守门 (Guards) + 子代理失败接手 + 已知缺口 (19 个 per SRS + 3 P0 阻塞)
§9 签字栏 (5 角色 per AGENTS.md §3)
§10 修订履历 (v0.1 + 修订人 + 触发)
```

### 1.4 输出文件

| 文件 | 内容 | 预估大小 |
|---|---|---|
| `D:\Star\docs\design\BD-CANVAS-AGENT-001.md` | 专题 BD 完整 10 段, A1-A12 46 项 | ~50-70K 字 |

**仅输出 1 份文件**, 不拆 commit, 不写 report, 不动 implementation.

## 2. 范围外 (out-of-scope, 由其他子代理 / 总册 BD / root 处理)

| 类别 | 处理方 |
|---|---|
| 双核心之 2: 游戏化 32 项 详细 BD | 专题 BD 子代理 2 (bd-canvas-gamify-001) |
| 总册 BD (跨域共享部分) | root 写 (bd-canvas-total-001) |
| 详细设计 (DD) 文档 | 后续 P3-D 阶段, 待 SRS + BD 落档后启动 |
| 实现 (PHASE-* 报告) | 后续 P3-D.6 阶段, 待 DD 落档后启动 |
| Miro 通用 12 类 | ❌ 砍掉, 留 P3+ 评估 |
| 25 module 实体实现 | 25 module 各自 docs, 画布只联动不实装 |
| 5 域 Lead 真人到位 | per 守门 #14 v2, Mavis 临时代签, 真人到位后追溯签字 |

## 3. 已知缺口 (per 守门 #11 缺标比错标)

本 BD 须在 §8 显式列已知缺口 (≥ 19 个, per SRS AGENT v1.2 §10 19 缺口), 不得隐藏:
- **A12.1 多人同时编辑 WebSocket 选型未拍板** (P0 阻塞, 候选 NATS JetStream / native WebSocket / Socket.IO)
- **A12.2 PresenceCursor V0.1 design 已落档, A12 实装扩展待 P3-C/D 阶段**
- **A12.3 V0.1 localStorage + zustand persist 跟多人编辑冲突, 实施时需重构持久化层 (P0)**
- **A12.5 V0.1 comment_pin 渲染保留, thread + @ 数据结构 + 25 module notification 域对接实装待 P3-C 阶段**
- **A12.6 CRDT 选型未拍板** (P0 阻塞, 候选 Yjs / Automerge / LWW)
- **A12.7 具体权限矩阵待 5 域 Lead 真人到位后决策, 拍板前走 view-only 兜底**
- **A12 4 跨 session 续做 P0 阻塞总结** (WSS 选型 + CRDT 选型 + 5 域 Lead + 25 module 联动)
- **A1 avatar 字段** (per SRS §10 #1)
- **A2 domain 字段** (per SRS §10 #2)
- **A2 parent_session_id 字段** (per SRS §10 #3)
- **A2 pipeline_agent_ids 字段** (per SRS §10 #4)
- **A3 WebSocket 选型** (per SRS §10 #5, 跟 A12 同源)
- **A6 5 域 Lead 真人未到位** (per SRS §10 #6, 跨域跨专题)
- **A7 token_budget 字段** (per SRS §10 #7)
- **A9 /agent-relationships 路由** (per SRS §10 #8, 跟 A11 协同)
- **A11 arg_edge kind** (per SRS §10 #9)
- **A11 5 维度 effect tier 模块实装待 P3-C 阶段** (per SRS §10 #10, 跨域跨专题)
- **A11 跨 session 续做 5 域 Lead + Memgraph 部署 + L0↔L1 通信 + TMO 边界** (per SRS §10 #18, 跨域)
- **A19 store 持久化** (per SRS §10 #19, 跟 A12.3 同源, 跨域)

**DDD Review 必查**: schema gap 5 字段 (#1-#4+#7) + A12 4 P0 阻塞 (#11+#15+#16+#17) + A11 跨专题 (#18) + 守门 #1 禁回溯叙事 + 守门 #14 v2/v3/v4 + 守门 #13 14 张表 100% 覆盖

## 4. 守门硬约束 (per 守门 #1 + 守门 #9 #3 + 守门 #13 + 守门 #14 + 守门 #15)

- 文档结构严格 10 段 (per BD-AGENT-RELATIONSHIP-001 模板), 不增不减
- 46 项每项 5 view 跨域覆盖 (不重写总册 BD 跨域部分)
- 已知缺口 ≥ 19 个 (含 3 P0 阻塞, 跨域)
- NFR 6 类 (性能/可靠性/安全/易用/可观测/ARG)
- 守门 19 项 + 26 派生规 跨域
- 5 角色签字栏 per AGENTS.md §3 (架构师 / SRE Lead / 平台 / 评审主持 / PM)
- **5 域 Lead ≠ Star 22 DDD bounded context** disclaimer 显式 (per 2026-08-31 22:45 JST Q1-D 拍板)
- **5 域 Lead 真人未到位前 Mavis 临时代签, 真人到位后追溯签字** (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
- **AI mock 接口** (per 守门 #23 v2, GAMIFY G5 走 mock, 真实 LLM 留 P2)
- **DB W/T/M 三類横展** (per 守门 #13, 14 张表 100% 覆盖, 禁止混在)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses)
- 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
- 0 文件改动除输出 BD
- 0 commit, 仅产出 markdown (root 统一 commit per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
- **不重写 4 SRS commit** (per 守门 #1 禁回溯叙事, BD 是新方向, 不回写 SRS)
- **不重写专题 BD** (本专题 BD 是 A1-A12 详细, 总册 BD 是跨域汇总, 2 专题 BD 互不重复)
- **A11 必须派生自 `BD-AGENT-RELATIONSHIP-001.md` v0.1 模板** (5-tier 架构 + 7 张表 W/T/M + 13 端点 + 8 UC)
- **A12 必须派生自 `frontend-canvas-design.md` v0.1 §4.1 模式 A Realtime 通道 + §4.6 PresenceCursor 升级** (V0.1 已有 design, 扩展实现)

## 5. 落地清单

| # | 文件 | 内容 | 行数预估 |
|---|---|---|---|
| 1 | `D:\Star\docs\design\BD-CANVAS-AGENT-001.md` | 10 段 BD, A1-A12 46 项 | ~1200-1700 行 |

预估 0 commit (root 统一 commit), 1 文件, ~50-70K 字。

## 6. 返报告知 (per 守门 #9 v27 collect_output)

子代理返回时, 报告必须含:
1. 实际写入文件路径 + 字节数
2. 10 段是否齐全 (§0~§10)
3. 5 view 覆盖检查 (機能/データ/動作/モジュール/ネットワーク) 跨 A1-A12
4. **14 张表 W/T/M 100% 覆盖** (A11 7 张 + A12 7 张, 列出每张表归到 W/T/M 哪類)
5. 23 API 端点 + 5 WebSocket 跨域汇总 (BFF REST + WebSocket, 列表)
6. 已知缺口清单 (≥ 19 个, 含 3 P0 阻塞)
7. 跨专题引用 (引用 SRS-CANVAS-{AGENT v1.2, 001 v1.1} + 现有 canvas design + 25 module + BD-AGENT-RELATIONSHIP-001 v0.1 + BD-AGENT-VIEW-001 v0.1 + 守门 19 项 + 拍板 4 阶段)
8. **A11 派生自 BD-AGENT-RELATIONSHIP-001 v0.1 模板** (5-tier 架构 + 7 张表 W/T/M + 13 端点 + 8 UC, 1:1 映射)
9. **A12 派生自 `frontend-canvas-design.md` v0.1 §4.1 模式 A Realtime 通道 + §4.6 PresenceCursor 升级** (V0.1 design + 扩展实现, 1:1 映射)
10. 守门 19/19 跨域覆盖 (含 #1 v15 / #9 #3 / #9 v19 / #10 / #11 / #13 / #14 v2/v3/v4 / #15 / #23 v2)
11. 任何意外 / 偏离 / 简化 / 跳过 项, 显式标注

不要只回 "done" — 必须给可验证证据.

## 7. 元数据 (per AGENTS.md §3 守门 #10 + 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 + 9/10 12:45 JST v0.62 反转 + 17:34 JST v0.63 反转)

- 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手`
- 审批: `架构师 (Mavis 接手 agent per DEC-008)` (5 角色签字栏 per AGENTS.md §3)
- 日期: 2026-09-10 JST
- 关联 commit: 留空 (root 统一 commit 时填)
- 关联文档: SRS-CANVAS-AGENT-001.md v1.2 (本 BD 派生源) + 2 份平行 SRS + 现有 canvas design + BD-AGENT-RELATIONSHIP-001 v0.1 (A11 模板) + BD-AGENT-VIEW-001 v0.1 (平行 BD)
- 拍板来源: 2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" + 17:34 JST v0.63 反转多人编辑 + 17:21 JST ARG 图论构造 + 17:08 JST 双核心

## 8. 起点

读完 14 份必读后, 用 Write 工具写 `D:\Star\docs\design\BD-CANVAS-AGENT-001.md` (10 段, 全量覆盖).
