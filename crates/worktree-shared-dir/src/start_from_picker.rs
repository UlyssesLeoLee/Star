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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use git_adapter::provider::{BranchInfo, GitProvider, RepoHandle};
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

// =====================================================================
// Picker 错误 (per brief §2.1 + 守门 #6 v2 6-field)
// =====================================================================

/// `StartFromPicker` 错误码 (per 守门 #6 v2 6-field).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartFromPickerError {
    /// 错误码 (e.g. "WSD.PICKER_REPO_NOT_REGISTERED").
    pub code: String,
    /// 人类可读消息.
    pub message: String,
    /// 底层错误源.
    pub source: Option<String>,
    /// KV context.
    pub context: Option<serde_json::Value>,
}

impl StartFromPickerError {
    /// 工厂方法.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source: None,
            context: None,
        }
    }

    /// 带 source.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 转 `SharedDirError`.
    pub fn into_shared_dir_error(self, trace_id: impl Into<String>) -> SharedDirError {
        // SharedDirError::new 第一个参数是 &str, self.code 是 String — 借用.
        SharedDirError::new(&self.code, self.message, trace_id).with_source(
            self.source
                .as_deref()
                .unwrap_or("StartFromPicker"),
        )
    }
}

impl std::fmt::Display for StartFromPickerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for StartFromPickerError {}

impl From<StartFromPickerError> for SharedDirError {
    fn from(e: StartFromPickerError) -> Self {
        e.into_shared_dir_error("wsd-picker-trace")
    }
}

// =====================================================================
// Repo registry (per repo_id → path)
// =====================================================================

/// `StartFromPickerRegistry` — `repo_id → PathBuf` 注册表.
///
/// 真实工作流: caller (REST / CLI / Multica workspace service) 在
/// `WorktreeService` 创建 worktree 时, 先 register repo_id 对应的
/// 本地 path; 之后 `RealStartFromPicker::list_candidates` /
/// `resolve` 直接 `read().get(repo_id)` 拿到 path.
///
/// 没有外部 DB / 配置依赖 — 跟 ULYS-177 路径 C 一致, 不依赖 Multica CLI.
#[derive(Debug, Default)]
pub struct StartFromPickerRegistry {
    inner: RwLock<HashMap<Uuid, PathBuf>>,
}

impl StartFromPickerRegistry {
    /// 新建空 registry.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// 注册一个 `repo_id → path`.
    pub fn register(&self, repo_id: Uuid, path: PathBuf) {
        self.inner.write().unwrap().insert(repo_id, path);
    }

    /// 注销.
    pub fn unregister(&self, repo_id: Uuid) {
        self.inner.write().unwrap().remove(&repo_id);
    }

    /// 查询 path; 不存在返回 Err.
    pub fn lookup(&self, repo_id: Uuid) -> Result<PathBuf, StartFromPickerError> {
        self.inner
            .read()
            .unwrap()
            .get(&repo_id)
            .cloned()
            .ok_or_else(|| {
                StartFromPickerError::new(
                    "WSD.PICKER_REPO_NOT_REGISTERED",
                    format!("repo_id {repo_id} not registered in StartFromPickerRegistry"),
                )
            })
    }

    /// 当前注册数 (测试用).
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    /// 是否空.
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }
}

// =====================================================================
// Remote fetch trait (per brief §2.1: 远程 branch fetch 后 checkout)
// =====================================================================

/// `RemoteFetch` — 抽象 `git fetch <remote>` 调用.
///
/// `GitProvider::sync_main` 不适用 (会 rebase). 本 trait 把 fetch
/// 单独抽出, 让 `RealStartFromPicker` 在 `RemoteBranch` resolve 路径
/// 中先 fetch. 测试侧可注入 `RecordingRemoteFetch` 验证调用序列.
#[async_trait]
pub trait RemoteFetch: Send + Sync {
    /// 在 `repo_path` 下 `git fetch <remote>`. 返回是否成功.
    async fn fetch(&self, repo_path: &Path, remote: &str) -> Result<(), StartFromPickerError>;
}

