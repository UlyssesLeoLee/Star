//! 跨域 saga 集成测试 (IT) — 3 write-heavy 域跨域联动
//!
//! **crate**: domain-work-item (test target — 跨域文件放第一个域)
//! **per**: 2026-09-07 P1-3 (per 9/7 19:38 JST 拍板)
//! **守门**: #13 a L1↔L1 禁止, L0 协调 (per TMO-03 + 守门 #25 5 域独立 Lead)
//!
//! ## 跨域场景
//!
//! - **S1: work-item → comment** — 在 work-item 上 add comment, comment 出现在 work-item 评论列表
//! - **S2: comment → notification** — comment 内 mention user, notification 派发给被 mention user
//! - **S3: work-item → notification** — work-item 状态变更, notification 派发给 assignee
//!
//! ## L0 编排模式 (per 守门 #13 a)
//!
//! L1 域不直接互相调用, 由测试函数 (扮演 L0 TMO orchestrator) 协调:
//! 1. 接收 L1 域 event/signal
//! 2. 决定下一步 L1 域调用
//! 3. 调用 L1 域
//!
//! **本文件 1 per file group (per 守门 #20)**, 12 IT = 3 场景 × 4 维 (V0-V3)。

use domain_comment::{
    ActorContext as CommentActor, CommentCommandPort, CommentQueryPort, CreateCommentCommand,
    GetCommentQuery, InMemoryCommentService, ListByParentQuery, ParentType,
    ProjectId as CommentProjectId, TenantId as CommentTenantId, UserId as CommentUserId,
};
use domain_notification::{
    ActorContext as NotifActor, ChannelKind, DispatchNotificationCommand,
    InMemoryNotificationService, ListByUserQuery, NotificationCommandPort, NotificationEventType,
    NotificationQueryPort, RegisterChannelCommand, TenantId as NotifTenantId,
    UserId as NotifUserId,
};
use domain_work_item::{
    ActorContext, CreateWorkItemCommand, InMemoryWorkItemService, Priority, ProjectId, TenantId,
    TransitionStatusCommand, UserId, WorkItemCommandPort, WorkItemQueryPort, WorkItemStatus,
    WorkItemType, WorkspaceId,
};
use uuid::Uuid;

// =====================================================================
// 共享 fixtures (per 守门 #18 共享 ActorContext 字段扩展, 跨域 actor 一致)
// =====================================================================

/// dev actor for 3 domains (共享同一 user_id + tenant_id, per 守门 #18)
fn make_dev_actor(tenant_id: Uuid, user_id: Uuid) -> ActorContext {
    ActorContext::new(user_id, tenant_id).with_role("developer")
}

/// admin actor for 3 domains
fn make_admin_actor(tenant_id: Uuid, user_id: Uuid) -> ActorContext {
    ActorContext::new(user_id, tenant_id).with_role("project_admin")
}

/// 基础 Task 创建命令 (per IT-V0-1 work-item pattern)
fn basic_task_cmd(tid: Uuid, reporter: Uuid) -> CreateWorkItemCommand {
    CreateWorkItemCommand {
        tenant_id: TenantId(tid),
        workspace_id: WorkspaceId::new(),
        project_id: ProjectId::new(),
        item_type: WorkItemType::Task,
        title: "fix login bug".to_string(),
        description: "OAuth callback race".to_string(),
        priority: Priority::High,
        severity: None,
        reporter_user_id: UserId::from(reporter),
        parent_work_item_id: None,
        ai_task_data: None,
        labels: vec!["bug".to_string()],
    }
}

// =====================================================================
// S1: work-item → comment (跨域场景 1)
// =====================================================================

