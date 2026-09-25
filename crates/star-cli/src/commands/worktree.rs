//! `star worktree ...` (MVP 17 核心 #11-#13: create / enter / status)
//!
//! ULYS-218.2 加 `start-from-picker` 子命令 (per FR-ORCA-009 CLI 列表).
//!
//! ## 子命令概览
//!
//! - `star worktree create <id>` — MVP 17 stub (mock JSON 输出)
//! - `star worktree enter <id>` — MVP 17 stub (stdout 打路径, 供 shell eval)
//! - `star worktree status` — MVP 17 stub (mock JSON 输出)
//! - `star worktree start-from-picker <repo>` — ULYS-218.2 新增 (per FR-ORCA-009 CLI).
//!   调 backend `worktree-shared-dir::RealStartFromPicker::list_candidates` /
//!   `resolve`. `<repo>` 是 git repo 本地路径 (后续 Multica 接入 name/UUID
//!   解析后扩展, MVP 阶段 path 是最诚实的最小契约).
//!
//! ## DI 简述
//!
//! 本 crate 当前没有 DI 容器 (per ULYS-196 Skill Registry 阶段 CLI 仍是
//! self-contained 模式). Picker 子命令在 `run` 内直接构造
//! `Arc<CliGitProvider>` + `Arc<StartFromPickerRegistry>` + `Arc<RealStartFromPicker>`,
//! 这是"每次调用 new 一份"的轻量模式 (无全局状态, 进程退出即回收).

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Args, Subcommand};
use serde::Serialize;
use uuid::Uuid;

use git_adapter::provider::GitProvider;
use git_adapter::CliGitProvider;
use worktree_shared_dir::{
    parse_candidate_id, PickerCandidate, RealStartFromPicker, SharedDirError,
    StartFromPickerRegistry,
};
use worktree_shared_dir::start_from_picker::StartFromPicker;

use crate::error::StarError;
use crate::output;

/// `star worktree ...` 顶层 subcommand 枚举
#[derive(Debug, Subcommand)]
pub(crate) enum WorktreeCommand {
    /// MVP 17 核心 #11: stub create (mock JSON)
    Create { id: String },
    /// MVP 17 核心 #12: stub enter (stdout 打路径)
    Enter { id: String },
    /// MVP 17 核心 #13: stub status (mock JSON)
    Status,
    /// ULYS-218.2 / FR-ORCA-009 CLI: 列 4 选 1 start-from picker candidates
    StartFromPicker(StartFromPickerArgs),
}