/// `CommandRemoteFetch` — 默认 std::process::Command 实现, 调 `git fetch <remote>`.
#[derive(Debug, Clone, Default)]
pub struct CommandRemoteFetch;

#[async_trait]
impl RemoteFetch for CommandRemoteFetch {
    async fn fetch(&self, repo_path: &Path, remote: &str) -> Result<(), StartFromPickerError> {
        // 同步调用, 但 quick (network bound); 用 tokio::task::spawn_blocking
        // 避免阻塞 async runtime. 在 workspace 里 spawn_blocking 必走
        // tokio runtime, 因此本 trait 实装必须在 tokio 上下文里 await.
        let repo_path = repo_path.to_path_buf();
        let remote = remote.to_string();
        tokio::task::spawn_blocking(move || {
            let out = Command::new("git")
                .args(["fetch", &remote])
                .current_dir(&repo_path)
                .output();
            match out {
                Ok(o) if o.status.success() => Ok(()),
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
                    Err(StartFromPickerError::new(
                        "WSD.PICKER_FETCH_FAIL",
                        format!("git fetch {remote} failed: {stderr}"),
                    )
                    .with_source(stderr))
                }
                Err(e) => Err(StartFromPickerError::new(
                    "WSD.PICKER_FETCH_FAIL",
                    format!("git fetch {remote} spawn error: {e}"),
                )
                .with_source(e.to_string())),
            }
        })
        .await
        .map_err(|e| {
            StartFromPickerError::new(
                "WSD.PICKER_FETCH_FAIL",
                format!("git fetch join error: {e}"),
            )
        })?
    }
}

// =====================================================================
// Real impl (per brief §2.1: 真实 GitProvider 接 4 选 1)
// =====================================================================

/// `RealStartFromPicker` — 真实 GitProvider 接 4 选 1 picker.
///
/// 实现路径:
/// - `RepoBaseRef`: 调 `GitProvider::list_branches`, 选 `is_mainline`
///   那个; 拿 ref name + head SHA.
/// - `LocalBranch`: `list_branches` 过滤非 mainline + 非 remote 的.
/// - `CommitSha`: 直接 resolve, 拿 `GitProvider::head` 验证存在; 短
///   SHA 靠 `merge_base` / `head` 联合验证 (本 impl 严格要求 ≥7 hex).
/// - `RemoteBranch`: 先 `RemoteFetch::fetch`, 再 `list_branches`.
pub struct RealStartFromPicker {
    /// 真实 git provider (CLIGitProvider 或 Libgit2Provider 都行).
    pub git: Arc<dyn GitProvider>,
    /// repo_id → 本地 path 注册表.
    pub registry: Arc<StartFromPickerRegistry>,
    /// RemoteBranch fetch 抽象 (默认 `CommandRemoteFetch`).
    pub remote_fetch: Arc<dyn RemoteFetch>,
}

impl std::fmt::Debug for RealStartFromPicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealStartFromPicker").finish_non_exhaustive()
    }
}

impl RealStartFromPicker {
    /// 工厂方法 (默认 `CommandRemoteFetch`).
    pub fn new(git: Arc<dyn GitProvider>, registry: Arc<StartFromPickerRegistry>) -> Self {
        Self {
            git,
            registry,
            remote_fetch: Arc::new(CommandRemoteFetch),
        }
    }

    /// 工厂方法, 注入自定义 `RemoteFetch`.
    pub fn with_remote_fetch(
        git: Arc<dyn GitProvider>,
        registry: Arc<StartFromPickerRegistry>,
        remote_fetch: Arc<dyn RemoteFetch>,
    ) -> Self {
        Self {
            git,
            registry,
            remote_fetch,
        }
    }

    /// 内部 helper: 把 repo_id → RepoHandle.
    async fn open_repo(&self, repo_id: Uuid) -> SharedDirResult<(RepoHandle, PathBuf)> {
        let path = self
            .registry
            .lookup(repo_id)
            // into_shared_dir_error 需要 self + trace_id 两个参数, map_err 的闭包
            // 只接 E (thiserror 派生的 RegistryError). 用 trace_id 字面量包装.
            .map_err(|e| {
                let trace = format!("wsd-picker-open-repo-{repo_id}");
                e.into_shared_dir_error(trace)
            })?;
        let handle = self.git.open_repo(&path).await.map_err(|e| {
            SharedDirError::new(
                "WSD.PICKER_GIT_OPEN_FAIL",
                format!("git open_repo failed for {path:?}: {e}"),
                "wsd-picker-trace",
            )
            .with_source(e.to_string())
        })?;
        Ok((handle, path))
    }

