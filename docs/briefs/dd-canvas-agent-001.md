# Brief: dd-canvas-agent-001

**Agent**: worker (root 派发, 2 子代理并行之 1/2 per 守门 #9 v20 + v27 3 段 fallback)
**Phase**: P3-D.5 DD 详细设计 (18:25 JST Ulysses 拍板"完善详细设计文档")
**Created**: 2026-09-10 18:27 JST
**Token 预算**: ~0.5M (守门 #4 / #19 估算, 1 SRE·周 = 1.2M 留 2.4x 缓冲)
**Worktree**: 在 root 当前 main worktree 直实装 (per 守门 #9 #3 实证 5/5 RPC 不可靠, 不派二级子代理)

---

## 0. 触发

2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档" — 基于 BD-CANVAS-AGENT-001.md v0.1 (105KB, 46 项 A1-A12 详细设计) + SRS-CANVAS-AGENT-001.md v1.2 (155KB) 制作 agent 管理域详细设计 DD.

撤回 17:08 JST 砍多人编辑决定 (per 17:34 JST v0.63 反转) + 17:21 JST 拍板 ARG 图论构造 (A11 10 项).

## 1. 范围 (in-scope)

### 1.1 本专题 DD 覆盖 — Agent 管理域 (双核心之 1)

**核心**: A1-A12 (12 子能力, 46 项) 的**详细实现** (Rust struct + TS interface + SQL DDL + 4 时序图 + 74 测试 + 5 状态机 + 11 共享类型 + 13 关键 class).

| 子能力 | 来源 BD | 详细实现 |
|---|---|---|
| A1 agent 节点 | BD §4.1 | `AgentNode` struct + `agent_node` React 组件 + StatusPill 60+ 映射 + 双击路由 |
| A2 agent 拓扑 | BD §4.2 | `AgentTopology` struct + bezier connector + 5 域分组 + 父子 + pipeline |
| A3 agent 状态 | BD §4.3 | `AgentStatus` enum (14 状态) + WS sync + audit + notification |
| A4 worktree | BD §4.4 | `AgentWorktreeRelation` + status 联动 |
| A5 work-item | BD §4.5 | `AgentWorkItemRelation` + status 联动 |
| A6 操作菜单 | BD §4.6 | `AgentOperation` (start/stop/restart) + permission (5 域 Lead 决策) |
| A7 监控面板 | BD §4.7 | `AgentMonitor` (token/cost/runtime) + budget 对比 + 告警 |
| A8 聚类/排序/过滤 | BD §4.8 | `AgentCluster` 算法 + sort/filter |
| A9 跨域引用 | BD §4.9 | 跟 SRS-AGENT-VIEW-001 / SRS-AGENT-RELATIONSHIP-001 协同 |
| A10 settings | BD §4.10 | 跟 V0.1 AgentSettingsTab 集成 |
| A11 ARG | BD §4.11 | 10 类关系 + 4 维度 + 5 模板 + 同步桥 + 成就 (派生自 DD-AGENT-RELATIONSHIP-001 v0.1 13 关键 class) |
| A12 多人编辑 | BD §4.12 (per 17:34 JST v0.63 反转) | 多人 + cursor + 增删改 + Follow + 评论 + @ + CRDT + audit (派生自 `frontend-canvas-design.md` v0.1 §4.1 + §4.6) |
| **合计** | | | **46 项详细实现** |

### 1.2 引用 baseline (必读, 不能编造)

| 文档 | 用途 | 路径 |
|---|---|---|
| **派生源 BD (核心)** | A1-A12 46 项基本设计 | `D:\Star\docs\design\BD-CANVAS-AGENT-001.md` v0.1 (105KB) |
| **派生源 SRS** | A1-A12 46 项需求 + 14 张表 W/T/M | `D:\Star\docs\requirements\SRS-CANVAS-AGENT-001.md` v1.2 (155KB) |
| **DD 模板 (15 章节, 13 关键 class + 4 effect + 5 状态机 + 11 共享类型 + 4 时序图 + 8 Cypher + 10 challenges prompt + 74 测试 模板)** | **必读, 严格按 15 章节 + 10 段** | `D:\Star\docs\design\DD-AGENT-RELATIONSHIP-001.md` v0.1 (94KB, 9/9 落档) |
| **A11 ARG 派生 (最详细源)** | 13 关键 class + 4 effect + 5 状态机 + 11 共享类型 + 4 时序图 | DD-AGENT-RELATIONSHIP-001 v0.1 §3-§10 (94KB) |
| **总册 SRS + 总册 BD (root 写)** | 跨域接口 + 共享约束 | `D:\Star\docs/requirements/SRS-CANVAS-001.md` v1.1 + `D:\Star\docs/design/BD-CANVAS-001.md` v0.1 (本批 root 写) |
| **平行 DD 参照** | 5 view + 跨块接口 | `D:\Star\docs\design\DD-AGENT-VIEW-001.md` (待补) |
| **A12 多人编辑派生源** | V0.1 §4.1 模式 A Realtime 通道 + §4.6 PresenceCursor | `D:\Star\docs\frontend-canvas-design.md` v0.1 |
| **V0.1 实装代码** | CanvasView.tsx 11 处 element + tool + minimap | `D:\Star\frontend/src/components/CanvasView.tsx` |
| **V0.1 game 4 份 PHASE** (A10 + G12 集成) | 5 份实装报告 | `D:\Star\docs\reports\PHASE-AGENT-{SETTINGS,GAME,ROGUELIKE,MANGA,THEME}-IMPL-REPORT.md` |
| **SRS-STAR-AGENT-RUNTIME-001.md v1.0** | 14 状态机 + 9 SA Archetype | 53KB |
| **SRS-AGENT-VIEW-001.md v1.0** | A9 跨域引用 | 31KB |
| **SRS-AGENT-RELATIONSHIP-001.md v0.1** | 10 类关系 + 4 维度 + 5 模板 + 4 表 + 8 UC | 37KB |
| **Ulysses 18:25 JST 拍板原文** | "完善详细设计文档" — 这是核心方向, **不可偏离** |
| **Ulysses 17:34 JST v0.63 反转** | "多人编辑是要的" — A12 8 项必含, 撤回 17:08 JST 砍多人编辑决定 |

### 1.3 文档结构 (per DD-AGENT-RELATIONSHIP-001 v0.1 模板, 10 段 + 15 章节)

```
§0 文档信息 / 修订履历
§1 文档目的 / 适用范围
§2 系统架构 (System Architecture) — 5 view 跨域 + 5-tier + 4 sequence diagram
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
附录 C: 已知缺口 (19 个 per BD §8.3, 含 3 P0 阻塞, DDD Review 必查)
附录 D: 5 角色签字栏 (per AGENTS.md §3)
附录 E: 修订履历 (v0.1 + 修订人 + 触发)
```

### 1.4 输出文件

| 文件 | 内容 | 预估大小 |
|---|---|---|
| `D:\Star\docs\design\DD-CANVAS-AGENT-001.md` | 专题 DD 完整 10 段 + 5 附录, A1-A12 46 项详细 | ~80-110K 字 |

**仅输出 1 份文件**, 不拆 commit, 不写 report, 不动 implementation.

## 2. 范围外 (out-of-scope, 由其他子代理 / 总册 DD / root 处理)

| 类别 | 处理方 |
|---|---|
| 双核心之 2: 游戏化 32 项 详细 DD | 专题 DD 子代理 2 (dd-canvas-gamify-001) |
| 总册 DD (跨域共享部分) | root 写 (dd-canvas-total-001) |
| 实现 (PHASE-* 报告) | 后续 P3-D.6 阶段, 待 DD 落档后启动 |
| 单元测试 (UT) 代码实装 | P3-D.6 阶段 |
| E2E 测试 (Playwright) 实装 | P3-D.6 阶段 |
| Miro 通用 12 类 | ❌ 砍掉, 留 P3+ 评估 |
| 25 module 实体实现 | 25 module 各自 docs, 画布只联动不实装 |

## 3. 已知缺口 (per 守门 #11 缺标比错标)

本 DD 须在附录 C 显式列已知缺口 (≥ 19 个, per BD AGENT §8.3 19 缺口), 不得隐藏:
- **A12.1 多人同时编辑 WebSocket 选型未拍板** (P0 阻塞)
- **A12.2 PresenceCursor V0.1 design 已落档, A12 实装扩展待 P3-C/D 阶段**
- **A12.3 V0.1 localStorage + zustand persist 跟多人编辑冲突**
- **A12.5 V0.1 comment_pin 渲染保留, thread + @ 数据结构 + 25 module notification 域对接实装**
- **A12.6 CRDT 选型未拍板** (P0 阻塞)
- **A12.7 具体权限矩阵待 5 域 Lead 真人到位后决策**
- **A12 4 跨 session 续做 P0 阻塞总结**
- **A1 avatar 字段**
- **A2 domain / parent_session_id / pipeline_agent_ids 字段**
- **A3 WebSocket 选型 (跟 A12 同源)**
- **A6 5 域 Lead 真人未到位**
- **A7 token_budget 字段**
- **A9 /agent-relationships 路由**
- **A11 arg_edge kind**
- **A11 5 维度 effect tier 模块实装待 P3-C 阶段**
- **A11 跨 session 续做 (5 域 Lead + Memgraph + L0↔L1 + TMO)**
- **A19 store 持久化 (跟 A12.3 同源)**
- **DDD Review 必查 7 缺口**

## 4. 守门硬约束 (per 守门 #1 + 守门 #9 #3 + 守门 #13 + 守门 #14 + 守门 #15 + 守门 #23 v2)

- 文档结构严格 10 段 + 5 附录 (per DD-AGENT-RELATIONSHIP-001 模板), 不增不减
- 46 项每项 5 view + 详细实现 (Rust struct + TS interface + SQL DDL)
- 已知缺口 ≥ 19 个 (含 3 P0 阻塞, 跨域)
- 14 张表 W/T/M 100% 覆盖 (SQL DDL 完整, 禁止混在)
- 23 API 端点 + 5 WebSocket 完整 spec
- 13 关键 class 跨域 (C-1/C-2/C-3/C-4/C-7/C-8/C-11/C-12/C-13/C-14/C-15/C-21/C-16, P3-D 落地)
- 5 状态机 (Edge / Agent / Trust Score 5 档 / Template Instance / Achievement)
- 11 共享类型 (ARGEvent / Decision / Output / Verdict / LLMClient / AchievementUnlock / TemplateInstance / ARGState / EscalationInfo / PeerReviewVerdict / ChallengePrompt)
- 4 关键时序图 (写关系 / 协作影响 / 成就评估 / 离线降级)
- 74+ 测试用例 (UT + IT + E2E + PT, 跨域 ≥ 30)
- 守门 19 项 + 26 派生规 跨域 (含 #23 v2)
- 5 角色签字栏 per AGENTS.md §3
- **5 域 Lead ≠ Star 22 DDD bounded context** disclaimer 显式
- **5 域 Lead 真人未到位前 Mavis 临时代签, 真人到位后追溯签字** (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
- **AI mock 接口** (per 守门 #23 v2, GAMIFY G5 走 mock)
- **DB W/T/M 三類横展** (per 守门 #13, 14 张表 100% 覆盖)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 + #14 v4)
- 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
- 0 文件改动除输出 DD
- 0 commit, 仅产出 markdown (root 统一 commit)
- **不重写 4 SRS + 2 BD 6 commit** (per 守门 #1 禁回溯叙事, DD 是新方向)
- **A11 必须派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1** (13 关键 class + 4 effect + 5 状态机 + 11 共享类型 + 4 时序图, 1:1 映射)
- **A12 必须派生自 `frontend-canvas-design.md` v0.1 §4.1 模式 A Realtime 通道 + §4.6 PresenceCursor 升级**

## 5. 落地清单

| # | 文件 | 内容 | 行数预估 |
|---|---|---|---|
| 1 | `D:\Star\docs\design\DD-CANVAS-AGENT-001.md` | 10 段 + 5 附录, A1-A12 46 项详细 | ~1500-2200 行 |

预估 0 commit (root 统一 commit), 1 文件, ~80-110K 字。

## 6. 返报告知 (per 守门 #9 v27 collect_output)

子代理返回时, 报告必须含:
1. 实际写入文件路径 + 字节数
2. 10 段 + 5 附录是否齐全
3. 5 view 详细实现覆盖检查 (機能/データ/動作/モジュール/ネットワーク)
4. **14 张表 SQL DDL 完整** (列出每张表的 W/T/M 归类)
5. 23 API 端点 + 5 WebSocket OpenAPI spec 完整 (列表)
6. **13 关键 class 跨域** (列出 class 名 + 字段数 + 方法数)
7. **5 状态机** (列出每个状态机的状态数 + 转移函数)
8. **11 共享类型** (列出每个类型定义)
9. **4 关键时序图** (写关系 / 协作影响 / 成就评估 / 离线降级, 每个 5-10 步骤)
10. 已知缺口清单 (≥ 19 个, 含 3 P0 阻塞)
11. 跨专题引用 (引用了哪些 SRS / BD / DD / 设计 / 25 module / 守门 / 拍板, 具体 §)
12. **A11 1:1 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1** (13 关键 class + 4 effect + 5 状态机 + 11 共享类型 + 4 时序图)
13. **A12 1:1 派生自 `frontend-canvas-design.md` v0.1 §4.1 + §4.6**
14. 守门 19/19 跨域覆盖 (含 #1 v15 / #9 #3 / #9 v19 / #10 / #11 / #13 / #14 v2/v3/v4 / #15 / #23 v2)
15. 任何意外 / 偏离 / 简化 / 跳过 项, 显式标注

不要只回 "done" — 必须给可验证证据.

## 7. 元数据 (per AGENTS.md §3 守门 #10 + 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 + 9/10 12:45 JST v0.62 反转 + 17:34 JST v0.63 反转)

- 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手`
- 审批: `架构师 (Mavis 接手 agent per DEC-008)` (5 角色签字栏 per AGENTS.md §3)
- 日期: 2026-09-10 JST
- 关联 commit: 留空 (root 统一 commit 时填)
- 关联文档: BD-CANVAS-AGENT-001.md v0.1 (本 DD 派生源) + 2 份平行 SRS + 2 份平行 BD + 现有 canvas design + DD-AGENT-RELATIONSHIP-001 v0.1 (A11 模板)
- 修订履历必须含 v0.1 (2026-09-10 18:25 JST 拍板"完善详细设计文档" + 17:34 JST v0.63 反转多人编辑 + 17:21 JST ARG 图论构造 + 17:08 JST 双核心), 显式标 5 阶段拍板

## 8. 起点

读完 15 份必读后, 用 Write 工具写 `D:\Star\docs\design\DD-CANVAS-AGENT-001.md` (10 段 + 5 附录, 全量覆盖).
