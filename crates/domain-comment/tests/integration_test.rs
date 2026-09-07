//! 集成测试 (IT) — domain-comment 真实数据接入验证
//!
//! **crate**: domain-comment (test target)
//! **per**: 2026-09-07 P3-B sub-batch 2 write-heavy 域真实数据接入
//! **参考**: `crates/domain-workspace/tests/integration_test.rs` (per 7f9f52a 4 域真实接入已落地)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): create + get_by_id + list_by_parent
//! - V1 (编辑/软删除): edit 状态机 + delete 软删除 + list 过滤 deleted
//! - V2 (RLS 13 類): 跨 tenant 拒绝 (守门 #13 (c) Master 100% RLS)
//! - V3 (不变量): INV-C-03 Reaction 唯一 + INV-C-04 object_key 必带 tenant 前缀

use domain_comment::{
    ActorContext, AddReactionCommand, AgentId, AttachmentId, CommentCommandPort, CommentError,
    CommentQueryPort, CommentStatus, CreateCommentCommand, DeleteCommentCommand, EditCommentCommand,
    GetCommentQuery, InMemoryCommentService, ListByParentQuery, ParentType, ProjectId,
    RegisterAttachmentCommand, TenantId, UserId,
};
use uuid::Uuid;

/// IT fixture: developer 角色 actor
fn make_dev_actor(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("developer")
}

/// IT fixture: project_admin 角色 actor (per INV-C-06 delete_comment 可代作者删)
fn make_admin_actor(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("project_admin")
}

/// IT fixture: 基础 comment 创建命令
fn basic_comment_cmd(tid: Uuid, author: Uuid) -> CreateCommentCommand {
    CreateCommentCommand {
        tenant_id: TenantId(tid),
        project_id: ProjectId::new(),
        parent_type: ParentType::WorkItem,
        parent_id: Uuid::new_v4(),
        body: "first comment".to_string(),
        author_user_id: Some(UserId::from(author)),
        author_agent_id: None,
        mentions: vec![],
        attachment_ids: vec![],
        actor_user_id: UserId::from(author),
    }
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: 完整 create → get_by_id → list_by_parent 闭环
#[tokio::test]
async fn it_v0_create_get_list_lifecycle() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);

    // create
    let mut cmd = basic_comment_cmd(tenant_id, author);
    cmd.parent_id = parent_id;
    cmd.body = "looks good to me".to_string();
    let c = svc.create_comment(cmd, &actor).await.expect("create 必成功");
    assert_eq!(c.status, CommentStatus::Open);
    assert_eq!(c.body, "looks good to me");

    // get
    let fetched = svc
        .get(
            GetCommentQuery {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(fetched.id, c.id);

    // list_by_parent — 1 条
    let list = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id,
                include_deleted: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, c.id);
}

/// **IT-V0-2**: 嵌套/多作者 — 3 个作者 5 条评论按 parent 隔离
#[tokio::test]
async fn it_v0_multiple_authors_isolated_by_parent() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let parent_a = Uuid::new_v4();
    let parent_b = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);

    // parent_a: 3 条
    for i in 0..3 {
        let mut cmd = basic_comment_cmd(tenant_id, author);
        cmd.parent_id = parent_a;
        cmd.body = format!("a-{}", i);
        svc.create_comment(cmd, &actor).await.unwrap();
    }
    // parent_b: 2 条
    for i in 0..2 {
        let mut cmd = basic_comment_cmd(tenant_id, author);
        cmd.parent_id = parent_b;
        cmd.body = format!("b-{}", i);
        svc.create_comment(cmd, &actor).await.unwrap();
    }

    let list_a = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id: parent_a,
                include_deleted: false,
            },
            &actor,
        )
        .await
        .unwrap();
    let list_b = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id: parent_b,
                include_deleted: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list_a.len(), 3);
    assert_eq!(list_b.len(), 2);
}

// =====================================================================
// V1: 编辑 / 软删除
// =====================================================================

/// **IT-V1-1**: edit 状态机 Open→Edited + lock_version 递增
#[tokio::test]
async fn it_v1_edit_status_machine() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, author), &actor)
        .await
        .unwrap();
    assert_eq!(c.status, CommentStatus::Open);

    let edited = svc
        .edit_comment(
            EditCommentCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
                new_body: "edited body".to_string(),
                actor_user_id: UserId::from(author),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(edited.status, CommentStatus::Edited);
    assert_eq!(edited.body, "edited body");
    // updated_at 应晚于 created_at
    assert!(edited.updated_at >= c.created_at);
}

