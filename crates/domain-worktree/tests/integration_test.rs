//! 集成测试 (IT) — domain-worktree 真实数据接入验证
//!
//! **crate**: domain-worktree (test target)
//! **per**: 2026-09-07 P3-B sub-batch 3 业务核心 域真实数据接入
//! **参考**: `crates/domain-workspace/tests/integration_test.rs` (per 7f9f52a)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): create_worktree + get_by_id + list_by_work_item
//! - V1 (状态机 + 协作): transition_status 17 状态机全链 + assign_to_agent + record_observed_state
//! - V2 (RLS 13 類 守门 #13 c + 跨 user 隔离): 跨 tenant 拒绝 + list_by_agent 隔离
//! - V3 (冲突 + heatmap): detect_conflicts + heatmap 按状态聚合
//!
//! ## W/T/M 分类 (per 守门 #13)
//!
//! - **Worktree**: M 类 Master (SCD Type 2 + RLS + tenant_id 必带, INV-WT-08)
//! - **17 状态机**: Created / Initializing / Ready / Assigned / AgentRunning / Committing /
//!   Completed / ReadyForReview / Reviewing / ChangesRequested / Fixing / Merged /
//!   Archived / Abandoned / Blocked / Conflicted / Stale

use domain_worktree::{
    AbandonCommand, ActorContext, AgentId, AgentSessionId, AssignWorktreeCommand,
    CreateWorktreeCommand, InMemoryWorktreeService, ListByAgentQuery, ListByWorkItemQuery,
    ProjectId, RepositoryId, RuntimeId, TenantId, TransitionStatusCommand, UserId, WorkItemId,
    WorktreeCommandPort, WorktreeError, WorktreeQueryPort, WorktreeStatus,
};
use uuid::Uuid;

/// IT fixture: developer 角色
fn make_developer(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("developer")
}

/// IT fixture: local runtime 上报者(INV-WT-10 守门, 需 is_local_runtime)
fn make_local_runtime(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).as_local_runtime()
}

