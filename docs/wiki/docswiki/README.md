---
title: 'Docswiki 设计拓扑索引'
date: 2026-09-06
source: 8 拓扑文件 + 7 设计源头 (S1-S7) + 152 节点笔记
status: obsidian-wiki-baseline
classification: obsidian-wiki
version: 0.2
revision: 'v0.2 @ 2026-09-06 Ulysses(per 19:39 JST)— Mavis 接手; v0.1 @ 2026-09-06 初版 (1 索引 + 7 拓扑)'
supersedes: null
in-topology: []
related: ["S1", "S2", "S3", "S4", "S5", "S6", "S7"]
see-also: ["00-design-topology"]
guards:
  - id: '#1'
    name: 0 unsafe + 0 err
    evidence: mermaid 语法自检, git 提交
  - id: '#3'
    name: 5 域独立 Lead / 3 view 平行
    evidence: View-AgentView / View-LangGraph / View-AgentRuntime [[wikilink]]
  - id: '#7'
    name: 0 unsafe
    evidence: 纯 markdown, 无代码
  - id: '#11'
    name: 缺标比错标
    evidence: 每图末「已知缺口」显式列
  - id: '#12'
    name: AI 協作文档治理
    evidence: 0 回溯叙事, 395+ 处 file:line 引用
  - id: '#19'
    name: agent 交互 Python 化
    evidence: scripts/automation/obsidian_topology_linkify.py
  - id: '#10'
    name: 代签规则应用
    evidence: author=Ulysses (per 19:39 JST 授权)
tags:
  - index
  - obsidian-wiki
  - design-topology
  - obsidian-wiki
  - design-topology
---

# 设计应有的工程拓扑 (Design Topology)

> **目的**: 把项目 4 个设计源头吃透后, 绘出"工程应有的拓扑", 用于后续跟实际工程实现的拓扑对比.
> **原则 (per 2026-09-06 拍板)**:
> - 100% 文档实证 — 每条边都 cite 一个文档位置 (file:line)
> - 缺标比错标安全 — 文档没写的宁可缺标, 不编
> - 拓扑 = 工程应有的视角, 不是文档层级结构
>
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权)
> **签批**: 🟢 Mavis 接手终审
> **日期**: 2026-09-06 JST

---

## 1. 设计源头 (Source Documents)

| # | 源头 | 路径 | 关键内容 |
|---|---|---|---|
| **[[S1]]** | Agent View 基本设计书 | [`docs/design/BD-AGENT-VIEW-001.md`](../../design/BD-AGENT-VIEW-001.md) | 3-tier UI / 10 模块 / 派生链 / 7 节点 画布 |
| **[[S2]]** | LangGraph view 基本设计 | [`docs/architecture/2026-09-03-langgraph/02-basic-design.md`](../../architecture/2026-09-03-langgraph/02-basic-design.md) | 2-level hierarchical + 22 组件 ([[C-01]]..[[C-22]]) + 7 TMO 节点 ([[M-N1]]..[[M-N7]]) + 9 SA 类型 |
| **[[S3]]** | LangGraph view 詳細設計 | [`docs/architecture/2026-09-03-langgraph/03-detailed-design.md`](../../architecture/2026-09-03-langgraph/03-detailed-design.md) | 25 模块 ([[M-01]]..[[M-25]]) + 7 subgraph + 7 TMO class + LangGraph 节点 [[M-N1]]..[[M-N7]] |
| **[[S4]]** | Agent Runtime 基本设计 | [`docs/architecture/2026-09-03-agent-runtime/02-basic-design.md`](../../architecture/2026-09-03-agent-runtime/02-basic-design.md) | L0 派发 + L1 ECS + L2 业务共享池 + 9 SA Archetype + 13 Systems + 31 domain-* 目标 |
| **[[S5]]** | Agent Runtime 詳細設計 | [`docs/architecture/2026-09-03-agent-runtime/03-detailed-design.md`](../../architecture/2026-09-03-agent-runtime/03-detailed-design.md) | 9 domain-* crate 新建 + ECS Component 12 类 + 状态机 + 9 Systems + DB schema 5 张表 |
| **[[S6]]** | 22 domain 接入顺序 | [`docs/architecture/2026-08-26-upgrade/spec/integration/01-22-domain-integration-spec.md`](../../architecture/2026-08-26-upgrade/spec/integration/01-22-[[domain-integration-spec]].md) | 6 Tier × 22 domain crate + 接入工作量估算 |
| **[[S7]]** | PostgreSQL Checkpointer Tier 3 | [`docs/architecture/2026-08-26-upgrade/adr/0047-postgresql-checkpointer-tier3.md`](../../architecture/2026-08-26-upgrade/adr/0047-postgresql-checkpointer-tier3.md) | 5 张表 schema + W/T/M 严格分类 + 12 Reducer 跨 Tier |

