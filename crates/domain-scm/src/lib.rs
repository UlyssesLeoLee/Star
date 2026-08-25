//! SCM 域(SCM Adapter + Repository Sync)
//!
//! **crate**: `domain-scm`
//! **上游 spec**: docs/specs/domain-scm-spec.md
//! **基本设计**: docs/basic-design.md §4.7
//! **数据设计**: docs/data-design.md §4.18 (`scm` schema)
//! **API 设计**: docs/api-design.md §3.19
//!
//! ## 职责
//!
//! 描述 GitHub / GitLab / Bitbucket SCM 适配器抽象,通过统一 Port 抽象避免 Domain 层出现
//! 厂商特有对象(§4.7.1,§19.1,REQ-SCM-001/002)。包含 Repository 聚合根 + 5 个子实体
//! (Branch / Commit / PullRequest / Review / Pipeline / WebhookEvent) + 1 个值对象 SyncState。
//!
//! ## 关键不变量
//!
//! - INV-SCM-01: Domain 层不出现厂商特有对象(由 ACL 翻译,§4.7.1,REQ-SCM-002)
//! - INV-SCM-02: MVP 仅支持 Connected 所有权(§4.7.4,§30.6)
//! - INV-SCM-03: Bidirectional Sync 必须有 Loop 防护(Idempotency Key + Sync Token,§4.7.6,RISK-027)
//! - INV-SCM-04: Repository 必带 tenant_id + project_id,跨 tenant 拒绝(§6.1,REQ-SEC-001)
//! - INV-SCM-05: Repository Credential 走 Credential Broker,不存明文(§5.4)
//! - INV-SCM-06: PR Content 必带 tenant_id(Object Storage Key 前缀,§6.1)
//! - INV-SCM-07: PullRequest.state 状态机严格按 §7.5 迁移
//! - INV-SCM-08: Webhook 入站 100% 写 Audit(§9.3)
//!
//! ## 上游依赖
//!
//! 本 crate 仅依赖自身外部依赖,无跨 domain-* crate 依赖。

#![allow(missing_docs)]
#![warn(rust_2018_idioms)]

// =====================================================================
// 子模块装载
// =====================================================================

pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariants;
pub mod macros;
pub mod port;
pub mod service;
pub mod value_object;

// =====================================================================
// 便捷 re-export
// =====================================================================

