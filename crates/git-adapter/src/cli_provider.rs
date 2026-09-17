//! `cli_provider.rs` — `CliGitProvider` (CLI fallback, per DD §10 CLI fallback 段)
//!
//! Per ULYS-57.1 T2.3 (WORKTREE-CANVAS-IMPL-PLAN-001 §4.2):
//! - 14 方法全部实现, 通过 `git` CLI 命令 fallback
//! - 阶段 1 简化: 核心 4 方法 (open_repo / list_worktrees / status / head) 走 CLI,
//!   其他方法返回 NotImplemented (阶段 2 实装)
//! - NFR-REL-001: Git Source of Truth 不可变 (read-only 命令 + write 命令显式 commit)
//!
//! 设计要点:
//! - 用 tokio::process::Command 异步调 git CLI
//! - stdout 解析成结构体 (WorktreeInfo / / StatusEntry / / BranchInfo)
//! - 失败映射到 GitError 6-field

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::error::GitError;
use crate::provider::{
    BranchInfo, ConflictInfo, Diff, DiffFile, GitProvider, MergeResult, MergeStrategy, RepoHandle,
    StatusEntry, StatusKind, SyncResult, WorktreeInfo,
};

/// CLI git provider (fallback)
#[derive(Debug, Clone)]
pub struct CliGitProvider {
    /// git 二进制路径 (e.g. "/usr/bin/git" or "C:/Program Files/Git/bin/git.exe")
    pub git_path: PathBuf,
}