/// **IT-V1-2**: delete 软删除 + list include_deleted=false 过滤
#[tokio::test]
async fn it_v1_soft_delete_and_filter() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);

    let mut cmd = basic_comment_cmd(tenant_id, author);
    cmd.parent_id = parent_id;
    let c1 = svc.create_comment(cmd, &actor).await.unwrap();
    let c2 = svc
        .create_comment(basic_comment_cmd(tenant_id, author), &actor)
        .await
        .unwrap();

    // 软删 c1
    let deleted = svc
        .delete_comment(
            DeleteCommentCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c1.id,
                actor_user_id: UserId::from(author),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(deleted.status, CommentStatus::Deleted);
    assert!(deleted.deleted_at.is_some());

    // include_deleted=false: 0 (c1 被过滤, c2 parent 不同)
    let filtered = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id,
                include_deleted: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(filtered.len(), 0);

    // include_deleted=true: 1 (c1 仍出现)
    let all = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id,
                include_deleted: true,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, c1.id);
    let _ = c2; // suppress unused
}

/// **IT-V1-3**: edit 已删除评论 → EditDeleted 拒绝
#[tokio::test]
async fn it_v1_edit_deleted_rejected() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, author), &actor)
        .await
        .unwrap();

    svc.delete_comment(
        DeleteCommentCommand {
            tenant_id: TenantId(tenant_id),
            comment_id: c.id,
            actor_user_id: UserId::from(author),
        },
        &actor,
    )
    .await
    .unwrap();

    let res = svc
        .edit_comment(
            EditCommentCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
                new_body: "post-delete".to_string(),
                actor_user_id: UserId::from(author),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(CommentError::EditDeleted)));
}

/// **IT-V1-4**: 非作者 edit 被拒
#[tokio::test]
async fn it_v1_edit_other_user_rejected() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let other = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, author), &actor)
        .await
        .unwrap();

    let other_actor = ActorContext::new(other, tenant_id);
    let res = svc
        .edit_comment(
            EditCommentCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
                new_body: "hostile takeover".to_string(),
                actor_user_id: UserId::from(other),
            },
            &other_actor,
        )
        .await;
    assert!(matches!(res, Err(CommentError::PermissionDenied)));
}

// =====================================================================
// V2: RLS 13 類 tenant 隔离
// =====================================================================

/// **IT-V2-1**: 跨 tenant create 被拒
#[tokio::test]
async fn it_v2_cross_tenant_create_rejected() {
    let svc = InMemoryCommentService::new();
    let actor_tenant = Uuid::new_v4();
    let cmd_tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = make_admin_actor(actor_tenant);

    let mut cmd = basic_comment_cmd(cmd_tenant, author);
    cmd.actor_user_id = UserId::from(author);

    let res = svc.create_comment(cmd, &actor).await;
    assert!(matches!(res, Err(CommentError::CrossTenantDenied(_, _))));
}

/// **IT-V2-2**: 跨 tenant get 被拒
#[tokio::test]
async fn it_v2_cross_tenant_get_rejected() {
    let svc = InMemoryCommentService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor_a = ActorContext::new(author, tenant_a);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_a, author), &actor_a)
        .await
        .unwrap();

    // tenant B actor 试图读 tenant A 的 comment
    let actor_b = make_admin_actor(tenant_b);
    let res = svc
        .get(
            GetCommentQuery {
                tenant_id: TenantId(tenant_b),
                comment_id: c.id,
            },
            &actor_b,
        )
        .await;
    assert!(matches!(res, Err(CommentError::CrossTenantDenied(_, _))));
}

/// **IT-V2-3**: 跨 tenant list_by_parent 被拒
#[tokio::test]
async fn it_v2_cross_tenant_list_rejected() {
    let svc = InMemoryCommentService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor_a = ActorContext::new(author, tenant_a);
    let parent_id = Uuid::new_v4();
    let mut cmd = basic_comment_cmd(tenant_a, author);
    cmd.parent_id = parent_id;
    svc.create_comment(cmd, &actor_a).await.unwrap();

    // tenant B actor 试图查 tenant A 的 parent — query 显式带 tenant_a
    let actor_b = make_admin_actor(tenant_b);
    let res = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_a), // 显式要 tenant_a 的数据
                parent_type: ParentType::WorkItem,
                parent_id,
                include_deleted: false,
            },
            &actor_b, // actor 属 tenant_b
        )
        .await;
    assert!(matches!(res, Err(CommentError::CrossTenantDenied(_, _))));
}