pub use context::ActorContext;
pub use entity::{
    Branch, Commit, Pipeline, PullRequest, Repository, Review, SyncState, WebhookEvent,
};
pub use error::ScmError;
pub use event::{
    BranchCreated, CommitLinked, EventMeta, PullRequestLinked, PullRequestStateChanged,
    RepositoryLinked, RepositoryRegistered, ScmEvent, SyncStateChanged, WebhookReceived,
};
pub use invariants::{
    check_invariant_01_no_vendor_objects_in_domain, check_invariant_02_connected_only,
    check_invariant_03_bidirectional_loop_guard, check_invariant_04_tenant_project_required,
    check_invariant_05_no_plaintext_credential, check_invariant_06_pr_content_tenant,
    check_invariant_07_pr_state_machine, check_invariant_08_webhook_idempotency,
    check_pr_transition_invariants, check_register_invariants, run_invariants,
    ALL_INVARIANT_CHECKS,
};
pub use port::{
    ConfigureWebhookCommand, LinkToProjectCommand, ListBranchesQuery, ListPullRequestQuery,
    ListWebhookEventsQuery, RecordWebhookEventCommand, RegisterRepositoryCommand,
    RotateTokenCommand, ScmCommandPort, ScmPort, ScmQueryPort, ScmRepository,
    TransitionPullRequestCommand, UpdateSyncStateCommand,
};
pub use service::{InMemoryScmPort, InMemoryScmService};
pub use value_object::{
    roles, BranchId, CommitId, ConflictStrategy, ExternalRepositoryId, PipelineId, PipelineStatus,
    ProjectId, PullRequestId, PullRequestState, RepositoryId, RepositoryOwnership, ReviewId,
    ReviewState, ScmProvider, ScmProviderId, SyncStatus, TenantId, UserId, WebhookEventId,
    WebhookEventType, WorkItemId,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{PipelineStatus, ReviewState, ScmProvider};

    // -------- 测试夹具 --------

    fn make_test_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(UserId::new(), tenant_id)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(ProjectId::new())
    }

    fn make_register_cmd(tenant_id: TenantId) -> RegisterRepositoryCommand {
        RegisterRepositoryCommand {
            tenant_id,
            project_id: ProjectId::new(),
            provider: ScmProvider::Github,
            external_id: ExternalRepositoryId::new("acme/foo"),
            url: "https://github.com/acme/foo".to_string(),
            default_branch: "main".to_string(),
            ownership: RepositoryOwnership::Connected,
            credential_id: None,
        }
    }

    fn make_pr(tenant_id: TenantId, repository_id: RepositoryId) -> PullRequest {
        let now = chrono::Utc::now();
        PullRequest {
            id: PullRequestId::new(),
            repository_id,
            tenant_id,
            external_id: "42".to_string(),
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            title: "Test PR".to_string(),
            description: Some("Description".to_string()),
            author_user_id: Some(UserId::new()),
            state: PullRequestState::Draft,
            linked_work_item_id: None,
            review_ids: vec![],
            pipeline_ids: vec![],
            merged_at: None,
            merged_by_user_id: None,
            created_at: now,
            updated_at: now,
            closed_at: None,
            lock_version: 1,
        }
    }

    // -------- 1. ActorContext + 强类型 ID smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        assert!(!actor.tenant_id.as_uuid().is_nil());
        assert!(actor.has_role(roles::PROJECT_ADMIN));
        assert!(!actor.user_id.as_uuid().is_nil());
    }

    // -------- 2. 字段数审计 --------

    #[test]
    fn field_count_audit() {
        assert_eq!(Repository::FIELD_COUNT, 17);
        assert_eq!(Branch::FIELD_COUNT, 11);
        assert_eq!(Commit::FIELD_COUNT, 13);
        assert_eq!(PullRequest::FIELD_COUNT, 19);
        assert_eq!(Review::FIELD_COUNT, 9);
        assert_eq!(Pipeline::FIELD_COUNT, 10);
        assert_eq!(WebhookEvent::FIELD_COUNT, 12);
    }

    // -------- 3. create_repository 成功路径 --------

    #[tokio::test]
    async fn register_repository_success() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_register_cmd(tenant_id);
        let repo = svc
            .register_repository(cmd, actor)
            .await
            .expect("注册成功");
        assert_eq!(svc.count_repositories().await, 1);
        assert!(repo.is_read_only()); // Connected
        assert_eq!(repo.lock_version, 1);
        assert_eq!(repo.provider, ScmProvider::Github);
    }

    // -------- 4. INV-SCM-02:非 Connected Ownership 被拒 --------

    #[tokio::test]
    async fn invariant_02_managed_ownership_rejected() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let mut cmd = make_register_cmd(tenant_id);
        cmd.ownership = RepositoryOwnership::Managed; // MVP 阶段拒绝
        let res = svc.register_repository(cmd, actor).await;
        assert!(matches!(res, Err(ScmError::InvalidState(_))));
    }

    // -------- 5. INV-SCM-04:跨租户访问 Repository 被拒 --------

    #[tokio::test]
    async fn invariant_04_cross_tenant_access_denied() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();
        let actor_a = make_test_actor(tenant_a);
        let cmd = make_register_cmd(tenant_a);
        let repo = svc
            .register_repository(cmd, actor_a.clone())
            .await
            .unwrap();

        // tenant_b 试图读 repository
        let viewer_b = ActorContext::new(UserId::new(), tenant_b)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(ProjectId::new());
        let res = svc.get_repository(repo.id, viewer_b).await;
        assert!(matches!(res, Err(ScmError::PermissionDenied)));
    }

    // -------- 6. INV-SCM-05:URL 含明文凭据被拒 --------

    #[tokio::test]
    async fn invariant_05_url_with_plaintext_credential_rejected() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let mut cmd = make_register_cmd(tenant_id);
        // URL 含用户名密码 → 违反 INV-SCM-05
        cmd.url = "https://user:pass@github.com/acme/foo".to_string();
        let res = svc.register_repository(cmd, actor).await;
        assert!(matches!(res, Err(ScmError::InvalidState(_))));
    }

    // -------- 7. INV-SCM-07:PR 状态机非法迁移被拒 --------

    #[tokio::test]
    async fn invariant_07_invalid_pr_transition_rejected() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_register_cmd(tenant_id);
        let repo = svc.register_repository(cmd, actor.clone()).await.unwrap();
        let pr = make_pr(tenant_id, repo.id);
        svc.seed_pull_request(pr.clone()).await;

        // Draft → Merged(非法:必须经过 OPEN/REVIEWING/...)
        let res = svc
            .transition_pull_request(
                TransitionPullRequestCommand {
                    pull_request_id: pr.id,
                    repository_id: repo.id,
                    tenant_id,
                    next_state: PullRequestState::Merged,
                    triggered_by: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(ScmError::InvalidState(_))));
    }

    // -------- 8. INV-SCM-07:PR 状态机合法迁移通过 --------

    #[tokio::test]
    async fn pr_state_machine_legal_transition_works() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_register_cmd(tenant_id);
        let repo = svc.register_repository(cmd, actor.clone()).await.unwrap();
        let pr = make_pr(tenant_id, repo.id);
        svc.seed_pull_request(pr.clone()).await;

        // Draft → Open → Reviewing → Approved → Mergeable → Merged
        for next in [
            PullRequestState::Open,
            PullRequestState::Reviewing,
            PullRequestState::Approved,
            PullRequestState::Mergeable,
            PullRequestState::Merged,
        ] {
            let res = svc
                .transition_pull_request(
                    TransitionPullRequestCommand {
                        pull_request_id: pr.id,
                        repository_id: repo.id,
                        tenant_id,
                        next_state: next,
                        triggered_by: None,
                    },
                    actor.clone(),
                )
                .await;
            assert!(res.is_ok(), "迁移 {:?} 失败: {:?}", next, res);
        }

        let stored = svc
            .find_pull_request_by_id(pr.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.state, PullRequestState::Merged);
        assert!(stored.merged_at.is_some());
    }

    // -------- 9. INV-SCM-08:重复 Webhook 事件被拒(SC-004) --------

    #[tokio::test]
    async fn invariant_08_duplicate_webhook_rejected() {
        let svc = InMemoryScmService::new_for_test();
        // 第一次入站
        let cmd1 = RecordWebhookEventCommand {
            provider: ScmProvider::Github,
            event_type: WebhookEventType::Push,
            payload: r#"{"head":"abc"}"#.to_string(),
            signature: Some("sha256=abc".to_string()),
            idempotency_key: Some("delivery-12345".to_string()),
        };
        let evt = svc.record_webhook_event(cmd1).await.expect("首次入站成功");
        assert!(!evt.is_processed);

        // 第二次入站(同一 idempotency_key)→ SC-004 Conflict
        let cmd2 = RecordWebhookEventCommand {
            provider: ScmProvider::Github,
            event_type: WebhookEventType::Push,
            payload: r#"{"head":"abc"}"#.to_string(),
            signature: Some("sha256=abc".to_string()),
            idempotency_key: Some("delivery-12345".to_string()),
        };
        let res = svc.record_webhook_event(cmd2).await;
        assert!(matches!(res, Err(ScmError::Conflict(_))));
    }

    // -------- 10. update_sync_state 成功路径 + 事件 ----

    #[tokio::test]
    async fn update_sync_state_success() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_register_cmd(tenant_id);
        let repo = svc.register_repository(cmd, actor.clone()).await.unwrap();

        // 设置为 Behind 状态
        let res = svc
            .update_sync_state(
                UpdateSyncStateCommand {
                    repository_id: repo.id,
                    tenant_id,
                    sync_status: SyncStatus::Behind,
                    sync_token: Some("etag-abc".to_string()),
                    synced_at: chrono::Utc::now(),
                },
                actor,
            )
            .await
            .expect("更新成功");
        assert_eq!(res.sync_status, SyncStatus::Behind);
        assert_eq!(res.sync_token.as_deref(), Some("etag-abc"));
        assert!(res.last_synced_at.is_some());
        assert_eq!(res.lock_version, 2);
    }

    // -------- 11. link_to_project 成功路径 ----

    #[tokio::test]
    async fn link_to_project_success() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_register_cmd(tenant_id);
        let repo = svc.register_repository(cmd, actor.clone()).await.unwrap();
        let new_project = ProjectId::new();

        let res = svc
            .link_to_project(
                LinkToProjectCommand {
                    repository_id: repo.id,
                    tenant_id,
                    project_id: new_project,
                },
                actor,
            )
            .await
            .expect("关联成功");
        assert_eq!(res.project_id, new_project);
    }

    // -------- 12. list_branches 过滤(protected_only) ----

    #[tokio::test]
    async fn list_branches_filter_protected() {
        let svc = InMemoryScmService::new_for_test();
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();
        let actor = ActorContext::new(UserId::new(), tenant_id)
            .with_role(roles::PROJECT_ADMIN)
            .with_project(project_id);
        let mut cmd = make_register_cmd(tenant_id);
        cmd.project_id = project_id; // 与 actor 的 project_ids 对齐
        let repo = svc.register_repository(cmd, actor.clone()).await.unwrap();

        // 注入 2 个 Branch:1 protected,1 not
        let now = chrono::Utc::now();
        let b1 = Branch {
            id: BranchId::new(),
            repository_id: repo.id,
            tenant_id,
            name: "main".to_string(),
            head_commit_id: None,
            base_commit_id: None,
            is_protected: true,
            is_default: true,
            created_at: now,
            updated_at: now,
            lock_version: 1,
        };
        let b2 = Branch {
            id: BranchId::new(),
            repository_id: repo.id,
            tenant_id,
            name: "feature/foo".to_string(),
            head_commit_id: None,
            base_commit_id: None,
            is_protected: false,
            is_default: false,
            created_at: now,
            updated_at: now,
            lock_version: 1,
        };
        svc.seed_branch(b1).await;
        svc.seed_branch(b2).await;

        // 全部
        let all = svc
            .list_branches(
                ListBranchesQuery {
                    tenant_id,
                    repository_id: repo.id,
                    protected_only: false,
                },
                actor.clone(),
            )
            .await
            .unwrap();
        assert_eq!(all.len(), 2);

        // 仅 protected
        let prot = svc
            .list_branches(
                ListBranchesQuery {
                    tenant_id,
                    repository_id: repo.id,
                    protected_only: true,
                },
                actor,
            )
            .await
            .unwrap();
        assert_eq!(prot.len(), 1);
        assert!(prot[0].is_protected);
    }

    // -------- 13. 事件总线烟囱测试 + ScmEvent subject 校验 ----

    #[tokio::test]
    async fn event_bus_receives_registered() {
        let (svc, mut rx) = InMemoryScmService::new();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_register_cmd(tenant_id);
        svc.register_repository(cmd, actor).await.unwrap();

        // 第一个事件应为 RepositoryRegistered
        let evt = rx.try_recv().expect("应收到事件");
        assert!(matches!(evt, ScmEvent::RepositoryRegistered(_)));
        assert_eq!(evt.subject(), "star.events.scm.repository.registered.v1");
    }

    // -------- 14. (额外)SyncStateChanged 事件 subject 校验 ----

    #[tokio::test]
    async fn event_sync_state_changed_subject() {
        let (svc, mut rx) = InMemoryScmService::new();
        let tenant_id = TenantId::new();
        let actor = make_test_actor(tenant_id);
        let cmd = make_register_cmd(tenant_id);
        let repo = svc.register_repository(cmd, actor.clone()).await.unwrap();
        // 消费掉 register 事件
        let _ = rx.try_recv();

        svc.update_sync_state(
            UpdateSyncStateCommand {
                repository_id: repo.id,
                tenant_id,
                sync_status: SyncStatus::Behind,
                sync_token: Some("tok-1".to_string()),
                synced_at: chrono::Utc::now(),
            },
            actor,
        )
        .await
        .unwrap();

        let evt = rx.try_recv().expect("应收到 sync_state.changed 事件");
        assert!(matches!(evt, ScmEvent::SyncStateChanged(_)));
        assert_eq!(evt.subject(), "star.events.scm.sync_state.changed.v1");
    }

    // -------- 15. (额外)Review / Pipeline 状态枚举字面量校验 ----

    #[test]
    fn review_pipeline_state_as_str() {
        assert_eq!(ReviewState::Approved.as_str(), "APPROVED");
        assert_eq!(ReviewState::ChangesRequested.as_str(), "CHANGES_REQUESTED");
        assert_eq!(ReviewState::Commented.as_str(), "COMMENTED");
        assert_eq!(ReviewState::Dismissed.as_str(), "DISMISSED");
        assert_eq!(PipelineStatus::Success.as_str(), "SUCCESS");
        assert_eq!(PipelineStatus::Failed.as_str(), "FAILED");
    }

    // -------- 16. (额外)ScmProvider 字面量校验 + FromStr 解析 ----

    #[test]
    fn scm_provider_str_roundtrip() {
        use std::str::FromStr;
        for s in ["github", "gitlab", "gitea", "bitbucket"] {
            let p = ScmProvider::from_str(s).expect("解析");
            assert_eq!(p.as_str(), s);
        }
        let bad = ScmProvider::from_str("invalid");
        assert!(bad.is_err());
    }
}
