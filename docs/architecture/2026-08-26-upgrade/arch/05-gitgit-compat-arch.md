# 05. GitGit Compatibility Architecture

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **依赖**：[GitGit IDE Boundary Spec](../../responsibility-matrix/gitgit-ide-boundary.md) · [ADR-0022 IDE Placement](../../adr/0022-ide-placement.md)

---

## 1. 目标

GitGit 对任何 AI Agent / IDE 来说**就是标准 Git**。任何具备 Git 能力的工具**自动**会用 GitGit。

## 2. 必须兼容的标准 Git 命令

| 命令 | 必须行为 |
|---|---|
| `git clone` | 通过 smart HTTP / SSH 拉取 |
| `git fetch` | 标准 Git 协议 |
| `git pull` | fetch + merge |
| `git push` | receive-pack |
| `git commit` | 本地 commit |
| `git log` / `git diff` / `git blame` | 跟标准 Git 100% 一致 |
| `git branch` / `git switch` / `git checkout` | 100% 一致 |
| `git merge` / `git rebase` | 100% 一致（含 conflict 标记） |
| `git tag` | 100% 一致 |
| `git worktree add/remove/list/prune` | 100% 一致 |

**Agent 不需要"gitgit"命令前缀**。

## 3. 智能 HTTP 协议实现

```
GET  /info/refs?service=git-upload-pack
POST /git-upload-pack
GET  /info/refs?service=git-receive-pack
POST /git-receive-pack
```

**与 GitHub / GitLab / Gitea 行为兼容**（这是 VersionControlProvider 抽象的同构前提）。

## 4. SSH 协议实现

- 复用 GitGit 现成 SSH server（per 1da5f2c commit `fix(server): wire up smart-HTTP`）
- receive-pack 必须 fail-closed（per PLAN-002 M1 提到的 ISS-117 决策）

## 5. REST API（OpenAPI 3.1）

per [gitgit-ide-boundary.md §5.1](../../responsibility-matrix/gitgit-ide-boundary.md)：

```
GET    /api/v1/repos
POST   /api/v1/repos
GET    /api/v1/repos/{owner}/{name}
GET    /api/v1/repos/{owner}/{name}/commits
GET    /api/v1/repos/{owner}/{name}/branches
GET    /api/v1/repos/{owner}/{name}/tags
GET    /api/v1/repos/{owner}/{name}/tree/{sha}
GET    /api/v1/repos/{owner}/{name}/blob/{sha}
GET    /api/v1/repos/{owner}/{name}/blame/{path}
GET    /api/v1/repos/{owner}/{name}/diff
POST   /api/v1/repos/{owner}/{name}/hooks
GET    /api/v1/repos/{owner}/{name}/worktrees
POST   /api/v1/repos/{owner}/{name}/worktrees
DELETE /api/v1/repos/{owner}/{name}/worktrees/{id}
```

**关键约束**：所有 endpoint 表达"标准 Git 仓库对象"，**不表达** Issue / PR / Project / Agent / Context / CI。

## 6. Git 原生事件

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

**STAR 在上层把这些事件转译为软件工程领域事件**（per [arch/03](03-star-ai-compat-arch.md)）。

## 7. 不可污染 Core 的守门测试

```bash
# 1. 不含 vendor 命名空间
grep -rE "ClaudeAdapter|CodexAdapter|CursorAdapter|CopilotAdapter|VSCodeAdapter|JetBrainsAdapter" src/  # 必须为空

# 2. 不含 IDE 概念
grep -rE '"ide"|"agent"|"task"|"issue"|"context"|"rag"' src/  # 应只在注释或变量名出现

# 3. 标准 Git 兼容测试
git clone http://localhost:8080/owner/repo.git && cd repo && git log --oneline | head -5
# 输出必须 100% 跟 git 协议一致

# 4. Smart HTTP 协议测试
GIT_TRACE=1 git clone http://localhost:8080/owner/repo.git
# 日志必须显示标准 git-upload-pack / receive-pack 交换

# 5. SSH 测试
GIT_SSH_COMMAND="ssh -i test_key" git clone ssh://git@localhost/owner/repo.git
```

## 8. 关键决策

- **Git Object 存储**：沿用 gix 库（per Rust 生态共识；不重新发明 wheel）
- **Smart HTTP 实现**：axum 已有 server，扩展即可
- **LFS**：用 git-lfs 标准（GitHub/GitLab/Gitea 都支持）
- **Worktree**：标准 git worktree（per Git 二进制自带能力）

## 9. 签字栏 / 修订历史

per [arch/01](01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