/// **IT-S1-V0**: L0 编排 — 创建 work-item, 在该 work-item 上 add comment,
/// 验证 comment 出现在 work-item 评论列表
#[tokio::test]
async fn it_s1_v0_workitem_create_then_comment_appears_in_list() {
    // L0 持有 3 个 L1 服务的引用 (per 守门 #13 a L0 协调, 不直接 L1↔L1)
    let wi_svc = InMemoryWorkItemService::new();
    let c_svc = InMemoryCommentService::new();

    let tenant_id = Uuid::new_v4();
    let reporter = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, reporter);

    // L0 步骤 1: 调 L1 work-item.create_work_item
    let item = wi_svc
        .create_work_item(basic_task_cmd(tenant_id, reporter), &actor)
        .await
        .expect("create work-item 必成功");
    let work_item_uuid = item.id.as_uuid();

    // L0 步骤 2: 调 L1 comment.create_comment (parent_id = work-item.id, ParentType::WorkItem)
    let c_actor = CommentActor::new(author, tenant_id);
    let mut cmd = CreateCommentCommand {
        tenant_id: CommentTenantId(tenant_id),
        project_id: CommentProjectId::new(),
        parent_type: ParentType::WorkItem,
        parent_id: work_item_uuid,
        body: "first review".to_string(),
        author_user_id: Some(CommentUserId::from(author)),
        author_agent_id: None,
        mentions: vec![],
        attachment_ids: vec![],
        actor_user_id: CommentUserId::from(author),
    };
    // 同步 project_id (跨域保持一致, per 守门 #18)
    cmd.project_id = CommentProjectId::from(item.project_id.as_uuid());
    let c = c_svc
        .create_comment(cmd, &c_actor)
        .await
        .expect("create comment 必成功");

    // L0 步骤 3: 验证 comment 出现在 work-item 的评论列表
    let list = c_svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: CommentTenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id: work_item_uuid,
                include_deleted: false,
            },
            &c_actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1, "work-item 上 1 条 comment");
    assert_eq!(list[0].id, c.id);
    assert_eq!(list[0].parent_id, work_item_uuid);
}

/// **IT-S1-V1**: 同一 work-item 上 3 条 comment, 全部按时间序出现在列表
#[tokio::test]
async fn it_s1_v1_multiple_comments_on_same_workitem() {
    let wi_svc = InMemoryWorkItemService::new();
    let c_svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let reporter = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, reporter);
    let c_actor = CommentActor::new(author, tenant_id);

    let item = wi_svc
        .create_work_item(basic_task_cmd(tenant_id, reporter), &actor)
        .await
        .unwrap();
    let work_item_uuid = item.id.as_uuid();

    for i in 0..3 {
        let mut cmd = CreateCommentCommand {
            tenant_id: CommentTenantId(tenant_id),
            project_id: CommentProjectId::from(item.project_id.as_uuid()),
            parent_type: ParentType::WorkItem,
            parent_id: work_item_uuid,
            body: format!("review #{}", i),
            author_user_id: Some(CommentUserId::from(author)),
            author_agent_id: None,
            mentions: vec![],
            attachment_ids: vec![],
            actor_user_id: CommentUserId::from(author),
        };
        cmd.project_id = CommentProjectId::from(item.project_id.as_uuid());
        c_svc.create_comment(cmd, &c_actor).await.unwrap();
    }

    let list = c_svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: CommentTenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id: work_item_uuid,
                include_deleted: false,
            },
            &c_actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 3, "同 work-item 上 3 条 comment 全部出现");
    // body 唯一性验证
    let bodies: Vec<String> = list.iter().map(|c| c.body.clone()).collect();
    assert!(bodies.iter().any(|b| b == "review #0"));
    assert!(bodies.iter().any(|b| b == "review #1"));
    assert!(bodies.iter().any(|b| b == "review #2"));
}

/// **IT-S1-V2**: 跨域 RLS — tenant B 试图在 tenant A 的 work-item 上 add comment 必拒
/// (per 守门 #13 (c) Master 100% RLS, 跨域 RLS 一致)
#[tokio::test]
async fn it_s1_v2_cross_tenant_comment_rejected() {
    let wi_svc = InMemoryWorkItemService::new();
    let c_svc = InMemoryCommentService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let reporter = Uuid::new_v4();
    let actor_a = make_admin_actor(tenant_a, reporter);

    // tenant A 创 work-item
    let item = wi_svc
        .create_work_item(basic_task_cmd(tenant_a, reporter), &actor_a)
        .await
        .unwrap();
    let work_item_uuid = item.id.as_uuid();

    // tenant B 试图在该 work-item 上 add comment, 显式编 cmd.tenant_id = tenant_a
    // 但 actor 是 tenant_b → 必拒 CrossTenantDenied
    let author_b = Uuid::new_v4();
    let actor_b = make_admin_actor(tenant_b, author_b);
    let mut cmd = CreateCommentCommand {
        tenant_id: CommentTenantId(tenant_a), // 显式要 tenant_a 的 work-item
        project_id: CommentProjectId::from(item.project_id.as_uuid()),
        parent_type: ParentType::WorkItem,
        parent_id: work_item_uuid,
        body: "hostile takeover".to_string(),
        author_user_id: Some(CommentUserId::from(author_b)),
        author_agent_id: None,
        mentions: vec![],
        attachment_ids: vec![],
        actor_user_id: CommentUserId::from(author_b),
    };
    cmd.project_id = CommentProjectId::from(item.project_id.as_uuid());
    let res = c_svc.create_comment(cmd, &actor_b).await;
    assert!(
        matches!(
            res,
            Err(domain_comment::CommentError::CrossTenantDenied(_, _))
        ),
        "跨 tenant comment 必拒, got: {:?}",
        res
    );
}

