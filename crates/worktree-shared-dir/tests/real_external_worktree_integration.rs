//! `real_external_worktree_integration.rs` — Real `RealExternalWorktreeImport` 集成测试
//! (per ULYS-218.4 / ULYS-231 §6).
//!
//! 真实 git repo + `git worktree add` + `git worktree remove` 端到端覆盖
//! `RealExternalWorktreeImport::scan / import / cleanup_stale`:
//!
//! - `real_scan_external_worktrees_returns_porcelain_v2` — 真实 git worktree list 解析
//! - `real_cleanup_stale_removes_removed_worktree` — `git worktree remove` 后 cleanup_stale
//! - `real_import_marks_managed_true_via_real_git_workflow` — import 路径真实端到端
//!
//! 守门:
//! - #19: 仅依赖 git-adapter CLI impl + tempfile (system git binary)
//! - 串行执行 (serial_test) — 避免 parallel tempdir 污染
//!
//! ## Skip 条件
//!
//! 没有 `git` binary 的 CI 环境 skip (`skip_if_no_git()`); 实测环境是 Windows +
//! Git for Windows, 默认有 git.

#![allow(clippy::needless_return)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use chrono::Utc;
use git_adapter::CliGitProvider;
use graph_core::types::RepoId;
use serial_test::serial;

use worktree_shared_dir::{
    ExternalWorktreeImport, ExternalWorktreeImportRegistry, RealExternalWorktreeImport,
};

// =====================================================================
// Test helpers
// =====================================================================