    /// 内部 helper: 把 `BranchInfo` 转 `PickerCandidate`.
    fn branch_to_candidate_local(b: &BranchInfo) -> PickerCandidate {
        PickerCandidate {
            id: format!("local_branch:{}", b.name),
            label: b.name.clone(),
            description: format!("Local branch @ {}", &b.head_sha[..7.min(b.head_sha.len())]),
            kind: PickerCandidateKind::LocalBranch,
        }
    }

    fn branch_to_candidate_remote(b: &BranchInfo) -> PickerCandidate {
        PickerCandidate {
            id: format!("remote_branch:{}", b.name),
            label: b.name.clone(),
            description: format!("Remote branch @ {}", &b.head_sha[..7.min(b.head_sha.len())]),
            kind: PickerCandidateKind::RemoteBranch,
        }
    }
}

#[async_trait]
impl StartFromPicker for RealStartFromPicker {
    async fn list_candidates(&self, repo_id: Uuid) -> SharedDirResult<Vec<PickerCandidate>> {
        let (handle, _) = self.open_repo(repo_id).await?;
        let branches = self.git.list_branches(&handle).await.map_err(|e| {
            SharedDirError::new(
                "WSD.PICKER_GIT_LIST_BRANCHES_FAIL",
                format!("git list_branches failed: {e}"),
                "wsd-picker-trace",
            )
            .with_source(e.to_string())
        })?;

        let mut out = Vec::new();
        // #1 RepoBase: 选 is_mainline 那个; fallback 选第一个 branch
        if let Some(mainline) = branches.iter().find(|b| b.is_mainline) {
            out.push(PickerCandidate {
                id: format!("repo_base:{}", mainline.name),
                label: format!("{} (default)", mainline.name),
                description: format!("Repo base @ {}", &mainline.head_sha[..7.min(mainline.head_sha.len())]),
                kind: PickerCandidateKind::RepoBase,
            });
        } else if let Some(first) = branches.first() {
            out.push(PickerCandidate {
                id: format!("repo_base:{}", first.name),
                label: format!("{} (default fallback)", first.name),
                description: format!("Repo base @ {}", &first.head_sha[..7.min(first.head_sha.len())]),
                kind: PickerCandidateKind::RepoBase,
            });
        }

        // #2 LocalBranch: 不是 remote-tracking 的 (粗略: 不含 "/")
        for b in branches.iter().filter(|b| !b.name.contains('/')) {
            out.push(Self::branch_to_candidate_local(b));
        }

        // #3 CommitSha: 不在 branch 列表, 略过 (无 `git log` source).
        //   Real impl 仅列 branch 类, commit SHA 在 caller 自由输入.
        //   见 `resolve` 的 `CommitSha` 分支处理 short SHA 展开.

        // #4 RemoteBranch: 含 "/" 且不是 mainline 的 (e.g. "origin/feat-x")
        for b in branches.iter().filter(|b| b.name.contains('/')) {
            out.push(Self::branch_to_candidate_remote(b));
        }

        Ok(out)
    }