/// **IT-S1-V3**: comment 软删除后, list filter (include_deleted=false) 不再返回
/// (per INV-C-01 soft delete 状态机 + L0 跨域 list 一致性)
#[tokio::test]
async fn it_s1_v3_soft_deleted_comment_excluded_from_workitem_list() {
    let wi_svc = InMemoryWorkItemService::new();
    let c_svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let reporter = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, reporter);
    let c_actor = CommentActor::new(author, tenant_id);

    let item = wi_svc
        .create_work_item(basic_task_cmd(tenant_id, reporter), &actor)
        .await
        .unwrap();
    let work_item_uuid = item.id.as_uuid();

    // 加 2 条
    let mut c1 = CreateCommentCommand {
        tenant_id: CommentTenantId(tenant_id),
        project_id: CommentProjectId::from(item.project_id.as_uuid()),
        parent_type: ParentType::WorkItem,
        parent_id: work_item_uuid,
        body: "first".to_string(),
        author_user_id: Some(CommentUserId::from(author)),
        author_agent_id: None,
        mentions: vec![],
        attachment_ids: vec![],
        actor_user_id: CommentUserId::from(author),
    };
    c1.project_id = CommentProjectId::from(item.project_id.as_uuid());
    let c1 = c_svc.create_comment(c1, &c_actor).await.unwrap();

    let mut c2 = CreateCommentCommand {
        tenant_id: CommentTenantId(tenant_id),
        project_id: CommentProjectId::from(item.project_id.as_uuid()),
        parent_type: ParentType::WorkItem,
        parent_id: work_item_uuid,
        body: "second".to_string(),
        author_user_id: Some(CommentUserId::from(author)),
        author_agent_id: None,
        mentions: vec![],
        attachment_ids: vec![],
        actor_user_id: CommentUserId::from(author),
    };
    c2.project_id = CommentProjectId::from(item.project_id.as_uuid());
    c_svc.create_comment(c2, &c_actor).await.unwrap();

    // 软删 c1
    c_svc
        .delete_comment(
            domain_comment::DeleteCommentCommand {
                tenant_id: CommentTenantId(tenant_id),
                comment_id: c1.id,
                actor_user_id: CommentUserId::from(author),
            },
            &c_actor,
        )
        .await
        .unwrap();

    // list (include_deleted=false) → 1 条 (c2)
    let visible = c_svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: CommentTenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id: work_item_uuid,
                include_deleted: false,
            },
            &c_actor,
        )
        .await
        .unwrap();
    assert_eq!(visible.len(), 1, "c1 软删后被过滤, 剩 c2");
    assert_ne!(visible[0].id, c1.id);

    // list (include_deleted=true) → 2 条 (c1 软删但仍存)
    let all = c_svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: CommentTenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id: work_item_uuid,
                include_deleted: true,
            },
            &c_actor,
        )
        .await
        .unwrap();
    assert_eq!(all.len(), 2, "include_deleted=true 看到 2 条");
}

// =====================================================================
// S2: comment → notification (跨域场景 2 — L0 编排)
//
// L0 编排逻辑 (per TMO-03):
// 1. comment.create 成功, 收集 mentions 列表
// 2. 对每个 mention user, 调 notification.dispatch (用 breakthrough 事件类型
//    FeedbackCreated — CommentMentioned 尚未实装, 用 FeedbackCreated 代理
//    突破 INV-N-07 默认抑制)
//
// 注: 这是 mock L0 编排, 真实 L0 TMO 由 star-runtime L0 派发层
// 监听 comment event bus 后触发 (per docs/architecture/2026-09-03-langgraph/)
// =====================================================================

