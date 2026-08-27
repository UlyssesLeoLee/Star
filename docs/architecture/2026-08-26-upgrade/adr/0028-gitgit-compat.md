# ADR-0028: GitGit 兼容性架构

> **状态**：🟡 Draft v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签（per §6 签字栏）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)（待归档）
> **依赖**：[ADR-0022 IDE Placement](0022-ide-placement.md) · [ADR-0023 Version Control Provider](0023-version-control-provider.md) · [GitGit IDE Boundary Spec](../responsibility-matrix/gitgit-ide-boundary.md)
> **关联**：[arch/05 GitGit Compat Arch](../architecture/2026-08-26-upgrade/arch/05-gitgit-compat-arch.md)

---

## 1. 背景与问题

GitGit 在 2026-08-26 之前是 single-crate MVP（per arch/01 §1.1）：

- 已合并 main（5007883 + 1da5f2c）：single-crate gitgit MVP
- 14-crate 设计已归档（c822a60）
- 已实现 Smart HTTP 协议（per 1da5f2c commit `fix(server): wire up smart-HTTP`）
- **不包含** Issue / PR / Project / Agent / Context / CI 等任何上层概念
- **不包含** REST API（仅 Git 协议）

当 GitGit 作为 Version Control Provider 之一被纳入 4 Provider 并列抽象（per ADR-0023）时，需要明确：**GitGit 对任何 AI Agent / IDE 来说就是标准 Git**。

## 2. 决策

**采用 "GitGit 100% 兼容标准 Git 协议 + 最小 REST API 仅表达 Git 仓库对象" 的兼容性架构。**

### 2.1 必须兼容的标准 Git 命令（per arch/05 §2）

| 命令 | 必须行为 |
|---|---|
| `git clone` | 通过 smart HTTP / SSH 拉取 |
| `git fetch` / `git pull` / `git push` | 标准 Git 协议 |
| `git commit` | 本地 commit |
| `git log` / `git diff` / `git blame` | 跟标准 Git 100% 一致 |
| `git branch` / `git switch` / `git checkout` | 100% 一致 |
| `git merge` / `git rebase` | 100% 一致（含 conflict 标记） |
| `git tag` | 100% 一致 |
| `git worktree add/remove/list/prune` | 100% 一致 |

**Agent 不需要"gitgit"命令前缀**。

### 2.2 Smart HTTP 协议（per arch/05 §3）

```
GET  /info/refs?service=git-upload-pack
POST /git-upload-pack
GET  /info/refs?service=git-receive-pack
POST /git-receive-pack
```

**与 GitHub / GitLab / Gitea 行为兼容**（这是 VersionControlProvider 抽象的同构前提，per ADR-0023）。

### 2.3 SSH 协议（per arch/05 §4）

- 复用 GitGit 现成 SSH server
- receive-pack 必须 fail-closed（per PLAN-002 M1 提到的 ISS-117 决策）

### 2.4 REST API 子集边界（per arch/05 §5 + P1-J 修复 2026-08-27）

MVP 12 endpoints（必实现）+ 2 扩展（Phase 2+）= 完整 14：

| MVP 12（必实现） | 扩展 2（per Phase 2+） |
|---|---|
| `GET /api/v1/repos` | `POST /api/v1/repos`（创建仓库，Phase 2+） |
| `GET /api/v1/repos/{owner}/{name}` | `POST /api/v1/repos/{owner}/{name}/hooks`（webhook 订阅，Phase 2+） |
| `GET /api/v1/repos/{owner}/{name}/commits` |  |
| `GET /api/v1/repos/{owner}/{name}/branches` |  |
| `GET /api/v1/repos/{owner}/{name}/tags` |  |
| `GET /api/v1/repos/{owner}/{name}/tree/{sha}` |  |
| `GET /api/v1/repos/{owner}/{name}/blob/{sha}` |  |
| `GET /api/v1/repos/{owner}/{name}/blame/{path}` |  |
| `GET /api/v1/repos/{owner}/{name}/diff` |  |
| `GET /api/v1/repos/{owner}/{name}/worktrees` |  |
| `POST /api/v1/repos/{owner}/{name}/worktrees` |  |
| `DELETE /api/v1/repos/{owner}/{name}/worktrees/{id}` |  |

### 2.5 关键架构约束

- 所有 endpoint 表达"标准 Git 仓库对象"，**不表达** Issue / PR / Project / Agent / Context / CI（per arch/05 §5）
- OpenAPI 3.1 规范完整对齐 JSON Schema 2020-12（per spec/rest/01-rest-strategy.md §1）
- 4xx / 5xx 响应**统一**引用 `agent-api/v1#Error`（per P1-G 修复 2026-08-27）
- Git 原生事件（RepositoryCreated / CommitCreated / RefUpdated / WorktreeCreated 等 13 类）由 STAR 上层转译为软件工程领域事件（per arch/03）

### 2.6 不可污染 Core 的守门测试（per arch/05 §7）

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

## 3. 备选方案与拒绝理由

### 备选 A：GitGit 自定义扩展协议（独有 features）
- 拒绝理由：违反 ADR-0023 Version Control Provider 抽象的同构前提；Agent 接入 GitGit 必须 0 适配

### 备选 B：REST API 暴露 Issue / PR / Project / Agent 概念
- 拒绝理由：违反 GitGit IDE Boundary 守门规则（per responsibility-matrix/gitgit-ide-boundary.md）；这些是 STAR 层概念

### 备选 C：fork libgit2 / 自研 Git object 库
- 拒绝理由：沿用 gix 库是 Rust 生态共识（per arch/05 §8）；不重新发明 wheel

## 4. 后果与影响

### 4.1 正面

- 任意 Coding Agent / IDE 立即把 GitGit 当标准 Git 用（per arch/05 §1 目标）
- 4 Provider（GitGit / GitHub / GitLab / Gitea）同构前提成立（per ADR-0023）
- Git Object 存储沿用 gix 库（Rust 生态共识）
- LFS 用 git-lfs 标准（GitHub/GitLab/Gitea 都支持）

### 4.2 负面 / 成本

- REST API 子集严格限制（不能表达 Issue / PR 等 STAR 概念）
- Smart HTTP + SSH 双协议栈需长期维护
- 守门测试 5 类必须全跑过才能合入

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 某款 Git 客户端实现细节不一致 | 中 | 中 | GitHub / GitLab / Gitea 实测通过作为基线 |
| gix 库 API 破坏性变更 | 中 | 中 | 锁版本 + 自有 wrapper 层 |
| Smart HTTP 性能 | 低 | 中 | 已有 axum server，扩展即可 |

## 5. 与其他 ADR 的关系

- **依赖**：[ADR-0022 IDE Placement](0022-ide-placement.md) — GitGit 不做 IDE 概念
- **依赖**：[ADR-0023 Version Control Provider](0023-version-control-provider.md) — 4 Provider 同构前提
- **依赖**：[ADR-0025 Vendor Adapter Anti-Contamination](0025-vendor-adapter-anti-contamination.md) — Core 不允许 vendor 命名空间
- **被依赖**：[ADR-0026 STAR AI Compat](0026-star-ai-compat.md) — Git 兼容性是 L4 兜底层

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
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版：5 节（标准 Git 命令 + Smart HTTP + SSH + REST 12+2 endpoints + 守门测试） | Phase B 起草（per 2026-08-26 升级 Plan） |
