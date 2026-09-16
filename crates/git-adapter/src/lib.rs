//! `git-adapter` — AI Worktree Graph Canvas Git Provider (ULYS-57.1 T2)
//!
//! Per ULYS-57.1 T2.1 + T2.2 + T2.3 (WORKTREE-CANVAS-IMPL-PLAN-001 §4.2):
//! - `GitProvider` Trait + 14 方法 (per DD-WORKTREE-CANVAS-001 §10 + D-GIT-001)
//! - `Libgit2Provider` (推荐, per D-GIT-001) — Merge / Rebase / Sync 全实现
//! - `CliGitProvider` (CLI fallback, per DD §10 CLI fallback 段)
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证
//! - #6 v2 GitError 6-field (code / message / source / location / context / trace_id)
//! - #7 `unsafe_code = "forbid"`
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]
//! - #19 v19 不动 V0.1 任何业务 logic (本 crate 新建, 不改既有)
//! - NFR-REL-001: Git Source of Truth 不可变 (libgit2 read-only 默认 + CLI 调用 git 命令)

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]

pub mod cli_provider;
pub mod error;
pub mod libgit2_provider;
pub mod provider;

pub use cli_provider::CliGitProvider;
pub use error::GitError;
pub use libgit2_provider::Libgit2Provider;
pub use provider::{
    ConflictInfo, Diff, DiffFile, DiffHunk, GitProvider, MergeResult, RepoHandle, StatusEntry,
    StatusKind, SyncResult, WorktreeInfo,
};