/// 创建临时 git repo, 做 1 个 commit, 返回 PathBuf.
fn init_git_repo_with_commit() -> Option<PathBuf> {
    let tmp = std::env::temp_dir().join(format!(
        "ulys231-test-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    if std::fs::create_dir_all(&tmp).is_err() {
        return None;
    }
    let run = |args: &[&str]| -> bool {
        Command::new("git")
            .current_dir(&tmp)
            .args(args)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };
    if !run(&["init", "-q", "-b", "main"]) {
        return None;
    }
    run(&["config", "user.email", "test@example.com"]);
    run(&["config", "user.name", "test"]);
    run(&["config", "commit.gpgsign", "false"]);
    std::fs::write(tmp.join("README.md"), "hello\n").ok()?;
    if !run(&["add", "."]) {
        return None;
    }
    if !run(&["commit", "-q", "-m", "init"]) {
        return None;
    }
    Some(tmp)
}

/// 添加一个 worktree 到 git repo, 返回 wt path.
fn git_worktree_add(repo: &Path, wt_path: &Path, branch: &str) -> bool {
    Command::new("git")
        .current_dir(repo)
        .args(["worktree", "add", "-q", "-b", branch])
        .arg(wt_path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `git worktree remove` 强制删除 (含 dirty file).
fn git_worktree_remove(repo: &Path, wt_path: &Path) -> bool {
    Command::new("git")
        .current_dir(repo)
        .args(["worktree", "remove", "--force"])
        .arg(wt_path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn skip_if_no_git() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
}

/// 构造 RealExternalWorktreeImport + Registry, repo 已 register.
fn make_real(
    repo_id: RepoId,
    path: PathBuf,
) -> (
    RealExternalWorktreeImport,
    Arc<ExternalWorktreeImportRegistry>,
) {
    let git: Arc<dyn git_adapter::GitProvider> = Arc::new(CliGitProvider::default());
    let registry = Arc::new(ExternalWorktreeImportRegistry::new());
    registry.register(repo_id, path);
    let real = RealExternalWorktreeImport::new(git, registry.clone());
    (real, registry)
}

// =====================================================================
// Integration tests (per ULYS-218.4 §6: ≥ 3 条)
// =====================================================================

/// AC-1 hidden default: 真实 git worktree add 后, scan 返回 porcelain
/// 含 path / head_commit / branch, 且 is_managed=false (per ULYS-218.4 §2.3).
#[tokio::test]
#[serial]
async fn real_scan_external_worktrees_returns_porcelain_v2() {
    if skip_if_no_git() {
        eprintln!("[skip] git binary not available");
        return;
    }
    let Some(repo) = init_git_repo_with_commit() else {
        eprintln!("[skip] init_git_repo_with_commit failed");
        return;
    };

    // git worktree add 一个新分支
    let wt_path = repo.join("wt-feat");
    assert!(
        git_worktree_add(&repo, &wt_path, "feat-x"),
        "git worktree add failed"
    );

    // 注册 + scan
    let repo_id = RepoId::new_v4();
    let (real, _reg) = make_real(repo_id, repo.clone());
    let items = real.scan(repo_id).await.expect("scan failed");

    // 真实 git worktree list 至少返回 1 个 main + 1 个 feat-x = 2
    assert!(
        items.len() >= 2,
        "expected ≥ 2 worktrees (main + feat-x), got {}: {:?}",
        items.len(),
        items
    );
    let feat = items
        .iter()
        .find(|w| w.path == wt_path)
        .expect("feat-x worktree 应在 scan 里");
    assert_eq!(feat.branch.as_deref(), Some("feat-x"));
    assert!(!feat.is_managed, "scan 默认全部 is_managed=false per AC-1");
    assert_eq!(feat.head_commit.len(), 40, "head_commit 应是 full SHA");
    assert!(
        feat.head_commit.chars().all(|c| c.is_ascii_hexdigit()),
        "head_commit 应是 hex"
    );
}

/// AC-3 cleanup_stale: `git worktree remove` 后, cleanup_stale 找差异.
#[tokio::test]
#[serial]
async fn real_cleanup_stale_removes_removed_worktree() {
    if skip_if_no_git() {
        return;
    }
    let Some(repo) = init_git_repo_with_commit() else {
        return;
    };

    // 先 add 1 个 wt
    let wt_path = repo.join("wt-removed");
    assert!(git_worktree_add(&repo, &wt_path, "feat-removed"));

    let repo_id = RepoId::new_v4();
    let (real, registry) = make_real(repo_id, repo.clone());

    // 第一次 scan: 应包含 wt-removed
    let before = real.scan(repo_id).await.unwrap();
    let before_paths: Vec<PathBuf> = before.iter().map(|w| w.path.clone()).collect();
    assert!(
        before_paths.contains(&wt_path),
        "scan 应包含 wt-removed: {before_paths:?}"
    );

    // import 它 (per AC-2 进 managed 集合) — cleanup_stale 只 diff managed,
    // 不 import 的话它根本不会被当成 "曾经 managed 现在消失" 的候选.
    real.import(repo_id, wt_path.clone())
        .await
        .expect("import failed");
    assert_eq!(real.managed_paths(repo_id), vec![wt_path.clone()]);

    // git worktree remove --force (强删, 避免 dirty block) — 模拟用户在
    // Orca 外部直接删掉这个已 import 的 worktree.
    assert!(
        git_worktree_remove(&repo, &wt_path),
        "git worktree remove failed"
    );

    // 第二次 scan: wt-removed 应消失
    let after_scan = real.scan(repo_id).await.unwrap();
    let after_paths: Vec<PathBuf> = after_scan.iter().map(|w| w.path.clone()).collect();
    assert!(
        !after_paths.contains(&wt_path),
        "scan 不应再含已 remove 的 wt-removed: {after_paths:?}"
    );

    // cleanup_stale: managed 里的 wt-removed 已经不在 porcelain 里了
    // → 应精确返回 [wt_path] (per Real impl §2.2 cleanup_stale 逻辑, 只 diff
    // managed, 不碰主仓 registry path).
    let stale = real.cleanup_stale(repo_id).await.unwrap();
    assert_eq!(stale, vec![wt_path.clone()], "应精确返回被外部删除的 wt-removed");
    assert!(
        real.managed_paths(repo_id).is_empty(),
        "cleanup_stale 后 wt-removed 应从 managed 里摘掉"
    );
    let _ = registry;
}

/// AC-2 import: scan 后 import 一个 path, 应标 is_managed=true.
#[tokio::test]
#[serial]
async fn real_import_marks_managed_true_via_real_git_workflow() {
    if skip_if_no_git() {
        return;
    }
    let Some(repo) = init_git_repo_with_commit() else {
        return;
    };

    let wt_path = repo.join("wt-import");
    assert!(git_worktree_add(&repo, &wt_path, "feat-import"));

    let repo_id = RepoId::new_v4();
    let (real, _reg) = make_real(repo_id, repo.clone());

    // 先 scan 拿到 path
    let items = real.scan(repo_id).await.unwrap();
    assert!(items.iter().any(|w| w.path == wt_path));

    // import
    let _wt_id = real
        .import(repo_id, wt_path.clone())
        .await
        .expect("import failed");

    // 再 scan 应标 is_managed=true
    let after = real.scan(repo_id).await.unwrap();
    let imported = after
        .iter()
        .find(|w| w.path == wt_path)
        .expect("wt-import 应仍在 scan 里");
    assert!(
        imported.is_managed,
        "import 后 scan 应标 is_managed=true"
    );
}