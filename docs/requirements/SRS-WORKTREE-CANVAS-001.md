# SRS-WORKTREE-CANVAS-001

> **AI Worktree Graph Canvas — 无限画布需求文档 v1.0** (per 日本 IPA SEC 標準 / 要件定義書 テンプレート)
>
> - 状态: Requirements Baseline (v1.0 初版落档)
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: (root 统一 commit 时填, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
> - 关联 issue: `Multica ULYS-57` (制作 Worktree 页面各级文档)
> - 上位要件: 无独立上位 SRS (本 SRS 为新领域主册; 平行引用 `SRS-CANVAS-001.md` v1.1 无限画布总册 + `SRS-AGENT-VIEW-001.md` v1.0 个体视图 + `SRS-AGENT-RELATIONSHIP-001.md` v0.1 ARG)
> - 平行专题 SRS: 0 (本 SRS 为独立专题主册, 不下挂子专题)
> - 关联后续基本設計: `docs/design/BD-WORKTREE-CANVAS-001.md` v1.0 (本 commit 同期落档)
> - 关联后续詳細設計: `docs/design/DD-WORKTREE-CANVAS-001.md` v1.0 (本 commit 同期落档)
> - 关联追踪矩阵: `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` v1.0 (本 commit 同期落档)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3 Mavis 接手代签, 5 域真人到位后切真人)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST)
> - 日期: 2026-09-15 JST
> - 受众: 詳細設計工程師 / 架構審查者 / UI/UX 設計師 / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-WORKTREE-CANVAS-001 |
| 文书名 | AI Worktree Graph Canvas — 无限画布 需求定义书 |
| 版本 | v1.0 (初版) |
| 作成日 | 2026-09-15 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 commit | (root 统一 commit 时填, per 守门 #1 v15) |
| 关联文档 | `BD-WORKTREE-CANVAS-001.md` v1.0 (同期落档) + `DD-WORKTREE-CANVAS-001.md` v1.0 (同期落档) + `TRACEABILITY-WORKTREE-CANVAS-001.md` v1.0 (同期落档) |
| 上位文档 | 无 (新领域主册) |
| 平行文档 | `SRS-CANVAS-001.md` v1.1 (无限画布总册) + `SRS-AGENT-VIEW-001.md` v1.0 (个体视图) + `SRS-AGENT-RELATIONSHIP-001.md` v0.1 (ARG) |
| 拍板来源 | 2026-09-15 Multica ULYS-57 issue 创建者发令 "你是一名资深软件架构师...针对一个面向 AI 并行开发场景的 Worktree 无限画布管理模块, 依次生成需求文档 / 基本设计 / 详细设计" |

### 0.2 修订履历 (本 SRS)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-15 JST | Ulysses — Mavis 接手 (per 守门 #14 v3) | 初版落档, 13 段 (文档信息/目的/用语/前提/业务需求/约束/场景/数据/接口/验收/风险/签字/修订), **126 唯一 ID** (103 FR + 23 NFR 子段, 去重后 21 唯一 NFR ID), 39 用户故事 (≥ 39 满足), 13 已知缺口, 5 个视图模式 + 6 级 Semantic Zoom + 7 状态机 + 11 风险类型 + 5 层 Canvas 性能策略 | 2026-09-15 Multica ULYS-57 issue 创建者发令 |
| v1.1 | 2026-09-17 JST | Ulysses — Mavis 接手 (per 守门 #14 v3, self-review C-01..C-04 修正) | 修正: 4 文档总数对齐 **126 唯一 ID**; FR-ACTION-002 Merge 改 Destructive (per self-review C-02); Trace §11 AC 43 项全表化 (per self-review C-03); BD §0.3/§37 总数同步 (per self-review C-04) | 2026-09-17 ULYS-62 self-review 修正落地 |
| v1.2 | 2026-09-19 JST | Ulysses — Mavis 接手 (per 守门 #14 v3 + self-review 整体审查 m-7 派生) | 修正: "Mavis 永久代签" 措辞 → "Mavis 接手代签 (5 域真人到位后切真人)" (修订人栏, per self-review m-7) | 2026-09-19 04:55 JST 自审整体审查 + 9/18 23:14 JST 评论者发令 "没动的也都处理到位" |

### 0.3 撤回记录 (per 守门 #1 禁回溯叙事)

本 SRS 为初版落档, 暂无撤回记录。未来如有重要撤回需在此显式记录 (per 守门 #1 禁回溯叙事约束)。

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 制定 STAR / Multica 平台 **AI Worktree Graph Canvas** (AI Worktree 无限画布管理模块) 的需求规格说明书。

本模块定位 **不是** 传统 Git GUI, **不是** 单纯 Commit Graph, 而是帮助 **人类监督 AI Agent 并行产生的软件开发分支世界** 的低认知负荷画布系统。

本文档涵盖:

1. **核心产品定位** — Human-first / Worktree-first / Graph-native / AI-native (R1-R10 设计原则)
2. **一级认知模型** — Project / Repository / Mainline / Worktree / Task / Agent Session / Worktree Relationship / Risk / Merge Readiness
3. **节点类型** — 11 类 Node (Repository / Branch / Worktree / Commit / Task / AgentSession / PullRequest / File / Symbol / TestRun / Issue)
4. **边类型** — 13 类 Edge (BASED_ON / USES_BRANCH / DERIVED_FROM / WORKS_ON / IMPLEMENTED_IN / MODIFIES / MODIFIES_SYMBOL / CONFLICTS_WITH / OVERLAPS_WITH / DEPENDS_ON / BLOCKS / SUPERSEDES / MERGED_INTO)
5. **7 种 Worktree 人类态** — RUNNING / WAITING / READY / DIVERGED / CONFLICT / MERGED / STALE
6. **5 个视图模式** — TREE / DEPENDENCY / RISK / AGENT / HISTORY
7. **6 级 Semantic Zoom** — L0 Project / L1 Repo+Worktree / L2 Task+Agent / L3 Commit / L4 File / L5 Symbol
8. **Risk Engine V1-V3** — Git Status / Diff Overlap / Symbol Overlap
9. **Health Score** — 0-100, 13 影响因素, 可解释
10. **AI Explanation** — 基于 Graph + Git + Risk Engine, 不凭空生成
11. **Focus Mode** — 1/2/3-hop Neighborhood
12. **Search DSL** — `show:` / `agent:` / `behind:>20` / `health:<50` / NL Query
13. **18 个核心 Action** — Create / Open / Open in IDE / Compare / Sync / Rebase / Merge / PR / Lock / Unlock / Delete / Cleanup / Archive / Mark Superseded / Set Dependency / Remove Dependency / Focus / Explain Risk
14. **事件总线** — 15 个事件, 4 类消费者
15. **MVP 范围** — 5 Node / 7 State / 4 Edge / 8 Action (per 28 节)

**总需求数**: **126 唯一 ID** (FR 103 + NFR 23 子段, 去重后 21 唯一 NFR ID)。FR 详细: FR-WT 38 + FR-UI 14 + FR-GRAPH 12 + FR-RISK 11 + FR-AGENT 8 + FR-EXPLAIN 4 + FR-SEARCH 6 + FR-ACTION 10 = **103 FR ID**; NFR §4.9.x 子段 23 (去重后 21 唯一 ID)。

本 SRS 是 P3-D 阶段 "Worktree 页面文档化" 的主册产物, 跟 `BD-WORKTREE-CANVAS-001.md` v1.0 + `DD-WORKTREE-CANVAS-001.md` v1.0 + `TRACEABILITY-WORKTREE-CANVAS-001.md` v1.0 共同构成 ULYS-57 issue 4 份文档。

作为后续实装 / 测试 / 验收的唯一依据 (per 守门 #1 禁回溯叙事约束, 后阶段不允许重新定义核心概念)。

### 1.2 背景 (用户痛点)

随着 AI Agent 并行开发场景普及 (Multica / 各类 coding agent / Claude Code / Codex / OpenCode), **单个 Repository 同时存在 10-1000 个 AI 生成的 Worktree**。传统 Git GUI 以 Branch + Commit + DAG 为中心, 无法帮助人类回答:

- 当前有多少 Worktree? 哪些已合并 / 未合并? 哪些落后 Main? 哪些领先? 哪些 Diverged?
- 哪些 Worktree 之间存在潜在代码冲突 / 修改了相同文件 / 修改了相同 Symbol?
- 哪些 Worktree 之间存在依赖 / 阻塞 / 重叠关系?
- 哪些 Worktree 可安全合并? 哪些需要先同步 Main? 哪些已长期无活动 / 应当清理?
- 哪个 Agent 在处理哪个 Task? 哪些 AI 任务互相产生冲突?
- **某个异常为什么发生**? **下一步人类应该优先处理什么**?

**R7 Exception-first** 违反: 传统 Git GUI 把所有节点同色等大显示, 异常节点淹没在 1000 个 commit 中。R10 Graph as Context 违反: 传统 Git GUI 把图数据锁在 GUI 后端, AI Agent 拿不到结构化工程上下文。

**核心问题**: 人类无法以低认知负荷监督 AI Worktree 世界。

### 1.3 包含范围

- Worktree Canvas 页面 `/worktree-canvas` (路由 + UI + 交互 + 键盘 + 多视图切换)
- 11 类 Node 渲染 (默认 UI 仅突出 Repository / Mainline / Worktree / Task / AgentSession)
- 13 类 Edge 渲染 (Tree Backbone + Semantic Graph Edge, 双重结构)
- 5 个视图模式 (TREE / DEPENDENCY / RISK / AGENT / HISTORY)
- 6 级 Semantic Zoom (L0 Project / L1 Repo+Worktree / L2 Task+Agent / L3 Commit / L4 File / L5 Symbol)
- 7 种 Worktree 状态机 (RUNNING / WAITING / READY / DIVERGED / CONFLICT / MERGED / STALE) + 内部 Machine State
- Health Score (0-100) + Risk Engine (V1/V2/V3) + 冲突预测
- AI Explanation Layer (基于 Graph + Git + Risk)
- Focus Mode (1/2/3-hop Neighborhood)
- Search DSL + NL Query
- 18 个核心 Action + 危险分类 (Safe / Warning / Destructive) + 确认机制
- 右侧固定 Inspector (Overview / Git State / Task / Agent / Risk / Tests / Files / Commits / Relations / History / Actions)
- Merged Worktree 处理 (折叠 / 弱化 / Recently Merged + 过滤)
- 事件总线 (15 个事件)
- Graph Repository Interface (Neo4j / Memgraph / Embedded / Custom 可替换)
- 性能策略 (LOD + Viewport Virtualization + Semantic Zoom + Graph Clustering + Lazy Expansion + Incremental Layout + Incremental Query)
- Git Source of Truth (Branch / Commit / Ref / Worktree / Diff / Merge-base / Ahead / Behind / Status)
- Graph Derived Data (Dependency / Conflict / Overlap / Superseded / Risk / Health)
- AI Derived Explanation (仅基于结构化数据, 不修改 Source of Truth)
- Runtime Agent State (Agent Session / Tool Call / Token Usage / Result State)

### 1.4 不包含范围

- **Git 操作底层执行** (依赖外部 Git CLI / libgit2 / Gitoxide, 不在本 SRS 范围)
- **AI Agent 执行引擎** (per `SRS-STAR-AGENT-RUNTIME-001.md` v1.0, 本 SRS 仅消费 Agent Session 数据)
- **图数据库引擎选型** (抽象为 `GraphRepository Trait`, 4 个备选 Neo4j / Memgraph / Embedded / Custom, 实装期选定)
- **Canvas 渲染引擎选型** (抽象为 `CanvasRenderer Trait`, 备选 React-Flow / PixiJS / 自研 WebGL/Canvas)
- **CRDT 协同算法选型** (Yjs vs Automerge vs LWW, 在 BD/DD 拍板)
- **L5 Symbol 级 AST 分析** (MVP 不要求, V3 Risk Engine 备选)
- **MCP 协议适配** (per `SRS-MULTICA-RUNTIME-001.md`, 平行专题)
- **多用户实时协同** (per `SRS-CANVAS-AGENT-001.md` v1.2 A12 多人编辑 8 项, 本 SRS 不重写, 仅消费其能力)
- **Notebook / Dashboard / 历史时间轴** (per `SRS-CANVAS-001.md` v1.1 双核心外, 平行专题)
- **Commit 内部 diff 内容渲染** (L4/L5 zoom 才展开)
- **PR Review 评论系统** (per `SRS-AGENT-RELATIONSHIP-001.md` ARG 平行专题, 本 SRS 仅消费 PR Node)

### 1.5 用户故事

| 编号 | 角色 | 故事 | 优先级 |
|---|---|---|---|
| US-1 | 项目经理 (PM) | 作为 PM, 我希望打开应用就看到当前 Repository 的 Worktree 全景图 (无限画布), 一眼看到 5 状态计数 (Ready / Running / Conflict / Stale / Merged), 不需要点击展开 | P0 |
| US-2 | 5 域 Lead (未来真人到位) | 作为 Lead, 我希望在画布上筛选我负责域的 Worktree, 看清每个 Worktree 的 Agent / Task / Risk / Test 状态 | P0 |
| US-3 | Dev | 作为 Dev, 我希望点击某个 Worktree 立刻看到 Ahead/Behind 数字 + 空间距离表达, 不用对比 Git log | P0 |
| US-4 | SRE | 作为 SRE, 我希望看到所有 Conflict / Stale Worktree 高亮突出, 一键 Sync Main 或 Merge, 不需要在多个 Tab 切换 | P0 |
| US-5 | PM | 作为 PM, 我希望看到 AI Agent 在处理哪个 Task / 哪个 Worktree, 并能切换 AGENT VIEW 按 Agent 维度看 | P0 |
| US-6 | Dev | 作为 Dev, 我希望快速对比 2 个 Worktree 之间的文件 / Symbol 重叠, 预判合并冲突 | P0 |
| US-7 | Dev | 作为 Dev, 我希望输入 `show:conflict` 或 `behind:>20` 立刻过滤, 不用滚动 1000 个节点 | P0 |
| US-8 | SRE | 作为 SRE, 我希望看到 Health Score 扣分明细, 每项可点开看原因, 不接受黑盒评分 | P0 |
| US-9 | PM | 作为 PM, 我希望用自然语言问 "显示所有还未合并、落后 main 超过 10 commit、由 Codex 处理的 Worktree", 系统翻译为 Graph Query | P1 |
| US-10 | Dev | 作为 Dev, 我希望点击某个 Worktree 进入 Focus Mode, 1-hop 只看 Parent / Child / Dependency / Conflict 节点, 其他节点降透明度 | P0 |
| US-11 | SRE | 作为 SRE, 我希望风险节点 (CONFLICTS_WITH / DEPENDS_ON) 高亮, 正常节点弱化 (R7 Exception-first) | P0 |
| US-12 | PM | 作为 PM, 我希望看到 Worktree 之间 DERIVED_FROM / SUPERSEDES / BLOCKS 关系, 不用问"这个 WT 是不是已经被替代" | P1 |
| US-13 | Dev | 作为 Dev, 我希望双击 Worktree 卡片, 右侧 Inspector 展开 11 个 tab (Overview/Git/Task/Agent/Risk/Tests/Files/Commits/Relations/History/Actions), 不弹 Modal | P0 |
| US-14 | 5 域 Lead | 作为 Lead, 我希望看到 RISK VIEW 只突出有 Risk 的 Worktree, 其他节点透明度降到 20% | P1 |
| US-15 | PM | 作为 PM, 我希望切换 DEPENDENCY VIEW 看到 DEPENDS_ON / BLOCKS 边粗线突出, TREE 边降透明度 | P1 |
| US-16 | SRE | 作为 SRE, 我希望缩放画布时, L0/L1/L2 信息粒度自动切换 (远看只显示 Repository + Worktree, 近看才展开 Commit/File/Symbol) | P0 |
| US-17 | Dev | 作为 Dev, 我希望 Hover 某条 CONFLICTS_WITH 边, 显示 shared_files / shared_symbols / confidence / reason | P0 |
| US-18 | PM | 作为 PM, 我希望 Merged Worktree 自动折叠到 "Recently Merged" 抽屉, 24h / 7d / All 过滤, 不直接删除 (保留历史 Provenance) | P1 |
| US-19 | SRE | 作为 SRE, 我希望 Lock / Unlock Worktree, 防止其他 Agent 同时操作同一 Worktree | P0 |
| US-20 | Dev | 作为 Dev, 我希望 Set Dependency (A 依赖 B 完成才能 Merge), 系统在 B 未合并时禁止 A Merge 并提示原因 | P1 |
| US-21 | Dev | 作为 Dev, 我希望 Sync Main / Rebase / Merge / Create PR 等危险操作分类 (Safe / Warning / Destructive), Destructive 必须二次确认 | P0 |
| US-22 | PM | 作为 PM, 我希望 Explanation Layer 告诉我 "为什么这个 Worktree 是 CONFLICT 状态" — 自然语言解释, 不只是 `conflict=true` | P0 |
| US-23 | Dev | 作为 Dev, 我希望 Compare 2 个 Worktree, 看到文件级 + Symbol 级差异, 不需要打开 GitHub | P0 |
| US-24 | SRE | 作为 SRE, 我希望 Cleanup 一键删除所有 STALE 状态且 Last Activity > 7d 的 Worktree, 分类为 Destructive 二次确认 | P1 |
| US-25 | PM | 作为 PM, 我希望 Archive 一个 Worktree (不再 Active 但保留历史), 区别于 Delete | P1 |
| US-26 | Dev | 作为 Dev, 我希望 Mark Superseded (A 取代 B), 系统在 B 的节点上加角标, 不让用户再 Merge B | P2 |
| US-27 | 5 域 Lead | 作为 Lead, 我希望看到 100+ Worktree 时画布不卡顿 (LOD + Viewport Virtualization 性能策略) | P0 |
| US-28 | SRE | 作为 SRE, 我希望 1000 Worktree 时画布仍可交互 (Graph Clustering + Lazy Expansion) | P1 |
| US-29 | Dev | 作为 Dev, 我希望看到 Health Score 0-100 的变化趋势 (Last 24h 折线), 不是单点 | P2 |
| US-30 | Dev | 作为 Dev, 我希望键盘快捷键全操作 (V/H/+/-/1/2/3/F/S/Esc) | P1 |
| US-31 | PM | 作为 PM, 我希望 AGENT VIEW 按 Agent 维度聚类, Task → Agent → Worktree 树形拓扑 | P1 |
| US-32 | PM | 作为 PM, 我希望 HISTORY VIEW 看到已合并 Worktree 历史 + SUPERSEDES 关系 | P2 |
| US-33 | Dev | 作为 Dev, 我希望 Search DSL 支持组合 (`show:unmerged behind:>10 agent:codex`) | P1 |
| US-34 | PM | 作为 PM, 我希望自然语言查询翻译为 Graph Query + Git Filter | P1 |
| US-35 | SRE | 作为 SRE, 我希望 Focus Mode 支持 2-hop / 3-hop Neighborhood, 看更深层依赖链 | P1 |
| US-36 | Dev | 作为 Dev, 我希望 Minimap + 自动居中 + Fit to View, 1000 Worktree 时快速定位 | P1 |
| US-37 | Dev | 作为 Dev, 我希望 Inspector 的 Relations tab 显示所有相关 Edge (13 类) + reason + risk_score | P0 |
| US-38 | PM | 作为 PM, 我希望 HISTORY tab 显示该 Worktree 派生链 (基于 DERIVED_FROM 反向遍历) | P2 |
| US-39 | SRE | 作为 SRE, 我希望查看 Audit Log (谁在何时 Sync / Merge / Delete / Lock 哪个 Worktree) | P1 |

**用户故事总数**: **39 项** (per §30 输出要求 §8 使用场景 + §7 用户故事 满足)

---

## §2 用語定义 (Ubiquitous Language)

| 用語 | 定义 | 出处 / 备注 |
|---|---|---|
| Worktree | Git worktree (per git-worktree(1)), AI Agent 并行开发的核心实体; 第一版 UI 默认显示节点 | per issue §七 |
| Mainline | Repository 的 primary branch (e.g. main / master / develop), 所有 Worktree 默认派生源头 | 本 SRS 新增 (per §四 视觉模型) |
| Project | 1 个用户级工程容器, 包含 1+ Repository + 1+ Mainline + 1+ Worktree | 本 SRS 新增 (per §七 节点) |
| Repository | Git repository, Worktree 容器, L0 zoom 唯一显示节点 | per issue §七 |
| Branch | Git branch (本地 / 远程), Worktree 通过 USES_BRANCH 关联 | per issue §七 |
| Task | 项目任务卡 (per `SRS-STAR-AGENT-RUNTIME-001.md` 6 状态机), 通过 IMPLEMENTED_IN 关联 Worktree | per issue §七 |
| AgentSession | 1 个 AI Agent 执行的会话实例 (per `SRS-STAR-AGENT-RUNTIME-001.md` 14 状态机), 通过 WORKS_ON 关联 Worktree | per issue §十五 |
| Commit | Git commit object, L3 zoom 展开节点, 默认折叠 | per issue §七 |
| PullRequest | GitHub / GitLab PR, 通过 REVIEWS 关联 Worktree | per issue §七 |
| File | 源代码文件, L4 zoom 展开节点, 通过 MODIFIES 关联 Worktree | per issue §七 |
| Symbol | 源代码符号 (function / class / variable / type), L5 zoom 展开节点, 通过 MODIFIES_SYMBOL 关联 Worktree | per issue §七 |
| TestRun | 测试运行结果, 通过 VALIDATES 关联 Worktree | per issue §七 |
| Issue | GitHub / GitLab Issue, L3 zoom 展开节点 | per issue §七 |
| Worktree 状态 (Human State) | RUNNING / WAITING / READY / DIVERGED / CONFLICT / MERGED / STALE 7 种 | 本 SRS 新增 (per §九) |
| Worktree 状态 (Machine State) | 内部更细粒度状态 (e.g. agent_executing / git_clean / git_dirty / merge_in_progress / ci_running), UI 第一层映射到 Human State | 本 SRS 新增 (per §九) |
| Health Score | Worktree 健康评分, 0-100, 13 影响因素, 可解释 | per issue §十二 |
| Risk | 风险事件, 11 种类型 (MergeConflict / FileOverlap / SymbolOverlap / Stale / Divergence / Dependency / TestFailure / BuildFailure / AgentIncomplete / ReviewMissing / Superseded) | per issue §十三 |
| Risk Engine V1 | 基于 Git Status + Ahead/Behind + File overlap 的风险检测 | per issue §十三 |
| Risk Engine V2 | 基于 Diff overlap + Line overlap 的风险检测 | per issue §十三 |
| Risk Engine V3 | 基于 Symbol overlap + AST-level overlap 的风险检测 | per issue §十三 |
| Conflict Prediction | 在 `git merge` 之前发现潜在冲突 (WT-A mod file X, WT-B mod file X) | per issue §十四 |
| Focus Mode | 点击某 Worktree 后, 当前节点突出, 1/2/3-hop 邻居高亮, 其他节点降透明度 | per issue §十七 |
| N-hop Neighborhood | Graph Neighborhood 查询, 1-hop = 直接邻居, 2-hop = 2 层传递, 3-hop = 3 层传递 | 本 SRS 新增 |
| Semantic Zoom | 缩放画布时改变信息粒度 (L0-L5), 不是单纯缩放 UI | per issue §六 + R6 |
| View Mode | TREE / DEPENDENCY / RISK / AGENT / HISTORY 5 种视图 | per issue §十八 |
| LOD (Level of Detail) | 性能策略: 远距离只显示粗粒度节点 (Repository), 近距离才显示细粒度 (Symbol) | per issue §二十七 |
| Viewport Virtualization | 性能策略: 只渲染当前视口内的节点, 视口外节点不渲染 | per issue §二十七 |
| Graph Clustering | 性能策略: 远距离把多节点合并为 Cluster 节点, 近距离才展开 | per issue §二十七 |
| Lazy Expansion | 性能策略: 节点默认折叠, 用户点开才展开子节点 | per issue §二十七 |
| Incremental Layout | 性能策略: 节点位置增量计算, 不全量重排 | per issue §二十七 |
| Incremental Query | 性能策略: Graph Query 增量, 不每次全量查询 | per issue §二十七 |
| Graph Backbone | TREE VIEW 默认结构 (Main → Worktree 派生树) | 本 SRS 新增 |
| Semantic Graph Edge | 非 TREE 派生关系的边 (CONFLICTS_WITH / DEPENDS_ON / BLOCKS / SUPERSEDES 等) | per issue §四 |
| Search DSL | 结构化搜索语法 (`show:conflict behind:>20 agent:codex health:<50`) | per issue §二十 |
| NL Query | 自然语言查询, AI 翻译为 Graph Query + Git Filter | per issue §二十 |
| Inspector | 右侧固定面板, 11 tab (Overview/Git/Task/Agent/Risk/Tests/Files/Commits/Relations/History/Actions) | per issue §二十一 |
| AI Explanation | 基于 Graph + Git + Risk Engine 的自然语言解释, 不凭空生成 | per issue §十六 |
| Merge Readiness | Worktree 可安全合并的判断 (Health Score >= 80 + 无 Conflict + Ahead == 0 + 无 Pending Dependency) | 本 SRS 新增 |
| Action 分类 | Safe (无副作用) / Warning (有副作用, 单次确认) / Destructive (高风险, 二次确认 + 审计) | per issue §二十二 |
| Event Bus | 15 个事件 (WorktreeCreated/Changed/Merged/Deleted + BranchUpdated + MainUpdated + AgentStarted/Stopped + TaskChanged + TestCompleted + RiskDetected/Resolved + HealthChanged + RelationCreated/Removed) | per issue §二十五 |
| Graph Repository | 抽象图数据库接口 (`GraphRepository Trait`), 底层 Neo4j / Memgraph / Embedded / Custom 可替换 | per issue §三十三 |
| Canvas Renderer | 抽象画布渲染接口 (`CanvasRenderer Trait`), 备选 React-Flow / PixiJS / 自研 WebGL/Canvas | 本 SRS 新增 |
| Git Provider | 抽象 Git 适配接口 (`GitProvider Trait`), 底层 git CLI / libgit2 / Gitoxide 可替换 | 本 SRS 新增 |
| Agent Runtime | 抽象 Agent 适配接口 (`AgentRuntime Trait`), 底层 Multica / Claude Code / Codex 可替换 | per issue §三十三 |
| Source of Truth | 数据真源: Git Source of Truth (Branch/Commit/Ref/Worktree/Diff/Merge-base/Ahead/Behind/Status) | per issue §二十三 |
| Derived Data | 派生数据: Graph Derived Data (Dependency/Conflict/Overlap/Superseded/Risk/Health) + AI Derived Explanation | per issue §二十三 |
| Non-destructive Visualization | R9 原则: 画布不直接修改 Git 数据, 高风险操作走 Action 流程 | per issue §二 + §九 |
| Tree Backbone + Graph Edge | 视觉结构: 默认 Tree Backbone (派生关系) + Semantic Graph Edge (语义关系) | per issue §四 |
| Provenance | Worktree 派生历史 (基于 DERIVED_FROM 反向遍历), 即使 Merged 后保留 | per issue §十九 |
| Recently Merged | 已合并 Worktree 折叠区域, 默认 24h / 7d / All 过滤 | per issue §十九 |

---

## §3 业务背景 / 前提条件

### 3.1 业务背景

- STAR / Multica 平台已落地 22 个 domain-* crate + 47 个 workspace package + 无限画布 V0.1 (`frontend-canvas-design.md` v0.1) + Agent View V0.1 (`SRS-AGENT-VIEW-001.md` v1.0) + ARG V0.1 (`SRS-AGENT-RELATIONSHIP-001.md` v0.1)
- **新痛点** (per 2026-09-15 Multica ULYS-57 issue 创建者发令): AI Agent 并行开发场景下, 单 Repository 可同时存在 10-1000 Worktree, 人类无法以低认知负荷监督
- 现有 12 个 worktree mock seed (per `frontend/src/lib/seed.ts`)
- 5 域 (player/economy/match/social/admin) Lead 真人未到位, Mavis 临时代签 (per `AGENTS.md §4 守门 #3 v2`)

### 3.2 前提条件

- P-1: zustand store 已落地 worktrees / workItems / agentSessions / branches 集合 (per `frontend/src/lib/store`)
- P-2: StatusPill 组件已落地 60+ 状态色码 (per `frontend-canvas-design.md` §2.3)
- P-3: 路由系统已就绪 (Next.js 14.2.5 App Router, 跟 V0.1 Canvas 一致)
- P-4: i18n 系统已就绪 (3 语言 zh-CN / en / ja)
- P-5: SVG 基础工具已落地 (lucide-react / recharts 等)
- P-6: Graph 抽象层尚未落地 (per `multica-platform` skill, 决策中: Neo4j vs Memgraph vs Embedded)
- P-7: Risk Engine 尚未落地 (本 SRS 全新模块)
- P-8: Git Adapter 尚未落地 (per `multica-platform` skill, 决策中: git CLI vs libgit2 vs Gitoxide)
- P-9: Agent Bridge 尚未落地 (per `SRS-STAR-AGENT-RUNTIME-001.md` v1.0, 实装期选定 Agent Runtime)
- P-10: 不引入 Vue / Angular 等新框架 (复用 Next.js 14.2.5)
- P-11: 数据库 = PostgreSQL 15+ (per `mavis` desktop 现有栈), Graph 数据可存 PG JSONB 或专用 Graph DB

### 3.3 业务规则 (Business Rules)

- BR-1: 一个 Worktree 同一时刻只能处于 1 个 Human State (互斥)
- BR-2: Human State ↔ Machine State 是 1:N 映射 (per §九)
- BR-3: 一个 Worktree 派生自 1 个 Parent Worktree (或 Mainline), 通过 DERIVED_FROM Edge
- BR-4: 多个 Worktree 可通过 DEPENDS_ON / BLOCKS / CONFLICTS_WITH / OVERLAPS_WITH / SUPERSEDES 互相关联
- BR-5: 一个 Agent Session 同一时刻只能 WORKS_ON 1 个 Worktree (per `SRS-STAR-AGENT-RUNTIME-001.md` BR)
- BR-6: 一个 Task 通过 IMPLEMENTED_IN 关联 1+ Worktree (1:N)
- BR-7: Health Score ∈ [0, 100], 0 = 极差, 100 = 完美, 13 影响因素可拆解 (per §十二)
- BR-8: Risk Score ∈ [0, 1], 0 = 无风险, 1 = 极高风险, 11 种 Risk 类型可叠加
- BR-9: AI Explanation 必须基于 Graph + Git + Risk 结构化数据, 禁止凭空生成
- BR-10: 画布默认 UI 不直接修改 Git 数据 (R9 Non-destructive Visualization), 高风险操作走 Action 流程
- BR-11: Source of Truth 不可被 AI 或画布修改, 仅 Derived Data 可被 Graph Risk Engine 写入
- BR-12: Merged Worktree 必须保留 Provenance (基于 DERIVED_FROM 反向遍历), 不直接删除
- BR-13: Action 分类 = Safe / Warning / Destructive, Destructive 必须二次确认 + 写 Audit Log
- BR-14: Event Bus 15 个事件, 4 类消费者 (Graph / UI / Risk Engine / Notification) 解耦
- BR-15: Performance 默认 100 Worktree 流畅, 1000 Worktree 可交互 (LOD + Viewport Virtualization)
- BR-16: Real-time 通过 Git Observer 监听 HEAD / Refs / Working Tree / Index / Worktree List / Agent Runtime / Test State, 但事件合并 + 防抖避免全量重算
- BR-17: Graph Repository 必须抽象为 Trait, 4 备选 (Neo4j / Memgraph / Embedded / Custom), MVP 选定 1 个推荐方案
- BR-18: Canvas Renderer 必须抽象为 Trait, 备选 React-Flow / PixiJS / 自研, MVP 选定 1 个推荐方案
- BR-19: Git Provider 必须抽象为 Trait, 备选 git CLI / libgit2 / Gitoxide, MVP 选定 1 个推荐方案
- BR-20: 视图模式 = 5 种 (TREE / DEPENDENCY / RISK / AGENT / HISTORY), 切换不重算数据, 仅切换 Edge 颜色 / 节点透明度

---

## §4 业务需求

### 4.1 功能需求 (Functional Requirements) — FR-WT (Worktree 核心)

#### 4.1.1 FR-WT-001 Worktree 节点展示

| 项 | 内容 |
|---|---|
| ID | FR-WT-001 |
| 描述 | 系统应在无限画布上为每个 Worktree 渲染 1 个节点卡片, 默认突出显示 |
| 输入 | `Worktree[]` (from Git Source of Truth + Graph Derived Data) |
| 输出 | Canvas 上 N 个 Worktree Node |
| 业务规则 | BR-1 + R2 Worktree-first |
| 优先级 | P0 |

#### 4.1.2 FR-WT-002 Worktree 状态压缩 (7 态)

| 项 | 内容 |
|---|---|
| ID | FR-WT-002 |
| 描述 | 系统应把 Worktree 内部 Machine State 压缩为 7 种 Human State (RUNNING / WAITING / READY / DIVERGED / CONFLICT / MERGED / STALE) |
| 输入 | Machine State (内部细粒度) |
| 输出 | Human State (7 选 1) |
| 业务规则 | BR-1 + §九 状态模型 |
| 优先级 | P0 |

#### 4.1.3 FR-WT-003 Worktree 卡片字段 (11 项)

| 项 | 内容 |
|---|---|
| ID | FR-WT-003 |
| 描述 | 系统应在 Worktree Node 卡片显示 11 项字段: Worktree Name / Branch Name / Human State / Agent / Task / Ahead / Behind / Dirty State / Health Score / Last Activity / Test State / Risk Count |
| 输入 | Worktree + Graph Derived Data |
| 输出 | 卡片 UI |
| 业务规则 | §十 卡片定义; 默认禁止展示长 commit hash / 复杂 refs / 内部 Git 对象 |
| 优先级 | P0 |

#### 4.1.4 FR-WT-004 Worktree 状态机

| 项 | 内容 |
|---|---|
| ID | FR-WT-004 |
| 描述 | 系统应实现 7 态状态机, 每态定义进入条件 / 离开条件 / 优先级 / UI 表现 / 可执行动作 / 风险级别 / 是否需用户处理 |
| 输入 | Git Event + Agent Event + Risk Event |
| 输出 | Human State 迁移 |
| 业务规则 | BR-1 + BR-2 + §九 |
| 优先级 | P0 |

#### 4.1.5 FR-WT-005 Ahead / Behind 可视化 (3 维)

| 项 | 内容 |
|---|---|
| ID | FR-WT-005 |
| 描述 | 系统应同时用 3 维表达 Ahead / Behind: (1) 数字 (2) 空间距离 (3) Edge 信息; 视觉距离不得严格线性映射, 可用 `visualDistance = log(commitDistance + 1)` |
| 输入 | Git `git rev-list --count` 输出 |
| 输出 | Canvas 上 Ahead / Behind Edge 渲染 |
| 业务规则 | §十一 |
| 优先级 | P0 |

#### 4.1.6 FR-WT-006 Health Score (0-100, 13 因素)

| 项 | 内容 |
|---|---|
| ID | FR-WT-006 |
| 描述 | 系统应为每个 Worktree 计算 Health Score ∈ [0, 100], 13 因素加权 (Conflict / Behind / Ahead / Dirty / Test / Build / Review / Agent / 活跃时间 / 文件重叠 / Symbol 重叠 / 依赖 / 阻塞), 可解释 (每扣分项可查看理由) |
| 输入 | 13 因素状态 |
| 输出 | `HealthScore { value: u8, deductions: Deduction[] }` |
| 业务规则 | BR-7 + §十二 |
| 优先级 | P0 |

#### 4.1.7 FR-WT-007 Worktree 颜色编码 (R7 Exception-first)

| 项 | 内容 |
|---|---|
| ID | FR-WT-007 |
| 描述 | 系统应根据 Human State 用色码编码, CONFLICT / STALE / DIVERGED 高饱和, MERGED 灰度, READY 绿色, RUNNING 蓝色, WAITING 黄色 |
| 输入 | Human State |
| 输出 | 节点颜色 + 边框 |
| 业务规则 | R7 + §三十四 UI 原则 (不能只靠颜色) |
| 优先级 | P0 |

#### 4.1.8 FR-WT-008 Worktree 图标 + 文本双重编码

| 项 | 内容 |
|---|---|
| ID | FR-WT-008 |
| 描述 | 系统应同时用图标 (e.g. ⚠️ Conflict / 🕐 Stale / ✅ Ready) 和文本编码状态, 不允许只靠颜色 (色盲 + i18n) |
| 输入 | Human State |
| 输出 | 节点图标 + 文本 |
| 业务规则 | §三十四 |
| 优先级 | P0 |

#### 4.1.9 FR-WT-009 Worktree Dirty State 指示

| 项 | 内容 |
|---|---|
| ID | FR-WT-009 |
| 描述 | 系统应在卡片显示 Dirty 状态 (有未提交修改), 用角标或图标 |
| 输入 | Git `git status --porcelain` |
| 输出 | 角标 |
| 业务规则 | §十 卡片字段 |
| 优先级 | P0 |

#### 4.1.10 FR-WT-010 Worktree Last Activity

| 项 | 内容 |
|---|---|
| ID | FR-WT-010 |
| 描述 | 系统应在卡片显示 Last Activity (ISO 8601 → 相对时间 e.g. "5 分钟前"), 超过 7d 高亮 STALE |
| 输入 | Git log + Agent activity + Test activity |
| 输出 | 相对时间字符串 |
| 业务规则 | §十 |
| 优先级 | P0 |

#### 4.1.11 FR-WT-011 Worktree Test State

| 项 | 内容 |
|---|---|
| ID | FR-WT-011 |
| 描述 | 系统应在卡片显示 Test 状态 (passed / failed / running / none), 用图标 + 文本双重编码 |
| 输入 | TestRun Node (VALIDATES Edge) |
| 输出 | 测试状态 UI |
| 业务规则 | §十 + §七 |
| 优先级 | P0 |

#### 4.1.12 FR-WT-012 Worktree Risk Count

| 项 | 内容 |
|---|---|
| ID | FR-WT-012 |
| 描述 | 系统应在卡片显示 Risk Count (关联 Risk 数量), > 0 时高亮 |
| 输入 | Graph Risk Engine |
| 输出 | 风险角标 |
| 业务规则 | §十 + §十三 |
| 优先级 | P0 |

#### 4.1.13 FR-WT-013 Worktree Lock / Unlock

| 项 | 内容 |
|---|---|
| ID | FR-WT-013 |
| 描述 | 系统应支持 Lock / Unlock Worktree, Locked 节点加锁图标, 其他 Agent 操作被拒绝 |
| 输入 | User Action |
| 输出 | Locked 状态 + Audit Log |
| 业务规则 | §二十二 + BR-13 |
| 优先级 | P0 |

#### 4.1.14 FR-WT-014 Worktree Archive

| 项 | 内容 |
|---|---|
| ID | FR-WT-014 |
| 描述 | 系统应支持 Archive Worktree (保留历史但不再 Active), 区别于 Delete |
| 输入 | User Action |
| 输出 | Archived 状态 |
| 业务规则 | §二十二 |
| 优先级 | P1 |

#### 4.1.15 FR-WT-015 Worktree Mark Superseded

| 项 | 内容 |
|---|---|
| ID | FR-WT-015 |
| 描述 | 系统应支持 Mark Superseded (A 取代 B), 在 B 节点加角标, 不允许 Merge B |
| 输入 | User Action |
| 输出 | SUPERSEDES Edge + 角标 |
| 业务规则 | §二十二 + §八 |
| 优先级 | P2 |

#### 4.1.16 FR-WT-016 Worktree Cleanup

| 项 | 内容 |
|---|---|
| ID | FR-WT-016 |
| 描述 | 系统应支持 Cleanup 一键删除所有 STALE + Last Activity > 7d 的 Worktree, 分类 Destructive, 二次确认 + Audit |
| 输入 | User Action |
| 输出 | 批量删除 |
| 业务规则 | §二十二 + BR-13 |
| 优先级 | P1 |

#### 4.1.17 FR-WT-017 Worktree 派生链显示 (Provenance)

| 项 | 内容 |
|---|---|
| ID | FR-WT-017 |
| 描述 | 系统应在 Inspector 的 History tab 显示该 Worktree 派生链 (基于 DERIVED_FROM 反向遍历) |
| 输入 | DERIVED_FROM Edge |
| 输出 | 派生链 UI |
| 业务规则 | §十九 + BR-12 |
| 优先级 | P2 |

#### 4.1.18 FR-WT-018 Worktree Merge Readiness 判断

| 项 | 内容 |
|---|---|
| ID | FR-WT-018 |
| 描述 | 系统应计算每个 Worktree 的 Merge Readiness: Health >= 80 + 无 Conflict + Ahead == 0 + 无 Pending Dependency = READY |
| 输入 | Health + Risk + Ahead + Dependency |
| 输出 | `MergeReadiness { ready: bool, blockers: BlockReason[] }` |
| 业务规则 | §二 + §二十二 |
| 优先级 | P0 |

#### 4.1.19 FR-WT-019 Worktree 视图模式切换 (5 模式)

| 项 | 内容 |
|---|---|
| ID | FR-WT-019 |
| 描述 | 系统应支持 TREE / DEPENDENCY / RISK / AGENT / HISTORY 5 种视图模式切换, 不重算数据, 仅切换 Edge 颜色 / 节点透明度 |
| 输入 | User Mode Selection |
| 输出 | Canvas 渲染参数 |
| 业务规则 | §十八 + BR-20 |
| 优先级 | P0 |

#### 4.1.20 FR-WT-020 Worktree TREE VIEW (派生树)

| 项 | 内容 |
|---|---|
| ID | FR-WT-020 |
| 描述 | 系统应实现 TREE VIEW: 以 Main → Worktree 派生为核心, DERIVED_FROM 边为骨架 |
| 输入 | Worktree + DERIVED_FROM |
| 输出 | TREE 视图渲染 |
| 业务规则 | §十八 |
| 优先级 | P0 |

#### 4.1.21 FR-WT-021 Worktree DEPENDENCY VIEW

| 项 | 内容 |
|---|---|
| ID | FR-WT-021 |
| 描述 | 系统应实现 DEPENDENCY VIEW: 突出 DEPENDS_ON / BLOCKS 边 (粗线高亮), TREE 边降透明度 |
| 输入 | DEPENDS_ON + BLOCKS Edge |
| 输出 | DEPENDENCY 视图渲染 |
| 业务规则 | §十八 |
| 优先级 | P1 |

#### 4.1.22 FR-WT-022 Worktree RISK VIEW

| 项 | 内容 |
|---|---|
| ID | FR-WT-022 |
| 描述 | 系统应实现 RISK VIEW: 正常节点透明度 20%, 高风险节点 100% + 边框发光 |
| 输入 | Worktree + Risk |
| 输出 | RISK 视图渲染 |
| 业务规则 | §十八 + R7 |
| 优先级 | P1 |

#### 4.1.23 FR-WT-023 Worktree AGENT VIEW

| 项 | 内容 |
|---|---|
| ID | FR-WT-023 |
| 描述 | 系统应实现 AGENT VIEW: 按 Agent 维度聚类, Task → Agent → Worktree 树形拓扑 |
| 输入 | AgentSession + Task + Worktree |
| 输出 | AGENT 视图渲染 |
| 业务规则 | §十八 + §十五 |
| 优先级 | P1 |

#### 4.1.24 FR-WT-024 Worktree HISTORY VIEW

| 项 | 内容 |
|---|---|
| ID | FR-WT-024 |
| 描述 | 系统应实现 HISTORY VIEW: 显示已合并 Worktree + SUPERSEDES 历史 |
| 输入 | MERGED Worktree + SUPERSEDES Edge |
| 输出 | HISTORY 视图渲染 |
| 业务规则 | §十八 + §十九 |
| 优先级 | P2 |

#### 4.1.25 FR-WT-025 Worktree Merged 处理

| 项 | 内容 |
|---|---|
| ID | FR-WT-025 |
| 描述 | 系统应将 Merged Worktree 默认折叠到 "Recently Merged" 抽屉, 支持 24h / 7d / All 过滤, 保留 Provenance |
| 输入 | MERGED Worktree |
| 输出 | 折叠抽屉 + 过滤 |
| 业务规则 | §十九 + BR-12 |
| 优先级 | P1 |

#### 4.1.26 FR-WT-026 Worktree 双击跳详情

| 项 | 内容 |
|---|---|
| ID | FR-WT-026 |
| 描述 | 系统应在双击 Worktree 卡片时, 右侧 Inspector 展开, 不弹 Modal |
| 输入 | User Action (double-click) |
| 输出 | Inspector 打开 |
| 业务规则 | §二十一 + R9 |
| 优先级 | P0 |

#### 4.1.27 FR-WT-027 Worktree Health Score 趋势

| 项 | 内容 |
|---|---|
| ID | FR-WT-027 |
| 描述 | 系统应在 Inspector 显示 Health Score Last 24h 折线图 (1h 间隔, 24 个数据点) |
| 输入 | HealthScore history |
| 输出 | recharts 折线图 |
| 业务规则 | §十二 |
| 优先级 | P2 |

#### 4.1.28 FR-WT-028 Worktree Health Score 扣分明细

| 项 | 内容 |
|---|---|
| ID | FR-WT-028 |
| 描述 | 系统应在 Inspector 显示 Health Score 13 影响因素扣分明细, 每项可点开看原因 |
| 输入 | HealthScore deductions |
| 输出 | 扣分明细 UI |
| 业务规则 | §十二 + R8 Explainable State |
| 优先级 | P0 |

#### 4.1.29 FR-WT-029 Worktree Compare 2 个 Worktree

| 项 | 内容 |
|---|---|
| ID | FR-WT-029 |
| 描述 | 系统应支持 Compare 2 个 Worktree, 显示文件级 + Symbol 级差异 |
| 输入 | 2 Worktree IDs |
| 输出 | Diff View |
| 业务规则 | §二十二 |
| 优先级 | P0 |

#### 4.1.30 FR-WT-030 Worktree Open in IDE

| 项 | 内容 |
|---|---|
| ID | FR-WT-030 |
| 描述 | 系统应支持 Open in IDE (VSCode / IntelliJ / Cursor), 跳转 `vscode://file/{path}` 或类似协议 |
| 输入 | User Action |
| 输出 | IDE 打开 |
| 业务规则 | §二十二 |
| 优先级 | P1 |

#### 4.1.31 FR-WT-031 Worktree Set Dependency

| 项 | 内容 |
|---|---|
| ID | FR-WT-031 |
| 描述 | 系统应支持 Set Dependency (A 依赖 B 完成才能 Merge), B 未合并时禁止 A Merge 并提示原因 |
| 输入 | 2 Worktree IDs |
| 输出 | DEPENDS_ON Edge + Merge 拦截 |
| 业务规则 | §二十二 + §八 |
| 优先级 | P1 |

#### 4.1.32 FR-WT-032 Worktree Remove Dependency

| 项 | 内容 |
|---|---|
| ID | FR-WT-032 |
| 描述 | 系统应支持 Remove Dependency, 删除 DEPENDS_ON Edge, 不删其他数据 |
| 输入 | Edge ID |
| 输出 | Edge 删除 |
| 业务规则 | §二十二 |
| 优先级 | P1 |

#### 4.1.33 FR-WT-033 Worktree Sync Main

| 项 | 内容 |
|---|---|
| ID | FR-WT-033 |
| 描述 | 系统应支持 Sync Main (git fetch + rebase onto main), 分类 Warning, 二次确认 |
| 输入 | Worktree ID |
| 输出 | Rebase 执行 |
| 业务规则 | §二十二 + BR-13 |
| 优先级 | P0 |

#### 4.1.34 FR-WT-034 Worktree Rebase

| 项 | 内容 |
|---|---|
| ID | FR-WT-034 |
| 描述 | 系统应支持 Rebase 到指定 Branch, 分类 Warning |
| 输入 | Worktree ID + Target Branch |
| 输出 | Rebase 执行 |
| 业务规则 | §二十二 + BR-13 |
| 优先级 | P1 |

#### 4.1.35 FR-WT-035 Worktree Merge

| 项 | 内容 |
|---|---|
| ID | FR-WT-035 |
| 描述 | 系统应支持 Merge Worktree 到 Main, 分类 Destructive (影响 Main), 二次确认 + Audit |
| 输入 | Worktree ID |
| 输出 | Merge 执行 |
| 业务规则 | §二十二 + BR-13 + Merge Readiness 检查 |
| 优先级 | P0 |

#### 4.1.36 FR-WT-036 Worktree Create PR

| 项 | 内容 |
|---|---|
| ID | FR-WT-036 |
| 描述 | 系统应支持 Create PR (GitHub / GitLab API), 分类 Safe |
| 输入 | Worktree ID + Title + Body |
| 输出 | PR 创建 |
| 业务规则 | §二十二 |
| 优先级 | P0 |

#### 4.1.37 FR-WT-037 Worktree Delete

| 项 | 内容 |
|---|---|
| ID | FR-WT-037 |
| 描述 | 系统应支持 Delete Worktree (git worktree remove), 分类 Destructive, 二次确认 + Audit |
| 输入 | Worktree ID |
| 输出 | Worktree 删除 |
| 业务规则 | §二十二 + BR-13 |
| 优先级 | P0 |

#### 4.1.38 FR-WT-038 Worktree Focus (1/2/3-hop)

| 项 | 内容 |
|---|---|
| ID | FR-WT-038 |
| 描述 | 系统应支持 Focus Mode (1/2/3-hop Neighborhood), 当前节点突出 + 邻居高亮 + 其他节点降透明度 |
| 输入 | Worktree ID + hop count |
| 输出 | Canvas 渲染参数 |
| 业务规则 | §十七 |
| 优先级 | P0 |

### 4.2 功能需求 — FR-UI (无限画布 UI)

#### 4.2.1 FR-UI-001 无限画布 (Infinite Canvas)

| 项 | 内容 |
|---|---|
| ID | FR-UI-001 |
| 描述 | 系统应实现 Miro 风格无限画布, 支持无限平移 + 缩放 + 自动布局 + 手动拖动 + 节点折叠/展开 + 局部聚焦 + 多选 + 框选 |
| 输入 | 节点 + 边集合 |
| 输出 | Canvas 渲染 |
| 业务规则 | §五 |
| 优先级 | P0 |

#### 4.2.2 FR-UI-002 搜索定位

| 项 | 内容 |
|---|---|
| ID | FR-UI-002 |
| 描述 | 系统应在画布顶部搜索框, 支持 Search DSL + 自然语言查询, 命中节点自动居中 + 高亮 |
| 输入 | Query String |
| 输出 | Canvas 视图调整 |
| 业务规则 | §二十 + §五 |
| 优先级 | P0 |

#### 4.2.3 FR-UI-003 Minimap

| 项 | 内容 |
|---|---|
| ID | FR-UI-003 |
| 描述 | 系统应在画布右下角显示 Minimap, 含 viewport 矩形 + 节点缩略, 支持点击跳转 |
| 输入 | Canvas viewport |
| 输出 | Minimap 渲染 |
| 业务规则 | §五 |
| 优先级 | P1 |

#### 4.2.4 FR-UI-004 自动居中 + Fit to View

| 项 | 内容 |
|---|---|
| ID | FR-UI-004 |
| 描述 | 系统应支持自动居中 (Auto Center) + Fit to View (一键缩放到全图), 快捷键 1 |
| 输入 | User Action |
| 输出 | Viewport 调整 |
| 业务规则 | §五 |
| 优先级 | P0 |

#### 4.2.5 FR-UI-005 Focus Mode (UI)

| 项 | 内容 |
|---|---|
| ID | FR-UI-005 |
| 描述 | 系统应在 Focus Mode 时, 当前节点 100% 透明度, 1-hop 邻居 80%, 其他节点 20% |
| 输入 | Focus Worktree ID + hop |
| 输出 | 节点透明度参数 |
| 业务规则 | §十七 + §五 |
| 优先级 | P0 |

#### 4.2.6 FR-UI-006 图层控制

| 项 | 内容 |
|---|---|
| ID | FR-UI-006 |
| 描述 | 系统应支持图层控制 (Layer Panel), 显示/隐藏特定 Node 类型 (Repository / Commit / File / Symbol) + Edge 类型 (CONFLICTS / DEPENDS_ON) |
| 输入 | User Toggle |
| 输出 | 图层可见性 |
| 业务规则 | §五 |
| 优先级 | P1 |

#### 4.2.7 FR-UI-007 视图模式切换 UI

| 项 | 内容 |
|---|---|
| ID | FR-UI-007 |
| 描述 | 系统应在顶部工具栏显示 5 个视图模式 Tab (TREE / DEPENDENCY / RISK / AGENT / HISTORY), 切换不重算数据 |
| 输入 | User Click |
| 输出 | 视图模式 |
| 业务规则 | §十八 + BR-20 |
| 优先级 | P0 |

#### 4.2.8 FR-UI-008 状态过滤

| 项 | 内容 |
|---|---|
| ID | FR-UI-008 |
| 描述 | 系统应支持按 Human State 过滤 (e.g. 只看 RUNNING + WAITING), 多选 |
| 输入 | State Filter |
| 输出 | 节点可见性 |
| 业务规则 | §五 + §二十 |
| 优先级 | P0 |

#### 4.2.9 FR-UI-009 Edge 类型过滤

| 项 | 内容 |
|---|---|
| ID | FR-UI-009 |
| 描述 | 系统应支持按 Edge 类型过滤 (e.g. 只显示 CONFLICTS_WITH + DEPENDS_ON) |
| 输入 | Edge Filter |
| 输出 | 边可见性 |
| 业务规则 | §五 + §二十 |
| 优先级 | P1 |

#### 4.2.10 FR-UI-010 节点拖动

| 项 | 内容 |
|---|---|
| ID | FR-UI-010 |
| 描述 | 系统应支持手动拖动节点 (非派生视图, 允许用户自定义布局) |
| 输入 | User Drag |
| 输出 | 节点坐标更新 |
| 业务规则 | §五 + R9 |
| 优先级 | P0 |

#### 4.2.11 FR-UI-011 节点折叠/展开

| 项 | 内容 |
|---|---|
| ID | FR-UI-011 |
| 描述 | 系统应支持节点折叠 (隐藏子节点) / 展开 (显示子节点), L3+ 默认折叠 |
| 输入 | User Toggle |
| 输出 | 子节点可见性 |
| 业务规则 | §五 |
| 优先级 | P0 |

#### 4.2.12 FR-UI-012 多选 + 框选

| 项 | 内容 |
|---|---|
| ID | FR-UI-012 |
| 描述 | 系统应支持多选 (Ctrl/Cmd + Click) + 框选 (drag empty area) |
| 输入 | User Selection |
| 输出 | Selected Set |
| 业务规则 | §五 |
| 优先级 | P1 |

#### 4.2.13 FR-UI-013 键盘快捷键

| 项 | 内容 |
|---|---|
| ID | FR-UI-013 |
| 描述 | 系统应支持键盘快捷键全操作: V (切换视图) / H (隐藏 Merged) / + - (zoom) / 1 (Fit) / 2 (Center) / 3 (Focus) / F (搜索) / Esc (退出 Focus) |
| 输入 | Keyboard Event |
| 输出 | Canvas 动作 |
| 业务规则 | §五 + §三十四 |
| 优先级 | P1 |

#### 4.2.14 FR-UI-014 Inspector (右侧固定面板)

| 项 | 内容 |
|---|---|
| ID | FR-UI-014 |
| 描述 | 系统应实现右侧固定 Inspector, 11 tab (Overview / Git State / Task / Agent / Risk / Tests / Files / Commits / Relations / History / Actions), 禁止大量 Modal |
| 输入 | Selected Node |
| 输出 | Inspector 内容 |
| 业务规则 | §二十一 + R9 |
| 优先级 | P0 |

### 4.3 功能需求 — FR-GRAPH (Graph 数据模型)

#### 4.3.1 FR-GRAPH-001 Graph Repository 抽象 Trait

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-001 |
| 描述 | 系统应定义 `GraphRepository Trait`, 抽象 6 个核心方法: `upsertNode / upsertEdge / deleteNode / deleteEdge / query / traverse` |
| 输入 | Node / Edge / Query |
| 输出 | Result |
| 业务规则 | §二十四 + §三十三 + BR-17 |
| 优先级 | P0 |

#### 4.3.2 FR-GRAPH-002 Node 类型 11 种

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-002 |
| 描述 | 系统应支持 11 种 Node 类型: Repository / Branch / Worktree / Commit / Task / AgentSession / PullRequest / File / Symbol / TestRun / Issue |
| 输入 | Node 定义 |
| 输出 | Graph Schema |
| 业务规则 | §七 |
| 优先级 | P0 |

#### 4.3.3 FR-GRAPH-003 Edge 类型 13 种

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-003 |
| 描述 | 系统应支持 13 种 Edge 类型: BASED_ON / USES_BRANCH / DERIVED_FROM / WORKS_ON / IMPLEMENTED_IN / MODIFIES / MODIFIES_SYMBOL / CONFLICTS_WITH / OVERLAPS_WITH / DEPENDS_ON / BLOCKS / SUPERSEDES / MERGED_INTO |
| 输入 | Edge 定义 |
| 输出 | Graph Schema |
| 业务规则 | §八 |
| 优先级 | P0 |

#### 4.3.4 FR-GRAPH-004 Worktree Node 字段

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-004 |
| 描述 | 系统应在 Worktree Node 上保存字段: id / name / branch / human_state / machine_state / ahead / behind / dirty / health_score / last_activity / agent_id / task_id / risk_count / test_state / created_at / locked / archived |
| 输入 | Worktree 数据 |
| 输出 | Node |
| 业务规则 | §七 + §十 |
| 优先级 | P0 |

#### 4.3.5 FR-GRAPH-005 AgentSession Node 字段

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-005 |
| 描述 | 系统应在 AgentSession Node 上保存字段: agent_id / agent_type / model / status / task_id / worktree_id / started_at / last_activity / token_usage / tool_calls / result_state |
| 输入 | Agent 数据 |
| 输出 | Node |
| 业务规则 | §十五 |
| 优先级 | P0 |

#### 4.3.6 FR-GRAPH-006 CONFLICTS_WITH Edge 字段

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-006 |
| 描述 | 系统应在 CONFLICTS_WITH Edge 上保存字段: risk_score / shared_files / shared_symbols / detected_at / reason / confidence |
| 输入 | Risk Engine 输出 |
| 输出 | Edge |
| 业务规则 | §十四 |
| 优先级 | P0 |

#### 4.3.7 FR-GRAPH-007 Graph Query Cypher 子集

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-007 |
| 描述 | 系统应支持 Cypher 查询子集: MATCH / WHERE / RETURN / WITH / ORDER BY / LIMIT, 适配 4 备选 Graph DB |
| 输入 | Query String |
| 输出 | Result |
| 业务规则 | §三十三 |
| 优先级 | P0 |

#### 4.3.8 FR-GRAPH-008 Graph Index (Worktree + Risk)

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-008 |
| 描述 | 系统应在 Worktree (id / branch / human_state) + Risk (risk_score / detected_at) 上建 Graph Index, 查询加速 |
| 输入 | Index Schema |
| 输出 | Index |
| 业务规则 | §二十七 |
| 优先级 | P0 |

#### 4.3.9 FR-GRAPH-009 Graph Clustering (远距离)

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-009 |
| 描述 | 系统应在远距离 (L0/L1) 时把 5+ Worktree 合并为 Cluster 节点, 显示 "Cluster (5 WT)" |
| 输入 | LOD level + Worktree |
| 输出 | Cluster Node |
| 业务规则 | §六 + §二十七 |
| 优先级 | P1 |

#### 4.3.10 FR-GRAPH-010 Graph Lazy Expansion

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-010 |
| 描述 | 系统应在用户点开 Cluster 才展开子节点, 不默认全展开 |
| 输入 | User Click |
| 输出 | 展开动画 |
| 业务规则 | §二十七 |
| 优先级 | P1 |

#### 4.3.11 FR-GRAPH-011 Graph Delta Update

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-011 |
| 描述 | 系统应通过 Event Bus 增量更新 Graph (节点增删改), 不每次全量重算 |
| 输入 | Event |
| 输出 | Graph Delta |
| 业务规则 | §二十六 |
| 优先级 | P0 |

#### 4.3.12 FR-GRAPH-012 Graph Neighborhood Query (1/2/3-hop)

| 项 | 内容 |
|---|---|
| ID | FR-GRAPH-012 |
| 描述 | 系统应支持 N-hop Neighborhood 查询, 通过 Cypher 路径查询实现 (e.g. `MATCH (n:Worktree {id:$id})-[:CONFLICTS_WITH|DEPENDS_ON*1..3]-(m) RETURN m`) |
| 输入 | Node ID + hop count |
| 输出 | Neighbor Set |
| 业务规则 | §十七 |
| 优先级 | P0 |

### 4.4 功能需求 — FR-RISK (Risk Engine)

#### 4.4.1 FR-RISK-001 Risk 类型 11 种

| 项 | 内容 |
|---|---|
| ID | FR-RISK-001 |
| 描述 | 系统应支持 11 种 Risk 类型: MergeConflict / FileOverlap / SymbolOverlap / Stale / Divergence / Dependency / TestFailure / BuildFailure / AgentIncomplete / ReviewMissing / Superseded |
| 输入 | Risk Definition |
| 输出 | Risk Type |
| 业务规则 | §十三 |
| 优先级 | P0 |

#### 4.4.2 FR-RISK-002 Risk Engine V1 (Git Status)

| 项 | 内容 |
|---|---|
| ID | FR-RISK-002 |
| 描述 | 系统应实现 Risk Engine V1: 基于 Git Status + Ahead/Behind + File overlap 的风险检测 (per §十三 V1) |
| 输入 | Git Status + Diff |
| 输出 | Risk[] |
| 业务规则 | §十三 |
| 优先级 | P0 |

#### 4.4.3 FR-RISK-003 Risk Engine V2 (Diff Overlap)

| 项 | 内容 |
|---|---|
| ID | FR-RISK-003 |
| 描述 | 系统应实现 Risk Engine V2: 基于 Diff overlap + Line overlap 的风险检测 |
| 输入 | Diff |
| 输出 | Risk[] |
| 业务规则 | §十三 |
| 优先级 | P1 |

#### 4.4.4 FR-RISK-004 Risk Engine V3 (Symbol Overlap + AST)

| 项 | 内容 |
|---|---|
| ID | FR-RISK-004 |
| 描述 | 系统应实现 Risk Engine V3: 基于 Symbol overlap + AST-level overlap 的风险检测, 需要 tree-sitter 解析 |
| 输入 | Source Code + AST |
| 输出 | Risk[] |
| 业务规则 | §十三 |
| 优先级 | P2 |

#### 4.4.5 FR-RISK-005 Conflict Prediction (前置检测)

| 项 | 内容 |
|---|---|
| ID | FR-RISK-005 |
| 描述 | 系统应在 `git merge` 之前发现潜在冲突 (WT-A mod file X, WT-B mod file X), 输出 CONFLICTS_WITH Edge |
| 输入 | 2 Worktree IDs |
| 输出 | Conflict Prediction |
| 业务规则 | §十四 |
| 优先级 | P0 |

#### 4.4.6 FR-RISK-006 Risk Score (0-1)

| 项 | 内容 |
|---|---|
| ID | FR-RISK-006 |
| 描述 | 系统应为每个 Risk 计算 Risk Score ∈ [0, 1], 考虑 shared_files 数 / shared_symbols 数 / confidence |
| 输入 | Risk data |
| 输出 | Risk Score |
| 业务规则 | §十四 + BR-8 |
| 优先级 | P0 |

#### 4.4.7 FR-RISK-007 Risk 解释 (R8 Explainable)

| 项 | 内容 |
|---|---|
| ID | FR-RISK-007 |
| 描述 | 系统应为每个 Risk 生成可解释 reason 字段 (e.g. "Both WT-A and WT-B modified src/auth.rs line 42-58"), 用于 AI Explanation |
| 输入 | Risk data |
| 输出 | Reason |
| 业务规则 | §十六 + R8 |
| 优先级 | P0 |

#### 4.4.8 FR-RISK-008 Risk 实时检测

| 项 | 内容 |
|---|---|
| ID | FR-RISK-008 |
| 描述 | 系统应在 Git Observer 检测到 Refs / Status 变化时, 触发 Risk 重新计算 (增量) |
| 输入 | Git Event |
| 输出 | Risk Delta |
| 业务规则 | §二十六 |
| 优先级 | P0 |

#### 4.4.9 FR-RISK-009 Risk 解决跟踪

| 项 | 内容 |
|---|---|
| ID | FR-RISK-009 |
| 描述 | 系统应跟踪 Risk Resolved 事件 (e.g. Conflict 解决), 更新 Risk 状态为 Resolved |
| 输入 | Event |
| 输出 | Risk Status Update |
| 业务规则 | §二十五 |
| 优先级 | P1 |

#### 4.4.10 FR-RISK-010 Risk 推荐 (下一步)

| 项 | 内容 |
|---|---|
| ID | FR-RISK-010 |
| 描述 | 系统应根据 Risk + Health Score 推荐"下一步优先处理" (per §三.20), 通过 Explanation Layer 输出 |
| 输入 | Worktree + Risk + Health |
| 输出 | Recommendation |
| 业务规则 | §三 + §十六 |
| 优先级 | P1 |

#### 4.4.11 FR-RISK-011 Risk Audit Log

| 项 | 内容 |
|---|---|
| ID | FR-RISK-011 |
| 描述 | 系统应记录所有 Risk 检测 + 解决事件到 Audit Log (per `BD-WORKTREE-CANVAS-001` §0), SCD Type 2 |
| 输入 | Risk Event |
| 输出 | Audit Record |
| 业务规则 | §二十九 |
| 优先级 | P1 |

### 4.5 功能需求 — FR-AGENT (Agent 集成)

#### 4.5.1 FR-AGENT-001 Agent Bridge 抽象 Trait

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-001 |
| 描述 | 系统应定义 `AgentRuntime Trait`, 抽象 4 方法: `getSession / listSessions / subscribeEvents / getSessionMetrics` |
| 输入 | Session ID |
| 输出 | Session Data |
| 业务规则 | §二十四 + §三十三 + BR-19 |
| 优先级 | P0 |

#### 4.5.2 FR-AGENT-002 Agent Node 显示

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-002 |
| 描述 | 系统应在 AGENT VIEW 把 AgentSession 渲染为中心节点, Worktree 作为子节点, Task 作为孙节点 |
| 输入 | AgentSession + Worktree + Task |
| 输出 | AGENT 视图渲染 |
| 业务规则 | §十五 + §十八 |
| 优先级 | P1 |

#### 4.5.3 FR-AGENT-003 Agent 状态实时同步

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-003 |
| 描述 | 系统应通过 AgentRuntime.subscribeEvents 实时同步 Agent Session 状态变化到画布 (状态色码实时变) |
| 输入 | Agent Event |
| 输出 | Canvas Update |
| 业务规则 | §二十六 |
| 优先级 | P0 |

#### 4.5.4 FR-AGENT-004 多 Agent 同 Task (1:N)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-004 |
| 描述 | 系统应支持 1 个 Task 由多个 Agent 处理, 各自生成不同 Worktree (e.g. Keep / Compare / Discard / Merge Selected Result) |
| 输入 | Task ID |
| 输出 | 多 Worktree 列表 |
| 业务规则 | §十五 |
| 优先级 | P2 |

#### 4.5.5 FR-AGENT-005 Agent Token Usage 显示

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-005 |
| 描述 | 系统应在 AgentSession 节点上显示 Token Usage (per §十五), 用于 SRE 监控 |
| 输入 | AgentSession.token_usage |
| 输出 | Token 数字 |
| 业务规则 | §十五 |
| 优先级 | P2 |

#### 4.5.6 FR-AGENT-006 Agent Tool Calls 计数

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-006 |
| 描述 | 系统应在 AgentSession 节点上显示 Tool Calls 计数 |
| 输入 | AgentSession.tool_calls |
| 输出 | 数字 |
| 业务规则 | §十五 |
| 优先级 | P2 |

#### 4.5.7 FR-AGENT-007 Agent 历史轨迹

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-007 |
| 描述 | 系统应在 Inspector Agent tab 显示 Agent 历史轨迹 (started_at → last_activity 时间线 + tool_calls 列表) |
| 输入 | AgentSession.events |
| 输出 | Timeline UI |
| 业务规则 | §十五 + §二十一 |
| 优先级 | P2 |

#### 4.5.8 FR-AGENT-008 Agent Result State 显示

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-008 |
| 描述 | 系统应在 AgentSession 节点上显示 Result State (success / partial / failed / pending) |
| 输入 | AgentSession.result_state |
| 输出 | 状态 UI |
| 业务规则 | §十五 |
| 优先级 | P1 |

### 4.6 功能需求 — FR-EXPLAIN (AI Explanation Layer)

#### 4.6.1 FR-EXPLAIN-001 AI Explanation API

| 项 | 内容 |
|---|---|
| ID | FR-EXPLAIN-001 |
| 描述 | 系统应提供 Explain Risk / Explain State / Explain Recommendation API, 调用 LLM 基于 Graph + Git + Risk 结构化数据生成自然语言解释 |
| 输入 | Worktree ID + Context Type |
| 输出 | Natural Language Explanation |
| 业务规则 | §十六 + BR-9 |
| 优先级 | P0 |

#### 4.6.2 FR-EXPLAIN-002 Explanation 来源限制

| 项 | 内容 |
|---|---|
| ID | FR-EXPLAIN-002 |
| 描述 | 系统应只允许 LLM 基于已有结构化数据解释, 禁止凭空生成 (per §十六), 通过 prompt engineering 限制 |
| 输入 | Structured Context |
| 输出 | Constrained Explanation |
| 业务规则 | §十六 + BR-9 |
| 优先级 | P0 |

#### 4.6.3 FR-EXPLAIN-003 Explanation Tooltip

| 项 | 内容 |
|---|---|
| ID | FR-EXPLAIN-003 |
| 描述 | 系统应在 Hover Worktree 时显示 Tooltip (1 句话解释状态), Click 显示完整解释 |
| 输入 | Worktree ID |
| 输出 | Tooltip + Full Text |
| 业务规则 | §十六 |
| 优先级 | P0 |

#### 4.6.4 FR-EXPLAIN-004 Explanation Cache

| 项 | 内容 |
|---|---|
| ID | FR-EXPLAIN-004 |
| 描述 | 系统应缓存 Explanation (Worktree ID + Risk Hash → Explanation), 避免重复 LLM 调用, TTL 5min |
| 输入 | Worktree + Risk |
| 输出 | Cached Explanation |
| 业务规则 | 性能 |
| 优先级 | P1 |

### 4.7 功能需求 — FR-SEARCH (搜索 + 过滤)

#### 4.7.1 FR-SEARCH-001 Search DSL Parser

| 项 | 内容 |
|---|---|
| ID | FR-SEARCH-001 |
| 描述 | 系统应实现 Search DSL Parser, 支持 `show:conflict / show:unmerged / show:ready / show:stale / agent:codex / behind:>20 / ahead:>10 / health:<50 / modified:auth.rs / task:payment / inactive:>24h`, 组合条件 |
| 输入 | Query String |
| 输出 | Filter Object |
| 业务规则 | §二十 |
| 优先级 | P0 |

#### 4.7.2 FR-SEARCH-002 Graph Query 翻译

| 项 | 内容 |
|---|---|
| ID | FR-SEARCH-002 |
| 描述 | 系统应将 DSL Filter 翻译为 Graph Query (Cypher) + Git Filter, 由 Query Engine 执行 |
| 输入 | Filter |
| 输出 | Result Set |
| 业务规则 | §二十 + §二十四 |
| 优先级 | P0 |

#### 4.7.3 FR-SEARCH-003 NL Query (LLM 翻译)

| 项 | 内容 |
|---|---|
| ID | FR-SEARCH-003 |
| 描述 | 系统应支持自然语言查询 "显示所有还未合并、落后 main 超过 10 commit、由 Codex 处理的 Worktree", LLM 翻译为 Graph Query + Git Filter |
| 输入 | NL Query |
| 输出 | Filter + Result |
| 业务规则 | §二十 |
| 优先级 | P1 |

#### 4.7.4 FR-SEARCH-004 实时搜索高亮

| 项 | 内容 |
|---|---|
| ID | FR-SEARCH-004 |
| 描述 | 系统应在搜索时, 命中节点高亮 (边框), 未命中降透明度到 30% |
| 输入 | Search Result |
| 输出 | Canvas 渲染 |
| 业务规则 | §二十 |
| 优先级 | P0 |

#### 4.7.5 FR-SEARCH-005 搜索历史

| 项 | 内容 |
|---|---|
| ID | FR-SEARCH-005 |
| 描述 | 系统应在搜索框保留最近 10 条历史 (localStorage), 支持 ↑↓ 浏览 |
| 输入 | User Search |
| 输出 | History UI |
| 业务规则 | UX |
| 优先级 | P2 |

#### 4.7.6 FR-SEARCH-006 保存搜索

| 项 | 内容 |
|---|---|
| ID | FR-SEARCH-006 |
| 描述 | 系统应支持保存搜索 (Save Search), 命名保存到 localStorage / Backend, 一键复用 |
| 输入 | Query + Name |
| 输出 | Saved Search |
| 业务规则 | UX |
| 优先级 | P2 |

### 4.8 功能需求 — FR-ACTION (操作 + 权限)

#### 4.8.1 FR-ACTION-001 Action Engine 抽象

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-001 |
| 描述 | 系统应定义 `ActionEngine Trait`, 抽象 3 方法: `execute / validate / audit`, 所有 18 Action 通过此接口 |
| 输入 | Action + Context |
| 输出 | Action Result |
| 业务规则 | §二十二 + §二十四 + BR-13 |
| 优先级 | P0 |

#### 4.8.2 FR-ACTION-002 Action 分类 (3 类)

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-002 |
| 描述 | 系统应把 18 Action 分类为 Safe (6 项: Create / Open / Open in IDE / Compare / Focus / Explain Risk) / Warning (6 项: Sync Main / Rebase / Create PR / Lock / Unlock / Archive) / Destructive (6 项: **Merge** / Mark Superseded / Set Dependency / Remove Dependency / Delete / Cleanup) — per FR-WT-035 v1.1 Merge 改 Destructive (self-review C-02, 2026-09-17 JST); Force 修饰 (Force Merge / Force Rebase / Force Delete) 不计入 18 项 (作为 Force 修饰的子操作, 走 Destructive 流程) |
| 输入 | Action |
| 输出 | Classification |
| 业务规则 | §二十二 + BR-13 |
| 优先级 | P0 |

#### 4.8.3 FR-ACTION-003 二次确认机制

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-003 |
| 描述 | 系统应实现二次确认机制, Destructive Action 弹出 Confirm Dialog (含 Action 名称 + 影响范围 + 撤销方案) |
| 输入 | Destructive Action |
| 输出 | Confirm UI |
| 业务规则 | §二十二 + BR-13 |
| 优先级 | P0 |

#### 4.8.4 FR-ACTION-004 权限检查 (RBAC)

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-004 |
| 描述 | 系统应在 BFF 层强制 RBAC 权限检查 (per `AGENTS.md` 守门 + multica-platform 权限模型), 5 域 Lead / PM / SRE / Dev 4 角色 |
| 输入 | User + Action |
| 输出 | Allow / Deny |
| 业务规则 | §二十二 + §二十九 |
| 优先级 | P0 |

#### 4.8.5 FR-ACTION-005 Audit Log (所有 Action)

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-005 |
| 描述 | 系统应记录所有 Action 到 Audit Log (who / when / what / result), SCD Type 2 不可篡改 |
| 输入 | Action Execution |
| 输出 | Audit Record |
| 业务规则 | §二十二 + §二十九 |
| 优先级 | P0 |

#### 4.8.6 FR-ACTION-006 Idempotency Key

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-006 |
| 描述 | 系统应为每个 Action 提供 Idempotency Key (UUID), 重复请求自动去重 |
| 输入 | Action + Idempotency Key |
| 输出 | Idempotent Result |
| 业务规则 | §二十二 |
| 优先级 | P0 |

#### 4.8.7 FR-ACTION-007 撤销机制 (部分 Action)

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-007 |
| 描述 | 系统应为 Lock / Archive / Set Dependency / Mark Superseded 提供撤销 (Undo), Delete / Merge 等不可撤销 |
| 输入 | Recent Action |
| 输出 | Undo Result |
| 业务规则 | §二十二 |
| 优先级 | P1 |

#### 4.8.8 FR-ACTION-008 批量操作

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-008 |
| 描述 | 系统应支持批量操作 (e.g. 选中 5 个 Worktree 全部 Sync Main), 二次确认 |
| 输入 | Multi-select + Action |
| 输出 | Batch Result |
| 业务规则 | §二十二 |
| 优先级 | P1 |

#### 4.8.9 FR-ACTION-009 进度反馈 (长时间操作)

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-009 |
| 描述 | 系统应在 Rebase / Merge / Sync 等长时间操作时, 显示进度条 + 当前步骤 (e.g. "正在 rebase 3/10") |
| 输入 | Long Action |
| 输出 | Progress UI |
| 业务规则 | UX |
| 优先级 | P0 |

#### 4.8.10 FR-ACTION-010 错误处理 + Retry

| 项 | 内容 |
|---|---|
| ID | FR-ACTION-010 |
| 描述 | 系统应实现统一错误处理 + Retry 策略 (指数退避, max 3 次), 网络错误自动重试, Git 冲突提示用户 |
| 输入 | Action Error |
| 输出 | Error UI + Retry |
| 业务规则 | §二十九 |
| 优先级 | P0 |

### 4.9 非功能需求 (Non-Functional Requirements)

#### 4.9.1 NFR-PERF-001 100 Worktree 流畅

| 项 | 内容 |
|---|---|
| ID | NFR-PERF-001 |
| 描述 | 系统应在 100 Worktree 时, 渲染 FPS >= 30, 交互延迟 < 100ms |
| 测量 | `frontend/bench/canvas-100.ts` |
| 优先级 | P0 |

#### 4.9.2 NFR-PERF-002 1000 Worktree 可交互

| 项 | 内容 |
|---|---|
| ID | NFR-PERF-002 |
| 描述 | 系统应在 1000 Worktree 时, 通过 LOD + Clustering 保持可交互 (FPS >= 15, 延迟 < 300ms) |
| 测量 | `frontend/bench/canvas-1000.ts` |
| 优先级 | P1 |

#### 4.9.3 NFR-PERF-003 Graph Query 响应时间

| 项 | 内容 |
|---|---|
| ID | NFR-PERF-003 |
| 描述 | 系统应保证 Graph Query (1-hop / N-hop) 响应时间 < 200ms (100 Worktree), < 1s (1000 Worktree) |
| 测量 | `bench/graph-query.ts` |
| 优先级 | P0 |

#### 4.9.4 NFR-PERF-004 Risk Engine 增量更新

| 项 | 内容 |
|---|---|
| ID | NFR-PERF-004 |
| 描述 | 系统应在 Git Event 触发时, Risk Engine 增量更新 < 500ms, 不全量重算 |
| 测量 | `bench/risk-delta.ts` |
| 优先级 | P0 |

#### 4.9.5 NFR-PERF-005 AI Explanation 缓存

| 项 | 内容 |
|---|---|
| ID | NFR-PERF-005 |
| 描述 | 系统应在 Explanation 缓存命中时, 响应时间 < 50ms |
| 测量 | `bench/explain-cache.ts` |
| 优先级 | P1 |

#### 4.9.6 NFR-SCALE-001 水平扩展 (Graph DB)

| 项 | 内容 |
|---|---|
| ID | NFR-SCALE-001 |
| 描述 | 系统应通过 Graph Repository Trait 抽象, 支持水平扩展 (Neo4j Causal Cluster / Memgraph HA) |
| 测量 | 设计评审 |
| 优先级 | P1 |

#### 4.9.7 NFR-SCALE-002 Canvas 渲染引擎可替换

| 项 | 内容 |
|---|---|
| ID | NFR-SCALE-002 |
| 描述 | 系统应通过 Canvas Renderer Trait 抽象, 支持 React-Flow / PixiJS / 自研 WebGL/Canvas 替换 |
| 测量 | 设计评审 |
| 优先级 | P1 |

#### 4.9.8 NFR-MAINT-001 模块解耦

| 项 | 内容 |
|---|---|
| ID | NFR-MAINT-001 |
| 描述 | 系统应实现原子化解耦, 14 模块 (Graph Core / Git Observer / Git Adapter / Worktree Service / Relationship Engine / Risk Engine / Health Engine / Agent Context Bridge / Canvas Renderer / Layout Engine / Query Engine / Action Engine / Event Bus / Persistence) 互不直接依赖, 通过接口通信 |
| 测量 | 设计评审 + 代码审查 |
| 优先级 | P0 |

#### 4.9.9 NFR-MAINT-002 API 优先

| 项 | 内容 |
|---|---|
| ID | NFR-MAINT-002 |
| 描述 | 系统应 API 优先 (OpenAPI 3.0 规范), 所有模块通过 API 通信, 便于未来替换 UI |
| 测量 | OpenAPI Schema 100% 覆盖 |
| 优先级 | P0 |

#### 4.9.10 NFR-REL-001 故障恢复

| 项 | 内容 |
|---|---|
| ID | NFR-REL-001 |
| 描述 | 系统应在 Git Observer 断连时, 自动重连 + 重放 missed events (last 1h) |
| 测量 | 故障注入测试 |
| 优先级 | P0 |

#### 4.9.11 NFR-REL-002 事件合并 + 防抖

| 项 | 内容 |
|---|---|
| ID | NFR-REL-002 |
| 描述 | 系统应在 Git Observer 检测到高频事件时 (e.g. 1s 内 100 次 Status 变化), 合并 + 防抖, 避免 Graph 全量重算 |
| 测量 | 压力测试 |
| 优先级 | P0 |

#### 4.9.12 NFR-CON-001 事务边界

| 项 | 内容 |
|---|---|
| ID | NFR-CON-001 |
| 描述 | 系统应明确定义事务边界 (e.g. Merge Action = Git Commit + Graph Update + Audit Log 在同一事务), 失败回滚 |
| 测量 | 设计评审 |
| 优先级 | P0 |

#### 4.9.13 NFR-CACHE-001 多级缓存

| 项 | 内容 |
|---|---|
| ID | NFR-CACHE-001 |
| 描述 | 系统应实现多级缓存 (L1 内存 LRU + L2 Redis + L3 Graph DB 自身), TTL 分层 |
| 测量 | 缓存命中率 > 80% |
| 优先级 | P0 |

#### 4.9.14 NFR-TEST-001 测试覆盖

| 项 | 内容 |
|---|---|
| ID | NFR-TEST-001 |
| 描述 | 系统应保证 Unit Test 覆盖率 >= 80% (Rust) / >= 70% (TS), Integration Test 覆盖所有 18 Action, E2E Test 覆盖 5 视图模式 |
| 测量 | `cargo tarpaulin` + `vitest run --coverage` |
| 优先级 | P0 |

#### 4.9.15 NFR-OBS-001 可观测性

| 项 | 内容 |
|---|---|
| ID | NFR-OBS-001 |
| 描述 | 系统应实现可观测性 (Prometheus metrics + OpenTelemetry traces + structured logs), 关键指标: Graph Query 延迟 / Risk Engine 延迟 / Action 成功率 / Cache 命中率 |
| 测量 | Grafana Dashboard |
| 优先级 | P0 |

#### 4.9.16 NFR-A11Y-001 可访问性

| 项 | 内容 |
|---|---|
| ID | NFR-A11Y-001 |
| 描述 | 系统应保证 WCAG 2.1 AA 合规 (键盘可操作 + ARIA 标签 + 颜色对比度), 盲人辅助测试通过 |
| 测量 | axe-core + NVDA 测试 |
| 优先级 | P1 |

#### 4.9.17 NFR-PERM-001 权限模型

| 项 | 内容 |
|---|---|
| ID | NFR-PERM-001 |
| 描述 | 系统应实现 5 角色 RBAC 权限模型 (Owner / PM / 5 域 Lead / SRE / Dev), 资源级 (Project / Repository / Worktree) |
| 测量 | 权限矩阵 100% 覆盖 |
| 优先级 | P0 |

#### 4.9.18 NFR-SEC-001 数据加密

| 项 | 内容 |
|---|---|
| ID | NFR-SEC-001 |
| 描述 | 系统应保证敏感数据 (Git Token / Agent Token) 加密存储 (AES-256), 传输 TLS 1.3 |
| 测量 | 安全审计 |
| 优先级 | P0 |

#### 4.9.19 NFR-SEC-002 Git 隔离

| 项 | 内容 |
|---|---|
| ID | NFR-SEC-002 |
| 描述 | 系统应在执行 Git 命令时, 使用最小权限 (e.g. git worktree add 限定 path), 防止 path traversal |
| 测量 | fuzzing 测试 |
| 优先级 | P0 |

#### 4.9.20 NFR-AUDIT-001 审计日志

| 项 | 内容 |
|---|---|
| ID | NFR-AUDIT-001 |
| 描述 | 系统应保证所有 Action + Risk Event + Graph 写入都有审计日志, 保留 1 年, 不可篡改 (SCD Type 2) |
| 测量 | 审计日志完整性验证 |
| 优先级 | P0 |

#### 4.9.21 NFR-I18N-001 国际化

| 项 | 内容 |
|---|---|
| ID | NFR-I18N-001 |
| 描述 | 系统应支持 3 语言 (zh-CN / en / ja), 通过 i18n 字典切换, 不允许硬编码字符串 |
| 测量 | `i18next` 测试 |
| 优先级 | P0 |

#### 4.9.22 NFR-THEME-001 Dark/Light Theme

| 项 | 内容 |
|---|---|
| ID | NFR-THEME-001 |
| 描述 | 系统应支持 Dark / Light Theme 切换, 通过 CSS variable 实现, 跟随系统 |
| 测量 | 主题切换测试 |
| 优先级 | P0 |

#### 4.9.23 NFR-KEYB-001 键盘可操作

| 项 | 内容 |
|---|---|
| ID | NFR-KEYB-001 |
| 描述 | 系统应保证全功能键盘可操作 (无鼠标依赖), 快捷键不与浏览器/系统冲突 |
| 测量 | 键盘 only 测试 |
| 优先级 | P0 |

---

## §5 业务约束

### 5.1 技术栈约束

- 前端: Next.js 14.2.5 + TypeScript 5.x + React 18 (per `SRS-CANVAS-001.md` v1.1 复用栈)
- Canvas Renderer: React-Flow 11.x (MVP 推荐方案, 备选 PixiJS / 自研 WebGL)
- 状态管理: zustand 4.x (per `frontend/src/lib/store`)
- UI 组件: 自研 Component Library (per V0.1) + lucide-react + recharts
- 后端: Rust 1.80+ (per `mavis` desktop 栈, `cargo check`)
- Graph DB: Neo4j 5.x (MVP 推荐方案, 备选 Memgraph / Embedded / Custom) — 决策点 BD 拍板
- Git Adapter: libgit2 (MVP 推荐方案, 备选 git CLI / Gitoxide) — 决策点 BD 拍板
- 数据库: PostgreSQL 15+ (per 现有栈, Audit Log / Risk / Health 时序)
- Cache: Redis 7.x (L2 cache)
- LLM: Claude Sonnet / GPT-4o (Explanation Layer)

### 5.2 架构约束

- 14 模块必须原子化解耦 (per §二十四 + NFR-MAINT-001)
- Graph Repository / Canvas Renderer / Git Provider / Agent Runtime 4 大抽象 Trait (per §三十三 + BR-17-19)
- 事件总线解耦 (per §二十五), 4 类消费者 (Graph / UI / Risk Engine / Notification)
- API 优先 (OpenAPI 3.0, per NFR-MAINT-002)

### 5.3 数据约束

- Source of Truth (Git) 不可被 AI 或画布修改 (per BR-11)
- Derived Data (Graph) 可被 Risk Engine 写入
- AI Explanation 仅基于结构化数据 (per BR-9)
- Audit Log 不可篡改 (SCD Type 2, per NFR-AUDIT-001)

### 5.4 UI 约束

- 不能只靠颜色表达状态 (per §三十四, 色盲 + i18n)
- 必须用图形 + 文字双重编码
- 不允许频繁 Modal (用 Inspector, per §二十一)
- 不允许 Commit Graph 默认霸占主画面 (per §三十四)
- 避免无意义动画 (per §三十四)
- 异常优先 (R7): 正常节点弱化, 异常高亮
- 渐进式披露 (R5): 默认只展示当前决策所需信息
- Semantic Zoom (R6): 缩放改变信息粒度

---

## §6 使用场景

### 6.1 主流程: 用户打开 Worktree Canvas

```
1. User 打开 /worktree-canvas
2. 系统读 URL (?repo=xxx) → 加载 Repository + Worktree 集合
3. 系统调 GraphRepository.query() 拉取 Worktree + Edge
4. 系统触发 Risk Engine V1 全量扫 (后增量)
5. 系统计算 Health Score 全量
6. 系统调 Layout Engine 自动布局 (TREE VIEW 默认)
7. 系统调 Canvas Renderer 渲染 (LOD 默认 L1, 显示 Repository + Worktree)
8. 系统订阅 Git Observer + AgentRuntime Event
9. 系统渲染完成, 用户看到 Worktree 全景图 (US-1 满足)
```

### 6.2 异常流程: 用户发现 Conflict

```
1. 用户看到 CONFLICTS_WITH 边高亮 (R7 Exception-first)
2. 用户 Hover 边 → 显示 shared_files + shared_symbols + confidence
3. 用户 Click 边 → 双击其中 1 个 Worktree → Inspector 打开
4. Inspector Git State tab 显示 Conflict 详情
5. Inspector Risk tab 显示 RiskScore + reason
6. 用户 Click "Explain Risk" → AI Explanation API 调用 → "WT-A 和 WT-payment 同时修改了 PaymentService.createOrder 符号, 建议先 Sync Main 再 Merge"
7. 用户决定: Sync Main (FR-WT-033) → Warning 二次确认 → 执行
```

### 6.3 异常流程: 用户批量清理 Stale Worktree

```
1. 用户输入 Search DSL "show:stale inactive:>7d"
2. 系统过滤, 命中 N 个 Worktree, 高亮 + 其他降透明度
3. 用户多选 (Ctrl+Click) 或框选
4. 用户右键 → Cleanup (FR-WT-016)
5. 系统弹 Confirm Dialog (Destructive): "将删除 N 个 Worktree, 影响: ..."
6. 用户确认 → 执行批量删除 → Audit Log 记录
7. 画布实时更新, 节点消失 + Recently Merged 增加 (per §十九)
```

---

## §7 数据需求

### 7.1 Node Schema (11 种, MVP 重点 5 种)

#### 7.1.1 Worktree Node (P0)

```rust
struct WorktreeNode {
    id: WorktreeId,                    // UUID
    repo_id: RepoId,
    branch: String,                    // git branch
    name: String,                      // human-readable
    human_state: HumanState,           // 7 态枚举
    machine_state: MachineState,       // 内部细粒度
    ahead: u32,                        // commits ahead of main
    behind: u32,                       // commits behind main
    dirty: bool,                       // uncommitted changes
    health_score: HealthScore,         // 0-100
    last_activity: DateTime,           // ISO 8601
    agent_id: Option<AgentId>,         // FK AgentSession
    task_id: Option<TaskId>,           // FK Task
    risk_count: u32,                   // 关联 Risk 数
    test_state: TestState,             // passed/failed/running/none
    locked: bool,
    archived: bool,
    created_at: DateTime,
    merged_at: Option<DateTime>,
    parent_id: Option<WorktreeId>,     // DERIVED_FROM 来源
}
```

#### 7.1.2 Repository Node (P0)

```rust
struct RepositoryNode {
    id: RepoId,
    name: String,
    url: String,                       // git remote
    default_branch: String,
    active_worktree_count: u32,
    risk_count: u32,
    ready_count: u32,
    merged_count: u32,
    last_commit_at: DateTime,
}
```

#### 7.1.3 Mainline Node (P0, Repository 子节点)

```rust
struct MainlineNode {
    id: MainlineId,
    repo_id: RepoId,
    branch: String,                    // main / master / develop
    head_sha: String,
    last_update: DateTime,
}
```

#### 7.1.4 Task Node (P0)

```rust
struct TaskNode {
    id: TaskId,
    title: String,
    status: TaskStatus,                // todo/in_progress/review/blocked/done/wontfix
    assignee_id: Option<UserId>,
    worktree_ids: Vec<WorktreeId>,     // IMPLEMENTED_IN
    due_date: Option<Date>,
}
```

#### 7.1.5 AgentSession Node (P0)

```rust
struct AgentSessionNode {
    id: AgentId,
    agent_type: String,                // claude-code / codex / opencode / custom
    model: String,                     // claude-sonnet-4 / gpt-4o / etc.
    status: AgentStatus,               // 14 状态
    task_id: Option<TaskId>,
    worktree_id: Option<WorktreeId>,   // WORKS_ON
    started_at: DateTime,
    last_activity: DateTime,
    token_usage: TokenUsage,
    tool_calls: u32,
    result_state: AgentResultState,    // success/partial/failed/pending
}
```

(其他 6 种 Node: Commit / Branch / PullRequest / File / Symbol / TestRun / Issue, 默认折叠, schema 在 DD 详)

### 7.2 Edge Schema (13 种)

#### 7.2.1 DERIVED_FROM (P0)

```rust
struct DerivedFromEdge {
    from: WorktreeId,
    to: WorktreeId,                    // parent
    created_at: DateTime,
}
```

#### 7.2.2 CONFLICTS_WITH (P0)

```rust
struct ConflictsWithEdge {
    from: WorktreeId,
    to: WorktreeId,
    risk_score: f32,                   // 0-1
    shared_files: Vec<FilePath>,
    shared_symbols: Vec<SymbolRef>,
    detected_at: DateTime,
    reason: String,
    confidence: f32,                   // 0-1
}
```

#### 7.2.3 DEPENDS_ON (P1)

```rust
struct DependsOnEdge {
    from: WorktreeId,
    to: WorktreeId,
    required_state: HumanState,        // 默认 MERGED
    created_at: DateTime,
}
```

(其他 11 种 Edge: BASED_ON / USES_BRANCH / WORKS_ON / IMPLEMENTED_IN / MODIFIES / MODIFIES_SYMBOL / OVERLAPS_WITH / BLOCKS / SUPERSEDES / MERGED_INTO / VALIDATES / REVIEWS, schema 在 DD 详)

### 7.3 数据来源分类

| 来源 | 数据 | 写入方 | 消费者 |
|---|---|---|---|
| Git Source of Truth | Branch / Commit / Ref / Worktree / Diff / Merge-base / Ahead / Behind / Status | Git CLI / libgit2 (只读) | Graph Core, Risk Engine, Canvas |
| Graph Derived Data | Dependency / Conflict / Overlap / Superseded / Risk / Health | Graph Core (派生), Risk Engine (写入) | Canvas, UI, AI Explanation |
| AI Derived Explanation | 自然语言解释 | LLM (基于结构化数据) | Canvas, Tooltip |
| Runtime Agent State | Agent Session / Tool Call / Token Usage / Result State | AgentRuntime | Canvas (AGENT VIEW), UI |

---

## §8 接口需求

### 8.1 BFF API (OpenAPI 3.0)

| 路径 | 方法 | 用途 |
|---|---|---|
| `/api/v1/repos` | GET | 列 Repository |
| `/api/v1/repos/{id}` | GET | Repository 详情 |
| `/api/v1/repos/{id}/worktrees` | GET | 列 Worktree |
| `/api/v1/worktrees/{id}` | GET | Worktree 详情 |
| `/api/v1/worktrees/{id}/graph` | GET | 1/2/3-hop Neighborhood |
| `/api/v1/worktrees/{id}/health` | GET | Health Score + 扣分明细 |
| `/api/v1/worktrees/{id}/risks` | GET | 关联 Risk 列表 |
| `/api/v1/worktrees/{id}/actions/{action}` | POST | 执行 Action (18 个) |
| `/api/v1/search` | POST | Search DSL / NL Query |
| `/api/v1/explain/{worktree_id}` | GET | AI Explanation |
| `/api/v1/audit` | GET | Audit Log 查询 |

### 8.2 WebSocket / SSE 事件流

```typescript
// SSE Endpoint: /api/v1/events/stream
type CanvasEvent =
  | { type: 'worktree.created', payload: WorktreeNode }
  | { type: 'worktree.changed', payload: WorktreeNode }
  | { type: 'worktree.deleted', payload: { id: WorktreeId } }
  | { type: 'worktree.merged', payload: { id: WorktreeId, merged_at: DateTime } }
  | { type: 'branch.updated', payload: { repo_id: RepoId, branch: String, head_sha: String } }
  | { type: 'main.updated', payload: { repo_id: RepoId, new_head: String } }
  | { type: 'agent.started', payload: AgentSessionNode }
  | { type: 'agent.stopped', payload: { id: AgentId, result: AgentResultState } }
  | { type: 'task.changed', payload: TaskNode }
  | { type: 'test.completed', payload: { worktree_id: WorktreeId, test_state: TestState } }
  | { type: 'risk.detected', payload: RiskEvent }
  | { type: 'risk.resolved', payload: { risk_id: RiskId } }
  | { type: 'health.changed', payload: { worktree_id: WorktreeId, health: HealthScore } }
  | { type: 'relation.created', payload: Edge }
  | { type: 'relation.removed', payload: { edge_id: EdgeId } };
```

---

## §9 Worktree 状态定义

### 9.1 7 种 Human State 定义

| Human State | 进入条件 | 离开条件 | 优先级 | UI 表现 | 可执行动作 | 风险级别 | 需用户处理 |
|---|---|---|---|---|---|---|---|
| RUNNING | Agent active + git clean + no risk | Agent stopped / git dirty / risk detected | 1 | 蓝色边框 + ▶ 图标 | Sync / Compare / Open / Focus / Explain | Low | No |
| WAITING | Agent awaiting (awaiting_human/feedback/tool) | Agent resumes | 2 | 黄色边框 + ⏸ 图标 | Focus / Explain / Open | Low | No |
| READY | Health >= 80 + no conflict + ahead == 0 + no pending dep | Action triggered / risk detected | 3 | 绿色边框 + ✓ 图标 | Merge / Create PR / Sync / Compare / Open | Low | No (但推荐 Merge) |
| DIVERGED | Ahead > 0 + Behind > 0 (divergence threshold) | Sync / Rebase / Merge | 4 | 橙色边框 + ⇄ 图标 | Sync / Rebase / Merge / Compare | Medium | Yes (建议 Sync) |
| CONFLICT | CONFLICTS_WITH edge exists OR merge-base conflict | Conflict resolved / Manual override | 5 | 红色边框 + ⚠ 图标 | Sync / Rebase / Compare / Explain | High | Yes |
| MERGED | `git merge` 成功 | (终态) | 6 | 灰度 + ✓ Merged 图标 | View History / Archive | None | No |
| STALE | Last Activity > 7d AND not MERGED | Activity resumed / Cleanup | 7 | 灰度边框 + 🕐 图标 | Sync / Cleanup / Delete / Archive | Medium | No (但建议清理) |

### 9.2 Machine State (内部细粒度, UI 不直接展示)

```rust
enum MachineState {
    // RUNNING 子态
    AgentExecuting,
    GitClean,
    GitDirty,
    MergeInProgress,
    RebaseInProgress,
    CiRunning,
    
    // WAITING 子态
    AwaitingHuman,
    AwaitingFeedback,
    AwaitingTool,
    
    // ... 其他
    
    // 终态
    MergedClean,
    Archived,
    Deleted,
}
```

---

## §10 Worktree 卡片字段定义 (12 项, MVP)

| 字段 | 来源 | UI 表现 |
|---|---|---|
| Worktree Name | Worktree.name | 卡片标题 (12px 字号, bold) |
| Branch Name | Worktree.branch | 副标题 (10px 字号, ink-dim) |
| Human State | Worktree.human_state | 色码 + 图标 + 文本 (三重编码) |
| Agent | Worktree.agent_id → AgentSessionNode.agent_type | 头像 + agent_type 缩写 (e.g. "CC" for Claude Code) |
| Task | Worktree.task_id → TaskNode.title | Task 标题截断 (前 20 字符) |
| Ahead | `git rev-list --count main..WT` | `+{n}` 数字 + ↑ 图标 |
| Behind | `git rev-list --count WT..main` | `-{n}` 数字 + ↓ 图标 |
| Dirty State | `git status --porcelain` | • 角标 (有未提交) |
| Health Score | HealthEngine.compute() | 0-100 数字 + 色码环 (>=80 绿 / 50-79 黄 / <50 红) |
| Last Activity | max(git log, agent activity, test activity) | 相对时间 (e.g. "5 分钟前", "2 天前") |
| Test State | TestRun aggregation | 图标 + 文本 (✓ passed / ✗ failed / ◌ running / — none) |
| Risk Count | Risk Engine output | 数字角标 (>0 高亮红) |

**默认禁止展示**: 长 commit hash / 复杂 refs / 内部 Git 对象 / 完整 commit DAG (per §十 + §三十四)

---

## §11 AI Explanation Layer

### 11.1 Explanation 类型

| 类型 | 触发 | 输出示例 |
|---|---|---|
| Explain State | Hover Worktree | "WT-A 处于 CONFLICT 状态, 因为它与 WT-payment 同时修改了 PaymentService.createOrder 符号" |
| Explain Risk | Hover Risk Edge / Click Explain Risk | "WT-A 与 WT-payment 冲突, shared_symbols = [PaymentService.createOrder], risk_score = 0.85" |
| Explain Health | Click Health Score | "WT-A Health = 65, 扣分项: Behind -10 (落后 19 commit) / Conflict -25 (与 WT-payment) / 建议 Sync Main" |
| Recommend Next | Click "下一步" | "建议优先处理 WT-A: Health 65, CONFLICT 状态, Sync Main 后即可 Merge" |

### 11.2 Explanation 数据流

```
Graph Delta + Git Diff + Risk Engine Output
  ↓ Aggregate (Structured Context)
  ↓ LLM Call (Claude Sonnet / GPT-4o, prompt = "只基于以下结构化数据解释, 不添加外部知识")
  ↓ Explanation Text
  ↓ Cache (Redis, TTL 5min, key = WorktreeId + RiskHash)
  ↓ Return to UI
```

### 11.3 Prompt Engineering 约束

- 必须明确告知 LLM: "你只能基于以下 JSON 数据解释, 不允许添加 JSON 之外的推断"
- 必须提供完整结构化 Context (Worktree + Risk + Health + Git Status)
- 必须提供 Explanation 长度限制 (1-3 句话)
- 必须提供 Output Format (Markdown or Plain Text)

---

## §12 验收标准 (受入基準 / Acceptance Criteria)

### 12.1 功能验收 (Functional AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-F-1 | Worktree Node 渲染正确, 7 状态色码 + 图标 + 文本三重编码 | 手动 / vitest |
| AC-F-2 | Worktree 卡片显示 12 项字段, 默认不显示 commit hash | 手动 |
| AC-F-3 | Ahead / Behind 同时用数字 + 空间距离表达, visualDistance = log(n+1) | 手动 / 单元测试 |
| AC-F-4 | Health Score 0-100, 13 因素加权, 可点开每项扣分理由 | 手动 |
| AC-F-5 | Risk Engine V1 检测 11 种 Risk, 输出 risk_score + reason | 手动 / 单元测试 |
| AC-F-6 | CONFLICTS_WITH 边 Hover 显示 shared_files / shared_symbols / confidence | 手动 |
| AC-F-7 | 5 视图模式 (TREE/DEPENDENCY/RISK/AGENT/HISTORY) 切换不重算数据 | 手动 / e2e |
| AC-F-8 | 6 级 Semantic Zoom, L0-L5 信息粒度自动切换 | 手动 |
| AC-F-9 | Search DSL 支持组合条件 (show:conflict behind:>10 agent:codex) | 手动 / vitest |
| AC-F-10 | NL Query 翻译为 Graph Query + Git Filter | 手动 / e2e |
| AC-F-11 | Focus Mode 1/2/3-hop, 当前节点 100%, 邻居 80%, 其他 20% | 手动 |
| AC-F-12 | 18 Action 全部可执行, Safe/Warning/Destructive 分类正确 | 手动 / e2e |
| AC-F-13 | Destructive Action 二次确认 + Audit Log | 手动 / 单元测试 |
| AC-F-14 | AI Explanation 基于结构化数据, 不凭空生成 | 手动 / 单元测试 (LLM 输出约束) |
| AC-F-15 | Inspector 11 tab 全部可访问, 不弹 Modal | 手动 |
| AC-F-16 | Merged Worktree 折叠到 Recently Merged, 24h/7d/All 过滤 | 手动 |
| AC-F-17 | Audit Log 记录所有 Action, SCD Type 2 不可篡改 | 单元测试 |
| AC-F-18 | Risk Resolved 事件跟踪 | 单元测试 |
| AC-F-19 | 5 角色 RBAC 权限 (Owner/PM/Lead/SRE/Dev) | 单元测试 |
| AC-F-20 | 多用户实时协同 (per `SRS-CANVAS-AGENT-001` v1.2 A12 引用) | e2e |

### 12.2 性能验收 (Performance AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-P-1 | 100 Worktree 渲染 FPS >= 30, 交互延迟 < 100ms | `bench/canvas-100.ts` |
| AC-P-2 | 1000 Worktree 通过 LOD + Clustering 保持可交互 (FPS >= 15, 延迟 < 300ms) | `bench/canvas-1000.ts` |
| AC-P-3 | Graph Query (N-hop) 响应 < 200ms (100 WT), < 1s (1000 WT) | `bench/graph-query.ts` |
| AC-P-4 | Risk Engine 增量更新 < 500ms | `bench/risk-delta.ts` |
| AC-P-5 | Explanation 缓存命中响应 < 50ms | `bench/explain-cache.ts` |
| AC-P-6 | Cache 命中率 >= 80% | Prometheus metric |
| AC-P-7 | 事件合并 + 防抖, 1s 内 100 次 Status 变化不触发全量重算 | 压力测试 |

### 12.3 质量验收 (Quality AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-Q-1 | 14 模块原子化解耦, 通过接口通信 | 设计评审 + 代码审查 |
| AC-Q-2 | Graph Repository / Canvas Renderer / Git Provider / Agent Runtime 4 抽象 Trait | 设计评审 |
| AC-Q-3 | Unit Test 覆盖率 >= 80% (Rust) / >= 70% (TS) | `cargo tarpaulin` + `vitest --coverage` |
| AC-Q-4 | Integration Test 覆盖所有 18 Action | `pytest integration_tests/` |
| AC-Q-5 | E2E Test 覆盖 5 视图模式 + 6 zoom + 18 Action | Playwright |
| AC-Q-6 | OpenAPI 3.0 规范 100% 覆盖 | `swagger validate` |
| AC-Q-7 | i18n 3 语言 (zh-CN/en/ja) 无硬编码字符串 | `i18next` 测试 |
| AC-Q-8 | WCAG 2.1 AA 合规, 键盘可操作 | axe-core + NVDA |
| AC-Q-9 | Prometheus metrics + OpenTelemetry traces + structured logs | Grafana Dashboard |
| AC-Q-10 | 审计日志 SCD Type 2 不可篡改 | 审计日志完整性验证 |

### 12.4 文档验收 (Documentation AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-D-1 | 本 SRS (要件定義書) 落档 | `docs/requirements/SRS-WORKTREE-CANVAS-001.md` 存在 |
| AC-D-2 | BD (基本設計書) 落档 | `docs/design/BD-WORKTREE-CANVAS-001.md` 存在 |
| AC-D-3 | DD (詳細設計書) 落档 | `docs/design/DD-WORKTREE-CANVAS-001.md` 存在 |
| AC-D-4 | 追踪矩阵 (Requirement ↔ BD ↔ DD ↔ Test) 落档 | `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` 存在 |
| AC-D-5 | Self-Review (per §三十六) 落地 Critical / Major / Minor 三级问题清单 | 内嵌在 SRS/BD/DD 末尾 |
| AC-D-6 | commit author = Ulysses (per AGENTS.md §1 代签) | `git log --format='%an <%ae>' HEAD` |

---

## §13 已知风险 / 未解決問題

| # | 风险 / 缺口 | 影响 | 缓解 / 后续 |
|---|---|---|---|
| 1 | Graph DB 选型未拍板 (Neo4j vs Memgraph vs Embedded vs Custom) | 4 备选, MVP 推荐 Neo4j 5.x, 备选 Memgraph | BD §0 决策点 D-GRAPH-001 拍板; 抽象 `GraphRepository Trait` 不绑死 |
| 2 | Canvas Renderer 选型未拍板 (React-Flow vs PixiJS vs 自研) | 100+ 节点 React-Flow 可能卡, 1000+ 必须 WebGL | BD §0 D-CANVAS-001 拍板; MVP 选 React-Flow + LOD + Clustering 兜底 |
| 3 | Git Adapter 选型未拍板 (git CLI vs libgit2 vs Gitoxide) | 性能 + 跨平台差异 | BD §0 D-GIT-001 拍板; MVP 选 libgit2 (Rust 原生 + 跨平台) |
| 4 | Risk Engine V3 (AST/Symbol) 依赖 tree-sitter, 多语言解析器维护成本 | 仅支持主流语言 (Rust/TS/Python/Go/Java) | V3 标记 P2, MVP 只到 V2 (Diff Overlap) |
| 5 | Agent Runtime 适配层尚未落地, MVP 依赖 mock seed | AGENT VIEW 数据是 mock | P3-E 阶段接入真实 AgentRuntime (per `SRS-STAR-AGENT-RUNTIME-001`) |
| 6 | LLM 调用成本不可忽略 (Sonnet $3/M tokens) | Explanation Layer 1000+ Worktree 时成本爆炸 | Cache (Redis, TTL 5min) + 批量调用 + 降级到模板生成 |
| 7 | 1000 Worktree 性能边界未实测 | 可能需要 WebGL/Canvas 自研渲染器 | P2 实装期做性能压测; 不达预期切换 PixiJS |
| 8 | 多用户协同 (A12 多人编辑) 与本 SRS 的边界 | 本 SRS 仅消费 A12 能力, 不重写 | per `SRS-CANVAS-AGENT-001` v1.2 引用 |
| 9 | Risk Score 算法未标准化 (各 Risk 类型权重) | 0-1 评分主观 | MVP 采用推荐权重 + 阈值, 后续根据数据校准 |
| 10 | Worktree 状态机边界 (e.g. Agent crashed 算什么?) | 边界 case 未穷尽 | MVP 状态机以 §9.1 为准, 后续 per 实际反馈迭代 |
| 11 | Symbol Overlap 依赖源码解析, 多语言支持 | V3 P2, 多语言覆盖不全 | MVP 不实现 V3, 后续按语言优先级迭代 |
| 12 | Audit Log 长期保留 (1 年) 数据量爆炸 | PostgreSQL 性能 | 分区表 (按月) + 归档到 S3 冷存储 |
| 13 | 键盘快捷键与浏览器/系统冲突 | 部分用户快捷键失效 | 默认快捷键 + 用户自定义设置 (P2) |

**DDD Review 必查**: 缺口 #1/#2/#3 选型 + #6 LLM 成本 + #7 1000 WT 性能边界

---

## §14 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 🟢 Mavis 接手 (per DEC-008) | 2026-09-15 | 8/27 19:39 JST 用户授权代签 |
| SRE Lead | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 平台 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 评审主持 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| PM | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |

**真人到位后追溯签字覆盖** = 修订历史表 +1 行 (per AGENTS.md §3 7 段结构 + 9/3 19:35 JST 拍板 D 维持)

---

## §15 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v1.0 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 15 段 (文档信息/目的/用语/前提/业务需求/约束/场景/数据/接口/状态/卡片/AI/验收/风险/签字), **126 唯一 ID** (FR 103 + NFR 23 子段, 去重后 21 唯一 NFR ID), 39 用户故事 (≥ 39 满足), 13 已知缺口, 5 视图模式 + 6 级 Semantic Zoom + 7 状态机 + 11 风险类型 + 5 层性能策略 + 14 模块 + 18 Action | 2026-09-15 Multica ULYS-57 issue 创建者发令 |
| v1.1 | Ulysses — Mavis 接手 (per 守门 #14 v3, self-review C-01..C-04 修正) | 修正: §0 总数对齐 126; §1.1 总数对齐 126; FR-ACTION-002 Merge 改 Destructive (per C-02); Trace §11 AC 43 项全表化 (per C-03); BD §0.3/§37 总数同步 (per C-04) | 2026-09-17 ULYS-62 self-review 修正落地 |

---

## 附录 A: 需求追踪矩阵 (Requirement ↔ 上位设计 ↔ 验收)

| Requirement ID | Requirement Summary | 上位设计章节 | 验收 AC | 优先级 |
|---|---|---|---|---|
| FR-WT-001 | Worktree 节点展示 | BD §0 / DD §0 | AC-F-1 | P0 |
| FR-WT-002 | Worktree 状态压缩 (7 态) | BD §20 / DD §23 | AC-F-1 | P0 |
| FR-WT-003 | Worktree 卡片 12 字段 | BD §18 / DD §7 | AC-F-2 | P0 |
| FR-WT-004 | Worktree 状态机 | BD §20 / DD §23 | AC-F-1 | P0 |
| FR-WT-005 | Ahead/Behind 3 维可视化 | BD §21 / DD §36 | AC-F-3 | P0 |
| FR-WT-006 | Health Score 13 因素 | BD §21 / DD §34 | AC-F-4 | P0 |
| FR-WT-007 | Worktree 颜色编码 | BD §24 / DD §29 | AC-F-1 | P0 |
| FR-WT-008 | Worktree 图标 + 文本 | BD §24 / DD §29 | AC-F-1 | P0 |
| FR-WT-009 | Dirty 状态指示 | BD §18 / DD §7 | AC-F-2 | P0 |
| FR-WT-010 | Last Activity | BD §18 / DD §7 | AC-F-2 | P0 |
| FR-WT-011 | Test State | BD §18 / DD §7 | AC-F-2 | P0 |
| FR-WT-012 | Risk Count | BD §18 / DD §7 | AC-F-2 | P0 |
| FR-WT-013 | Lock/Unlock | BD §22 / DD §18 | AC-F-12 | P0 |
| FR-WT-014 | Archive | BD §22 / DD §18 | AC-F-12 | P1 |
| FR-WT-015 | Mark Superseded | BD §22 / DD §18 | AC-F-12 | P2 |
| FR-WT-016 | Cleanup 批量 | BD §22 / DD §18 | AC-F-12 | P1 |
| FR-WT-017 | Provenance 派生链 | BD §21 / DD §7 | AC-F-15 | P2 |
| FR-WT-018 | Merge Readiness | BD §21 / DD §34 | AC-F-12 | P0 |
| FR-WT-019 | 视图模式切换 5 模式 | BD §25 / DD §29 | AC-F-7 | P0 |
| FR-WT-020 | TREE VIEW | BD §25 / DD §29 | AC-F-7 | P0 |
| FR-WT-021 | DEPENDENCY VIEW | BD §25 / DD §29 | AC-F-7 | P1 |
| FR-WT-022 | RISK VIEW | BD §25 / DD §29 | AC-F-7 | P1 |
| FR-WT-023 | AGENT VIEW | BD §25 / DD §29 | AC-F-7 | P1 |
| FR-WT-024 | HISTORY VIEW | BD §25 / DD §29 | AC-F-7 | P2 |
| FR-WT-025 | Merged 处理 | BD §25 / DD §29 | AC-F-16 | P1 |
| FR-WT-026 | 双击跳详情 | BD §26 / DD §29 | AC-F-15 | P0 |
| FR-WT-027 | Health 趋势 | BD §21 / DD §34 | AC-F-15 | P2 |
| FR-WT-028 | Health 扣分明细 | BD §21 / DD §34 | AC-F-4 | P0 |
| FR-WT-029 | Compare 2 Worktree | BD §22 / DD §18 | AC-F-12 | P0 |
| FR-WT-030 | Open in IDE | BD §22 / DD §18 | AC-F-12 | P1 |
| FR-WT-031 | Set Dependency | BD §22 / DD §18 | AC-F-12 | P1 |
| FR-WT-032 | Remove Dependency | BD §22 / DD §18 | AC-F-12 | P1 |
| FR-WT-033 | Sync Main | BD §22 / DD §18 | AC-F-12 | P0 |
| FR-WT-034 | Rebase | BD §22 / DD §18 | AC-F-12 | P1 |
| FR-WT-035 | Merge | BD §22 / DD §18 | AC-F-12 | P0 |
| FR-WT-036 | Create PR | BD §22 / DD §18 | AC-F-12 | P0 |
| FR-WT-037 | Delete | BD §22 / DD §18 | AC-F-12 | P0 |
| FR-WT-038 | Focus N-hop | BD §24 / DD §37 | AC-F-11 | P0 |
| FR-UI-001 | 无限画布 | BD §24 / DD §30 | AC-F-1 | P0 |
| FR-UI-002 | 搜索定位 | BD §26 / DD §28 | AC-F-9 | P0 |
| FR-UI-003 | Minimap | BD §24 / DD §30 | AC-F-1 | P1 |
| FR-UI-004 | 自动居中 + Fit | BD §24 / DD §30 | AC-F-1 | P0 |
| FR-UI-005 | Focus UI | BD §24 / DD §30 | AC-F-11 | P0 |
| FR-UI-006 | 图层控制 | BD §24 / DD §30 | AC-F-1 | P1 |
| FR-UI-007 | 视图模式切换 UI | BD §25 / DD §29 | AC-F-7 | P0 |
| FR-UI-008 | 状态过滤 | BD §26 / DD §28 | AC-F-9 | P0 |
| FR-UI-009 | Edge 类型过滤 | BD §26 / DD §28 | AC-F-9 | P1 |
| FR-UI-010 | 节点拖动 | BD §24 / DD §31 | AC-F-1 | P0 |
| FR-UI-011 | 节点折叠/展开 | BD §24 / DD §30 | AC-F-1 | P0 |
| FR-UI-012 | 多选 + 框选 | BD §24 / DD §30 | AC-F-1 | P1 |
| FR-UI-013 | 键盘快捷键 | BD §26 / DD §30 | AC-F-1 | P1 |
| FR-UI-014 | Inspector | BD §26 / DD §29 | AC-F-15 | P0 |
| FR-GRAPH-001 | Graph Repository Trait | BD §5 / DD §17 | AC-Q-2 | P0 |
| FR-GRAPH-002 | 11 Node 类型 | BD §17 / DD §7 | AC-F-1 | P0 |
| FR-GRAPH-003 | 13 Edge 类型 | BD §18 / DD §8 | AC-F-1 | P0 |
| FR-GRAPH-004 | Worktree Node 字段 | BD §17 / DD §7 | AC-F-2 | P0 |
| FR-GRAPH-005 | AgentSession Node 字段 | BD §17 / DD §7 | AC-F-15 | P0 |
| FR-GRAPH-006 | CONFLICTS_WITH Edge 字段 | BD §18 / DD §8 | AC-F-6 | P0 |
| FR-GRAPH-007 | Cypher 子集 | BD §11 / DD §27 | AC-Q-2 | P0 |
| FR-GRAPH-008 | Graph Index | BD §15 / DD §26 | AC-P-3 | P0 |
| FR-GRAPH-009 | Graph Clustering | BD §10 / DD §31 | AC-P-2 | P1 |
| FR-GRAPH-010 | Lazy Expansion | BD §10 / DD §31 | AC-P-2 | P1 |
| FR-GRAPH-011 | Graph Delta Update | BD §13 / DD §32 | AC-P-4 | P0 |
| FR-GRAPH-012 | N-hop Query | BD §11 / DD §37 | AC-F-11 | P0 |
| FR-RISK-001 | 11 Risk 类型 | BD §6 / DD §13 | AC-F-5 | P0 |
| FR-RISK-002 | V1 Git Status | BD §6 / DD §33 | AC-F-5 | P0 |
| FR-RISK-003 | V2 Diff Overlap | BD §6 / DD §33 | AC-F-5 | P1 |
| FR-RISK-004 | V3 Symbol/AST | BD §6 / DD §33 | AC-F-5 | P2 |
| FR-RISK-005 | Conflict Prediction | BD §6 / DD §33 | AC-F-5 | P0 |
| FR-RISK-006 | Risk Score 0-1 | BD §22 / DD §35 | AC-F-6 | P0 |
| FR-RISK-007 | Risk 解释 | BD §6 / DD §35 | AC-F-14 | P0 |
| FR-RISK-008 | Risk 实时检测 | BD §13 / DD §32 | AC-P-4 | P0 |
| FR-RISK-009 | Risk 解决跟踪 | BD §13 / DD §35 | AC-F-18 | P1 |
| FR-RISK-010 | Risk 推荐 | BD §6 / DD §35 | AC-F-14 | P1 |
| FR-RISK-011 | Risk Audit Log | BD §6 / DD §44 | AC-Q-10 | P1 |
| FR-AGENT-001 | AgentRuntime Trait | BD §8 / DD §15 | AC-Q-2 | P0 |
| FR-AGENT-002 | Agent Node 显示 | BD §25 / DD §29 | AC-F-7 | P1 |
| FR-AGENT-003 | Agent 状态实时同步 | BD §13 / DD §15 | AC-F-7 | P0 |
| FR-AGENT-004 | 多 Agent 同 Task | BD §25 / DD §29 | AC-F-7 | P2 |
| FR-AGENT-005 | Token Usage | BD §25 / DD §29 | AC-F-7 | P2 |
| FR-AGENT-006 | Tool Calls 计数 | BD §25 / DD §29 | AC-F-7 | P2 |
| FR-AGENT-007 | Agent 历史轨迹 | BD §26 / DD §15 | AC-F-15 | P2 |
| FR-AGENT-008 | Result State | BD §25 / DD §29 | AC-F-7 | P1 |
| FR-EXPLAIN-001 | Explanation API | BD §16 / DD §40 | AC-F-14 | P0 |
| FR-EXPLAIN-002 | 来源限制 | BD §16 / DD §40 | AC-F-14 | P0 |
| FR-EXPLAIN-003 | Tooltip | BD §24 / DD §40 | AC-F-14 | P0 |
| FR-EXPLAIN-004 | Cache | BD §16 / DD §40 | AC-P-5 | P1 |
| FR-SEARCH-001 | Search DSL Parser | BD §26 / DD §28 | AC-F-9 | P0 |
| FR-SEARCH-002 | Graph Query 翻译 | BD §26 / DD §28 | AC-F-9 | P0 |
| FR-SEARCH-003 | NL Query | BD §26 / DD §28 | AC-F-10 | P1 |
| FR-SEARCH-004 | 实时搜索高亮 | BD §24 / DD §28 | AC-F-9 | P0 |
| FR-SEARCH-005 | 搜索历史 | BD §26 / DD §28 | AC-F-9 | P2 |
| FR-SEARCH-006 | 保存搜索 | BD §26 / DD §28 | AC-F-9 | P2 |
| FR-ACTION-001 | ActionEngine Trait | BD §12 / DD §18 | AC-Q-2 | P0 |
| FR-ACTION-002 | Action 3 分类 | BD §22 / DD §18 | AC-F-12 | P0 |
| FR-ACTION-003 | 二次确认 | BD §22 / DD §18 | AC-F-13 | P0 |
| FR-ACTION-004 | RBAC | BD §34 / DD §44 | AC-F-19 | P0 |
| FR-ACTION-005 | Audit Log | BD §35 / DD §43 | AC-F-17 | P0 |
| FR-ACTION-006 | Idempotency Key | BD §22 / DD §42 | AC-Q-3 | P0 |
| FR-ACTION-007 | 撤销 | BD §22 / DD §18 | AC-F-12 | P1 |
| FR-ACTION-008 | 批量操作 | BD §22 / DD §18 | AC-F-12 | P1 |
| FR-ACTION-009 | 进度反馈 | BD §22 / DD §18 | AC-F-12 | P0 |
| FR-ACTION-010 | 错误处理 + Retry | BD §31 / DD §39 | AC-Q-3 | P0 |
| NFR-PERF-001..005 | 性能 5 项 | BD §33 / DD §46 | AC-P-1..5 | P0-P1 |
| NFR-SCALE-001..002 | 可扩展性 | BD §24 / DD §30 | AC-Q-2 | P1 |
| NFR-MAINT-001..002 | 可维护性 | BD §1 / DD §1 | AC-Q-1,2 | P0 |
| NFR-REL-001..002 | 可靠性 | BD §31 / DD §49 | AC-Q-3 | P0 |
| NFR-CON-001 | 事务边界 | BD §31 / DD §40 | AC-Q-3 | P0 |
| NFR-CACHE-001 | 多级缓存 | BD §14 / DD §24 | AC-P-6 | P0 |
| NFR-TEST-001 | 测试覆盖 | BD §36 / DD §45-48 | AC-Q-3..5 | P0 |
| NFR-OBS-001 | 可观测性 | BD §35 / DD §45 | AC-Q-9 | P0 |
| NFR-A11Y-001 | 可访问性 | BD §34 / DD §30 | AC-Q-8 | P1 |
| NFR-PERM-001 | 权限 | BD §34 / DD §44 | AC-F-19 | P0 |
| NFR-SEC-001..002 | 安全 | BD §34 / DD §44 | AC-Q-3 | P0 |
| NFR-AUDIT-001 | 审计 | BD §35 / DD §43 | AC-Q-10 | P0 |
| NFR-I18N-001 | 国际化 | BD §34 / DD §29 | AC-Q-7 | P0 |
| NFR-THEME-001 | 主题 | BD §34 / DD §29 | AC-Q-3 | P0 |
| NFR-KEYB-001 | 键盘 | BD §34 / DD §30 | AC-Q-8 | P0 |

(共 **126 唯一 ID** (103 FR + 23 NFR 子段 = 126 行计数), per §一 总需求数, 此表覆盖 **100%** (126/126 唯一 ID = 100%), 自审校核)

---

## 附录 B: Self Review (STEP 1 自审, per §三十六)

### B.1 Completeness (完整性)

- ✅ §0 文档信息 / 修订履历 — 完整 (1 表 + 1 修订历史表 + 1 撤回记录表)
- ✅ §1 文档目的 — 完整 (5 子段: 目的/背景/包含范围/不包含范围/用户故事)
- ✅ §2 用语定义 — 完整 (52 用语, 覆盖 Worktree / Graph / Risk / Health / Action 等核心概念)
- ✅ §3 业务背景 / 前提条件 — 完整 (3 子段: 业务背景/前提条件 11 项/业务规则 20 项)
- ✅ §4 业务需求 — 完整 (8 子段: FR-WT 38 + FR-UI 14 + FR-GRAPH 12 + FR-RISK 11 + FR-AGENT 8 + FR-EXPLAIN 4 + FR-SEARCH 6 + FR-ACTION 10 = 103 FR + 23 NFR)
- ✅ §5 业务约束 — 完整 (4 子段: 技术栈/架构/数据/UI)
- ✅ §6 使用场景 — 完整 (3 子段: 主流程 + 2 异常流程)
- ✅ §7 数据需求 — 完整 (3 子段: Node Schema 5 项 MVP + Edge Schema 3 项 MVP + 数据来源 4 类)
- ✅ §8 接口需求 — 完整 (2 子段: BFF API 11 项 + SSE Event 15 种)
- ✅ §9 Worktree 状态定义 — 完整 (2 子段: 7 Human State + Machine State)
- ✅ §10 Worktree 卡片字段 — 完整 (12 项 MVP 字段)
- ✅ §11 AI Explanation — 完整 (3 子段: 类型 + 数据流 + Prompt 约束)
- ✅ §12 验收标准 — 完整 (4 子段: 功能 AC 20 项 + 性能 AC 7 项 + 质量 AC 10 项 + 文档 AC 6 项)
- ✅ §13 已知风险 — 完整 (13 缺口 + DDD Review 必查 4 项)
- ✅ §14 签字栏 — 完整 (5 角色)
- ✅ §15 修订历史 — 完整 (1 行 v1.0)
- ✅ 附录 A 需求追踪矩阵 — 完整 (126 唯一 ID 100% 覆盖, v1.1 self-review C-01 修正)
- ✅ 附录 B Self Review — 本段

### B.2 Consistency (一致性, 跨段术语)

- ✅ **Worktree**: §2 / §7 / §9 / §10 全部一致 (Git worktree, AI Agent 并行开发的核心实体)
- ✅ **Human State**: §2 / §9 全部一致 (7 态: RUNNING/WAITING/READY/DIVERGED/CONFLICT/MERGED/STALE)
- ✅ **Risk**: §2 / §4.4 / §11 全部一致 (11 类型, Risk Score 0-1)
- ✅ **Health Score**: §2 / §4.1 / §12 全部一致 (0-100, 13 因素)
- ✅ **Edge**: §2 / §4.3 / §7.2 全部一致 (13 类型)
- ✅ **Node**: §2 / §4.3 / §7.1 全部一致 (11 类型)
- ✅ **Action**: §2 / §4.8 / §22 全部一致 (18 项, 3 分类)
- ✅ **Focus Mode**: §2 / §4.1.38 / §4.2.5 全部一致 (1/2/3-hop)
- ✅ **View Mode**: §2 / §4.1.19-24 / §4.2.7 全部一致 (5 模式)
- ✅ **Semantic Zoom**: §2 / §4.2 / §6 全部一致 (6 级 L0-L5)

### B.3 Traceability (可追踪性)

- ✅ **126 唯一 ID** (103 FR + 23 NFR 子段, 去重后 21 唯一 NFR ID) → 附录 A 追踪矩阵 100% 覆盖 (v1.1 self-review C-01 修正)
- ✅ 每项需求 → BD / DD 章节映射 → AC 验收 → 测试目标 完整闭环
- ✅ 业务规则 20 项 (BR-1..20) 在 §3.3 集中定义, §4 / §7 / §9 引用

### B.4 Ambiguity (歧义性)

- ✅ 每项需求都有 ID + 描述 + 输入 + 输出 + 业务规则 + 优先级 6 字段
- ✅ 无 "根据实际情况决定" 这类无意义描述 (per §三十五)
- ✅ 所有重要功能都有默认方案 (per §三十五), 备选方案仅在 §5.1 技术栈约束 + §13 风险缺口标注
- ✅ 关键术语有明确定义 (per §2 用語集 52 项)

### B.5 Over-design (过度设计)

- ✅ 未过度设计 14 模块边界清晰 (FR-GRAPH-001 / FR-ACTION-001 / FR-AGENT-001 抽象 Trait)
- ✅ 未过度设计 Risk 类型, 严格 11 类型 (per §十三)
- ✅ 未过度设计 Node 类型, 严格 11 类型 (per §七)
- ✅ ⚠️ **Minor #1**: §4.9 NFR 23 项可能略多, 但都在 issue §二十九范围内, 不构成过度设计

### B.6 Under-design (设计不足)

- ✅ Node / Edge Schema 5+3 项 MVP 在 §7 详细定义, DD 详
- ✅ 7 Human State 进入/离开/优先级/UI/动作/风险/需处理 7 字段在 §9.1 完整定义
- ✅ 18 Action 分类在 §4.8 + §二十二 完整定义
- ✅ Health Score 13 因素在 §4.1.6 + §十二 验收定义
- ✅ Risk Engine V1/V2/V3 演进路径在 §4.4 + §十三 定义
- ✅ 性能 5 项 (10/100/500/1000 WT) + 5 层策略 (LOD + Viewport Virtualization + Semantic Zoom + Graph Clustering + Lazy Expansion + Incremental Layout + Incremental Query) 在 §4.9 + §二十七 定义
- ⚠️ **Minor #2**: Graph DB / Canvas Renderer / Git Provider 选型未拍板 (per §十三 缺口 #1/#2/#3), 但本 SRS 阶段不要求拍板, BD §0 决策点拍板 (符合 §三十五: "可以指出备选方案, 但必须选定推荐方案", MVP 推荐方案在 §5.1 给出)

### B.7 Scalability (可扩展性)

- ✅ 4 大抽象 Trait (Graph Repository / Canvas Renderer / Git Provider / Agent Runtime) 保证底层可替换
- ✅ 14 模块解耦保证未来添加模块不破坏现有
- ✅ Event Bus 解耦保证新增事件不破坏现有消费者
- ✅ Risk Engine V1 → V2 → V3 演进路径
- ✅ 5 视图模式 + 6 Semantic Zoom + 18 Action 可扩展

### B.8 Performance (性能)

- ✅ 5 级规模 (10/100/500/1000 WT) 性能目标明确 (AC-P-1..5)
- ✅ 5 层性能策略明确 (LOD + Viewport Virtualization + Semantic Zoom + Graph Clustering + Lazy Expansion + Incremental Layout + Incremental Query)
- ✅ Graph Delta Update 增量避免全量重算 (per §二十六)
- ✅ 多级缓存 (L1 内存 LRU + L2 Redis + L3 Graph DB) 保证 80% 命中率 (NFR-CACHE-001)

### B.9 Security (安全)

- ✅ 5 角色 RBAC 权限 (NFR-PERM-001)
- ✅ 数据加密 (NFR-SEC-001, AES-256 + TLS 1.3)
- ✅ Git 隔离 (NFR-SEC-002, 防 path traversal)
- ✅ Audit Log SCD Type 2 不可篡改 (NFR-AUDIT-001)
- ✅ Source of Truth 不可被 AI 修改 (BR-11)

### B.10 AI Agent Compatibility (AI Agent 兼容性)

- ✅ AgentSession 一等实体 (per §十五)
- ✅ AGENT VIEW 一等视图 (FR-WT-023)
- ✅ AI Explanation 基于结构化数据 (FR-EXPLAIN-002)
- ✅ Graph as Context (R10): Graph 数据库为 AI Agent 提供工程上下文
- ✅ Multi Agent 同 Task 支持 (FR-AGENT-004)

### B.11 Graph Consistency (Graph 一致性)

- ✅ 11 Node + 13 Edge 类型统一 (per §七 / §八)
- ✅ Source of Truth (Git) vs Derived Data (Graph) 边界清晰 (BR-11)
- ✅ Graph Repository Trait 抽象 (per §三十三)
- ✅ Graph Delta Update 增量 (per §二十六)

### B.12 Git Correctness (Git 正确性)

- ✅ 18 Action 覆盖 Git 核心操作 (Create/Open/Compare/Sync/Rebase/Merge/PR/Lock/Delete/Cleanup)
- ✅ Merge Readiness 判断 (FR-WT-018) 避免错误合并
- ✅ Set Dependency (FR-WT-031) 强制依赖顺序
- ✅ Conflict Prediction (FR-RISK-005) 前置检测
- ⚠️ **Minor #3**: Ahead/Behind 算法实现细节依赖 Git Provider 选型, DD 详

### B.13 UX Cognitive Load (UX 认知负荷)

- ✅ R1 Human-first: 7 状态压缩 + 颜色 + 图标 + 文本 4 重编码
- ✅ R5 Progressive Disclosure: 默认只显示当前决策所需信息
- ✅ R6 Semantic Zoom: 6 级信息粒度切换
- ✅ R7 Exception-first: 正常节点弱化, 异常高亮
- ✅ R8 Explainable State: Health Score 13 因素 + AI Explanation
- ✅ R9 Non-destructive Visualization: 画布不直接修改 Git, 高风险走 Action
- ✅ 12 项卡片字段在 §10 严格定义 (不长 commit hash / 不复杂 refs)
- ✅ Inspector 11 tab 替代 Modal (per §二十一)

### B.14 Critical / Major / Minor 问题清单

| 级别 | 数量 | 说明 |
|---|---|---|
| Critical | 0 | 无 |
| Major | 0 | 无 |
| Minor | 3 | 见 §B.5/#1, §B.6/#2, §B.12/#3, 均为合理范围, 不修正 |

**Self Review 结论**: SRS v1.0 通过自审, 无 Critical / Major 问题, 进入 STEP 2 基本设计。