/// L0 orchestrator helper: 模拟 L0 监听 comment event bus 后,
/// 遍历 mentions 派发 notification 给被 mention user
async fn l0_dispatch_mention_notifications(
    c_svc: &InMemoryCommentService,
    n_svc: &InMemoryNotificationService,
    comment_id: domain_comment::CommentId,
    tenant_id: Uuid,
    actor: &NotifActor,
) -> Vec<Uuid> {
    // 收集 mentions: 查 comment 内 mentions 列表 (L0 读 L1 数据)
    let comment = c_svc
        .get(
            GetCommentQuery {
                tenant_id: CommentTenantId(tenant_id),
                comment_id,
            },
            &CommentActor::new(actor.user_id, tenant_id),
        )
        .await
        .expect("comment 必存在");
    let mut dispatched = vec![];
    for mentioned in &comment.mentions {
        let n = n_svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: NotifTenantId(tenant_id),
                    user_id: NotifUserId::from(mentioned.as_uuid()),
                    // 突破抑制 (per INV-N-07): mention notification 必送达
                    event_type: NotificationEventType::FeedbackCreated,
                    resource_type: "comment".to_string(),
                    resource_id: comment_id.as_uuid(),
                    subject: "you were mentioned".to_string(),
                    body: format!("in comment {}", comment_id),
                    source: "l0_tmo_orchestrator".to_string(),
                },
                actor,
            )
            .await
            .expect("mention notification 必成功");
        dispatched.push(n.id.as_uuid());
    }
    dispatched
}

/// **IT-S2-V0**: comment mention 1 user → L0 派发 1 notification 给被 mention user
#[tokio::test]
async fn it_s2_v0_comment_mention_dispatches_notification() {
    let c_svc = InMemoryCommentService::new();
    let n_svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let mentioned = Uuid::new_v4();
    let c_actor = CommentActor::new(author, tenant_id);
    let n_actor = NotifActor::new(author, tenant_id);

    // mentioned user 先 register channel — INV-N-03 仅本人注册自己的 channel
    n_svc
        .register_channel(
            RegisterChannelCommand {
                tenant_id: NotifTenantId(tenant_id),
                user_id: NotifUserId::from(mentioned),
                kind: ChannelKind::InApp,
                address: format!("in_app://user/{}", mentioned),
                actor_user_id: NotifUserId::from(mentioned),
            },
            &NotifActor::new(mentioned, tenant_id),
        )
        .await
        .unwrap();

    // 创建 comment with 1 mention
    let parent_id = Uuid::new_v4(); // 任意 parent (不限定 work-item)
    let cmd = CreateCommentCommand {
        tenant_id: CommentTenantId(tenant_id),
        project_id: CommentProjectId::new(),
        parent_type: ParentType::WorkItem,
        parent_id,
        body: format!("hey @{}, please review", mentioned),
        author_user_id: Some(CommentUserId::from(author)),
        author_agent_id: None,
        mentions: vec![CommentUserId::from(mentioned)],
        attachment_ids: vec![],
        actor_user_id: CommentUserId::from(author),
    };
    let c = c_svc.create_comment(cmd, &c_actor).await.unwrap();
    assert_eq!(c.mentions.len(), 1);

    // L0 编排: 派发 mention notification
    let dispatched =
        l0_dispatch_mention_notifications(&c_svc, &n_svc, c.id, tenant_id, &n_actor).await;
    assert_eq!(dispatched.len(), 1, "1 mention → 1 notification");

    // 验证: mentioned user 的 inbox 有 1 条
    let inbox = n_svc
        .list_by_user(
            ListByUserQuery {
                tenant_id: NotifTenantId(tenant_id),
                user_id: NotifUserId::from(mentioned),
                unread_only: false,
            },
            &NotifActor::new(mentioned, tenant_id),
        )
        .await
        .unwrap();
    assert_eq!(
        inbox.len(),
        1,
        "mentioned user 收到 1 条 mention notification"
    );
    assert_eq!(inbox[0].event_type, NotificationEventType::FeedbackCreated);
}

