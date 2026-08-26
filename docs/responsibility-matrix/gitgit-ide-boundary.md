# GitGit IDE Boundary Specification

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签
> **依赖**：[ADR-0022 IDE Placement](../adr/0022-ide-placement.md) · [STAR vs GitGit Matrix](star-vs-gitgit.md)

---

## 1. 范围

本规范定义 GitGit 面向 IDE / AI Agent / Coding Agent 提供的**接口边界**。这是"GitGit 是否需要知道 IDE"这个核心问题的答案。

## 2. 黄金原则

> **GitGit 对 IDE / AI Agent 来说就是标准 Git。**

任何 AI Agent / IDE 只要会 Git，**自动**会用 GitGit。不需要 `gitgit` 命令前缀，不需要 GitGit 专用 SDK，不需要 GitGit 专用 plugin。

## 3. GitGit 必须支持的标准 Git 命令

```bash
# Clone / Fetch / Pull
git clone
git fetch
git pull

# Commit / Push
git commit
git push

# Branch / Tag
git branch
git switch
git checkout
git tag

# Diff / Log / Blame
git status
git diff
git log
git blame

# Merge / Rebase
git merge
git rebase

# Worktree（GitGit 是 worktree 底层）
git worktree add
git worktree list
git worktree remove
```

**所有命令必须 work 100% 跟标准 Git 兼容**。Agent / IDE 看到的是 `git`，不是 `gitgit`。

## 4. GitGit 暴露的 HTTP API（Git 协议层）

| API | 用途 | 协议 |
|---|---|---|
| `GET /info/refs?service=git-upload-pack` | Git 客户端探测 | git smart HTTP |
| `POST /git-upload-pack` | Fetch / Clone | git smart HTTP |
| `GET /info/refs?service=git-receive-pack` | Push 前探测 | git smart HTTP |
| `POST /git-receive-pack` | Push | git smart HTTP |
| `git-upload-pack` over SSH | SSH 协议 | SSH |
| `git-receive-pack` over SSH | Push over SSH | SSH |
| `GET /repos/{owner}/{name}.git/...` | Git LFS | Git LFS |

**所有端点必须与 GitHub / GitLab / Gitea 行为兼容**。这是 STAR 抽象 `VersionControlProvider` 的实现基础（per ADR-0023）。

## 5. GitGit 可以提供（非 Git 协议，但仍是底层）

> 这些是"GitGit 可以给 IDE / Agent 用的增强能力"，**不破坏标准 Git 兼容**。

### 5.1 仓库级 REST API（OpenAPI 3.1）

```
GET    /api/v1/repos                       # 列出仓库
POST   /api/v1/repos                       # 创建仓库
GET    /api/v1/repos/{owner}/{name}        # 仓库详情
GET    /api/v1/repos/{owner}/{name}/commits
GET    /api/v1/repos/{owner}/{name}/branches
GET    /api/v1/repos/{owner}/{name}/tags
GET    /api/v1/repos/{owner}/{name}/tree/{sha}
GET    /api/v1/repos/{owner}/{name}/blob/{sha}
GET    /api/v1/repos/{owner}/{name}/blame/{path}
GET    /api/v1/repos/{owner}/{name}/diff
POST   /api/v1/repos/{owner}/{name}/hooks  # 注册 webhook
GET    /api/v1/repos/{owner}/{name}/worktrees
POST   /api/v1/repos/{owner}/{name}/worktrees  # worktree add
DELETE /api/v1/repos/{owner}/{name}/worktrees/{id}  # worktree remove
```

> **关键约束**：这些 API 表达的是"标准 Git 仓库对象"，**不表达** Issue / PR / Project / Agent / Context / CI 等上层概念。

### 5.2 文件级元数据

- Repository metadata（name / description / default_branch）
- Commit metadata（oid / author / committer / message / tree / parents）
- Branch metadata（name / commit / protected）
- Tag metadata（name / commit / annotated? / message）
- Tree metadata（path / mode / oid）
- Blob metadata（oid / size）
- Worktree metadata（path / branch / head_commit）

### 5.3 事件流（Git 原生事件）

```
RepositoryCreated
CommitCreated
BranchCreated
BranchDeleted
TagCreated
TagDeleted
RefUpdated
WorktreeCreated
WorktreeRemoved
ObjectsReceived
ObjectsFetched
MergeCompleted
ConflictDetected
```

> **关键约束**：GitGit 发出的事件**只描述 Git 原生对象**。STAR 在上层把这些事件转译为软件工程领域事件（per §34 任务原文）。

## 6. GitGit 不应提供

| 能力 | 替代 / 理由 |
|---|---|
| Issue 面板 | STAR 责任 |
| Task 面板 | STAR 责任 |
| Project Dashboard | STAR 责任 |
| AI Chat | STAR 责任 |
| Prompt 管理 | STAR 责任 |
| Agent Session | STAR 责任 |
| RAG | STAR 责任 |
| Knowledge Graph | STAR 责任 |
| Code Explanation | STAR 责任（GitGit 只提供 raw diff + blame） |
| Code Generation | STAR 责任 |
| Code Review Workflow | STAR 责任 |
| MR 审批流程 | STAR 责任（GitGit 只提供底层 PR 对象） |
| CI/CD 编排 | STAR 责任 |
| Sprint | STAR 责任 |
| Roadmap | STAR 责任 |
| DORA | STAR 责任 |
| 企业权限策略 | STAR 责任 |
| Human Approval（软件工程意义） | STAR 责任（GitGit 只做 SSH key 认证） |
| Agent Lease | STAR 责任 |
| Agent Resume | STAR 责任 |
| 多 Agent 协作 | STAR 责任 |
| AST / Symbol / Type 索引 | STAR 责任（GitGit 只做路径索引） |
| Semantic Search | STAR 责任 |
| Code Embedding | STAR 责任 |
| Context Graph | STAR 责任 |
| Task-aware Retrieval | STAR 责任 |
| IDE 状态对象 | STAR 责任（per ADR-0024） |

## 7. 测试与守门

### 7.1 必跑测试

```bash
# 1. 标准 Git 兼容测试
git clone http://localhost:8080/owner/repo.git
git push origin main
git worktree add ../wt feature-branch
# 所有命令必须 100% 等价于标准 Git 行为

# 2. IDE-无关测试
# 不应有任何 IDE-specific 代码 / schema
grep -rE 'ide|vscode|jetbrains|cursor' src/  # 应只在注释或变量名出现，无逻辑

# 3. Vendor-neutral 测试
grep -rE 'if.*provider.*==.*"claude"|if.*provider.*==.*"codex"' src/  # 应为空
```

### 7.2 PR 守门

任何 PR 修改 GitGit 时必查：
- 是否新增 IDE/AI 相关能力？→ 拒
- 是否破坏标准 Git 兼容？→ 拒
- 是否引入 vendor-specific 逻辑？→ 拒

## 8. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 9. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版 | Phase B 起草 |