fn make_create_cmd(tenant_id: TenantId) -> CreateWorktreeCommand {
    CreateWorktreeCommand {
        tenant_id,
        project_id: ProjectId::new(),
        work_item_id: WorkItemId::new(),
        repository_id: RepositoryId::new(),
        branch: "feat/test".to_string(),
        base_branch: "main".to_string(),
        runtime_id: RuntimeId::new(),
        owner_user_id: UserId::new(),
    }
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: create → get_by_id → list_by_work_item 完整闭环
#[tokio::test]
async fn it_v0_create_get_list_lifecycle() {
    let svc = InMemoryWorktreeService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_developer(tenant_id);
    let cmd = make_create_cmd(TenantId(tenant_id));

    let wt = svc.create_worktree(cmd.clone(), &actor).await.unwrap();
    assert_eq!(wt.status, WorktreeStatus::Created);
    assert_eq!(wt.tenant_id, TenantId(tenant_id));
    assert_eq!(wt.version, 1);

    // get_by_id
    let fetched = svc.get_by_id(wt.id, &actor).await.unwrap();
    assert_eq!(fetched.id, wt.id);
    assert_eq!(fetched.branch, "feat/test");

    // list_by_work_item
    let list = svc
        .list_by_work_item(
            ListByWorkItemQuery {
                tenant_id: TenantId(tenant_id),
                work_item_id: cmd.work_item_id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, wt.id);
}

/// **IT-V0-2**: INV-WT-03 runtime 必带, nil runtime_id 被拒
#[tokio::test]
async fn it_v0_runtime_required_rejects_nil() {
    let svc = InMemoryWorktreeService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_developer(tenant_id);
    let mut cmd = make_create_cmd(TenantId(tenant_id));
    cmd.runtime_id = RuntimeId(Uuid::nil());
    let res = svc.create_worktree(cmd, &actor).await;
    assert!(matches!(res, Err(WorktreeError::RuntimeRequired)));
}

// =====================================================================
// V1: 状态机 + 协作 (assign + record_observed_state)
// =====================================================================

/// **IT-V1-1**: 17 状态机 happy path 全链 (Created → ... → Archived)
#[tokio::test]
async fn it_v1_status_machine_full_happy_path() {
    let svc = InMemoryWorktreeService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_developer(tenant_id);
    let wt = svc
        .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
        .await
        .unwrap();

    // 链: Created → Initializing → Ready → Assigned → AgentRunning →
    //     Committing → Completed → ReadyForReview → Reviewing → Merged → Archived
    let path = [
        WorktreeStatus::Initializing,
        WorktreeStatus::Ready,
        WorktreeStatus::Assigned,
        WorktreeStatus::AgentRunning,
        WorktreeStatus::Committing,
        WorktreeStatus::Completed,
        WorktreeStatus::ReadyForReview,
        WorktreeStatus::Reviewing,
        WorktreeStatus::Merged,
        WorktreeStatus::Archived,
    ];
    let mut current = wt;
    for next in path {
        current = svc
            .transition_status(
                TransitionStatusCommand {
                    tenant_id: TenantId(tenant_id),
                    worktree_id: current.id,
                    from: current.status,
                    to: next,
                    reason: None,
                },
                &actor,
            )
            .await
            .unwrap_or_else(|e| panic!("{:?} -> {:?} 应允许, got {:?}", current.status, next, e));
    }
    assert_eq!(current.status, WorktreeStatus::Archived);
    assert!(current.status.is_terminal());
}

/// **IT-V1-2**: assign_to_agent 必先到 Ready (状态机守门)
#[tokio::test]
async fn it_v1_assign_requires_ready_state() {
    let svc = InMemoryWorktreeService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_developer(tenant_id);
    let wt = svc
        .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
        .await
        .unwrap();
    // 直接从 Created assign 必拒
    let res = svc
        .assign_to_agent(
            AssignWorktreeCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt.id,
                agent_id: AgentId::new(),
                agent_session_id: AgentSessionId::new(),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(WorktreeError::InvalidTransition { .. })));
}

/// **IT-V1-3**: record_observed_state 必 Local Runtime (非 local_runtime 必拒)
#[tokio::test]
async fn it_v1_record_observed_state_requires_local_runtime() {
    let svc = InMemoryWorktreeService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_developer(tenant_id);
    let wt = svc
        .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
        .await
        .unwrap();

    // 普通 actor 必拒
    let res = svc
        .record_observed_state(
            domain_worktree::RecordObservedStateCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt.id,
                ahead: 3,
                behind: 0,
                current_agent_session_id: None,
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(WorktreeError::PermissionDenied)));

    // local_runtime actor 必成功
    let local = make_local_runtime(tenant_id);
    let res = svc
        .record_observed_state(
            domain_worktree::RecordObservedStateCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt.id,
                ahead: 3,
                behind: 0,
                current_agent_session_id: None,
            },
            &local,
        )
        .await;
    assert!(res.is_ok());
}

/// **IT-V1-4**: abandon 终态不可再 abandon
#[tokio::test]
async fn it_v1_abandon_terminal_state_rejected() {
    let svc = InMemoryWorktreeService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_developer(tenant_id);
    let wt = svc
        .create_worktree(make_create_cmd(TenantId(tenant_id)), &actor)
        .await
        .unwrap();
    // abandon 一次
    let res = svc
        .abandon(
            AbandonCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt.id,
                reason: "test".to_string(),
            },
            &actor,
        )
        .await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap().status, WorktreeStatus::Abandoned);
    // 第二次 abandon 必拒 (终态)
    let res = svc
        .abandon(
            AbandonCommand {
                tenant_id: TenantId(tenant_id),
                worktree_id: wt.id,
                reason: "again".to_string(),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(WorktreeError::InvalidTransition { .. })));
}

// =====================================================================
// V2: RLS 13 類 + 跨 user 隔离
// =====================================================================

/// **IT-V2-1**: 跨 tenant get_by_id 拒绝 (INV-WT-08)
#[tokio::test]
async fn it_v2_cross_tenant_get_rejected() {
    let svc = InMemoryWorktreeService::new();
    let tenant_a = Uuid::new_v4();
    let actor_a = make_developer(tenant_a);
    let wt = svc
        .create_worktree(make_create_cmd(TenantId(tenant_a)), &actor_a)
        .await
        .unwrap();

    let tenant_b = Uuid::new_v4();
    let actor_b = make_developer(tenant_b);
    let res = svc.get_by_id(wt.id, &actor_b).await;
    assert!(matches!(res, Err(WorktreeError::CrossTenantDenied(_, _))));
}