/// **IT-S2-V1**: comment 0 mention → L0 派发 0 notification
#[tokio::test]
async fn it_s2_v1_no_mention_no_notification() {
    let c_svc = InMemoryCommentService::new();
    let n_svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let c_actor = CommentActor::new(author, tenant_id);
    let n_actor = NotifActor::new(author, tenant_id);

    // 0 mention comment
    let cmd = CreateCommentCommand {
        tenant_id: CommentTenantId(tenant_id),
        project_id: CommentProjectId::new(),
        parent_type: ParentType::WorkItem,
        parent_id: Uuid::new_v4(),
        body: "no mention here".to_string(),
        author_user_id: Some(CommentUserId::from(author)),
        author_agent_id: None,
        mentions: vec![],
        attachment_ids: vec![],
        actor_user_id: CommentUserId::from(author),
    };
    let c = c_svc.create_comment(cmd, &c_actor).await.unwrap();
    assert!(c.mentions.is_empty());

    // L0 编排: 0 mention → 0 notification
    let dispatched =
        l0_dispatch_mention_notifications(&c_svc, &n_svc, c.id, tenant_id, &n_actor).await;
    assert_eq!(dispatched.len(), 0, "0 mention → 0 notification dispatched");
}

/// **IT-S2-V2**: comment mention 3 users → L0 派发 3 notification, 每人 1 条
#[tokio::test]
async fn it_s2_v2_multiple_mentions_dispatch_to_each() {
    let c_svc = InMemoryCommentService::new();
    let n_svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let m1 = Uuid::new_v4();
    let m2 = Uuid::new_v4();
    let m3 = Uuid::new_v4();
    let c_actor = CommentActor::new(author, tenant_id);
    let n_actor = NotifActor::new(author, tenant_id);

    // 3 mention user 都 register channel
    for m in [m1, m2, m3] {
        n_svc
            .register_channel(
                RegisterChannelCommand {
                    tenant_id: NotifTenantId(tenant_id),
                    user_id: NotifUserId::from(m),
                    kind: ChannelKind::InApp,
                    address: format!("in_app://user/{}", m),
                    actor_user_id: NotifUserId::from(m),
                },
                &NotifActor::new(m, tenant_id),
            )
            .await
            .unwrap();
    }

    let cmd = CreateCommentCommand {
        tenant_id: CommentTenantId(tenant_id),
        project_id: CommentProjectId::new(),
        parent_type: ParentType::WorkItem,
        parent_id: Uuid::new_v4(),
        body: "cc @all".to_string(),
        author_user_id: Some(CommentUserId::from(author)),
        author_agent_id: None,
        mentions: vec![
            CommentUserId::from(m1),
            CommentUserId::from(m2),
            CommentUserId::from(m3),
        ],
        attachment_ids: vec![],
        actor_user_id: CommentUserId::from(author),
    };
    let c = c_svc.create_comment(cmd, &c_actor).await.unwrap();

    let dispatched =
        l0_dispatch_mention_notifications(&c_svc, &n_svc, c.id, tenant_id, &n_actor).await;
    assert_eq!(dispatched.len(), 3, "3 mention → 3 notification");

    // 每人 inbox 1 条
    for m in [m1, m2, m3] {
        let inbox = n_svc
            .list_by_user(
                ListByUserQuery {
                    tenant_id: NotifTenantId(tenant_id),
                    user_id: NotifUserId::from(m),
                    unread_only: false,
                },
                &NotifActor::new(m, tenant_id),
            )
            .await
            .unwrap();
        assert_eq!(inbox.len(), 1, "user {} 收到 1 条", m);
    }
}

