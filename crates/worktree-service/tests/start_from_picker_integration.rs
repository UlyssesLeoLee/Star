//! `start_from_picker_integration.rs` — 集成测试 (per ULYS-194 §6 ≥ 2 integration).
//!
//! 测试 `WorktreeService::pick_start_from_candidates` 在不同 source 配置下的行为.
//!
//! 守门 #19: 仅依赖 worktree-service 内部 + git-adapter trait 全部方法实现.

#![allow(clippy::needless_return)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use git_adapter::provider::BranchInfo as GitBranch;
use git_adapter::{
    Diff, GitError, GitProvider, MergeResult, RepoHandle, StatusEntry, WorktreeInfo,
};
use graph_core::state::{HumanState, TestState};
use graph_core::types::{RepoId, WorktreeId};
use uuid::Uuid;

use worktree_service::{
    pick_start_from_candidates, worktree_to_existing_candidate, DefaultStartFromPickerSource,
    InMemoryWorktreeService, NoopPickerSource, PickerCandidate, StartFromPickerSource, Worktree,
    WorktreeService,
};
use worktree_shared_dir::{PickerCandidateKind as SdKind};

// =====================================================================
// StubGit — 满足 GitProvider 全部 14 个方法 (用默认值), 只 list_branches 返回预设
// =====================================================================

#[derive(Debug, Clone)]
struct StubGit {
    branches: Vec<GitBranch>,
    should_fail: bool,
}

#[async_trait]
impl GitProvider for StubGit {
    async fn open_repo(&self, _: &Path) -> Result<RepoHandle, GitError> {
        Ok(RepoHandle { path: PathBuf::from("/tmp"), repo_id: None })
    }
    async fn list_worktrees(&self, _: &RepoHandle) -> Result<Vec<WorktreeInfo>, GitError> {
        Ok(Vec::new())
    }
    async fn create_worktree(
        &self,
        _: &RepoHandle,
        _: &str,
        _: &Path,
        _: Option<&str>,
    ) -> Result<WorktreeInfo, GitError> {
        unimplemented!()
    }
    async fn remove_worktree(
        &self,
        _: &RepoHandle,
        _: &Path,
        _: bool,
    ) -> Result<(), GitError> {
        Ok(())
    }
    async fn diff(&self, _: &RepoHandle, _: &str, _: &str) -> Result<Diff, GitError> {
        unimplemented!()
    }
    async fn merge_base(&self, _: &RepoHandle, _: &str, _: &str) -> Result<String, GitError> {
        Ok(String::new())
    }
    async fn rev_list_count(&self, _: &RepoHandle, _: &str) -> Result<u32, GitError> {
        Ok(0)
    }
    async fn status(&self, _: &RepoHandle) -> Result<Vec<StatusEntry>, GitError> {
        Ok(Vec::new())
    }
    async fn sync_main(
        &self,
        _: &RepoHandle,
        _: &Path,
    ) -> Result<git_adapter::SyncResult, GitError> {
        unimplemented!()
    }
    async fn rebase(&self, _: &RepoHandle, _: &Path, _: &str) -> Result<(), GitError> {
        unimplemented!()
    }
    async fn merge(
        &self,
        _: &RepoHandle,
        _: &Path,
        _: &str,
        _: git_adapter::provider::MergeStrategy,
    ) -> Result<MergeResult, GitError> {
        unimplemented!()
    }
    async fn head(&self, _: &RepoHandle, _: &Path) -> Result<String, GitError> {
        Ok(String::new())
    }
    async fn last_commit_at(&self, _: &RepoHandle, _: &Path) -> Result<DateTime<Utc>, GitError> {
        Ok(Utc::now())
    }
    async fn list_branches(&self, _: &RepoHandle) -> Result<Vec<GitBranch>, GitError> {
        if self.should_fail {
            return Err(GitError::new("GIT.STUB_FAIL", "stub fail"));
        }
        Ok(self.branches.clone())
    }
}

fn branch(name: &str, sha: &str, is_main: bool) -> GitBranch {
    GitBranch {
        name: name.to_string(),
        head_sha: sha.to_string(),
        is_mainline: is_main,
        updated_at: Utc::now(),
    }
}

fn make_wt(repo_id: RepoId, branch_name: &str, archived: bool) -> Worktree {
    let now = Utc::now();
    Worktree {
        id: WorktreeId::from(Uuid::new_v4()),
        repo_id,
        name: branch_name.to_string(),
        branch: branch_name.to_string(),
        path: PathBuf::from(format!("/worktrees/{branch_name}")),
        human_state: HumanState::Running,
        test_state: TestState::None,
        ahead: 0,
        behind: 0,
        dirty: false,
        locked: false,
        archived,
        health_score: 100,
        last_activity: now,
        created_at: now,
        merged_at: None,
        locked_by: None,
        agent_type: None,
    }
}

