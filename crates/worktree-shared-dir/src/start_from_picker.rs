//! `start_from_picker.rs` — ULYS-158.5 / FR-ORCA-009 (Start-from Picker, 4 选 1).
//!
//! Per `docs/ecosystem-survey/orca-design-survey.md` v1.0 §3 FR-ORCA-009 +
//! ULYS-218 (ULYS-158.5) description §1.1:
//!
//! 4 选 1 candidates:
//! - `#1` RepoBaseRef        : 仓库默认 base ref (origin/main or origin/master)
//! - `#2` LocalBranch(String): 另一个本地分支
//! - `#3` CommitSha(String)  : 特定 commit SHA (full 40 hex or short)
//! - `#4` RemoteBranch(String): 远程 branch (fetch 后 checkout)
//!
//! ## 模块边界 (per brief §8 + ULYS-158.4 blocker)
//!
//! 本模块只提供 **trait + types + 4 选 1 resolve 逻辑**, **不** 把
//! `StartFrom` 接入 `WorktreeService::create` trait 签名. 该接入需要
//! `WorktreeCreateRequest::start_from` 字段, 而该字段属于 ULYS-158.4
//! (`worktree_create_async`) 的范围, 当前 in_progress. ULYS-158.4
//! 完成后, caller 可在本 trait 之上加一行 `RealWorktreeCreateAsync::start`
//! 即可接通.
//!
//! ## 不依赖 Multica CLI
//!
//! 同 ULYS-177 路径 C 拍板: 即便 CLI v0.4.42 不含 `start_from_picker`
//! schema, 本 crate 通过 `git-adapter::GitProvider` 直读 git 即可工作.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{SharedDirError, SharedDirResult};
use crate::shared_dir_types::{PickerCandidate, PickerCandidateKind};

// =====================================================================
// 4 选 1 解析结果 (per brief §2.1 + Orca v1.0 §3 FR-ORCA-009)
// =====================================================================

/// 4 选 1 解析结果 — caller (e.g. `RealWorktreeCreateAsync::start`) 据此
/// 调 `git-adapter::GitProvider::create_worktree` 时传 `base_branch` / `start_point`.
///
/// 注意: 本 enum **不** 复制 `git worktree add --detach` 语义; `CommitSha`
/// 在 caller 侧应被映射成 `git checkout <sha> && git switch -c <branch>`
/// 或 `git worktree add --detach` per 实现选择.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum StartFrom {
    /// `#1` 仓库 base ref (origin/main or origin/master) — `git-adapter` 默认 base.
    RepoBaseRef {
        /// 解析后的 ref name (e.g. "refs/remotes/origin/main")
        ref_name: String,
    },
    /// `#2` 另一个本地分支.
    LocalBranch {
        /// 分支名 (e.g. "feat-x")
        branch: String,
    },
    /// `#3` 特定 commit SHA (full 40 hex or short ≥ 7).
    CommitSha {
        /// 完整 SHA (caller 必须已经 resolve 完整 40 hex)
        sha: String,
    },
    /// `#4` 远程 branch — fetch 后 checkout.
    RemoteBranch {
        /// 远程名 (e.g. "origin")
        remote: String,
        /// 分支名 (e.g. "feat-y")
        branch: String,
    },
}

// =====================================================================
// Picker 服务 trait
// =====================================================================

/// `StartFromPicker` — 4 选 1 picker service (per FR-ORCA-009).
///
/// 实现要求 (per brief §2.1):
/// - `list_candidates`: 给一个 `repo_id`, 列所有可 pick 的 4 选 1 candidates.
/// - `resolve`: 给一个候选 + `repo_id`, 返回 `StartFrom` (可被
///   `RealWorktreeCreateAsync::start` 接受).
#[async_trait]
pub trait StartFromPicker: Send + Sync {
    /// 列出所有 4 选 1 candidates (per repo).
    ///
    /// 实现侧典型行为: 调 `git-adapter::GitProvider::list_branches`
    /// 拆 LocalBranch / RemoteBranch 两组; 调 `rev-parse HEAD~` 或
    /// `git config init.defaultBranch` 拿 RepoBaseRef; commit SHA 通过
    /// `git log --pretty=format:%H -n <limit>` 取最近 N 条.
    async fn list_candidates(&self, repo_id: Uuid) -> SharedDirResult<Vec<PickerCandidate>>;

    /// 把候选 resolve 成 `StartFrom`.
    ///
    /// `candidate_id` 来自 `list_candidates` 输出. 实现侧需要按
    /// `candidate.kind` 分派, e.g.:
    /// - `RepoBase` → 调 `GitProvider::head` 拿 base ref HEAD SHA
    /// - `LocalBranch(name)` → 直接返回
    /// - `CommitSha(short)` → 调 `GitProvider::merge_base` / `rev-parse` 展开到 full SHA
    /// - `RemoteBranch(name)` → 先 `git fetch`, 再 resolve
    async fn resolve(
        &self,
        repo_id: Uuid,
        candidate: PickerCandidate,
    ) -> SharedDirResult<StartFrom>;
}

// =====================================================================
// 4 选 1 解析 helper (pure function, 不依赖 GitProvider, 方便 unit test)
// =====================================================================

