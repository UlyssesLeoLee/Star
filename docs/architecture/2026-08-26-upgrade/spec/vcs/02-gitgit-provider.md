# 35. GitGit Provider Implementation

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/vcs/01-version-control-provider.md](01-version-control-provider.md) · [arch/05 GitGit Compat Arch](../../arch/05-gitgit-compat-arch.md)

## 1. GitGit 作为 Provider

GitGit 是 STAR 默认的 Version Control Provider。完整实现 per [spec/vcs/01](../vcs/01-version-control-provider.md) trait。

## 2. 关键约束

- GitGit 必须**完全兼容**标准 Git（per arch/05）
- Agent / IDE 用 GitGit 时用 `git` 命令，不是 `gitgit` 命令
- GitGit 后端是 Rust 实现，per c89f858 之前 main (1da5f2c) 已有 CLI + axum server + smart HTTP

## 3. 当前实现状态（c89f858 之前）

| 模块 | 状态 |
|---|---|
| `src/main.rs` CLI entry | ✅ |
| `src/cli.rs` clap derive | ✅ |
| `src/repo/store.rs` | ✅ |
| `src/repo/refs.rs` | ✅ |
| `src/server/smart.rs` Smart HTTP | ✅ (per 1da5f2c) |
| `src/server/auth.rs` | ✅ |
| `src/server/subprocess.rs` | ✅ |

## 4. Phase D 增量

- 标准化 REST API（per [gitgit-ide-boundary.md §5.1](../../../../responsibility-matrix/gitgit-ide-boundary.md)）
- OpenAPI 3.1 spec 输出
- 完整 Webhook 事件流

## 5. Phase 2+ 增量

- LFS 完整支持
- Repository Mirror
- CODEOWNERS 解析
- 自定义权限边界

## 6. 实施位置

- GitGit 仓库独立演进（在 `D:/GitGit/feature/ide-boundary` 分支）
- STAR 端：`crates/star-vcs/src/gitgit.rs` 调 GitGit REST API

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
