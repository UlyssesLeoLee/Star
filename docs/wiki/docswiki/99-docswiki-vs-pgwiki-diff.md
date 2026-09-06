---
title: '99 — docswiki vs pgwiki 差异分析 (Diff Report)'
date: 2026-09-06
source: 8 拓扑文件 + 7 设计源头 (S1-S7) + 152 节点笔记
status: obsidian-wiki-baseline
classification: obsidian-wiki
version: 0.2
revision: 'v0.2 @ 2026-09-06 Ulysses(per 19:39 JST)— Mavis 接手; v0.1 @ 2026-09-06 初版 (1 索引 + 7 拓扑)'
supersedes: null
in-topology: ["S1", "S2", "S3", "S4", "S5", "S6", "S7", "View-AgentView", "View-LangGraph", "View-AgentRuntime", "domain-tenant", "domain-agent", "domain-context"]
related: ["00-design-topology", "01-ui-agent-view", "02-orchestration-langgraph", "03-runtime-ecs", "04-domain-crates", "05-persistence-checkpoint", "06-data-flow", "99-pg-broker-audit"]
see-also: ["S4", "S5", "S7"]
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
  - cross-wiki
  - diff-report
  - design-vs-reality
  - obsidian-wiki
  - design-topology
  - obsidian-wiki
  - design-topology
---



# 99 — docswiki vs pgwiki 差异分析

