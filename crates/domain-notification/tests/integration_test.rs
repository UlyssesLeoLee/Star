//! 集成测试 (IT) — domain-notification 真实数据接入验证
//!
//! **crate**: domain-notification (test target)
//! **per**: 2026-09-07 P3-B sub-batch 2 write-heavy 域真实数据接入
//! **参考**: `crates/domain-workspace/tests/integration_test.rs` (per 7f9f52a 4 域真实接入已落地)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): register_channel + dispatch + get + list_by_user + mark_read
//! - V1 (模板 + 抑制): upsert_template 复用 ID + INV-N-07 默认抑制策略
//! - V2 (RLS 13 類): 跨 tenant 拒绝 (守门 #13 (c) Master 100% RLS)
//! - V3 (跨用户隔离): 仅本人可读/可标记 (INV-N-03 守门)

use domain_notification::{
    ActorContext, ChannelKind, DispatchNotificationCommand, GetNotificationQuery,
    InMemoryNotificationService, ListByUserQuery, MarkReadCommand, NotificationCommandPort,
    NotificationError, NotificationEventType, NotificationQueryPort, NotificationStatus, ProjectId,
    RegisterChannelCommand, TenantId, UpsertTemplateCommand, UserId,
};
use uuid::Uuid;

/// IT fixture: 普通 user actor (per INV-N-03 仅本人可读/可标记)
fn make_user_actor(tenant_id: Uuid, user_id: Uuid) -> ActorContext {
    ActorContext::new(user_id, tenant_id)
}

/// IT fixture: project_admin 角色 actor (per INV-N-04)
fn make_admin_actor(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("project_admin")
}

/// IT fixture: 基础 register_channel 命令
fn basic_channel_cmd(tid: Uuid, user: Uuid) -> RegisterChannelCommand {
    RegisterChannelCommand {
        tenant_id: TenantId(tid),
        user_id: UserId::from(user),
        kind: ChannelKind::InApp,
        address: format!("in_app://user/{}", user),
        actor_user_id: UserId::from(user),
    }
}

