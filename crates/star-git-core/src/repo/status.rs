//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/GitGit (AGPL-3.0) with the author's
//! dual-license permission (MIT OR Apache-2.0) for Star,
//! per 2026-11-14 Ulysses 拍板 ("gitgit 我电脑里就有，直接用").
//! Source: D:\GitGit @ 69770e6
//! ═══════════════════════════════════════════════════════════════════

//! 路径校验 (GitGit src/repo/status.rs 的 validate_rel_path 移植子集)

use crate::error::{Result, StarGitError};

/// 校验仓库相对路径: 非空 / 非绝对 / 不含 `..` (per GitGit ADR-0024 §2.1)
pub fn validate_rel_path(path: &str) -> Result<&str> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(StarGitError::Http("empty path".to_string()));
    }
    if trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return Err(StarGitError::Http(format!(
            "path must be repository-relative: {trimmed}"
        )));
    }
    if trimmed.contains("..") {
        return Err(StarGitError::Http(format!(
            "path must not contain `..`: {trimmed}"
        )));
    }
    Ok(trimmed)
}