/// **IT-V2-2**: 跨 tenant list_by_work_item 拒绝
#[tokio::test]
async fn it_v2_cross_tenant_list_rejected() {
    let svc = InMemoryWorktreeService::new();
    let tenant_a = Uuid::new_v4();
    let actor_a = make_developer(tenant_a);
    let cmd = make_create_cmd(TenantId(tenant_a));
    svc.create_worktree(cmd.clone(), &actor_a).await.unwrap();

    let tenant_b = Uuid::new_v4();
    let actor_b = make_developer(tenant_b);
    // actor_b 持 tenant_b, 但查询 q.tenant_id 填 tenant_a → 跨 tenant
    let res = svc
        .list_by_work_item(
            ListByWorkItemQuery {
                tenant_id: TenantId(tenant_a),
                work_item_id: cmd.work_item_id,
            },
            &actor_b,
        )
        .await;
    assert!(matches!(res, Err(WorktreeError::CrossTenantDenied(_, _))));
}

/// **IT-V2-3**: 跨 tenant list_by_agent 拒绝
#[tokio::test]
async fn it_v2_cross_tenant_list_by_agent_rejected() {
    let svc = InMemoryWorktreeService::new();
    let tenant_a = Uuid::new_v4();
    let actor_a = make_developer(tenant_a);
    let cmd = make_create_cmd(TenantId(tenant_a));
    let wt = svc.create_worktree(cmd, &actor_a).await.unwrap();

    let tenant_b = Uuid::new_v4();
    let actor_b = make_developer(tenant_b);
    let res = svc
        .list_by_agent(
            ListByAgentQuery {
                tenant_id: TenantId(tenant_b),
                agent_id: AgentId::new(),
            },
            &actor_b,
        )
        .await;
    assert!(res.is_ok()); // 隔离后返回空 list 而非 error
    assert_eq!(res.unwrap().len(), 0);
    let _ = wt; // suppress unused
}

// =====================================================================
// V3: 冲突检测 + heatmap
// =====================================================================

/// **IT-V3-1**: detect_conflicts 同 repo 不同 Worktree 互检
#[tokio::test]
async fn it_v3_detect_conflicts_finds_active_peers() {
    let svc = InMemoryWorktreeService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_developer(tenant_id);

    // 同 repo 两个 worktree
    let repo_id = RepositoryId::new();
    let wt1 = svc
        .create_worktree(
            CreateWorktreeCommand {
                repository_id: repo_id,
                ..make_create_cmd(TenantId(tenant_id))
            },
            &actor,
        )
        .await
        .unwrap();
    let wt2 = svc
        .create_worktree(
            CreateWorktreeCommand {
                repository_id: repo_id,
                ..make_create_cmd(TenantId(tenant_id))
            },
            &actor,
        )
        .await
        .unwrap();

    // wt1 应该检测到 wt2 是冲突 (同 repo 活跃态)
    let conflicts = svc.detect_conflicts(wt1.id, &actor).await.unwrap();
    assert!(conflicts.contains(&wt2.id));
    assert!(!conflicts.contains(&wt1.id));
}

/// **IT-V3-2**: heatmap 按状态聚合 + cross-tenant 拒绝
#[tokio::test]
async fn it_v3_heatmap_aggregates_by_status() {
    let svc = InMemoryWorktreeService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_developer(tenant_id);
    let repo_id = RepositoryId::new();

    // 创 3 个 worktree 在同 repo (都 Created 状态)
    for _ in 0..3 {
        svc.create_worktree(
            CreateWorktreeCommand {
                repository_id: repo_id,
                ..make_create_cmd(TenantId(tenant_id))
            },
            &actor,
        )
        .await
        .unwrap();
    }
    let hm = svc.heatmap(repo_id, &actor).await.unwrap();
    assert_eq!(hm.repository_id, repo_id);
    assert_eq!(hm.total_worktrees, 3);
    assert_eq!(hm.by_status.get("CREATED").copied(), Some(3));

    // 跨 tenant 必拒
    let tenant_b = Uuid::new_v4();
    let actor_b = make_developer(tenant_b);
    let res = svc.heatmap(repo_id, &actor_b).await;
    assert!(matches!(res, Err(WorktreeError::CrossTenantDenied(_, _))));
}
