//! 集成测试 (IT) — domain-workspace 真实数据接入验证
//!
//! **crate**: domain-workspace (test target)
//! **per**: 2026-09-07 P3-B sub-batch 1 read-heavy 域真实数据接入
//! **参考**: `crates/domain-form/src/lib.rs` (OPT-WORKER-09 done per 3a27a13)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): create + get_by_id + get_by_key + list
//! - V1 (成员协作): add_member + remove_member + list_members
//! - V2 (RLS 隔离): 跨 tenant 拒绝 (守门 #13 (c) Master 100% RLS)
//! - V3 (事件总线): WorkspaceCreated + MemberAdded + MemberRemoved 全部入流

use domain_workspace::{
    AddMemberCommand, ActorContext, CreateWorkspaceCommand, InMemoryWorkspaceService,
    ListWorkspaceQuery, RemoveMemberCommand, TenantId, UpdateWorkspaceCommand, UserId,
    WorkspaceCommandPort, WorkspaceError, WorkspaceEvent, WorkspaceQueryPort, WorkspaceRole,
};
use uuid::Uuid;

/// IT fixture: 创建带 admin role 的 actor (per 守门 #13 RLS 13 類)
fn make_admin_actor(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("workspace_admin")
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: 完整 create → get_by_id → get_by_key → list 闭环
#[tokio::test]
async fn it_v0_create_get_list_lifecycle() {
    let svc = InMemoryWorkspaceService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let actor = make_admin_actor(tenant_id);

    // create
    let ws = svc
        .create_workspace(
            CreateWorkspaceCommand {
                tenant_id: TenantId(tenant_id),
                workspace_key: "acme".to_string(),
                name: "Acme Workspace".to_string(),
                description: Some("main".to_string()),
                owner_user_id: UserId::new(),
            },
            actor.clone(),
        )
        .await
        .expect("create 必成功");

    assert_eq!(ws.version, 1);
    assert_eq!(ws.workspace_key, "acme");

    // get_by_id
    let fetched = svc.get_by_id(ws.id, actor.clone()).await.unwrap();
    assert_eq!(fetched.id, ws.id);

    // get_by_key
    let by_key = svc
        .get_by_key(TenantId(tenant_id), "acme", actor.clone())
        .await
        .unwrap();
    assert_eq!(by_key.id, ws.id);

    // list
    let list = svc
        .list_workspaces(
            ListWorkspaceQuery {
                tenant_id: TenantId(tenant_id),
                limit: 50,
                offset: 0,
            },
            actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, ws.id);
}

/// **IT-V0-2**: update 走乐观锁,version conflict 拒绝
#[tokio::test]
async fn it_v0_update_version_conflict() {
    let svc = InMemoryWorkspaceService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let actor = make_admin_actor(tenant_id);
    let ws = svc
        .create_workspace(
            CreateWorkspaceCommand {
                tenant_id: TenantId(tenant_id),
                workspace_key: "alpha".to_string(),
                name: "Alpha".to_string(),
                description: None,
                owner_user_id: UserId::new(),
            },
            actor.clone(),
        )
        .await
        .unwrap();

    // 第一次 update 必成功
    let updated = svc
        .update_workspace(
            UpdateWorkspaceCommand {
                workspace_id: ws.id,
                tenant_id: TenantId(tenant_id),
                expected_version: 1,
                name: Some("Alpha v2".to_string()),
                description: None,
            },
            actor.clone(),
        )
        .await
        .unwrap();
    assert_eq!(updated.version, 2);

    // 第二次 update 仍用 expected_version=1 → Conflict
    let res = svc
        .update_workspace(
            UpdateWorkspaceCommand {
                workspace_id: ws.id,
                tenant_id: TenantId(tenant_id),
                expected_version: 1,
                name: Some("Alpha v3".to_string()),
                description: None,
            },
            actor,
        )
        .await;
    assert!(matches!(res, Err(WorkspaceError::Conflict(_))));
}

// =====================================================================
// V1: 成员协作
// =====================================================================

/// **IT-V1-1**: add_member → list_members → remove_member 完整协作闭环
#[tokio::test]
async fn it_v1_member_add_list_remove() {
    let svc = InMemoryWorkspaceService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let actor = make_admin_actor(tenant_id);
    let ws = svc
        .create_workspace(
            CreateWorkspaceCommand {
                tenant_id: TenantId(tenant_id),
                workspace_key: "team".to_string(),
                name: "Team".to_string(),
                description: None,
                owner_user_id: UserId::new(),
            },
            actor.clone(),
        )
        .await
        .unwrap();

    // owner 已是 Admin 成员
    let initial = svc.list_members(ws.id, actor.clone()).await.unwrap();
    assert_eq!(initial.len(), 1);
    assert!(initial[0].is_admin());

    // 加 1 个 Member
    let new_user = UserId::new();
    let added = svc
        .add_member(
            AddMemberCommand {
                workspace_id: ws.id,
                tenant_id: TenantId(tenant_id),
                user_id: new_user,
                role: WorkspaceRole::Member,
            },
            actor.clone(),
        )
        .await
        .unwrap();
    assert_eq!(added.role, WorkspaceRole::Member);

    let after_add = svc.list_members(ws.id, actor.clone()).await.unwrap();
    assert_eq!(after_add.len(), 2);

    // 移除
    svc.remove_member(
        RemoveMemberCommand {
            workspace_id: ws.id,
            tenant_id: TenantId(tenant_id),
            user_id: new_user,
        },
        actor,
    )
    .await
    .unwrap();

    let after_remove = svc.list_members(ws.id, make_admin_actor(tenant_id)).await.unwrap();
    assert_eq!(after_remove.len(), 1);
}

// =====================================================================
// V2: RLS 13 類 tenant 隔离
// =====================================================================

/// **IT-V2-1**: 跨 tenant 访问被拒(守门 #13 (c) Master 100% RLS 13 類)
#[tokio::test]
async fn it_v2_cross_tenant_rejected() {
    let svc = InMemoryWorkspaceService::new_for_test();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let actor_a = make_admin_actor(tenant_a);
    let ws_a = svc
        .create_workspace(
            CreateWorkspaceCommand {
                tenant_id: TenantId(tenant_a),
                workspace_key: "a".to_string(),
                name: "A".to_string(),
                description: None,
                owner_user_id: UserId::new(),
            },
            actor_a,
        )
        .await
        .unwrap();

    // tenant B 试图访问 tenant A 的 workspace
    let actor_b = make_admin_actor(tenant_b);
    let res = svc.get_by_id(ws_a.id, actor_b).await;
    assert!(matches!(res, Err(WorkspaceError::PermissionDenied)));
}

/// **IT-V2-2**: 同 workspace_key 不同 tenant 互不干扰(RLS 隔离生效)
#[tokio::test]
async fn it_v2_same_key_different_tenant_isolated() {
    let svc = InMemoryWorkspaceService::new_for_test();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let actor_a = make_admin_actor(tenant_a);
    let actor_b = make_admin_actor(tenant_b);

    // 同样 workspace_key="shared" 两个 tenant 各创建一个
    svc.create_workspace(
        CreateWorkspaceCommand {
            tenant_id: TenantId(tenant_a),
            workspace_key: "shared".to_string(),
            name: "A Shared".to_string(),
            description: None,
            owner_user_id: UserId::new(),
        },
        actor_a,
    )
    .await
    .unwrap();
    svc.create_workspace(
        CreateWorkspaceCommand {
            tenant_id: TenantId(tenant_b),
            workspace_key: "shared".to_string(),
            name: "B Shared".to_string(),
            description: None,
            owner_user_id: UserId::new(),
        },
        actor_b,
    )
    .await
    .unwrap();

    // 各 tenant list 只看到自己
    let list_a = svc
        .list_workspaces(
            ListWorkspaceQuery {
                tenant_id: TenantId(tenant_a),
                limit: 50,
                offset: 0,
            },
            make_admin_actor(tenant_a),
        )
        .await
        .unwrap();
    let list_b = svc
        .list_workspaces(
            ListWorkspaceQuery {
                tenant_id: TenantId(tenant_b),
                limit: 50,
                offset: 0,
            },
            make_admin_actor(tenant_b),
        )
        .await
        .unwrap();
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_b.len(), 1);
    assert_eq!(list_a[0].name, "A Shared");
    assert_eq!(list_b[0].name, "B Shared");
}

