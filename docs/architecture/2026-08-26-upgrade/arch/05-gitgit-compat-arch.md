# 05. GitGit Compatibility Architecture

> **状态**：🟡 草案 v0.3
> **日期**：2026-08-26
> **依赖**：[GitGit IDE Boundary Spec](../../../responsibility-matrix/gitgit-ide-boundary.md) · [ADR-0022 IDE Placement](../adr/0022-ide-placement.md)

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

per [gitgit-ide-boundary.md §5.1](../../../responsibility-matrix/gitgit-ide-boundary.md)：

**MVP 12 endpoints 子集边界**（per P1-J 修复 2026-08-27）：MVP 退出条件 acceptance/04 §3 第四条 = "REST API 12 endpoints"。完整 14 = 12 MVP + 2 扩展（`POST /api/v1/repos` + `POST /api/v1/repos/{owner}/{name}/hooks`）。**每端点补 4xx / 5xx error response 块**（per P1-5 / F-19 / INTERFACE-REVIEW-C P1-5 + INTERFACE-REVIEW-A 🟡 #19 修复 2026-08-27）：

| Endpoint | 用途 | 4xx / 5xx 响应 |
|---|---|---|
| `GET /api/v1/repos` | 列出仓库 | 500 → `Error` (INTERNAL) |
| `GET /api/v1/repos/{owner}/{name}` | 仓库详情 | 404 → `Error` (REPO_NOT_FOUND) / 500 → `Error` (INTERNAL) |
| `GET /api/v1/repos/{owner}/{name}/commits` | Commit 列表 | 404 → `Error` (REPO_NOT_FOUND) / 500 → `Error` (INTERNAL) |
| `GET /api/v1/repos/{owner}/{name}/branches` | Branch 列表 | 404 → `Error` (REPO_NOT_FOUND) / 500 → `Error` (INTERNAL) |
| `GET /api/v1/repos/{owner}/{name}/tags` | Tag 列表 | 404 → `Error` (REPO_NOT_FOUND) / 500 → `Error` (INTERNAL) |
| `GET /api/v1/repos/{owner}/{name}/tree/{sha}` | Tree 对象 | 400 → `Error` (INVALID_SHA) / 404 → `Error` (OBJECT_NOT_FOUND) / 500 → `Error` (INTERNAL) |
| `GET /api/v1/repos/{owner}/{name}/blob/{sha}` | Blob 对象 | 400 → `Error` (INVALID_SHA) / 404 → `Error` (OBJECT_NOT_FOUND) / 500 → `Error` (INTERNAL) |
| `GET /api/v1/repos/{owner}/{name}/blame/{path}` | Blame | 404 → `Error` (FILE_NOT_FOUND) / 500 → `Error` (INTERNAL) |
| `GET /api/v1/repos/{owner}/{name}/diff` | Diff | 400 → `Error` (INVALID_RANGE) / 500 → `Error` (INTERNAL) |
| `GET /api/v1/repos/{owner}/{name}/worktrees` | Worktree 列表 | 404 → `Error` (REPO_NOT_FOUND) / 500 → `Error` (INTERNAL) |
| `POST /api/v1/repos/{owner}/{name}/worktrees` | 创建 worktree | 400 → `Error` (VALIDATION_FAILED) / 404 → `Error` (REPO_NOT_FOUND) / 409 → `Error` (WORKTREE_CONFLICT) / 500 → `Error` (INTERNAL) |
| `DELETE /api/v1/repos/{owner}/{name}/worktrees/{id}` | 删除 worktree | 404 → `Error` (WORKTREE_NOT_FOUND) / 409 → `Error` (WORKTREE_DIRTY) / 500 → `Error` (INTERNAL) |
| `POST /api/v1/repos` *(Phase 2+ 扩展)* | 创建仓库 | 400 → `Error` (VALIDATION_FAILED) / 403 → `Error` (PERMISSION_DENIED) / 409 → `Error` (REPO_EXISTS) / 500 → `Error` (INTERNAL) |
| `POST /api/v1/repos/{owner}/{name}/hooks` *(Phase 2+ 扩展)* | 注册 webhook | 400 → `Error` (VALIDATION_FAILED) / 404 → `Error` (REPO_NOT_FOUND) / 500 → `Error` (INTERNAL) |

**error response 块**（OpenAPI 3.1 `responses` 块必须为每个端点显式列）：

```yaml
responses:
  '4xx':
    description: "Client error"
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/Error'  # = agent-api/v1#Error 6 字段
  '5xx':
    description: "Server error"
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/Error'
```

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-C P1-5 + INTERFACE-REVIEW-A 🟡 #19（F-19）每端点补 4xx/5xx error response 块（含具体 error code）+ OpenAPI 3.1 `responses` 块示例。完整 14 = 12 MVP + 2 扩展（per P1-J 数字已对齐 gitgit-ide-boundary §5.1）。

