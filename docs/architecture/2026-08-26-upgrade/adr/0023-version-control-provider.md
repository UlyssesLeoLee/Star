# ADR-0023: Version Control Provider Abstraction

> **状态**：🟢 Accepted v0.2 (per 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 守门 #12 docs 触发 + 守门 #10 author=Ulysses)
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签
> **依赖**：[ADR-0022 IDE Placement](0022-ide-placement.md)
> **关联**：[GitHub/GitLab Compatibility Spec](../spec/vcs/03-github-gitlab-compat.md)（起草中）

---

## 1. 背景

当前 GitGit 是 STAR 唯一的版本控制后端。**但用户已经在用 GitHub / GitLab / Gitea**。如果 STAR 强制迁移到 GitGit，会造成巨大用户摩擦。

2026-08-26 调研确认 GitHub / GitLab / Gitea 的 API 都有公开能力（无需 Vendor 为 STAR 适配）：
- GitHub：REST API + GraphQL + GitHub MCP Server（官方）
- GitLab：REST API + GraphQL
- Gitea：REST API

## 2. 决策

**引入 VersionControlProvider 抽象层，GitGit / GitHub / GitLab / Gitea 并列。**

### 2.1 Provider 接口（核心 trait）

```rust
// crates/star-vcs/src/provider.rs
#[async_trait]
pub trait VersionControlProvider: Send + Sync {
    fn name(&self) -> &str;                 // "gitgit" / "github" / "gitlab" / "gitea"
    fn capabilities(&self) -> ProviderCapabilities;
    
    async fn clone(&self, url: &str, dest: &Path) -> Result<Repository>;
    async fn fetch(&self, repo: &Repository) -> Result<()>;
    async fn push(&self, repo: &Repository) -> Result<()>;
    async fn create_pr(&self, repo: &Repository, pr: PullRequestSpec) -> Result<PullRequest>;
    async fn list_repos(&self, owner: &str) -> Result<Vec<RepoInfo>>;
    // ... 更多 method 由接口层定义
}
```

### 2.2 Provider 实现

```
crates/star-vcs/
├── src/
│   ├── provider.rs           (trait)
│   ├── capabilities.rs
│   ├── gitgit.rs             (GitGit provider)
│   ├── github.rs             (GitHub provider)
│   ├── gitlab.rs             (GitLab provider)
│   ├── gitea.rs              (Gitea provider)
│   └── other.rs
```

### 2.3 能力差异

| 能力 | GitGit | GitHub | GitLab | Gitea |
|---|---|---|---|---|
| Repository | ✅ | ✅ | ✅ | ✅ |
| Standard Git 协议 (clone/push/pull) | ✅ | ✅ | ✅ | ✅ |
| 自定义 PR/MR API | ✅ (star mr) | ✅ (REST + MCP) | ✅ (REST) | ✅ (REST) |
| Webhook | ✅ | ✅ | ✅ | ✅ |
| 自定义权限边界 | ✅ | ⚠️ (GitHub 限制) | ✅ | ✅ |
| 自定义事件流 | ✅ | ⚠️ (限制) | ✅ | ✅ |

**关键约束**：Agent / IDE 不应该知道具体 Provider 类型。所有调用走 `star` CLI 或 `star` MCP server。

### 2.4 用户可同时使用

```bash
# 用户用 GitHub 做 source-of-truth
star repo add github:myorg/main https://github.com/myorg/main

# 但底层 Worktree 跑在 GitGit
star worktree create STAR-1024 --provider gitgit
```

## 3. 备选方案与拒绝理由

### 备选 A：只用 GitGit
- 拒绝理由：用户摩擦大、商业可接受度低、失去 GitHub 庞大生态（100M+ 开发者）

### 备选 B：只支持 GitHub
- 拒绝理由：单点依赖、不中立、违反 ADR-0021

### 备选 C：每个 Provider 独立适配，不抽 trait
- 拒绝理由：重复实现、Agent/IDE 需要知道 Provider 名、违反 §37 任务原文

## 4. 后果

### 4.1 正面
- 用户零摩擦迁移
- Provider 平等（GitGit 不会"低人一等"）
- 任何新 Provider 都能加（Bitbucket、SourceHut、self-hosted Gitea 等）

### 4.2 成本
- 抽象层引入复杂度
- 4 个 Provider × N 个 method = 4N 实现 + 测试
- 必须维护"能力差异表"，避免 Agent 误用



---

## 5.5. 实施状态 (per OPT-NEXT-08 v0.2 拍板, 守门 #12 docs 触发)

> **本节由 OPT-NEXT-08 (2026-09-07 14:30 JST) 升版 v0.1 → v0.2 时落地, 提供实施证据链**

| 维度 | 内容 |
|---|---|
| **决策 scope** | VCS Core 归 GitGit |
| **目标 crate / 落地位置** | `n/a (架构原则)` |
| **落地 commit (per `git log -p --follow`)** | 无 commit (原则文档) |
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
| v0.2 | 2026-09-07 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | OPT-NEXT-08 升版 v0.1 → v0.2: 新增 §5.5 实施状态 (scope/crate/commit/守门 0 违反 实证) + 修订历史 v0.2 行 (per 守门 #10 author=Ulysses + 守门 #12 docs 触发 + 守门 #11 缺标比错标) | 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 23 草案 ADR 一次性升版 |
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版 | Phase B 起草 |