// =====================================================================
// V3: 事件总线
// =====================================================================

/// **IT-V3-1**: 完整事件流 Created + MemberAdded + MemberRemoved
#[tokio::test]
async fn it_v3_event_bus_full_stream() {
    let (svc, mut rx) = InMemoryWorkspaceService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_admin_actor(tenant_id);

    // create → Created 事件
    let ws = svc
        .create_workspace(
            CreateWorkspaceCommand {
                tenant_id: TenantId(tenant_id),
                workspace_key: "ev".to_string(),
                name: "Event".to_string(),
                description: None,
                owner_user_id: UserId::new(),
            },
            actor.clone(),
        )
        .await
        .unwrap();
    let evt1 = rx.try_recv().expect("应收到 Created 事件");
    assert!(matches!(evt1, WorkspaceEvent::Created(_)));
    assert_eq!(evt1.subject(), "star.events.workspace.workspace.created.v1");

    // add member → MemberAdded 事件
    let new_user = UserId::new();
    svc.add_member(
        AddMemberCommand {
            workspace_id: ws.id,
            tenant_id: TenantId(tenant_id),
            user_id: new_user,
            role: WorkspaceRole::Member,
        },
        actor.clone(),
    )
    .await
    .unwrap();
    let evt2 = rx.try_recv().expect("应收到 MemberAdded 事件");
    assert!(matches!(evt2, WorkspaceEvent::MemberAdded(_)));
    assert_eq!(evt2.subject(), "star.events.workspace.member.added.v1");

    // remove member → MemberRemoved 事件
    svc.remove_member(
        RemoveMemberCommand {
            workspace_id: ws.id,
            tenant_id: TenantId(tenant_id),
            user_id: new_user,
        },
        actor,
    )
    .await
    .unwrap();
    let evt3 = rx.try_recv().expect("应收到 MemberRemoved 事件");
    assert!(matches!(evt3, WorkspaceEvent::MemberRemoved(_)));
    assert_eq!(evt3.subject(), "star.events.workspace.member.removed.v1");
}