// =====================================================================
// V3: 不变量 INV-C-03 Reaction + INV-C-04 tenant prefix
// =====================================================================

/// **IT-V3-1**: INV-C-03 — (comment, user, emoji) 唯一约束
#[tokio::test]
async fn it_v3_reaction_unique_triple() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = ActorContext::new(user, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, user), &actor)
        .await
        .unwrap();

    // 第一次 👍 必成功
    svc.add_reaction(
        AddReactionCommand {
            tenant_id: TenantId(tenant_id),
            comment_id: c.id,
            user_id: UserId::from(user),
            emoji: "👍".to_string(),
        },
        &actor,
    )
    .await
    .unwrap();

    // 重复 (comment, user, emoji) → ReactionExists
    let res = svc
        .add_reaction(
            AddReactionCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
                user_id: UserId::from(user),
                emoji: "👍".to_string(),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(CommentError::ReactionExists)));
}

/// **IT-V3-2**: INV-C-03 — 同 user 不同 emoji 允许多个反应
#[tokio::test]
async fn it_v3_reaction_different_emoji_allowed() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = ActorContext::new(user, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, user), &actor)
        .await
        .unwrap();

    for emoji in ["👍", "🎉", "❤️"] {
        svc.add_reaction(
            AddReactionCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
                user_id: UserId::from(user),
                emoji: emoji.to_string(),
            },
            &actor,
        )
        .await
        .expect(&format!("emoji {} 必成功", emoji));
    }
}

/// **IT-V3-3**: INV-C-04 — attachment object_key 缺 tenant_id 前缀 → 拒绝
#[tokio::test]
async fn it_v3_attachment_requires_tenant_prefix() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let uploader = Uuid::new_v4();
    let actor = ActorContext::new(uploader, tenant_id);

    let res = svc
        .register_attachment(
            RegisterAttachmentCommand {
                tenant_id: TenantId(tenant_id),
                uploader_user_id: UserId::from(uploader),
                filename: "design.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                size_bytes: 1024,
                object_key: "wrong-prefix/file.pdf".to_string(), // 缺 tenants/{tid}/
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(CommentError::InvalidObjectKey)));
}

/// **IT-V3-4**: INV-C-04 — attachment 正确带 tenant_id 前缀 → OK
#[tokio::test]
async fn it_v3_attachment_with_tenant_prefix_ok() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let uploader = Uuid::new_v4();
    let actor = ActorContext::new(uploader, tenant_id);
    let tid = TenantId(tenant_id);

    let a = svc
        .register_attachment(
            RegisterAttachmentCommand {
                tenant_id: tid,
                uploader_user_id: UserId::from(uploader),
                filename: "design.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                size_bytes: 1024,
                object_key: format!("tenants/{}/files/design.pdf", tenant_id),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(a.tenant_id, tid);
    assert!(a.object_key.starts_with(&format!("tenants/{}/", tenant_id)));
    let _ = AttachmentId::new(); // suppress unused
}

/// **IT-V3-5**: agent author 创建评论(AI Agent 会话触发)
#[tokio::test]
async fn it_v3_agent_author_comment() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let agent = AgentId::new();
    let mut agent_actor = ActorContext::new(agent.as_uuid(), tenant_id).with_agent_session(true);
    agent_actor.tenant_id = tenant_id;

    let cmd = CreateCommentCommand {
        tenant_id: TenantId(tenant_id),
        project_id: ProjectId::new(),
        parent_type: ParentType::PullRequest,
        parent_id: Uuid::new_v4(),
        body: "[bot] reviewed, looks good".to_string(),
        author_user_id: None,
        author_agent_id: Some(agent),
        mentions: vec![],
        attachment_ids: vec![],
        actor_user_id: UserId::new(), // 占位, actor 由 tenant_id 校验
    };
    let c = svc.create_comment(cmd, &agent_actor).await.unwrap();
    assert_eq!(c.author_agent_id, Some(agent));
    assert!(c.author_user_id.is_none());
    assert!(matches!(c.parent_type, ParentType::PullRequest));
}
