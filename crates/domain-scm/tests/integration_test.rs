//! 集成测试 (IT) — domain-scm 真实数据接入验证
//!
//! **crate**: domain-scm (test target)
//! **per**: 2026-09-07 P3-B sub-batch 3 业务核心 域真实数据接入
//! **参考**: `crates/domain-workspace/tests/integration_test.rs` (per 7f9f52a)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): register_repository + get_repository + list_by_project + update_sync_state
//! - V1 (PR 状态机 + create_mr): Draft → Open → Reviewing → Approved → Mergeable → Merged
//! - V2 (RLS 13 類 守门 #13 c): 跨 tenant 拒绝 (register + get + list)
//! - V3 (Webhook 幂等 INV-SCM-03 + 事件总线): record_webhook_event + IdempotencyConflict + ScmEvent subject
//!
//! ## W/T/M 分类 (per 守门 #13)
//!
//! - **Repository / Branch / Commit / PR / Review / Pipeline / Policy**: M 类 Master (SCD Type 2)
//! - **WebhookEvent**: T 类 Transaction (Append-only, INV-SCM-08)

use domain_scm::{
    ActorContext, ConflictStrategy, CreateMRInput, InMemoryScmService, ProjectId,
    PullRequestState, RegisterRepositoryCommand, RepositoryOwnership, ScmCommandPort, ScmError,
    ScmEvent, ScmProvider, ScmQueryPort, SyncStatus, TenantId, UpdateSyncStateCommand,
    WebhookEventInput, WebhookEventType,
};
use uuid::Uuid;

/// IT fixture: project_admin 角色 + 关联 project
fn make_project_admin(tenant_id: Uuid, project_id: ProjectId) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id)
        .with_role("project_admin")
        .with_project(project_id.as_uuid())
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: register → get → list_by_project 完整闭环 + lock_version=1
#[tokio::test]
async fn it_v0_register_get_list_lifecycle() {
    let svc = InMemoryScmService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_project_admin(tenant_id, project_id);

    let cmd = RegisterRepositoryCommand {
        tenant_id: TenantId(tenant_id),
        project_id,
        provider: ScmProvider::Github,
        external_id: "acme/foo".to_string(),
        url: "https://github.com/acme/foo".to_string(),
        default_branch: "main".to_string(),
        ownership: RepositoryOwnership::Connected,
        conflict_strategy: ConflictStrategy::LatestWins,
        credential_id: Some(Uuid::new_v4()),
    };
    let repo = svc.register_repository(cmd, actor.clone()).await.unwrap();
    assert_eq!(repo.lock_version, 1);
    assert_eq!(repo.provider, ScmProvider::Github);
    assert_eq!(repo.sync_status, SyncStatus::InSync);

    // get_repository
    let fetched = svc.get_repository(repo.id, actor.clone()).await.unwrap();
    assert_eq!(fetched.id, repo.id);
    assert_eq!(fetched.external_id.0, "acme/foo");

    // list_by_project
    let list = svc
        .list_by_project(project_id, actor)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, repo.id);
    assert_eq!(svc.repo_count().await, 1);
}

/// **IT-V0-2**: update_sync_state 推进 lock_version + sync_token
#[tokio::test]
async fn it_v0_update_sync_state_advances_version() {
    let svc = InMemoryScmService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_project_admin(tenant_id, project_id);

    let repo = svc
        .register_repository(
            RegisterRepositoryCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                provider: ScmProvider::Gitlab,
                external_id: "team/bar".to_string(),
                url: "https://gitlab.com/team/bar".to_string(),
                default_branch: "main".to_string(),
                ownership: RepositoryOwnership::Connected,
                conflict_strategy: ConflictStrategy::ManualReview,
                credential_id: Some(Uuid::new_v4()),
            },
            actor.clone(),
        )
        .await
        .unwrap();
    assert_eq!(repo.lock_version, 1);

    // 推进 sync 状态: InSync -> Behind
    let r2 = svc
        .update_sync_state(
            UpdateSyncStateCommand {
                repository_id: repo.id,
                tenant_id: TenantId(tenant_id),
                new_status: SyncStatus::Behind,
                new_token: Some("etag-xyz".to_string()),
            },
            actor,
        )
        .await
        .unwrap();
    assert_eq!(r2.lock_version, 2);
    assert_eq!(r2.sync_status, SyncStatus::Behind);
    assert_eq!(r2.sync_token.as_deref(), Some("etag-xyz"));
    assert!(r2.last_synced_at.is_some());
}