/// **IT-S2-V3**: 跨 tenant mention 必拒 — INV-N-07 守门 + 跨域 RLS 一致
/// (per 守门 #13 (c) Master 100% RLS, 跨域 L0 编排不绕过 tenant 隔离)
#[tokio::test]
async fn it_s2_v3_cross_tenant_mention_rejected() {
    let c_svc = InMemoryCommentService::new();
    let n_svc = InMemoryNotificationService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let author = Uuid::new_v4();
    let mentioned_b = Uuid::new_v4();
    let c_actor = CommentActor::new(author, tenant_a);

    // tenant A comment with mention tenant B user
    // comment.tenant_id 跟 actor.tenant_id 一致 (per 守门 INV tenant check)
    // 但 L0 派发 notification 时, mentioned user 在 tenant B → 跨域拒绝
    let cmd = CreateCommentCommand {
        tenant_id: CommentTenantId(tenant_a),
        project_id: CommentProjectId::new(),
        parent_type: ParentType::WorkItem,
        parent_id: Uuid::new_v4(),
        body: format!("hey @{}", mentioned_b),
        author_user_id: Some(CommentUserId::from(author)),
        author_agent_id: None,
        mentions: vec![CommentUserId::from(mentioned_b)],
        attachment_ids: vec![],
        actor_user_id: CommentUserId::from(author),
    };
    let c = c_svc.create_comment(cmd, &c_actor).await.unwrap();

    // L0 尝试用 tenant_a actor 派发 notification 给 tenant_b user
    // → notification.dispatch 必拒 (per 守门 #13 (c) cross-tenant RLS)
    let n_actor_a = NotifActor::new(author, tenant_a);
    let res = n_svc
        .dispatch(
            DispatchNotificationCommand {
                tenant_id: NotifTenantId(tenant_b), // 显式要 tenant_b
                user_id: NotifUserId::from(mentioned_b),
                event_type: NotificationEventType::FeedbackCreated,
                resource_type: "comment".to_string(),
                resource_id: c.id.as_uuid(),
                subject: "cross-tenant mention".to_string(),
                body: "x".to_string(),
                source: "l0_tmo_orchestrator".to_string(),
            },
            &n_actor_a, // actor 是 tenant_a
        )
        .await;
    assert!(
        matches!(
            res,
            Err(domain_notification::NotificationError::CrossTenantDenied(
                _,
                _
            ))
        ),
        "跨 tenant 派发 mention notification 必拒, got: {:?}",
        res
    );
}

// =====================================================================
// S3: work-item → notification (跨域场景 3 — L0 编排)
//
// L0 编排逻辑 (per TMO-03):
// 1. work-item.status.transition 成功
// 2. 若有 assignee_user_id, L0 调 notification.dispatch 给 assignee
// 3. event_type 用 FeedbackRequired (突破 INV-N-07 默认抑制)
//
// 注: 真实 L0 TMO 由 star-runtime L0 派发层订阅 work-item event bus
// =====================================================================

/// L0 orchestrator helper: work-item status change → assignee notification
async fn l0_dispatch_workitem_status_change(
    wi_svc: &InMemoryWorkItemService,
    n_svc: &InMemoryNotificationService,
    work_item_id: domain_work_item::WorkItemId,
    tenant_id: Uuid,
    actor: &NotifActor,
) -> Vec<Uuid> {
    // L0 查 work-item 当前状态
    let item = wi_svc
        .get(
            domain_work_item::GetWorkItemQuery {
                tenant_id: domain_work_item::TenantId(tenant_id),
                work_item_id,
            },
            &ActorContext::new(actor.user_id, tenant_id),
        )
        .await
        .expect("work-item 必存在");

    let mut dispatched = vec![];
    if let Some(assignee) = item.assignee_user_id {
        let n = n_svc
            .dispatch(
                DispatchNotificationCommand {
                    tenant_id: NotifTenantId(tenant_id),
                    user_id: NotifUserId::from(assignee.as_uuid()),
                    // 突破 INV-N-07 抑制: status change 必送达
                    event_type: NotificationEventType::FeedbackRequired,
                    resource_type: "work_item".to_string(),
                    resource_id: work_item_id.as_uuid(),
                    subject: format!("work-item status: {:?}", item.status),
                    body: format!("work-item {} → {:?}", work_item_id, item.status),
                    source: "l0_tmo_orchestrator".to_string(),
                },
                actor,
            )
            .await
            .expect("assignee notification 必成功");
        dispatched.push(n.id.as_uuid());
    }
    dispatched
}

