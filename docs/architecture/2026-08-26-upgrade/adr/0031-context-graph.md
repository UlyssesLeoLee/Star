# ADR-0031: Context Graph 范围

> **状态**：🟢 Accepted v0.2 (per 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 守门 #12 docs 触发 + 守门 #10 author=Ulysses)
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签（per §6 签字栏）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)（待归档）
> **依赖**：[ADR-0026 STAR AI Compat](0026-star-ai-compat.md) · [ADR-0029 Universal Submit](0029-universal-submit.md) · [ADR-0030 Agent Resume](0030-agent-lease-heartbeat-resume.md)
> **关联**：[arch/03 STAR AI Compat Arch](../arch/03-star-ai-compat-arch.md) · [spec/context/01 Context API](../spec/context/01-context-api.md)

---

## 1. 背景与问题

Agent 接 STAR Issue 时，盲目在 Repository 中找上下文 = 浪费时间 + 烧 token + 漏信息（per spec/context/01 §1）。

需要 Context API `star context get STAR-1024` 一站式拉取当前任务所有相关信息。但 Context 本身需要一个 **Context Graph** 作为底层数据模型：

- 哪些节点必须支持？
- 哪些关系必须支持？
- 哪些放 Phase 2+？

需要明确 MVP 阶段 Context Graph 的范围边界。

## 2. 决策

**MVP 阶段 Context Graph 严格限制为 4 类节点 + 5 类关系。Phase 2+ 再扩展。**

### 2.1 MVP 4 类节点（per spec/context/01 §4.1）

| 节点 | 字段 |
|---|---|
| **Issue** | id / title / status / labels |
| **Repository** | id / provider / url |
| **Worktree** | id / path / branch / head_commit |
| **Commit** | sha / author / message / files_changed |

### 2.2 MVP 5 类关系（per spec/context/01 §4.2）

| 关系 | 含义 |
|---|---|
| `implements` | Worktree implements Issue |
| `modifies` | Commit modifies Worktree |
| `references` | Commit references Issue |
| `belongs_to` | Worktree belongs_to Repository |
| `derived_from` | Commit derived_from Commit (parent) |

### 2.3 Phase 2+ 留待（per spec/context/01 §4.3）

**节点**（12 类待补）：
- Symbol / File / MR / Test / Pipeline / Deployment / Incident
- Agent / User / Document / Package / Vulnerability

**关系**（10 类待补）：
- `depends_on` / `generated_by` / `reviewed_by` / `tested_by` / `deployed_by`
- `caused_by` / `fixed_by` / `related_to` / `located_in` / `opened_in`

### 2.4 Context API 响应（per spec/context/01 §3）

```bash
star context get STAR-1024 --json
star context current --json
star context get STAR-1024 --depth=full --json
```

返回 14 字段：
`issue` / `requirement` / `acceptance_criteria` / `related_issues` / `related_mr` /
`architecture_decisions` / `relevant_documents` / `relevant_files` / `relevant_symbols` /
`relevant_tests` / `relevant_dependencies` / `historical_changes` / `schema_version` (= `context-api/v1`)

### 2.5 检索流程（per spec/context/01 §5）

```
Graph Narrowing
   ↓
Semantic Retrieval
   ↓
Code Retrieval
   ↓
Symbol Retrieval
   ↓
LLM Context
```

**避免把整个 Repository 塞给模型**。

### 2.6 Token Budget（per spec/context/01 §6）

| Depth | Token 上限 |
|---|---|
| `minimal` (默认) | < 5K tokens |
| `normal` | < 20K tokens |
| `full` | 无上限（必须显式指定） |

### 2.7 关键架构约束

- MVP 严格 4 + 5，不增加节点 / 关系类型
- 节点和关系 schema 由 `context-api/v1` 稳定化（per acceptance/13 Schema Stability）
- Token Budget 不可超 `normal` 默认（避免烧 token）
- Graph Narrowing 必须先于 Semantic Retrieval（避免大海捞针）
- Context API 不直接读 GitHub / GitLab / Gitea（per ADR-0023 Provider 抽象）

### 2.8 实施位置（per spec/context/01 §7）

- `crates/star-context/` — Context service
- `crates/star-context/src/graph.rs` — 简化版 context graph (4 节点 + 5 关系)
- `crates/star-context/src/retrieval.rs` — 4 段检索 pipeline

## 3. 备选方案与拒绝理由

### 备选 A：MVP 阶段就支持 12+ 节点 / 15+ 关系
- 拒绝理由：MVP 范围爆炸；graph storage / query engine 实现风险高；2/3 节点类型在 MVP 实际用不到

### 备选 B：不用 Graph，直接用 RAG 向量数据库
- 拒绝理由：丢失显式关系推理；RAG 黑盒不透明；不利于 Audit / 错误调试