/// **IT-V0-3**: 同 (tenant, provider, external_id) 重复注册被 Conflict 拒绝
#[tokio::test]
async fn it_v0_duplicate_register_conflict() {
    let svc = InMemoryScmService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_project_admin(tenant_id, project_id);

    let base = || RegisterRepositoryCommand {
        tenant_id: TenantId(tenant_id),
        project_id,
        provider: ScmProvider::Github,
        external_id: "acme/dup".to_string(),
        url: "https://github.com/acme/dup".to_string(),
        default_branch: "main".to_string(),
        ownership: RepositoryOwnership::Connected,
        conflict_strategy: ConflictStrategy::LatestWins,
        credential_id: Some(Uuid::new_v4()),
    };
    svc.register_repository(base(), actor.clone()).await.unwrap();
    let res = svc.register_repository(base(), actor).await;
    assert!(matches!(res, Err(ScmError::Conflict(_))));
}

// =====================================================================
// V1: PR 状态机 + create_mr helper
// =====================================================================

/// **IT-V1-1**: create_mr 走 helper → state=Draft → transition Draft→Open→Reviewing→Approved→Mergeable→Merged
#[tokio::test]
async fn it_v1_pr_state_machine_full_lifecycle() {
    let svc = InMemoryScmService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_project_admin(tenant_id, project_id);

    let repo = svc
        .register_repository(
            RegisterRepositoryCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                provider: ScmProvider::Github,
                external_id: "team/pr-test".to_string(),
                url: "https://github.com/team/pr-test".to_string(),
                default_branch: "main".to_string(),
                ownership: RepositoryOwnership::Connected,
                conflict_strategy: ConflictStrategy::LatestWins,
                credential_id: Some(Uuid::new_v4()),
            },
            actor.clone(),
        )
        .await
        .unwrap();

    // create_mr helper 创 Draft
    let pr = svc
        .create_mr(
            CreateMRInput {
                tenant_id: TenantId(tenant_id),
                repository_id: repo.id,
                title: "feat: add WIP".to_string(),
                description: Some("draft PR".to_string()),
                base: "main".to_string(),
                head: "feat/wip".to_string(),
            },
            actor.clone(),
        )
        .await
        .unwrap();
    assert_eq!(pr.state, PullRequestState::Draft);
    assert_eq!(pr.source_branch, "feat/wip");
    assert_eq!(pr.target_branch, "main");
    assert_eq!(svc.pr_count().await, 1);

    // 链: Draft -> Open
    let pr = svc
        .transition_pull_request(domain_scm::RecordPullRequestTransitionCommand {
            pr_id: pr.id,
            new_state: PullRequestState::Open,
            actor: actor.clone(),
        })
        .await
        .unwrap();
    assert_eq!(pr.state, PullRequestState::Open);

    // Open -> Reviewing
    let pr = svc
        .transition_pull_request(domain_scm::RecordPullRequestTransitionCommand {
            pr_id: pr.id,
            new_state: PullRequestState::Reviewing,
            actor: actor.clone(),
        })
        .await
        .unwrap();
    assert_eq!(pr.state, PullRequestState::Reviewing);

    // Reviewing -> Approved (INV-SCM-07 严格迁移)
    let pr = svc
        .transition_pull_request(domain_scm::RecordPullRequestTransitionCommand {
            pr_id: pr.id,
            new_state: PullRequestState::Approved,
            actor: actor.clone(),
        })
        .await
        .unwrap();
    assert_eq!(pr.state, PullRequestState::Approved);

    // Approved -> Mergeable -> Merged
    let pr = svc
        .transition_pull_request(domain_scm::RecordPullRequestTransitionCommand {
            pr_id: pr.id,
            new_state: PullRequestState::Mergeable,
            actor: actor.clone(),
        })
        .await
        .unwrap();
    assert_eq!(pr.state, PullRequestState::Mergeable);

    let pr = svc
        .transition_pull_request(domain_scm::RecordPullRequestTransitionCommand {
            pr_id: pr.id,
            new_state: PullRequestState::Merged,
            actor,
        })
        .await
        .unwrap();
    assert_eq!(pr.state, PullRequestState::Merged);
    assert!(pr.state.is_terminal());
}