/// Start-from Picker 子命令参数 (per FR-ORCA-009 CLI, ULYS-218.2).
///
/// `<repo>` 当前是 git repo 本地路径 (`-p` 等同 positional 别名,
/// 跟 `git -C <path>` 命名风格对齐). 后续 Multica 项目名/UUID 接入后
/// 扩展 `--repo-id <uuid>` flag.
#[derive(Debug, Args)]
pub(crate) struct StartFromPickerArgs {
    /// git repo 本地路径 (per FR-ORCA-009 CLI 列表)
    pub repo: String,
    /// `--resolve <kind>:<value>`: resolve 单个候选, 输出 `StartFrom` JSON.
    /// `kind` ∈ {`repo_base`, `local_branch`, `commit_sha`, `remote_branch`}.
    #[arg(long, value_name = "KIND:VALUE")]
    pub resolve: Option<String>,
    /// `--json`: 列模式时改输出 `PickerCandidate[]` JSON (默认 table)
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct Worktree {
    id: String,
    path: String,
    branch: String,
    head_commit: String,
    dirty: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct WorktreeStatus {
    worktree: Worktree,
    last_commit: String,
    uncommitted_files: u32,
}

/// 把 `SharedDirError` 转成 `StarError::Picker` (per CLI 错误模型 §5).
fn picker_err(e: SharedDirError) -> StarError {
    // 走 `format!("[{}] {}", code, message)`, 保留 backend 6-field error 的 code 前缀,
    // 方便用户/agent 排障时定位到 worktree-shared-dir 的具体错误码
    // (e.g. `WSD.PICKER_ID_INVALID` / `WSD.PICKER_GIT_OPEN_FAIL`).
    StarError::Picker(format!("[{}] {}", e.code, e.message))
}

/// 构造轻量 DI: `Arc<dyn GitProvider>` + registry + `RealStartFromPicker`.
fn build_picker() -> Arc<RealStartFromPicker> {
    let git: Arc<dyn GitProvider> = Arc::new(CliGitProvider::new());
    let registry = Arc::new(StartFromPickerRegistry::new());
    Arc::new(RealStartFromPicker::new(git, registry))
}

/// 渲染 picker 候选为人类可读 table (per FR-ORCA-009 CLI 默认输出).
///
/// 4 选 1 分组: REPO BASE REF / LOCAL BRANCH / COMMIT SHA / REMOTE BRANCH.
/// 每组 prefix 一行 group header, 后续每行 `<label>  <description>`.
/// 空组不渲染 header (避免噪音).
fn render_candidates_table(candidates: &[PickerCandidate]) -> String {
    use worktree_shared_dir::PickerCandidateKind as K;
    const HEADERS: &[(K, &str)] = &[
        (K::RepoBase, "REPO BASE REF"),
        (K::LocalBranch, "LOCAL BRANCH"),
        (K::CommitSha, "COMMIT SHA"),
        (K::RemoteBranch, "REMOTE BRANCH"),
    ];
    let mut out = String::new();
    for (kind, header) in HEADERS {
        let group: Vec<&PickerCandidate> =
            candidates.iter().filter(|c| c.kind == *kind).collect();
        if group.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(header);
        out.push('\n');
        for c in group {
            out.push_str("  ");
            out.push_str(&c.label);
            if !c.description.is_empty() {
                out.push_str("  ");
                out.push_str(&c.description);
            }
            out.push('\n');
        }
    }
    if out.is_empty() {
        // 0 候选: 不渲染空 table, 让 caller 走 JSON 或显式 no-candidates 输出.
        return String::new();
    }
    out
}

/// 把 `<repo>` 字符串解析成绝对路径. 解析失败或路径不存在抛 `StarError::Picker`.
///
/// MVP 阶段 `<repo>` 必须是 git repo 本地路径 (per FR-ORCA-009 CLI 契约).
/// 后续 Multica 项目名/UUID 接入后, 这步会先调 project registry 解析,
/// 再 fallback 到 path.
fn resolve_repo_path(repo: &str) -> Result<PathBuf, StarError> {
    let path = PathBuf::from(repo);
    if !path.exists() {
        return Err(StarError::Picker(format!(
            "repo path does not exist: {repo:?}"
        )));
    }
    // 软链接 / 相对路径规范化
    Ok(std::fs::canonicalize(&path).unwrap_or(path))
}

/// 把 `<repo>` 路径注册到 picker registry, 返回内部使用的 repo_id (UUID v4).
fn register_repo(picker: &RealStartFromPicker, path: PathBuf) -> Uuid {
    let registry = Arc::clone(&picker.registry);
    let repo_id = Uuid::new_v4();
    registry.register(repo_id, path);
    repo_id
}

impl WorktreeCommand {
    /// 顶层 dispatcher: sync MVP 17 子命令 + async StartFromPicker 子命令.
    ///
    /// 签名是 `async fn` 因为 `StartFromPicker::list_candidates` / `resolve`
    /// 是 `async`. MVP 17 stub 走 sync 分支 (无 await, 但在 tokio runtime
    /// 上下文里调用无副作用).
    pub(crate) async fn run(self) -> Result<(), StarError> {
        match self {
            WorktreeCommand::Create { id } => worktree_create_stub(&id),
            WorktreeCommand::Enter { id } => worktree_enter_stub(&id),
            WorktreeCommand::Status => worktree_status_stub(),
            WorktreeCommand::StartFromPicker(args) => run_start_from_picker(args).await,
        }
    }
}

/// MVP 17 stub: `star worktree create <id>` → mock JSON
fn worktree_create_stub(id: &str) -> Result<(), StarError> {
    let path = format!("/repos/owner/repo/wt-{id}");
    let wt = Worktree {
        id: format!("wt-{id}"),
        path: path.clone(),
        branch: format!("feature/{id}"),
        head_commit: "deadbeef0000000000000000000000000000000".to_string(),
        dirty: false,
    };
    println!(
        "{}",
        output::json_pretty(serde_json::json!({
            "schema_version": output::SCHEMA_VERSION,
            "mock": true,
            "tool": "worktree create",
            "worktree": wt,
        }))?
    );
    Ok(())
}

/// MVP 17 stub: `star worktree enter <id>` → stdout 打路径 (供 shell eval)
fn worktree_enter_stub(id: &str) -> Result<(), StarError> {
    let path = format!("/repos/owner/repo/wt-{id}");
    println!("{path}");
    Ok(())
}

/// MVP 17 stub: `star worktree status` → mock JSON
fn worktree_status_stub() -> Result<(), StarError> {
    let wt = Worktree {
        id: "wt-current".to_string(),
        path: "/repos/owner/repo".to_string(),
        branch: "main".to_string(),
        head_commit: "deadbeef0000000000000000000000000000000".to_string(),
        dirty: false,
    };
    let status = WorktreeStatus {
        worktree: wt,
        last_commit: "deadbeef".to_string(),
        uncommitted_files: 0,
    };
    println!(
        "{}",
        output::json_pretty(serde_json::json!({
            "schema_version": output::SCHEMA_VERSION,
            "mock": true,
            "tool": "worktree status",
            "status": status,
        }))?
    );
    Ok(())
}

/// `star worktree start-from-picker <repo>` 顶层 handler (per FR-ORCA-009 CLI).
///
/// 三种模式:
/// - 默认 (`--resolve` 未传): 调 `list_candidates`, table 或 JSON 输出.
/// - `--resolve <kind>:<value>`: 调 `resolve`, 输出 `StartFrom` JSON.
/// - `--resolve` + 隐式 `--json` 不冲突: resolve 模式固定 JSON (per spec §2.2).
async fn run_start_from_picker(args: StartFromPickerArgs) -> Result<(), StarError> {
    let path = resolve_repo_path(&args.repo)?;
    let picker = build_picker();
    let repo_id = register_repo(&picker, path);

    // mode 1: --resolve <kind>:<value>
    if let Some(spec) = args.resolve.as_deref() {
        let (kind, value) = parse_candidate_id(spec).map_err(picker_err)?;
        let candidate = PickerCandidate {
            id: spec.to_string(),
            label: value.clone(),
            description: String::new(),
            kind,
        };
        let resolved = picker.resolve(repo_id, candidate).await.map_err(picker_err)?;
        println!(
            "{}",
            output::json_pretty(serde_json::json!({
                "schema_version": output::SCHEMA_VERSION,
                "tool": "worktree start-from-picker resolve",
                "repo_id": repo_id,
                "spec": spec,
                "start_from": resolved,
                "description": worktree_shared_dir::describe_start_from(&resolved),
            }))?
        );
        return Ok(());
    }

    // mode 2: 列 candidates
    let candidates = picker
        .list_candidates(repo_id)
        .await
        .map_err(picker_err)?;

    if args.json {
        println!(
            "{}",
            output::json_pretty(serde_json::json!({
                "schema_version": output::SCHEMA_VERSION,
                "tool": "worktree start-from-picker list",
                "repo_id": repo_id,
                "count": candidates.len(),
                "candidates": candidates,
            }))?
        );
        return Ok(());
    }

    // 默认 table 输出
    let table = render_candidates_table(&candidates);
    if table.is_empty() {
        println!("(no candidates)");
        return Ok(());
    }
    print!("{table}");
    Ok(())
}

// =====================================================================
// Tests (per issue ULYS-218.2 §6 验证: 新增 unit test (CLI 参数解析 + kind/value 拆分) ≥ 5 条)
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: `parse_candidate_id` 拆分 4 选 1 kind 全部命中 (走 backend 已 ship helper)
    #[test]
    fn parse_candidate_id_recognizes_all_four_kinds() {
        assert_eq!(
            parse_candidate_id("repo_base:main").unwrap(),
            (worktree_shared_dir::PickerCandidateKind::RepoBase, "main".into())
        );
        assert_eq!(
            parse_candidate_id("local_branch:feat-x").unwrap(),
            (
                worktree_shared_dir::PickerCandidateKind::LocalBranch,
                "feat-x".into()
            )
        );
        assert_eq!(
            parse_candidate_id("commit_sha:abc1234").unwrap(),
            (
                worktree_shared_dir::PickerCandidateKind::CommitSha,
                "abc1234".into()
            )
        );
        assert_eq!(
            parse_candidate_id("remote_branch:origin/feat-y").unwrap(),
            (
                worktree_shared_dir::PickerCandidateKind::RemoteBranch,
                "origin/feat-y".into()
            )
        );
    }

    /// Test 2: `parse_candidate_id` 缺冒号拒绝
    #[test]
    fn parse_candidate_id_rejects_missing_colon() {
        let err = parse_candidate_id("feat-x").unwrap_err();
        assert_eq!(err.code, "WSD.PICKER_ID_INVALID");
    }

    /// Test 3: `parse_candidate_id` 未知 kind 拒绝
    #[test]
    fn parse_candidate_id_rejects_unknown_kind() {
        let err = parse_candidate_id("weird_kind:foo").unwrap_err();
        assert_eq!(err.code, "WSD.PICKER_ID_INVALID");
    }

    /// Test 4: `render_candidates_table` 4 选 1 分组 + 空组跳过
    #[test]
    fn render_candidates_table_groups_and_skips_empty() {
        use worktree_shared_dir::PickerCandidateKind as K;
        let candidates = vec![
            PickerCandidate {
                id: "repo_base:main".into(),
                label: "main (default)".into(),
                description: "Repo base @ abc1234".into(),
                kind: K::RepoBase,
            },
            PickerCandidate {
                id: "local_branch:feat-x".into(),
                label: "feat-x".into(),
                description: "Local branch @ def5678".into(),
                kind: K::LocalBranch,
            },
            // COMMIT SHA 组空 -> 不渲染 header
            PickerCandidate {
                id: "remote_branch:origin/feat-y".into(),
                label: "origin/feat-y".into(),
                description: "Remote branch @ 9ab0123".into(),
                kind: K::RemoteBranch,
            },
        ];
        let table = render_candidates_table(&candidates);
        assert!(table.contains("REPO BASE REF"), "should have REPO BASE REF header: {table}");
        assert!(table.contains("main (default)"), "should list main: {table}");
        assert!(table.contains("LOCAL BRANCH"), "should have LOCAL BRANCH header: {table}");
        assert!(table.contains("feat-x"), "should list feat-x: {table}");
        assert!(table.contains("REMOTE BRANCH"), "should have REMOTE BRANCH header: {table}");
        assert!(!table.contains("COMMIT SHA"), "should skip empty COMMIT SHA group: {table}");
    }

    /// Test 5: `render_candidates_table` 空输入返回空串 (避免空 header 噪音)
    #[test]
    fn render_candidates_table_empty_returns_empty_string() {
        let table = render_candidates_table(&[]);
        assert_eq!(table, "");
    }

    /// Test 6: `picker_err` 把 backend `SharedDirError` code 透传到 `StarError::Picker`
    #[test]
    fn picker_err_includes_backend_code() {
        let backend_err = worktree_shared_dir::SharedDirError::new(
            "WSD.PICKER_GIT_OPEN_FAIL",
            "git open_repo failed".to_string(),
            "trace-xyz",
        );
        let cli_err = picker_err(backend_err);
        match cli_err {
            StarError::Picker(msg) => {
                assert!(msg.contains("WSD.PICKER_GIT_OPEN_FAIL"), "code should be in msg: {msg}");
                assert!(msg.contains("git open_repo failed"), "message should be in msg: {msg}");
            }
            _ => panic!("expected StarError::Picker, got something else"),
        }
    }

    /// Test 7: `resolve_repo_path` 路径不存在报错
    #[test]
    fn resolve_repo_path_missing_path_errors() {
        let result = resolve_repo_path("/this/path/should/not/exist/abc-xyz");
        match result {
            Err(StarError::Picker(msg)) => {
                assert!(msg.contains("does not exist"), "msg should mention missing: {msg}");
            }
            Ok(p) => panic!("expected error for missing path, got Ok({p:?})"),
            Err(other) => panic!("expected Picker error, got {other:?}"),
        }
    }
}
