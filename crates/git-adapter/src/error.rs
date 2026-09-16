//! `error.rs` — `GitError` 6-field (per 守门 #6 v2 + DD §10)
//!
//! 6 字段: code / message / source / location / context / trace_id
//!
//! 错误码列表 (per DD §10 + §38 错误码):
//! - GIT.REPO_NOT_FOUND        : open_repo 找不到 path
//! - GIT.WORKTREE_NOT_FOUND    : worktree 不存在
//! - GIT.WORKTREE_CREATE_FAIL  : create_worktree 失败 (branch 已存在 / path 占用)
//! - GIT.WORKTREE_REMOVE_FAIL  : remove_worktree 失败 (有未提交修改 + !force)
//! - GIT.MERGE_CONFLICT        : merge 失败 (有 conflict)
//! - GIT.MERGE_FAILED          : merge 异常 (per DD §38)
//! - GIT.REBASE_CONFLICT       : rebase 冲突
//! - GIT.REBASE_FAILED         : rebase 异常
//! - GIT.SYNC_REMOTE_FAIL      : fetch origin 失败
//! - GIT.STATUS_PARSE_FAIL     : porcelain 解析失败
//! - GIT.IO_FAIL              : 文件读写失败 (CLI fallback)
//! - GIT.CLI_NOT_FOUND         : git CLI 不在 PATH
//! - GIT.NETWORK_FAIL          : 网络错误 (fetch / clone)
//! - GIT.PERMISSION_DENIED     : SSH key / token 鉴权失败

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// 6-field GitError (per 守门 #6 v2 + DD §10 + §38)
///
/// 错误码 (e.g. `GIT.MERGE_CONFLICT`), 字段顺序固定: code / message / source /
/// location / context / trace_id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitError {
    /// 错误码 (e.g. "GIT.MERGE_CONFLICT")
    pub code: String,
    /// 人类可读消息
    pub message: String,
    /// 底层错误源描述 (保留 error chain 追溯)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 现场位置 (file:line)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// KV context (per 守门 #13 d audit context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Trace ID (关联 SSE / REST 请求)
    pub trace_id: String,
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (at {}, trace={})",
            self.code,
            self.message,
            self.location.as_deref().unwrap_or("?"),
            self.trace_id
        )?;
        if let Some(src) = &self.source {
            write!(f, "\n  cause: {src}")?;
        }
        Ok(())
    }
}

impl StdError for GitError {}

impl GitError {
    /// 构造一个新 GitError
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source: None,
            location: None,
            context: None,
            trace_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// 设置 source 字段
    #[must_use]
    pub fn with_source(mut self, source: impl fmt::Display) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// 设置 location 字段
    #[must_use]
    pub fn at(mut self, file: &str, line: u32) -> Self {
        self.location = Some(format!("{file}:{line}"));
        self
    }

    /// 设置 context KV
    #[must_use]
    pub fn with_context(mut self, ctx: serde_json::Value) -> Self {
        self.context = Some(ctx);
        self
    }