**关键约束**：
- 所有 endpoint 表达"标准 Git 仓库对象"，**不表达** Issue / PR / Project / Agent / Context / CI
- OpenAPI 3.1 规范完整对齐 JSON Schema 2020-12（per [spec/rest/01-rest-strategy.md §1](../spec/rest/01-rest-strategy.md)）
- 4xx / 5xx 响应**统一**引用 `agent-api/v1#Error`（per P1-G 修复 2026-08-27，per [spec/agent-api/01-schema.md §3.15](../spec/agent-api/01-schema.md)）

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

**GitGit 物理层事件命名空间**（per B-17 / INTERFACE-REVIEW-B 🟡 #17 修复 2026-08-27）：以上事件从 GitGit 物理层发出时**显式带 `.gitgit` 命名空间后缀**，避免与 STAR 业务层重名事件（per [arch/03 §8](03-star-ai-compat-arch.md)）混淆：

| GitGit 物理层事件 | 命名空间后缀形式 | 触发时机 |
|---|---|---|
| `RepositoryCreated` | `RepositoryCreated.gitgit` | `git init` / 仓库首次创建 |
| `CommitCreated` | `CommitCreated.gitgit` | `git commit` 成功 |
| `BranchCreated` | `BranchCreated.gitgit` | `git branch <name>` 成功 |
| `BranchDeleted` | `BranchDeleted.gitgit` | `git branch -d <name>` 成功 |
| `TagCreated` | `TagCreated.gitgit` | `git tag <name>` 成功 |
| `TagDeleted` | `TagDeleted.gitgit` | `git tag -d <name>` 成功 |
| `RefUpdated` | `RefUpdated.gitgit` | 任何 ref（branch / tag）更新 |
| `WorktreeCreated` | `WorktreeCreated.gitgit` | `git worktree add` 成功 |
| `WorktreeRemoved` | `WorktreeRemoved.gitgit` | `git worktree remove` 成功 |
| `ObjectsReceived` | `ObjectsReceived.gitgit` | git-receive-pack 完成 |
| `ObjectsFetched` | `ObjectsFetched.gitgit` | git-upload-pack 完成 |
| `MergeCompleted` | `MergeCompleted.gitgit` | git merge 退出码 0 |
| `ConflictDetected` | `ConflictDetected.gitgit` | git merge / rebase 退出码非 0 |

> **命名约定**（per B-17 修复 2026-08-27）：GitGit 物理层事件 = `<EventName>.gitgit`；STAR 业务层事件 = `<EventName>.star`（per [arch/03 §8](03-star-ai-compat-arch.md)）。GitGit 自身**只发** `.gitgit` 事件，**不**发 `.star` 事件（跨层职责，per ADR-0022 "IDE 归 STAR" 边界）。
>
> **触发链路**：GitGit 物理层事件触发 → STAR 业务层在 Application Service 内重发业务层事件（如 `WorktreeCreated.gitgit` → `WorktreeCreated.star`），保持 STAR 上层逻辑只看 `.star` 后缀事件。STAR 上层（[spec/flows/08 §1.1](../spec/flows/08-event-model.md) 13 个 STAR Domain Events）**不**含 `.gitgit` 后缀（隐式 `.star` 默认）。

**STAR 在上层把这些事件转译为软件工程领域事件**（per [arch/03](03-star-ai-compat-arch.md)）。

> v0.3 fix: 2026-08-27 per INTERFACE-REVIEW-B 🟡 #17（B-17）GitGit 物理层事件显式带 `.gitgit` 命名空间后缀（与 arch/03 §8 STAR `.star` 命名空间对照）。

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

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：5 节（标准 Git 命令 + 智能 HTTP + SSH + REST + 事件 + 守门测试） | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-J：§5 加 MVP 12 endpoints 子集边界（12 MVP + 2 扩展 = 14）+ 4xx/5xx 错误引用 `agent-api/v1#Error`（per P1-G） | 8 子代理 INTERFACE-REVIEW-C P1-5 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.3 | 2026-08-27 | Mavis（接手 agent per DEC-008）| P1-5/F-19：§5 14 端点每端点补 4xx/5xx error response 块（含具体 error code）+ OpenAPI 3.1 `responses` 块示例 · B-17：§6 GitGit 物理层事件显式带 `.gitgit` 命名空间后缀（跟 arch/03 §8 STAR `.star` 命名空间同步） | INTERFACE-REVIEW-A 🟡 #19 + INTERFACE-REVIEW-B 🟡 #17 + INTERFACE-REVIEW-C P1-5 |