/// Stub StartFromPickerSource — 返回预设 4 分类 (用于 with_picker_source 测试)
struct StubPicker {
    pub github: Vec<PickerCandidate>,
    pub local: Vec<PickerCandidate>,
    pub empty: PickerCandidate,
}

#[async_trait]
impl StartFromPickerSource for StubPicker {
    async fn list_github_branches(
        &self,
        _: RepoId,
    ) -> Result<Vec<PickerCandidate>, worktree_service::ServiceError> {
        Ok(self.github.clone())
    }
    async fn list_existing_worktrees(
        &self,
        _: RepoId,
    ) -> Result<Vec<PickerCandidate>, worktree_service::ServiceError> {
        Ok(Vec::new())
    }
    async fn list_local_paths(
        &self,
        _: RepoId,
    ) -> Result<Vec<PickerCandidate>, worktree_service::ServiceError> {
        Ok(self.local.clone())
    }
    fn empty_candidate(&self, repo_id: RepoId) -> PickerCandidate {
        let mut e = self.empty.clone();
        e.id = format!("empty:{repo_id}");
        e
    }
}

// =====================================================================
// 集成测试
// =====================================================================

#[tokio::test]
async fn integration_default_source_via_real_git_provider_shape() {
    // 集成: DefaultStartFromPickerSource + StubGit 满足 GitProvider 接口,
    // 调 pick_start_from_candidates, 验证 4 分类全走通.
    let git: Arc<dyn GitProvider> = Arc::new(StubGit {
        branches: vec![
            branch("main", "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111", true),
            branch("dev", "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222", false),
        ],
        should_fail: false,
    });
    let src = DefaultStartFromPickerSource::new(git);
    let repo_id = RepoId::from(Uuid::new_v4());

    let existing: Vec<Worktree> = vec![
        make_wt(repo_id, "main", false),
        make_wt(repo_id, "archived-feat", true), // 应过滤
    ];

    std::env::set_var("WORKTREE_LOCAL_PATHS", "/tmp/proj-x");
    let out = pick_start_from_candidates(repo_id, &existing, &src)
        .await
        .unwrap();
    std::env::remove_var("WORKTREE_LOCAL_PATHS");

    assert_eq!(out.github_branches.len(), 2, "kind 1: github remote branches");
    assert_eq!(out.existing_worktrees.len(), 1, "kind 2: existing (archived filtered)");
    assert_eq!(out.local_paths.len(), 1, "kind 3: local paths from env");
    assert!(out.empty.is_some(), "kind 4: empty always present");
    assert_eq!(out.total(), 5);
}

#[tokio::test]
async fn integration_worktree_service_trait_method_picks_with_noop_source() {
    // 集成: WorktreeService::pick_start_from_candidates trait 方法 + NoopPickerSource 默认.
    // 没 with_picker_source → 用 NoopPickerSource fallback (永远只返 empty).
    let svc = InMemoryWorktreeService::new();
    let repo_id = RepoId::from(Uuid::new_v4());
    svc.create(repo_id, "main", None).await.unwrap();
    svc.create(repo_id, "feat-y", None).await.unwrap();

    let out = svc.pick_start_from_candidates(repo_id).await.unwrap();
    // NoopPickerSource → github/local 都空; existing 来自 InMemoryWorktreeService::list
    assert!(out.github_branches.is_empty());
    assert!(out.local_paths.is_empty());
    assert_eq!(out.existing_worktrees.len(), 2, "in-memory existing picked up");
    assert!(out.empty.is_some());
    assert_eq!(out.total(), 3);
}

#[tokio::test]
async fn integration_worktree_service_with_custom_picker_source() {
    // 集成: 用户用 with_picker_source 注入自定义 source → 4 分类全部走自定义.
    let stub = Arc::new(StubPicker {
        github: vec![PickerCandidate {
            id: "gh:1".into(),
            label: "★ main".into(),
            description: "origin/main".into(),
            kind: SdKind::RemoteBranch,
        }],
        local: vec![PickerCandidate {
            id: "local:/tmp/x".into(),
            label: "x".into(),
            description: "/tmp/x".into(),
            kind: SdKind::RepoBase,
        }],
        empty: PickerCandidate {
            id: "empty:placeholder".into(),
            label: "Empty".into(),
            description: "blank".into(),
            kind: SdKind::CommitSha,
        },
    });
    let svc = InMemoryWorktreeService::new().with_picker_source(stub);

    let repo_id = RepoId::from(Uuid::new_v4());
    let wt = svc.create(repo_id, "main", None).await.unwrap();

    let out = svc.pick_start_from_candidates(repo_id).await.unwrap();
    assert_eq!(out.github_branches.len(), 1);
    assert_eq!(out.existing_worktrees.len(), 1);
    assert_eq!(out.local_paths.len(), 1);
    assert!(out.empty.is_some());

    // 验证 existing candidate 用 worktree_to_existing_candidate 派生
    assert_eq!(out.existing_worktrees[0].id, format!("wt:{}", wt.id));
    assert_eq!(
        out.existing_worktrees[0].kind,
        worktree_service::PickerCandidateKind::LocalBranch
    );
}