/// **IT-V1-2**: 终态 Merged 不可再迁移(INV-SCM-07 严格状态机)
#[tokio::test]
async fn it_v1_terminal_state_rejects_transition() {
    let svc = InMemoryScmService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_project_admin(tenant_id, project_id);

    let repo = svc
        .register_repository(
            RegisterRepositoryCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                provider: ScmProvider::Github,
                external_id: "team/term".to_string(),
                url: "u".to_string(),
                default_branch: "main".to_string(),
                ownership: RepositoryOwnership::Connected,
                conflict_strategy: ConflictStrategy::LatestWins,
                credential_id: Some(Uuid::new_v4()),
            },
            actor.clone(),
        )
        .await
        .unwrap();

    let pr = svc
        .create_mr(
            CreateMRInput {
                tenant_id: TenantId(tenant_id),
                repository_id: repo.id,
                title: "T".to_string(),
                description: None,
                base: "main".to_string(),
                head: "f".to_string(),
            },
            actor.clone(),
        )
        .await
        .unwrap();
    // 直接 Closed
    let pr = svc
        .transition_pull_request(domain_scm::RecordPullRequestTransitionCommand {
            pr_id: pr.id,
            new_state: PullRequestState::Closed,
            actor: actor.clone(),
        })
        .await
        .unwrap();
    assert!(pr.state.is_terminal());
    // Closed -> Open 必拒
    let res = svc
        .transition_pull_request(domain_scm::RecordPullRequestTransitionCommand {
            pr_id: pr.id,
            new_state: PullRequestState::Open,
            actor,
        })
        .await;
    assert!(matches!(res, Err(ScmError::InvalidState(_))));
}

// =====================================================================
// V2: RLS 13 類 跨 tenant 拒绝 (守门 #13 c)
// =====================================================================

/// **IT-V2-1**: 跨 tenant get_repository 拒绝
#[tokio::test]
async fn it_v2_cross_tenant_get_rejected() {
    let svc = InMemoryScmService::new_for_test();
    let tenant_a = Uuid::new_v4();
    let project_a = ProjectId::new();
    let actor_a = make_project_admin(tenant_a, project_a);

    let repo = svc
        .register_repository(
            RegisterRepositoryCommand {
                tenant_id: TenantId(tenant_a),
                project_id: project_a,
                provider: ScmProvider::Github,
                external_id: "iso/a".to_string(),
                url: "u".to_string(),
                default_branch: "main".to_string(),
                ownership: RepositoryOwnership::Connected,
                conflict_strategy: ConflictStrategy::LatestWins,
                credential_id: Some(Uuid::new_v4()),
            },
            actor_a,
        )
        .await
        .unwrap();

    // tenant B 试图读
    let tenant_b = Uuid::new_v4();
    let project_b = ProjectId::new();
    let actor_b = make_project_admin(tenant_b, project_b);
    let res = svc.get_repository(repo.id, actor_b).await;
    assert!(matches!(res, Err(ScmError::PermissionDenied(_))));
}