impl CliGitProvider {
    /// Construct with default git path lookup
    pub fn new() -> Self {
        let git_path = std::env::var("GIT_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("git"));
        Self { git_path }
    }

    /// Construct with explicit git path
    pub fn with_path(path: PathBuf) -> Self {
        Self { git_path: path }
    }

    /// 异步跑 git 命令
    async fn run(&self, args: &[&str], cwd: &Path) -> Result<String, GitError> {
        let mut cmd = tokio::process::Command::new(&self.git_path);
        cmd.args(args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            GitError::cli_not_found(self.git_path.to_str().unwrap_or("?")).with_source(e)
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(
                GitError::new("GIT.CLI_FAIL", format!("git {:?} failed", args)).with_source(stderr),
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 解析 git status --porcelain 一行
    fn parse_status_line(line: &str) -> Option<StatusEntry> {
        if line.len() < 4 {
            return None;
        }
        let xy = &line[..2];
        let path = line[3..].trim().to_string();
        let kind = match xy {
            // 工作区 modified (X=' ', Y=M/A/D/...)
            " M" | "MM" | "MD" => StatusKind::Modified,
            // staged (X=M/A/D/R/C/Y/I/S, Y=' ')
            "M " | "A " | "AM" | "AD" | "R " | "RC" | "D " => StatusKind::Staged,
            // untracked
            "??" => StatusKind::Untracked,
            // conflict
            "UU" | "AA" | "DD" | "UA" | "AU" | "UD" | "DU" => StatusKind::Conflicting,
            _ => return None,
        };
        Some(StatusEntry {
            path: PathBuf::from(path),
            kind,
        })
    }
}

impl Default for CliGitProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GitProvider for CliGitProvider {
    #[track_caller]
    async fn open_repo(&self, path: &Path) -> Result<RepoHandle, GitError> {
        let toplevel = self.run(&["rev-parse", "--show-toplevel"], path).await?;
        let trimmed = toplevel.trim();
        if trimmed.is_empty() {
            return Err(GitError::repo_not_found(path.to_str().unwrap_or("?")));
        }
        Ok(RepoHandle {
            path: PathBuf::from(trimmed),
            repo_id: None,
        })
    }

    #[track_caller]
    async fn list_worktrees(&self, repo: &RepoHandle) -> Result<Vec<WorktreeInfo>, GitError> {
        let out = self
            .run(&["worktree", "list", "--porcelain"], &repo.path)
            .await?;
        let mut worktrees = Vec::new();
        let mut cur: Option<WorktreeInfo> = None;
        for line in out.lines() {
            if let Some(rest) = line.strip_prefix("worktree ") {
                if let Some(c) = cur.take() {
                    worktrees.push(c);
                }
                cur = Some(WorktreeInfo {
                    path: PathBuf::from(rest),
                    name: rest.rsplit('/').next().unwrap_or(rest).to_string(),
                    branch: String::new(),
                    head: String::new(),
                    ahead: 0,
                    behind: 0,
                    dirty: false,
                    last_commit_at: None,
                });
            } else if let Some(rest) = line.strip_prefix("HEAD ") {
                if let Some(c) = cur.as_mut() {
                    c.head = rest.to_string();
                }
            } else if let Some(rest) = line.strip_prefix("branch ") {
                // strip refs/heads/ prefix
                let short = rest.strip_prefix("refs/heads/").unwrap_or(rest);
                if let Some(c) = cur.as_mut() {
                    c.branch = short.to_string();
                }
            }
        }
        if let Some(c) = cur.take() {
            worktrees.push(c);
        }
        Ok(worktrees)
    }

    #[track_caller]
    async fn create_worktree(
        &self,
        repo: &RepoHandle,
        branch: &str,
        path: &Path,
        base_branch: Option<&str>,
    ) -> Result<WorktreeInfo, GitError> {
        let mut args: Vec<String> = vec!["worktree".into(), "add".into()];
        let base_s;
        if let Some(b) = base_branch {
            args.push("-b".into());
            args.push(branch.to_string());
            base_s = b.to_string();
            args.push(base_s);
        } else {
            args.push(branch.to_string());
        }
        let path_s = path.to_string_lossy().to_string();
        args.push(path_s);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(&arg_refs, &repo.path).await?;
        Ok(WorktreeInfo {
            path: path.to_path_buf(),
            name: branch.to_string(),
            branch: branch.to_string(),
            head: String::new(),
            ahead: 0,
            behind: 0,
            dirty: false,
            last_commit_at: Some(Utc::now()),
        })
    }

    #[track_caller]
    async fn remove_worktree(
        &self,
        repo: &RepoHandle,
        path: &Path,
        force: bool,
    ) -> Result<(), GitError> {
        let mut args: Vec<String> = vec!["worktree".into(), "remove".into()];
        if force {
            args.push("--force".into());
        }
        let path_s = path.to_string_lossy().to_string();
        args.push(path_s);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(&arg_refs, &repo.path).await?;
        Ok(())
    }

    #[track_caller]
    async fn diff(&self, repo: &RepoHandle, from: &str, to: &str) -> Result<Diff, GitError> {
        let out = self
            .run(&["diff", "--numstat", from, to], &repo.path)
            .await?;
        let mut files = Vec::new();
        let mut total_added: u32 = 0;
        let mut total_removed: u32 = 0;
        for line in out.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }
            let added: u32 = parts[0].parse().unwrap_or(0);
            let removed: u32 = parts[1].parse().unwrap_or(0);
            let path = PathBuf::from(parts[2]);
            total_added += added;
            total_removed += removed;
            files.push(DiffFile {
                path,
                added_lines: vec![],
                removed_lines: vec![],
                hunks: vec![],
            });
        }
        Ok(Diff {
            files,
            total_added,
            total_removed,
        })
    }

    #[track_caller]
    async fn merge_base(&self, repo: &RepoHandle, a: &str, b: &str) -> Result<String, GitError> {
        let out = self.run(&["merge-base", a, b], &repo.path).await?;
        Ok(out.trim().to_string())
    }

    #[track_caller]
    async fn rev_list_count(&self, repo: &RepoHandle, range: &str) -> Result<u32, GitError> {
        let out = self
            .run(&["rev-list", "--count", range], &repo.path)
            .await?;
        let n: u32 = out.trim().parse().map_err(|e| {
            GitError::new("GIT.REV_LIST_PARSE", "rev-list count parse").with_source(e)
        })?;
        Ok(n)
    }

    #[track_caller]
    async fn status(&self, repo: &RepoHandle) -> Result<Vec<StatusEntry>, GitError> {
        let out = self.run(&["status", "--porcelain"], &repo.path).await?;
        let mut entries = Vec::new();
        for line in out.lines() {
            if let Some(e) = Self::parse_status_line(line) {
                entries.push(e);
            }
        }
        Ok(entries)
    }

    #[track_caller]
    async fn sync_main(
        &self,
        _repo: &RepoHandle,
        worktree_path: &Path,
    ) -> Result<SyncResult, GitError> {
        // fetch origin
        let fetch_out = self.run(&["fetch", "origin"], worktree_path).await?;
        let _ = fetch_out;
        // rebase onto origin/main
        let rebase_args = ["rebase", "origin/main"];
        let rebase_out = self.run(&rebase_args, worktree_path).await?;
        Ok(SyncResult {
            fetched_commits: 0,
            rebased_commits: 0,
            conflicts: Vec::new(),
        })
        .map(|s| {
            let _ = rebase_out;
            s
        })
    }

    #[track_caller]
    async fn rebase(
        &self,
        repo: &RepoHandle,
        worktree_path: &Path,
        target: &str,
    ) -> Result<(), GitError> {
        let _ = repo;
        self.run(&["rebase", target], worktree_path).await?;
        Ok(())
    }

    #[track_caller]
    async fn merge(
        &self,
        repo: &RepoHandle,
        worktree_path: &Path,
        branch: &str,
        strategy: MergeStrategy,
    ) -> Result<MergeResult, GitError> {
        let _ = repo;
        let strategy_arg: &str = match strategy {
            MergeStrategy::Merge => "--no-ff",
            MergeStrategy::Squash => "--squash",
            MergeStrategy::Rebase => "--ff-only",
            MergeStrategy::FastForward => "--ff-only",
        };
        let result = self
            .run(&["merge", strategy_arg, branch], worktree_path)
            .await;
        match result {
            Ok(stdout) => {
                let merge_commit = stdout.lines().last().map(|s| s.to_string());
                Ok(MergeResult {
                    success: true,
                    merge_commit,
                    conflicts: vec![],
                    strategy_used: strategy,
                })
            }
            Err(e) => {
                // 可能是 conflict; 检测 conflict 标记
                let err_str = e.to_string();
                if err_str.contains("CONFLICT") {
                    Ok(MergeResult {
                        success: false,
                        merge_commit: None,
                        conflicts: vec![ConflictInfo {
                            path: branch.to_string(),
                            error: err_str,
                        }],
                        strategy_used: strategy,
                    })
                } else {
                    Err(e)
                }
            }
        }
    }

    #[track_caller]
    async fn head(&self, _repo: &RepoHandle, worktree_path: &Path) -> Result<String, GitError> {
        let out = self.run(&["rev-parse", "HEAD"], worktree_path).await?;
        Ok(out.trim().to_string())
    }

    #[track_caller]
    async fn last_commit_at(
        &self,
        _repo: &RepoHandle,
        worktree_path: &Path,
    ) -> Result<DateTime<Utc>, GitError> {
        let out = self
            .run(&["log", "-1", "--format=%ct"], worktree_path)
            .await?;
        let ts: i64 = out.trim().parse().map_err(|e| {
            GitError::new("GIT.TIMESTAMP_PARSE", "last commit timestamp parse").with_source(e)
        })?;
        DateTime::from_timestamp(ts, 0).ok_or_else(|| {
            GitError::new("GIT.TIMESTAMP_RANGE", "last commit timestamp out of range")
        })
    }

    #[track_caller]
    async fn list_branches(&self, repo: &RepoHandle) -> Result<Vec<BranchInfo>, GitError> {
        let out = self
            .run(
                &[
                    "for-each-ref",
                    "--format=%(refname:short) %(objectname)",
                    "refs/heads/",
                ],
                &repo.path,
            )
            .await?;
        let mut branches = Vec::new();
        for line in out.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let name = parts[0].to_string();
            let head_sha = parts[1].to_string();
            let is_mainline = matches!(name.as_str(), "main" | "master");
            // updated_at 走 last_commit_at
            let updated_at = self
                .last_commit_at(repo, &repo.path)
                .await
                .unwrap_or_else(|_| Utc::now());
            branches.push(BranchInfo {
                name,
                head_sha,
                is_mainline,
                updated_at,
            });
        }
        Ok(branches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_line_classifies_modified() {
        let line = " M src/lib.rs";
        let e = CliGitProvider::parse_status_line(line).unwrap();
        assert_eq!(e.kind, StatusKind::Modified);
    }

    #[test]
    fn parse_status_line_classifies_staged() {
        let line = "M  src/lib.rs";
        let e = CliGitProvider::parse_status_line(line).unwrap();
        assert_eq!(e.kind, StatusKind::Staged);
    }

    #[test]
    fn parse_status_line_classifies_untracked() {
        let line = "?? new_file.rs";
        let e = CliGitProvider::parse_status_line(line).unwrap();
        assert_eq!(e.kind, StatusKind::Untracked);
    }

    #[test]
    fn parse_status_line_classifies_conflicting() {
        let line = "UU both_changed.txt";
        let e = CliGitProvider::parse_status_line(line).unwrap();
        assert_eq!(e.kind, StatusKind::Conflicting);
    }

    #[test]
    fn parse_status_line_rejects_short_line() {
        assert!(CliGitProvider::parse_status_line("").is_none());
        assert!(CliGitProvider::parse_status_line("XY").is_none());
    }

    #[test]
    fn parse_status_line_handles_renames_with_arrow() {
        let line = "R  old.txt -> new.txt";
        let e = CliGitProvider::parse_status_line(line).unwrap();
        assert_eq!(e.kind, StatusKind::Staged);
    }
}
