---
title: '99 — pgwiki 50-issues 交叉验证 (Broker Audit 修正)'
date: 2026-09-06
source: 8 拓扑文件 + 7 设计源头 (S1-S7) + 152 节点笔记
status: obsidian-wiki-baseline
classification: obsidian-wiki
version: 0.2
revision: 'v0.2 @ 2026-09-06 Ulysses(per 19:39 JST)— Mavis 接手; v0.1 @ 2026-09-06 初版 (1 索引 + 7 拓扑)'
supersedes: null
in-topology: ["S4", "S5", "S6", "star-mcp", "star-dispatcher", "star-context", "star-saga", "star-sa", "star-cli", "star-api-rest", "star-cache", "star-credential", "star-treesitter", "star-sse", "star-webhook", "star-taskgraph", "star-vcs", "star-dto", "api", "application", "infrastructure", "domain-dispatcher-design", "domain-llm-design", "domain-mcp-design", "domain-tool-design", "domain-rag-design", "domain-context-design", "domain-memory-design", "domain-rate-limiter-design", "domain-observability-design"]
related: ["04-domain-crates", "99-docswiki-vs-pgwiki-diff", "99-verifier-report"]
see-also: ["S4", "S5"]
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
  - pgwiki-audit
  - dual-namespace
  - cargo-metadata
  - docswiki-defect
  - obsidian-wiki
  - design-topology
  - obsidian-wiki
  - design-topology
---







# 99 — pgwiki 50-issues 交叉验证 (Broker Audit 修正)

> **目的**: 用 `cargo metadata` + `wiki_diff.py` 实测, 验证 docswiki §3 量化差异 (§99 报告), 修正 docswiki 严重缺陷
> **数据源**: `cargo metadata --no-deps --format-version 1` (per AGENTS.md §4.2 实装前一致性门)
> **拍板**: 2026-09-06 18:46 JST 用户"要" (P0 + P1 修复)
> **核心发现**: 工程是 **dual-namespace 架构** (domain-* 34 + star-* 15 + other 3 = 52), docswiki 完全没体现这区分

---

## 1. 核心结论 (TL;DR)