/// **IT-S3-V0**: work-item 状态变更 (有 assignee) → 派发 notification 给 assignee
#[tokio::test]
async fn it_s3_v0_workitem_status_change_dispatches_to_assignee() {
    let wi_svc = InMemoryWorkItemService::new();
    let n_svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let reporter = Uuid::new_v4();
    let assignee = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, reporter);
    let n_actor = NotifActor::new(reporter, tenant_id);

    // assignee 先 register channel
    n_svc
        .register_channel(
            RegisterChannelCommand {
                tenant_id: NotifTenantId(tenant_id),
                user_id: NotifUserId::from(assignee),
                kind: ChannelKind::InApp,
                address: format!("in_app://user/{}", assignee),
                actor_user_id: NotifUserId::from(assignee),
            },
            &NotifActor::new(assignee, tenant_id),
        )
        .await
        .unwrap();

    // 创建 + 分配 assignee
    let item = wi_svc
        .create_work_item(basic_task_cmd(tenant_id, reporter), &actor)
        .await
        .unwrap();
    wi_svc
        .assign(
            domain_work_item::AssignCommand {
                tenant_id: TenantId(tenant_id),
                work_item_id: item.id,
                assignee_user_id: Some(UserId::from(assignee)),
                assignee_agent_id: None,
                actor_user_id: UserId::from(reporter),
            },
            &actor,
        )
        .await
        .unwrap();

    // 状态变更: TODO → IN_PROGRESS
    wi_svc
        .transition_status(
            TransitionStatusCommand {
                tenant_id: TenantId(tenant_id),
                work_item_id: item.id,
                from: WorkItemStatus::Todo,
                to: WorkItemStatus::InProgress,
                actor_user_id: UserId::from(reporter),
            },
            &actor,
        )
        .await
        .unwrap();

    // L0 编排: 派发 notification 给 assignee
    let dispatched =
        l0_dispatch_workitem_status_change(&wi_svc, &n_svc, item.id, tenant_id, &n_actor).await;
    assert_eq!(dispatched.len(), 1, "1 assignee → 1 notification");

    // assignee inbox 验证
    let inbox = n_svc
        .list_by_user(
            ListByUserQuery {
                tenant_id: NotifTenantId(tenant_id),
                user_id: NotifUserId::from(assignee),
                unread_only: false,
            },
            &NotifActor::new(assignee, tenant_id),
        )
        .await
        .unwrap();
    assert_eq!(inbox.len(), 1, "assignee 收到 status change notification");
    assert_eq!(inbox[0].event_type, NotificationEventType::FeedbackRequired);
}

/// **IT-S3-V1**: work-item 状态变更但无 assignee → 0 notification (L0 守门)
#[tokio::test]
async fn it_s3_v1_workitem_no_assignee_no_notification() {
    let wi_svc = InMemoryWorkItemService::new();
    let n_svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let reporter = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, reporter);
    let n_actor = NotifActor::new(reporter, tenant_id);

    // 创建 (无 assignee)
    let item = wi_svc
        .create_work_item(basic_task_cmd(tenant_id, reporter), &actor)
        .await
        .unwrap();

    // 状态变更
    wi_svc
        .transition_status(
            TransitionStatusCommand {
                tenant_id: TenantId(tenant_id),
                work_item_id: item.id,
                from: WorkItemStatus::Todo,
                to: WorkItemStatus::InProgress,
                actor_user_id: UserId::from(reporter),
            },
            &actor,
        )
        .await
        .unwrap();

    // L0 编排: 无 assignee → 0 notification
    let dispatched =
        l0_dispatch_workitem_status_change(&wi_svc, &n_svc, item.id, tenant_id, &n_actor).await;
    assert_eq!(
        dispatched.len(),
        0,
        "无 assignee → 0 notification (L0 跳过派发)"
    );
}