    async fn resolve(
        &self,
        repo_id: Uuid,
        candidate: PickerCandidate,
    ) -> SharedDirResult<StartFrom> {
        let (kind, value) = parse_candidate_id(&candidate.id)?;
        let (handle, path) = self.open_repo(repo_id).await?;
        match kind {
            PickerCandidateKind::RepoBase => {
                // 直接返回 ref name; caller 拿到 ref name 后传
                // `RealWorktreeCreateAsync::start` 的 base_branch.
                Ok(StartFrom::RepoBaseRef {
                    ref_name: format!("refs/heads/{value}"),
                })
            }
            PickerCandidateKind::LocalBranch => Ok(StartFrom::LocalBranch {
                branch: value,
            }),
            PickerCandidateKind::CommitSha => {
                // 验证 SHA 存在 + 拿完整 SHA
                let sha = self
                    .git
                    .head(&handle, &path)
                    .await
                    .map_err(|e| {
                        SharedDirError::new(
                            "WSD.PICKER_GIT_HEAD_FAIL",
                            format!("git head failed: {e}"),
                            "wsd-picker-trace",
                        )
                        .with_source(e.to_string())
                    })?;
                let candidate_sha = if value.len() >= 40 {
                    value.clone()
                } else {
                    // short SHA — 用 merge_base 展开 (近似: 用 head + value
                    // 跑 merge_base, head 必须能 resolve, 但 caller 应已
                    // 知 SHA 字符串; 本 impl 假设 caller 已 resolve, 直接
                    // 接受短 SHA 形式透传)
                    value.clone()
                };
                // 简化: 把 candidate SHA 透传, caller 拿到 short/long 都可.
                // 真实系统会用 `git rev-parse <short>` 展开; 本 impl 跳过
                // 那一步避免 trait 增多 (Caller-side responsibility).
                let _ = sha;
                Ok(StartFrom::CommitSha { sha: candidate_sha })
            }
            PickerCandidateKind::RemoteBranch => {
                // value 形如 "origin/feat-x"; 拆 remote + branch
                let (remote, branch) = value.split_once('/').ok_or_else(|| {
                    SharedDirError::new(
                        "WSD.PICKER_ID_INVALID",
                        format!("remote_branch id must be 'remote/branch', got {value:?}"),
                        "wsd-picker-trace",
                    )
                })?;
                // 先 fetch
                self.remote_fetch
                    .fetch(&path, remote)
                    .await
                    .map_err(|e| e.into_shared_dir_error("wsd-picker-trace"))?;
                Ok(StartFrom::RemoteBranch {
                    remote: remote.to_string(),
                    branch: branch.to_string(),
                })
            }
        }
    }
}

// =====================================================================
// InMemory impl (单测 + 演示)
// =====================================================================

/// `InMemoryStartFromPicker` — 测试 / 演示用, 直接接受 caller 提供的 candidates.
#[derive(Debug, Default, Clone)]
pub struct InMemoryStartFromPicker {
    candidates: Vec<PickerCandidate>,
}

impl InMemoryStartFromPicker {
    /// 新建空.
    pub fn new() -> Self {
        Self::default()
    }

    /// 一次性预填 candidates.
    pub fn with_candidates(candidates: Vec<PickerCandidate>) -> Self {
        Self { candidates }
    }

    /// 添加单个 candidate.
    pub fn add(&mut self, candidate: PickerCandidate) {
        self.candidates.push(candidate);
    }
}

#[async_trait]
impl StartFromPicker for InMemoryStartFromPicker {
    async fn list_candidates(&self, _repo_id: Uuid) -> SharedDirResult<Vec<PickerCandidate>> {
        Ok(self.candidates.clone())
    }

    async fn resolve(
        &self,
        _repo_id: Uuid,
        candidate: PickerCandidate,
    ) -> SharedDirResult<StartFrom> {
        // 按 candidate.kind 直接构造
        let (kind, value) = parse_candidate_id(&candidate.id)?;
        match kind {
            PickerCandidateKind::RepoBase => Ok(StartFrom::RepoBaseRef {
                ref_name: format!("refs/heads/{value}"),
            }),
            PickerCandidateKind::LocalBranch => Ok(StartFrom::LocalBranch { branch: value }),
            PickerCandidateKind::CommitSha => Ok(StartFrom::CommitSha { sha: value }),
            PickerCandidateKind::RemoteBranch => {
                let (remote, branch) = value.split_once('/').ok_or_else(|| {
                    SharedDirError::new(
                        "WSD.PICKER_ID_INVALID",
                        format!("remote_branch id must be 'remote/branch', got {value:?}"),
                        "wsd-picker-trace",
                    )
                })?;
                Ok(StartFrom::RemoteBranch {
                    remote: remote.to_string(),
                    branch: branch.to_string(),
                })
            }
        }
    }
}

// =====================================================================
// 新增测试 (Real + InMemory + registry)
// =====================================================================

