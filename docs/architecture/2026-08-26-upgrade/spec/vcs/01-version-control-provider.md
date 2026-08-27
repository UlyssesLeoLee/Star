# 34. Version Control Provider Abstraction

> **状态**：🟡 草案 v0.1
> **依赖**：[ADR-0023 Version Control Provider](../../adr/0023-version-control-provider.md)

## 1. 4 个 Provider 并列

```
VersionControlProvider (trait)
├── GitGit
├── GitHub
├── GitLab
└── Gitea
```

## 2. 核心 trait

```rust
#[async_trait]
pub trait VersionControlProvider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;
    async fn clone(&self, url: &str, dest: &Path) -> Result<Repository>;
    async fn fetch(&self, repo: &Repository) -> Result<()>;
    async fn push(&self, repo: &Repository) -> Result<()>;
    async fn create_pr(&self, repo: &Repository, pr: PullRequestSpec) -> Result<PullRequest>;
    async fn list_repos(&self, owner: &str) -> Result<Vec<RepoInfo>>;
    async fn list_branches(&self, repo: &Repository) -> Result<Vec<BranchInfo>>;
    async fn get_file(&self, repo: &Repository, path: &str, ref_: &str) -> Result<FileContent>;
    async fn get_commit(&self, repo: &Repository, sha: &str) -> Result<Commit>;
    async fn get_diff(&self, repo: &Repository, base: &str, head: &str) -> Result<Diff>;
    async fn add_webhook(&self, repo: &Repository, url: &str, events: &[&str]) -> Result<WebhookId>;
}
```

## 3. 能力差异表

| 能力 | GitGit | GitHub | GitLab | Gitea |
|---|---|---|---|---|
| Repository CRUD | ✅ | ✅ | ✅ | ✅ |
| 标准 Git 协议 | ✅ | ✅ | ✅ | ✅ |
| 自定义 PR/MR API | ✅ | ✅ | ✅ | ✅ |
| Webhook | ✅ | ✅ | ✅ | ✅ |
| Self-hosted | ✅ | ⚠️ Enterprise | ✅ | ✅ |
| 大文件 LFS | ✅ | ✅ | ✅ | ✅ |
| 自定义权限 | ✅ | ⚠️ | ✅ | ✅ |

## 4. Agent / IDE 不感知

所有调用走 `star` CLI / MCP / REST。Provider 在 Application 层做翻译。

## 5. 实施位置

- `crates/star-vcs/` — Provider abstraction
- `crates/star-vcs/src/provider.rs` — trait
- `crates/star-vcs/src/gitgit.rs` — GitGit 实现
- `crates/star-vcs/src/github.rs` — GitHub 实现
- `crates/star-vcs/src/gitlab.rs` — GitLab 实现
- `crates/star-vcs/src/gitea.rs` — Gitea 实现

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
