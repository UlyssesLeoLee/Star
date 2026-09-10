# Brief: dd-canvas-total-001

**Agent**: root session (Mavis 接手, 不派子代理 per 守门 #9 #3 实证 5/5 RPC 不可靠, root 直实装 DD 总册)
**Phase**: P3-D.5 DD (详细设计, 18:25 JST Ulysses 拍板"完善详细设计文档")
**Created**: 2026-09-10 18:27 JST
**Token 预算**: ~0.15M (守门 #4 / #19 估算, root 直实装, DD 总册比专题 DD 短, 比 BD 总册长 ~20%)
**Worktree**: 在 root 当前 main worktree 直实装 (per 守门 #9 #3 实证 5/5 RPC 不可靠)

---

## 0. 触发

2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档" — 基于 3 份 SRS (SRS-CANVAS-001 v1.1 + AGENT v1.2 + GAMIFY v1.0, 累计 4 commit / 1.46M tokens) + 3 份 BD (BD-CANVAS-001 v0.1 + AGENT-001 v0.1 + GAMIFY-001 v0.1, 累计 2 commit / 4194 insertions) 制作 3 份 DD, 按 SRS + BD 1 总册 + 2 专题 拆分模式.

## 1. 范围 (in-scope)

### 1.1 本 DD 覆盖 — 总册详细设计 (跨域共享部分)

**核心**: 5 view 跨域详细实现 + 14 张表 SQL DDL + 32 API 端点 spec + 5 WebSocket protocol + 14 张表 W/T/M 100% 覆盖 + 6 类 NFR benchmark + 19 守门 + 26 派生规.

| 子能力 | 来源 BD | 内容 |
|---|---|---|
| 系统架构 (5 view 跨域) | BD-CANVAS-001 §2 | 5 view 跨域 + 5-tier 架构 + 5 跨域交互模式 |
| 概念 module 布局 | BD-CANVAS-001 §3 | 4 文件 (本批 DD) + 6 crate (A11) + V0.1 复用 5 组件 |
| 数据模型 (14 张表 W/T/M) | BD-CANVAS-001 §4 | 14 张表 SQL DDL + Rust struct + zustand store 扩展 + 29 张表跨域汇总 |
| 接口设计 (32 API + 5 WS) | BD-CANVAS-001 §5 | 32 REST 端点 + 5 WebSocket + 内部 5 协议 + 错误处理 |
| 5 view 詳細 | BD-CANVAS-001 §6 | 機能/データ/動作/モジュール/ネットワーク 5 view 跨域详细实现 |
| NFR benchmark | BD-CANVAS-001 §7 | 性能/可靠性/安全/易用/可观测/ARG 6 类 benchmark |
| 守门 + 已知缺口 | BD-CANVAS-001 §8 | 19 守门 + 26 派生规 + 12 已知缺口 (含 3 P0 阻塞) |
| 25 module 联动 | BD-CANVAS-001 §6.3 | 9 类 25 module (work-item/worktree/agent/relation/comment/automation/audit/search/notification) + V0.1 game 4 份 |
| 双核心 1 (管理 agent) | BD-CANVAS-AGENT-001 (A1-A12 46 项) | 派生自专题 1 DD |
| 双核心 2 (游戏化) | BD-CANVAS-GAMIFY-001 (G1-G12 32 项) | 派生自专题 2 DD |
| 多人编辑 (v0.63 反转) | BD-CANVAS-AGENT-001 §4.12 + frontend-canvas-design.md §4.1+§4.6 | 派生自专题 1 DD |
| ARG 图论构造 | BD-CANVAS-AGENT-001 §4.11 + DD-AGENT-RELATIONSHIP-001 v0.1 (94KB, 13 关键 class) | 派生自专题 1 DD + ARG DD 模板 |

### 1.2 引用 baseline (必读, 不能编造)

| 文档 | 用途 | 路径 |
|---|---|---|
| 总册 SRS | 顶层需求 + 双核心索引 + 跨块接口 | `D:\Star\docs\requirements\SRS-CANVAS-001.md` v1.1 (58KB) |
| 总册 BD (本批派生) | 5 view 跨域 + 5-tier 架构 + 14 张表 W/T/M 100% 覆盖 | `D:\Star\docs\design\BD-CANVAS-001.md` v0.1 (50KB) |
| 专题 SRS (双核心之 1) | agent 管理 46 项 + A11 ARG + A12 多人编辑 | `D:\Star\docs\requirements\SRS-CANVAS-AGENT-001.md` v1.2 (155KB) |
| 专题 BD (双核心之 1) | agent 管理 46 项详细设计 | `D:\Star\docs\design\BD-CANVAS-AGENT-001.md` v0.1 (105KB) |
| 专题 SRS (双核心之 2) | 游戏化 32 项 + G11 W/T/M + G5 AI mock | `D:\Star\docs\requirements\SRS-CANVAS-GAMIFY-001.md` v1.0 (92KB) |
| 专题 BD (双核心之 2) | 游戏化 32 项详细设计 | `D:\Star\docs\design\BD-CANVAS-GAMIFY-001.md` v0.1 (111KB) |
| **DD 模板 (15 章节)** | 严格按 15 章节 + 10 段 IPA SEC 模板 | `D:\Star\docs\design\DD-AGENT-RELATIONSHIP-001.md` v0.1 (94KB, 9/9 落档, 13 关键 class + 4 effect + 5 状态机 + 4 时序图 + 11 共享类型 + 8 Cypher + 10 challenges prompt + 74 测试 模板) |
| 平行 DD 参照 (agent view) | 5 view + 数据模型 + 5 域 跨域 | `D:\Star\docs\design\DD-AGENT-VIEW-001.md` (待补, 9/5 落档) |
| V0.1 canvas design | 现状基线 (14 element + 4 frame + 8 connector + 9 e2e 守门) | `D:\Star\docs\frontend-canvas-design.md` v0.1 |
| V0.1 实装代码 | CanvasView.tsx 11 处 element 渲染 + 工具栏 + minimap | `D:\Star\frontend\src/components/CanvasView.tsx` |
| ARG SRS (A11 主源) | 10 类关系 + 4 维度 + 5 模板 | `D:\Star\docs\requirements\SRS-AGENT-RELATIONSHIP-001.md` v0.1 |
| V0.1 game 4 份 PHASE | Roguelike + Manga + Theme + Settings (GAMIFY G12 集成) | `D:\Star\docs/reports/PHASE-AGENT-{GAME,ROGUELIKE,MANGA,THEME,SETTINGS}-IMPL-REPORT.md` |
| 守门交叉引用 | 19 守门 + 26 派生规 | `D:\Star\AGENTS.md` §4 |
| 拍板 5 阶段历史 | 17:00 + 17:08 + 17:21 + 17:34 + 18:00 JST | 总册 BD §10.1 跨拍板派生 |
| Token OLU 估算 | 1 SRE·周 ≈ 1.2M tokens (per STAR-OLU-001 v0.1) | `D:\Star\docs/STAR-OLU-001.md` v0.1 |

### 1.3 文档结构 (per DD-AGENT-RELATIONSHIP-001 v0.1 模板, 10 段 + 15 章节)

```
§0 文档信息 / 修订履历
§1 文档目的 / 适用范围 — 1.1 文档目的 + 1.2 In-Scope + 1.3 Out-of-Scope
§2 系统架构 (System Architecture) — 5 view 跨域 + 5-tier + 5 跨域交互模式 + 4 sequence diagram
§3 概念 module 布局 (Conceptual Module Layout) — 24 组件 → 18 Rust module + 1 Python LangGraph module + 跨域组件映射
§4 关键 class (Key Classes) — 13 关键 class 完整字段 + 方法签名 + 错误处理 (C-1..C-24, P3-D 落地 C-1/C-2/C-3/C-4/C-7/C-8/C-11/C-12/C-13/C-14/C-15/C-21/C-16)
§5 状态机 (State Machines) — 5 状态机 Rust enum + 状态转移函数 (Edge / Agent / Trust Score 5 档 / Template Instance / Achievement)
§6 共享类型 (Shared Types) — 11 共享类型完整定义 (ARGEvent / Decision / Output / Verdict / LLMClient / AchievementUnlock / TemplateInstance / ARGState / EscalationInfo / PeerReviewVerdict / ChallengePrompt)
§7 接口协议 (Interface Protocols) — 5 WebSocket 协议 + 内部 5 协议 + 32 API 端点 OpenAPI spec + 错误处理 6 类
§8 时序图 (Sequence Diagrams) — 4 关键时序图 (写关系 / 协作影响 / 成就评估 / 离线降级)
§9 数据持久化 (Data Persistence) — 14 张表 SQL DDL + Memgraph Cypher schema + zustand store 扩展
§10 测试用例 (Test Cases) — UT + IT + E2E + PT 4 类测试, 跨域测试 ≥ 30 个
附录 A: 跨专题引用清单
附录 B: 跨域组件映射 (4 文件 DD + 6 crate + 5 V0.1 复用 + 25 module 联动)
附录 C: 已知缺口 (12 个 per BD §8.3, 含 3 P0 阻塞, DDD Review 必查)
附录 D: 5 角色签字栏 (per AGENTS.md §3)
附录 E: 修订履历 (v0.1 + 修订人 + 触发)
```

### 1.4 输出文件

| 文件 | 内容 | 预估大小 |
|---|---|---|
| `D:\Star\docs\design\DD-CANVAS-001.md` | 总册 DD 完整 10 段 + 5 附录 | ~60-80K 字 |

**仅输出 1 份文件**, 不拆 commit, 不写 report, 不动 implementation.

## 2. 范围外 (out-of-scope, 由其他子代理 / 专题 DD / root 处理)

| 类别 | 处理方 |
|---|---|
| 双核心之 1: agent 管理 46 项 详细 DD | 专题 DD 子代理 1 (dd-canvas-agent-001) |
| 双核心之 2: 游戏化 32 项 详细 DD | 专题 DD 子代理 2 (dd-canvas-gamify-001) |
| 实现 (PHASE-* 报告) | 后续 P3-D.6 阶段, 待 DD 落档后启动 |
| 单元测试 (UT) 代码实装 | P3-D.6 阶段 |
| E2E 测试 (Playwright) 实装 | P3-D.6 阶段 |
| 性能测试 (PT) 脚本实装 | P3-D.6 阶段 |
| Miro 通用 12 类 (12 diagram / 模板 / 集成 / 移动 / a11y) | ❌ 砍掉, 留 P3+ 评估 (per 17:08 JST) |
| 25 module 实体实现 | 25 module 各自 docs, 画布只联动不实装 |

## 3. 已知缺口 (per 守门 #11 缺标比错标)

本 DD 须在附录 C 显式列已知缺口 (≥ 12 个, 跨域), 不得隐藏:
- **多人编辑 WebSocket 选型未拍板** (per AGENT A12.1, P0 阻塞, 候选 NATS JetStream / native WebSocket / Socket.IO) → 附录 C
- **冲突解决 CRDT 选型未拍板** (per AGENT A12.6, P0 阻塞, 候选 Yjs / Automerge / LWW) → 附录 C
- **view/comment/edit 3 级权限矩阵** (per AGENT A12.7, 5 域 Lead 真人到位后决策, 拍板前 view-only 兜底) → 附录 C
- **V0.1 canvas + 双核心 + ARG 跨域 schema 协调** (V0.1 本地 zustand persist vs 新 backend 持久化层, P0 阻塞) → 附录 C
- **5 域 Lead 真人未到位** (per 守门 #14 v2, Mavis 临时代签, 真人到位后追溯签字 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D) → 附录 C
- **ARG 5 维度 effect tier 模块实装待 P3-C 阶段** (per AGENT A11, per `SRS-AGENT-RELATIONSHIP-001.md` §1.3) → 附录 C
- **Memgraph 部署** (Docker 启动 port 7687 Bolt + 7444 HTTP, 数据卷持久化, per `SRS-AGENT-RELATIONSHIP-001.md` §3.2 PR-4) → 附录 C
- **L0↔L1 通信协议** (per `SRS-STAR-AGENT-RUNTIME-001.md` §4.4, 跨 5 域 Lead 责任边界) → 附录 C
- **TMO 9 节点 vs ARG 边界梳理** (per `SRS-AGENT-RELATIONSHIP-001.md` §5.3, 不取代 LangGraph 任务卡 DAG, 平行层) → 附录 C
- **3 commit 跨域 v0.62 反转** (per 2026-09-10 12:45 JST, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses) → 附录 C
- **守门 #1 v15 docs 同步饱和** (per 6 commit 累计 ~3.01M tokens, 第 68-73 次新事件触发仍允许, 后续实装需新事件触发) → 附录 C
- **守门 #23 v2 AI 第三方 API 禁止** (per GAMIFY G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2, 跨域硬约束) → 附录 C

## 4. 守门硬约束 (per 守门 #1 + 守门 #9 #3 + 守门 #13 + 守门 #14 + 守门 #15 + 守门 #23 v2)

- 文档结构严格 10 段 + 5 附录 (per DD-AGENT-RELATIONSHIP-001 模板), 不增不减
- 总册 DD 跨域覆盖, 不重复 2 专题 DD 内容, 仅汇总 + 跨域接口
- 5 view 跨域 (機能/データ/動作/モジュール/ネットワーク) 详细实现
- 14 张表 SQL DDL 完整 (per 守门 #13 W/T/M 100% 覆盖, 禁止混在)
- 32 API 端点 + 5 WebSocket OpenAPI spec 完整
- 13 关键 class 跨域 (C-1..C-24, P3-D 落地 C-1/C-2/C-3/C-4/C-7/C-8/C-11/C-12/C-13/C-14/C-15/C-21/C-16)
- 5 状态机 Rust enum + 状态转移函数完整
- 11 共享类型完整定义
- 4 关键时序图 (写关系 / 协作影响 / 成就评估 / 离线降级)
- 已知缺口 ≥ 12 个 (含 3 P0 阻塞, 跨域显式列)
- NFR 6 类 benchmark (性能/可靠性/安全/易用/可观测/ARG)
- 守门 19 项 + 26 派生规 (per AGENTS.md §4) 跨域汇总
- 5 角色签字栏 per AGENTS.md §3 (架构师 / SRE Lead / 平台 / 评审主持 / PM)
- **5 域 Lead ≠ Star 22 DDD bounded context** disclaimer 显式 (per 2026-08-31 22:45 JST Q1-D 拍板)
- **5 域 Lead 真人未到位前 Mavis 临时代签, 真人到位后追溯签字** (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
- **AI mock 接口** (per 守门 #23 v2, GAMIFY G5 走 mock, 真实 LLM 留 P2, 跨域硬约束)
- **DB W/T/M 三類横展** (per 守门 #13, 14 张表 100% 覆盖, 禁止混在)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses)
- 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠, root 直实装)
- 0 文件改动除输出 DD
- 0 commit, 仅产出 markdown (root 统一 commit per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
- **不重写 4 SRS + 2 BD 6 commit** (per 守门 #1 禁回溯叙事, DD 是新方向, 不回写 SRS + BD)
- **不写 PHASE-* 报告** (后续 P3-D.6 阶段)
- **不更新 automation-design.md / registry.md** (root 统一)

## 5. 落地清单

| # | 文件 | 内容 | 行数预估 |
|---|---|---|---|
| 1 | `D:\Star\docs\design\DD-CANVAS-001.md` | 10 段 DD + 5 附录, 总册跨域 | ~1000-1300 行 |

预估 0 commit (root 统一 commit), 1 文件, ~60-80K 字。

## 6. 返报告知 (per 守门 #9 v27 collect_output)

root 撰写后, 报告必须含:
1. 实际写入文件路径 + 字节数
2. 10 段 + 5 附录是否齐全
3. 5 view 详细实现覆盖检查 (機能/データ/動作/モジュール/ネットワーク)
4. 14 张表 SQL DDL 完整 (列出每张表的 W/T/M 归类)
5. 32 API 端点 + 5 WebSocket OpenAPI spec 跨域汇总
6. 13 关键 class 跨域 (列出 class 名 + 字段数 + 方法数)
7. 5 状态机 (列出每个状态机的状态数 + 转移函数)
8. 11 共享类型 (列出每个类型定义)
9. 4 关键时序图 (写关系 / 协作影响 / 成就评估 / 离线降级)
10. 已知缺口清单 (≥ 12 个, 含 3 P0 阻塞)
11. 跨专题引用 (引用 SRS-CANVAS-{AGENT v1.2, GAMIFY v1.0, 001 v1.1} + BD-CANVAS-{AGENT v0.1, GAMIFY v0.1, 001 v0.1} + 现有 canvas design + 25 module + DD-AGENT-RELATIONSHIP-001 v0.1 模板 + 守门 19 项 + 拍板 5 阶段)
12. 守门 19/19 跨域覆盖 (含 #1 v15 / #9 #3 / #9 v19 / #10 / #11 / #13 / #14 v2/v3/v4 / #15 / #23 v2)
13. 任何意外 / 偏离 / 简化 / 跳过 项, 显式标注

## 7. 元数据 (per AGENTS.md §3 守门 #10 + 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 + 9/10 12:45 JST v0.62 反转 + 18:00 JST 拍板"基于需求文档制作基本设计文档")

- 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手`
- 审批: `架构师 (Mavis 接手 agent per DEC-008)` (5 角色签字栏 per AGENTS.md §3)
- 日期: 2026-09-10 JST
- 关联 commit: 留空 (root 统一 commit 时填)
- 关联文档: 3 份 SRS v1.x + 3 份 BD v0.1 + 现有 frontend-canvas-design.md v0.1 + 现有 V0.1 game 4 份 + 2 份平行专题 DD + DD-AGENT-RELATIONSHIP-001 v0.1 模板
- 拍板来源: 2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档" + 18:00 JST BD 拍板 + 17:34 JST v0.63 反转多人编辑 + 17:21 JST ARG 图论构造 + 17:08 JST 双核心

## 8. 起点

读完 14 份必读后, 用 Write 工具写 `D:\Star\docs\design\DD-CANVAS-001.md` (10 段 + 5 附录, root 直实装).