---

## 2. 拓扑图谱索引 (Topology Index)

| # | 文件 | 内容 | 节点数 | 关键引用 |
|---|---|---|---|---|
| **T0** | [`00-design-topology.md`](./00-design-topology.md) | **主图: 设计应有的工程总览** (UI / 业务编排 / 运行时 / 数据 / 平台) | 35-40 | [[S1]]-[[S7]] |
| **T1** | [`01-ui-agent-view.md`](./01-ui-agent-view.md) | Agent View 派生视图 (frontend/src/lib + components) | 15 | [[S1]] |
| **T2** | [`02-orchestration-langgraph.md`](./02-orchestration-langgraph.md) | LangGraph 后端 L0 + L1 + TMO + 9 SA + Cross-Cutting | 30-35 | [[S2]], [[S3]] |
| **T3** | [`03-runtime-ecs.md`](./03-runtime-ecs.md) | Agent Runtime L0 派发 + L1 ECS + L2 业务共享池 | 25-30 | [[S4]], [[S5]] |
| **T4** | [`04-domain-crates.md`](./04-[[domain-crates]].md) | 22 domain crate Tier 1-6 接入顺序 + 依赖 | 22 | [[S6]] |
| **T5** | [`05-persistence-checkpoint.md`](./05-persistence-checkpoint.md) | 3-tier checkpoint + 5 张表 W/T/M 分类 | 8-10 | [[S2]] §2.4, [[S7]] |
| **T6** | [`06-data-flow.md`](./06-data-flow.md) | 跨 view 关键数据流 (WS / REST / MCP / 事件) | 15-20 | [[S2]], [[S3]], [[S4]] |

---

## 3. 视图关系 (View Relationship)

per [AGENTS.md §6 架构 view 索引](../../AGENTS.md):

```
LangGraph view ([[S2]]/[[S3]])        Agent Runtime view ([[S4]]/[[S5]])        Agent View ([[S1]])
├── 9 SA Type ([[SA-01]]..[[SA-09]])  ├── L1 ECS 9 Archetype            ├── 派生画布
├── L0 全体代理 ([[T-N1]]..[[T-N7]])  ├── L0 派发 (Tokio + SQLite)      ├── zustand store
├── L1 任务卡子代理            ├── L2 业务共享池 (LLM/MCP/...)   ├── 7 派生纯函数
├── TMO 7 节点 ([[M-N1]]..[[M-N7]])   ├── 13 Systems                    ├── 3 类节点 (agent/wt/wi)
├── 22 组件 ([[C-01]]..[[C-22]])      ├── 12 ECS Components             ├── 7 FR + 6 EX
├── 25 模块 ([[M-01]]..[[M-25]])      ├── 9 domain-* 新建                └── 10 modules
└── 7 protocols + 3 Reducer   └── 31 domain-* 目标
                                    
        平行 (per 9/3 19:00 JST 拍板 A 独立目录)        ↘ zustand store 共享数据
```

