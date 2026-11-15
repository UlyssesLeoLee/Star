//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/GitGit (AGPL-3.0) with the author's
//! dual-license permission (MIT OR Apache-2.0) for Star,
//! per 2026-11-14 Ulysses 拍板 ("gitgit 我电脑里就有，直接用").
//! Source: D:\GitGit @ 69770e6
//! ═══════════════════════════════════════════════════════════════════

//! 精简错误类型 (GitGit src/error.rs 移植子集: conflict 内核用到的变体)

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StarGitError {
    /// `git` CLI 子进程失败
    #[error("git subprocess `{cmd}` failed: {source}")]
    GitSubprocess {
        cmd: String,
        #[source]
        source: std::io::Error,
    },

    /// `git` CLI 非零退出
    #[error("git `{cmd}` exited with {status}: {stderr}")]
    GitExit {
        cmd: String,
        status: i32,
        stderr: String,
    },

    /// HTTP/语义层错误 (与 GitGit 同名保留, 便于 diff 对齐)
    #[error("git error: {0}")]
    Http(String),

    /// 远端/同步层错误 (与 GitGit 对齐, remote_sync 模块用)
    #[error("remote error: {0}")]
    Remote(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, StarGitError>;