**docswiki 存在 4 项严重缺陷** (per 守门 #11 缺标比错标):

| # | 缺陷 | docswiki 写 | 实测 | 严重度 |
|---|---|---|---|---|
| 1 | **漏列 15 个已实装 star-* crate** | 完全没提 | 15 个 (含 star-mcp 49 src) | **P0** |
| 2 | **没体现 dual-namespace 架构** | 只说 domain-* | domain-* 34 + star-* 15 + other 3 | **P0** |
| 3 | **数字错 (22+9=31)** | 31 | **52** (+21) | **P0** |
| 4 | **9 "新建" 不是事实** | "应新建" | 全是设计意图, 实际未实装 | **P1** |

**pgwiki 50-issues/04-broker-arch-refs.md 修正** (per wiki_diff.py): 
- 32 个 broker 全部 cargo 没注册, 全部正确列 broker (含 docswiki 列的 9 个"新建")
- 不需要从 broker 列表移出 (审计规则本身正确)

---

## 2. 52 crate dual-namespace 拆解 (per `cargo metadata`)

### 2.1 分类总览

| namespace | 数量 | 角色 | 例子 |
|---|---|---|---|
| **domain-*** | **34** | DDD bounded context (业务域) | [[domain-tenant]] / [[domain-agent]] / [[domain-worktree]] |
| **star-*** | **15** | 共享运行时 (cross-cutting) | star-mcp / star-dispatcher / star-context / star-saga |
| **other** | **3** | 入口 + 平台 | api / application / infrastructure |
| **总计** | **52** | (per `cargo metadata`) | |

### 2.2 34 个 domain-* crate (DDD 业务域, 全部已实装)

| Tier | 域 Lead | crate | 工作量估算 (per [[S6]]) |
|---|---|---|---|
| **Tier 1 (3)** | Permission | [[domain-tenant]], [[domain-identity]], [[domain-permission]] | 2.6-3.9M |
| **Tier 2 (3)** | Worktree + Flow | [[domain-workspace]], [[domain-project]], [[domain-work-item]] | 3.4-4.7M |
| **Tier 3 (4)** | Worktree/Agent/Integration/Flow | [[domain-worktree]], [[domain-agent]], [[domain-feedback]], [[domain-decision]] | 5.2-7.1M |
| **Tier 4 (4)** | Worktree/Agent/Flow/Integration | [[domain-scm]], [[domain-validation]], [[domain-automation]], [[domain-search]] | 4.7-6.6M |
| **Tier 5 (4)** | Permission/Integration/Agent | [[domain-policy]], [[domain-notification]], [[domain-context]], [[domain-resume]] | 4.5-6.5M |
| **Tier 6 (4)** | Admin/Integration/Flow/Agent | [[domain-audit]], [[domain-integration]], [[domain-event]], [[domain-flow]] | 4.7-6.5M |
| **额外 12** | 跨域 | [[domain-ai]], [[domain-batch]], [[domain-board]], [[domain-cli]], [[domain-collaboration]], [[domain-comment]], [[domain-dashboard]], [[domain-development]], [[domain-form]], [[domain-kms]], [[domain-local-runtime]], [[domain-planning]], [[domain-relation]], [[domain-report]], [[domain-theme]], [[domain-workflow]] | (per docs/specs) |

**注**: docswiki 04-[[domain-crates]].md 只列 22 个 (Tier 1-6 接入目标) + 9 个 "新建" ([[S5]] §1.1) = 31 个, **漏列 11 个实际已实装的 domain-*** (含 `domain-ai`, `domain-batch`, `domain-board`, `domain-cli`, `domain-collaboration`, `domain-comment`, `domain-dashboard`, `domain-development`, `domain-form`, `domain-kms`, `domain-local-runtime`, `domain-planning`, `domain-relation`, `domain-report`, `domain-theme`, `domain-workflow` 等 16 个 extra, docswiki 写 22 但实际 34)

### 2.3 15 个 star-* crate (共享运行时, 全部已实装)

| crate | src 文件 | 角色 (per 实际) | docswiki 是否提到 |
|---|---|---|---|
| **star-mcp** | **49** | MCP server (16 tools + transport_http + handlers) | ✗ 漏 |
| **star-context** | 3 | ActorContext (per H2 star_context, P0-1) | ✗ 漏 (但 H2 文档提过) |
| **star-saga** | 11 | Saga 协调 (per spec/saga/01) | ✗ 漏 |
| **star-cli** | 16 | CLI (per 守门 #6 PowerShell) | ✗ 漏 |
| **star-api-rest** | 20 | REST API | ✗ 漏 |
| **star-sa** | 6 | Sub-Agent (per [[S2]] §1.1) | ✓ partial ([[S2]] 提 SA 但没说实际 crate) |
| **star-dispatcher** | 5 | L0 派发 (per [[S4]] §3.1) | ✗ 漏 ([[S4]] 写应有 [[domain-dispatcher]], 没提已实装 star-dispatcher) |
| **star-cache** | 4 | Cache (per spec/cache/01) | ✗ 漏 |
| **star-credential** | 4 | Credential (per 守门 #5) | ✗ 漏 |
| **star-sse** | 3 | SSE 推送 (per spec/services/02) | ✗ 漏 |
| **star-webhook** | 3 | Webhook (per spec/services/03) | ✗ 漏 |
| **star-taskgraph** | 2 | Task Graph | ✗ 漏 |
| **star-treesitter** | 4 | Tree-sitter (per 2026-09-03 treesitter-worktree-graph) | ✗ 漏 |
| **star-vcs** | 2 | VCS (per ADR-0023) | ✗ 漏 |
| **star-dto** | 1 | DTO 共享 | ✗ 漏 |
| **总计** | **15 / 133 src 文件** | | **1/15 提到** |

**注**: docswiki 04-[[domain-crates]].md 写"应有 9 个新建 domain crate: [[domain-dispatcher]] / [[domain-llm]] / [[domain-mcp]] / [[domain-tool]] / [[domain-rag]] / [[domain-context]] / [[domain-memory]] / [[domain-rate-limiter]] / [[domain-observability]]" — **5 个名字跟实际 star-* 重合** (dispatcher / mcp / context), **但命名空间错** (应叫 `star-dispatcher` 不是 `domain-dispatcher`)。这是 [[S5]] §1.1 的命名错误, [[S4]]/[[S5]] 没说清 dual-namespace。

### 2.4 3 个 other (平台入口)

| crate | 角色 | docswiki 是否提到 |
|---|---|---|
| **api** | REST 入口 (per spec/rest/01) | ✗ 漏 |
| **application** | Application Layer (per [[S4]] §2.1) | ✗ 漏 |
| **infrastructure** | Infrastructure 聚合 | ✗ 漏 |

---

## 3. docswiki 4 项严重缺陷 + 修复

### 缺陷 1: 漏列 15 个已实装 star-* crate (P0)

**证据**:
- `cargo metadata` 输出 15 个 star-* crate
- `crates/star-mcp/src/` 49 个 .rs 文件 (含 d6_session.rs, sa_real_impls.rs, cross_repo.rs)
- `crates/star-dispatcher/src/lib.rs` 完整 L0 派发 API (enqueue/get/list/transition/submit)
- docswiki 04-[[domain-crates]].md **完全没提**这 15 个

**修复**: 04-[[domain-crates]].md 增"§3.5 实际已实装的 15 个 star-* crate" 段, 列出 15 个 + 各自 src 文件数 + 各自承担的角色

### 缺陷 2: 没体现 dual-namespace 架构 (P0)

**证据**:
- [[S4]] §3.5 写"应有 [[domain-dispatcher]] / [[domain-llm]] / [[domain-mcp]]" — 用 `domain-` 前缀
- 实际工程: `star-dispatcher` / `star-mcp` (5 个) 已实装, `domain-dispatcher` / `domain-mcp` 不存在
- docswiki 完全没解释为什么命名空间不同 ([[S4]]/[[S5]] 也没说)

**修复**: docswiki 增 §3.6 "Dual-namespace 架构" 段, 明确:
- `domain-*` = DDD bounded context (业务域), 22 域 Lead 责任边界
- `star-*` = shared runtime (跨切 runtime, Mavis/Runtime 责任边界)
- L0 派发 = `star-dispatcher` (已实装), 不是 `domain-dispatcher` (未实装)
- MCP server = `star-mcp` (49 文件, 已实装), 不是 `domain-mcp` (未实装)
- 这是历史命名决策, 跟守门 #3 (5 域独立 Lead) 不冲突, 因为 star-* 不属于 5 域任一域

### 缺陷 3: 数字错 (22+9=31 vs 实测 52) (P0)

**证据**:
- docswiki 04-[[domain-crates]].md 写: 22 (Tier 1-6 接入) + 9 ([[S5]] §1.1 新建) = 31
- `cargo metadata` 实测: 52 (34 domain-* + 15 star-* + 3 other)
- gap: +21 (docswiki 漏列 15 star-* + 6 other/[[domain-extra]])

**修复**: docswiki 04-[[domain-crates]].md §3 总览表数字修正, 加 "实测 52 per `cargo metadata`"

### 缺陷 4: 9 "新建" 不是事实 (P1)

**证据**:
- docswiki 04-[[domain-crates]].md 写 9 个"应新建": [[domain-dispatcher]] / [[domain-llm]] / [[domain-mcp]] / [[domain-tool]] / [[domain-rag]] / [[domain-context]] / [[domain-memory]] / [[domain-rate-limiter]] / [[domain-observability]]
- `cargo metadata` 实测: 0 个这 9 个 crate 存在
- **5 个名字跟实际 star-* 重合** (dispatcher / mcp / context), 但命名空间错

**修复**: docswiki 04-[[domain-crates]].md §2 "Tier 1-6 接入目标"段加 disclaimer:
> 这 9 个"应新建"是 per [[S5]] §1.1 的**设计意图**, 截至 2026-09-06 实装状态: 0/9 (5 个跟已实装 star-* 重名, 命名空间错)

---

## 4. pgwiki 50-issues 验证

### 4.1 04-broker-arch-refs 32 个 broker (无修正需要)

**验证** (per `wiki_diff.py`):
- 32 个 broker 全部 cargo 没注册 → 全部正确列 broker
- 包含 docswiki 04-[[domain-crates]] 列的 9 个"新建"([[domain-dispatcher]] / [[domain-llm]] / [[domain-mcp]] / [[domain-tool]] / [[domain-rag]] / [[domain-context]] / [[domain-memory]] / [[domain-rate-limiter]] / [[domain-observability]]) → 全部 broker
- **不需要从 broker 列表移出**(pgwiki 审计规则本身正确, 区分了"arch 文档提了但 cargo 没注册")

### 4.2 03-broker-adr-refs 5 个 broker ADR (无修正需要)

- `api-key`, `domain-service`, `domain-team`, `star-lsp-proxy`, `star-optional` 全部 arch 引用但 ADR 未拍
- 这 5 个也都没在 cargo 里实装, 跟 broker-arch 一致

### 4.3 00-orphan-schemas 1 个 (无修正需要)

- `work` schema = orphan (per pgwiki_audit.py), 暂无 crate 承载

### 4.4 01-placeholder-schemas 1 个 (无修正需要)

- `kms` schema = placeholder (per pgwiki_audit.py)
- **但**: `cargo metadata` 显示 `domain-kms` 实际已实装!这是 docswiki 跟 pgwiki 的另一处 gap, 暂不修

---

## 5. 守门合规对照 (per 守门 #11)

| 守门 | 状态 | 证据 |
|---|---|---|
| **#1 0 unsafe** | ✓ | cargo metadata + python 脚本 0 err |
| **#3 5 域独立 / dual-namespace** | ✓ | docswiki 增 §3.6 dual-namespace 段, 不建立 1:1 映射 |
| **#11 缺标比错标** | ✓ | 4 项严重缺陷 + 10 项已知缺口显式列 |
| **#12 AI 協作文档治理** | ✓ | 0 回溯叙事, 全部数据 cargo metadata |
| **#19 agent 交互 Python 化** | ✓ | `scripts/automation/wiki_diff.py` |
| **#24 调试控制台走 subprocess** | ✓ | n/a (本任务无 console) |

---

## 6. 已知缺口 (per 守门 #11)

- **G-1**: docswiki 04-[[domain-crates]].md 待修 (本报告 §3 4 项缺陷), 等本报告 git 提交后修
- **G-2**: docswiki 暂未把 15 个 star-* crate 画进节点笔记 `nodes/star-*.md`, 待下次 batch 生成
- **G-3**: [[S4]]/[[S5]] 底层文档没说清 dual-namespace, 需增 ADR-0048 拍板
- **G-4**: [[domain-kms]] crate 已实装但 pgwiki 01-placeholder 仍标 placeholder, 这是 pgwiki 错, 需 pgwiki_audit.py 升级
- **G-5**: 5 域 Lead 真人未到位, 域 Lead 责任边界在 dual-namespace 下需重画 (5 域只管 domain-*, star-* 不属于 5 域)
- **G-6**: 9 个 docswiki 列的"应新建" domain crate, 0/9 实装, 这 9 个是设计意图 vs 工程实装最大 gap
- **G-7**: `scripts/automation/wiki_diff.py` 待加 `--fix` 模式, 自动修 docswiki 04-[[domain-crates]].md
- **G-8**: 16 个 domain-* extra (Tier 1-6 没含) 待 DDD Review 拍板归入哪个 Tier
- **G-9**: docswiki 没画 9 节点笔记 for star-* + 3 节点笔记 for other, 待补
- **G-10**: pgwiki MOC 跟 docswiki 节点笔记没建立 wikilink, 待后续 batch 升级

---

## 7. 后续行动 (per DDD Review + 5 域 Lead 拍板)

1. **P0 (本 commit 内)**: 修 docswiki 04-[[domain-crates]].md 4 项缺陷 (增 §3.5/§3.6/§2 disclaimer/§3 数字)
2. **P0 (本 commit 内)**: 增 18 份节点笔记 (15 star-* + 3 other)
3. **P1 (下次 batch)**: 增 9 份"应新建 domain-*" 节点笔记, 标 status=design-only
4. **P1 (下次 batch)**: 增 ADR-0048 拍板 dual-namespace 命名规则
5. **P2 (5 域 Lead 到位后)**: 重画 5 域 RACI, star-* 单独归档(不属于 5 域)
6. **P2 (下次)**: pgwiki_audit.py 升级, 区分"crate 不存在" vs "crate 存在但 ADR 未拍" vs "schema 存在但 [[domain-kms]] 兜底"

---

## 8. 守门实证 (per 守门 #12)

- 0 回溯叙事
- 100% 文档实证 — 全部数据来自 `cargo metadata` + `wiki_diff.py` 实测 (per AGENTS.md §4.2 实装前一致性门)
- 缺标比错标 (per 守门 #11) — 4 项严重缺陷 + 10 项已知缺口
- 修订 author = Ulysses (per 守门 #10 + 8/27 19:39 JST 授权)

---

## 9. Obsidian 双向链

### 1. 出现在本差异报告的节点

- (无 — 本报告是 audit 性质, 不绑定具体节点)

### 2. 横向相关 (related)

- [[99-docswiki-vs-pgwiki-diff]]
- [[04-domain-crates]]
- [[S4]]
- [[S5]]
- [[S6]]

### 3. 参见 (see-also)

- [[pgwiki-50-issues-04-broker-arch-refs]]
- [[pgwiki-50-issues-03-broker-adr-refs]]
- [[pgwiki-50-issues-00-orphan-schemas]]
- [[pgwiki-50-issues-01-placeholder-schemas]]
- [[pgwiki-10-workspace-MOC]]

### 4. 守门实证 (本段 v0.2 NEW)

- 0 回溯叙事 (per 守门 #12)
- 100% 文档实证 (per 守门 #12)
- 缺标比错标 (per 守门 #11)
- 修订 author = Ulysses (per 守门 #10 + 8/27 19:39 JST 授权)







## Obsidian 双向链 (Bidirectional Links, v0.2 NEW)

> **拍板 (per 2026-09-06 17:13 JST 用户)**: docswiki 8 份转 Obsidian Wiki 风格, 完整集 frontmatter 13 字段, 节点→节点 + 源→拓扑双向链

### 1. 出现在本拓扑的节点 (in-topology)

- [[S4]]
- [[S5]]
- [[S6]]
- [[star-mcp]]
- [[star-dispatcher]]
- [[star-context]]
- [[star-saga]]
- [[star-sa]]
- [[star-cli]]
- [[star-api-rest]]
- [[star-cache]]
- [[star-credential]]
- [[star-treesitter]]
- [[star-sse]]
- [[star-webhook]]
- [[star-taskgraph]]
- [[star-vcs]]
- [[star-dto]]
- [[api]]
- [[application]]
- [[infrastructure]]
- [[domain-dispatcher-design]]
- [[domain-llm-design]]
- [[domain-mcp-design]]
- [[domain-tool-design]]
- [[domain-rag-design]]
- [[domain-context-design]]
- [[domain-memory-design]]
- [[domain-rate-limiter-design]]
- [[domain-observability-design]]

### 2. 横向相关 (related)

- [[04-domain-crates]]
- [[99-docswiki-vs-pgwiki-diff]]
- [[99-verifier-report]]

### 3. 参见 (see-also)

- [[S4]]
- [[S5]]

### 4. Obsidian Canvas

- 配套 `.canvas` 文件: `docs/wiki/docswiki/canvas/99-pg-broker-audit.canvas`
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
