# Brief: bd-canvas-total-001

**Agent**: root session (Mavis 接手, 不派子代理 per 守门 #9 #3 实证 5/5 RPC 不可靠, root 直实装 BD 总册)
**Phase**: P3-D.5 BD (基本设计, 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档")
**Created**: 2026-09-10 18:02 JST
**Token 预算**: ~0.1M (守门 #4 / #19 估算, root 直实装, BD 总册比专题 BD 短)
**Worktree**: 在 root 当前 main worktree 直实装 (per 守门 #9 #3 实证 5/5 RPC 不可靠)

---

## 0. 触发

2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" — 基于 3 份 SRS 文档 (SRS-CANVAS-001 v1.1 总册 58KB + SRS-CANVAS-AGENT-001 v1.2 155KB + SRS-CANVAS-GAMIFY-001 v1.0 92KB, 共 4 commit 4f56979 / 73d490f / 396833a / f35b3f5 累计 1.46M tokens, 第 68-71 次新事件触发) 制作 3 份 BD, 按 SRS 1 总册 + 2 专题 拆分模式.

## 1. 范围 (in-scope)

### 1.1 本 BD 覆盖 — 总册基本设计 (跨域共享部分)

**核心**: 双核心 (管理 agent + 游戏化) + 多人编辑 (v0.63 反转) + ARG (10 类关系) + 跨块接口 + 共享约束 + 25 module 联动 + 14 张表 W/T/M + 4 守门合并.

| 子能力 | 来源 SRS | 内容 | 项数 |
|---|---|---|---|
| 系统架构 (5 view 跨域) | 总册 + 2 专题交叉 | 5 view (機能/データ/動作/モジュール/ネットワーク) 跨域覆盖 | n/a |
| 跨块接口 (3 SRS 互引用) | 总册 §4.5 | 2 专题 SRS 互引用 + 25 module 联动 + 双核心 + ARG 互引用 | 13 跨块 |
| 共享约束 (跨域 NFR) | 总册 §5 + §7 | 性能 + a11y + 安全 + 扩展性 + i18n + 兼容性 + 约束 + 风险 | 6 类 + 13 风险 |
| API 端点 (BFF + WS) | 总册 §6.2 | V0.1 9 API + 双核心扩展 13 API + ARG 5 API + A12 5 API = 32 API | 32 API |
| 25 module 联动 | 总册 §6.3 | 9 类 25 module (work-item/worktree/agent/relation/comment/automation/audit/search/notification) + V0.1 game 4 份 | 10 module |
| 14 张表 W/T/M (跨域) | 总册 §7 + GAMIFY §7 + AGENT §7 | A11 7 张 + A12 7 张 = 14 张 100% 覆盖 | 14 表 |
| 多人编辑 P0 阻塞 | 总册 §4.4 + AGENT A12 | WSS 选型 + CRDT 选型 + 5 域 Lead 决策 | 3 P0 阻塞 |
| 5 域 Lead 真人 + 4 拍板 | 总册 §7.1 + §9.3 | 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D + 9/8 15:19 JST 第 6 次强化 + 9/10 12:45 JST v0.62 反转 | 4 拍板 |
| 守门跨域汇总 | 总册 §9.4 | 19 守门 + 26 派生规 + 守门 #14 v2/v3/v4 全部 | 19+26 守门 |

### 1.2 引用 baseline (必读, 不能编造)

| 文档 | 用途 | 路径 |
|---|---|---|
| 总册 SRS | 顶层需求 + 双核心索引 + 跨块接口 | `D:\Star\docs\requirements\SRS-CANVAS-001.md` v1.1 (58KB) |
| 专题 SRS (双核心之 1) | agent 管理 46 项 + A11 ARG + A12 多人编辑 | `D:\Star\docs\requirements\SRS-CANVAS-AGENT-001.md` v1.2 (155KB) |
| 专题 SRS (双核心之 2) | 游戏化 32 项 + G11 W/T/M + G5 AI mock | `D:\Star\docs\requirements\SRS-CANVAS-GAMIFY-001.md` v1.0 (92KB) |
| **BD 模板 (10 段)** | 严格按 10 段 IPA SEC 模板 | `D:\Star\docs\design\BD-AGENT-RELATIONSHIP-001.md` v0.1 (55KB, 9/8 落档, ARG 10 段模板) |
| 平行 BD 参照 | agent view 画布 + 关联 view | `D:\Star\docs\design\BD-AGENT-VIEW-001.md` v0.1 (44KB, 9/5 落档) |
| V0.1 canvas design | 现状基线 (14 element + 4 frame + 8 connector + 9 e2e 守门) | `D:\Star\docs\frontend-canvas-design.md` v0.1 |
| V0.1 实装代码 | CanvasView.tsx 11 处 element 渲染 + 工具栏 + minimap | `D:\Star\frontend\src\components/CanvasView.tsx` |
| ARG SRS (A11 主源) | 10 类关系 + 4 维度 + 5 模板 | `D:\Star\docs\requirements\SRS-AGENT-RELATIONSHIP-001.md` v0.1 |
| V0.1 game 4 份 PHASE | Roguelike + Manga + Theme + Settings (GAMIFY G12 集成) | `D:\Star\docs\reports/PHASE-AGENT-{GAME,ROGUELIKE,MANGA,THEME,SETTINGS}-IMPL-REPORT.md` |
| 守门交叉引用 | 16 守门 + 26 派生规 | `D:\Star\AGENTS.md` §4 |
| 拍板 4 阶段历史 | 17:00 + 17:08 + 17:21 + 17:34 JST (含 v0.63 反转) | 总册 §9.3 跨拍板派生 |
| Token OLU 估算 | 1 SRE·周 ≈ 1.2M tokens (per STAR-OLU-001 v0.1) | `D:\Star\docs/STAR-OLU-001.md` v0.1 |

### 1.3 文档结构 (per BD-AGENT-RELATIONSHIP-001 v0.1 模板, 10 段)

```
§0 目的 (Purpose)
§1 适用范围 (Scope) — 1.1 In-Scope (5 子段: 数据层/同步桥/UI 层/成就引擎/4 维协作影响) + 1.2 Out-of-Scope (Miro 通用 12 类砍掉)
§2 系统架构 (System Architecture) — 5 view 跨域 (機能/データ/動作/モジュール/ネットワーク)
§3 组件一览 (Components) — 新增组件 / 复用组件 / 模块划分
§4 数据模型 (Data Model) — 14 张表 W/T/M + Memgraph schema + LangGraph state schema + zustand store 扩展
§5 接口设计 (Interface Design) — BFF REST API 32 端点 + WebSocket 5 端点 + 内部 5 协议
§6 5 view 詳細 (5 views) — 機能 view (跨域 FR) + データ view (14 表) + 動作 view (5 域) + モジュール view (4 文件 + 6 crate) + ネットワーク view (e2e)
§7 NFR (Non-Functional Requirements) — 性能/可靠性/安全/易用/成就/可观测 6 类
§8 守门 (Guards) + 子代理失败接手 + 已知缺口 (12 跨域 + 3 P0 阻塞)
§9 签字栏 (5 角色 per AGENTS.md §3: 架构师/SRE Lead/平台/评审主持/PM)
§10 修订履历 (v0.1 + 修订人 + 触发)
```

### 1.4 输出文件

| 文件 | 内容 | 预估大小 |
|---|---|---|
| `D:\Star\docs\design\BD-CANVAS-001.md` | 总册 BD 完整 10 段 | ~25-35K 字 |

**仅输出 1 份文件**, 不拆 commit, 不写 report, 不动 implementation.

## 2. 范围外 (out-of-scope, 由其他子代理 / 专题 BD / root 处理)

| 类别 | 处理方 |
|---|---|
| 双核心之 1: agent 管理 46 项 详细 BD | 专题 BD 子代理 1 (bd-canvas-agent-001) |
| 双核心之 2: 游戏化 32 项 详细 BD | 专题 BD 子代理 2 (bd-canvas-gamify-001) |
| 详细设计 (DD) 文档 | 后续 P3-D 阶段, 待 SRS + BD 落档后启动 |
| 实现 (PHASE-* 报告) | 后续 P3-D.6 阶段, 待 DD 落档后启动 |
| Miro 通用 12 类 (12 diagram / 模板 / 集成 / 移动 / a11y) | ❌ 砍掉, 留 P3+ 评估 |
| 25 module 实体实现 | 25 module 各自 docs, 画布只联动不实装 |
| 5 域 Lead 真人到位 | per 守门 #14 v2, Mavis 临时代签, 真人到位后追溯签字 |

## 3. 已知缺口 (per 守门 #11 缺标比错标)

本 BD 须在 §8 显式列已知缺口 (≥ 8 个, 跨域), 不得隐藏:
- **多人编辑 WebSocket 选型未拍板** (per AGENT A12.1, P0 阻塞, 候选 NATS JetStream / native WebSocket / Socket.IO) → §8
- **冲突解决 CRDT 选型未拍板** (per AGENT A12.6, P0 阻塞, 候选 Yjs / Automerge / LWW) → §8
- **view/comment/edit 3 级权限矩阵** (per AGENT A12.7, 5 域 Lead 真人到位后决策, 拍板前 view-only 兜底) → §8
- **V0.1 canvas + 双核心 + ARG 跨域 schema 协调** (V0.1 本地 zustand persist vs 新 backend 持久化层, P0 阻塞) → §8
- **5 域 Lead 真人未到位** (per 守门 #14 v2, Mavis 临时代签, 真人到位后追溯签字 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D) → §8
- **ARG 5 维度 effect tier 模块实装待 P3-C 阶段** (per AGENT A11, per `SRS-AGENT-RELATIONSHIP-001.md` §1.3) → §8
- **Memgraph 部署** (Docker 启动 port 7687 Bolt + 7444 HTTP, 数据卷持久化, per `SRS-AGENT-RELATIONSHIP-001.md` §3.2 PR-4) → §8
- **L0↔L1 通信协议** (per `SRS-STAR-AGENT-RUNTIME-001.md` §4.4, 跨 5 域 Lead 责任边界) → §8
- **TMO 9 节点 vs ARG 边界梳理** (per `SRS-AGENT-RELATIONSHIP-001.md` §5.3, 不取代 LangGraph 任务卡 DAG, 平行层) → §8
- **3 commit 跨域 v0.62 反转** (per 2026-09-10 12:45 JST, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses) → §8
- **守门 #1 v15 docs 同步饱和** (per 4 commit 累计 1.46M tokens, 第 68-71 次新事件触发仍允许, 后续 BD + DD + 实装需新事件触发) → §8
- **守门 #23 v2 AI 第三方 API 禁止** (per GAMIFY G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2, 跨域硬约束) → §8

## 4. 守门硬约束 (per 守门 #1 + 守门 #9 #3 + 守门 #13 + 守门 #14 + 守门 #15)

- 文档结构严格 10 段 (per BD-AGENT-RELATIONSHIP-001 模板), 不增不减
- 总册 BD 跨域覆盖, 不重复 2 专题 BD 内容, 仅汇总 + 跨域接口
- 5 view 跨域 (機能/データ/動作/モジュール/ネットワーク) 完整覆盖
- 14 张表 W/T/M 100% 覆盖 (per 守门 #13) — A11 7 张 + A12 7 张跨域汇总
- 32 API 端点 + 5 WebSocket 跨域汇总
- 已知缺口 ≥ 8 个 (含 3 P0 阻塞, 跨域显式列)
- NFR 6 类 (性能/可靠性/安全/易用/成就/可观测) 跨域共享基线
- 守门 19 项 + 26 派生规 (per AGENTS.md §4) 跨域汇总 + v3x 候选
- 5 角色签字栏 per AGENTS.md §3 (架构师 / SRE Lead / 平台 / 评审主持 / PM)
- **5 域 Lead ≠ Star 22 DDD bounded context** disclaimer 显式 (per 2026-08-31 22:45 JST Q1-D 拍板)
- **5 域 Lead 真人未到位前 Mavis 临时代签, 真人到位后追溯签字** (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
- **AI mock 接口** (per 守门 #23 v2, GAMIFY G5 走 mock, 真实 LLM 留 P2, 跨域硬约束)
- **DB W/T/M 三類横展** (per 守门 #13, 14 张表 100% 覆盖, 禁止混在)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses)
- 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠, root 直实装)
- 0 文件改动除输出 BD
- 0 commit, 仅产出 markdown (root 统一 commit per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
- **不重写 4 SRS commit** (per 守门 #1 禁回溯叙事, BD 是新方向, 不回写 SRS)

## 5. 落地清单

| # | 文件 | 内容 | 行数预估 |
|---|---|---|---|
| 1 | `D:\Star\docs\design\BD-CANVAS-001.md` | 10 段 BD, 总册跨域 | ~700-1000 行 |

预估 0 commit (root 统一 commit), 1 文件, ~25-35K 字。

## 6. 返报告知 (per 守门 #9 v27 collect_output)

root 撰写后, 报告必须含:
1. 实际写入文件路径 + 字节数
2. 10 段是否齐全
3. 5 view 覆盖检查 (機能/データ/動作/モジュール/ネットワーク)
4. 14 张表 W/T/M 100% 覆盖 (列出每张表归到 W/T/M 哪類)
5. 32 API 端点 + 5 WebSocket 跨域汇总 (列表)
6. 已知缺口清单 (≥ 8 个, 含 3 P0 阻塞)
7. 跨专题引用 (引用 SRS-CANVAS-{AGENT v1.2, GAMIFY v1.0, 001 v1.1} + 现有 canvas design + 25 module + 守门 19 项 + 拍板 4 阶段)
8. 守门 19/19 跨域覆盖 (含 #1 v15 / #9 #3 / #9 v19 / #10 / #11 / #13 / #14 v2/v3/v4 / #15 / #23 v2)
9. 任何意外 / 偏离 / 简化 / 跳过 项, 显式标注

## 7. 元数据 (per AGENTS.md §3 守门 #10 + 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 + 9/10 12:45 JST v0.62 反转)

- 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手`
- 审批: `架构师 (Mavis 接手 agent per DEC-008)` (5 角色签字栏 per AGENTS.md §3)
- 日期: 2026-09-10 JST
- 关联 commit: 留空 (root 统一 commit 时填)
- 关联文档: 3 份 SRS v1.x + 现有 frontend-canvas-design.md v0.1 + 现有 V0.1 game 4 份 + 2 份平行专题 BD
- 拍板来源: 2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" + 17:34 JST v0.63 反转多人编辑

## 8. 起点

读完 13 份必读后, 用 Write 工具写 `D:\Star\docs\design\BD-CANVAS-001.md` (10 段, root 直实装).