    /// 设置 trace_id
    #[must_use]
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = trace_id.into();
        self
    }

    /// 工厂: RepoNotFound
    pub fn repo_not_found(path: &str) -> Self {
        Self::new(
            "GIT.REPO_NOT_FOUND",
            format!("repository not found at {path}"),
        )
    }

    /// 工厂: WorktreeNotFound
    pub fn worktree_not_found(path: &str) -> Self {
        Self::new(
            "GIT.WORKTREE_NOT_FOUND",
            format!("worktree not found at {path}"),
        )
    }

    /// 工厂: MergeConflict (per DD §38)
    pub fn merge_conflict(conflicts: Vec<String>) -> Self {
        Self::new(
            "GIT.MERGE_CONFLICT",
            format!("merge failed with {} conflict(s)", conflicts.len()),
        )
        .with_context(serde_json::json!({ "conflicts": conflicts }))
    }

    /// 工厂: MergeFailed
    pub fn merge_failed(e: impl fmt::Display) -> Self {
        Self::new("GIT.MERGE_FAILED", "merge failed").with_source(e)
    }

    /// 工厂: RebaseConflict
    pub fn rebase_conflict(e: impl fmt::Display) -> Self {
        Self::new("GIT.REBASE_CONFLICT", "rebase conflict").with_source(e)
    }

    /// 工厂: RebaseFailed
    pub fn rebase_failed(e: impl fmt::Display) -> Self {
        Self::new("GIT.REBASE_FAILED", "rebase failed").with_source(e)
    }

    /// 工厂: SyncRemoteFail
    pub fn sync_remote_fail(e: impl fmt::Display) -> Self {
        Self::new("GIT.SYNC_REMOTE_FAIL", "sync remote failed").with_source(e)
    }

    /// 工厂: StatusParseFail
    pub fn status_parse_fail(line: &str) -> Self {
        Self::new(
            "GIT.STATUS_PARSE_FAIL",
            format!("failed to parse git status porcelain line: {line}"),
        )
    }

    /// 工厂: IoFail
    pub fn io_fail(path: &str, e: impl fmt::Display) -> Self {
        Self::new("GIT.IO_FAIL", format!("IO failed at {path}")).with_source(e)
    }

    /// 工厂: CliNotFound
    pub fn cli_not_found(git_path: &str) -> Self {
        Self::new(
            "GIT.CLI_NOT_FOUND",
            format!("git CLI not found at {git_path}"),
        )
    }

    /// 工厂: NetworkFail
    pub fn network_fail(e: impl fmt::Display) -> Self {
        Self::new("GIT.NETWORK_FAIL", "network operation failed").with_source(e)
    }
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        // 把 libgit2 错误映射到 GitError
        let code = match e.code() {
            git2::ErrorCode::NotFound => "GIT.NOT_FOUND",
            git2::ErrorCode::Exists => "GIT.ALREADY_EXISTS",
            git2::ErrorCode::Ambiguous => "GIT.AMBIGUOUS",
            git2::ErrorCode::Conflict => "GIT.REPO_LOCKED",
            git2::ErrorCode::Locked => "GIT.LOCKED",
            git2::ErrorCode::Uncommitted => "GIT.DIRTY_WORKTREE",
            _ => "GIT.LIBGIT2",
        };
        GitError::new(code, e.message()).with_source(e)
    }
}

impl From<std::io::Error> for GitError {
    fn from(e: std::io::Error) -> Self {
        GitError::io_fail("?", e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_six_fields() {
        let e = GitError::new("X.Y", "msg")
            .with_source("inner")
            .at("file.rs", 42)
            .with_context(serde_json::json!({"a": 1}))
            .with_trace_id("trace-1");
        assert_eq!(e.code, "X.Y");
        assert_eq!(e.message, "msg");
        assert_eq!(e.source.as_deref(), Some("inner"));
        assert_eq!(e.location.as_deref(), Some("file.rs:42"));
        assert_eq!(e.trace_id, "trace-1");
        assert!(e.context.is_some());
    }

    #[test]
    fn factories_set_codes() {
        let e = GitError::repo_not_found("/tmp/x");
        assert_eq!(e.code, "GIT.REPO_NOT_FOUND");
        let e = GitError::worktree_not_found("/tmp/wt");
        assert_eq!(e.code, "GIT.WORKTREE_NOT_FOUND");
        let e = GitError::merge_conflict(vec!["a.rs".into()]);
        assert_eq!(e.code, "GIT.MERGE_CONFLICT");
        let e = GitError::merge_failed("boom");
        assert_eq!(e.code, "GIT.MERGE_FAILED");
        let e = GitError::rebase_conflict("x");
        assert_eq!(e.code, "GIT.REBASE_CONFLICT");
        let e = GitError::cli_not_found("/usr/bin/git");
        assert_eq!(e.code, "GIT.CLI_NOT_FOUND");
        let e = GitError::network_fail("dns");
        assert_eq!(e.code, "GIT.NETWORK_FAIL");
    }

    #[test]
    fn libgit2_error_maps_to_git_error() {
        // 触发一个真实 git2 错误: 打开不存在的 repo
        let r = git2::Repository::open("/nonexistent/path/foo");
        assert!(r.is_err());
        let ge: GitError = r.err().unwrap().into();
        assert_eq!(ge.code, "GIT.NOT_FOUND");
    }
}
