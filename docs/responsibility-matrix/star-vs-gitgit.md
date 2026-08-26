# 责任矩阵：STAR vs GitGit

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签
> **依赖**：[ADR-0022 IDE Placement](../adr/0022-ide-placement.md)
> **关联**：[GitGit IDE Boundary Spec](gitgit-ide-boundary.md)

---

## 1. 责任划分正交表

> 任何能力**且仅且**出现在 STAR 或 GitGit 一侧。出现重复 = 立刻标红 + 推动重写。

| # | 能力 | STAR | GitGit |
|---|---|---|---|
| 1 | Repository | — | ✅ |
| 2 | Git Object Database | — | ✅ |
| 3 | Commit / Branch / Tag / Ref | — | ✅ |
| 4 | Diff / Blame / History | — | ✅ |
| 5 | Merge / Rebase / Conflict Detection | — | ✅ |
| 6 | Git Protocol / SSH / Smart HTTP | — | ✅ |
| 7 | Git LFS | — | ✅ |
| 8 | Repository Mirror | — | ✅ |
| 9 | Worktree 底层（add/remove/list/status） | — | ✅ |
| 10 | Protected Branch 底层能力 | — | ✅ |
| 11 | Protected Tag 底层能力 | — | ✅ |
| 12 | CODEOWNERS 底层解析 | — | ✅ |
| 13 | Git 原生权限边界 | — | ✅ |
| 14 | Git 原生事件（RefUpdated / CommitCreated / ...） | — | ✅ |
| 15 | Repository Snapshot | — | ✅ |
| 16 | Object Streaming | — | ✅ |
| 17 | Partial Clone / Sparse Checkout | — | ✅ |
| 18 | Large File Support | — | ✅ |
| 19 | File-level History | — | ✅ |
| 20 | Commit Graph | — | ✅ |
| 21 | File Change Events（WebHook） | — | ✅ |
| 22 | Git-compatible Storage | — | ✅ |
| 23 | Git-compatible Transport | — | ✅ |
| **24** | **Workspace（研发工作空间）** | ✅ | — |
| 25 | Project / Issue / Task | ✅ | — |
| 26 | Work Graph | ✅ | — |
| 27 | Kanban / Sprint / Roadmap | ✅ | — |
| 28 | Requirement / Design / Documentation | ✅ | — |
| 29 | AI Agent Orchestration | ✅ | — |
| 30 | Context Management / RAG | ✅ | — |
| 31 | Knowledge Graph | ✅ | — |
| 32 | Code Intelligence（AST / Symbol / Type） | ✅ | — |
| 33 | Code Navigation | ✅ | — |
| 34 | IDE Gateway | ✅ | — |
| 35 | AI Coding Gateway | ✅ | — |
| 36 | Workspace Orchestration | ✅ | — |
| 37 | Worktree ↔ Issue 绑定（高层） | ✅ | — |
| 38 | MR Workflow | ✅ | — |
| 39 | CI/CD Orchestration | ✅ | — |
| 40 | Security / Compliance | ✅ | — |
| 41 | Approval（企业级） | ✅ | — |
| 42 | Analytics / DORA / Value Stream | ✅ | — |
| 43 | Human + AI Collaboration | ✅ | — |
| 44 | Agent Session / Agent Identity | ✅ | — |
| 45 | Agent Permission / Agent Lease | ✅ | — |
| 46 | Human-in-the-Loop | ✅ | — |
| 47 | Multi-Agent Coordination | ✅ | — |
| 48 | Agent Resume / Handoff | ✅ | — |
| 49 | Issue ↔ Code 关联 | ✅ | — |
| 50 | Task ↔ Worktree 关联 | ✅ | — |
| 51 | Code Review Workflow | ✅ | — |
| 52 | Code Generation / Modification 建议 | ✅ | — |
| 53 | Code Explanation | ✅ | — |
| 54 | Test Suggestion | ✅ | — |
| 55 | Change Impact Analysis | ✅ | — |
| 56 | Editor 工作流 | ✅ | — |
| 57 | 用户界面 | ✅ | — |
| 58 | Project-level 开发规范 | ✅ | — |
| 59 | 企业级安全策略 | ✅ | — |
| 60 | Web UI | ✅ | — |

## 2. 边界原则

### 2.1 GitGit 可以提供（但 STAR 应优先）

- 文件读取接口（FS API）
- 文件写入接口
- Commit 内容
- Branch 内容
- Diff（结构化 + 人类可读两种）
- Blame
- History
- Rename Detection
- Merge Base
- Conflict Marker
- Repository Status
- File Change Events
- Repository Watch
- Object Streaming
- Partial Clone
- Sparse Checkout
- Large File Support
- Repository Snapshot
- Commit Graph
- File-level History

### 2.2 GitGit 提供"代码智能底座"（如未来需要）

- 文件变更事件
- Commit 级别代码变化
- 文件路径索引
- 基础文本搜索（**非**AST / 非 semantic）
- Diff 计算
- 文件历史
- Repository Snapshot
- Object-level Metadata

### 2.3 仍必须放 STAR

- AST / Symbol / Type / Call Graph / Dependency Graph / Semantic Search
- Code Embedding / RAG / Context Graph
- Task-aware Retrieval
- AI 决策相关
- 用户可见的工作流

## 3. 违反案例库

| 案例 | 描述 | 处置 |
|---|---|---|
| **V-001** | "GitGit 暴露 `gitgit issue list`" | 立刻拒；Issue 是 STAR 责任，GitGit 不应有 issue 概念 |
| **V-002** | "GitGit 暴露 `gitgit ai-review`" | 立刻拒；AI 决策是 STAR 责任 |
| **V-003** | "GitGit 暴露 `gitgit context get`" | 立刻拒；Context 是 STAR 责任 |
| **V-004** | "GitGit 增加 Web UI" | 立刻拒；UI 是 STAR 责任（GitGit 提供 CLI + HTTP API） |
| **V-005** | "GitGit 增加 Lark / Slack 通知" | 立刻拒；Notification 是 STAR 责任，GitGit 只发 Git 原生事件 |

## 4. 灰色地带的处置原则

| 场景 | 判定 |
|---|---|
| "GitGit 提供 CI 跑测试" | 拒。CI 是 STAR 责任。GitGit 只提供 WebHook 通知 CI 系统。 |
| "GitGit 暴露 PR 评审 UI" | 拒。MR 流程是 STAR 责任。GitGit 只暴露底层 commit/branch 状态。 |
| "GitGit 内置 JWT 认证" | 拒。Auth 边界是 STAR 责任。GitGit 用 SSH key + PAT 这种 Git 原生凭证。 |
| "GitGit 暴露 '我的待办' API" | 拒。这是 STAR domain-work-item。 |
| "GitGit 提供 'LSP server' 暴露 commit-level 代码智能" | **可**。这是 GitGit "代码智能底座"范围（per §8.3 任务原文），但完整 AST/Symbol/Type 必须放 STAR。 |

## 5. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版（60 行责任正交表 + 边界原则） | Phase B 起草 |