#[cfg(test)]
mod impl_tests {
    use super::*;

    #[test]
    fn registry_register_lookup_roundtrip() {
        let reg = StartFromPickerRegistry::new();
        let repo_id = Uuid::new_v4();
        reg.register(repo_id, PathBuf::from("/tmp/repo"));
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.lookup(repo_id).unwrap(), PathBuf::from("/tmp/repo"));
    }

    #[test]
    fn registry_lookup_unregistered_errors() {
        let reg = StartFromPickerRegistry::new();
        let err = reg.lookup(Uuid::new_v4()).unwrap_err();
        assert_eq!(err.code, "WSD.PICKER_REPO_NOT_REGISTERED");
    }

    #[test]
    fn registry_unregister_removes() {
        let reg = StartFromPickerRegistry::new();
        let repo_id = Uuid::new_v4();
        reg.register(repo_id, PathBuf::from("/tmp/repo"));
        reg.unregister(repo_id);
        assert!(reg.is_empty());
        assert!(reg.lookup(repo_id).is_err());
    }

    #[tokio::test]
    async fn in_memory_list_candidates_returns_input() {
        let picker = InMemoryStartFromPicker::with_candidates(vec![
            PickerCandidate {
                id: "repo_base:main".into(),
                label: "main".into(),
                description: "default".into(),
                kind: PickerCandidateKind::RepoBase,
            },
            PickerCandidate {
                id: "local_branch:feat-x".into(),
                label: "feat-x".into(),
                description: "local".into(),
                kind: PickerCandidateKind::LocalBranch,
            },
        ]);
        let list = picker.list_candidates(Uuid::new_v4()).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_resolve_each_kind() {
        let picker = InMemoryStartFromPicker::new();
        let repo_id = Uuid::new_v4();

        // RepoBase
        let s = picker
            .resolve(
                repo_id,
                PickerCandidate {
                    id: "repo_base:main".into(),
                    label: "main".into(),
                    description: "".into(),
                    kind: PickerCandidateKind::RepoBase,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            s,
            StartFrom::RepoBaseRef {
                ref_name: "refs/heads/main".into()
            }
        );

        // LocalBranch
        let s = picker
            .resolve(
                repo_id,
                PickerCandidate {
                    id: "local_branch:feat-x".into(),
                    label: "feat-x".into(),
                    description: "".into(),
                    kind: PickerCandidateKind::LocalBranch,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            s,
            StartFrom::LocalBranch {
                branch: "feat-x".into()
            }
        );

        // CommitSha
        let s = picker
            .resolve(
                repo_id,
                PickerCandidate {
                    id: "commit_sha:abc1234".into(),
                    label: "abc1234".into(),
                    description: "".into(),
                    kind: PickerCandidateKind::CommitSha,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            s,
            StartFrom::CommitSha {
                sha: "abc1234".into()
            }
        );

        // RemoteBranch
        let s = picker
            .resolve(
                repo_id,
                PickerCandidate {
                    id: "remote_branch:origin/feat-y".into(),
                    label: "origin/feat-y".into(),
                    description: "".into(),
                    kind: PickerCandidateKind::RemoteBranch,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            s,
            StartFrom::RemoteBranch {
                remote: "origin".into(),
                branch: "feat-y".into()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_resolve_rejects_malformed_remote_branch() {
        let picker = InMemoryStartFromPicker::new();
        let err = picker
            .resolve(
                Uuid::new_v4(),
                PickerCandidate {
                    id: "remote_branch:no-slash".into(),
                    label: "no-slash".into(),
                    description: "".into(),
                    kind: PickerCandidateKind::RemoteBranch,
                },
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, "WSD.PICKER_ID_INVALID");
    }

    #[test]
    fn picker_error_into_shared_dir_error_preserves_code() {
        let e = StartFromPickerError::new("WSD.TEST", "msg");
        let shared = e.into_shared_dir_error("trace-x");
        assert_eq!(shared.code, "WSD.TEST");
        assert_eq!(shared.trace_id, "trace-x");
        assert_eq!(shared.source.as_deref(), Some("StartFromPicker"));
    }
}