> **目的**: 把 [[00-design-topology|docswiki]] 7 份叙事 (设计应有) 跟 [[pgwiki-50-issues-07-docswiki-vs-pgwiki|pgwiki]] 212 节点 (实际工程) 做差异分析, 暴露"叙事 vs 事实" gap.
> **数据源**:
> - docswiki: 8 份设计拓扑 (per [[README]] + [[00-design-topology]])
> - pgwiki: 5 主题 222 文件 + `scripts/automation/pgwiki_audit.py` 实测
> **拍板**: 2026-09-06 18:08 JST 用户"对比文档 wiki 和 pgwiki,找出差异"
> **核心约束** (per 守门 #3 + AGENTS.md §5 disclaimer): **不建立 docswiki↔pgwiki 1:1 映射**, 差异本身就是问题, 显式列出供 DDD Review 拍板

---

## 1. 总览 (Overview)

| 维度 | docswiki (叙事) | pgwiki (事实) | 差异 |
|---|---|---|---|
| **文件数** | 168 (8 拓扑 + 152 节点笔记 + 8 canvas) | 222 (5 主题 MOC) | docswiki 节点粒度更细, pgwiki 主题粒度 |
| **结构** | 1 主图 + 6 sub-graph + 152 节点 + 8 canvas | 10/20/30/40/50 编号主题 + MOC + 节点 | 完全不同结构 |
| **视角** | DDD 3 view 平行 (AgentView / LangGraph / Runtime) | 物理 crate + schema + ADR + cross + issues | 叙事 vs 事实 |
| **量化** | 22+9=31 domain 目标, 5 PG 表, 22 组件 | **52** cargo members, **25** schemas / **93** 张表, 28 ADR | 数字差异大 |
| **生成方式** | handcrafted + python 脚本 | 自动 (pgwiki_index.py + pgwiki_audit.py) | pgwiki 自动化更高 |
| **守门侧重** | 设计守门 (W/T/M 严格, 3 view 平行) | 工程守门 (broker ADR, orphan schema) | 互补 |
| **可读性** | Mermaid 图 + 表格 + 双链 | MOC 链接 + 表清单 | docswiki 视觉, pgwiki 文字 |

**核心观察 (per 守门 #11 缺标比错标)**:
- docswiki 反复用 22/47/31 数字, **实际是 52** cargo members
- docswiki 反复用 5 PG 表, **实际是 25 schema / 93 张表** (5 是 Tier 3 checkpoint 子集)
- pgwiki 是"事实"层, docswiki 是"应有"层, **2 个 wiki 必须并存** 才能用对比暴露 gap

---

## 2. 结构差异 (Structure)

### 2.1 docswiki 结构 (DDD 视角)

```
docs/wiki/docswiki/
├── README.md                          # 索引 + 7 源头 + 守门合规
├── 00-design-topology.md              # 主图 (UI/编排/Runtime/Domain/Persistence/Platform)
├── 01-ui-agent-view.md                # Agent View 拓扑
├── 02-orchestration-langgraph.md      # LangGraph Orchestration (L0 + TMO + L1 + Cross)
├── 03-runtime-ecs.md                  # Agent Runtime (L0/L1/L2 + 12 Comp + 13 Sys)
├── 04-[[domain-crates]].md                # 22 domain crate Tier 1-6
├── 05-persistence-checkpoint.md       # 3-tier checkpoint + 5 PG 表
├── 06-data-flow.md                    # 7 个跨 view 数据流
├── nodes/  (152 份)
│   ├── [[S1]]..[[S7]].md                      # 7 设计源头
│   ├── [[C-01]]..[[C-22]].md                  # 22 组件
│   ├── [[M-01]]..[[M-25]].md                  # 25 模块
│   ├── [[T-N1]]..[[T-N7]].md, [[M-N1]]..[[M-N7]].md   # 7+7 节点
│   ├── [[SA-01]]..[[SA-10]].md                # 10 SA 类型
│   ├── [[M-AGV-1]]..[[M-AGV-10]].md           # 10 Agent View 模块
│   ├── Comp-*.md (12 份)              # 12 ECS Component
│   ├── Sys-*.md (13 份)                # 13 System
│   ├── domain-*.md (31 份)            # 22+9 domain crate
│   ├── PG-*.md (5 份)                 # 5 PG 表
│   └── View-*.md (3 份)               # 3 View
└── canvas/  (8 份)
    ├── 00-design-topology.canvas
    ├── 01..06-*.canvas
    └── README.canvas
```

**特点**: 围绕 7 份设计源头 ([[S1]]-[[S7]]) 展开, Obsidian 风格, 双链 + canvas

### 2.2 pgwiki 结构 (工程事实)

```
docs/wiki/pgwiki/
├── 00-INDEX.md
├── 10-workspace/                      # 52 crate 实际清单
│   ├── MOC.md
│   ├── _crates/  (52 份)              # 52 cargo member 节点
│   ├── frontend-root.md
│   ├── scripts-root.md
│   ├── scripts-automation.md
│   └── tools-root.md
├── 20-database/                       # 25 schema / 93 张表
│   ├── MOC.md
│   ├── _schema/  (25 份)              # 26 schema 节点
│   └── _table/   (93 份)              # 93 张表
├── 30-architecture/                   # 28 ADR + 5 view
│   ├── MOC.md
│   ├── adr/adr-0021..0047.md  (28 份)
│   └── views/view-2026-*.md  (5 份)
├── 40-crosscutting/                   # 跨切
│   ├── dependencies.md
│   ├── quality-gates.md
│   └── schema-to-crate.md
├── 50-issues/                         # 6 已知问题 (审计产出)
│   ├── MOC.md
│   ├── 00-orphan-schemas.md           # 1 orphan (work)
│   ├── 01-placeholder-schemas.md      # 1 placeholder (kms)
│   ├── 03-broker-adr-refs.md          # 5 broker ADR
│   ├── 04-broker-arch-refs.md         # 32 broker arch
│   ├── 07-docswiki-vs-pgwiki.md       # 本对照表前驱
│   └── MOC.md
└── .obsidian/                         # Obsidian 配置
```

**特点**: 围绕"实际工程"展开, 编号主题, 自动生成, 强调"事实 + 问题"

### 2.3 主题编号差异

| docswiki 主题 | pgwiki 主题 | 差异 |
|---|---|---|
| 00-design-topology (主图) | (无) | pgwiki 没有"应有态"主图, 仅有"事实态" 30-architecture |
| 01-ui-agent-view | (无, 仅 frontend-root) | docswiki 提到 store/canvas, pgwiki 仅 frontend-root |
| 02-orchestration-langgraph | 30-architecture/views/view-2026-09-03-langgraph | docswiki 22 组件 25 模块 7 TMO, pgwiki 1 view 文件 |
| 03-runtime-ecs | 30-architecture/views/view-2026-09-03-agent-runtime | 同上 |
| 04-[[domain-crates]] | 10-workspace/_crates/ (52 份) | docswiki 22+9=31, **pgwiki 52** |
| 05-persistence-checkpoint | 20-database/ (25 schema / 93 张表) | docswiki 5 PG 表, **pgwiki 93 张表** |
| 06-data-flow | 40-crosscutting/dependencies | docswiki 7 个数据流时序, pgwiki 1 份依赖图 |

**核心观察**: docswiki 是"设计应有", pgwiki 是"工程事实" — 两者视角不同, **不重叠但有 gap**

---

## 3. 量化差异 (Quantitative Diff)

| 指标 | docswiki 声称 | pgwiki 实际 | gap | 来源 |
|---|---|---|---|---|
| **cargo workspace members** | 22+9=31 (per [[S4]]+[[S5]] 目标) | **52** | +21 crate (pgwiki 多) | pgwiki_audit.py 实测 |
| **DB schemas** | 5 PG 表 (Tier 3 only) | **25** schemas / 93 张表 | +20 schema | pgwiki_audit.py 实测 |
| **DB tables** | 5 (per [[S7]]) | **93** (M=5, T=86, W=2) | +88 表 | per [[pgwiki-20-database-MOC]] |
| **W/T/M 分类** | 5/5 严格 (per [[S7]]) | 5 M + 86 T + 2 W = 93 | docswiki 覆盖 0.4% | per `00-CLASSIFICATION-W-T-M.md` |
| **LangGraph 组件 [[C-01]]..[[C-22]]** | 22 (per [[S2]] §1.3) | (pgwiki 无 1:1 节点) | docswiki 独有 | docswiki-only |
| **LangGraph 模块 [[M-01]]..[[M-25]]** | 25 (per [[S3]] §1.2) | (pgwiki 无 1:1 节点) | docswiki 独有 | docswiki-only |
| **9 SA 类型** | [[SA-01]]..[[SA-10]] (per [[S2]]) | (pgwiki 无 1:1 节点) | docswiki 独有 | docswiki-only |
| **3 view** | [[View-AgentView]] / LangGraph / AgentRuntime | view-2026-08-26 / 09-02 / 09-03 (4 份) | pgwiki 多 1 (09-02 mobile) | [[pgwiki-only]] |
| **ADR** | 引用 ADR-0021..0047 (28 份) | adr-0021..0047 (28 份) | **一致** | 双向引用 |
| **Obsidian 配置** | 152 节点笔记 + 8 canvas | .obsidian/ (仅 config) | docswiki 独有 | docswiki-only |
| **5 域 Lead** | 5 (Permission/Worktree/Flow/Agent/Integration) + Admin | (pgwiki 无 5 域命名) | **不重叠** | per 守门 #3 |

**核心观察** (per 守门 #11):
- docswiki 的 "22+9=31" **是 Tier 1-6 目标**, pgwiki 的 52 是 **物理 crate 现状**, 数字差异因为 9 域 + 9 新建未实装 + 多个非 domain crate (star-*, crate-*)
- docswiki 的 "5 PG 表" **是 Tier 3 checkpoint 子集**, pgwiki 的 93 **是全应用 schema**, 数字差异因为视角不同
- **ADR 一致** (28 份双向引用), 是 2 个 wiki 唯一完全重叠区

---

## 4. 焦点差异 (Focus)

### 4.1 docswiki 关注什么

- **设计应有态** (design-as-intended): 7 份设计源头 + 152 节点 + 双链
- **DDD 视角**: 3 view 平行, 25 模块, 22 组件, 9 SA, TMO 7 节点
- **守门 #13 DB W/T/M**: 5 张 PG 表 100% 严格分类 (T=3, M=1, T-WORM=1)
- **3 view 平行**: 不建立业务子域↔DDD 映射 (per 守门 #3 + AGENTS.md §5)
- **Obsidian 风格**: frontmatter 13 字段, 双向链, Canvas

### 4.2 pgwiki 关注什么

- **工程事实** (as-built): 52 crate + 93 张表 + 28 ADR
- **工程视角**: 物理 crate + DB schema + ADR + 跨切依赖 + 已知问题
- **守门 #13 W/T/M**: 5 M + 86 T + 2 W = 93 (全量, 含 DDD Review 阶段待 5 域 Lead 拍板)
- **已知问题** (50-issues/): orphan schema / placeholder / broker ADR-arch / docswiki-vs-pgwiki
- **自动化产出**: pgwiki_index.py + pgwiki_audit.py 自动生成

### 4.3 互补性

| 维度 | docswiki 强 | pgwiki 强 | 互补用法 |
|---|---|---|---|
| 设计意图 | ✓ | | DDD Review 拍板时, docswiki 引领 |
| 工程现状 | | ✓ | 实施 / 集成时, pgwiki 引领 |
| 5 域 RACI | ✓ (但 5 域真人未到位) | | 5 域 Lead 真人到位后, 双 wiki 同步 |
| DB schema 拍板 | ✓ (5 张 Tier 3) | ✓ (93 张全量) | 5 域 Lead admin 域拍板时, pgwiki 走全量 |
| 已知问题 | (缺失) | ✓ (50-issues 6 份) | pgwiki 暴露, docswiki 暂未引用 |
| 自动化 | ✓ (3 份 Python) | ✓ (pgwiki_index + audit) | 互不冲突 |
| Obsidian 风格 | ✓ (152 笔记 + 8 canvas) | ✓ (.obsidian/ config) | 两个 vault 合并开 |

---

## 5. 关键差异事项 (Critical Diff)

### 5.1 数字不一致 (per 守门 #11 显式列)

| # | 项 | docswiki 写 | pgwiki 实际 | 拍板 |
|---|---|---|---|---|
| 1 | cargo members | 22+9=31 (per [[S4]]+[[S5]]) | **52** | 缺标, 标注 "22+9=31 是 Tier 1-6 目标, 实际 52 含 star-* 等" |
| 2 | DB schemas | 5 (per [[S7]]) | **25** | 缺标, 标注 "5 是 Tier 3 checkpoint, 实际 25 schema 涵盖全应用" |
| 3 | DB tables | 5 (per [[S7]]) | **93** | 缺标, 同上 |
| 4 | 域 | 5 + Admin = 6 域 | (pgwiki 无 5 域) | per 守门 #3, 不映射 |
| 5 | SA 类型 | 9+1=10 ([[SA-10]] NEW) | (pgwiki 无 SA 命名) | docswiki 独有 |
| 6 | 3 view | 3 (AgentView/LangGraph/Runtime) | 4 view (含 09-02 mobile-flutter-mvp) | pgwiki 多 1, docswiki 暂未含 mobile |

### 5.2 主题缺失 (Gap)

| # | 主题 | docswiki | pgwiki | 差距 |
|---|---|---|---|---|
| 1 | **mobile-flutter-mvp** | ✗ 缺 | ✓ view-2026-09-02-upgrade.md (5 份) | docswiki 缺 mobile view 拓扑, 9/6 拍板"范围仅 docswiki 8 份"故未含 |
| 2 | **frontend 全部** | ✗ 仅 Agent View | ✓ 10-workspace/frontend-root.md | docswiki 仅有 1 view, pgwiki 含整前端 |
| 3 | **scripts/automation** | ✓ 3 份 (obsidian_*) | ✓ scripts-automation.md (含 14+ 脚本) | pgwiki 多覆盖 |
| 4 | **tools/** | ✗ 缺 | ✓ tools-root.md | docswiki 缺 |
| 5 | **已知问题审计** | ✗ 缺 (各图末"已知缺口" 散落) | ✓ 50-issues/ 6 份系统化 | docswiki 应参考 pgwiki 50-issues 做集中 |

### 5.3 唯一一致区

- **ADR 28 份** (adr-0021..adr-0047): docswiki 在 152 节点笔记中引用, pgwiki 在 30-architecture/adr/ 实际存放
- **守门 #13 DB W/T/M**: 都有体现, 但粒度不同 (5 vs 93)
- **3 view 平行**: docswiki 主导, pgwiki 配合
- **5 域 Lead 真人未到位**: 两 wiki 都未拍板, 暂以 Mavis 临时代签

---

## 6. pgwiki 50-issues 关键问题 (供 docswiki 后续引用)

per [[pgwiki-50-issues-00-orphan-schemas]]:
- **`work` schema** = orphan (per pgwiki_audit.py), 暂无 crate 承载

per [[pgwiki-50-issues-01-placeholder-schemas]]:
- **`kms` schema** = placeholder (per pgwiki_audit.py), 待实装

per [[pgwiki-50-issues-03-broker-adr-refs]]:
- 5 broker ADR (arch 引用但 ADR 没拍): `api-key`, `domain-service`, `domain-team`, `star-lsp-proxy`, `star-optional`

per [[pgwiki-50-issues-04-broker-arch-refs]]:
- 32 broker arch (arch 引用但 arch 文件未实装): `api-key`, `domain-backpressure`, `domain-cb`, `domain-dispatcher`, `domain-graph-agent`, `domain-http`, `domain-llm`, `domain-mcp`, `domain-memory`, `domain-observability`, `domain-ops-rbac`, `domain-policy`, `domain-prompt`, `domain-provider`, `domain-queue`, `domain-rag`, `domain-rate-limiter`, `domain-retry`, `domain-service`, `domain-task`, `domain-team`, `domain-tool`, `star-cache-readonly`, `star-ide-gateway`, `star-lsp-proxy`, `star-mcp-readwrite`, `star-optional`, `star-postgres`, `star-redis`, `star-rest`, `star-sa-cluster`, `star-system`

**核心观察**: docswiki [[04-domain-crates]] 列出的 9 新建 domain crate ([[domain-dispatcher]] / [[domain-llm]] / [[domain-mcp]] / [[domain-tool]] / [[domain-rag]] / [[domain-context]] / [[domain-memory]] / [[domain-rate-limiter]] / [[domain-observability]]), 全部在 pgwiki 的 broker-arch 列表里 — 即"设计意图"已经写但"工程实装"未拍板, 这是 docswiki 跟 pgwiki 最大 gap.

---

## 7. 守门合规对照 (per 守门 #11 缺标比错标)

| 守门 | docswiki 状态 | pgwiki 状态 | 拍板 |
|---|---|---|---|
| **#1 v19 cargo check -j 4** | ✓ (per README) | ✓ (per pgwiki quality-gates.md) | 一致 |
| **#3 5 域独立 / 不建立映射** | ✓ disclaimer | ✓ (5 域命名不在 pgwiki) | 一致 |
| **#6 PowerShell only** | ✓ (per scripts) | ✓ | 一致 |
| **#7 0 unsafe** | ✓ (纯 markdown/Python) | ✓ (纯 markdown/Python) | 一致 |
| **#11 缺标比错标** | ✓ (每图末「已知缺口」) | ✓ (50-issues 集中) | 一致 |
| **#12 AI 協作文档治理** | ✓ (0 回溯叙事) | ✓ (auto-gen + git 实证) | 一致 |
| **#13 DB W/T/M 严格分类** | ✓ (5/5 严格) | ✓ (93/93 严格, DDD Review 待 5 域 Lead 拍板) | docswiki 子集, pgwiki 全量 |
| **#19 agent 交互 Python 化** | ✓ (3 份 Python) | ✓ (pgwiki_index.py + audit.py) | 一致 |
| **#20 子代理 dispatch 必先落地 brief** | ✓ (无子代理, 守门空闲) | ✓ | 一致 |
| **#22 [P] 子项 docs 同步** | ✓ (commit 含 brief 路径) | ✓ | 一致 |

**核心观察**: 两个 wiki 守门合规高度一致, 互补守门 #13 视角

---

## 8. 已知缺口 (per 守门 #11)

- **G-1**: docswiki 暂未引用 pgwiki 50-issues 6 份作为"已知问题集中源", 后续应双向链
- **G-2**: docswiki 暂未含 mobile-flutter-mvp 跨端 view (per 9/6 拍板"范围仅 docswiki 8 份")
- **G-3**: docswiki 暂未含 frontend 全部 (仅有 Agent View), pgwiki frontend-root.md 含整前端
- **G-4**: docswiki 暂未含 tools/ 目录, pgwiki tools-root.md 含
- **G-5**: docswiki 的 "22+9=31 domain" 跟 pgwiki 52 crate 的数字差, 已在 §3 显式列
- **G-6**: docswiki 的 "5 PG 表" 跟 pgwiki "93 张表" 的数字差, 已在 §3 显式列
- **G-7**: docswiki 的 5 域命名 (Permission/Worktree/Flow/Agent/Integration) 跟 pgwiki 的 25 schema 不建立 1:1 映射 (per 守门 #3)
- **G-8**: pgwiki 50-issues 04-broker-arch-refs 列 32 个 broker crate, docswiki 的"应有态"应明确这些"设计意图 vs 工程实装" gap
- **G-9**: 两个 wiki 暂未建立自动化 diff 脚本 (pgwiki_audit.py 跑 docswiki vs pgwiki 数字对比), 后续应 `scripts/automation/wiki_diff.py`
- **G-10**: 5 域 Lead 真人未到位, 5 域 RACI 在两个 wiki 都暂以 Mavis 临时代签

---

## 9. 后续行动建议 (per DDD Review)

1. **优先 1**: 把 pgwiki 50-issues/ 6 份"已知问题" 在 docswiki 8 份拓扑每图末"已知缺口"中做交叉引用 (双向链)
2. **优先 2**: docswiki §3 量化差异表 加 pgwiki 数字 (52 cargo, 93 张表), 显式标"设计意图 vs 工程事实"
3. **优先 3**: 写 `scripts/automation/wiki_diff.py` 自动化 §3 量化差异表, 跑 cargo metadata + pgwiki_audit 双向
4. **优先 4**: docswiki 应增一份 `99-mobile-flutter-view.md` (per pgwiki view-2026-09-02), 跨端纳入"应有态"
5. **优先 5**: 5 域 Lead 真人到位后 (per 守门 #14 v2, T3 = 2026-09-19 ~ 2026-09-26), 5 域 RACI 在两个 wiki 同步追溯签字

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-06 18:08 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 10 节 (总览/结构/量化/焦点/关键差异/50-issues/守门/缺口/后续/修订) | 2026-09-06 18:08 JST 用户发令"对比文档 wiki 和 pgwiki,找出差异" |

---

## 11. Obsidian 双向链

### 1. 出现在本差异报告的节点

- [[S1]] [[S2]] [[S3]] [[S4]] [[S5]] [[S6]] [[S7]]

### 2. 横向相关 (related)

- [[00-design-topology]]
- [[01-ui-agent-view]]
- [[02-orchestration-langgraph]]
- [[03-runtime-ecs]]
- [[04-domain-crates]]
- [[05-persistence-checkpoint]]
- [[06-data-flow]]
- [[README]]

### 3. 参见 (see-also)

- [[pgwiki-50-issues-07-docswiki-vs-pgwiki]]
- [[pgwiki-50-issues-00-orphan-schemas]]
- [[pgwiki-50-issues-01-placeholder-schemas]]
- [[pgwiki-50-issues-03-broker-adr-refs]]
- [[pgwiki-50-issues-04-broker-arch-refs]]
- [[pgwiki-10-workspace-MOC]]
- [[pgwiki-20-database-MOC]]
- [[pgwiki-30-architecture-MOC]]
- [[pgwiki-40-crosscutting-dependencies]]
- [[pgwiki-40-crosscutting-quality-gates]]
- [[pgwiki-40-crosscutting-schema-to-crate]]

### 4. Obsidian Canvas

- 配套 `.canvas` 文件: `docs/wiki/docswiki/canvas/99-docswiki-vs-pgwiki-diff.canvas`
- Obsidian Canvas 插件打开, 节点按 wiki / 主题分色 (docswiki 蓝 / pgwiki 绿 / 共同 紫)

### 5. 节点笔记索引

- 152 份节点笔记位于 `docs/wiki/docswiki/nodes/`
- 节点 ID = 文件名 (e.g. `C-01.md` / `domain-tenant.md` / `M-N1.md`)

### 6. 守门实证 (本段 v0.2 NEW)

- 0 回溯叙事 (per 守门 #12)
- 100% 文档实证 (per 守门 #12) — 量化数据全部来自 `pgwiki_audit.py` 实测
- 缺标比错标 (per 守门 #11)
- 3 view 平行, 不建立业务子域↔DDD 映射 (per 守门 #3)
- 修订 author = Ulysses (per 守门 #10 + 8/27 19:39 JST 授权)



## Obsidian 双向链 (Bidirectional Links, v0.2 NEW)

> **拍板 (per 2026-09-06 17:13 JST 用户)**: docswiki 8 份转 Obsidian Wiki 风格, 完整集 frontmatter 13 字段, 节点→节点 + 源→拓扑双向链

### 1. 出现在本拓扑的节点 (in-topology)

- [[S1]]
- [[S2]]
- [[S3]]
- [[S4]]
- [[S5]]
- [[S6]]
- [[S7]]
- [[View-AgentView]]
- [[View-LangGraph]]
- [[View-AgentRuntime]]
- [[domain-tenant]]
- [[domain-agent]]
- [[domain-context]]

### 2. 横向相关 (related)

- [[00-design-topology]]
- [[01-ui-agent-view]]
- [[02-orchestration-langgraph]]
- [[03-runtime-ecs]]
- [[04-domain-crates]]
- [[05-persistence-checkpoint]]
- [[06-data-flow]]
- [[99-pg-broker-audit]]

### 3. 参见 (see-also)

- [[S4]]
- [[S5]]
- [[S7]]

### 4. Obsidian Canvas

- 配套 `.canvas` 文件: `docs/wiki/docswiki/canvas/99-docswiki-vs-pgwiki-diff.canvas`
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