**核心约束** (per 守门 #3 反转 + AGENTS.md §5):
- **3 view 平行**, 不建立业务子域↔DDD bounded context 映射
- LangGraph subgraph 实现 SA-XX 业务逻辑, Agent Runtime ECS 提供底层 Runtime
- Agent View 通过 zustand store 共享数据 (workItems / worktrees / agentSessions), **不**直接调用其他 view 的 action

---

## 4. 守门合规 (per AGENTS.md §4)

| 守门 | 内容 | 拓扑体现 |
|---|---|---|
| **#3 5 域独立 Lead** | 不兼任, 不建立业务子域↔DDD 映射 | 拓扑不画 5 域边界, 改用 DDD bounded context |
| **#13 DB W/T/M 严格分类** | 100% 表覆盖, RLS 必携 | [`05-persistence-checkpoint.md`](./05-persistence-checkpoint.md) 标 5 张表分类 |
| **#1 v20 -j 4 守门** | cargo check 实证 | 拓扑节点跟实际 crate 名一致 |
| **#4 token-OLU** | 1 SRE·周 ≈ 1.2M tokens | 节点 + 边数 < 200 (避免超过单 sub-session 上限) |
| **#12 缺标比错标** | 已知缺口显式列 | 拓扑在每图末标「已知缺口」 |

---

## 5. 已知缺口 (per 守门 #11 缺标比错标安全)

- **G-1**: 拓扑只覆盖 [[S1]]-[[S7]] 设计源头, 不含 30+ domain spec 单文件 (per [`docs/specs/`](../../specs/)) — 等 DDD Review 拍板后再扩
- **G-2**: 拓扑不画 Phase D-I 阶段性演进的 commit 链 (per [`docs/architecture/2026-08-26-upgrade/adr/0035-0038`](../../architecture/2026-08-26-upgrade/adr/)) — 拓扑是设计的"应有态", 实际渐进落地
- **G-3**: 拓扑不画 mobile-flutter-mvp 跨端 (per [`docs/architecture/2026-09-02-upgrade/spec/mobile/01-flutter-mvp-design.md`](../../architecture/2026-09-02-upgrade/spec/mobile/)) — 暂未跟 3 view 整合
- **G-4**: 拓扑不画 cross-domain-5b-mermaid 早期图 (per [`docs/architecture/cross-domain-5b-mermaid.md`](../../architecture/cross-domain-5b-mermaid.md)) — 该图基于 RGS 镜像, 已被守门 #3 拍板拒绝, 改用 DDD bounded context
- **G-5**: 5 域 Lead 真人未到位 (per [`docs/recruitment/5-business-domain-lead-referral.md`](../../recruitment/5-business-[[domain-lead-referral]].md)) — 拓扑中 5 域 RACI 暂以 Mavis 临时代签占位

---

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 7 份拓扑文件 + 7 源头 + 守门 4 项 + 已知缺口 5 项 | 2026-09-06 10:06 JST 用户发令"根据文档里的设计制作设计拓扑,如实根据设计内容制作图谱,而不是文档的层级结构" |


## Obsidian 双向链 (Bidirectional Links, v0.2 NEW)

> **拍板 (per 2026-09-06 17:13 JST 用户)**: docswiki 8 份转 Obsidian Wiki 风格, 完整集 frontmatter 13 字段, 节点→节点 + 源→拓扑双向链

### 1. 出现在本拓扑的节点 (in-topology)

_无_

### 2. 横向相关 (related)

- [[S1]]
- [[S2]]
- [[S3]]
- [[S4]]
- [[S5]]
- [[S6]]
- [[S7]]

### 3. 参见 (see-also)

- [[00-design-topology]]

### 4. Obsidian Canvas

- 配套 `.canvas` 文件: `docs/wiki/docswiki/canvas/README.canvas`
- Obsidian Canvas 插件打开, 节点按 sub-graph 分色, 边显式标

### 5. 节点笔记索引

- 152 份节点笔记位于 `docs/wiki/docswiki/nodes/`
- 节点 ID = 文件名 (e.g. `C-01.md` / `domain-tenant.md` / `M-N1.md`)

### 6. 守门实证 (本段 v0.2 NEW)

- 0 回溯叙事 (per 守门 #12)
- 100% 文档实证 (per 守门 #12)
- 缺标比错标 (per 守门 #11)
- 3 view 平行, 不建立业务子域↔DDD 映射 (per 守门 #3)
- 修订 author = Ulysses (per 守门 #10 + 8/27 19:39 JST 授权)
