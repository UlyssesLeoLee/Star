# 36. GitHub / GitLab Compatibility

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/vcs/01-version-control-provider.md](01-version-control-provider.md)

## 1. 用户组合（per §36 任务原文）

```
STAR + GitHub
STAR + GitLab
STAR + GitGit
```

GitGit 不强制用户立即迁移。

## 2. GitHub Provider

- 用 GitHub REST API + GraphQL（GitHub 官方）
- 优先用 GitHub MCP Server（如客户环境已部署）
- PAT / OAuth 认证

## 3. GitLab Provider

- 用 GitLab REST API + GraphQL
- PAT / OAuth 认证

## 4. Gitea Provider

- 用 Gitea REST API
- PAT 认证

## 5. 能力差异处理

| 差异 | 处理 |
|---|---|
| GitHub Branch Protection API 有限 | Provider capability 表如实反映 |
| GitLab Merge Request ≠ GitHub Pull Request | 抽象为 "merge_request" |
| Gitea API 略有不同 | per provider 适配 |

## 6. 关键约束

- 用户**不**需要"先迁到 GitGit"才能用 STAR
- 任何 Git Provider 都能被 STAR 接入
- 4 个 Provider 在 UI / CLI / API 上平等

## 7. 实施位置

- `crates/star-vcs/src/github.rs`
- `crates/star-vcs/src/gitlab.rs`
- `crates/star-vcs/src/gitea.rs`
- 每个 provider 必须有完整测试 + 至少 1 个真实 e2e 测试

## 8. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