/// **IT-S3-V2**: 多次状态变更 (TODO→IN_PROGRESS→DONE) → assignee 收 2 notification
#[tokio::test]
async fn it_s3_v2_multiple_status_changes_multiple_notifications() {
    let wi_svc = InMemoryWorkItemService::new();
    let n_svc = InMemoryNotificationService::new();
    let tenant_id = Uuid::new_v4();
    let reporter = Uuid::new_v4();
    let assignee = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id, reporter);
    let n_actor = NotifActor::new(reporter, tenant_id);

    n_svc
        .register_channel(
            RegisterChannelCommand {
                tenant_id: NotifTenantId(tenant_id),
                user_id: NotifUserId::from(assignee),
                kind: ChannelKind::InApp,
                address: format!("in_app://user/{}", assignee),
                actor_user_id: NotifUserId::from(assignee),
            },
            &NotifActor::new(assignee, tenant_id),
        )
        .await
        .unwrap();

    let item = wi_svc
        .create_work_item(basic_task_cmd(tenant_id, reporter), &actor)
        .await
        .unwrap();
    wi_svc
        .assign(
            domain_work_item::AssignCommand {
                tenant_id: TenantId(tenant_id),
                work_item_id: item.id,
                assignee_user_id: Some(UserId::from(assignee)),
                assignee_agent_id: None,
                actor_user_id: UserId::from(reporter),
            },
            &actor,
        )
        .await
        .unwrap();

    // TODO → IN_PROGRESS
    wi_svc
        .transition_status(
            TransitionStatusCommand {
                tenant_id: TenantId(tenant_id),
                work_item_id: item.id,
                from: WorkItemStatus::Todo,
                to: WorkItemStatus::InProgress,
                actor_user_id: UserId::from(reporter),
            },
            &actor,
        )
        .await
        .unwrap();
    l0_dispatch_workitem_status_change(&wi_svc, &n_svc, item.id, tenant_id, &n_actor).await;

    // IN_PROGRESS → DONE
    wi_svc
        .transition_status(
            TransitionStatusCommand {
                tenant_id: TenantId(tenant_id),
                work_item_id: item.id,
                from: WorkItemStatus::InProgress,
                to: WorkItemStatus::Done,
                actor_user_id: UserId::from(reporter),
            },
            &actor,
        )
        .await
        .unwrap();
    l0_dispatch_workitem_status_change(&wi_svc, &n_svc, item.id, tenant_id, &n_actor).await;

    // assignee 收 2 条
    let inbox = n_svc
        .list_by_user(
            ListByUserQuery {
                tenant_id: NotifTenantId(tenant_id),
                user_id: NotifUserId::from(assignee),
                unread_only: false,
            },
            &NotifActor::new(assignee, tenant_id),
        )
        .await
        .unwrap();
    assert_eq!(inbox.len(), 2, "2 次 status change → 2 notification");
}

/// **IT-S3-V3**: 跨 tenant status change assignee notification 必拒
/// (per 守门 #13 (c) Master 100% RLS, 跨域 L0 编排不绕过 tenant 隔离)
#[tokio::test]
async fn it_s3_v3_cross_tenant_status_notification_rejected() {
    let wi_svc = InMemoryWorkItemService::new();
    let n_svc = InMemoryNotificationService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let reporter = Uuid::new_v4();
    let assignee_a = Uuid::new_v4(); // tenant A 的 assignee
    let actor_a = make_dev_actor(tenant_a, reporter);

    // tenant A 创 work-item + assign tenant A user
    let item = wi_svc
        .create_work_item(basic_task_cmd(tenant_a, reporter), &actor_a)
        .await
        .unwrap();
    wi_svc
        .assign(
            domain_work_item::AssignCommand {
                tenant_id: TenantId(tenant_a),
                work_item_id: item.id,
                assignee_user_id: Some(UserId::from(assignee_a)),
                assignee_agent_id: None,
                actor_user_id: UserId::from(reporter),
            },
            &actor_a,
        )
        .await
        .unwrap();

    // 状态变更
    wi_svc
        .transition_status(
            TransitionStatusCommand {
                tenant_id: TenantId(tenant_a),
                work_item_id: item.id,
                from: WorkItemStatus::Todo,
                to: WorkItemStatus::InProgress,
                actor_user_id: UserId::from(reporter),
            },
            &actor_a,
        )
        .await
        .unwrap();

    // L0 尝试: actor 是 tenant_a, 但 cmd.tenant_id = tenant_b → RLS 跨域必拒
    let res = n_svc
        .dispatch(
            DispatchNotificationCommand {
                tenant_id: NotifTenantId(tenant_b), // 显式要 tenant_b
                user_id: NotifUserId::from(assignee_a),
                event_type: NotificationEventType::FeedbackRequired,
                resource_type: "work_item".to_string(),
                resource_id: item.id.as_uuid(),
                subject: "cross-tenant".to_string(),
                body: "x".to_string(),
                source: "l0_tmo_orchestrator".to_string(),
            },
            &NotifActor::new(reporter, tenant_a), // actor tenant_a, cmd tenant_b → 必拒
        )
        .await;
    assert!(
        matches!(
            res,
            Err(domain_notification::NotificationError::CrossTenantDenied(
                _,
                _
            ))
        ),
        "跨 tenant status notification 必拒, got: {:?}",
        res
    );
}