### 备选 C：Context Graph 走 Git Provider（GitHub / GitLab）
- 拒绝理由：违反 ADR-0023 Provider 抽象；GitHub 不支持自定义 graph 节点

## 4. 后果与影响

### 4.1 正面

- Agent 一站式拉取当前任务所有相关信息（per spec/context/01 §1 目标）
- 4 节点 + 5 关系 = MVP 范围可控，2-3 个月内可实装
- Token Budget 3 级，避免烧 token
- Context API 输出稳定（`context-api/v1`）

### 4.2 负面 / 成本

- 节点 / 关系覆盖范围有限（Symbol / MR / Test 等 Phase 2+ 才有）
- 复杂查询（"所有依赖 STAR-1024 的 Symbol"）MVP 阶段不支持
- 4 段检索 pipeline 调优成本

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 4 节点不够用 | 中 | 中 | 快速迭代到 Phase 2+ 12 节点 |
| 5 关系有缺（`assigned_to` Agent 缺） | 中 | 中 | Agent 信息存 Issue 节点 metadata，Phase 2+ 独立 |
| Token Budget `normal` < 20K 仍过大 | 中 | 低 | `minimal` < 5K 默认开启 |

## 5. 与其他 ADR 的关系

- **依赖**：[ADR-0026 STAR AI Compat](0026-star-ai-compat.md) — 5 通道都需要 Context API
- **依赖**：[ADR-0029 Universal Submit](0029-universal-submit.md) — Submit 12 步第 4 步 Diff / 第 5 步 Validation 用 `relevant_files`
- **依赖**：[ADR-0030 Agent Resume](0030-agent-lease-heartbeat-resume.md) — Resume payload 11 字段第 9 个 `relevant_context` 来自 Graph
- **被依赖**：MCP `get_context` tool (per ADR-0032) 直接调用 Context API



---

## 5.5. 实施状态 (per OPT-NEXT-08 v0.2 拍板, 守门 #12 docs 触发)

> **本节由 OPT-NEXT-08 (2026-09-07 14:30 JST) 升版 v0.1 → v0.2 时落地, 提供实施证据链**

| 维度 | 内容 |
|---|---|
| **决策 scope** | Context Graph (MVP 4 节点 + 5 关系, Phase 2+ 12+10 节点/关系) |
| **目标 crate / 落地位置** | `star-context` |
| **落地 commit (per `git log -p --follow`)** | 无 commit (Phase 2+ 推进) |
| **守门 #1 v19** | `cargo check --workspace --all-targets -j 4` 0 err 32.27s (per 9/3 RF-001 T1.5 step 1 验证) |
| **守门 #3 v2** | Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 拍板 B), author=Ulysses (per 守门 #10) |
| **守门 #4.2** | Runtime 名称实装前一致性门 — 本 ADR 落档即满足"概念→物理 crate"映射, 后续实装前必先 ADR 拍板 |
| **守门 #10** | commit author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST + 21:59 JST 三次强化) |
| **守门 #12** | docs 同步 = 实施前 git log --follow 实证; 缺标比错标安全 (per 8/26 JST) |
| **守门 #13 W/T/M** | 本 ADR 不涉及 DB schema 分类 (架构原则 / 决策类), 不触发 W/T/M 横展開 |
| **守门 #14 v2** | 5 域 Lead CONTENT 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界) 已显式列出 (per 9/3 19:43 JST) |

**实施完成度**:
- ✅ ADR 决策本身落地 (本节"决策"内容)
- ⏳ 后续实装 = 等 P3-B/F/H 拍板启动 + 5 域 Lead 真人到位 (per ADR §"签字栏" DDD Review 阶段补)

**已知缺口 (per 守门 #11 缺标比错标)**:
- 5 域 Lead 真人到位前, 实施由 Mavis 临时代签 (per 9/3 11:35 JST 拍板 B), 真人到位后追溯签字
- 本 ADR 实施状态只反映 commit / 守门 0 违反 实证, 不反映 22 domain 业务实装进展 (per P3-A H2 阶段 11/25 实证)

## 6. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | Platform Engineer | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.2 | 2026-09-07 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | OPT-NEXT-08 升版 v0.1 → v0.2: 新增 §5.5 实施状态 (scope/crate/commit/守门 0 违反 实证) + 修订历史 v0.2 行 (per 守门 #10 author=Ulysses + 守门 #12 docs 触发 + 守门 #11 缺标比错标) | 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 23 草案 ADR 一次性升版 |
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版：MVP 4 节点 + 5 关系 + 12 Phase 2+ 节点 / 10 Phase 2+ 关系 + 3 段 Token Budget | Phase B 起草（per 2026-08-26 升级 Plan） |