/// IT fixture: 基础 dispatch 命令
fn basic_dispatch_cmd(
    tid: Uuid,
    user: Uuid,
    evt: NotificationEventType,
) -> DispatchNotificationCommand {
    DispatchNotificationCommand {
        tenant_id: TenantId(tid),
        user_id: UserId::from(user),
        event_type: evt,
        resource_type: "work_item".to_string(),
        resource_id: Uuid::new_v4(),
        subject: "Test notification".to_string(),
        body: "body".to_string(),
        source: "test".to_string(),
    }
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: register_channel → dispatch → get → list_by_user → mark_read 完整闭环
#[tokio::test]
async fn it_v0_full_lifecycle() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = make_user_actor(tenant_id, user);

    // 1. register_channel
    let ch = svc
        .register_channel(basic_channel_cmd(tenant_id, user), &actor)
        .await
        .expect("register_channel 必成功");
    assert_eq!(ch.user_id, UserId::from(user));
    assert!(ch.enabled);

    // 2. dispatch(用 ValidationFailed 这种突破抑制的事件)
    let n = svc
        .dispatch(
            basic_dispatch_cmd(tenant_id, user, NotificationEventType::ValidationFailed),
            &actor,
        )
        .await
        .expect("dispatch 必成功");
    assert_eq!(n.user_id, UserId::from(user));
    assert_eq!(n.status, NotificationStatus::Sent);
    assert!(n.sent_at.is_some());
    assert!(n.read_at.is_none());

    // 3. get
    let fetched = svc
        .get(
            GetNotificationQuery {
                tenant_id: TenantId(tenant_id),
                notification_id: n.id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(fetched.id, n.id);

    // 4. list_by_user
    let list = svc
        .list_by_user(
            ListByUserQuery {
                tenant_id: TenantId(tenant_id),
                user_id: UserId::from(user),
                unread_only: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);

    // 5. mark_read
    let read = svc
        .mark_read(
            MarkReadCommand {
                tenant_id: TenantId(tenant_id),
                notification_id: n.id,
                actor_user_id: UserId::from(user),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(read.status, NotificationStatus::Read);
    assert!(read.read_at.is_some());
}

/// **IT-V0-2**: list_by_user unread_only 过滤
#[tokio::test]
async fn it_v0_list_unread_only_filter() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = make_user_actor(tenant_id, user);

    svc.register_channel(basic_channel_cmd(tenant_id, user), &actor)
        .await
        .unwrap();

    // 派发 3 条都突破抑制
    let mut ids = vec![];
    for _ in 0..3 {
        let n = svc
            .dispatch(
                basic_dispatch_cmd(tenant_id, user, NotificationEventType::ValidationFailed),
                &actor,
            )
            .await
            .unwrap();
        ids.push(n.id);
    }

    // 全部未读
    let all = svc
        .list_by_user(
            ListByUserQuery {
                tenant_id: TenantId(tenant_id),
                user_id: UserId::from(user),
                unread_only: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(all.len(), 3);

    // 标记第 1 条已读
    svc.mark_read(
        MarkReadCommand {
            tenant_id: TenantId(tenant_id),
            notification_id: ids[0],
            actor_user_id: UserId::from(user),
        },
        &actor,
    )
    .await
    .unwrap();

    // unread_only=true → 剩 2
    let unread = svc
        .list_by_user(
            ListByUserQuery {
                tenant_id: TenantId(tenant_id),
                user_id: UserId::from(user),
                unread_only: true,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(unread.len(), 2);
}

// =====================================================================
// V1: 模板 + INV-N-07 默认抑制
// =====================================================================

/// **IT-V1-1**: upsert_template 同 (tenant, project, event) 复用同一 ID (UPSERT 语义)
#[tokio::test]
async fn it_v1_template_upsert_reuses_id() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin = make_admin_actor(tenant_id);
    let event = NotificationEventType::ValidationFailed;

    // 第一次 create
    let t1 = svc
        .upsert_template(
            UpsertTemplateCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                event_type: event,
                channel_kinds: vec![ChannelKind::InApp],
                subject: "validation failed".to_string(),
                body_template: "step {{step}} failed: {{error}}".to_string(),
                actor_user_id: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();

    // 第二次 update 同 (tenant, project, event) → 复用 ID
    let t2 = svc
        .upsert_template(
            UpsertTemplateCommand {
                tenant_id: TenantId(tenant_id),
                project_id,
                event_type: event,
                channel_kinds: vec![ChannelKind::InApp, ChannelKind::Email],
                subject: "validation failed v2".to_string(),
                body_template: "step {{step}} failed: {{error}} (retry)".to_string(),
                actor_user_id: UserId::from(admin.user_id),
            },
            &admin,
        )
        .await
        .unwrap();
    assert_eq!(t1.id, t2.id, "UPSERT 必须复用同 ID");
    assert_eq!(t2.subject, "validation failed v2");
}

/// **IT-V1-2**: INV-N-07 — 默认抑制事件 dispatch → EventSuppressed
#[tokio::test]
async fn it_v1_invn07_default_suppression() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = make_user_actor(tenant_id, user);

    // WorkItemCreated 是默认抑制
    let res = svc
        .dispatch(
            basic_dispatch_cmd(tenant_id, user, NotificationEventType::WorkItemCreated),
            &actor,
        )
        .await;
    assert!(matches!(res, Err(NotificationError::EventSuppressed(_))));
}

/// **IT-V1-3**: INV-N-07 — 突破抑制事件 dispatch → OK
#[tokio::test]
async fn it_v1_invn07_breakthrough_dispatched() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = make_user_actor(tenant_id, user);

    for evt in [
        NotificationEventType::ValidationFailed,
        NotificationEventType::AgentSessionCrashed,
        NotificationEventType::ProtectedActionDenied,
    ] {
        let n = svc
            .dispatch(basic_dispatch_cmd(tenant_id, user, evt), &actor)
            .await
            .unwrap();
        assert_eq!(n.event_type, evt);
    }
}

/// **IT-V1-4**: upsert_template 需 admin role (INV-N-04)
#[tokio::test]
async fn it_v1_template_requires_admin() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = make_user_actor(tenant_id, user); // 无 admin 角色

    let res = svc
        .upsert_template(
            UpsertTemplateCommand {
                tenant_id: TenantId(tenant_id),
                project_id: ProjectId::new(),
                event_type: NotificationEventType::ValidationFailed,
                channel_kinds: vec![ChannelKind::InApp],
                subject: "x".to_string(),
                body_template: "y".to_string(),
                actor_user_id: UserId::from(user),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(NotificationError::PermissionDenied)));
}

// =====================================================================
// V2: RLS 13 類 tenant 隔离
// =====================================================================

/// **IT-V2-1**: 跨 tenant register_channel 被拒
#[tokio::test]
async fn it_v2_cross_tenant_register_channel_rejected() {
    let svc = InMemoryNotificationService::new();
    let actor_tenant = Uuid::new_v4();
    let cmd_tenant = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = make_user_actor(actor_tenant, user);

    let mut cmd = basic_channel_cmd(cmd_tenant, user); // 显式 cmd 另一个 tenant
    cmd.actor_user_id = UserId::from(user);
    let res = svc.register_channel(cmd, &actor).await;
    assert!(matches!(
        res,
        Err(NotificationError::CrossTenantDenied(_, _))
    ));
}

/// **IT-V2-2**: 跨 tenant dispatch 被拒
#[tokio::test]
async fn it_v2_cross_tenant_dispatch_rejected() {
    let svc = InMemoryNotificationService::new();
    let actor_tenant = Uuid::new_v4();
    let cmd_tenant = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = make_user_actor(actor_tenant, user);

    let res = svc
        .dispatch(
            basic_dispatch_cmd(cmd_tenant, user, NotificationEventType::ValidationFailed),
            &actor,
        )
        .await;
    assert!(matches!(
        res,
        Err(NotificationError::CrossTenantDenied(_, _))
    ));
}

/// **IT-V2-3**: 跨 tenant get 被拒
#[tokio::test]
async fn it_v2_cross_tenant_get_rejected() {
    let svc = InMemoryNotificationService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor_a = make_user_actor(tenant_a, user);
    let n = svc
        .dispatch(
            basic_dispatch_cmd(tenant_a, user, NotificationEventType::ValidationFailed),
            &actor_a,
        )
        .await
        .unwrap();

    // tenant B 同 user actor 试图读 tenant A 的通知
    let actor_b = make_user_actor(tenant_b, user);
    let res = svc
        .get(
            GetNotificationQuery {
                tenant_id: TenantId(tenant_b),
                notification_id: n.id,
            },
            &actor_b,
        )
        .await;
    assert!(matches!(
        res,
        Err(NotificationError::CrossTenantDenied(_, _))
    ));
}

// =====================================================================
// V3: 跨用户隔离 INV-N-03(仅本人可读/可标记)
// =====================================================================

/// **IT-V3-1**: 别人读我的通知被拒(INV-N-03)
#[tokio::test]
async fn it_v3_other_user_cannot_read() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let me = Uuid::new_v4();
    let other = Uuid::new_v4();
    let me_actor = make_user_actor(tenant_id, me);
    let n = svc
        .dispatch(
            basic_dispatch_cmd(tenant_id, me, NotificationEventType::ValidationFailed),
            &me_actor,
        )
        .await
        .unwrap();

    let other_actor = make_user_actor(tenant_id, other);
    let res = svc
        .get(
            GetNotificationQuery {
                tenant_id: TenantId(tenant_id),
                notification_id: n.id,
            },
            &other_actor,
        )
        .await;
    assert!(matches!(res, Err(NotificationError::PermissionDenied)));
}

/// **IT-V3-2**: 别人 list 我的通知被拒(INV-N-03)
#[tokio::test]
async fn it_v3_other_user_cannot_list() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let me = Uuid::new_v4();
    let other = Uuid::new_v4();
    let me_actor = make_user_actor(tenant_id, me);
    svc.dispatch(
        basic_dispatch_cmd(tenant_id, me, NotificationEventType::ValidationFailed),
        &me_actor,
    )
    .await
    .unwrap();

    // other actor 试图 list 别人的
    let other_actor = make_user_actor(tenant_id, other);
    let res = svc
        .list_by_user(
            ListByUserQuery {
                tenant_id: TenantId(tenant_id),
                user_id: UserId::from(me), // 显式要 me 的列表
                unread_only: false,
            },
            &other_actor, // actor 是 other
        )
        .await;
    assert!(matches!(res, Err(NotificationError::PermissionDenied)));
}

/// **IT-V3-3**: 别人 mark_read 我的通知被拒(INV-N-03)
#[tokio::test]
async fn it_v3_other_user_cannot_mark_read() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let me = Uuid::new_v4();
    let other = Uuid::new_v4();
    let me_actor = make_user_actor(tenant_id, me);
    let n = svc
        .dispatch(
            basic_dispatch_cmd(tenant_id, me, NotificationEventType::ValidationFailed),
            &me_actor,
        )
        .await
        .unwrap();

    let other_actor = make_user_actor(tenant_id, other);
    let res = svc
        .mark_read(
            MarkReadCommand {
                tenant_id: TenantId(tenant_id),
                notification_id: n.id,
                actor_user_id: UserId::from(other),
            },
            &other_actor,
        )
        .await;
    assert!(matches!(res, Err(NotificationError::PermissionDenied)));
}

/// **IT-V3-4**: 多用户隔离 — 各 user 只看到自己通知
#[tokio::test]
async fn it_v3_multi_user_isolation() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();
    let actor_a = make_user_actor(tenant_id, user_a);
    let actor_b = make_user_actor(tenant_id, user_b);

    // user_a 注册 channel + dispatch 1 条
    svc.register_channel(basic_channel_cmd(tenant_id, user_a), &actor_a)
        .await
        .unwrap();
    svc.dispatch(
        basic_dispatch_cmd(tenant_id, user_a, NotificationEventType::ValidationFailed),
        &actor_a,
    )
    .await
    .unwrap();

    // user_b dispatch 2 条
    svc.register_channel(basic_channel_cmd(tenant_id, user_b), &actor_b)
        .await
        .unwrap();
    for _ in 0..2 {
        svc.dispatch(
            basic_dispatch_cmd(tenant_id, user_b, NotificationEventType::ValidationFailed),
            &actor_b,
        )
        .await
        .unwrap();
    }

    // user_a list → 1
    let list_a = svc
        .list_by_user(
            ListByUserQuery {
                tenant_id: TenantId(tenant_id),
                user_id: UserId::from(user_a),
                unread_only: false,
            },
            &actor_a,
        )
        .await
        .unwrap();
    // user_b list → 2
    let list_b = svc
        .list_by_user(
            ListByUserQuery {
                tenant_id: TenantId(tenant_id),
                user_id: UserId::from(user_b),
                unread_only: false,
            },
            &actor_b,
        )
        .await
        .unwrap();
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_b.len(), 2);
    // 互不污染
    for n in &list_a {
        assert_eq!(n.user_id, UserId::from(user_a));
    }
    for n in &list_b {
        assert_eq!(n.user_id, UserId::from(user_b));
    }
}

/// **IT-V3-5**: mark_read 终态后再次 mark_read → InvalidState(INV-N-07 守门 + 状态机)
#[tokio::test]
async fn it_v3_mark_read_terminal_state() {
    let svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = make_user_actor(tenant_id, user);
    let n = svc
        .dispatch(
            basic_dispatch_cmd(tenant_id, user, NotificationEventType::ValidationFailed),
            &actor,
        )
        .await
        .unwrap();

    // 第一次 mark_read → OK
    svc.mark_read(
        MarkReadCommand {
            tenant_id: TenantId(tenant_id),
            notification_id: n.id,
            actor_user_id: UserId::from(user),
        },
        &actor,
    )
    .await
    .unwrap();

    // 第二次 mark_read → InvalidState (Read 是终态)
    let res = svc
        .mark_read(
            MarkReadCommand {
                tenant_id: TenantId(tenant_id),
                notification_id: n.id,
                actor_user_id: UserId::from(user),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(NotificationError::InvalidState(_))));
}
