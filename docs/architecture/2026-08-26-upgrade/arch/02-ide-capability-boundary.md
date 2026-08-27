# 02. IDE Capability Boundary Analysis

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **依赖**：[ADR-0022 IDE Placement](../../adr/0022-ide-placement.md) · [STAR vs GitGit Matrix](../../responsibility-matrix/star-vs-gitgit.md)

---

## 1. IDE 能力 4 层分解

按"从底到顶"分层，每层有明确的责任方：

| 层 | 责任方 | 内容 |
|---|---|---|
| L1 Git / Shell / FS | GitGit + 任何 Unix | 标准 Git 命令、文件读写、shell 执行 |
| L2 VCS Provider API | GitGit + GitHub/GitLab/Gitea | Repository / Commit / Branch / Tag API |
| L3 Code Intelligence | STAR | AST / Symbol / Type / Reference / Call Graph / Semantic Search |
| L4 AI Agent / IDE Experience | STAR | RAG / Context Graph / Agent Session / Code Generation / Review / MR Workflow |

**关键判定**：**L3 + L4 全部归 STAR**。L1 + L2 由 Git Provider 提供，GitGit 是其中一个实现。

## 2. 能力归属正交表（30 行核心能力）

| # | 能力 | L1 Git/Shell | L2 VCS Provider | L3 Code Intel | L4 AI/IDE |
|---|---|---|---|---|---|
| 1 | `git clone` | ✅ GitGit | — | — | — |
| 2 | `git worktree add` | ✅ GitGit | — | — | — |
| 3 | Smart HTTP / SSH | ✅ GitGit | — | — | — |
| 4 | 仓库级 REST API | — | ✅ GitGit | — | — |
| 5 | Commit/Tree/Blob 元数据 | — | ✅ GitGit | — | — |
| 6 | Diff / Blame / History | — | ✅ GitGit | — | — |
| 7 | AST 解析 | — | — | ✅ STAR | — |
| 8 | Symbol Index | — | — | ✅ STAR | — |
| 9 | Type Info | — | — | ✅ STAR | — |
| 10 | Call Graph | — | — | ✅ STAR | — |
| 11 | Find References | — | — | ✅ STAR | — |
| 12 | Semantic Search | — | — | ✅ STAR | — |
| 13 | Code Embedding | — | — | — | ✅ STAR |
| 14 | RAG | — | — | — | ✅ STAR |
| 15 | Context Graph | — | — | — | ✅ STAR |
| 16 | Agent Session | — | — | — | ✅ STAR |
| 17 | Prompt 管理 | — | — | — | ✅ STAR |
| 18 | Tool Calling | — | — | — | ✅ STAR |
| 19 | Code Generation | — | — | — | ✅ STAR |
| 20 | Code Explanation | — | — | — | ✅ STAR |
| 21 | Code Review 建议 | — | — | — | ✅ STAR |
| 22 | MR Workflow | — | — | — | ✅ STAR |
| 23 | CI/CD 编排 | — | — | — | ✅ STAR |
| 24 | Approval | — | — | — | ✅ STAR |
| 25 | Human-in-the-Loop | — | — | — | ✅ STAR |
| 26 | Multi-Agent | — | — | — | ✅ STAR |
| 27 | Agent Lease | — | — | — | ✅ STAR |
| 28 | Agent Resume | — | — | — | ✅ STAR |
| 29 | Web UI | — | — | — | ✅ STAR |
| 30 | Project-level 规范 | — | — | — | ✅ STAR |

## 3. 边界守门规则

### 3.1 GitGit 不应提供

- 任何"理解代码语义"的能力（AST / Symbol / Type / Reference / Call Graph）
- 任何"理解用户意图"的能力（Task / Issue / RAG / Agent）
- 任何"驱动 IDE 行为"的能力（Open File / Selection / Diagnostic 上报）
- 任何"理解企业"的能力（RBAC / Approval / Compliance）

### 3.2 STAR 不应直接做

- 绕过 Version Control Provider 直接写 commit（必须经 GitGit/GitHub/GitLab/Gitea）
- 绕过 L1 直接调 fs（必须用 Git worktree 隔离）
- 复制 Git 对象（必须用 LFS / Git LFS 标准）

## 4. 能力暴露的"是否暴露"决策表

| 能力 | 是否暴露到外部 | 暴露协议 |
|---|---|---|
| GitGit Repository 列表 | ✅ | Git Provider REST API + MCP server |
| GitGit Commit 内容 | ✅ | Git Protocol + REST + MCP |
| GitGit Branch 操作 | ✅ | Git Protocol + REST + MCP |
| GitGit Worktree 操作 | ✅ | Git worktree 命令 + REST + MCP |
| STAR Symbol Index | ✅ | LSP + MCP + REST |
| STAR Issue | ❌ 内部 + ✅ MCP | MCP |
| STAR Agent Session | ❌ 内部 + ✅ MCP (limited) | MCP |
| STAR MR Workflow | ❌ 内部 + ✅ MCP + REST | MCP + REST |
| STAR RAG | ❌ 内部 | 不暴露 |
| STAR Approval | ❌ 内部 | 不暴露 |
| STAR Human-in-the-Loop | ❌ 内部 | 通过 Agent Lifecycle 暴露状态 |

## 5. 风险登记

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 边界漂移（GitGit 偷偷加 IDE 能力） | 中 | 高 | Phase D 必跑 `gitgit-ide-boundary.md` §7 测试 |
| 边界漂移（STAR 偷偷调 Git 对象层） | 中 | 高 | Code review + Linter |
| L3/L4 边界混淆 | 中 | 中 | ADR-0022 + 责任矩阵守门 |

## 6. 签字栏 / 修订历史

per [arch/01-current-architecture-analysis.md](01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