#[tokio::test]
async fn integration_git_provider_failure_returns_empty_for_kind_1_only() {
    // 集成: git provider 失败 → 只 kind 1 空, kind 2/3/4 不受影响 (per FR-ORCA-009).
    let git: Arc<dyn GitProvider> = Arc::new(StubGit {
        branches: vec![],
        should_fail: true, // 模拟 git remote 不可用
    });
    let src = DefaultStartFromPickerSource::new(git);
    let repo_id = RepoId::from(Uuid::new_v4());

    std::env::set_var("WORKTREE_LOCAL_PATHS", "/tmp/y");
    let out = pick_start_from_candidates(repo_id, &[], &src).await.unwrap();
    std::env::remove_var("WORKTREE_LOCAL_PATHS");

    assert!(out.github_branches.is_empty(), "kind 1 fails → empty");
    assert!(out.existing_worktrees.is_empty(), "kind 2 from existing → empty");
    assert_eq!(out.local_paths.len(), 1, "kind 3 from env → 1");
    assert!(out.empty.is_some(), "kind 4 always present");
    assert_eq!(out.total(), 2);
}

#[tokio::test]
async fn integration_picker_candidate_kind_mapping_correct() {
    // 集成: 验证 PickerCandidateKind 复用 worktree-shared-dir enum, 不重复定义.
    // 4 来源 → 4 enum variants.
    let git: Arc<dyn GitProvider> = Arc::new(StubGit {
        branches: vec![branch("main", "aaa1111", true)],
        should_fail: false,
    });
    let src = DefaultStartFromPickerSource::new(git);
    let repo_id = RepoId::from(Uuid::new_v4());

    std::env::set_var("WORKTREE_LOCAL_PATHS", "/tmp/z");
    let existing = vec![make_wt(repo_id, "main", false)];
    let out = pick_start_from_candidates(repo_id, &existing, &src)
        .await
        .unwrap();
    std::env::remove_var("WORKTREE_LOCAL_PATHS");

    // kind 1 → RemoteBranch
    assert!(matches!(out.github_branches[0].kind, SdKind::RemoteBranch));
    // kind 2 → LocalBranch
    assert!(matches!(out.existing_worktrees[0].kind, SdKind::LocalBranch));
    // kind 3 → RepoBase
    assert!(matches!(out.local_paths[0].kind, SdKind::RepoBase));
    // kind 4 → CommitSha (sentinel Empty)
    assert!(matches!(out.empty.as_ref().unwrap().kind, SdKind::CommitSha));
}

#[tokio::test]
async fn integration_select_by_id_round_trips_through_all_categories() {
    // 集成: select_by_id 全分类命中 (UI 端 confirm callback 路径).
    let git: Arc<dyn GitProvider> = Arc::new(StubGit {
        branches: vec![branch("main", "aaa1111", true)],
        should_fail: false,
    });
    let src = DefaultStartFromPickerSource::new(git);
    let repo_id = RepoId::from(Uuid::new_v4());

    std::env::set_var("WORKTREE_LOCAL_PATHS", "/tmp/rt");
    let existing = vec![make_wt(repo_id, "main", false)];
    let cands = pick_start_from_candidates(repo_id, &existing, &src)
        .await
        .unwrap();
    std::env::remove_var("WORKTREE_LOCAL_PATHS");

    let gh_id = cands.github_branches[0].id.clone();
    let ex_id = cands.existing_worktrees[0].id.clone();
    let lp_id = cands.local_paths[0].id.clone();
    let em_id = cands.empty.as_ref().unwrap().id.clone();

    use worktree_service::select_by_id;
    assert!(select_by_id(&cands, &gh_id).is_some());
    assert!(select_by_id(&cands, &ex_id).is_some());
    assert!(select_by_id(&cands, &lp_id).is_some());
    assert!(select_by_id(&cands, &em_id).is_some());
    assert!(select_by_id(&cands, "no-such-id").is_none());

    // worktree_to_existing_candidate 字段映射也顺便验证
    let c = worktree_to_existing_candidate(&existing[0]);
    assert_eq!(c.id, format!("wt:{}", existing[0].id));
    assert_eq!(c.label, "main");
    assert!(c.description.contains("main"));
}

// 抑制 unused warning for NoopPickerSource 引用 (re-export 验证)
#[allow(dead_code)]
fn _noop_picker_source_compiles() -> NoopPickerSource {
    NoopPickerSource
}