/// **IT-V2-2**: 跨 tenant list_by_project 隔离(空 list)
#[tokio::test]
async fn it_v2_cross_tenant_list_isolated() {
    let svc = InMemoryScmService::new_for_test();
    let tenant_a = Uuid::new_v4();
    let project_a = ProjectId::new();
    let actor_a = make_project_admin(tenant_a, project_a);

    svc.register_repository(
        RegisterRepositoryCommand {
            tenant_id: TenantId(tenant_a),
            project_id: project_a,
            provider: ScmProvider::Github,
            external_id: "iso/list".to_string(),
            url: "u".to_string(),
            default_branch: "main".to_string(),
            ownership: RepositoryOwnership::Connected,
            conflict_strategy: ConflictStrategy::LatestWins,
            credential_id: Some(Uuid::new_v4()),
        },
        actor_a,
    )
    .await
    .unwrap();

    // tenant B 列同一 project_id 应为空
    let tenant_b = Uuid::new_v4();
    let project_b = ProjectId::new();
    let actor_b = make_project_admin(tenant_b, project_b);
    let list = svc.list_by_project(project_a, actor_b).await.unwrap();
    assert_eq!(list.len(), 0);
}

// =====================================================================
// V3: Webhook 幂等 (INV-SCM-03) + 事件总线
// =====================================================================

/// **IT-V3-1**: record_webhook_event 幂等 + WebhookReceived 事件 subject
#[tokio::test]
async fn it_v3_webhook_idempotency_and_event() {
    let (svc, mut rx) = InMemoryScmService::new();
    let tenant_id = Uuid::new_v4();

    let input = WebhookEventInput {
        tenant_id: TenantId(tenant_id),
        repository_id: None,
        provider: ScmProvider::Github,
        event_type: WebhookEventType::Push,
        external_event_id: "gh-evt-9999".to_string(),
        raw_payload: serde_json::json!({"ref": "refs/heads/main"}),
    };
    let we = svc.record_webhook_event(input.clone()).await.unwrap();
    assert_eq!(svc.webhook_count().await, 1);
    assert!(!we.processed);
    assert!(!we.loop_breaker_id.is_nil());

    // 收到 WebhookReceived 事件
    let evt = rx.try_recv().expect("应收到 WebhookReceived 事件");
    assert!(matches!(evt, ScmEvent::WebhookReceived(_)));
    assert_eq!(evt.subject(), "star.events.scm.webhook.received.v1");

    // 重复 external_event_id → IdempotencyConflict (INV-SCM-03 Loop 防护)
    let res = svc.record_webhook_event(input).await;
    assert!(matches!(res, Err(ScmError::IdempotencyConflict)));
    // 计数仍为 1 (没新增)
    assert_eq!(svc.webhook_count().await, 1);
}

/// **IT-V3-2**: register_repository 触发 RepositoryRegistered 事件 subject 校验
#[tokio::test]
async fn it_v3_register_repository_event_subject() {
    let (svc, mut rx) = InMemoryScmService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_project_admin(tenant_id, project_id);

    svc.register_repository(
        RegisterRepositoryCommand {
            tenant_id: TenantId(tenant_id),
            project_id,
            provider: ScmProvider::Github,
            external_id: "evt/repo".to_string(),
            url: "u".to_string(),
            default_branch: "main".to_string(),
            ownership: RepositoryOwnership::Connected,
            conflict_strategy: ConflictStrategy::LatestWins,
            credential_id: Some(Uuid::new_v4()),
        },
        actor,
    )
    .await
    .unwrap();

    let evt = rx.try_recv().expect("应收到 RepositoryRegistered 事件");
    assert!(matches!(evt, ScmEvent::RepositoryRegistered(_)));
    assert_eq!(evt.subject(), "star.events.scm.repository.registered.v1");
}