/// 校验 candidate id 是否匹配某个已知 PickerCandidateKind.
///
/// `candidate.id` 通常是 `<kind>:<value>` 格式 (e.g. `"local_branch:feat-x"`,
/// `"remote_branch:origin/feat-y"`, `"commit_sha:abc1234"`, `"repo_base:main"`).
/// 实现侧可任意格式, 这里给一个 permissive parser, 校验 kind prefix 即可.
pub fn parse_candidate_id(candidate_id: &str) -> SharedDirResult<(PickerCandidateKind, String)> {
    let (kind_str, value) = candidate_id.split_once(':').ok_or_else(|| {
        SharedDirError::new(
            "WSD.PICKER_ID_INVALID",
            format!("candidate id must be '<kind>:<value>', got {candidate_id:?}"),
            "wsd-picker-default-trace",
        )
    })?;
    let kind = match kind_str.trim() {
        "repo_base" => PickerCandidateKind::RepoBase,
        "local_branch" => PickerCandidateKind::LocalBranch,
        "commit_sha" => PickerCandidateKind::CommitSha,
        "remote_branch" => PickerCandidateKind::RemoteBranch,
        other => {
            return Err(SharedDirError::new(
                "WSD.PICKER_ID_INVALID",
                format!("unknown kind {other:?}; expect one of repo_base/local_branch/commit_sha/remote_branch"),
                "wsd-picker-default-trace",
            ));
        }
    };
    let value = value.trim();
    if value.is_empty() {
        return Err(SharedDirError::new(
            "WSD.PICKER_ID_INVALID",
            "candidate id value is empty",
            "wsd-picker-default-trace",
        ));
    }
    Ok((kind, value.to_string()))
}

/// 把 `StartFrom` 转成 caller (e.g. CLI/REST/SSE) 可消费的简短描述.
///
/// 用途: UI 选完 picker 后, 把"我要从 X 拉 worktree"展示给用户.
/// 跟 `StartFrom` derive 的 `Debug` 不同, 这里给稳定的人类可读串.
pub fn describe_start_from(start: &StartFrom) -> String {
    match start {
        StartFrom::RepoBaseRef { ref_name } => format!("repo base ref: {ref_name}"),
        StartFrom::LocalBranch { branch } => format!("local branch: {branch}"),
        StartFrom::CommitSha { sha } => {
            let short = &sha[..sha.len().min(7)];
            format!("commit sha: {short}")
        }
        StartFrom::RemoteBranch { remote, branch } => format!("remote branch: {remote}/{branch}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_candidate_id_recognizes_all_four_kinds() {
        assert_eq!(
            parse_candidate_id("repo_base:main").unwrap(),
            (PickerCandidateKind::RepoBase, "main".into())
        );
        assert_eq!(
            parse_candidate_id("local_branch:feat-x").unwrap(),
            (PickerCandidateKind::LocalBranch, "feat-x".into())
        );
        assert_eq!(
            parse_candidate_id("commit_sha:abc1234").unwrap(),
            (PickerCandidateKind::CommitSha, "abc1234".into())
        );
        assert_eq!(
            parse_candidate_id("remote_branch:origin/feat-y").unwrap(),
            (PickerCandidateKind::RemoteBranch, "origin/feat-y".into())
        );
    }

    #[test]
    fn parse_candidate_id_rejects_missing_colon() {
        let err = parse_candidate_id("feat-x").unwrap_err();
        assert_eq!(err.code, "WSD.PICKER_ID_INVALID");
    }

    #[test]
    fn parse_candidate_id_rejects_unknown_kind() {
        let err = parse_candidate_id("weird_kind:foo").unwrap_err();
        assert_eq!(err.code, "WSD.PICKER_ID_INVALID");
    }

    #[test]
    fn parse_candidate_id_rejects_empty_value() {
        let err = parse_candidate_id("local_branch:").unwrap_err();
        assert_eq!(err.code, "WSD.PICKER_ID_INVALID");
        let err = parse_candidate_id("local_branch:   ").unwrap_err();
        assert_eq!(err.code, "WSD.PICKER_ID_INVALID");
    }

    #[test]
    fn describe_start_from_short_sha_for_commit() {
        let full = "a".repeat(40);
        let s = describe_start_from(&StartFrom::CommitSha { sha: full });
        assert_eq!(s, "commit sha: aaaaaaa");
    }

    #[test]
    fn describe_start_from_short_sha_handles_short_input() {
        let s = describe_start_from(&StartFrom::CommitSha {
            sha: "abc".into(),
        });
        assert_eq!(s, "commit sha: abc");
    }

    #[test]
    fn describe_start_from_each_variant_stable() {
        assert_eq!(
            describe_start_from(&StartFrom::RepoBaseRef {
                ref_name: "refs/remotes/origin/main".into()
            }),
            "repo base ref: refs/remotes/origin/main"
        );
        assert_eq!(
            describe_start_from(&StartFrom::LocalBranch {
                branch: "feat-x".into()
            }),
            "local branch: feat-x"
        );
        assert_eq!(
            describe_start_from(&StartFrom::RemoteBranch {
                remote: "origin".into(),
                branch: "feat-y".into()
            }),
            "remote branch: origin/feat-y"
        );
    }
}
